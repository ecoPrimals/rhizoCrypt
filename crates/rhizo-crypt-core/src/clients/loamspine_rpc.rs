// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2024–2026 ecoPrimals Project

//! `LoamSpine` tarpc RPC types and client.
//!
//! This module contains the tarpc service definition and related types
//! for connecting to the `LoamSpine` permanent storage service.
//!
//! ## Feature Gate
//!
//! This module is only compiled when the `live-clients` feature is enabled.
//! Without the feature, the `LoamSpine` client operates in scaffolded mode.

use serde::{Deserialize, Serialize};

// ============================================================================
// tarpc Service Definition (mirrors the ledger primal's RPC interface)
// ============================================================================

/// tarpc service trait for `LoamSpine` commit operations.
///
/// This trait provides the subset of `LoamSpine` RPC used by `rhizoCrypt`
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

#[cfg(test)]
#[expect(clippy::unwrap_used, reason = "test code")]
mod tests {
    use super::*;

    #[test]
    fn test_commit_request_roundtrip() {
        let request = RpcCommitSessionRequest {
            session_id: "session-001".to_string(),
            merkle_root: "abc123".to_string(),
            summary: RpcDehydrationSummary {
                session_type: "general".to_string(),
                vertex_count: 42,
                leaf_count: 10,
                started_at: 1_000_000,
                ended_at: 2_000_000,
                outcome: "completed".to_string(),
            },
            committer_did: "did:key:test".to_string(),
        };

        let json = serde_json::to_string(&request).unwrap();
        let decoded: RpcCommitSessionRequest = serde_json::from_str(&json).unwrap();
        assert_eq!(decoded.session_id, "session-001");
        assert_eq!(decoded.summary.vertex_count, 42);
        assert_eq!(decoded.committer_did, "did:key:test");
    }

    #[test]
    fn test_commit_response_success() {
        let response = RpcCommitSessionResponse {
            accepted: true,
            commit_id: Some("commit-001".to_string()),
            spine_entry_hash: Some("hash-abc".to_string()),
            error: None,
        };

        let json = serde_json::to_string(&response).unwrap();
        let decoded: RpcCommitSessionResponse = serde_json::from_str(&json).unwrap();
        assert!(decoded.accepted);
        assert_eq!(decoded.commit_id.as_deref(), Some("commit-001"));
        assert!(decoded.error.is_none());
    }

    #[test]
    fn test_commit_response_error() {
        let response = RpcCommitSessionResponse {
            accepted: false,
            commit_id: None,
            spine_entry_hash: None,
            error: Some("Session already committed".to_string()),
        };

        let json = serde_json::to_string(&response).unwrap();
        let decoded: RpcCommitSessionResponse = serde_json::from_str(&json).unwrap();
        assert!(!decoded.accepted);
        assert!(decoded.commit_id.is_none());
        assert!(decoded.error.is_some());
    }

    #[test]
    fn test_commit_status_roundtrip() {
        let response = RpcCommitStatusResponse {
            commit_id: "commit-001".to_string(),
            status: "committed".to_string(),
            spine_entry_hash: Some("hash-def".to_string()),
            error: None,
        };

        let json = serde_json::to_string(&response).unwrap();
        let decoded: RpcCommitStatusResponse = serde_json::from_str(&json).unwrap();
        assert_eq!(decoded.status, "committed");
    }

    #[test]
    fn test_health_response_roundtrip() {
        let response = RpcHealthResponse {
            status: "ok".to_string(),
            version: "0.14.0".to_string(),
            spine_count: 7,
        };

        let json = serde_json::to_string(&response).unwrap();
        let decoded: RpcHealthResponse = serde_json::from_str(&json).unwrap();
        assert_eq!(decoded.status, "ok");
        assert_eq!(decoded.spine_count, 7);
    }
}
