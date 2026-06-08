// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2024–2026 ecoPrimals Project

//! `RhizoCrypt` main implementation.
//!
//! The core DAG engine with lock-free concurrency for maximum performance.
//!
//! The dehydration pipeline (summary generation, attestation collection,
//! permanent storage commit) lives in the `dehydration_ops` submodule.

mod branch_ops;
mod dehydration_ops;
mod lifecycle;

use crate::config::RhizoCryptConfig;
use crate::dehydration;
use crate::discovery::DiscoveryRegistry;
use crate::error::{Result, RhizoCryptError};
use crate::event::EventType;
use crate::merkle::{MerkleProof, MerkleRoot};
use crate::metrics::PrimalMetrics;
use crate::primal::PrimalState;
#[cfg(test)]
use crate::primal::{PrimalHealth, PrimalLifecycle};
use crate::session::Session;
use crate::slice::{self, ResolutionOutcome, Slice};
use crate::store::{DagBackend, DagStore, InMemoryPayloadStore};
use crate::types::{Did, SessionId, SliceId, Timestamp, VertexId};
use crate::vertex::Vertex;

use dashmap::DashMap;
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::{OnceCell, RwLock};

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
    // Capability-based discovery registry shared across dehydration & integration
    discovery_registry: Arc<DiscoveryRegistry>,
    // Lazily-resolved signing client (discovered from registry on first use)
    signing_client: OnceCell<Option<crate::clients::SigningClient>>,
    // Provenance notifier (optional, non-fatal)
    provenance_notifier: Arc<crate::types_ecosystem::provenance::ProvenanceNotifier>,
    // Cross-gate mesh event listener (optional, non-fatal)
    mesh_listener: Arc<crate::types_ecosystem::mesh::MeshEventListener>,
}

impl RhizoCrypt {
    /// Create a new `RhizoCrypt` instance.
    #[must_use]
    pub fn new(config: RhizoCryptConfig) -> Self {
        use crate::types_ecosystem::mesh::MeshEventListener;
        use crate::types_ecosystem::provenance::ProvenanceNotifier;

        let registry = Arc::new(DiscoveryRegistry::new(crate::constants::PRIMAL_NAME));

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
            discovery_registry: Arc::clone(&registry),
            signing_client: OnceCell::new(),
            provenance_notifier: Arc::new(ProvenanceNotifier::with_discovery(Arc::clone(
                &registry,
            ))),
            mesh_listener: Arc::new(MeshEventListener::new(registry)),
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

    /// Get the shared discovery registry.
    ///
    /// Callers can register endpoints to enable capability-based discovery
    /// for dehydration attestations, permanent storage, and provenance notifications.
    #[must_use]
    pub const fn discovery_registry(&self) -> &Arc<DiscoveryRegistry> {
        &self.discovery_registry
    }

    /// Get the cross-gate mesh event listener.
    ///
    /// Used to record trust establishment events (from bearDog or any
    /// signing provider) into the DAG.
    #[must_use]
    pub const fn mesh_listener(&self) -> &Arc<crate::types_ecosystem::mesh::MeshEventListener> {
        &self.mesh_listener
    }

    /// Get the lazily-resolved signing client.
    ///
    /// On first call, attempts to discover a signing provider via the
    /// capability registry. Returns `None` if no provider is available
    /// (standalone mode). The result is cached for subsequent calls.
    pub async fn signing_client(&self) -> Option<&crate::clients::SigningClient> {
        self.signing_client
            .get_or_init(|| async {
                match crate::clients::SigningClient::discover(&self.discovery_registry).await {
                    Ok(client) => {
                        tracing::info!("Signing provider discovered — vertices will be signed");
                        Some(client)
                    }
                    Err(e) => {
                        tracing::debug!(
                            error = %e,
                            "No signing provider available — vertices will be unsigned"
                        );
                        None
                    }
                }
            })
            .await
            .as_ref()
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
    // GC / TTL Sweeper
    // ========================================================================

    /// Sweep expired sessions based on `max_duration` from `SessionConfig`.
    ///
    /// Walks all sessions, identifies those whose `created_at + max_duration`
    /// has elapsed, discards them, and cleans up associated state. Returns
    /// the number of sessions reaped.
    pub async fn gc_sweep(&self) -> usize {
        let now = Timestamp::now();
        let expired: Vec<_> = self
            .sessions
            .iter()
            .filter(|entry| {
                let session = entry.value();
                !session.is_terminal()
                    && now.duration_since(session.created_at) >= session.config.max_duration
            })
            .map(|entry| *entry.key())
            .collect();

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

    /// Spawn a background mesh event poller that polls bearDog's
    /// `auth.events.poll` and appends trust events to a dedicated
    /// mesh-trust DAG session.
    ///
    /// Auto-provisions a `SessionType::Custom { domain: "mesh-trust" }`
    /// session on the first event received. Each polled event becomes a
    /// `Vertex` appended to that session.
    ///
    /// Returns a `JoinHandle` for cancellation. Non-fatal — poll failures
    /// are logged and retried.
    #[must_use]
    pub fn spawn_mesh_poller(self: &Arc<Self>) -> tokio::task::JoinHandle<()> {
        use crate::session::{SessionBuilder, SessionType};
        use crate::vertex::VertexBuilder;

        let primal = Arc::clone(self);
        let interval = crate::constants::MESH_POLL_INTERVAL;

        tokio::spawn(async move {
            let mut ticker = tokio::time::interval(interval);
            ticker.tick().await; // skip first immediate tick

            let mut mesh_session_id: Option<SessionId> = None;

            loop {
                ticker.tick().await;
                if !primal.state.is_running() {
                    tracing::debug!("Mesh poller exiting: primal no longer running");
                    break;
                }

                let count = match primal.mesh_listener.poll_events().await {
                    Ok(n) => n,
                    Err(e) => {
                        tracing::debug!(error = %e, "Mesh poll error (will retry)");
                        continue;
                    }
                };

                if count == 0 {
                    continue;
                }

                let events = primal.mesh_listener.drain_events().await;
                if events.is_empty() {
                    continue;
                }

                // Auto-provision mesh-trust session on first event
                if mesh_session_id.is_none() {
                    let session = SessionBuilder::new(SessionType::Custom {
                        domain: "mesh-trust".into(),
                    })
                    .with_name("mesh-trust-events")
                    .build();

                    match primal.create_session(session) {
                        Ok(id) => {
                            tracing::info!(
                                %id,
                                "Auto-provisioned mesh-trust session for cross-gate events"
                            );
                            mesh_session_id = Some(id);
                        }
                        Err(e) => {
                            tracing::warn!(
                                error = %e,
                                "Failed to create mesh-trust session (events buffered)"
                            );
                            continue;
                        }
                    }
                }

                let Some(session_id) = mesh_session_id else {
                    continue;
                };

                for event in events {
                    let event_type = event.into_event_type();
                    let vertex = VertexBuilder::new(event_type).build();

                    if let Err(e) = primal.append_vertex(session_id, vertex).await {
                        tracing::warn!(
                            error = %e,
                            "Failed to append mesh trust event to DAG"
                        );
                    }
                }

                tracing::info!(
                    count,
                    %session_id,
                    "Appended mesh trust events to DAG"
                );
            }
        })
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

#[cfg(test)]
#[path = "../rhizocrypt_tests.rs"]
mod tests;

#[cfg(test)]
#[path = "../rhizocrypt_tests_extended.rs"]
mod tests_extended;
