// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2024–2026 ecoPrimals Project

//! Wire types for `LoamSpine` JSON-RPC communication.
//!
//! Contains the JSON-RPC 2.0 protocol types, method negotiation state,
//! and `LoamSpine`-specific request/response DTOs. Extracted from the
//! client module to keep both files under the 700-line threshold.

use crate::error::RhizoCryptError;
use serde::{Deserialize, Serialize};

/// JSON-RPC error code for "method not found" per JSON-RPC 2.0 spec.
pub const METHOD_NOT_FOUND_CODE: i32 = -32601;

// ============================================================================
// Method Negotiation
// ============================================================================

/// Tracks whether the server supports native or compat method names.
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum MethodSupport {
    Unknown = 0,
    Native = 1,
    Compat = 2,
}

impl MethodSupport {
    pub(super) const fn from_u8(v: u8) -> Self {
        match v {
            1 => Self::Native,
            2 => Self::Compat,
            _ => Self::Unknown,
        }
    }

    pub(super) const fn to_u8(self) -> u8 {
        match self {
            Self::Unknown => 0,
            Self::Native => 1,
            Self::Compat => 2,
        }
    }
}

/// Distinguishes "method not found" from other errors during negotiation.
#[derive(Debug)]
pub(super) enum NegotiableError {
    MethodNotFound,
    Other(RhizoCryptError),
}

impl NegotiableError {
    pub(super) fn into_rhizo_error(self) -> RhizoCryptError {
        match self {
            Self::MethodNotFound => {
                RhizoCryptError::integration("JSON-RPC method not found on server")
            }
            Self::Other(e) => e,
        }
    }
}

// ============================================================================
// JSON-RPC 2.0 Types
// ============================================================================

#[derive(Debug, Serialize)]
pub(super) struct JsonRpcRequest<'a, T> {
    pub jsonrpc: &'static str,
    pub method: &'a str,
    pub params: T,
    pub id: u64,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub(super) enum JsonRpcResponse<T> {
    Success {
        jsonrpc: String,
        result: T,
        id: u64,
    },
    Error {
        jsonrpc: String,
        error: JsonRpcError,
        id: u64,
    },
}

impl<T> JsonRpcResponse<T> {
    /// Validate JSON-RPC 2.0 protocol conformance and extract the result.
    ///
    /// Checks that the response version is "2.0" and the response ID matches
    /// the request ID, per the JSON-RPC 2.0 specification.
    pub(super) fn into_result(self, expected_id: u64) -> std::result::Result<T, NegotiableError> {
        match self {
            Self::Success {
                jsonrpc,
                result,
                id,
            } => {
                Self::validate_protocol(&jsonrpc, id, expected_id)?;
                Ok(result)
            }
            Self::Error {
                jsonrpc,
                error,
                id,
            } => {
                Self::validate_protocol(&jsonrpc, id, expected_id)?;
                if error.code == METHOD_NOT_FOUND_CODE {
                    return Err(NegotiableError::MethodNotFound);
                }
                let detail =
                    error.data.as_ref().map(|d| format!(" (data: {d})")).unwrap_or_default();
                Err(NegotiableError::Other(RhizoCryptError::integration(format!(
                    "Permanent storage RPC error [{}]: {}{detail}",
                    error.code, error.message
                ))))
            }
        }
    }

    pub(super) fn validate_protocol(
        jsonrpc: &str,
        id: u64,
        expected_id: u64,
    ) -> std::result::Result<(), NegotiableError> {
        if jsonrpc != crate::constants::JSONRPC_VERSION {
            return Err(NegotiableError::Other(RhizoCryptError::integration(format!(
                "Invalid JSON-RPC version: expected \"{}\", got \"{jsonrpc}\"",
                crate::constants::JSONRPC_VERSION
            ))));
        }
        if id != expected_id {
            return Err(NegotiableError::Other(RhizoCryptError::integration(format!(
                "JSON-RPC response ID mismatch: expected {expected_id}, got {id}"
            ))));
        }
        Ok(())
    }
}

#[derive(Debug, Deserialize)]
pub struct JsonRpcError {
    pub code: i32,
    pub message: String,
    #[serde(default)]
    pub data: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize)]
pub(super) struct EmptyParams {}

// ============================================================================
// LoamSpine API Types
// ============================================================================

#[derive(Debug, Clone, Serialize)]
pub(super) struct CommitSessionRequest {
    pub session_id: String,
    pub merkle_root: String,
    pub summary: RpcDehydrationSummary,
    pub committer_did: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub(super) struct RpcDehydrationSummary {
    pub session_type: String,
    pub vertex_count: u64,
    pub leaf_count: u64,
    pub started_at: u64,
    pub ended_at: u64,
    pub outcome: String,
}

#[derive(Debug, Deserialize)]
pub(super) struct CommitSessionResponse {
    pub accepted: bool,
    pub commit_id: Option<String>,
    pub spine_entry_hash: Option<String>,
    pub entry_index: Option<u64>,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub(super) struct CheckoutSliceRequest {
    pub spine_id: String,
    pub entry_hash: String,
    pub holder_did: String,
}

#[derive(Debug, Deserialize)]
pub(super) struct CheckoutSliceResponse {
    pub spine_id: String,
    pub entry_index: u64,
    pub certificate_id: Option<String>,
    pub owner_did: String,
}

#[derive(Debug, Deserialize)]
pub(super) struct HealthCheckResponse {
    pub status: String,
    pub version: String,
    pub spine_count: u64,
}

impl HealthCheckResponse {
    pub(super) fn is_healthy(&self) -> bool {
        self.status == "ok" || self.status == "healthy"
    }
}
