// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2024–2026 ecoPrimals Project

//! Capability definitions for primal discovery.
//!
//! Capabilities are primal-agnostic: they describe WHAT a service provides,
//! not WHO provides it. Any primal may implement any capability.

use serde::{Deserialize, Serialize};
use std::borrow::Cow;

/// Capability identifier - what a service can do.
///
/// Capabilities are primal-agnostic: they describe WHAT a service provides,
/// not WHO provides it. Any primal may implement any capability.
///
/// # Infant Discovery
///
/// Primals start with zero knowledge and discover capabilities at runtime.
/// There is no hardcoding of which primal provides which capability.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[non_exhaustive]
pub enum Capability {
    // === Identity & Cryptography ===
    /// DID resolution and verification.
    DidVerification,
    /// Cryptographic signing operations.
    Signing,
    /// Signature verification.
    SignatureVerification,
    /// Attestation and credential requests.
    Attestation,

    // === Discovery & Mesh ===
    /// Service discovery and registration.
    ServiceDiscovery,

    // === Payload Storage ===
    /// Content-addressed payload storage.
    PayloadStorage,
    /// Content-addressed payload retrieval.
    PayloadRetrieval,

    // === Permanent Storage ===
    /// Permanent/immutable storage commits.
    PermanentCommit,
    /// Slice checkout from permanent storage.
    SliceCheckout,
    /// Slice resolution back to permanent storage.
    SliceResolution,

    // === Compute ===
    /// Compute task orchestration.
    ComputeOrchestration,
    /// Compute task event streaming.
    ComputeEvents,

    // === Provenance ===
    /// Provenance chain queries.
    ProvenanceQuery,
    /// Attribution and contribution tracking.
    Attribution,

    /// Custom capability for extensibility.
    Custom(Cow<'static, str>),
}

impl Capability {
    /// Create a custom capability.
    #[inline]
    #[must_use]
    pub fn custom(name: impl Into<Cow<'static, str>>) -> Self {
        Self::Custom(name.into())
    }
}

impl std::fmt::Display for Capability {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            // Identity & Cryptography
            Self::DidVerification => write!(f, "did:verification"),
            Self::Signing => write!(f, "crypto:signing"),
            Self::SignatureVerification => write!(f, "crypto:verification"),
            Self::Attestation => write!(f, "attestation:request"),
            // Discovery & Mesh
            Self::ServiceDiscovery => write!(f, "discovery:service"),
            // Payload Storage
            Self::PayloadStorage => write!(f, "payload:storage"),
            Self::PayloadRetrieval => write!(f, "payload:retrieval"),
            // Permanent Storage
            Self::PermanentCommit => write!(f, "storage:permanent:commit"),
            Self::SliceCheckout => write!(f, "slice:checkout"),
            Self::SliceResolution => write!(f, "slice:resolution"),
            // Compute
            Self::ComputeOrchestration => write!(f, "compute:orchestration"),
            Self::ComputeEvents => write!(f, "compute:events"),
            // Provenance
            Self::ProvenanceQuery => write!(f, "provenance:query"),
            Self::Attribution => write!(f, "provenance:attribution"),
            // Custom
            Self::Custom(name) => write!(f, "custom:{name}"),
        }
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;

    #[test]
    fn test_capability_display() {
        assert_eq!(Capability::DidVerification.to_string(), "did:verification");
        assert_eq!(Capability::custom("myapp:feature").to_string(), "custom:myapp:feature");
    }

    #[test]
    fn test_capability_display_all() {
        // Test all capability display strings
        assert_eq!(Capability::DidVerification.to_string(), "did:verification");
        assert_eq!(Capability::Signing.to_string(), "crypto:signing");
        assert_eq!(Capability::SignatureVerification.to_string(), "crypto:verification");
        assert_eq!(Capability::Attestation.to_string(), "attestation:request");
        assert_eq!(Capability::ServiceDiscovery.to_string(), "discovery:service");
        assert_eq!(Capability::PayloadStorage.to_string(), "payload:storage");
        assert_eq!(Capability::PayloadRetrieval.to_string(), "payload:retrieval");
        assert_eq!(Capability::PermanentCommit.to_string(), "storage:permanent:commit");
        assert_eq!(Capability::SliceCheckout.to_string(), "slice:checkout");
        assert_eq!(Capability::SliceResolution.to_string(), "slice:resolution");
        assert_eq!(Capability::ComputeOrchestration.to_string(), "compute:orchestration");
        assert_eq!(Capability::ComputeEvents.to_string(), "compute:events");
        assert_eq!(Capability::ProvenanceQuery.to_string(), "provenance:query");
        assert_eq!(Capability::Attribution.to_string(), "provenance:attribution");
    }

    #[test]
    fn test_capability_equality() {
        assert_eq!(Capability::Signing, Capability::Signing);
        assert_ne!(Capability::Signing, Capability::Attestation);
        assert_eq!(Capability::Custom("test".into()), Capability::Custom("test".into()));
        assert_ne!(Capability::Custom("test1".into()), Capability::Custom("test2".into()));
    }

    #[test]
    fn test_capability_hash() {
        use std::collections::HashSet;
        let mut set = HashSet::new();
        set.insert(Capability::Signing);
        set.insert(Capability::Signing); // duplicate
        set.insert(Capability::Attestation);
        assert_eq!(set.len(), 2);
    }
}
