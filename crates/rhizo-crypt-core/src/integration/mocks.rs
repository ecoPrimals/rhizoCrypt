// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2024–2026 ecoPrimals Project

//! Mock implementations for testing.
//!
//! **This module contains test-only code.** Production code should use
//! runtime-discovered clients via the discovery module.
//!
//! ## Usage
//!
//! ```no_run
//! # use rhizo_crypt_core::integration::{mocks::MockSigningProvider, SigningProvider};
//! # use rhizo_crypt_core::types::Did;
//! # tokio::runtime::Runtime::new().unwrap().block_on(async {
//! let client = MockSigningProvider::permissive();
//! let did = Did::new("did:key:test");
//! assert!(client.verify_did(&did).await?);
//! # Ok::<(), rhizo_crypt_core::error::RhizoCryptError>(())
//! # });
//! ```

use crate::dehydration::{Attestation, AttestationStatement, DehydrationSummary};
use crate::error::Result;
use crate::session::LoamCommitRef;
use crate::slice::{ResolutionOutcome, Slice, SliceOrigin};
use crate::types::{Did, PayloadRef, Signature, Timestamp};
use crate::vertex::Vertex;

use super::{PayloadStorageProvider, PermanentStorageProvider, SigningProvider};

use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use tokio::sync::RwLock;

// ============================================================================
// MockSigningProvider
// ============================================================================

/// Mock signing provider for testing - works with ANY signing capability.
///
/// Provides configurable behavior for DID verification and signing operations.
/// This mock is capability-based, not vendor-specific.
#[derive(Debug, Default, Clone)]
pub struct MockSigningProvider {
    /// Always verify DIDs as valid.
    pub always_valid: bool,
    /// Always verify signatures as valid.
    pub signatures_valid: bool,
}

impl MockSigningProvider {
    /// Create a new mock provider that accepts everything.
    #[must_use]
    pub const fn permissive() -> Self {
        Self {
            always_valid: true,
            signatures_valid: true,
        }
    }

    /// Create a new mock provider that rejects everything.
    #[must_use]
    pub const fn strict() -> Self {
        Self {
            always_valid: false,
            signatures_valid: false,
        }
    }
}

impl SigningProvider for MockSigningProvider {
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
            signature: bytes::Bytes::from_static(&[0xDE, 0xAD, 0xBE, 0xEF]),
            attested_at: Timestamp::now(),
            verified: self.signatures_valid,
        })
    }
}

// ============================================================================
// MockPermanentStorageProvider
// ============================================================================

/// Mock permanent storage provider for testing - works with ANY permanent storage capability.
///
/// Provides a simple in-memory implementation that tracks commit indices.
/// This mock is capability-based, not vendor-specific.
#[derive(Debug, Default)]
pub struct MockPermanentStorageProvider {
    next_index: AtomicU64,
}

impl Clone for MockPermanentStorageProvider {
    fn clone(&self) -> Self {
        Self {
            next_index: AtomicU64::new(self.next_index.load(Ordering::SeqCst)),
        }
    }
}

impl MockPermanentStorageProvider {
    /// Create a new mock provider.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }
}

impl PermanentStorageProvider for MockPermanentStorageProvider {
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
// MockPayloadStorageProvider
// ============================================================================

/// Mock payload storage provider for testing - works with ANY content-addressed storage.
///
/// Provides an in-memory payload store for testing.
/// This mock is capability-based, not vendor-specific.
#[derive(Debug, Default, Clone)]
pub struct MockPayloadStorageProvider {
    payloads: Arc<RwLock<HashMap<[u8; 32], bytes::Bytes>>>,
}

impl MockPayloadStorageProvider {
    /// Create a new mock provider.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }
}

impl PayloadStorageProvider for MockPayloadStorageProvider {
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
        let client = MockSigningProvider::permissive();

        let did = Did::new("did:key:test");
        assert!(client.verify_did(&did).await.unwrap());

        let signature = client.sign(b"test data", &did).await.unwrap();
        assert!(!signature.as_bytes().is_empty());

        assert!(client.verify_signature(b"test", &signature, &did).await.unwrap());
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn test_mock_beardog_strict() {
        let client = MockSigningProvider::strict();

        let did = Did::new("did:key:test");
        assert!(!client.verify_did(&did).await.unwrap());
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn test_mock_loamspine_client() {
        let client = MockPermanentStorageProvider::new();

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
        let client = MockPayloadStorageProvider::new();

        let data = bytes::Bytes::from("test payload");
        let payload_ref = client.put_payload(data.clone()).await.unwrap();

        assert!(client.payload_exists(&payload_ref).await.unwrap());

        let retrieved = client.get_payload(&payload_ref).await.unwrap();
        assert_eq!(retrieved, Some(data));
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn test_mock_beardog_sign_vertex() {
        let client = MockSigningProvider::permissive();
        let did = Did::new("did:key:test");

        let vertex = VertexBuilder::new(EventType::SessionStart).build();

        let signature = client.sign_vertex(&vertex, &did).await.unwrap();
        assert!(!signature.as_bytes().is_empty());
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn test_mock_beardog_verify_vertex_signature() {
        let permissive_client = MockSigningProvider::permissive();
        let strict_client = MockSigningProvider::strict();

        let vertex = VertexBuilder::new(EventType::SessionStart).build();

        assert!(permissive_client.verify_vertex_signature(&vertex).await.unwrap());
        assert!(!strict_client.verify_vertex_signature(&vertex).await.unwrap());
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn test_mock_beardog_request_attestation() {
        let client = MockSigningProvider::permissive();
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
        let client = MockPermanentStorageProvider::new();

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
        let client = MockPermanentStorageProvider::new();
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
        let client1 = MockPermanentStorageProvider::new();

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
        let client = MockPayloadStorageProvider::new();

        // Query for non-existent payload
        let fake_ref = PayloadRef::new([1u8; 32], 100);

        assert!(!client.payload_exists(&fake_ref).await.unwrap());
        assert!(client.get_payload(&fake_ref).await.unwrap().is_none());
    }
}

// ============================================================================
// NEW: Capability-Based Mock Adapters
// ============================================================================

/// Mock protocol adapter for testing capability clients.
///
/// This adapter simulates ANY provider by returning predefined responses.
/// Use with capability clients for testing without real services.
///
/// ## Example
///
/// ```no_run
/// # use rhizo_crypt_core::integration::mocks::MockProtocolAdapter;
/// # use rhizo_crypt_core::clients::adapters::ProtocolAdapter;
/// # tokio::runtime::Runtime::new().unwrap().block_on(async {
/// // Create mock adapter
/// let adapter = MockProtocolAdapter::permissive();
///
/// // Use adapter directly (e.g. for testing protocol layer)
/// let _response = adapter.call_json("verify_did", "{}").await?;
/// # Ok::<(), rhizo_crypt_core::error::RhizoCryptError>(())
/// # });
/// ```
#[derive(Debug, Clone)]
pub struct MockProtocolAdapter {
    /// Responses to return for each method
    responses: Arc<RwLock<HashMap<String, String>>>,
    /// Whether to succeed or fail all calls
    permissive: bool,
}

impl MockProtocolAdapter {
    /// Create a new mock adapter that returns success for all calls.
    #[must_use]
    pub fn permissive() -> Self {
        Self {
            responses: Arc::new(RwLock::new(HashMap::new())),
            permissive: true,
        }
    }

    /// Create a new mock adapter that fails all calls.
    #[must_use]
    pub fn strict() -> Self {
        Self {
            responses: Arc::new(RwLock::new(HashMap::new())),
            permissive: false,
        }
    }

    /// Set a response for a specific method.
    ///
    /// The response should be JSON-serializable.
    pub async fn set_response<T: serde::Serialize>(&self, method: &str, response: T) -> Result<()> {
        let json = serde_json::to_string(&response).map_err(|e| {
            crate::error::RhizoCryptError::integration(format!("Mock serialization failed: {e}"))
        })?;

        let mut responses = self.responses.write().await;
        responses.insert(method.to_string(), json);
        Ok(())
    }
}

#[async_trait::async_trait]
impl crate::clients::adapters::ProtocolAdapter for MockProtocolAdapter {
    fn protocol(&self) -> &str {
        "mock"
    }

    async fn call_json(&self, method: &str, _args: &str) -> Result<String> {
        if !self.permissive {
            return Err(crate::error::RhizoCryptError::integration(
                "Mock adapter is in strict mode",
            ));
        }

        // Check for pre-configured response
        let responses = self.responses.read().await;
        if let Some(response) = responses.get(method) {
            return Ok(response.clone());
        }

        // Return default responses based on method
        match method {
            "sign" => Ok(r#"{"bytes":[222,173,190,239]}"#.to_string()),
            "verify_did" | "verify_signature" => Ok("true".to_string()),
            "put_payload" => Ok(r#"{"hash":[0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],"size":0}"#.to_string()),
            "get_payload" => Ok("null".to_string()),
            "payload_exists" => Ok("false".to_string()),
            _ => Err(crate::error::RhizoCryptError::integration(format!("Mock adapter: unknown method '{method}'"))),
        }
    }

    async fn call_oneway_json(&self, method: &str, _args: &str) -> Result<()> {
        if !self.permissive {
            return Err(crate::error::RhizoCryptError::integration(
                "Mock adapter is in strict mode",
            ));
        }

        // One-way calls always succeed in permissive mode
        tracing::debug!("Mock adapter: one-way call to '{}'", method);
        Ok(())
    }

    async fn is_healthy(&self) -> bool {
        self.permissive
    }

    fn endpoint(&self) -> &str {
        "mock://test"
    }
}

/// Factory for creating mock capability clients.
///
/// Provides a convenient way to create pre-configured mock clients for testing
/// without needing a real discovery registry.
///
/// ## Example
///
/// ```no_run
/// # use rhizo_crypt_core::integration::mocks::MockCapabilityFactory;
/// // Create factory with permissive mock adapter
/// let factory = MockCapabilityFactory::permissive();
/// // Get the adapter for configuring mock responses or passing to clients
/// let _adapter = factory.adapter();
/// ```
#[derive(Debug, Clone)]
pub struct MockCapabilityFactory {
    adapter: Arc<MockProtocolAdapter>,
}

impl MockCapabilityFactory {
    /// Create a factory that returns permissive mock clients.
    #[must_use]
    pub fn permissive() -> Self {
        Self {
            adapter: Arc::new(MockProtocolAdapter::permissive()),
        }
    }

    /// Create a factory that returns strict mock clients.
    #[must_use]
    pub fn strict() -> Self {
        Self {
            adapter: Arc::new(MockProtocolAdapter::strict()),
        }
    }

    /// Get the underlying mock adapter for configuration.
    #[must_use]
    pub fn adapter(&self) -> Arc<MockProtocolAdapter> {
        Arc::clone(&self.adapter)
    }
}

#[cfg(test)]
mod capability_mock_tests {
    use super::*;
    use crate::clients::adapters::ProtocolAdapter;

    #[tokio::test]
    async fn test_mock_protocol_adapter_permissive() {
        let adapter = MockProtocolAdapter::permissive();

        // Should return default responses
        let result = adapter.call_json("verify_did", "{}").await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "true");
    }

    #[tokio::test]
    async fn test_mock_protocol_adapter_strict() {
        let adapter = MockProtocolAdapter::strict();

        // Should fail all calls
        let result = adapter.call_json("verify_did", "{}").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_mock_protocol_adapter_custom_response() {
        let adapter = MockProtocolAdapter::permissive();

        // Set custom response
        adapter.set_response("my_method", "custom_response").await.unwrap();

        // Should return custom response
        let result = adapter.call_json("my_method", "{}").await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), r#""custom_response""#);
    }
}
