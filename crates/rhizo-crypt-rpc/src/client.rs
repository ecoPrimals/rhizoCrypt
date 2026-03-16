// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2024–2026 ecoPrimals Project

//! RPC client implementation.

use crate::error::{RpcError, RpcResult};
use crate::service::{
    AppendEventRequest, CheckoutSliceRequest, CreateSessionRequest, HealthStatus, QueryRequest,
    RhizoCryptRpcClient as GeneratedClient, ServiceMetrics, SessionInfo,
};
use rhizo_crypt_core::{
    DehydrationStatus, MerkleProof, MerkleRoot, SessionId, Slice, SliceId, Vertex, VertexId,
};
use std::net::SocketAddr;
use tarpc::tokio_serde::formats::Bincode;
use tarpc::{client, context};
use tracing::info;

/// RPC client for rhizoCrypt.
///
/// Provides a high-level async API for interacting with a rhizoCrypt service.
/// All methods are compile-time type-checked via tarpc.
#[derive(Debug)]
pub struct RpcClient {
    inner: GeneratedClient,
}

impl RpcClient {
    /// Connect to a rhizoCrypt RPC server.
    ///
    /// # Errors
    ///
    /// Returns `RpcError::Connection` if the connection fails.
    pub async fn connect(addr: SocketAddr) -> RpcResult<Self> {
        let transport = tarpc::serde_transport::tcp::connect(&addr, Bincode::default)
            .await
            .map_err(|e| RpcError::Connection(e.to_string()))?;

        info!("connected to rhizoCrypt RPC at {}", addr);

        let inner = GeneratedClient::new(client::Config::default(), transport).spawn();

        Ok(Self {
            inner,
        })
    }

    // ========================================================================
    // Session Operations
    // ========================================================================

    /// Create a new session.
    ///
    /// # Errors
    ///
    /// Returns `RpcError::Transport` if the RPC call fails.
    pub async fn create_session(&self, request: CreateSessionRequest) -> RpcResult<SessionId> {
        self.inner
            .create_session(context::current(), request)
            .await
            .map_err(|e| RpcError::Transport(e.to_string()))?
    }

    /// Get session info.
    ///
    /// # Errors
    ///
    /// Returns `RpcError::Transport` if the RPC call fails.
    pub async fn get_session(&self, session_id: SessionId) -> RpcResult<SessionInfo> {
        self.inner
            .get_session(context::current(), session_id)
            .await
            .map_err(|e| RpcError::Transport(e.to_string()))?
    }

    /// List all active sessions.
    ///
    /// # Errors
    ///
    /// Returns `RpcError::Transport` if the RPC call fails.
    pub async fn list_sessions(&self) -> RpcResult<Vec<SessionInfo>> {
        self.inner
            .list_sessions(context::current())
            .await
            .map_err(|e| RpcError::Transport(e.to_string()))?
    }

    /// Discard a session.
    ///
    /// # Errors
    ///
    /// Returns `RpcError::Transport` if the RPC call fails.
    pub async fn discard_session(&self, session_id: SessionId) -> RpcResult<()> {
        self.inner
            .discard_session(context::current(), session_id)
            .await
            .map_err(|e| RpcError::Transport(e.to_string()))?
    }

    // ========================================================================
    // Event Operations
    // ========================================================================

    /// Append an event to a session.
    ///
    /// # Errors
    ///
    /// Returns `RpcError::Transport` if the RPC call fails.
    pub async fn append_event(&self, request: AppendEventRequest) -> RpcResult<VertexId> {
        self.inner
            .append_event(context::current(), request)
            .await
            .map_err(|e| RpcError::Transport(e.to_string()))?
    }

    /// Append multiple events in a batch.
    ///
    /// # Errors
    ///
    /// Returns `RpcError::Transport` if the RPC call fails.
    pub async fn append_batch(
        &self,
        requests: Vec<AppendEventRequest>,
    ) -> RpcResult<Vec<VertexId>> {
        self.inner
            .append_batch(context::current(), requests)
            .await
            .map_err(|e| RpcError::Transport(e.to_string()))?
    }

    // ========================================================================
    // Query Operations
    // ========================================================================

    /// Get a specific vertex by ID.
    ///
    /// # Errors
    ///
    /// Returns `RpcError::Transport` if the RPC call fails.
    pub async fn get_vertex(
        &self,
        session_id: SessionId,
        vertex_id: VertexId,
    ) -> RpcResult<Vertex> {
        self.inner
            .get_vertex(context::current(), session_id, vertex_id)
            .await
            .map_err(|e| RpcError::Transport(e.to_string()))?
    }

    /// Get the current frontier (DAG tips).
    ///
    /// # Errors
    ///
    /// Returns `RpcError::Transport` if the RPC call fails.
    pub async fn get_frontier(&self, session_id: SessionId) -> RpcResult<Vec<VertexId>> {
        self.inner
            .get_frontier(context::current(), session_id)
            .await
            .map_err(|e| RpcError::Transport(e.to_string()))?
    }

    /// Get genesis vertices (DAG roots).
    ///
    /// # Errors
    ///
    /// Returns `RpcError::Transport` if the RPC call fails.
    pub async fn get_genesis(&self, session_id: SessionId) -> RpcResult<Vec<VertexId>> {
        self.inner
            .get_genesis(context::current(), session_id)
            .await
            .map_err(|e| RpcError::Transport(e.to_string()))?
    }

    /// Query vertices with filters.
    ///
    /// # Errors
    ///
    /// Returns `RpcError::Transport` if the RPC call fails.
    pub async fn query_vertices(&self, request: QueryRequest) -> RpcResult<Vec<Vertex>> {
        self.inner
            .query_vertices(context::current(), request)
            .await
            .map_err(|e| RpcError::Transport(e.to_string()))?
    }

    /// Get children of a vertex.
    ///
    /// # Errors
    ///
    /// Returns `RpcError::Transport` if the RPC call fails.
    pub async fn get_children(
        &self,
        session_id: SessionId,
        vertex_id: VertexId,
    ) -> RpcResult<Vec<VertexId>> {
        self.inner
            .get_children(context::current(), session_id, vertex_id)
            .await
            .map_err(|e| RpcError::Transport(e.to_string()))?
    }

    // ========================================================================
    // Merkle Operations
    // ========================================================================

    /// Get the Merkle root for a session.
    ///
    /// # Errors
    ///
    /// Returns `RpcError::Transport` if the RPC call fails.
    pub async fn get_merkle_root(&self, session_id: SessionId) -> RpcResult<MerkleRoot> {
        self.inner
            .get_merkle_root(context::current(), session_id)
            .await
            .map_err(|e| RpcError::Transport(e.to_string()))?
    }

    /// Generate inclusion proof for a vertex.
    ///
    /// # Errors
    ///
    /// Returns `RpcError::Transport` if the RPC call fails.
    pub async fn get_merkle_proof(
        &self,
        session_id: SessionId,
        vertex_id: VertexId,
    ) -> RpcResult<MerkleProof> {
        self.inner
            .get_merkle_proof(context::current(), session_id, vertex_id)
            .await
            .map_err(|e| RpcError::Transport(e.to_string()))?
    }

    /// Verify a Merkle proof.
    ///
    /// # Errors
    ///
    /// Returns `RpcError::Transport` if the RPC call fails.
    pub async fn verify_proof(&self, root: MerkleRoot, proof: MerkleProof) -> RpcResult<bool> {
        self.inner
            .verify_proof(context::current(), root, proof)
            .await
            .map_err(|e| RpcError::Transport(e.to_string()))?
    }

    // ========================================================================
    // Slice Operations
    // ========================================================================

    /// Checkout a slice from `LoamSpine`.
    ///
    /// # Errors
    ///
    /// Returns `RpcError::Transport` if the RPC call fails.
    pub async fn checkout_slice(&self, request: CheckoutSliceRequest) -> RpcResult<SliceId> {
        self.inner
            .checkout_slice(context::current(), request)
            .await
            .map_err(|e| RpcError::Transport(e.to_string()))?
    }

    /// Get slice info.
    ///
    /// # Errors
    ///
    /// Returns `RpcError::Transport` if the RPC call fails.
    pub async fn get_slice(&self, slice_id: SliceId) -> RpcResult<Slice> {
        self.inner
            .get_slice(context::current(), slice_id)
            .await
            .map_err(|e| RpcError::Transport(e.to_string()))?
    }

    /// List active slices.
    ///
    /// # Errors
    ///
    /// Returns `RpcError::Transport` if the RPC call fails.
    pub async fn list_slices(&self) -> RpcResult<Vec<Slice>> {
        self.inner
            .list_slices(context::current())
            .await
            .map_err(|e| RpcError::Transport(e.to_string()))?
    }

    /// Resolve a slice (commit back to `LoamSpine`).
    ///
    /// # Errors
    ///
    /// Returns `RpcError::Transport` if the RPC call fails.
    pub async fn resolve_slice(&self, slice_id: SliceId, session_id: SessionId) -> RpcResult<()> {
        self.inner
            .resolve_slice(context::current(), slice_id, session_id)
            .await
            .map_err(|e| RpcError::Transport(e.to_string()))?
    }

    // ========================================================================
    // Dehydration Operations
    // ========================================================================

    /// Trigger dehydration of a session to `LoamSpine`.
    ///
    /// # Errors
    ///
    /// Returns `RpcError::Transport` if the RPC call fails.
    pub async fn dehydrate(&self, session_id: SessionId) -> RpcResult<MerkleRoot> {
        self.inner
            .dehydrate(context::current(), session_id)
            .await
            .map_err(|e| RpcError::Transport(e.to_string()))?
    }

    /// Get dehydration status.
    ///
    /// # Errors
    ///
    /// Returns `RpcError::Transport` if the RPC call fails.
    pub async fn get_dehydration_status(
        &self,
        session_id: SessionId,
    ) -> RpcResult<DehydrationStatus> {
        self.inner
            .get_dehydration_status(context::current(), session_id)
            .await
            .map_err(|e| RpcError::Transport(e.to_string()))?
    }

    // ========================================================================
    // Health & Metrics
    // ========================================================================

    /// Health check.
    ///
    /// # Errors
    ///
    /// Returns `RpcError::Transport` if the RPC call fails.
    pub async fn health(&self) -> RpcResult<HealthStatus> {
        self.inner
            .health(context::current())
            .await
            .map_err(|e| RpcError::Transport(e.to_string()))?
    }

    /// Get service metrics.
    ///
    /// # Errors
    ///
    /// Returns `RpcError::Transport` if the RPC call fails.
    pub async fn metrics(&self) -> RpcResult<ServiceMetrics> {
        self.inner
            .metrics(context::current())
            .await
            .map_err(|e| RpcError::Transport(e.to_string()))?
    }

    // ========================================================================
    // Capability Discovery (Spring-as-Niche Standard)
    // ========================================================================

    /// List capabilities this primal provides.
    ///
    /// # Errors
    ///
    /// Returns `RpcError::Transport` if the RPC call fails.
    pub async fn list_capabilities(&self) -> RpcResult<Vec<crate::service::CapabilityDescriptor>> {
        self.inner
            .list_capabilities(context::current())
            .await
            .map_err(|e| RpcError::Transport(e.to_string()))?
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
#[expect(clippy::unwrap_used, reason = "test code")]
mod tests {
    use super::*;
    use rhizo_crypt_core::{Did, EventType, SessionState, SessionType, SliceMode, Timestamp};
    use std::net::SocketAddr;

    // ------------------------------------------------------------------------
    // Client Creation and Configuration
    // ------------------------------------------------------------------------

    #[tokio::test]
    async fn test_connect_to_invalid_address_returns_connection_error() {
        // Connect to a port that should have no server listening
        let addr: SocketAddr = "127.0.0.1:1".parse().unwrap();
        let result = RpcClient::connect(addr).await;
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(matches!(err, RpcError::Connection(_)), "Expected Connection error, got: {err:?}");
    }

    #[tokio::test]
    async fn test_connect_to_unreachable_address_returns_error() {
        // Use a non-routable address (or loopback with closed port)
        let addr: SocketAddr = "127.0.0.1:47999".parse().unwrap();
        let result = RpcClient::connect(addr).await;
        assert!(result.is_err());
    }

    // ------------------------------------------------------------------------
    // Method Calls When Not Connected (Error Paths)
    // ------------------------------------------------------------------------

    /// Create a client with a transport whose peer has been dropped.
    /// Method calls will fail with transport errors.
    fn client_with_dropped_transport() -> RpcClient {
        let (client_io, _server_io) = tokio::io::duplex(4096);
        // Drop _server_io so the connection is broken
        let transport = tarpc::serde_transport::Transport::from((client_io, Bincode::default()));
        let inner = GeneratedClient::new(client::Config::default(), transport).spawn();
        RpcClient {
            inner,
        }
    }

    #[tokio::test]
    async fn test_create_session_without_connection_errors_gracefully() {
        let client = client_with_dropped_transport();
        let request = CreateSessionRequest {
            session_type: SessionType::default(),
            description: Some("test".to_string()),
            parent_session: None,
            max_vertices: Some(1000),
            ttl_seconds: Some(3600),
        };
        let result = client.create_session(request).await;
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(matches!(err, RpcError::Transport(_)), "Expected Transport error, got: {err:?}");
    }

    #[tokio::test]
    async fn test_get_session_without_connection_errors_gracefully() {
        let client = client_with_dropped_transport();
        let session_id = SessionId::now();
        let result = client.get_session(session_id).await;
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), RpcError::Transport(_)));
    }

    #[tokio::test]
    async fn test_list_sessions_without_connection_errors_gracefully() {
        let client = client_with_dropped_transport();
        let result = client.list_sessions().await;
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), RpcError::Transport(_)));
    }

    #[tokio::test]
    async fn test_discard_session_without_connection_errors_gracefully() {
        let client = client_with_dropped_transport();
        let session_id = SessionId::now();
        let result = client.discard_session(session_id).await;
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), RpcError::Transport(_)));
    }

    #[tokio::test]
    async fn test_append_event_without_connection_errors_gracefully() {
        let client = client_with_dropped_transport();
        let request = AppendEventRequest {
            session_id: SessionId::now(),
            event_type: EventType::SessionStart,
            agent: None,
            parents: vec![],
            metadata: vec![],
            payload_ref: None,
        };
        let result = client.append_event(request).await;
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), RpcError::Transport(_)));
    }

    #[tokio::test]
    async fn test_append_batch_without_connection_errors_gracefully() {
        let client = client_with_dropped_transport();
        let requests = vec![AppendEventRequest {
            session_id: SessionId::now(),
            event_type: EventType::SessionStart,
            agent: None,
            parents: vec![],
            metadata: vec![],
            payload_ref: None,
        }];
        let result = client.append_batch(requests).await;
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), RpcError::Transport(_)));
    }

    #[tokio::test]
    async fn test_health_without_connection_errors_gracefully() {
        let client = client_with_dropped_transport();
        let result = client.health().await;
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), RpcError::Transport(_)));
    }

    #[tokio::test]
    async fn test_metrics_without_connection_errors_gracefully() {
        let client = client_with_dropped_transport();
        let result = client.metrics().await;
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), RpcError::Transport(_)));
    }

    #[tokio::test]
    async fn test_get_vertex_without_connection_errors_gracefully() {
        let client = client_with_dropped_transport();
        let result = client.get_vertex(SessionId::now(), VertexId::ZERO).await;
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), RpcError::Transport(_)));
    }

    #[tokio::test]
    async fn test_get_frontier_without_connection_errors_gracefully() {
        let client = client_with_dropped_transport();
        let result = client.get_frontier(SessionId::now()).await;
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), RpcError::Transport(_)));
    }

    #[tokio::test]
    async fn test_get_merkle_root_without_connection_errors_gracefully() {
        let client = client_with_dropped_transport();
        let result = client.get_merkle_root(SessionId::now()).await;
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), RpcError::Transport(_)));
    }

    #[tokio::test]
    async fn test_checkout_slice_without_connection_errors_gracefully() {
        let client = client_with_dropped_transport();
        let request = CheckoutSliceRequest {
            spine_id: "spine-0".to_string(),
            entry_hash: "00".repeat(32),
            entry_index: 0,
            mode: SliceMode::Copy {
                allow_recopy: false,
            },
            owner: Did::new("did:eco:owner"),
            holder: Did::new("did:eco:holder"),
            session_id: SessionId::now(),
            checkout_vertex: VertexId::ZERO,
            certificate_id: None,
            duration_seconds: None,
        };
        let result = client.checkout_slice(request).await;
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), RpcError::Transport(_)));
    }

    #[tokio::test]
    async fn test_dehydrate_without_connection_errors_gracefully() {
        let client = client_with_dropped_transport();
        let result = client.dehydrate(SessionId::now()).await;
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), RpcError::Transport(_)));
    }

    // ------------------------------------------------------------------------
    // Serialization of Request/Response Types
    // ------------------------------------------------------------------------

    #[test]
    fn test_create_session_request_serialization() {
        let request = CreateSessionRequest {
            session_type: SessionType::default(),
            description: Some("test session".to_string()),
            parent_session: None,
            max_vertices: Some(5000),
            ttl_seconds: Some(7200),
        };
        let bytes = bincode::serialize(&request).unwrap();
        let parsed: CreateSessionRequest = bincode::deserialize(&bytes).unwrap();
        assert_eq!(parsed.description, request.description);
        assert_eq!(parsed.max_vertices, request.max_vertices);
        assert_eq!(parsed.ttl_seconds, request.ttl_seconds);
    }

    #[test]
    fn test_append_event_request_serialization() {
        let request = AppendEventRequest {
            session_id: SessionId::now(),
            event_type: EventType::DataCreate {
                schema: Some("test://schema".to_string()),
            },
            agent: None,
            parents: vec![VertexId::ZERO],
            metadata: vec![("key".to_string(), "value".to_string())],
            payload_ref: Some("ref://payload".to_string()),
        };
        let bytes = bincode::serialize(&request).unwrap();
        let parsed: AppendEventRequest = bincode::deserialize(&bytes).unwrap();
        assert_eq!(parsed.session_id, request.session_id);
        assert_eq!(parsed.metadata, request.metadata);
    }

    #[test]
    fn test_query_request_serialization() {
        let request = QueryRequest {
            session_id: SessionId::now(),
            event_types: Some(vec![EventType::SessionStart]),
            agent: None,
            start_time: None,
            end_time: None,
            limit: Some(100),
        };
        let bytes = bincode::serialize(&request).unwrap();
        let parsed: QueryRequest = bincode::deserialize(&bytes).unwrap();
        assert_eq!(parsed.session_id, request.session_id);
        assert_eq!(parsed.limit, request.limit);
    }

    #[test]
    fn test_checkout_slice_request_serialization() {
        let request = CheckoutSliceRequest {
            spine_id: "spine-42".to_string(),
            entry_hash: "ab".repeat(32),
            entry_index: 42,
            mode: SliceMode::Copy {
                allow_recopy: true,
            },
            owner: Did::new("did:eco:owner"),
            holder: Did::new("did:eco:holder"),
            session_id: SessionId::now(),
            checkout_vertex: VertexId::ZERO,
            certificate_id: None,
            duration_seconds: Some(3600),
        };
        let bytes = bincode::serialize(&request).unwrap();
        let parsed: CheckoutSliceRequest = bincode::deserialize(&bytes).unwrap();
        assert_eq!(parsed.spine_id, request.spine_id);
        assert_eq!(parsed.entry_index, request.entry_index);
        assert_eq!(parsed.duration_seconds, request.duration_seconds);
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
        let bytes = bincode::serialize(&status).unwrap();
        let parsed: HealthStatus = bincode::deserialize(&bytes).unwrap();
        assert!(parsed.healthy);
        assert_eq!(parsed.active_sessions, 5);
        assert_eq!(parsed.uptime_seconds, 3600);
    }

    #[test]
    fn test_service_metrics_serialization() {
        let metrics = ServiceMetrics {
            sessions_created: 10,
            sessions_resolved: 8,
            vertices_appended: 500,
            queries_executed: 100,
            slices_checked_out: 3,
            dehydrations_completed: 2,
        };
        let bytes = bincode::serialize(&metrics).unwrap();
        let parsed: ServiceMetrics = bincode::deserialize(&bytes).unwrap();
        assert_eq!(parsed.sessions_created, 10);
        assert_eq!(parsed.vertices_appended, 500);
    }

    #[test]
    fn test_session_info_serialization() {
        let info = SessionInfo {
            id: SessionId::now(),
            session_type: SessionType::default(),
            state: SessionState::Active,
            vertex_count: 42,
            created_at: Timestamp::now(),
            description: Some("test".to_string()),
        };
        let bytes = bincode::serialize(&info).unwrap();
        let parsed: SessionInfo = bincode::deserialize(&bytes).unwrap();
        assert_eq!(parsed.id, info.id);
        assert_eq!(parsed.vertex_count, 42);
    }
}
