// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2024–2026 ecoPrimals Project

//! RPC error types.

use rhizo_crypt_core::RhizoCryptError;
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// RPC-specific errors.
#[derive(Debug, Clone, Error, Serialize, Deserialize)]
pub enum RpcError {
    /// Core library error.
    #[error("core error: {0}")]
    Core(String),

    /// Transport error.
    #[error("transport error: {0}")]
    Transport(String),

    /// Serialization error.
    #[error("serialization error: {0}")]
    Serialization(String),

    /// Connection error.
    #[error("connection error: {0}")]
    Connection(String),

    /// Timeout error.
    #[error("timeout: {0}")]
    Timeout(String),

    /// Server not running.
    #[error("server not running")]
    ServerNotRunning,

    /// Invalid request.
    #[error("invalid request: {0}")]
    InvalidRequest(String),

    /// Session not found.
    #[error("session not found: {0}")]
    SessionNotFound(String),

    /// Vertex not found.
    #[error("vertex not found: {0}")]
    VertexNotFound(String),

    /// Slice not found.
    #[error("slice not found: {0}")]
    SliceNotFound(String),

    /// Internal error.
    #[error("internal error: {0}")]
    Internal(String),
}

impl From<RhizoCryptError> for RpcError {
    fn from(err: RhizoCryptError) -> Self {
        Self::Core(err.to_string())
    }
}

impl From<std::io::Error> for RpcError {
    fn from(err: std::io::Error) -> Self {
        Self::Transport(err.to_string())
    }
}

/// Result type for RPC operations.
pub type RpcResult<T> = Result<T, RpcError>;

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display() {
        let err = RpcError::Core("test error".to_string());
        assert_eq!(err.to_string(), "core error: test error");
    }

    #[test]
    fn test_error_variants() {
        let errors = vec![
            RpcError::Transport("transport failed".to_string()),
            RpcError::Serialization("bad data".to_string()),
            RpcError::Connection("refused".to_string()),
            RpcError::Timeout("30s".to_string()),
            RpcError::ServerNotRunning,
            RpcError::InvalidRequest("missing field".to_string()),
            RpcError::SessionNotFound("abc123".to_string()),
            RpcError::VertexNotFound("def456".to_string()),
            RpcError::SliceNotFound("ghi789".to_string()),
            RpcError::Internal("unexpected".to_string()),
        ];

        for err in errors {
            // Just ensure they all format correctly
            assert!(!err.to_string().is_empty());
        }
    }

    #[test]
    fn test_from_rhizocrypt_error() {
        let core_err = RhizoCryptError::config("bad config");
        let rpc_err: RpcError = core_err.into();
        assert!(rpc_err.to_string().contains("configuration"));
    }

    #[test]
    fn test_from_io_error() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
        let rpc_err: RpcError = io_err.into();
        assert!(rpc_err.to_string().contains("file not found"));
    }
}
