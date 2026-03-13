// SPDX-License-Identifier: AGPL-3.0-only
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
pub struct RpcClient {
    inner: GeneratedClient,
}

impl RpcClient {
    /// Connect to a rhizoCrypt RPC server.
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
    pub async fn create_session(&self, request: CreateSessionRequest) -> RpcResult<SessionId> {
        self.inner
            .create_session(context::current(), request)
            .await
            .map_err(|e| RpcError::Transport(e.to_string()))?
    }

    /// Get session info.
    pub async fn get_session(&self, session_id: SessionId) -> RpcResult<SessionInfo> {
        self.inner
            .get_session(context::current(), session_id)
            .await
            .map_err(|e| RpcError::Transport(e.to_string()))?
    }

    /// List all active sessions.
    pub async fn list_sessions(&self) -> RpcResult<Vec<SessionInfo>> {
        self.inner
            .list_sessions(context::current())
            .await
            .map_err(|e| RpcError::Transport(e.to_string()))?
    }

    /// Discard a session.
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
    pub async fn append_event(&self, request: AppendEventRequest) -> RpcResult<VertexId> {
        self.inner
            .append_event(context::current(), request)
            .await
            .map_err(|e| RpcError::Transport(e.to_string()))?
    }

    /// Append multiple events in a batch.
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
    pub async fn get_frontier(&self, session_id: SessionId) -> RpcResult<Vec<VertexId>> {
        self.inner
            .get_frontier(context::current(), session_id)
            .await
            .map_err(|e| RpcError::Transport(e.to_string()))?
    }

    /// Get genesis vertices (DAG roots).
    pub async fn get_genesis(&self, session_id: SessionId) -> RpcResult<Vec<VertexId>> {
        self.inner
            .get_genesis(context::current(), session_id)
            .await
            .map_err(|e| RpcError::Transport(e.to_string()))?
    }

    /// Query vertices with filters.
    pub async fn query_vertices(&self, request: QueryRequest) -> RpcResult<Vec<Vertex>> {
        self.inner
            .query_vertices(context::current(), request)
            .await
            .map_err(|e| RpcError::Transport(e.to_string()))?
    }

    /// Get children of a vertex.
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
    pub async fn get_merkle_root(&self, session_id: SessionId) -> RpcResult<MerkleRoot> {
        self.inner
            .get_merkle_root(context::current(), session_id)
            .await
            .map_err(|e| RpcError::Transport(e.to_string()))?
    }

    /// Generate inclusion proof for a vertex.
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
    pub async fn verify_proof(&self, root: MerkleRoot, proof: MerkleProof) -> RpcResult<bool> {
        self.inner
            .verify_proof(context::current(), root, proof)
            .await
            .map_err(|e| RpcError::Transport(e.to_string()))?
    }

    // ========================================================================
    // Slice Operations
    // ========================================================================

    /// Checkout a slice from LoamSpine.
    pub async fn checkout_slice(&self, request: CheckoutSliceRequest) -> RpcResult<SliceId> {
        self.inner
            .checkout_slice(context::current(), request)
            .await
            .map_err(|e| RpcError::Transport(e.to_string()))?
    }

    /// Get slice info.
    pub async fn get_slice(&self, slice_id: SliceId) -> RpcResult<Slice> {
        self.inner
            .get_slice(context::current(), slice_id)
            .await
            .map_err(|e| RpcError::Transport(e.to_string()))?
    }

    /// List active slices.
    pub async fn list_slices(&self) -> RpcResult<Vec<Slice>> {
        self.inner
            .list_slices(context::current())
            .await
            .map_err(|e| RpcError::Transport(e.to_string()))?
    }

    /// Resolve a slice (commit back to LoamSpine).
    pub async fn resolve_slice(&self, slice_id: SliceId, session_id: SessionId) -> RpcResult<()> {
        self.inner
            .resolve_slice(context::current(), slice_id, session_id)
            .await
            .map_err(|e| RpcError::Transport(e.to_string()))?
    }

    // ========================================================================
    // Dehydration Operations
    // ========================================================================

    /// Trigger dehydration of a session to LoamSpine.
    pub async fn dehydrate(&self, session_id: SessionId) -> RpcResult<MerkleRoot> {
        self.inner
            .dehydrate(context::current(), session_id)
            .await
            .map_err(|e| RpcError::Transport(e.to_string()))?
    }

    /// Get dehydration status.
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
    pub async fn health(&self) -> RpcResult<HealthStatus> {
        self.inner
            .health(context::current())
            .await
            .map_err(|e| RpcError::Transport(e.to_string()))?
    }

    /// Get service metrics.
    pub async fn metrics(&self) -> RpcResult<ServiceMetrics> {
        self.inner
            .metrics(context::current())
            .await
            .map_err(|e| RpcError::Transport(e.to_string()))?
    }
}
