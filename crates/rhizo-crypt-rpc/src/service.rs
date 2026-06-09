// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2024–2026 ecoPrimals Project

//! RPC service trait definition using tarpc.
//!
//! This is the core RPC interface for rhizoCrypt. The `#[tarpc::service]` macro
//! generates both client and server code from this trait, ensuring compile-time
//! type safety across the network boundary.

use crate::error::RpcError;
use crate::service_types::{
    AppendEventRequest, BranchRequest, BranchResponse, CapabilityDescriptor, CheckoutSliceRequest,
    CreateSessionRequest, DiffRequest, DiffResponse, FederateRequest, FederateResponse,
    HealthStatus, MergeRequest, PartialDehydrateResponse, QueryRequest, ServiceMetrics,
    SessionInfo, cached_capability_descriptors,
};
use rhizo_crypt_core::{
    MerkleProof, MerkleRoot, PayloadRef, Session, SessionBuilder, SessionId, SliceId, Vertex,
    VertexId,
};
use std::sync::Arc;

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

    /// Checkout a slice from permanent storage.
    async fn checkout_slice(request: CheckoutSliceRequest) -> Result<SliceId, RpcError>;

    /// Get slice info.
    async fn get_slice(slice_id: SliceId) -> Result<rhizo_crypt_core::Slice, RpcError>;

    /// List active slices.
    async fn list_slices() -> Result<Vec<rhizo_crypt_core::Slice>, RpcError>;

    /// Resolve a slice (commit back to permanent storage).
    async fn resolve_slice(slice_id: SliceId, session_id: SessionId) -> Result<(), RpcError>;

    // ========================================================================
    // Branch / Diff / Merge / Federate (Wave 60)
    // ========================================================================

    /// Create a new session branched from a parent at a checkout vertex.
    async fn branch_session(request: BranchRequest) -> Result<BranchResponse, RpcError>;

    /// Compute the structural diff between two sessions.
    async fn diff_sessions(request: DiffRequest) -> Result<DiffResponse, RpcError>;

    /// Create a merge vertex joining multiple frontier tips.
    async fn merge_branches(request: MergeRequest) -> Result<VertexId, RpcError>;

    /// Import vertices from a remote peer (diff-based federation).
    async fn federate(request: FederateRequest) -> Result<FederateResponse, RpcError>;

    // ========================================================================
    // Dehydration Operations
    // ========================================================================

    /// Compute a Merkle root of current vertices without closing the session.
    ///
    /// If `vertex_ids` is non-empty, only those vertices are included;
    /// otherwise all vertices in the session are covered. The session
    /// remains open — this is a read-only operation.
    async fn partial_dehydrate(
        session_id: SessionId,
        vertex_ids: Vec<VertexId>,
    ) -> Result<PartialDehydrateResponse, RpcError>;

    /// Trigger dehydration of a session to permanent storage.
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

    // ========================================================================
    // Capability Discovery (Spring-as-Niche Standard)
    // ========================================================================

    /// List capabilities this primal provides.
    async fn list_capabilities() -> Result<Vec<CapabilityDescriptor>, RpcError>;
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
        agents: session.agents.iter().cloned().collect(),
        genesis: session.genesis.iter().copied().collect(),
        frontier: session.frontier.iter().copied().collect(),
    }
}

/// Server implementation wrapper.
///
/// Implements `RhizoCryptRpc` by delegating to a `RhizoCrypt` primal instance.
#[derive(Clone)]
pub struct RhizoCryptRpcServer {
    pub(crate) primal: Arc<rhizo_crypt_core::RhizoCrypt>,
    pub(crate) start_time: std::time::Instant,
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

/// Attempt to cryptographically sign a vertex via the discovered signing provider.
///
/// When a signing provider (e.g. `BearDog`) is available and the vertex carries an
/// `agent` DID, the vertex's canonical bytes are signed with Ed25519 and the
/// resulting signature is attached. This makes DAG integrity independently
/// verifiable by any party holding the agent's public key.
///
/// Parse a `payload_ref` string into a [`PayloadRef`].
///
/// If the string is a 64-char hex hash, it is decoded as a content hash
/// (size unknown = 0). Otherwise the reference string itself is hashed
/// (Blake3) so the vertex carries a deterministic content-addressed
/// reference regardless of the URI scheme.
pub fn parse_payload_ref(s: &str) -> Option<PayloadRef> {
    if s.is_empty() {
        return None;
    }
    if s.len() == 64
        && let Ok(bytes) = hex::decode(s)
    {
        return Some(PayloadRef::from_hash(&bytes));
    }
    Some(PayloadRef::from_bytes(s.as_bytes()))
}

/// Gracefully degrades: if no provider is discovered or signing fails, the vertex
/// remains unsigned (matching standalone / pre-composition behavior).
pub async fn sign_vertex_if_available(primal: &rhizo_crypt_core::RhizoCrypt, vertex: &mut Vertex) {
    let Some(agent) = vertex.agent.clone() else {
        return;
    };

    let Some(client) = primal.signing_client().await else {
        return;
    };

    match client.sign_vertex(vertex, &agent).await {
        Ok(sig) => {
            tracing::trace!(agent = %agent, "Vertex signed via delegated crypto provider");
            vertex.signature = Some(sig);
        }
        Err(e) => {
            tracing::warn!(
                error = %e,
                agent = %agent,
                "Failed to sign vertex — continuing unsigned"
            );
        }
    }
}

impl RhizoCryptRpc for RhizoCryptRpcServer {
    // Session operations (thin delegation — kept inline)

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

    // Vertex / event operations (delegated to service_vertex_ops)

    async fn append_event(
        self,
        _: tarpc::context::Context,
        request: AppendEventRequest,
    ) -> Result<VertexId, RpcError> {
        self.impl_append_event(request).await
    }

    async fn append_batch(
        self,
        _: tarpc::context::Context,
        requests: Vec<AppendEventRequest>,
    ) -> Result<Vec<VertexId>, RpcError> {
        self.impl_append_batch(requests).await
    }

    async fn get_vertex(
        self,
        _: tarpc::context::Context,
        session_id: SessionId,
        vertex_id: VertexId,
    ) -> Result<Vertex, RpcError> {
        self.impl_get_vertex(session_id, vertex_id).await
    }

    async fn get_frontier(
        self,
        _: tarpc::context::Context,
        session_id: SessionId,
    ) -> Result<Vec<VertexId>, RpcError> {
        self.impl_get_frontier(session_id).await
    }

    async fn get_genesis(
        self,
        _: tarpc::context::Context,
        session_id: SessionId,
    ) -> Result<Vec<VertexId>, RpcError> {
        self.impl_get_genesis(session_id).await
    }

    async fn query_vertices(
        self,
        _: tarpc::context::Context,
        request: QueryRequest,
    ) -> Result<Vec<Vertex>, RpcError> {
        self.impl_query_vertices(request).await
    }

    async fn get_children(
        self,
        _: tarpc::context::Context,
        session_id: SessionId,
        vertex_id: VertexId,
    ) -> Result<Vec<VertexId>, RpcError> {
        self.impl_get_children(session_id, vertex_id).await
    }

    // Merkle operations (delegated to service_vertex_ops)

    async fn get_merkle_root(
        self,
        _: tarpc::context::Context,
        session_id: SessionId,
    ) -> Result<MerkleRoot, RpcError> {
        self.impl_get_merkle_root(session_id).await
    }

    async fn get_merkle_proof(
        self,
        _: tarpc::context::Context,
        session_id: SessionId,
        vertex_id: VertexId,
    ) -> Result<MerkleProof, RpcError> {
        self.impl_get_merkle_proof(session_id, vertex_id).await
    }

    async fn verify_proof(
        self,
        _: tarpc::context::Context,
        _root: MerkleRoot,
        proof: MerkleProof,
    ) -> Result<bool, RpcError> {
        self.impl_verify_proof(proof).await
    }

    // Slice operations (thin delegation — kept inline)

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

    // Branch / diff / merge / federate (delegated to service_branch_ops)

    async fn branch_session(
        self,
        _: tarpc::context::Context,
        request: BranchRequest,
    ) -> Result<BranchResponse, RpcError> {
        self.impl_branch_session(request).await
    }

    async fn diff_sessions(
        self,
        _: tarpc::context::Context,
        request: DiffRequest,
    ) -> Result<DiffResponse, RpcError> {
        self.impl_diff_sessions(request).await
    }

    async fn merge_branches(
        self,
        _: tarpc::context::Context,
        request: MergeRequest,
    ) -> Result<VertexId, RpcError> {
        self.impl_merge_branches(request).await
    }

    async fn federate(
        self,
        _: tarpc::context::Context,
        request: FederateRequest,
    ) -> Result<FederateResponse, RpcError> {
        self.impl_federate(request).await
    }

    // Dehydration operations (kept inline — moderate size)

    async fn partial_dehydrate(
        self,
        _: tarpc::context::Context,
        session_id: SessionId,
        vertex_ids: Vec<VertexId>,
    ) -> Result<PartialDehydrateResponse, RpcError> {
        let session = self.primal.get_session(session_id).map_err(RpcError::from)?;
        let dag_store = self.primal.dag_store().await.map_err(RpcError::from)?;
        let all_vertices = dag_store.get_all_vertices(session_id).await.map_err(RpcError::from)?;

        let total = all_vertices.len() as u64;

        let (selected, sealed_count) = if vertex_ids.is_empty() {
            let root = MerkleRoot::compute(&all_vertices).map_err(RpcError::from)?;
            (root, total)
        } else {
            let subset: Vec<_> = all_vertices
                .into_iter()
                .filter(|v| v.cached_id().is_some_and(|id| vertex_ids.contains(&id)))
                .collect();
            let count = subset.len() as u64;
            let root = MerkleRoot::compute(&subset).map_err(RpcError::from)?;
            (root, count)
        };

        Ok(PartialDehydrateResponse {
            merkle_root: hex::encode(selected.0),
            sealed_count,
            open_count: total.saturating_sub(sealed_count),
            session_open: session.state.is_active(),
        })
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

    // Health / metrics / capabilities (kept inline — small)

    async fn health(self, _: tarpc::context::Context) -> Result<HealthStatus, RpcError> {
        use rhizo_crypt_core::PrimalLifecycle;

        let state = self.primal.state();
        let session_count = self.primal.session_count();
        let vertex_count = self.primal.total_vertex_count();

        Ok(HealthStatus {
            healthy: state.is_running(),
            state: format!("{state}"),
            active_sessions: u64::try_from(session_count).unwrap_or(u64::MAX),
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

    async fn list_capabilities(
        self,
        _: tarpc::context::Context,
    ) -> Result<Vec<CapabilityDescriptor>, RpcError> {
        Ok(cached_capability_descriptors().to_vec())
    }
}

#[cfg(test)]
#[expect(clippy::unwrap_used, clippy::expect_used, reason = "test code")]
#[path = "service_tests.rs"]
mod tests;
