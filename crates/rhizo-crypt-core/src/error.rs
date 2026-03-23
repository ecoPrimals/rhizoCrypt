// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2024–2026 ecoPrimals Project

//! RhizoCrypt error types.
//!
//! This module defines all error types used throughout the DAG engine.

use crate::types::{SessionId, VertexId};
use std::fmt;
use thiserror::Error;

/// Main error type for RhizoCrypt operations.
#[derive(Debug, Error)]
pub enum RhizoCryptError {
    // === Configuration Errors ===
    /// Configuration error.
    #[error("configuration error: {0}")]
    Config(String),

    /// Invalid configuration value.
    #[error("invalid configuration value for '{key}': {reason}")]
    InvalidConfig {
        /// Configuration key.
        key: String,
        /// Reason for invalidity.
        reason: String,
    },

    // === Session Errors ===
    /// Session not found.
    #[error("session not found: {0}")]
    SessionNotFound(SessionId),

    /// Session already exists.
    #[error("session already exists: {0}")]
    SessionExists(SessionId),

    /// Session is not active.
    #[error("session {session_id} is not active: {state}")]
    SessionNotActive {
        /// Session ID.
        session_id: SessionId,
        /// Current state.
        state: String,
    },

    /// Session limit exceeded.
    #[error("session {session_id} exceeded {limit}: {value}")]
    SessionLimitExceeded {
        /// Session ID.
        session_id: SessionId,
        /// Limit type.
        limit: String,
        /// Current value.
        value: u64,
    },

    // === Vertex Errors ===
    /// Vertex not found.
    #[error("vertex not found: {0}")]
    VertexNotFound(VertexId),

    /// Invalid vertex structure.
    #[error("invalid vertex: {0}")]
    InvalidVertex(String),

    /// Parent vertex not found.
    #[error("parent vertex not found: {0}")]
    ParentNotFound(VertexId),

    /// Vertex hash mismatch.
    #[error("vertex hash mismatch: expected {expected}, got {actual}")]
    HashMismatch {
        /// Expected hash.
        expected: String,
        /// Actual hash.
        actual: String,
    },

    // === Signature Errors ===
    /// Signature required but missing.
    #[error("signature required for event type: {0}")]
    SignatureRequired(String),

    /// Invalid signature.
    #[error("invalid signature: {0}")]
    InvalidSignature(String),

    // === Storage Errors ===
    /// Storage operation failed.
    #[error("storage error: {0}")]
    Storage(String),

    /// Serialization failed.
    #[error("serialization error: {0}")]
    Serialization(String),

    /// Deserialization failed.
    #[error("deserialization error: {0}")]
    Deserialization(String),

    // === Merkle Errors ===
    /// Invalid Merkle proof.
    #[error("invalid Merkle proof: {0}")]
    InvalidProof(String),

    /// Merkle root mismatch.
    #[error("Merkle root mismatch: expected {expected}, got {actual}")]
    RootMismatch {
        /// Expected root.
        expected: String,
        /// Actual root.
        actual: String,
    },

    // === Slice Errors ===
    /// Slice not found.
    #[error("slice not found: {0}")]
    SliceNotFound(String),

    /// Invalid slice operation.
    #[error("invalid slice operation: {0}")]
    InvalidSliceOperation(String),

    /// Slice already resolved.
    #[error("slice already resolved: {0}")]
    SliceAlreadyResolved(String),

    /// Slice has expired.
    #[error("slice has expired: {slice_id}")]
    SliceExpired {
        /// Slice ID.
        slice_id: String,
    },

    /// Slice mode not allowed for operation.
    #[error("slice mode '{mode}' does not allow {operation}")]
    SliceModeNotAllowed {
        /// The slice mode.
        mode: String,
        /// The attempted operation.
        operation: String,
    },

    /// Re-slicing not allowed.
    #[error("re-slicing not allowed for slice: {0}")]
    ResliceNotAllowed(String),

    // === Dehydration Errors ===
    /// Dehydration failed.
    #[error("dehydration failed: {0}")]
    DehydrationFailed(String),

    /// Missing required attestation.
    #[error("missing required attestation from: {attester}")]
    MissingAttestation {
        /// Expected attester DID.
        attester: String,
    },

    /// Attestation verification failed.
    #[error("attestation verification failed: {0}")]
    AttestationVerificationFailed(String),

    /// Commit already exists.
    #[error("commit already exists: {0}")]
    CommitExists(String),

    // === Integration Errors ===
    /// Capability provider error (signing, storage, commit, etc.).
    ///
    /// Capability-based: rhizoCrypt only knows about capabilities it discovers
    /// at runtime, never specific primal names.
    #[error("capability provider error ({capability}): {message}")]
    CapabilityProvider {
        /// The capability that failed (e.g., "signing", "permanent_storage").
        capability: String,
        /// Error detail.
        message: String,
    },

    /// Integration error (service not discovered or unavailable).
    #[error("integration error: {0}")]
    Integration(String),

    // === IPC Errors (structured, absorbed from healthSpring V28) ===
    /// Unix socket IPC error with structured phase context.
    ///
    /// Provides observability into which phase of the IPC call failed,
    /// enabling targeted retries and diagnostics without a logging dependency.
    #[error("IPC error ({phase}): {message}")]
    Ipc {
        /// Phase of the IPC call that failed.
        phase: IpcErrorPhase,
        /// Human-readable error detail.
        message: String,
    },

    // === Internal Errors ===
    /// Internal error.
    #[error("internal error: {0}")]
    Internal(String),

    /// Operation timed out.
    #[error("operation timed out after {0} ms")]
    Timeout(u64),

    /// Operation was cancelled.
    #[error("operation was cancelled")]
    Cancelled,
}

/// Structured phase for IPC errors.
///
/// Absorbed from healthSpring V28 `SendError` pattern. Each variant identifies
/// the exact point of failure in the Unix socket IPC lifecycle, enabling
/// targeted retry strategies and structured observability.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum IpcErrorPhase {
    /// Socket connection failed (primal unreachable or socket missing).
    Connect,
    /// Request write to socket failed (broken pipe, timeout).
    Write,
    /// Response read from socket failed (timeout, truncated).
    Read,
    /// Response is not valid JSON.
    InvalidJson,
    /// HTTP response status was not 2xx.
    HttpStatus(u16),
    /// Response lacks a `result` field (JSON-RPC protocol violation).
    NoResult,
    /// JSON-RPC error object returned by the remote primal.
    JsonRpcError(i64),
}

impl fmt::Display for IpcErrorPhase {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Connect => write!(f, "connect"),
            Self::Write => write!(f, "write"),
            Self::Read => write!(f, "read"),
            Self::InvalidJson => write!(f, "invalid_json"),
            Self::HttpStatus(code) => write!(f, "http_{code}"),
            Self::NoResult => write!(f, "no_result"),
            Self::JsonRpcError(code) => write!(f, "jsonrpc_{code}"),
        }
    }
}

impl IpcErrorPhase {
    /// Returns `true` for JSON-RPC "method not found" (-32601).
    #[must_use]
    pub const fn is_method_not_found(&self) -> bool {
        matches!(self, Self::JsonRpcError(-32601))
    }

    /// Returns `true` for transport-level timeouts (Connect or Read phase).
    #[must_use]
    pub const fn is_timeout_likely(&self) -> bool {
        matches!(self, Self::Connect | Self::Read)
    }

    /// Returns `true` when automatic retry is reasonable.
    ///
    /// Retriable: Connect (transient socket), Write (broken pipe), Read (timeout).
    /// Not retriable: InvalidJson, NoResult, HttpStatus, JsonRpcError (application-level).
    #[must_use]
    pub const fn is_retriable(&self) -> bool {
        matches!(self, Self::Connect | Self::Write | Self::Read)
    }

    /// Returns `true` for phases caused by a well-formed but failed JSON-RPC call.
    #[must_use]
    pub const fn is_application_error(&self) -> bool {
        matches!(self, Self::JsonRpcError(_) | Self::NoResult)
    }
}

/// Outcome of a dispatched JSON-RPC call, separating protocol errors
/// from application results.
///
/// Absorbed from airSpring / biomeOS dispatch patterns. Protocol errors
/// (transport failures, malformed responses) are fundamentally different
/// from application errors (method returned an error object). Callers can
/// pattern-match to decide retry strategy, logging, or escalation.
#[derive(Debug)]
pub enum DispatchOutcome<T> {
    /// The call succeeded and returned a result.
    Ok(T),
    /// The remote primal returned a JSON-RPC error object.
    ApplicationError {
        /// JSON-RPC error code.
        code: i64,
        /// Human-readable error message.
        message: String,
    },
    /// A transport or protocol-level failure occurred.
    ProtocolError(RhizoCryptError),
}

impl<T> DispatchOutcome<T> {
    /// Returns `true` if the outcome is a successful result.
    #[must_use]
    pub const fn is_ok(&self) -> bool {
        matches!(self, Self::Ok(_))
    }

    /// Convert into a `Result`, folding both error variants into `RhizoCryptError`.
    ///
    /// # Errors
    ///
    /// Returns [`RhizoCryptError::ipc`] with the JSON-RPC error code for application
    /// failures, or the original [`RhizoCryptError`] for protocol/transport failures.
    pub fn into_result(self) -> Result<T> {
        match self {
            Self::Ok(val) => Ok(val),
            Self::ApplicationError {
                code,
                message,
            } => Err(RhizoCryptError::ipc(IpcErrorPhase::JsonRpcError(code), message)),
            Self::ProtocolError(e) => Err(e),
        }
    }
}

/// Extracts `(code, message)` from a JSON-RPC error object.
///
/// Centralizes the pattern used by every IPC adapter to parse the `error`
/// field from a JSON-RPC 2.0 response. Returns `None` if no error is present.
#[must_use]
pub fn extract_rpc_error(response: &serde_json::Value) -> Option<(i64, String)> {
    let error = response.get("error")?;
    let code = error.get("code").and_then(serde_json::Value::as_i64).unwrap_or(-1);
    let message = error
        .get("message")
        .and_then(serde_json::Value::as_str)
        .unwrap_or("Unknown error")
        .to_string();
    Some((code, message))
}

/// Extension trait for `Result<T, E>` that exits the process cleanly on error.
///
/// Absorbed from wetSpring V123 `OrExit` pattern. Validation binaries
/// (e.g., `rhizocrypt validate`) should never panic — they should print
/// a structured error message and exit with a non-zero status code.
///
/// # Usage
///
/// ```no_run
/// use rhizo_crypt_core::error::OrExit;
///
/// let config = std::fs::read_to_string("config.toml")
///     .or_exit("Failed to read configuration file");
/// ```
pub trait OrExit<T> {
    /// Unwrap the value or print the context message + error and exit with code 1.
    fn or_exit(self, context: &str) -> T;
}

impl<T, E: fmt::Display> OrExit<T> for std::result::Result<T, E> {
    fn or_exit(self, context: &str) -> T {
        match self {
            Ok(val) => val,
            Err(e) => {
                eprintln!("fatal: {context}: {e}");
                std::process::exit(1);
            }
        }
    }
}

impl<T> OrExit<T> for Option<T> {
    fn or_exit(self, context: &str) -> T {
        if let Some(val) = self {
            return val;
        }
        eprintln!("fatal: {context}");
        std::process::exit(1);
    }
}

// Validation harness and sinks live in `crate::validation`.
// Re-exported here for backward compatibility with `use rhizo_crypt_core::error::*`.
pub use crate::validation::{StderrSink, StringSink, ValidationHarness, ValidationSink};

impl RhizoCryptError {
    /// Create a configuration error.
    #[must_use]
    pub fn config(msg: impl Into<String>) -> Self {
        Self::Config(msg.into())
    }

    /// Create an invalid config error.
    #[must_use]
    pub fn invalid_config(key: impl Into<String>, reason: impl Into<String>) -> Self {
        Self::InvalidConfig {
            key: key.into(),
            reason: reason.into(),
        }
    }

    /// Create an internal error.
    #[must_use]
    pub fn internal(msg: impl Into<String>) -> Self {
        Self::Internal(msg.into())
    }

    /// Create a storage error.
    #[must_use]
    pub fn storage(msg: impl Into<String>) -> Self {
        Self::Storage(msg.into())
    }

    /// Create a session not found error.
    #[must_use]
    pub const fn session_not_found(session_id: SessionId) -> Self {
        Self::SessionNotFound(session_id)
    }

    /// Create a vertex not found error.
    #[must_use]
    pub const fn vertex_not_found(vertex_id: VertexId) -> Self {
        Self::VertexNotFound(vertex_id)
    }

    /// Create an integration error (service not discovered).
    #[must_use]
    pub fn integration(msg: impl Into<String>) -> Self {
        Self::Integration(msg.into())
    }

    /// Create a structured IPC error with phase context.
    #[must_use]
    pub fn ipc(phase: IpcErrorPhase, msg: impl Into<String>) -> Self {
        Self::Ipc {
            phase,
            message: msg.into(),
        }
    }

    /// Create an invalid input error.
    #[must_use]
    pub fn invalid_input(msg: impl Into<String>) -> Self {
        Self::Config(msg.into())
    }

    /// Returns true if this is a recoverable error.
    #[must_use]
    pub const fn is_recoverable(&self) -> bool {
        matches!(
            self,
            Self::Timeout(_)
                | Self::Storage(_)
                | Self::Integration(_)
                | Self::CapabilityProvider { .. }
                | Self::Ipc { .. }
        )
    }

    /// Create a capability provider error.
    #[must_use]
    pub fn capability_provider(capability: impl Into<String>, message: impl Into<String>) -> Self {
        Self::CapabilityProvider {
            capability: capability.into(),
            message: message.into(),
        }
    }

    /// Returns true if this is a not-found error.
    #[must_use]
    pub const fn is_not_found(&self) -> bool {
        matches!(
            self,
            Self::SessionNotFound(_)
                | Self::VertexNotFound(_)
                | Self::ParentNotFound(_)
                | Self::SliceNotFound(_)
        )
    }
}

/// Result type for RhizoCrypt operations.
pub type Result<T> = std::result::Result<T, RhizoCryptError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display() {
        let err = RhizoCryptError::config("test error");
        assert_eq!(err.to_string(), "configuration error: test error");
    }

    #[test]
    fn test_invalid_config() {
        let err = RhizoCryptError::invalid_config("max_vertices", "must be positive");
        assert!(err.to_string().contains("max_vertices"));
        assert!(err.to_string().contains("must be positive"));
    }

    #[test]
    fn test_is_recoverable() {
        assert!(RhizoCryptError::Timeout(1000).is_recoverable());
        assert!(RhizoCryptError::storage("disk full").is_recoverable());
        assert!(RhizoCryptError::integration("service unavailable").is_recoverable());
        assert!(RhizoCryptError::capability_provider("signing", "timeout").is_recoverable());
        assert!(!RhizoCryptError::config("invalid").is_recoverable());
    }

    #[test]
    fn test_capability_provider_error() {
        let err = RhizoCryptError::capability_provider("signing", "key not found");
        assert!(err.to_string().contains("signing"));
        assert!(err.to_string().contains("key not found"));
        assert!(err.is_recoverable());
    }

    #[test]
    fn test_is_not_found() {
        let session_id = SessionId::now();
        assert!(RhizoCryptError::SessionNotFound(session_id).is_not_found());
        assert!(!RhizoCryptError::config("test").is_not_found());
    }

    #[test]
    fn test_ipc_error_phases() {
        let connect = RhizoCryptError::ipc(IpcErrorPhase::Connect, "socket missing");
        assert!(connect.to_string().contains("connect"));
        assert!(connect.to_string().contains("socket missing"));
        assert!(connect.is_recoverable());

        let write = RhizoCryptError::ipc(IpcErrorPhase::Write, "broken pipe");
        assert!(write.to_string().contains("write"));

        let read = RhizoCryptError::ipc(IpcErrorPhase::Read, "timeout");
        assert!(read.to_string().contains("read"));

        let invalid = RhizoCryptError::ipc(IpcErrorPhase::InvalidJson, "unexpected EOF");
        assert!(invalid.to_string().contains("invalid_json"));

        let http = RhizoCryptError::ipc(IpcErrorPhase::HttpStatus(500), "internal");
        assert!(http.to_string().contains("http_500"));

        let no_result = RhizoCryptError::ipc(IpcErrorPhase::NoResult, "missing field");
        assert!(no_result.to_string().contains("no_result"));

        let rpc = RhizoCryptError::ipc(IpcErrorPhase::JsonRpcError(-32601), "method not found");
        assert!(rpc.to_string().contains("jsonrpc_-32601"));
    }

    #[test]
    fn test_ipc_error_phase_display() {
        assert_eq!(IpcErrorPhase::Connect.to_string(), "connect");
        assert_eq!(IpcErrorPhase::Write.to_string(), "write");
        assert_eq!(IpcErrorPhase::Read.to_string(), "read");
        assert_eq!(IpcErrorPhase::InvalidJson.to_string(), "invalid_json");
        assert_eq!(IpcErrorPhase::HttpStatus(404).to_string(), "http_404");
        assert_eq!(IpcErrorPhase::NoResult.to_string(), "no_result");
        assert_eq!(IpcErrorPhase::JsonRpcError(-32600).to_string(), "jsonrpc_-32600");
    }

    #[test]
    fn test_ipc_error_phase_equality() {
        assert_eq!(IpcErrorPhase::Connect, IpcErrorPhase::Connect);
        assert_ne!(IpcErrorPhase::Connect, IpcErrorPhase::Write);
        assert_eq!(IpcErrorPhase::HttpStatus(500), IpcErrorPhase::HttpStatus(500));
        assert_ne!(IpcErrorPhase::HttpStatus(500), IpcErrorPhase::HttpStatus(404));
    }

    #[test]
    fn test_ipc_phase_is_method_not_found() {
        assert!(IpcErrorPhase::JsonRpcError(-32601).is_method_not_found());
        assert!(!IpcErrorPhase::JsonRpcError(-32600).is_method_not_found());
        assert!(!IpcErrorPhase::Connect.is_method_not_found());
    }

    #[test]
    fn test_ipc_phase_is_timeout_likely() {
        assert!(IpcErrorPhase::Connect.is_timeout_likely());
        assert!(IpcErrorPhase::Read.is_timeout_likely());
        assert!(!IpcErrorPhase::Write.is_timeout_likely());
        assert!(!IpcErrorPhase::JsonRpcError(-1).is_timeout_likely());
    }

    #[test]
    fn test_ipc_phase_is_retriable() {
        assert!(IpcErrorPhase::Connect.is_retriable());
        assert!(IpcErrorPhase::Write.is_retriable());
        assert!(IpcErrorPhase::Read.is_retriable());
        assert!(!IpcErrorPhase::InvalidJson.is_retriable());
        assert!(!IpcErrorPhase::NoResult.is_retriable());
        assert!(!IpcErrorPhase::HttpStatus(500).is_retriable());
        assert!(!IpcErrorPhase::JsonRpcError(-32601).is_retriable());
    }

    #[test]
    fn test_ipc_phase_is_application_error() {
        assert!(IpcErrorPhase::JsonRpcError(-32601).is_application_error());
        assert!(IpcErrorPhase::NoResult.is_application_error());
        assert!(!IpcErrorPhase::Connect.is_application_error());
        assert!(!IpcErrorPhase::Read.is_application_error());
    }

    #[test]
    fn test_dispatch_outcome_ok() {
        let outcome: DispatchOutcome<i32> = DispatchOutcome::Ok(42);
        assert!(outcome.is_ok());
        assert_eq!(outcome.into_result().unwrap(), 42);
    }

    #[test]
    fn test_dispatch_outcome_application_error() {
        let outcome: DispatchOutcome<i32> = DispatchOutcome::ApplicationError {
            code: -32601,
            message: "method not found".into(),
        };
        assert!(!outcome.is_ok());
        let err = outcome.into_result().unwrap_err();
        assert!(err.to_string().contains("jsonrpc_-32601"));
        assert!(err.to_string().contains("method not found"));
    }

    #[test]
    fn test_dispatch_outcome_protocol_error() {
        let outcome: DispatchOutcome<i32> =
            DispatchOutcome::ProtocolError(RhizoCryptError::ipc(IpcErrorPhase::Connect, "timeout"));
        assert!(!outcome.is_ok());
        let err = outcome.into_result().unwrap_err();
        assert!(err.to_string().contains("connect"));
    }

    #[test]
    fn test_extract_rpc_error_present() {
        let response = serde_json::json!({
            "jsonrpc": "2.0",
            "error": {"code": -32601, "message": "Method not found"},
            "id": 1
        });
        let (code, msg) = extract_rpc_error(&response).unwrap();
        assert_eq!(code, -32601);
        assert_eq!(msg, "Method not found");
    }

    #[test]
    fn test_extract_rpc_error_absent() {
        let response = serde_json::json!({
            "jsonrpc": "2.0",
            "result": true,
            "id": 1
        });
        assert!(extract_rpc_error(&response).is_none());
    }

    #[test]
    fn test_extract_rpc_error_defaults() {
        let response = serde_json::json!({
            "error": {}
        });
        let (code, msg) = extract_rpc_error(&response).unwrap();
        assert_eq!(code, -1);
        assert_eq!(msg, "Unknown error");
    }
}
