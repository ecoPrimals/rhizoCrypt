// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2024–2026 ecoPrimals Project

//! RhizoCrypt error types.
//!
//! This module defines all error types used throughout the DAG engine.

use crate::types::{SessionId, VertexId};
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
#[allow(clippy::unwrap_used, clippy::expect_used)]
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
}
