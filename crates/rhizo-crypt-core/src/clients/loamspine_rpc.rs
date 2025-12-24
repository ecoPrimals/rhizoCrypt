//! LoamSpine tarpc RPC types and client.
//!
//! This module contains the tarpc service definition and related types
//! for connecting to the LoamSpine permanent storage service.
//!
//! ## Feature Gate
//!
//! This module is only compiled when the `live-clients` feature is enabled.
//! Without the feature, the LoamSpine client operates in scaffolded mode.

use serde::{Deserialize, Serialize};

// ============================================================================
// tarpc Service Definition (mirrors LoamSpine's RPC interface)
// ============================================================================

/// tarpc service trait for LoamSpine commit operations.
///
/// This trait provides the subset of LoamSpine RPC used by rhizoCrypt
/// for dehydration (session commit) operations.
#[tarpc::service]
pub trait LoamSpineRpc {
    /// Commit a rhizoCrypt session to permanent storage.
    async fn commit_session(request: RpcCommitSessionRequest) -> RpcCommitSessionResponse;

    /// Get the status of a pending commit.
    async fn get_commit_status(commit_id: String) -> RpcCommitStatusResponse;

    /// Health check.
    async fn health_check() -> RpcHealthResponse;
}

// ============================================================================
// RPC Types
// ============================================================================

/// Session commit request.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RpcCommitSessionRequest {
    /// Session ID to commit.
    pub session_id: String,
    /// Merkle root of the session DAG.
    pub merkle_root: String,
    /// Session summary for dehydration.
    pub summary: RpcDehydrationSummary,
    /// Committer DID.
    pub committer_did: String,
}

/// Dehydration summary.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RpcDehydrationSummary {
    /// Session type.
    pub session_type: String,
    /// Total vertex count.
    pub vertex_count: u64,
    /// Leaf vertex count.
    pub leaf_count: u64,
    /// Start timestamp (nanos).
    pub started_at: u64,
    /// End timestamp (nanos).
    pub ended_at: u64,
    /// Session outcome.
    pub outcome: String,
}

/// Session commit response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RpcCommitSessionResponse {
    /// Whether commit was accepted.
    pub accepted: bool,
    /// Commit ID for tracking.
    pub commit_id: Option<String>,
    /// Spine entry hash if committed.
    pub spine_entry_hash: Option<String>,
    /// Error message if rejected.
    pub error: Option<String>,
}

/// Commit status response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RpcCommitStatusResponse {
    /// Commit ID.
    pub commit_id: String,
    /// Status: pending, committed, failed.
    pub status: String,
    /// Spine entry hash if committed.
    pub spine_entry_hash: Option<String>,
    /// Error message if failed.
    pub error: Option<String>,
}

/// Health response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RpcHealthResponse {
    /// Service status.
    pub status: String,
    /// Service version.
    pub version: String,
    /// Spine count.
    pub spine_count: u64,
}
