//! # RhizoCrypt
//!
//! Core DAG Engine - Ephemeral Working Memory
//!
//! ## Overview
//!
//! RhizoCrypt is the ephemeral DAG engine of the ecoPrimals ecosystem. It provides
//! git-like functionality for capturing, linking, and eventually committing events
//! to the permanent LoamSpine layer.
//!
//! ## Key Concepts
//!
//! - **Vertex**: A single event in the DAG, content-addressed by Blake3 hash
//! - **Session**: A scoped DAG with lifecycle (create → grow → resolve → expire)
//! - **Dehydration**: The process of committing DAG results to LoamSpine
//! - **Slice**: A "checkout" of LoamSpine state into the DAG for async operations
//!
//! ## Quick Start
//!
//! ```rust
//! use rhizo_crypt_core::{PrimalLifecycle, RhizoCrypt, RhizoCryptConfig};
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let config = RhizoCryptConfig::default();
//! let mut primal = RhizoCrypt::new(config);
//! primal.start().await?;
//! # Ok(())
//! # }
//! ```
//!
//! ## Architecture
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────────┐
//! │                        RhizoCrypt                                │
//! │                     (Core DAG Engine)                            │
//! │                                                                  │
//! │  ┌─────────┐  ┌─────────┐  ┌─────────┐  ┌─────────────┐        │
//! │  │ Vertex  │  │  DAG    │  │ Merkle  │  │  Sessions   │        │
//! │  │ Store   │  │ Index   │  │ Trees   │  │  (scopes)   │        │
//! │  └─────────┘  └─────────┘  └─────────┘  └─────────────┘        │
//! └─────────────────────────────────────────────────────────────────┘
//! ```

#![forbid(unsafe_code)]
#![warn(missing_docs)]
#![warn(clippy::all)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![allow(clippy::module_name_repetitions)]
#![allow(clippy::must_use_candidate)]
#![allow(clippy::doc_markdown)]
// Nursery lint has known issues with async RwLock patterns
#![allow(clippy::significant_drop_tightening)]

pub mod clients;
pub mod config;
pub mod dehydration;
pub mod discovery;
pub mod error;
pub mod event;
pub mod integration;
pub mod merkle;
pub mod primal;
pub mod safe_env;
pub mod session;
pub mod slice;
pub mod store;
pub mod types;
pub mod vertex;

// Optional storage backends
#[cfg(feature = "sled")]
pub mod store_sled;

// Re-exports for convenience
pub use config::{
    DehydrationClientConfig, MetricsConfig, RhizoCryptConfig, RpcConfig, SliceConfig,
    StorageBackend, StorageConfig,
};
pub use dehydration::{
    Attestation, DehydrationConfig, DehydrationStatus, DehydrationSummary, ResultEntry,
};
pub use discovery::{
    Capability, ClientProvider, DiscoveryRegistry, DiscoveryStatus, ServiceEndpoint,
};
pub use error::{Result, RhizoCryptError};
pub use event::EventType;
pub use integration::{
    BearDogClient, ClientFactory, IntegrationStatus, LoamSpineClient, NestGateClient, ServiceStatus,
};
pub use safe_env::{CapabilityEnv, SafeEnv};

// Client modules
pub use clients::sweetgrass::{
    AgentContribution, ProvenanceChain, SessionAttribution, SweetGrassQueryable, VertexRef,
};
pub use clients::toadstool::{ComputeEvent, TaskId, ToadStoolClient, ToadStoolConfig};

// Test utilities - only available with test-utils feature or in tests
#[cfg(any(test, feature = "test-utils"))]
pub use integration::{MockBearDogClient, MockLoamSpineClient, MockNestGateClient};
pub use merkle::{MerkleProof, MerkleRoot};
pub use primal::{
    HealthReport, HealthStatus, PrimalError, PrimalHealth, PrimalLifecycle, PrimalState,
};
pub use session::{Session, SessionBuilder, SessionConfig, SessionState, SessionType};
pub use slice::{
    LoanTerms, ResolutionOutcome, ResolutionRoute, Slice, SliceBuilder, SliceConstraints,
    SliceMode, SliceOrigin, SliceState,
};
pub use store::{
    DagStore, InMemoryDagStore, InMemoryPayloadStore, PayloadStore, StorageHealth, StorageStats,
};
#[cfg(feature = "sled")]
pub use store_sled::SledDagStore;
pub use types::{ContentHash, Did, PayloadRef, SessionId, Signature, SliceId, Timestamp, VertexId};
pub use vertex::{MetadataValue, Vertex, VertexBuilder};

use hashbrown::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::RwLock;

/// The RhizoCrypt primal - Core DAG Engine.
pub struct RhizoCrypt {
    config: RhizoCryptConfig,
    state: PrimalState,
    started_at: Option<Instant>,
    dag_store: Arc<RwLock<Option<InMemoryDagStore>>>,
    payload_store: Arc<RwLock<Option<InMemoryPayloadStore>>>,
    sessions: Arc<RwLock<HashMap<SessionId, Session>>>,
    slices: Arc<RwLock<HashMap<SliceId, slice::Slice>>>,
    dehydration_status: Arc<RwLock<HashMap<SessionId, dehydration::DehydrationStatus>>>,
    // Metrics counters (atomic for lock-free updates)
    metrics: Arc<PrimalMetrics>,
}

/// Atomic metrics counters for the primal.
#[derive(Debug, Default)]
pub struct PrimalMetrics {
    /// Sessions created.
    pub sessions_created: AtomicU64,
    /// Sessions resolved/committed.
    pub sessions_resolved: AtomicU64,
    /// Vertices appended.
    pub vertices_appended: AtomicU64,
    /// Queries executed.
    pub queries_executed: AtomicU64,
    /// Slices checked out.
    pub slices_checked_out: AtomicU64,
    /// Dehydrations completed.
    pub dehydrations_completed: AtomicU64,
}

impl PrimalMetrics {
    /// Create new metrics.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Increment sessions created.
    #[inline]
    pub fn inc_sessions_created(&self) {
        self.sessions_created.fetch_add(1, Ordering::Relaxed);
    }

    /// Increment sessions resolved.
    #[inline]
    pub fn inc_sessions_resolved(&self) {
        self.sessions_resolved.fetch_add(1, Ordering::Relaxed);
    }

    /// Increment vertices appended.
    #[inline]
    pub fn inc_vertices_appended(&self) {
        self.vertices_appended.fetch_add(1, Ordering::Relaxed);
    }

    /// Increment queries executed.
    #[inline]
    pub fn inc_queries_executed(&self) {
        self.queries_executed.fetch_add(1, Ordering::Relaxed);
    }

    /// Increment slices checked out.
    #[inline]
    pub fn inc_slices_checked_out(&self) {
        self.slices_checked_out.fetch_add(1, Ordering::Relaxed);
    }

    /// Increment dehydrations completed.
    #[inline]
    pub fn inc_dehydrations_completed(&self) {
        self.dehydrations_completed.fetch_add(1, Ordering::Relaxed);
    }

    /// Get sessions created count.
    #[inline]
    #[must_use]
    pub fn get_sessions_created(&self) -> u64 {
        self.sessions_created.load(Ordering::Relaxed)
    }

    /// Get sessions resolved count.
    #[inline]
    #[must_use]
    pub fn get_sessions_resolved(&self) -> u64 {
        self.sessions_resolved.load(Ordering::Relaxed)
    }

    /// Get vertices appended count.
    #[inline]
    #[must_use]
    pub fn get_vertices_appended(&self) -> u64 {
        self.vertices_appended.load(Ordering::Relaxed)
    }

    /// Get queries executed count.
    #[inline]
    #[must_use]
    pub fn get_queries_executed(&self) -> u64 {
        self.queries_executed.load(Ordering::Relaxed)
    }

    /// Get slices checked out count.
    #[inline]
    #[must_use]
    pub fn get_slices_checked_out(&self) -> u64 {
        self.slices_checked_out.load(Ordering::Relaxed)
    }

    /// Get dehydrations completed count.
    #[inline]
    #[must_use]
    pub fn get_dehydrations_completed(&self) -> u64 {
        self.dehydrations_completed.load(Ordering::Relaxed)
    }
}

impl RhizoCrypt {
    /// Create a new RhizoCrypt instance.
    #[must_use]
    pub fn new(config: RhizoCryptConfig) -> Self {
        Self {
            config,
            state: PrimalState::Created,
            started_at: None,
            dag_store: Arc::new(RwLock::new(None)),
            payload_store: Arc::new(RwLock::new(None)),
            sessions: Arc::new(RwLock::new(HashMap::new())),
            slices: Arc::new(RwLock::new(HashMap::new())),
            dehydration_status: Arc::new(RwLock::new(HashMap::new())),
            metrics: Arc::new(PrimalMetrics::new()),
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
    pub async fn dag_store(&self) -> Result<InMemoryDagStore> {
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

    /// Create a new session.
    ///
    /// # Errors
    ///
    /// Returns an error if the primal is not running or max sessions exceeded.
    pub async fn create_session(&self, session: Session) -> Result<SessionId> {
        if !self.state.is_running() {
            return Err(RhizoCryptError::internal("primal not running"));
        }

        let mut sessions = self.sessions.write().await;
        if sessions.len() >= self.config.max_sessions {
            return Err(RhizoCryptError::internal("max sessions exceeded"));
        }

        let session_id = session.id;
        sessions.insert(session_id, session);
        self.metrics.inc_sessions_created();
        Ok(session_id)
    }

    /// Get a session by ID.
    ///
    /// # Errors
    ///
    /// Returns an error if the session is not found.
    pub async fn get_session(&self, session_id: SessionId) -> Result<Session> {
        let sessions = self.sessions.read().await;
        sessions
            .get(&session_id)
            .cloned()
            .ok_or_else(|| RhizoCryptError::session_not_found(session_id))
    }

    /// List all sessions.
    pub async fn list_sessions(&self) -> Vec<Session> {
        let sessions = self.sessions.read().await;
        sessions.values().cloned().collect()
    }

    /// Discard a session.
    ///
    /// # Errors
    ///
    /// Returns an error if the session is not found or primal not running.
    pub async fn discard_session(&self, session_id: SessionId) -> Result<()> {
        if !self.state.is_running() {
            return Err(RhizoCryptError::internal("primal not running"));
        }

        // Remove from session store
        let mut sessions = self.sessions.write().await;
        if sessions.remove(&session_id).is_none() {
            return Err(RhizoCryptError::session_not_found(session_id));
        }

        // Clean up DAG store
        let dag_store = self.dag_store().await?;
        dag_store.delete_session(session_id).await?;

        Ok(())
    }

    /// Append a vertex to a session.
    ///
    /// # Errors
    ///
    /// Returns an error if the session is not found or not active.
    pub async fn append_vertex(&self, session_id: SessionId, vertex: Vertex) -> Result<VertexId> {
        if !self.state.is_running() {
            return Err(RhizoCryptError::internal("primal not running"));
        }

        // Update session state
        {
            let mut sessions = self.sessions.write().await;
            let session = sessions
                .get_mut(&session_id)
                .ok_or_else(|| RhizoCryptError::session_not_found(session_id))?;

            if !session.is_active() {
                return Err(RhizoCryptError::internal("session not active"));
            }

            // Track the agent
            if let Some(ref agent) = vertex.agent {
                session.add_agent(agent.clone());
            }

            // Update frontier
            let parents: Vec<VertexId> = vertex.parents.clone();
            let mut v = vertex.clone();
            let vertex_id = v.id();
            session.update_frontier(vertex_id, &parents);
        }

        // Store the vertex
        let dag_store = self.dag_store().await?;
        let mut v = vertex;
        let vertex_id = v.id();
        dag_store.put_vertex(session_id, v).await?;
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

    /// Get session count.
    pub async fn session_count(&self) -> usize {
        self.sessions.read().await.len()
    }

    /// Get total vertex count across all sessions.
    pub async fn total_vertex_count(&self) -> u64 {
        let sessions = self.sessions.read().await;
        sessions.values().map(|s| s.vertex_count).sum()
    }

    /// Compute Merkle root for a session.
    ///
    /// # Errors
    ///
    /// Returns an error if the session is not found or primal not running.
    pub async fn compute_merkle_root(&self, session_id: SessionId) -> Result<merkle::MerkleRoot> {
        // Verify session exists
        let _ = self.get_session(session_id).await?;

        let dag_store = self.dag_store().await?;
        let vertices = dag_store.get_all_vertices(session_id).await?;

        Ok(merkle::MerkleRoot::compute(&vertices))
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
    ) -> Result<merkle::MerkleProof> {
        let dag_store = self.dag_store().await?;
        let vertices = dag_store.get_all_vertices(session_id).await?;

        if vertices.is_empty() {
            return Err(RhizoCryptError::vertex_not_found(vertex_id));
        }

        let root = merkle::MerkleRoot::compute(&vertices);

        // Find position of vertex
        let position = vertices
            .iter()
            .position(|v| v.compute_id() == vertex_id)
            .ok_or_else(|| RhizoCryptError::vertex_not_found(vertex_id))?;

        merkle::MerkleProof::generate(&vertices, position, root)
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
                if let Some(types) = event_types {
                    if !types.contains(&v.event_type) {
                        return false;
                    }
                }
                // Filter by agent
                if let Some(a) = agent {
                    if v.agent.as_ref() != Some(a) {
                        return false;
                    }
                }
                true
            })
            .take(limit.unwrap_or(usize::MAX))
            .collect();

        self.metrics.inc_queries_executed();
        Ok(filtered)
    }

    // ========================================================================
    // Slice Operations
    // ========================================================================

    /// Checkout a slice.
    ///
    /// # Errors
    ///
    /// Returns an error if the primal is not running.
    pub async fn checkout_slice(&self, slice: slice::Slice) -> Result<SliceId> {
        if !self.state.is_running() {
            return Err(RhizoCryptError::internal("primal not running"));
        }

        let slice_id = slice.id;
        self.slices.write().await.insert(slice_id, slice);
        self.metrics.inc_slices_checked_out();
        Ok(slice_id)
    }

    /// Get a slice by ID.
    ///
    /// # Errors
    ///
    /// Returns an error if the slice is not found.
    pub async fn get_slice(&self, slice_id: SliceId) -> Result<slice::Slice> {
        self.slices
            .read()
            .await
            .get(&slice_id)
            .cloned()
            .ok_or_else(|| RhizoCryptError::SliceNotFound(slice_id.to_string()))
    }

    /// List all active slices.
    pub async fn list_slices(&self) -> Vec<slice::Slice> {
        self.slices.read().await.values().filter(|s| s.is_active()).cloned().collect()
    }

    /// Resolve a slice.
    ///
    /// # Errors
    ///
    /// Returns an error if the slice is not found or already resolved.
    pub async fn resolve_slice(
        &self,
        slice_id: SliceId,
        outcome: slice::ResolutionOutcome,
    ) -> Result<()> {
        let mut slices = self.slices.write().await;
        let slice = slices
            .get_mut(&slice_id)
            .ok_or_else(|| RhizoCryptError::SliceNotFound(slice_id.to_string()))?;

        if slice.is_resolved() {
            return Err(RhizoCryptError::SliceAlreadyResolved(slice_id.to_string()));
        }

        slice.state = slice::SliceState::Resolved {
            outcome,
            resolved_at: Timestamp::now(),
        };

        Ok(())
    }

    // ========================================================================
    // Dehydration Operations
    // ========================================================================

    /// Start dehydration of a session.
    ///
    /// # Errors
    ///
    /// Returns an error if the session is not found or already dehydrating.
    pub async fn dehydrate(&self, session_id: SessionId) -> Result<merkle::MerkleRoot> {
        // Set status to computing root
        {
            let mut status = self.dehydration_status.write().await;
            status.insert(session_id, dehydration::DehydrationStatus::ComputingRoot);
        }

        // Compute merkle root
        let root = self.compute_merkle_root(session_id).await?;

        // Set status to generating summary
        {
            let mut status = self.dehydration_status.write().await;
            status.insert(session_id, dehydration::DehydrationStatus::GeneratingSummary);
        }

        // In a full implementation, we would:
        // 1. Generate the dehydration summary
        // 2. Collect attestations
        // 3. Commit to LoamSpine
        // For now, we complete with a placeholder commit ref

        let commit_ref = session::LoamCommitRef {
            spine_id: String::new(),
            entry_hash: *root.as_bytes(),
            index: 0,
        };

        // Update status to complete
        {
            let mut status = self.dehydration_status.write().await;
            status.insert(
                session_id,
                dehydration::DehydrationStatus::Completed {
                    commit_ref,
                },
            );
        }

        self.metrics.inc_dehydrations_completed();
        Ok(root)
    }

    /// Get dehydration status for a session.
    pub async fn get_dehydration_status(
        &self,
        session_id: SessionId,
    ) -> dehydration::DehydrationStatus {
        self.dehydration_status
            .read()
            .await
            .get(&session_id)
            .cloned()
            .unwrap_or(dehydration::DehydrationStatus::Pending)
    }
}

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

        // Validate storage backend configuration
        if self.config.storage.backend == StorageBackend::Lmdb {
            return Err(PrimalError::StartupFailed(
                "LMDB storage backend is not yet implemented. Please use Memory (or Sled with feature flag)."
                    .to_string(),
            ));
        }

        self.state = PrimalState::Starting;
        tracing::info!(primal = %self.config.name, "starting");

        // Initialize stores
        {
            let mut dag_store = self.dag_store.write().await;
            *dag_store = Some(InMemoryDagStore::new());
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
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn test_rhizocrypt_lifecycle() {
        let config = RhizoCryptConfig::default();
        let mut primal = RhizoCrypt::new(config);

        assert_eq!(primal.state(), PrimalState::Created);

        primal.start().await.unwrap();
        assert_eq!(primal.state(), PrimalState::Running);
        assert!(primal.uptime_secs().is_some());

        let report = primal.health_check().await.unwrap();
        assert!(report.status.is_healthy());

        primal.stop().await.unwrap();
        assert_eq!(primal.state(), PrimalState::Stopped);
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn test_rhizocrypt_stores() {
        let config = RhizoCryptConfig::default();
        let mut primal = RhizoCrypt::new(config);

        // Should fail before start
        assert!(primal.dag_store().await.is_err());

        primal.start().await.unwrap();

        // Should work after start
        let dag_store = primal.dag_store().await.unwrap();
        assert_eq!(dag_store.session_count().await, 0);

        let payload_store = primal.payload_store().await.unwrap();
        assert_eq!(payload_store.payload_count().await, 0);
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn test_rhizocrypt_invalid_transitions() {
        let config = RhizoCryptConfig::default();
        let mut primal = RhizoCrypt::new(config);

        // Can't stop before starting
        assert!(primal.stop().await.is_err());

        primal.start().await.unwrap();

        // Can't start while running
        assert!(primal.start().await.is_err());
    }
}
