// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2024–2026 ecoPrimals Project

//! Client provider - runtime-resolved capability endpoints.
//!
//! This wraps the discovery registry and provides a clean interface
//! for obtaining clients to other primals.

use super::{Capability, DiscoveryRegistry, ServiceEndpoint};
use crate::error::{Result, RhizoCryptError};
use std::sync::Arc;

/// Provider for integration clients that resolves at runtime.
///
/// This wraps the discovery registry and provides a clean interface
/// for obtaining clients to other primals.
pub struct ClientProvider {
    registry: Arc<DiscoveryRegistry>,
}

impl ClientProvider {
    /// Create a new client provider.
    #[must_use]
    pub const fn new(registry: Arc<DiscoveryRegistry>) -> Self {
        Self {
            registry,
        }
    }

    // ============================================================================
    // Capability-Based Discovery (Infant Discovery Model) 🥇
    // ============================================================================

    /// Check if signing capabilities are available.
    ///
    /// This checks for services that provide cryptographic signing,
    /// regardless of which service implements it (could be BearDog, YubiKey, CloudKMS, etc.).
    pub async fn has_signing(&self) -> bool {
        self.registry.is_available(&Capability::Signing).await
    }

    /// Check if DID verification capabilities are available.
    ///
    /// This checks for services that can verify DIDs and resolve them to public keys.
    pub async fn has_did_verification(&self) -> bool {
        self.registry.is_available(&Capability::DidVerification).await
    }

    /// Check if permanent storage capabilities are available.
    ///
    /// This checks for services that provide permanent, immutable storage,
    /// regardless of which service implements it (could be LoamSpine, Arweave, IPFS, etc.).
    pub async fn has_permanent_storage(&self) -> bool {
        self.registry.is_available(&Capability::PermanentCommit).await
    }

    /// Check if payload storage capabilities are available.
    ///
    /// This checks for services that provide ephemeral blob storage,
    /// regardless of which service implements it — discovered at runtime via capabilities.
    pub async fn has_payload_storage(&self) -> bool {
        self.registry.is_available(&Capability::PayloadStorage).await
    }

    /// Check if compute orchestration capabilities are available.
    ///
    /// This checks for services that can orchestrate compute tasks,
    /// regardless of which service implements it (could be ToadStool, Kubernetes, Nomad, etc.).
    pub async fn has_compute(&self) -> bool {
        self.registry.is_available(&Capability::ComputeOrchestration).await
    }

    /// Check if provenance query capabilities are available.
    ///
    /// This checks for services that can answer provenance queries,
    /// regardless of which service implements it (could be SweetGrass, custom ledger, etc.).
    pub async fn has_provenance(&self) -> bool {
        self.registry.is_available(&Capability::ProvenanceQuery).await
    }

    /// Get endpoint for signing capabilities.
    ///
    /// # Errors
    ///
    /// Returns an error if no signing service has been discovered.
    pub async fn signing_endpoint(&self) -> Result<ServiceEndpoint> {
        self.registry
            .get_endpoint(&Capability::Signing)
            .await
            .ok_or_else(|| RhizoCryptError::integration("No signing service discovered"))
    }

    /// Get endpoint for DID verification capabilities.
    ///
    /// # Errors
    ///
    /// Returns an error if no DID verification service has been discovered.
    pub async fn did_verification_endpoint(&self) -> Result<ServiceEndpoint> {
        self.registry
            .get_endpoint(&Capability::DidVerification)
            .await
            .ok_or_else(|| RhizoCryptError::integration("No DID verification service discovered"))
    }

    /// Get endpoint for permanent storage capabilities.
    ///
    /// # Errors
    ///
    /// Returns an error if no permanent storage service has been discovered.
    pub async fn permanent_storage_endpoint(&self) -> Result<ServiceEndpoint> {
        self.registry
            .get_endpoint(&Capability::PermanentCommit)
            .await
            .ok_or_else(|| RhizoCryptError::integration("No permanent storage service discovered"))
    }

    /// Get endpoint for payload storage capabilities.
    ///
    /// # Errors
    ///
    /// Returns an error if no payload storage service has been discovered.
    pub async fn payload_storage_endpoint(&self) -> Result<ServiceEndpoint> {
        self.registry
            .get_endpoint(&Capability::PayloadStorage)
            .await
            .ok_or_else(|| RhizoCryptError::integration("No payload storage service discovered"))
    }

    /// Get endpoint for compute orchestration capabilities.
    ///
    /// # Errors
    ///
    /// Returns an error if no compute service has been discovered.
    pub async fn compute_endpoint(&self) -> Result<ServiceEndpoint> {
        self.registry
            .get_endpoint(&Capability::ComputeOrchestration)
            .await
            .ok_or_else(|| RhizoCryptError::integration("No compute service discovered"))
    }

    /// Get endpoint for provenance query capabilities.
    ///
    /// # Errors
    ///
    /// Returns an error if no provenance service has been discovered.
    pub async fn provenance_endpoint(&self) -> Result<ServiceEndpoint> {
        self.registry
            .get_endpoint(&Capability::ProvenanceQuery)
            .await
            .ok_or_else(|| RhizoCryptError::integration("No provenance service discovered"))
    }
}

#[cfg(test)]
#[expect(clippy::unwrap_used, reason = "test code")]
mod tests {
    use super::*;

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn test_client_provider() {
        let registry = Arc::new(DiscoveryRegistry::new("rhizoCrypt"));
        let provider = ClientProvider::new(Arc::clone(&registry));

        assert!(!provider.has_signing().await);

        registry
            .register_endpoint(ServiceEndpoint::new(
                "signingService",
                "127.0.0.1:9000".parse().unwrap(),
                vec![Capability::DidVerification, Capability::Signing],
            ))
            .await;

        assert!(provider.has_signing().await);
        assert!(provider.signing_endpoint().await.is_ok());
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn test_client_provider_capability_based() {
        let registry = Arc::new(DiscoveryRegistry::new("rhizoCrypt"));
        let provider = ClientProvider::new(Arc::clone(&registry));

        // Nothing registered yet - capability-based checks
        assert!(!provider.has_signing().await);
        assert!(!provider.has_did_verification().await);
        assert!(provider.signing_endpoint().await.is_err());

        // Register a signing service (could be BearDog, YubiKey, CloudKMS, etc.)
        registry
            .register_endpoint(ServiceEndpoint::new(
                "signingService", // Deliberately generic name
                "127.0.0.1:9000".parse().unwrap(),
                vec![Capability::DidVerification, Capability::Signing],
            ))
            .await;

        // Now capability-based checks work
        assert!(provider.has_signing().await);
        assert!(provider.has_did_verification().await);
        assert!(provider.signing_endpoint().await.is_ok());
        assert!(provider.did_verification_endpoint().await.is_ok());
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn test_client_provider_permanent_storage() {
        let registry = Arc::new(DiscoveryRegistry::new("rhizoCrypt"));
        let provider = ClientProvider::new(Arc::clone(&registry));

        assert!(!provider.has_permanent_storage().await);
        assert!(provider.permanent_storage_endpoint().await.is_err());

        registry
            .register_endpoint(ServiceEndpoint::new(
                "permanentStore",
                "127.0.0.1:9001".parse().unwrap(),
                vec![Capability::PermanentCommit, Capability::SliceCheckout],
            ))
            .await;

        assert!(provider.has_permanent_storage().await);
        assert!(provider.permanent_storage_endpoint().await.is_ok());
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn test_client_provider_payload_storage() {
        let registry = Arc::new(DiscoveryRegistry::new("rhizoCrypt"));
        let provider = ClientProvider::new(Arc::clone(&registry));

        assert!(!provider.has_payload_storage().await);
        assert!(provider.payload_storage_endpoint().await.is_err());

        registry
            .register_endpoint(ServiceEndpoint::new(
                "payloadStore",
                "127.0.0.1:9002".parse().unwrap(),
                vec![Capability::PayloadStorage, Capability::PayloadRetrieval],
            ))
            .await;

        assert!(provider.has_payload_storage().await);
        assert!(provider.payload_storage_endpoint().await.is_ok());
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn test_resolution_with_fallback() {
        let registry = Arc::new(DiscoveryRegistry::new("rhizoCrypt"));
        let provider = ClientProvider::new(Arc::clone(&registry));

        // Register multiple endpoints for signing (fallback candidates)
        registry
            .register_endpoint(ServiceEndpoint::new(
                "bearDog1",
                "127.0.0.1:9000".parse().unwrap(),
                vec![Capability::Signing],
            ))
            .await;
        registry
            .register_endpoint(ServiceEndpoint::new(
                "bearDog2",
                "127.0.0.1:9001".parse().unwrap(),
                vec![Capability::Signing],
            ))
            .await;

        // Resolution should succeed with one of the fallback endpoints
        let endpoint = provider.signing_endpoint().await.unwrap();
        assert!(
            endpoint.service_id.as_ref() == "bearDog1"
                || endpoint.service_id.as_ref() == "bearDog2"
        );
        assert!(provider.has_signing().await);
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn test_resolution_error_handling() {
        let registry = Arc::new(DiscoveryRegistry::new("rhizoCrypt"));
        let provider = ClientProvider::new(Arc::clone(&registry));

        // No endpoints registered - all resolution should fail with descriptive errors
        let err = provider.signing_endpoint().await.unwrap_err();
        assert!(err.to_string().contains("signing"));
        assert!(err.to_string().contains("discovered"));

        let err = provider.did_verification_endpoint().await.unwrap_err();
        assert!(err.to_string().contains("DID verification"));

        let err = provider.permanent_storage_endpoint().await.unwrap_err();
        assert!(err.to_string().contains("permanent storage"));

        let err = provider.payload_storage_endpoint().await.unwrap_err();
        assert!(err.to_string().contains("payload storage"));

        let err = provider.compute_endpoint().await.unwrap_err();
        assert!(err.to_string().contains("compute"));

        let err = provider.provenance_endpoint().await.unwrap_err();
        assert!(err.to_string().contains("provenance"));
    }
}
