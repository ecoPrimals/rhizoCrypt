// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2024–2026 ecoPrimals Project

//! RPC client implementation.

use crate::error::{RpcError, RpcResult};
use crate::service::RhizoCryptRpcClient as GeneratedClient;
use crate::service_types::{
    AppendEventRequest, CheckoutSliceRequest, CreateSessionRequest, HealthStatus, QueryRequest,
    ServiceMetrics, SessionInfo,
};
use rhizo_crypt_core::{
    DehydrationStatus, MerkleProof, MerkleRoot, SessionId, SessionTreeHash, Slice, SliceId, Vertex,
    VertexId,
};
use std::net::SocketAddr;
#[cfg(unix)]
use std::path::Path;
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
    /// Connect to a rhizoCrypt RPC server over TCP.
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

    /// Connect to a rhizoCrypt RPC server over a tarpc UDS (G64 C2 dual-socket).
    ///
    /// Uses the same bincode + length-delimited framing as TCP but over a
    /// Unix domain socket for sub-ms intra-gate composition.
    ///
    /// # Errors
    ///
    /// Returns `RpcError::Connection` if the connection fails.
    #[cfg(unix)]
    pub async fn connect_uds(path: impl AsRef<Path>) -> RpcResult<Self> {
        let path = path.as_ref();
        let transport = tarpc::serde_transport::unix::connect(path, Bincode::default)
            .await
            .map_err(|e| RpcError::Connection(e.to_string()))?;

        info!(path = %path.display(), "connected to rhizoCrypt tarpc UDS");

        let inner = GeneratedClient::new(client::Config::default(), transport).spawn();

        Ok(Self {
            inner,
        })
    }

    /// Connect via G65 protocol negotiation on a single socket.
    ///
    /// Sends `PROTOCOLS: tarpc,jsonrpc` and lets the server select the best
    /// protocol. If tarpc is negotiated, serves binary framing on the same
    /// socket. Falls back to JSON-RPC transparently.
    ///
    /// # Errors
    ///
    /// Returns `RpcError::Connection` if the connection or negotiation fails.
    #[cfg(unix)]
    pub async fn connect_negotiated(path: impl AsRef<Path>) -> RpcResult<Self> {
        use crate::protocol_negotiation::{IpcProtocol, negotiate_client};

        let path = path.as_ref();
        let mut stream = tokio::net::UnixStream::connect(path)
            .await
            .map_err(|e| RpcError::Connection(e.to_string()))?;

        let selected = negotiate_client(&mut stream, &[IpcProtocol::Tarpc, IpcProtocol::JsonRpc])
            .await
            .map_err(|e| RpcError::Connection(format!("G65 negotiation failed: {e}")))?;

        info!(
            path = %path.display(),
            protocol = selected.wire_name(),
            "G65 negotiated connection"
        );

        match selected {
            IpcProtocol::Tarpc => {
                let length_delimited =
                    tokio_util::codec::length_delimited::Builder::new().new_framed(stream);
                let transport = tokio_serde::Framed::new(length_delimited, Bincode::default());
                let inner = GeneratedClient::new(client::Config::default(), transport).spawn();
                Ok(Self {
                    inner,
                })
            }
            IpcProtocol::JsonRpc => Err(RpcError::Connection(
                "G65 negotiation selected JSON-RPC — use JSON-RPC client instead".into(),
            )),
        }
    }

    /// Connect via G65 protocol negotiation on any transport (G66).
    ///
    /// Uses [`TransportEndpoint`](rhizo_crypt_core::TransportEndpoint) to connect,
    /// then negotiates protocol on the resulting stream. Platform-agnostic:
    /// works on UDS (Unix), TCP (all platforms), or any future transport.
    ///
    /// # Errors
    ///
    /// Returns `RpcError::Connection` if the connection or negotiation fails.
    pub async fn connect_negotiated_transport(
        endpoint: &rhizo_crypt_core::TransportEndpoint,
    ) -> RpcResult<Self> {
        use crate::protocol_negotiation::{negotiate_client, IpcProtocol};

        let mut stream = rhizo_crypt_core::connect_transport(endpoint)
            .await
            .map_err(|e| RpcError::Connection(e.to_string()))?;

        let selected = negotiate_client(&mut stream, &[IpcProtocol::Tarpc, IpcProtocol::JsonRpc])
            .await
            .map_err(|e| RpcError::Connection(format!("G65 negotiation failed: {e}")))?;

        info!(
            endpoint = %endpoint,
            protocol = selected.wire_name(),
            "G65+G66 negotiated connection"
        );

        match selected {
            IpcProtocol::Tarpc => {
                let length_delimited =
                    tokio_util::codec::length_delimited::Builder::new().new_framed(stream);
                let transport = tokio_serde::Framed::new(length_delimited, Bincode::default());
                let inner = GeneratedClient::new(client::Config::default(), transport).spawn();
                Ok(Self { inner })
            }
            IpcProtocol::JsonRpc => Err(RpcError::Connection(
                "G65 negotiation selected JSON-RPC — use JSON-RPC client instead".into(),
            )),
        }
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

    /// Get the session tree hash (CAC L5 content-addressable cache key).
    ///
    /// # Errors
    ///
    /// Returns `RpcError::Transport` if the RPC call fails.
    pub async fn session_tree_hash(&self, session_id: SessionId) -> RpcResult<SessionTreeHash> {
        self.inner
            .session_tree_hash(context::current(), session_id)
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

    /// Checkout a slice from permanent storage.
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

    /// Resolve a slice (commit back to permanent storage).
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

    /// Trigger dehydration of a session to permanent storage.
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

    /// Dehydrate multiple sessions concurrently in a single call.
    ///
    /// # Errors
    ///
    /// Returns `RpcError::Transport` if the RPC call fails.
    pub async fn dehydrate_batch(
        &self,
        session_ids: Vec<SessionId>,
    ) -> RpcResult<Vec<crate::service_types::BatchDehydrateResult>> {
        self.inner
            .dehydrate_batch(context::current(), session_ids)
            .await
            .map_err(|e| RpcError::Transport(e.to_string()))?
    }

    /// Coordinated pipeline ingest: create session + batch append + optional dehydrate.
    ///
    /// # Errors
    ///
    /// Returns `RpcError::Transport` if the RPC call fails.
    pub async fn pipeline_ingest(
        &self,
        request: crate::service_types::PipelineIngestRequest,
    ) -> RpcResult<crate::service_types::PipelineIngestResponse> {
        self.inner
            .pipeline_ingest(context::current(), request)
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
    pub async fn list_capabilities(
        &self,
    ) -> RpcResult<Vec<crate::service_types::CapabilityDescriptor>> {
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
#[path = "client_tests.rs"]
mod tests;
