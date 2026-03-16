// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2024–2026 ecoPrimals Project

//! Generic signing client - works with ANY signing provider.
//!
//! This client provides cryptographic signing capabilities without knowing
//! or caring about who provides the service (BearDog, YubiKey, CloudKMS, etc.).
//!
//! ## Philosophy
//!
//! Request capabilities, not vendors:
//! - "I need crypto:signing" ✅
//! - "I need BearDog" ❌
//!
//! ## Usage
//!
//! ```no_run
//! # use rhizo_crypt_core::clients::capabilities::SigningClient;
//! # use rhizo_crypt_core::types::Did;
//! # use std::sync::Arc;
//! # tokio::runtime::Runtime::new().unwrap().block_on(async {
//! # let registry = Arc::new(rhizo_crypt_core::discovery::DiscoveryRegistry::new("doc-test"));
//! # registry.register_endpoint(rhizo_crypt_core::discovery::ServiceEndpoint::new(
//! #     "test-signer",
//! #     "127.0.0.1:9500".parse().unwrap(),
//! #     vec![rhizo_crypt_core::discovery::Capability::Signing],
//! # )).await;
//! // Discover ANY signing provider
//! let signer = SigningClient::discover(&registry).await?;
//!
//! // Sign data (works with any provider)
//! let did = Did::new("did:key:test");
//! let signature = signer.sign(b"data", &did).await?;
//!
//! // Verify signature
//! let valid = signer.verify(b"data", &signature, &did).await?;
//! # Ok::<(), rhizo_crypt_core::error::RhizoCryptError>(())
//! # });
//! ```

use crate::clients::adapters::{AdapterFactory, ProtocolAdapter, ProtocolAdapterExt};
use crate::dehydration::{Attestation, DehydrationSummary};
use crate::discovery::{Capability, DiscoveryRegistry};
use crate::error::{Result, RhizoCryptError};
use crate::types::{Did, Signature};
use crate::vertex::Vertex;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

// ============================================================================
// Signing Client (Generic)
// ============================================================================

/// Generic signing client - works with ANY provider.
///
/// This client is vendor-agnostic. It works with any service that provides
/// signing capabilities: BearDog, YubiKey, CloudKMS, HSM, etc.
///
/// ## Discovery
///
/// The client discovers providers at runtime through the discovery registry.
/// No compile-time knowledge of specific services.
///
/// ## Protocol
///
/// The client uses protocol adapters to communicate. It works with any protocol:
/// tarpc, HTTP, gRPC, WebSocket, etc.
#[derive(Debug, Clone)]
pub struct SigningClient {
    /// Protocol adapter (HTTP, tarpc, gRPC, etc.)
    adapter: Arc<Box<dyn ProtocolAdapter>>,
    /// Service endpoint
    endpoint: String,
    /// Service name (for logging only)
    service_name: Option<String>,
}

impl SigningClient {
    /// Discover and connect to ANY signing provider.
    ///
    /// # Arguments
    ///
    /// * `registry` - Discovery registry to query
    ///
    /// # Errors
    ///
    /// Returns error if:
    /// - No signing provider available
    /// - Connection fails
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use rhizo_crypt_core::clients::capabilities::SigningClient;
    /// # use std::sync::Arc;
    /// # tokio::runtime::Runtime::new().unwrap().block_on(async {
    /// # let registry = Arc::new(rhizo_crypt_core::discovery::DiscoveryRegistry::new("doc-test"));
    /// # registry.register_endpoint(rhizo_crypt_core::discovery::ServiceEndpoint::new(
    /// #     "test-signer", "127.0.0.1:9500".parse().unwrap(),
    /// #     vec![rhizo_crypt_core::discovery::Capability::Signing],
    /// # )).await;
    /// let signer = SigningClient::discover(&registry).await?;
    /// # Ok::<(), rhizo_crypt_core::error::RhizoCryptError>(())
    /// # });
    /// ```
    pub async fn discover(registry: &DiscoveryRegistry) -> Result<Self> {
        tracing::info!("🔍 Discovering signing capability provider...");

        // Query registry for ANY service providing signing capability
        let status = registry.discover(&Capability::Signing).await;

        let endpoint = match status {
            crate::discovery::DiscoveryStatus::Available(endpoints) => {
                endpoints.into_iter().next().ok_or_else(|| {
                    RhizoCryptError::integration("No signing providers in available list")
                })?
            }
            crate::discovery::DiscoveryStatus::Discovering => {
                return Err(RhizoCryptError::integration("Signing provider discovery in progress"));
            }
            crate::discovery::DiscoveryStatus::Failed(err) => {
                return Err(RhizoCryptError::integration(format!("Discovery failed: {err}")));
            }
            crate::discovery::DiscoveryStatus::Unavailable => {
                return Err(RhizoCryptError::integration(
                    "No signing provider available. \
                     Ensure discovery registry has at least one service providing 'crypto:signing'.",
                ));
            }
        };

        let service_name = Some(endpoint.service_id.as_ref().to_string());
        let endpoint_addr = endpoint.addr.to_string();

        tracing::info!(
            service = ?service_name,
            endpoint = %endpoint_addr,
            "✅ Discovered signing provider"
        );

        // Create protocol adapter based on endpoint
        let adapter = AdapterFactory::create(&endpoint_addr)?;

        Ok(Self {
            adapter: Arc::new(adapter),
            endpoint: endpoint_addr,
            service_name,
        })
    }

    /// Create client with explicit endpoint (for testing/override).
    ///
    /// # Arguments
    ///
    /// * `endpoint` - Service endpoint (e.g., "127.0.0.1:9500" for tarpc, or "http://..." when http-clients enabled)
    ///
    /// # Errors
    ///
    /// Returns error if endpoint is invalid or connection fails.
    pub fn with_endpoint(endpoint: &str) -> Result<Self> {
        let adapter = AdapterFactory::create(endpoint)?;

        Ok(Self {
            adapter: Arc::new(adapter),
            endpoint: endpoint.to_string(),
            service_name: None,
        })
    }

    /// Sign data with a DID.
    ///
    /// # Arguments
    ///
    /// * `data` - Data to sign
    /// * `signer` - DID of the signer
    ///
    /// # Errors
    ///
    /// Returns error if:
    /// - Service unavailable
    /// - Signing fails
    /// - DID not authorized
    pub async fn sign(&self, data: &[u8], signer: &Did) -> Result<Signature> {
        self.sign_owned(bytes::Bytes::copy_from_slice(data), signer).await
    }

    /// Sign data that is already owned as `Bytes` (zero-copy path).
    ///
    /// Prefer this over [`Self::sign`] when the caller already holds `Bytes` or `Vec<u8>`
    /// to avoid an extra allocation.
    ///
    /// # Errors
    ///
    /// Returns error if the signing service is unavailable or the DID is not authorized.
    pub async fn sign_owned(&self, data: bytes::Bytes, signer: &Did) -> Result<Signature> {
        tracing::debug!(
            signer = %signer,
            data_len = data.len(),
            "Requesting signature"
        );

        let request = SignRequest {
            data,
            signer: signer.clone(),
        };

        let response: SignResponse = self.adapter.call("sign", request).await?;

        Ok(Signature::from(response.signature))
    }

    /// Verify a signature.
    ///
    /// # Arguments
    ///
    /// * `data` - Original data
    /// * `signature` - Signature to verify
    /// * `signer` - DID of the signer
    ///
    /// # Errors
    ///
    /// Returns error if service unavailable or verification fails.
    pub async fn verify(&self, data: &[u8], signature: &Signature, signer: &Did) -> Result<bool> {
        self.verify_owned(bytes::Bytes::copy_from_slice(data), signature, signer).await
    }

    /// Verify a signature against owned data (zero-copy path).
    ///
    /// # Errors
    ///
    /// Returns error if service unavailable or verification fails.
    pub async fn verify_owned(
        &self,
        data: bytes::Bytes,
        signature: &Signature,
        signer: &Did,
    ) -> Result<bool> {
        tracing::debug!(
            signer = %signer,
            data_len = data.len(),
            "Verifying signature"
        );

        let request = VerifyRequest {
            data,
            signature: signature.0.clone(),
            signer: signer.clone(),
        };

        let response: VerifyResponse = self.adapter.call("verify", request).await?;

        Ok(response.valid)
    }

    /// Sign a vertex.
    ///
    /// # Arguments
    ///
    /// * `vertex` - Vertex to sign
    /// * `signer` - DID of the signer
    ///
    /// # Errors
    ///
    /// Returns error if signing fails.
    pub async fn sign_vertex(&self, vertex: &Vertex, signer: &Did) -> Result<Signature> {
        let canonical = vertex.to_canonical_bytes()?;
        self.sign_owned(canonical, signer).await
    }

    /// Verify a vertex signature.
    ///
    /// # Arguments
    ///
    /// * `vertex` - Vertex with signature
    ///
    /// # Errors
    ///
    /// Returns error if verification fails.
    pub async fn verify_vertex(&self, vertex: &Vertex) -> Result<bool> {
        if let (Some(sig), Some(agent)) = (&vertex.signature, &vertex.agent) {
            let canonical = vertex.to_canonical_bytes()?;
            self.verify_owned(canonical, sig, agent).await
        } else {
            Ok(false)
        }
    }

    /// Verify a DID.
    ///
    /// # Arguments
    ///
    /// * `did` - DID to verify
    ///
    /// # Errors
    ///
    /// Returns error if verification fails.
    pub async fn verify_did(&self, did: &Did) -> Result<bool> {
        tracing::debug!(did = %did, "Verifying DID");

        let request = VerifyDidRequest {
            did: did.clone(),
        };

        let response: VerifyDidResponse = self.adapter.call("verify_did", request).await?;

        Ok(response.valid)
    }

    /// Request an attestation.
    ///
    /// # Arguments
    ///
    /// * `attester` - DID of the attester
    /// * `summary` - Dehydration summary to attest
    ///
    /// # Errors
    ///
    /// Returns error if attestation fails.
    pub async fn request_attestation(
        &self,
        attester: &Did,
        summary: &DehydrationSummary,
    ) -> Result<Attestation> {
        tracing::debug!(attester = %attester, "Requesting attestation");

        let request = AttestRequest {
            attester: attester.clone(),
            summary: summary.clone(),
        };

        let response: AttestResponse = self.adapter.call("attest", request).await?;

        Ok(response.attestation)
    }

    /// Check if service is available.
    pub async fn is_available(&self) -> bool {
        self.adapter.is_healthy().await
    }

    /// Get service endpoint.
    #[must_use]
    pub fn endpoint(&self) -> &str {
        &self.endpoint
    }

    /// Get service name (if known).
    #[must_use]
    pub fn service_name(&self) -> Option<&str> {
        self.service_name.as_deref()
    }
}

// ============================================================================
// Request/Response DTOs
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
struct SignRequest {
    data: bytes::Bytes,
    signer: Did,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct SignResponse {
    signature: bytes::Bytes,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct VerifyRequest {
    data: bytes::Bytes,
    signature: bytes::Bytes,
    signer: Did,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct VerifyResponse {
    valid: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct VerifyDidRequest {
    did: Did,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct VerifyDidResponse {
    valid: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct AttestRequest {
    attester: Did,
    summary: DehydrationSummary,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct AttestResponse {
    attestation: Attestation,
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::discovery::{DiscoveryRegistry, ServiceEndpoint};
    use std::net::SocketAddr;

    #[test]
    fn test_signing_client_with_endpoint() {
        let client = SigningClient::with_endpoint("127.0.0.1:9500").unwrap();
        assert_eq!(client.endpoint(), "127.0.0.1:9500");
        assert!(client.service_name().is_none());
    }

    #[test]
    fn test_signing_client_invalid_endpoint() {
        let result = SigningClient::with_endpoint("not a url");
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_signing_client_availability() {
        let client = SigningClient::with_endpoint("127.0.0.1:9999").unwrap();
        // Should return false for non-existent service
        let available = client.is_available().await;
        // Note: This might be true or false depending on what's running
        // Just testing that the method doesn't panic
        let _ = available;
    }

    #[tokio::test]
    async fn test_signing_client_discover_no_providers() {
        let registry = DiscoveryRegistry::new("test-rhizocrypt");
        let result = SigningClient::discover(&registry).await;
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("No signing provider available"));
    }

    #[tokio::test]
    async fn test_signing_client_discover_with_provider() {
        let registry = DiscoveryRegistry::new("test-rhizocrypt");

        // Register a mock signing provider
        let addr: SocketAddr = "127.0.0.1:9500".parse().unwrap();
        let endpoint =
            ServiceEndpoint::new("test-signer".to_string(), addr, vec![Capability::Signing]);
        registry.register_endpoint(endpoint).await;

        let result = SigningClient::discover(&registry).await;
        assert!(result.is_ok());
        let client = result.unwrap();
        assert!(client.endpoint().contains("127.0.0.1:9500"));
        assert_eq!(client.service_name(), Some("test-signer"));
    }

    #[test]
    fn test_signing_request_serialization() {
        let did = Did::new("did:key:test");
        let request = SignRequest {
            data: bytes::Bytes::from_static(&[1, 2, 3]),
            signer: did,
        };

        let serialized = serde_json::to_string(&request).unwrap();
        assert!(serialized.contains("did:key:test"));

        let deserialized: SignRequest = serde_json::from_str(&serialized).unwrap();
        assert_eq!(&deserialized.data[..], &[1, 2, 3]);
    }

    #[test]
    fn test_signing_response_serialization() {
        let response = SignResponse {
            signature: bytes::Bytes::from_static(&[1, 2, 3, 4]),
        };

        let serialized = serde_json::to_string(&response).unwrap();
        let deserialized: SignResponse = serde_json::from_str(&serialized).unwrap();
        assert_eq!(&deserialized.signature[..], &[1, 2, 3, 4]);
    }

    #[test]
    fn test_verify_request_serialization() {
        let did = Did::new("did:key:verifier");
        let request = VerifyRequest {
            data: bytes::Bytes::from_static(&[5, 6, 7]),
            signature: bytes::Bytes::from_static(&[8, 9, 10]),
            signer: did,
        };

        let serialized = serde_json::to_string(&request).unwrap();
        let deserialized: VerifyRequest = serde_json::from_str(&serialized).unwrap();
        assert_eq!(&deserialized.data[..], &[5, 6, 7]);
        assert_eq!(&deserialized.signature[..], &[8, 9, 10]);
    }

    #[test]
    fn test_verify_response_serialization() {
        let response = VerifyResponse {
            valid: true,
        };
        let serialized = serde_json::to_string(&response).unwrap();
        let deserialized: VerifyResponse = serde_json::from_str(&serialized).unwrap();
        assert!(deserialized.valid);

        let response_false = VerifyResponse {
            valid: false,
        };
        let serialized = serde_json::to_string(&response_false).unwrap();
        let deserialized: VerifyResponse = serde_json::from_str(&serialized).unwrap();
        assert!(!deserialized.valid);
    }

    #[test]
    fn test_verify_did_request_serialization() {
        let did = Did::new("did:key:test123");
        let request = VerifyDidRequest {
            did: did.clone(),
        };

        let serialized = serde_json::to_string(&request).unwrap();
        assert!(serialized.contains("did:key:test123"));

        let deserialized: VerifyDidRequest = serde_json::from_str(&serialized).unwrap();
        assert_eq!(deserialized.did, did);
    }

    #[test]
    fn test_attest_request_serialization() {
        use crate::event::SessionOutcome;
        use crate::merkle::MerkleRoot;
        use crate::types::ContentHash;

        let attester = Did::new("did:key:attester");
        let summary = DehydrationSummary {
            session_id: crate::types::SessionId::new(uuid::Uuid::now_v7()),
            session_type: "test".to_string(),
            created_at: crate::types::Timestamp::now(),
            resolved_at: crate::types::Timestamp::now(),
            outcome: SessionOutcome::Success,
            merkle_root: MerkleRoot::new(ContentHash::from([0u8; 32])),
            vertex_count: 10,
            payload_bytes: 1000,
            results: vec![],
            agents: vec![],
            attestations: vec![],
        };
        let request = AttestRequest {
            attester,
            summary,
        };

        let serialized = serde_json::to_string(&request).unwrap();
        assert!(serialized.contains("did:key:attester"));

        let _deserialized: AttestRequest = serde_json::from_str(&serialized).unwrap();
        // Basic deserialization works
    }

    #[test]
    fn test_attest_response_serialization() {
        use crate::dehydration::{Attestation, AttestationStatement};
        use crate::types::ContentHash;

        let attestation = Attestation {
            attester: Did::new("did:key:attester"),
            statement: AttestationStatement::SessionSummary {
                summary_hash: ContentHash::from([1u8; 32]),
            },
            signature: bytes::Bytes::from_static(&[1, 2, 3, 4]),
            attested_at: crate::types::Timestamp::now(),
            verified: true,
        };
        let response = AttestResponse {
            attestation: attestation.clone(),
        };

        let serialized = serde_json::to_string(&response).unwrap();
        let deserialized: AttestResponse = serde_json::from_str(&serialized).unwrap();
        assert_eq!(deserialized.attestation.attester, attestation.attester);
        assert!(deserialized.attestation.verified);
    }

    #[test]
    fn test_verify_did_response_serialization() {
        let response = VerifyDidResponse {
            valid: true,
        };
        let serialized = serde_json::to_string(&response).unwrap();
        let deserialized: VerifyDidResponse = serde_json::from_str(&serialized).unwrap();
        assert!(deserialized.valid);

        let response_false = VerifyDidResponse {
            valid: false,
        };
        let serialized = serde_json::to_string(&response_false).unwrap();
        let deserialized: VerifyDidResponse = serde_json::from_str(&serialized).unwrap();
        assert!(!deserialized.valid);
    }

    #[test]
    fn test_sign_request_roundtrip() {
        let did = Did::new("did:key:roundtrip");
        let request = SignRequest {
            data: bytes::Bytes::from_static(&[10, 20, 30]),
            signer: did.clone(),
        };
        let serialized = serde_json::to_string(&request).unwrap();
        let deserialized: SignRequest = serde_json::from_str(&serialized).unwrap();
        assert_eq!(&deserialized.data[..], &[10, 20, 30]);
        assert_eq!(deserialized.signer, did);
    }

    #[test]
    fn test_sign_response_roundtrip() {
        let response = SignResponse {
            signature: bytes::Bytes::from_static(&[1, 2, 3, 4, 5, 6]),
        };
        let serialized = serde_json::to_string(&response).unwrap();
        let deserialized: SignResponse = serde_json::from_str(&serialized).unwrap();
        assert_eq!(&deserialized.signature[..], &response.signature[..]);
    }

    #[test]
    fn test_signing_client_clone() {
        let client = SigningClient::with_endpoint("127.0.0.1:9500").unwrap();
        let cloned = client.clone();
        assert_eq!(client.endpoint(), cloned.endpoint());
        assert_eq!(client.service_name(), cloned.service_name());
    }

    #[test]
    fn test_signing_client_debug() {
        let client = SigningClient::with_endpoint("127.0.0.1:9500").unwrap();
        let debug_str = format!("{client:?}");
        assert!(debug_str.contains("SigningClient"));
    }

    #[tokio::test]
    async fn test_signing_client_multiple_providers() {
        let registry = DiscoveryRegistry::new("test-rhizocrypt");

        // Register multiple signing providers
        let addr1: SocketAddr = "127.0.0.1:9500".parse().unwrap();
        registry
            .register_endpoint(ServiceEndpoint::new(
                "signer-1".to_string(),
                addr1,
                vec![Capability::Signing],
            ))
            .await;

        let addr2: SocketAddr = "127.0.0.1:9501".parse().unwrap();
        registry
            .register_endpoint(ServiceEndpoint::new(
                "signer-2".to_string(),
                addr2,
                vec![Capability::Signing],
            ))
            .await;

        // Discovery should return the first available
        let result = SigningClient::discover(&registry).await;
        assert!(result.is_ok());
        let client = result.unwrap();
        assert!(
            client.endpoint().contains("127.0.0.1:9500")
                || client.endpoint().contains("127.0.0.1:9501")
        );
    }

    #[test]
    fn test_signature_from_bytes() {
        let sig_bytes = vec![1, 2, 3, 4, 5];
        let signature = Signature::new(sig_bytes.clone());
        assert_eq!(signature.as_bytes(), &sig_bytes);
    }

    #[tokio::test]
    async fn test_signing_client_service_name_tracking() {
        let registry = DiscoveryRegistry::new("test-rhizocrypt");

        let addr: SocketAddr = "127.0.0.1:9500".parse().unwrap();
        let endpoint =
            ServiceEndpoint::new("beardog-hsm".to_string(), addr, vec![Capability::Signing]);
        registry.register_endpoint(endpoint).await;

        let client = SigningClient::discover(&registry).await.unwrap();
        assert_eq!(client.service_name(), Some("beardog-hsm"));
        assert!(client.endpoint().contains("127.0.0.1:9500"));
    }

    #[tokio::test]
    async fn test_verify_vertex_no_signature_or_agent() {
        use crate::event::EventType;
        use crate::vertex::VertexBuilder;

        let client = SigningClient::with_endpoint("127.0.0.1:9500").unwrap();
        let vertex = VertexBuilder::new(EventType::SessionStart).build();
        assert!(vertex.signature.is_none());
        assert!(vertex.agent.is_none());

        let result = client.verify_vertex(&vertex).await;
        assert!(result.is_ok());
        assert!(!result.unwrap());
    }

    #[tokio::test]
    async fn test_verify_vertex_no_signature_with_agent() {
        use crate::event::EventType;
        use crate::vertex::VertexBuilder;

        let client = SigningClient::with_endpoint("127.0.0.1:9500").unwrap();
        let vertex = VertexBuilder::new(EventType::SessionStart)
            .with_agent(Did::new("did:key:test"))
            .build();
        assert!(vertex.signature.is_none());
        assert!(vertex.agent.is_some());

        let result = client.verify_vertex(&vertex).await;
        assert!(result.is_ok());
        assert!(!result.unwrap());
    }

    #[tokio::test]
    async fn test_signing_client_discover_failed() {
        let registry = DiscoveryRegistry::new("test-rhizocrypt");
        registry.set_discovery_source("127.0.0.1:1".parse().unwrap()).await;

        let result = SigningClient::discover(&registry).await;
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(
            err_msg.contains("Discovery failed") || err_msg.contains("No signing provider"),
            "Expected discovery failure, got: {err_msg}"
        );
    }

    #[test]
    #[cfg(feature = "http-clients")]
    fn test_signing_client_endpoint_formats() {
        // Test various endpoint formats (HTTP/HTTPS - requires http-clients feature)
        let http_client = SigningClient::with_endpoint("http://localhost:9500").unwrap();
        assert_eq!(http_client.endpoint(), "http://localhost:9500");

        let https_client = SigningClient::with_endpoint("https://signing.example.com:443").unwrap();
        assert_eq!(https_client.endpoint(), "https://signing.example.com:443");

        // AdapterFactory auto-adds http:// for addresses without protocol
        let auto_http = SigningClient::with_endpoint("localhost:9500").unwrap();
        assert!(auto_http.endpoint().contains("localhost:9500"));
    }
}
