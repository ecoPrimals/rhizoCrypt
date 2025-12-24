//! Mock implementations for testing.
//!
//! **This module contains test-only code.** Production code should use
//! runtime-discovered clients via the discovery module.
//!
//! ## Usage
//!
//! ```rust,ignore
//! #[cfg(test)]
//! use rhizo_crypt_core::integration::mocks::*;
//!
//! let client = MockBearDogClient::permissive();
//! assert!(client.verify_did(&did).await?);
//! ```

use crate::dehydration::{Attestation, AttestationStatement, DehydrationSummary};
use crate::error::Result;
use crate::session::LoamCommitRef;
use crate::slice::{ResolutionOutcome, Slice, SliceOrigin};
use crate::types::{Did, PayloadRef, Signature, Timestamp};
use crate::vertex::Vertex;

use super::{BearDogClient, LoamSpineClient, NestGateClient};

use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use tokio::sync::RwLock;

// ============================================================================
// MockBearDogClient
// ============================================================================

/// Mock BearDog client for testing.
///
/// Provides configurable behavior for DID verification and signing operations.
#[derive(Debug, Default, Clone)]
pub struct MockBearDogClient {
    /// Always verify DIDs as valid.
    pub always_valid: bool,
    /// Always verify signatures as valid.
    pub signatures_valid: bool,
}

impl MockBearDogClient {
    /// Create a new mock client that accepts everything.
    #[must_use]
    pub const fn permissive() -> Self {
        Self {
            always_valid: true,
            signatures_valid: true,
        }
    }

    /// Create a new mock client that rejects everything.
    #[must_use]
    pub const fn strict() -> Self {
        Self {
            always_valid: false,
            signatures_valid: false,
        }
    }
}

impl BearDogClient for MockBearDogClient {
    async fn verify_did(&self, _did: &Did) -> Result<bool> {
        Ok(self.always_valid)
    }

    async fn sign(&self, _data: &[u8], _signer: &Did) -> Result<Signature> {
        Ok(Signature::new(vec![0xDE, 0xAD, 0xBE, 0xEF]))
    }

    async fn sign_vertex(&self, _vertex: &Vertex, _signer: &Did) -> Result<Signature> {
        Ok(Signature::new(vec![0xDE, 0xAD, 0xBE, 0xEF]))
    }

    async fn verify_signature(
        &self,
        _data: &[u8],
        _signature: &Signature,
        _signer: &Did,
    ) -> Result<bool> {
        Ok(self.signatures_valid)
    }

    async fn verify_vertex_signature(&self, _vertex: &Vertex) -> Result<bool> {
        Ok(self.signatures_valid)
    }

    async fn request_attestation(
        &self,
        attester: &Did,
        summary: &DehydrationSummary,
    ) -> Result<Attestation> {
        Ok(Attestation {
            attester: attester.clone(),
            statement: AttestationStatement::SessionSummary {
                summary_hash: summary.compute_hash(),
            },
            signature: vec![0xDE, 0xAD, 0xBE, 0xEF],
            attested_at: Timestamp::now(),
            verified: self.signatures_valid,
        })
    }
}

// ============================================================================
// MockLoamSpineClient
// ============================================================================

/// Mock LoamSpine client for testing.
///
/// Provides a simple in-memory implementation that tracks commit indices.
#[derive(Debug, Default)]
pub struct MockLoamSpineClient {
    next_index: AtomicU64,
}

impl Clone for MockLoamSpineClient {
    fn clone(&self) -> Self {
        Self {
            next_index: AtomicU64::new(self.next_index.load(Ordering::SeqCst)),
        }
    }
}

impl MockLoamSpineClient {
    /// Create a new mock client.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }
}

impl LoamSpineClient for MockLoamSpineClient {
    async fn commit(&self, _summary: &DehydrationSummary) -> Result<LoamCommitRef> {
        let index = self.next_index.fetch_add(1, Ordering::SeqCst);
        Ok(LoamCommitRef {
            spine_id: "mock-spine".to_string(),
            entry_hash: [0u8; 32],
            index,
        })
    }

    async fn verify_commit(&self, _commit_ref: &LoamCommitRef) -> Result<bool> {
        Ok(true)
    }

    async fn get_commit(&self, _commit_ref: &LoamCommitRef) -> Result<Option<DehydrationSummary>> {
        Ok(None) // Mock doesn't persist
    }

    async fn checkout_slice(
        &self,
        spine_id: &str,
        entry_hash: &[u8; 32],
        _holder: &Did,
    ) -> Result<SliceOrigin> {
        Ok(SliceOrigin {
            spine_id: spine_id.to_string(),
            entry_hash: *entry_hash,
            entry_index: 0,
            certificate_id: None,
            owner: Did::default(),
        })
    }

    async fn resolve_slice(&self, _slice: &Slice, _outcome: &ResolutionOutcome) -> Result<()> {
        Ok(())
    }
}

// ============================================================================
// MockNestGateClient
// ============================================================================

/// Mock NestGate client for testing.
///
/// Provides an in-memory payload store for testing.
#[derive(Debug, Default, Clone)]
pub struct MockNestGateClient {
    payloads: Arc<RwLock<HashMap<[u8; 32], bytes::Bytes>>>,
}

impl MockNestGateClient {
    /// Create a new mock client.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }
}

impl NestGateClient for MockNestGateClient {
    async fn put_payload(&self, data: bytes::Bytes) -> Result<PayloadRef> {
        let payload_ref = PayloadRef::from_bytes(&data);
        self.payloads.write().await.insert(payload_ref.hash, data);
        Ok(payload_ref)
    }

    async fn get_payload(&self, payload_ref: &PayloadRef) -> Result<Option<bytes::Bytes>> {
        let payloads = self.payloads.read().await;
        Ok(payloads.get(&payload_ref.hash).cloned())
    }

    async fn payload_exists(&self, payload_ref: &PayloadRef) -> Result<bool> {
        let payloads = self.payloads.read().await;
        Ok(payloads.contains_key(&payload_ref.hash))
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;
    use crate::dehydration::DehydrationSummaryBuilder;
    use crate::event::EventType;
    use crate::merkle::MerkleRoot;
    use crate::slice::{SliceBuilder, SliceMode};
    use crate::types::{SessionId, VertexId};
    use crate::vertex::VertexBuilder;

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn test_mock_beardog_client() {
        let client = MockBearDogClient::permissive();

        let did = Did::new("did:key:test");
        assert!(client.verify_did(&did).await.unwrap());

        let signature = client.sign(b"test data", &did).await.unwrap();
        assert!(!signature.as_bytes().is_empty());

        assert!(client.verify_signature(b"test", &signature, &did).await.unwrap());
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn test_mock_beardog_strict() {
        let client = MockBearDogClient::strict();

        let did = Did::new("did:key:test");
        assert!(!client.verify_did(&did).await.unwrap());
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn test_mock_loamspine_client() {
        let client = MockLoamSpineClient::new();

        let summary = DehydrationSummaryBuilder::new(
            SessionId::now(),
            "test",
            Timestamp::now(),
            MerkleRoot::ZERO,
        )
        .build();

        let commit_ref = client.commit(&summary).await.unwrap();
        assert_eq!(commit_ref.index, 0);

        let commit_ref2 = client.commit(&summary).await.unwrap();
        assert_eq!(commit_ref2.index, 1);

        assert!(client.verify_commit(&commit_ref).await.unwrap());
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn test_mock_nestgate_client() {
        let client = MockNestGateClient::new();

        let data = bytes::Bytes::from("test payload");
        let payload_ref = client.put_payload(data.clone()).await.unwrap();

        assert!(client.payload_exists(&payload_ref).await.unwrap());

        let retrieved = client.get_payload(&payload_ref).await.unwrap();
        assert_eq!(retrieved, Some(data));
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn test_mock_beardog_sign_vertex() {
        let client = MockBearDogClient::permissive();
        let did = Did::new("did:key:test");

        let vertex = VertexBuilder::new(EventType::SessionStart).build();

        let signature = client.sign_vertex(&vertex, &did).await.unwrap();
        assert!(!signature.as_bytes().is_empty());
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn test_mock_beardog_verify_vertex_signature() {
        let permissive_client = MockBearDogClient::permissive();
        let strict_client = MockBearDogClient::strict();

        let vertex = VertexBuilder::new(EventType::SessionStart).build();

        assert!(permissive_client.verify_vertex_signature(&vertex).await.unwrap());
        assert!(!strict_client.verify_vertex_signature(&vertex).await.unwrap());
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn test_mock_beardog_request_attestation() {
        let client = MockBearDogClient::permissive();
        let attester = Did::new("did:key:attester");

        let summary = DehydrationSummaryBuilder::new(
            SessionId::now(),
            "test-session",
            Timestamp::now(),
            MerkleRoot::ZERO,
        )
        .build();

        let attestation = client.request_attestation(&attester, &summary).await.unwrap();
        assert_eq!(attestation.attester.as_str(), "did:key:attester");
        assert!(attestation.verified);
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn test_mock_loamspine_get_commit() {
        let client = MockLoamSpineClient::new();

        let commit_ref = LoamCommitRef {
            spine_id: "test-spine".to_string(),
            entry_hash: [1u8; 32],
            index: 0,
        };

        // Mock always returns None (doesn't persist)
        let result = client.get_commit(&commit_ref).await.unwrap();
        assert!(result.is_none());
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn test_mock_loamspine_checkout_resolve_slice() {
        let client = MockLoamSpineClient::new();
        let holder = Did::new("did:key:holder");

        // Test checkout
        let origin = client.checkout_slice("test-spine", &[0u8; 32], &holder).await.unwrap();
        assert_eq!(origin.spine_id, "test-spine");

        // Test resolve with Copy mode
        let slice = SliceBuilder::new(
            origin,
            holder,
            SliceMode::Copy {
                allow_recopy: false,
            },
            SessionId::now(),
            VertexId::ZERO,
        )
        .build();

        let outcome = ResolutionOutcome::ReturnedUnchanged;
        assert!(client.resolve_slice(&slice, &outcome).await.is_ok());
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn test_mock_loamspine_clone() {
        let client1 = MockLoamSpineClient::new();

        let summary = DehydrationSummaryBuilder::new(
            SessionId::now(),
            "test",
            Timestamp::now(),
            MerkleRoot::ZERO,
        )
        .build();

        // Increment index
        let _ = client1.commit(&summary).await.unwrap();

        // Clone should have same index
        let client2 = client1.clone();
        let ref2 = client2.commit(&summary).await.unwrap();
        assert_eq!(ref2.index, 1); // continues from cloned state
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn test_mock_nestgate_payload_not_found() {
        let client = MockNestGateClient::new();

        // Query for non-existent payload
        let fake_ref = PayloadRef::new([1u8; 32], 100);

        assert!(!client.payload_exists(&fake_ref).await.unwrap());
        assert!(client.get_payload(&fake_ref).await.unwrap().is_none());
    }
}
