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

    #[cfg(test)]
    pub(crate) fn with_adapter(adapter: Box<dyn ProtocolAdapter>, endpoint: &str) -> Self {
        Self {
            adapter: Arc::new(adapter),
            endpoint: endpoint.to_string(),
            service_name: None,
        }
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
#[path = "signing_tests.rs"]
mod tests;
