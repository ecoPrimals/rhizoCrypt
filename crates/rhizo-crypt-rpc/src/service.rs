// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2024–2026 ecoPrimals Project

//! RPC service trait definition using tarpc.
//!
//! This is the core RPC interface for rhizoCrypt. The `#[tarpc::service]` macro
//! generates both client and server code from this trait, ensuring compile-time
//! type safety across the network boundary.

use crate::error::RpcError;
use rhizo_crypt_core::{
    DagStore, Did, EventType, MerkleProof, MerkleRoot, Session, SessionBuilder, SessionId,
    SessionState, SessionType, SliceId, SliceMode, Timestamp, Vertex, VertexBuilder, VertexId,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

// ============================================================================
// Request/Response Types
// ============================================================================

/// Session creation request.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateSessionRequest {
    /// Session type.
    pub session_type: SessionType,
    /// Optional description.
    pub description: Option<String>,
    /// Optional parent session.
    pub parent_session: Option<SessionId>,
    /// Maximum vertices allowed.
    pub max_vertices: Option<u64>,
    /// TTL in seconds.
    pub ttl_seconds: Option<u64>,
}

/// Session info response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionInfo {
    /// Session ID.
    pub id: SessionId,
    /// Session type.
    pub session_type: SessionType,
    /// Current state.
    pub state: SessionState,
    /// Vertex count.
    pub vertex_count: u64,
    /// Creation time.
    pub created_at: Timestamp,
    /// Description.
    pub description: Option<String>,
}

/// Event append request.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppendEventRequest {
    /// Target session.
    pub session_id: SessionId,
    /// Event type.
    pub event_type: EventType,
    /// Agent DID.
    pub agent: Option<Did>,
    /// Parent vertices (empty = use frontier).
    pub parents: Vec<VertexId>,
    /// Metadata key-value pairs.
    pub metadata: Vec<(String, String)>,
    /// Optional payload reference.
    pub payload_ref: Option<String>,
}

/// Query request for vertices.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryRequest {
    /// Session to query.
    pub session_id: SessionId,
    /// Filter by event types.
    pub event_types: Option<Vec<EventType>>,
    /// Filter by agent.
    pub agent: Option<Did>,
    /// Start time filter.
    pub start_time: Option<Timestamp>,
    /// End time filter.
    pub end_time: Option<Timestamp>,
    /// Maximum results.
    pub limit: Option<u32>,
}

/// Slice checkout request.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CheckoutSliceRequest {
    /// Spine ID from LoamSpine commit.
    pub spine_id: String,
    /// Entry hash from LoamSpine commit (hex-encoded, 32 bytes).
    pub entry_hash: String,
    /// Entry index in the spine.
    pub entry_index: u64,
    /// Slice mode.
    pub mode: SliceMode,
    /// Owner DID (lender).
    pub owner: Did,
    /// Holder DID (borrower).
    pub holder: Did,
    /// Session ID to associate the slice with.
    pub session_id: SessionId,
    /// Vertex ID marking the checkout point.
    pub checkout_vertex: VertexId,
    /// Optional certificate ID from the spine.
    pub certificate_id: Option<String>,
    /// Duration in seconds.
    pub duration_seconds: Option<u64>,
}

/// Health status response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthStatus {
    /// Whether the service is healthy.
    pub healthy: bool,
    /// Current state description.
    pub state: String,
    /// Active session count.
    pub active_sessions: u64,
    /// Total vertices in memory.
    pub total_vertices: u64,
    /// Uptime in seconds.
    pub uptime_seconds: u64,
}

/// Service metrics.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceMetrics {
    /// Sessions created.
    pub sessions_created: u64,
    /// Sessions resolved.
    pub sessions_resolved: u64,
    /// Vertices appended.
    pub vertices_appended: u64,
    /// Queries executed.
    pub queries_executed: u64,
    /// Slices checked out.
    pub slices_checked_out: u64,
    /// Dehydrations completed.
    pub dehydrations_completed: u64,
}

// ============================================================================
// tarpc Service Trait
// ============================================================================

/// rhizoCrypt RPC service.
///
/// This trait defines the complete RPC interface for rhizoCrypt. The `#[tarpc::service]`
/// macro generates:
/// - `RhizoCryptRpcClient` - async client stub
/// - `RhizoCryptRpcServer` - server trait to implement
///
/// All types are checked at compile time. No runtime schema validation needed.
#[tarpc::service]
pub trait RhizoCryptRpc {
    // ========================================================================
    // Session Operations
    // ========================================================================

    /// Create a new session.
    async fn create_session(request: CreateSessionRequest) -> Result<SessionId, RpcError>;

    /// Get session info.
    async fn get_session(session_id: SessionId) -> Result<SessionInfo, RpcError>;

    /// List all active sessions.
    async fn list_sessions() -> Result<Vec<SessionInfo>, RpcError>;

    /// Discard a session (delete without committing).
    async fn discard_session(session_id: SessionId) -> Result<(), RpcError>;

    // ========================================================================
    // Event Operations
    // ========================================================================

    /// Append an event to a session.
    async fn append_event(request: AppendEventRequest) -> Result<VertexId, RpcError>;

    /// Append multiple events in a batch.
    async fn append_batch(requests: Vec<AppendEventRequest>) -> Result<Vec<VertexId>, RpcError>;

    // ========================================================================
    // Query Operations
    // ========================================================================

    /// Get a specific vertex by ID.
    async fn get_vertex(session_id: SessionId, vertex_id: VertexId) -> Result<Vertex, RpcError>;

    /// Get the current frontier (DAG tips).
    async fn get_frontier(session_id: SessionId) -> Result<Vec<VertexId>, RpcError>;

    /// Get genesis vertices (DAG roots).
    async fn get_genesis(session_id: SessionId) -> Result<Vec<VertexId>, RpcError>;

    /// Query vertices with filters.
    async fn query_vertices(request: QueryRequest) -> Result<Vec<Vertex>, RpcError>;

    /// Get children of a vertex.
    async fn get_children(
        session_id: SessionId,
        vertex_id: VertexId,
    ) -> Result<Vec<VertexId>, RpcError>;

    // ========================================================================
    // Merkle Operations
    // ========================================================================

    /// Get the Merkle root for a session.
    async fn get_merkle_root(session_id: SessionId) -> Result<MerkleRoot, RpcError>;

    /// Generate inclusion proof for a vertex.
    async fn get_merkle_proof(
        session_id: SessionId,
        vertex_id: VertexId,
    ) -> Result<MerkleProof, RpcError>;

    /// Verify a Merkle proof.
    async fn verify_proof(root: MerkleRoot, proof: MerkleProof) -> Result<bool, RpcError>;

    // ========================================================================
    // Slice Operations
    // ========================================================================

    /// Checkout a slice from `LoamSpine`.
    async fn checkout_slice(request: CheckoutSliceRequest) -> Result<SliceId, RpcError>;

    /// Get slice info.
    async fn get_slice(slice_id: SliceId) -> Result<rhizo_crypt_core::Slice, RpcError>;

    /// List active slices.
    async fn list_slices() -> Result<Vec<rhizo_crypt_core::Slice>, RpcError>;

    /// Resolve a slice (commit back to `LoamSpine`).
    async fn resolve_slice(slice_id: SliceId, session_id: SessionId) -> Result<(), RpcError>;

    // ========================================================================
    // Dehydration Operations
    // ========================================================================

    /// Trigger dehydration of a session to `LoamSpine`.
    async fn dehydrate(session_id: SessionId) -> Result<MerkleRoot, RpcError>;

    /// Get dehydration status.
    async fn get_dehydration_status(
        session_id: SessionId,
    ) -> Result<rhizo_crypt_core::DehydrationStatus, RpcError>;

    // ========================================================================
    // Health & Metrics
    // ========================================================================

    /// Health check.
    async fn health() -> Result<HealthStatus, RpcError>;

    /// Get service metrics.
    async fn metrics() -> Result<ServiceMetrics, RpcError>;
}

// ============================================================================
// Server Implementation Helper
// ============================================================================

/// Convert a `Session` to `SessionInfo`.
fn session_to_info(session: &Session) -> SessionInfo {
    SessionInfo {
        id: session.id,
        session_type: session.session_type.clone(),
        state: session.state.clone(),
        vertex_count: session.vertex_count,
        created_at: session.created_at,
        description: session.name.clone(),
    }
}

/// Server implementation wrapper.
///
/// Implements `RhizoCryptRpc` by delegating to a `RhizoCrypt` primal instance.
#[derive(Clone)]
pub struct RhizoCryptRpcServer {
    primal: Arc<rhizo_crypt_core::RhizoCrypt>,
    start_time: std::time::Instant,
}

impl RhizoCryptRpcServer {
    /// Create a new RPC server wrapping a `RhizoCrypt` primal.
    #[must_use]
    pub fn new(primal: Arc<rhizo_crypt_core::RhizoCrypt>) -> Self {
        Self {
            primal,
            start_time: std::time::Instant::now(),
        }
    }
}

impl RhizoCryptRpc for RhizoCryptRpcServer {
    async fn create_session(
        self,
        _: tarpc::context::Context,
        request: CreateSessionRequest,
    ) -> Result<SessionId, RpcError> {
        let mut builder = SessionBuilder::new(request.session_type);

        if let Some(desc) = request.description {
            builder = builder.with_name(desc);
        }

        if let Some(max) = request.max_vertices {
            builder = builder.with_max_vertices(max);
        }

        if let Some(ttl) = request.ttl_seconds {
            builder = builder.with_max_duration(std::time::Duration::from_secs(ttl));
        }

        let session = builder.build();
        self.primal.create_session(session).map_err(RpcError::from)
    }

    async fn get_session(
        self,
        _: tarpc::context::Context,
        session_id: SessionId,
    ) -> Result<SessionInfo, RpcError> {
        let session = self.primal.get_session(session_id).map_err(RpcError::from)?;
        Ok(session_to_info(&session))
    }

    async fn list_sessions(self, _: tarpc::context::Context) -> Result<Vec<SessionInfo>, RpcError> {
        let sessions = self.primal.list_sessions();
        Ok(sessions.iter().map(session_to_info).collect())
    }

    async fn discard_session(
        self,
        _: tarpc::context::Context,
        session_id: SessionId,
    ) -> Result<(), RpcError> {
        self.primal.discard_session(session_id).await.map_err(RpcError::from)
    }

    async fn append_event(
        self,
        _: tarpc::context::Context,
        request: AppendEventRequest,
    ) -> Result<VertexId, RpcError> {
        let mut builder = VertexBuilder::new(request.event_type);

        if let Some(agent) = request.agent {
            builder = builder.with_agent(agent);
        }

        for parent in request.parents {
            builder = builder.with_parent(parent);
        }

        for (key, value) in request.metadata {
            builder = builder.with_metadata(key, value);
        }

        let vertex = builder.build();
        self.primal.append_vertex(request.session_id, vertex).await.map_err(RpcError::from)
    }

    async fn append_batch(
        self,
        _: tarpc::context::Context,
        requests: Vec<AppendEventRequest>,
    ) -> Result<Vec<VertexId>, RpcError> {
        let mut results = Vec::with_capacity(requests.len());
        for request in requests {
            let mut builder = VertexBuilder::new(request.event_type);
            if let Some(agent) = request.agent {
                builder = builder.with_agent(agent);
            }
            for parent in request.parents {
                builder = builder.with_parent(parent);
            }
            for (key, value) in request.metadata {
                builder = builder.with_metadata(key, value);
            }
            let vertex = builder.build();
            let id = self
                .primal
                .append_vertex(request.session_id, vertex)
                .await
                .map_err(RpcError::from)?;
            results.push(id);
        }
        Ok(results)
    }

    async fn get_vertex(
        self,
        _: tarpc::context::Context,
        session_id: SessionId,
        vertex_id: VertexId,
    ) -> Result<Vertex, RpcError> {
        tracing::debug!("get_vertex called: session={session_id:?}, vertex={vertex_id:?}");
        let result = self.primal.get_vertex(session_id, vertex_id).await;
        tracing::debug!("get_vertex result: {result:?}");
        result.map_err(RpcError::from)
    }

    async fn get_frontier(
        self,
        _: tarpc::context::Context,
        session_id: SessionId,
    ) -> Result<Vec<VertexId>, RpcError> {
        let session = self.primal.get_session(session_id).map_err(RpcError::from)?;
        Ok(session.frontier.into_iter().collect())
    }

    async fn get_genesis(
        self,
        _: tarpc::context::Context,
        session_id: SessionId,
    ) -> Result<Vec<VertexId>, RpcError> {
        let session = self.primal.get_session(session_id).map_err(RpcError::from)?;
        Ok(session.genesis.into_iter().collect())
    }

    async fn query_vertices(
        self,
        _: tarpc::context::Context,
        request: QueryRequest,
    ) -> Result<Vec<Vertex>, RpcError> {
        let event_types = request.event_types;
        let agent = request.agent;
        let limit = request.limit.map(|l| l as usize);

        self.primal
            .query_vertices(request.session_id, event_types.as_deref(), agent.as_ref(), limit)
            .await
            .map_err(RpcError::from)
    }

    async fn get_children(
        self,
        _: tarpc::context::Context,
        session_id: SessionId,
        vertex_id: VertexId,
    ) -> Result<Vec<VertexId>, RpcError> {
        let dag_store = self.primal.dag_store().await.map_err(RpcError::from)?;

        dag_store.get_children(session_id, vertex_id).await.map_err(RpcError::from)
    }

    async fn get_merkle_root(
        self,
        _: tarpc::context::Context,
        session_id: SessionId,
    ) -> Result<MerkleRoot, RpcError> {
        self.primal.compute_merkle_root(session_id).await.map_err(RpcError::from)
    }

    async fn get_merkle_proof(
        self,
        _: tarpc::context::Context,
        session_id: SessionId,
        vertex_id: VertexId,
    ) -> Result<MerkleProof, RpcError> {
        self.primal.generate_merkle_proof(session_id, vertex_id).await.map_err(RpcError::from)
    }

    async fn verify_proof(
        self,
        _: tarpc::context::Context,
        _root: MerkleRoot,
        proof: MerkleProof,
    ) -> Result<bool, RpcError> {
        let session_id = self
            .primal
            .session_for_vertex(proof.vertex_id)
            .ok_or_else(|| RpcError::VertexNotFound(proof.vertex_id.to_string()))?;

        let vertex =
            self.primal.get_vertex(session_id, proof.vertex_id).await.map_err(RpcError::from)?;

        Ok(proof.verify(&vertex))
    }

    async fn checkout_slice(
        self,
        _: tarpc::context::Context,
        request: CheckoutSliceRequest,
    ) -> Result<SliceId, RpcError> {
        use rhizo_crypt_core::slice;

        let entry_hash_bytes = hex::decode(&request.entry_hash)
            .map_err(|e| RpcError::InvalidRequest(format!("invalid entry_hash hex: {e}")))?;
        let mut entry_hash = [0u8; 32];
        let copy_len = entry_hash_bytes.len().min(32);
        entry_hash[..copy_len].copy_from_slice(&entry_hash_bytes[..copy_len]);

        let origin = slice::SliceOrigin {
            spine_id: request.spine_id,
            entry_hash,
            entry_index: request.entry_index,
            certificate_id: request.certificate_id,
            owner: request.owner,
        };

        let slice = slice::SliceBuilder::new(
            origin,
            request.holder,
            request.mode,
            request.session_id,
            request.checkout_vertex,
        )
        .build();

        self.primal.checkout_slice(slice).map_err(RpcError::from)
    }

    async fn get_slice(
        self,
        _: tarpc::context::Context,
        slice_id: SliceId,
    ) -> Result<rhizo_crypt_core::Slice, RpcError> {
        self.primal.get_slice(slice_id).map_err(RpcError::from)
    }

    async fn list_slices(
        self,
        _: tarpc::context::Context,
    ) -> Result<Vec<rhizo_crypt_core::Slice>, RpcError> {
        Ok(self.primal.list_slices())
    }

    async fn resolve_slice(
        self,
        _: tarpc::context::Context,
        slice_id: SliceId,
        _session_id: SessionId,
    ) -> Result<(), RpcError> {
        use rhizo_crypt_core::slice::ResolutionOutcome;

        self.primal
            .resolve_slice(slice_id, ResolutionOutcome::ReturnedUnchanged)
            .map_err(RpcError::from)
    }

    async fn dehydrate(
        self,
        _: tarpc::context::Context,
        session_id: SessionId,
    ) -> Result<MerkleRoot, RpcError> {
        self.primal.dehydrate(session_id).await.map_err(RpcError::from)
    }

    async fn get_dehydration_status(
        self,
        _: tarpc::context::Context,
        session_id: SessionId,
    ) -> Result<rhizo_crypt_core::DehydrationStatus, RpcError> {
        Ok(self.primal.get_dehydration_status(session_id))
    }

    async fn health(self, _: tarpc::context::Context) -> Result<HealthStatus, RpcError> {
        use rhizo_crypt_core::PrimalLifecycle;

        let state = self.primal.state();
        let session_count = self.primal.session_count();
        let vertex_count = self.primal.total_vertex_count();

        Ok(HealthStatus {
            healthy: state.is_running(),
            state: format!("{state}"),
            active_sessions: session_count as u64,
            total_vertices: vertex_count,
            uptime_seconds: self.start_time.elapsed().as_secs(),
        })
    }

    async fn metrics(self, _: tarpc::context::Context) -> Result<ServiceMetrics, RpcError> {
        let metrics = self.primal.metrics();

        Ok(ServiceMetrics {
            sessions_created: metrics.get_sessions_created(),
            sessions_resolved: metrics.get_sessions_resolved(),
            vertices_appended: metrics.get_vertices_appended(),
            queries_executed: metrics.get_queries_executed(),
            slices_checked_out: metrics.get_slices_checked_out(),
            dehydrations_completed: metrics.get_dehydrations_completed(),
        })
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;
    use rhizo_crypt_core::{PrimalLifecycle, RhizoCrypt, RhizoCryptConfig};
    use std::sync::Arc;

    #[test]
    fn test_request_serialization() {
        let request = CreateSessionRequest {
            session_type: SessionType::default(),
            description: Some("test".to_string()),
            parent_session: None,
            max_vertices: Some(1000),
            ttl_seconds: Some(3600),
        };

        let json = serde_json::to_string(&request).expect("serialize");
        let parsed: CreateSessionRequest = serde_json::from_str(&json).expect("deserialize");

        assert_eq!(parsed.description, request.description);
        assert_eq!(parsed.max_vertices, request.max_vertices);
    }

    #[test]
    fn test_health_status_serialization() {
        let status = HealthStatus {
            healthy: true,
            state: "running".to_string(),
            active_sessions: 5,
            total_vertices: 1000,
            uptime_seconds: 3600,
        };

        let json = serde_json::to_string(&status).expect("serialize");
        let parsed: HealthStatus = serde_json::from_str(&json).expect("deserialize");

        assert!(parsed.healthy);
        assert_eq!(parsed.active_sessions, 5);
    }

    async fn make_test_server() -> RhizoCryptRpcServer {
        let mut primal = RhizoCrypt::new(RhizoCryptConfig::default());
        primal.start().await.unwrap();
        let primal = Arc::new(primal);
        RhizoCryptRpcServer::new(primal)
    }

    #[tokio::test]
    async fn test_service_all_methods_accessible() {
        let server = make_test_server().await;

        // Create session
        let req = CreateSessionRequest {
            session_type: SessionType::default(),
            description: Some("test".to_string()),
            parent_session: None,
            max_vertices: Some(1000),
            ttl_seconds: Some(3600),
        };
        let session_id =
            server.clone().create_session(tarpc::context::current(), req).await.unwrap();

        // Get session
        let _ = server.clone().get_session(tarpc::context::current(), session_id).await.unwrap();

        // List sessions
        let _ = server.clone().list_sessions(tarpc::context::current()).await.unwrap();

        // Append event
        let append_req = AppendEventRequest {
            session_id,
            event_type: EventType::SessionStart,
            agent: None,
            parents: vec![],
            metadata: vec![],
            payload_ref: None,
        };
        let vertex_id =
            server.clone().append_event(tarpc::context::current(), append_req).await.unwrap();

        // Get vertex, frontier, genesis
        let _ = server
            .clone()
            .get_vertex(tarpc::context::current(), session_id, vertex_id)
            .await
            .unwrap();
        let _ = server.clone().get_frontier(tarpc::context::current(), session_id).await.unwrap();
        let _ = server.clone().get_genesis(tarpc::context::current(), session_id).await.unwrap();

        // Query vertices
        let query_req = QueryRequest {
            session_id,
            event_types: None,
            agent: None,
            start_time: None,
            end_time: None,
            limit: None,
        };
        let _ = server.clone().query_vertices(tarpc::context::current(), query_req).await.unwrap();

        // Merkle operations
        let _ =
            server.clone().get_merkle_root(tarpc::context::current(), session_id).await.unwrap();

        // Health and metrics
        let _ = server.clone().health(tarpc::context::current()).await.unwrap();
        let _ = server.clone().metrics(tarpc::context::current()).await.unwrap();
    }

    #[tokio::test]
    async fn test_dehydration_status() {
        let server = make_test_server().await;

        let req = CreateSessionRequest {
            session_type: SessionType::default(),
            description: None,
            parent_session: None,
            max_vertices: None,
            ttl_seconds: None,
        };
        let session_id =
            server.clone().create_session(tarpc::context::current(), req).await.unwrap();

        let status =
            server.get_dehydration_status(tarpc::context::current(), session_id).await.unwrap();
        assert!(matches!(status, rhizo_crypt_core::DehydrationStatus::Pending));
    }

    #[tokio::test]
    async fn test_get_slice_not_found() {
        let server = make_test_server().await;

        let nonexistent_id = SliceId::now();
        let result = server.get_slice(tarpc::context::current(), nonexistent_id).await;
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), RpcError::SliceNotFound(_)));
    }

    #[tokio::test]
    async fn test_list_slices_empty() {
        let server = make_test_server().await;

        let slices = server.list_slices(tarpc::context::current()).await.unwrap();
        assert!(slices.is_empty());
    }

    #[tokio::test]
    async fn test_resolve_slice_not_found() {
        let server = make_test_server().await;

        let nonexistent_id = SliceId::now();
        let session_id = SessionId::now();
        let result =
            server.resolve_slice(tarpc::context::current(), nonexistent_id, session_id).await;
        assert!(result.is_err());
    }
}
