// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2024–2026 ecoPrimals Project

//! `RhizoCrypt` main implementation.
//!
//! The core DAG engine with lock-free concurrency for maximum performance.

use crate::config::{RhizoCryptConfig, StorageBackend};
use crate::dehydration::{self, DehydrationSummary};
use crate::discovery::DiscoveryRegistry;
use crate::error::{Result, RhizoCryptError};
use crate::event::{EventType, SessionOutcome};
use crate::merkle::{MerkleProof, MerkleRoot};
use crate::metrics::PrimalMetrics;
use crate::primal::{
    HealthReport, HealthStatus, PrimalError, PrimalHealth, PrimalLifecycle, PrimalState,
};
use crate::session::{CommitRef, Session};
use crate::slice::{self, ResolutionOutcome, Slice};
use crate::store::{DagBackend, DagStore, InMemoryDagStore, InMemoryPayloadStore};
use crate::types::{Did, SessionId, SliceId, Timestamp, VertexId};
use crate::vertex::Vertex;

use dashmap::DashMap;
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::RwLock;

/// The `RhizoCrypt` primal - Core DAG Engine.
///
/// Uses lock-free concurrent data structures (`DashMap`) for maximum concurrency.
/// Multiple operations on different sessions can proceed in parallel without blocking.
///
/// ## Architecture
///
/// - **Lock-free session storage**: `DashMap` for concurrent access
/// - **Lock-free slice storage**: `DashMap` for concurrent operations  
/// - **Lock-free dehydration tracking**: `DashMap` for status updates
/// - **Atomic metrics**: Lock-free counters for performance tracking
///
/// ## Performance
///
/// - Concurrent reads: Zero blocking
/// - Concurrent writes to different keys: Zero blocking
/// - Fine-grained locking: Only when mutating same key
/// - Expected improvement: 10-100x vs `RwLock<HashMap>`
pub struct RhizoCrypt {
    config: RhizoCryptConfig,
    state: PrimalState,
    started_at: Option<Instant>,
    // Storage backends (initialized once at startup, dispatched via DagBackend)
    dag_store: Arc<RwLock<Option<DagBackend>>>,
    payload_store: Arc<RwLock<Option<InMemoryPayloadStore>>>,
    // Lock-free concurrent maps for session data
    sessions: Arc<DashMap<SessionId, Session>>,
    slices: Arc<DashMap<SliceId, Slice>>,
    dehydration_status: Arc<DashMap<SessionId, dehydration::DehydrationStatus>>,
    // O(1) vertex → session lookup (populated on append, cleaned on discard)
    vertex_session_index: Arc<DashMap<VertexId, SessionId>>,
    // Atomic metrics (lock-free)
    metrics: Arc<PrimalMetrics>,
    // Provenance notifier (optional, non-fatal)
    provenance_notifier: Arc<crate::types_ecosystem::provenance::ProvenanceNotifier>,
}

impl RhizoCrypt {
    /// Create a new `RhizoCrypt` instance.
    #[must_use]
    pub fn new(config: RhizoCryptConfig) -> Self {
        use crate::types_ecosystem::provenance::{ProvenanceNotifier, ProvenanceProviderConfig};

        Self {
            config,
            state: PrimalState::Created,
            started_at: None,
            dag_store: Arc::new(RwLock::new(None)),
            payload_store: Arc::new(RwLock::new(None)),
            sessions: Arc::new(DashMap::new()),
            slices: Arc::new(DashMap::new()),
            dehydration_status: Arc::new(DashMap::new()),
            vertex_session_index: Arc::new(DashMap::new()),
            metrics: Arc::new(PrimalMetrics::new()),
            provenance_notifier: Arc::new(ProvenanceNotifier::new(
                ProvenanceProviderConfig::from_env(),
            )),
        }
    }

    /// Get the metrics.
    #[inline]
    #[must_use]
    pub fn metrics(&self) -> &PrimalMetrics {
        &self.metrics
    }

    /// Get the configuration.
    #[must_use]
    pub const fn config(&self) -> &RhizoCryptConfig {
        &self.config
    }

    /// Get the DAG store (if running).
    ///
    /// # Errors
    ///
    /// Returns an error if the primal is not running.
    pub async fn dag_store(&self) -> Result<DagBackend> {
        let store = self.dag_store.read().await;
        store.clone().ok_or_else(|| RhizoCryptError::internal("primal not running"))
    }

    /// Get the payload store (if running).
    ///
    /// # Errors
    ///
    /// Returns an error if the primal is not running.
    pub async fn payload_store(&self) -> Result<InMemoryPayloadStore> {
        let store = self.payload_store.read().await;
        store.clone().ok_or_else(|| RhizoCryptError::internal("primal not running"))
    }

    /// Get uptime in seconds.
    #[must_use]
    pub fn uptime_secs(&self) -> Option<u64> {
        self.started_at.map(|s| s.elapsed().as_secs())
    }

    /// Look up which session owns a vertex (O(1) via index).
    #[must_use]
    pub fn session_for_vertex(&self, vertex_id: VertexId) -> Option<SessionId> {
        self.vertex_session_index.get(&vertex_id).map(|e| *e.value())
    }

    // ========================================================================
    // Session Operations
    // ========================================================================

    /// Create a new session (lock-free).
    ///
    /// # Errors
    ///
    /// Returns an error if the primal is not running or max sessions exceeded.
    pub fn create_session(&self, session: Session) -> Result<SessionId> {
        if !self.state.is_running() {
            return Err(RhizoCryptError::internal("primal not running"));
        }

        if self.sessions.len() >= self.config.max_sessions {
            return Err(RhizoCryptError::internal("max sessions exceeded"));
        }

        let session_id = session.id;
        self.sessions.insert(session_id, session);
        self.metrics.inc_sessions_created();
        Ok(session_id)
    }

    /// Get a session by ID (lock-free).
    ///
    /// # Errors
    ///
    /// Returns an error if the session is not found.
    pub fn get_session(&self, session_id: SessionId) -> Result<Session> {
        self.sessions
            .get(&session_id)
            .map(|entry| entry.value().clone())
            .ok_or_else(|| RhizoCryptError::session_not_found(session_id))
    }

    /// List all sessions (lock-free iterator).
    #[must_use]
    pub fn list_sessions(&self) -> Vec<Session> {
        self.sessions.iter().map(|entry| entry.value().clone()).collect()
    }

    /// Discard a session (lock-free removal).
    ///
    /// # Errors
    ///
    /// Returns an error if the session is not found or primal not running.
    pub async fn discard_session(&self, session_id: SessionId) -> Result<()> {
        if !self.state.is_running() {
            return Err(RhizoCryptError::internal("primal not running"));
        }

        if self.sessions.remove(&session_id).is_none() {
            return Err(RhizoCryptError::session_not_found(session_id));
        }

        let dag_store = self.dag_store().await?;
        dag_store.delete_session(session_id).await?;
        self.purge_session_artifacts(session_id);

        Ok(())
    }

    /// Remove all secondary state associated with a session.
    ///
    /// Cleans up slices, dehydration status, and the vertex→session index.
    /// Idempotent — safe to call even if some artifacts are already gone.
    fn purge_session_artifacts(&self, session_id: SessionId) {
        self.slices.retain(|_, v| v.session_id != session_id);
        self.dehydration_status.remove(&session_id);
        self.vertex_session_index.retain(|_, sid| *sid != session_id);
    }

    /// Get session count (lock-free).
    #[must_use]
    pub fn session_count(&self) -> usize {
        self.sessions.len()
    }

    /// Get total vertex count across all sessions (lock-free).
    #[must_use]
    pub fn total_vertex_count(&self) -> u64 {
        self.sessions.iter().map(|entry| entry.value().vertex_count).sum()
    }

    // ========================================================================
    // Vertex Operations
    // ========================================================================

    /// Append a vertex to a session (fine-grained locking).
    ///
    /// # Errors
    ///
    /// Returns an error if the session is not found or not active.
    pub async fn append_vertex(&self, session_id: SessionId, vertex: Vertex) -> Result<VertexId> {
        if !self.state.is_running() {
            return Err(RhizoCryptError::internal("primal not running"));
        }

        // Fine-grained lock: only lock this specific session
        let mut session_entry = self
            .sessions
            .get_mut(&session_id)
            .ok_or_else(|| RhizoCryptError::session_not_found(session_id))?;

        let session = session_entry.value_mut();

        if !session.is_active() {
            return Err(RhizoCryptError::internal("session not active"));
        }

        // Track the agent
        if let Some(ref agent) = vertex.agent {
            session.add_agent(agent.clone());
        }

        // Compute ID once, update frontier, then release session lock
        let parents = vertex.parents.clone();
        let mut vertex = vertex;
        let vertex_id = vertex.id()?;
        session.update_frontier(vertex_id, &parents);

        // Release session lock before expensive DAG operation
        drop(session_entry);

        // Store the vertex and index it for O(1) lookup
        let dag_store = self.dag_store().await?;
        dag_store.put_vertex(session_id, vertex).await?;
        self.vertex_session_index.insert(vertex_id, session_id);
        self.metrics.inc_vertices_appended();

        Ok(vertex_id)
    }

    /// Get a vertex by ID.
    ///
    /// # Errors
    ///
    /// Returns an error if the vertex is not found.
    pub async fn get_vertex(&self, session_id: SessionId, vertex_id: VertexId) -> Result<Vertex> {
        let dag_store = self.dag_store().await?;
        dag_store
            .get_vertex(session_id, vertex_id)
            .await?
            .ok_or_else(|| RhizoCryptError::vertex_not_found(vertex_id))
    }

    /// Get all vertices for a session in topological order.
    ///
    /// # Errors
    ///
    /// Returns an error if primal not running.
    pub async fn get_all_vertices(&self, session_id: SessionId) -> Result<Vec<Vertex>> {
        let dag_store = self.dag_store().await?;
        dag_store.get_all_vertices(session_id).await
    }

    /// Query vertices with filters.
    ///
    /// # Errors
    ///
    /// Returns an error if primal not running.
    pub async fn query_vertices(
        &self,
        session_id: SessionId,
        event_types: Option<&[EventType]>,
        agent: Option<&Did>,
        limit: Option<usize>,
    ) -> Result<Vec<Vertex>> {
        let vertices = self.get_all_vertices(session_id).await?;

        let filtered: Vec<Vertex> = vertices
            .into_iter()
            .filter(|v| {
                // Filter by event type
                if let Some(types) = event_types
                    && !types.contains(&v.event_type)
                {
                    return false;
                }
                if let Some(a) = agent
                    && v.agent.as_ref() != Some(a)
                {
                    return false;
                }
                true
            })
            .take(limit.unwrap_or(usize::MAX))
            .collect();

        self.metrics.inc_queries_executed();
        Ok(filtered)
    }

    // ========================================================================
    // Merkle Operations
    // ========================================================================

    /// Compute Merkle root for a session.
    ///
    /// # Errors
    ///
    /// Returns an error if the session is not found or primal not running.
    pub async fn compute_merkle_root(&self, session_id: SessionId) -> Result<MerkleRoot> {
        let _ = self.get_session(session_id)?;
        let dag_store = self.dag_store().await?;
        let vertices = dag_store.get_all_vertices(session_id).await?;
        MerkleRoot::compute(&vertices)
    }

    /// Generate Merkle proof for a vertex.
    ///
    /// # Errors
    ///
    /// Returns an error if the session or vertex is not found.
    pub async fn generate_merkle_proof(
        &self,
        session_id: SessionId,
        vertex_id: VertexId,
    ) -> Result<MerkleProof> {
        let dag_store = self.dag_store().await?;
        let vertices = dag_store.get_all_vertices(session_id).await?;

        if vertices.is_empty() {
            return Err(RhizoCryptError::vertex_not_found(vertex_id));
        }

        let root = MerkleRoot::compute(&vertices)?;
        let ids: Vec<VertexId> =
            vertices.iter().map(Vertex::compute_id).collect::<std::result::Result<Vec<_>, _>>()?;
        let position = ids
            .iter()
            .position(|id| *id == vertex_id)
            .ok_or_else(|| RhizoCryptError::vertex_not_found(vertex_id))?;

        MerkleProof::generate(&vertices, position, root)
    }

    // ========================================================================
    // Slice Operations
    // ========================================================================

    /// Checkout a slice (lock-free).
    ///
    /// # Errors
    ///
    /// Returns an error if the primal is not running.
    pub fn checkout_slice(&self, slice: Slice) -> Result<SliceId> {
        if !self.state.is_running() {
            return Err(RhizoCryptError::internal("primal not running"));
        }

        let slice_id = slice.id;
        self.slices.insert(slice_id, slice);
        self.metrics.inc_slices_checked_out();
        Ok(slice_id)
    }

    /// Get a slice by ID (lock-free).
    ///
    /// # Errors
    ///
    /// Returns an error if the slice is not found.
    pub fn get_slice(&self, slice_id: SliceId) -> Result<Slice> {
        self.slices
            .get(&slice_id)
            .map(|entry| entry.value().clone())
            .ok_or_else(|| RhizoCryptError::SliceNotFound(slice_id.to_string()))
    }

    /// List all active slices (lock-free iterator).
    #[must_use]
    pub fn list_slices(&self) -> Vec<Slice> {
        self.slices
            .iter()
            .filter_map(|entry| {
                let slice = entry.value();
                if slice.is_active() {
                    Some(slice.clone())
                } else {
                    None
                }
            })
            .collect()
    }

    /// Resolve a slice (fine-grained locking).
    ///
    /// # Errors
    ///
    /// Returns an error if the slice is not found or already resolved.
    pub fn resolve_slice(&self, slice_id: SliceId, outcome: ResolutionOutcome) -> Result<()> {
        let mut slice_entry = self
            .slices
            .get_mut(&slice_id)
            .ok_or_else(|| RhizoCryptError::SliceNotFound(slice_id.to_string()))?;

        {
            let slice = slice_entry.value_mut();

            if slice.is_resolved() {
                return Err(RhizoCryptError::SliceAlreadyResolved(slice_id.to_string()));
            }

            slice.state = slice::SliceState::Resolved {
                outcome,
                resolved_at: Timestamp::now(),
            };
        }
        drop(slice_entry);
        Ok(())
    }

    // ========================================================================
    // Dehydration Operations
    // ========================================================================

    /// Start dehydration of a session with full implementation.
    ///
    /// This method:
    /// 1. Computes the Merkle root of the DAG
    /// 2. Generates a dehydration summary
    /// 3. Collects attestations from participants (if required)
    /// 4. Commits to permanent storage via `PermanentStorageProvider`
    ///
    /// # Errors
    ///
    /// Returns an error if session not found, dehydration fails, or commit fails.
    pub async fn dehydrate(&self, session_id: SessionId) -> Result<MerkleRoot> {
        // Set status to computing root
        self.dehydration_status.insert(session_id, dehydration::DehydrationStatus::ComputingRoot);

        // Compute merkle root
        let root = self.compute_merkle_root(session_id).await?;

        // Set status to generating summary
        self.dehydration_status
            .insert(session_id, dehydration::DehydrationStatus::GeneratingSummary);

        // Generate the dehydration summary
        let summary = self.generate_dehydration_summary(session_id, root).await?;

        let config = dehydration::DehydrationConfig::default();
        let attestations = if config.required_attestations.is_empty() {
            Vec::new()
        } else {
            self.collect_attestations(session_id, &summary, &config).await
        };

        // Add attestations to summary
        let mut summary_with_attestations = summary;
        summary_with_attestations.attestations.extend(attestations);

        // Set status to committing
        self.dehydration_status.insert(session_id, dehydration::DehydrationStatus::Committing);

        // Commit to permanent storage
        let commit_ref = self.commit_to_permanent_storage(&summary_with_attestations).await?;

        // Notify provenance provider (non-fatal: dehydration succeeds regardless)
        self.provenance_notifier.notify_dehydration(&summary_with_attestations).await.ok();

        // Update status to complete
        self.dehydration_status.insert(
            session_id,
            dehydration::DehydrationStatus::Completed {
                commit_ref,
            },
        );

        self.metrics.inc_dehydrations_completed();
        Ok(root)
    }

    /// Generate a dehydration summary for a session.
    ///
    /// Walks the session DAG to extract:
    /// - Actual payload byte totals from the payload store
    /// - Frontier vertices as result entries with serialized event types
    /// - Per-agent summaries with roles and event counts from DAG vertices
    async fn generate_dehydration_summary(
        &self,
        session_id: SessionId,
        merkle_root: MerkleRoot,
    ) -> Result<DehydrationSummary> {
        let session = self.get_session(session_id)?;

        let payload_bytes = match self.payload_store().await {
            Ok(store) => u64::try_from(store.total_bytes().await).unwrap_or(u64::MAX),
            Err(_) => {
                session.vertex_count.saturating_mul(crate::constants::ESTIMATED_BYTES_PER_VERTEX)
            }
        };

        let mut results = Vec::new();
        for vertex_id in &session.frontier {
            if let Ok(vertex) = self.get_vertex(session_id, *vertex_id).await {
                let value = serde_json::to_value(&vertex.event_type).unwrap_or_default();
                let result = dehydration::ResultEntry {
                    result_type: vertex.event_type.name().to_string(),
                    key: vertex_id.to_string(),
                    value,
                    source_vertex: *vertex_id,
                    payload_ref: vertex.payload,
                };
                results.push(result);
            }
        }

        let agents = self.build_agent_summaries(session_id, &session).await;

        let session_type_str =
            serde_json::to_string(&session.session_type).unwrap_or_else(|_| "General".to_string());

        let mut summary = dehydration::DehydrationSummaryBuilder::new(
            session_id,
            session_type_str,
            session.created_at,
            merkle_root,
        )
        .with_outcome(SessionOutcome::Success)
        .with_vertex_count(session.vertex_count)
        .with_payload_bytes(payload_bytes);

        for result in results {
            summary = summary.with_result(result);
        }
        for agent in agents {
            summary = summary.with_agent(agent);
        }

        Ok(summary.build())
    }

    /// Build per-agent summaries by walking the session DAG for join/leave events.
    async fn build_agent_summaries(
        &self,
        session_id: SessionId,
        session: &Session,
    ) -> Vec<dehydration::AgentSummary> {
        use crate::event::{AgentRole, EventType};

        let all_vertices =
            self.query_vertices(session_id, None, None, None).await.unwrap_or_default();

        let mut agent_info: std::collections::HashMap<
            Did,
            (Option<Timestamp>, Option<Timestamp>, u64, AgentRole),
        > = std::collections::HashMap::new();

        for vertex in &all_vertices {
            if let Some(ref agent) = vertex.agent {
                let entry = agent_info.entry(agent.clone()).or_insert((
                    None,
                    None,
                    0,
                    AgentRole::Participant,
                ));
                entry.2 += 1;

                match &vertex.event_type {
                    EventType::AgentJoin {
                        role,
                    } => {
                        if entry.0.is_none() {
                            entry.0 = Some(vertex.timestamp);
                        }
                        entry.3 = role.clone();
                    }
                    EventType::AgentLeave {
                        ..
                    } => {
                        entry.1 = Some(vertex.timestamp);
                    }
                    _ => {}
                }
            }
        }

        for did in &session.agents {
            agent_info.entry(did.clone()).or_insert((
                Some(session.created_at),
                None,
                0,
                AgentRole::Participant,
            ));
        }

        agent_info
            .into_iter()
            .map(|(did, (joined, left, count, role))| {
                let role_str = match &role {
                    AgentRole::Owner => "owner",
                    AgentRole::Participant => "participant",
                    AgentRole::Observer => "observer",
                    AgentRole::Custom(s) => s.as_str(),
                };
                dehydration::AgentSummary {
                    agent: did,
                    joined_at: joined.unwrap_or(session.created_at),
                    left_at: left,
                    event_count: count,
                    role: role_str.to_string(),
                }
            })
            .collect()
    }

    /// Collect attestations from session participants via capability-based signing.
    ///
    /// Discovers a `SigningProvider` at runtime and requests attestations from
    /// each required attester. Returns whatever attestations could be collected
    /// within the configured timeout. If no signing provider is available,
    /// returns an empty set (attestations are optional for dehydration).
    async fn collect_attestations(
        &self,
        session_id: SessionId,
        summary: &DehydrationSummary,
        config: &dehydration::DehydrationConfig,
    ) -> Vec<dehydration::Attestation> {
        self.dehydration_status.insert(
            session_id,
            dehydration::DehydrationStatus::CollectingAttestations {
                collected: 0,
                required: config.required_attestations.len(),
            },
        );

        let summary_hash = summary.compute_hash();

        let registry = DiscoveryRegistry::new(crate::constants::PRIMAL_NAME);
        let signing_client = match crate::clients::SigningClient::discover(&registry).await {
            Ok(client) => client,
            Err(e) => {
                tracing::debug!(error = %e, "No signing provider available, skipping attestations");
                return Vec::new();
            }
        };

        let mut attestations = Vec::new();
        for attester in &config.required_attestations {
            match signing_client.sign(&summary_hash, attester).await {
                Ok(sig) => {
                    attestations.push(dehydration::Attestation {
                        attester: attester.clone(),
                        statement: dehydration::AttestationStatement::SessionSummary {
                            summary_hash,
                        },
                        signature: sig.into_bytes(),
                        attested_at: Timestamp::now(),
                        verified: true,
                    });
                    self.dehydration_status.insert(
                        session_id,
                        dehydration::DehydrationStatus::CollectingAttestations {
                            collected: attestations.len(),
                            required: config.required_attestations.len(),
                        },
                    );
                }
                Err(e) => {
                    tracing::warn!(
                        attester = %attester,
                        error = %e,
                        "Failed to collect attestation, continuing"
                    );
                }
            }
        }

        attestations
    }

    /// Commit dehydration summary to permanent storage.
    ///
    /// Uses capability-based discovery — any `PermanentStorageProvider` works.
    /// Falls back to a local reference when no provider is available,
    /// allowing dehydration to complete in standalone deployments.
    async fn commit_to_permanent_storage(&self, summary: &DehydrationSummary) -> Result<CommitRef> {
        use crate::clients::PermanentStorageClient;

        let registry = DiscoveryRegistry::new(crate::constants::PRIMAL_NAME);

        match PermanentStorageClient::discover(&registry).await {
            Ok(client) => {
                tracing::info!(
                    session_id = %summary.session_id,
                    merkle_root = %summary.merkle_root,
                    vertex_count = summary.vertex_count,
                    "Committing dehydration to permanent storage"
                );

                client.commit(summary).await.map_err(|e| {
                    RhizoCryptError::integration(format!(
                        "Failed to commit to permanent storage: {e}"
                    ))
                })
            }
            Err(e) => {
                tracing::warn!(
                    session_id = %summary.session_id,
                    error = %e,
                    "No permanent storage provider available, creating local reference"
                );

                Ok(CommitRef {
                    spine_id: format!("local-{}", summary.session_id),
                    entry_hash: *summary.merkle_root.as_bytes(),
                    index: 0,
                })
            }
        }
    }

    /// Get dehydration status for a session (lock-free).
    #[must_use]
    pub fn get_dehydration_status(&self, session_id: SessionId) -> dehydration::DehydrationStatus {
        self.dehydration_status
            .get(&session_id)
            .map_or(dehydration::DehydrationStatus::Pending, |entry| entry.value().clone())
    }

    // ========================================================================
    // GC / TTL Sweeper
    // ========================================================================

    /// Sweep expired sessions based on `max_duration` from `SessionConfig`.
    ///
    /// Walks all sessions, identifies those whose `created_at + max_duration`
    /// has elapsed, discards them, and cleans up associated state. Returns
    /// the number of sessions reaped.
    pub async fn gc_sweep(&self) -> usize {
        let now = Timestamp::now();
        let mut expired = Vec::new();

        for entry in self.sessions.iter() {
            let session = entry.value();
            if session.is_terminal() {
                continue;
            }
            let age = now.duration_since(session.created_at);
            if age >= session.config.max_duration {
                expired.push(*entry.key());
            }
        }

        let count = expired.len();
        for session_id in expired {
            tracing::info!(%session_id, "GC sweep: expiring session past TTL");
            if let Some((_, mut session)) = self.sessions.remove(&session_id) {
                session.discard(crate::session::DiscardReason::Timeout);
            }
            self.purge_session_artifacts(session_id);
            if let Ok(dag_store) = self.dag_store().await {
                dag_store.delete_session(session_id).await.ok();
            }
        }

        if count > 0 {
            tracing::info!(reaped = count, "GC sweep complete");
        }
        count
    }

    /// Spawn a background GC task that runs periodically.
    ///
    /// Returns a `JoinHandle` that can be used to cancel the sweeper on
    /// shutdown. The interval is taken from `config.gc_interval`.
    #[must_use]
    pub fn spawn_gc_sweeper(self: &Arc<Self>) -> tokio::task::JoinHandle<()> {
        let primal = Arc::clone(self);
        let interval = primal.config.gc_interval;
        tokio::spawn(async move {
            let mut ticker = tokio::time::interval(interval);
            ticker.tick().await; // skip first immediate tick
            loop {
                ticker.tick().await;
                if !primal.state.is_running() {
                    tracing::debug!("GC sweeper exiting: primal no longer running");
                    break;
                }
                primal.gc_sweep().await;
            }
        })
    }
}

// ============================================================================
// Trait Implementations
// ============================================================================

impl PrimalLifecycle for RhizoCrypt {
    fn state(&self) -> PrimalState {
        self.state
    }

    async fn start(&mut self) -> std::result::Result<(), PrimalError> {
        if self.state != PrimalState::Created && self.state != PrimalState::Stopped {
            return Err(PrimalError::InvalidTransition {
                from: self.state,
                to: PrimalState::Starting,
            });
        }

        self.state = PrimalState::Starting;
        tracing::info!(primal = %self.config.name, backend = ?self.config.storage.backend, "starting");

        // Initialize DAG store based on configured backend
        {
            let backend = match &self.config.storage.backend {
                StorageBackend::Memory => {
                    tracing::info!("using in-memory DAG store");
                    DagBackend::Memory(InMemoryDagStore::new())
                }
                #[cfg(feature = "redb")]
                StorageBackend::Redb => {
                    let path = self.config.storage.path.as_deref().unwrap_or("rhizocrypt.redb");
                    tracing::info!(path = %path, "using redb DAG store");
                    let store = crate::store_redb::RedbDagStore::open(path).map_err(|e| {
                        PrimalError::StartupFailed(format!("redb open failed: {e}"))
                    })?;
                    DagBackend::Redb(store)
                }
                #[cfg(not(feature = "redb"))]
                StorageBackend::Redb => {
                    return Err(PrimalError::StartupFailed(
                        "Redb storage requested but 'redb' feature not enabled. \
                         Recompile with `--features redb`."
                            .to_string(),
                    ));
                }
            };
            let mut dag_store = self.dag_store.write().await;
            *dag_store = Some(backend);
        }
        {
            let mut payload_store = self.payload_store.write().await;
            *payload_store = Some(InMemoryPayloadStore::new());
        }

        self.started_at = Some(Instant::now());
        self.state = PrimalState::Running;
        tracing::info!(primal = %self.config.name, "running");

        Ok(())
    }

    async fn stop(&mut self) -> std::result::Result<(), PrimalError> {
        if self.state != PrimalState::Running {
            return Err(PrimalError::InvalidTransition {
                from: self.state,
                to: PrimalState::Stopping,
            });
        }

        self.state = PrimalState::Stopping;
        tracing::info!(primal = %self.config.name, "stopping");

        // Clean up stores
        {
            let mut dag_store = self.dag_store.write().await;
            *dag_store = None;
        }
        {
            let mut payload_store = self.payload_store.write().await;
            *payload_store = None;
        }

        self.vertex_session_index.clear();
        self.started_at = None;
        self.state = PrimalState::Stopped;
        tracing::info!(primal = %self.config.name, "stopped");

        Ok(())
    }
}

impl PrimalHealth for RhizoCrypt {
    fn health_status(&self) -> HealthStatus {
        if self.state.is_running() {
            HealthStatus::Healthy
        } else {
            HealthStatus::Unhealthy {
                reason: format!("state: {}", self.state),
            }
        }
    }

    async fn health_check(&self) -> std::result::Result<HealthReport, PrimalError> {
        let mut report = HealthReport::new(&self.config.name, env!("CARGO_PKG_VERSION"))
            .with_status(self.health_status());

        if let Some(uptime) = self.uptime_secs() {
            report = report.with_uptime(uptime);
        }

        Ok(report)
    }
}

#[cfg(test)]
#[path = "rhizocrypt_tests.rs"]
mod tests;
