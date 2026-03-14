// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2024–2026 ecoPrimals Project

//! Capability-based client factory for dependency injection and testing.
//!
//! This module provides a factory for creating capability-based clients that
//! can work with ANY provider discovered at runtime.
//!
//! ## Usage
//!
//! ```no_run
//! # use rhizo_crypt_core::clients::CapabilityClientFactory;
//! # use std::sync::Arc;
//! # tokio::runtime::Runtime::new().unwrap().block_on(async {
//! # let registry = Arc::new(rhizo_crypt_core::discovery::DiscoveryRegistry::new("doc-test"));
//! # registry.register_endpoint(rhizo_crypt_core::discovery::ServiceEndpoint::new(
//! #     "test-signer", "127.0.0.1:9500".parse().unwrap(),
//! #     vec![rhizo_crypt_core::discovery::Capability::Signing],
//! # )).await;
//! # registry.register_endpoint(rhizo_crypt_core::discovery::ServiceEndpoint::new(
//! #     "test-storage", "127.0.0.1:9600".parse().unwrap(),
//! #     vec![rhizo_crypt_core::discovery::Capability::PayloadStorage],
//! # )).await;
//! // Production: discover and connect to real services
//! let factory = CapabilityClientFactory::new(registry);
//! let signer = factory.signing_client().await?;
//! let storage = factory.storage_client().await?;
//! # Ok::<(), rhizo_crypt_core::error::RhizoCryptError>(())
//! # });
//! ```

use crate::clients::capabilities::{
    ComputeClient, PermanentStorageClient, ProvenanceClient, SigningClient, StorageClient,
};
use crate::discovery::DiscoveryRegistry;
use crate::error::Result;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Factory for creating and caching capability-based clients.
///
/// This factory provides:
/// - Lazy initialization of clients (only created when needed)
/// - Client caching (same client instance reused)
/// - Mock support for testing
/// - Clean API for dependency injection
#[derive(Debug, Clone)]
pub struct CapabilityClientFactory {
    registry: Arc<DiscoveryRegistry>,
    cache: Arc<ClientCache>,
}

#[derive(Debug, Default)]
struct ClientCache {
    signing: RwLock<Option<Arc<SigningClient>>>,
    storage: RwLock<Option<Arc<StorageClient>>>,
    permanent: RwLock<Option<Arc<PermanentStorageClient>>>,
    compute: RwLock<Option<Arc<ComputeClient>>>,
    provenance: RwLock<Option<Arc<ProvenanceClient>>>,
}

impl CapabilityClientFactory {
    /// Create a new factory with the given discovery registry.
    ///
    /// Clients will be discovered and created lazily when requested.
    #[must_use]
    pub fn new(registry: Arc<DiscoveryRegistry>) -> Self {
        Self {
            registry,
            cache: Arc::new(ClientCache::default()),
        }
    }

    /// Get or create a signing client.
    ///
    /// The client is discovered using the registry and cached for subsequent calls.
    ///
    /// # Errors
    ///
    /// Returns `RhizoCryptError::Integration` if no signing service is discovered
    /// or connection fails.
    pub async fn signing_client(&self) -> Result<Arc<SigningClient>> {
        // Check cache first
        {
            let cache = self.cache.signing.read().await;
            if let Some(client) = cache.as_ref() {
                return Ok(Arc::clone(client));
            }
        }

        // Create new client
        let client = SigningClient::discover(&self.registry).await?;
        let client = Arc::new(client);

        // Cache it
        {
            let mut cache = self.cache.signing.write().await;
            *cache = Some(Arc::clone(&client));
        }

        Ok(client)
    }

    /// Get or create a storage client.
    ///
    /// The client is discovered using the registry and cached for subsequent calls.
    ///
    /// # Errors
    ///
    /// Returns `RhizoCryptError::Integration` if no storage service is discovered
    /// or connection fails.
    pub async fn storage_client(&self) -> Result<Arc<StorageClient>> {
        // Check cache first
        {
            let cache = self.cache.storage.read().await;
            if let Some(client) = cache.as_ref() {
                return Ok(Arc::clone(client));
            }
        }

        // Create new client
        let client = StorageClient::discover(&self.registry).await?;
        let client = Arc::new(client);

        // Cache it
        {
            let mut cache = self.cache.storage.write().await;
            *cache = Some(Arc::clone(&client));
        }

        Ok(client)
    }

    /// Get or create a permanent storage client.
    ///
    /// The client is discovered using the registry and cached for subsequent calls.
    ///
    /// # Errors
    ///
    /// Returns `RhizoCryptError::Integration` if no permanent storage service
    /// is discovered or connection fails.
    pub async fn permanent_storage_client(&self) -> Result<Arc<PermanentStorageClient>> {
        // Check cache first
        {
            let cache = self.cache.permanent.read().await;
            if let Some(client) = cache.as_ref() {
                return Ok(Arc::clone(client));
            }
        }

        // Create new client
        let client = PermanentStorageClient::discover(&self.registry).await?;
        let client = Arc::new(client);

        // Cache it
        {
            let mut cache = self.cache.permanent.write().await;
            *cache = Some(Arc::clone(&client));
        }

        Ok(client)
    }

    /// Get or create a compute client.
    ///
    /// The client is discovered using the registry and cached for subsequent calls.
    ///
    /// # Errors
    ///
    /// Returns `RhizoCryptError::Integration` if no compute service is discovered
    /// or connection fails.
    pub async fn compute_client(&self) -> Result<Arc<ComputeClient>> {
        // Check cache first
        {
            let cache = self.cache.compute.read().await;
            if let Some(client) = cache.as_ref() {
                return Ok(Arc::clone(client));
            }
        }

        // Create new client
        let client = ComputeClient::discover(&self.registry).await?;
        let client = Arc::new(client);

        // Cache it
        {
            let mut cache = self.cache.compute.write().await;
            *cache = Some(Arc::clone(&client));
        }

        Ok(client)
    }

    /// Get or create a provenance client.
    ///
    /// The client is discovered using the registry and cached for subsequent calls.
    ///
    /// # Errors
    ///
    /// Returns `RhizoCryptError::Integration` if no provenance service is discovered
    /// or connection fails.
    pub async fn provenance_client(&self) -> Result<Arc<ProvenanceClient>> {
        // Check cache first
        {
            let cache = self.cache.provenance.read().await;
            if let Some(client) = cache.as_ref() {
                return Ok(Arc::clone(client));
            }
        }

        // Create new client
        let client = ProvenanceClient::discover(&self.registry).await?;
        let client = Arc::new(client);

        // Cache it
        {
            let mut cache = self.cache.provenance.write().await;
            *cache = Some(Arc::clone(&client));
        }

        Ok(client)
    }

    /// Clear the client cache.
    ///
    /// Forces all clients to be re-discovered on next request.
    /// Useful for testing or when registry state changes.
    pub async fn clear_cache(&self) {
        let mut signing = self.cache.signing.write().await;
        let mut storage = self.cache.storage.write().await;
        let mut permanent = self.cache.permanent.write().await;
        let mut compute = self.cache.compute.write().await;
        let mut provenance = self.cache.provenance.write().await;

        *signing = None;
        *storage = None;
        *permanent = None;
        *compute = None;
        *provenance = None;
    }
}

#[cfg(any(test, feature = "test-utils"))]
impl CapabilityClientFactory {
    /// Create a factory with mock clients for testing.
    ///
    /// This bypasses discovery and returns pre-configured mock clients.
    ///
    /// # Panics
    ///
    /// Create a factory with an empty registry for testing.
    ///
    /// This creates a factory with a mock-friendly registry that can be
    /// populated with test services.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use rhizo_crypt_core::clients::CapabilityClientFactory;
    /// # use rhizo_crypt_core::discovery::{DiscoveryRegistry, ServiceEndpoint, Capability};
    /// # use std::sync::Arc;
    /// # tokio::runtime::Runtime::new().unwrap().block_on(async {
    /// let registry = Arc::new(DiscoveryRegistry::new("doc-test"));
    /// // Register mock services
    /// registry.register_endpoint(ServiceEndpoint::new(
    ///     "mock-signer",
    ///     "127.0.0.1:9999".parse().unwrap(),
    ///     vec![Capability::Signing],
    /// )).await;
    ///
    /// let factory = CapabilityClientFactory::new(registry);
    /// # Ok::<(), rhizo_crypt_core::error::RhizoCryptError>(())
    /// # });
    /// ```
    #[must_use]
    pub fn with_mocks() -> Self {
        // Return a factory with an empty registry for test setup
        Self::new(Arc::new(DiscoveryRegistry::new("rhizoCrypt-test")))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::discovery::{Capability, ServiceEndpoint};

    #[tokio::test]
    async fn test_factory_creation() {
        let registry = Arc::new(DiscoveryRegistry::new("test-primal"));
        let factory = CapabilityClientFactory::new(registry);

        // Factory should be created successfully
        assert!(Arc::strong_count(&factory.registry) >= 1);
    }

    #[tokio::test]
    async fn test_factory_with_mocks() {
        let factory = CapabilityClientFactory::with_mocks();

        // Should create a factory with empty registry
        assert!(Arc::strong_count(&factory.registry) >= 1);
    }

    #[tokio::test]
    async fn test_signing_client_discovery() {
        let registry = Arc::new(DiscoveryRegistry::new("test-primal"));

        // Register a signing service via discovery
        let endpoint = ServiceEndpoint::new(
            "test-signer",
            "127.0.0.1:9500".parse().unwrap(),
            vec![Capability::Signing],
        );
        registry.register_endpoint(endpoint).await;

        let factory = CapabilityClientFactory::new(registry);

        // Should be able to get signing client
        let result = factory.signing_client().await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_storage_client_discovery() {
        let registry = Arc::new(DiscoveryRegistry::new("test-primal"));

        // Register a storage service via discovery
        let endpoint = ServiceEndpoint::new(
            "test-storage",
            "127.0.0.1:9600".parse().unwrap(),
            vec![Capability::PayloadStorage],
        );
        registry.register_endpoint(endpoint).await;

        let factory = CapabilityClientFactory::new(registry);

        // Should be able to get storage client
        let result = factory.storage_client().await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_compute_client_discovery() {
        let registry = Arc::new(DiscoveryRegistry::new("test-primal"));

        // Register a compute service via discovery
        let endpoint = ServiceEndpoint::new(
            "test-compute",
            "127.0.0.1:9800".parse().unwrap(),
            vec![Capability::ComputeOrchestration],
        );
        registry.register_endpoint(endpoint).await;

        let factory = CapabilityClientFactory::new(registry);

        // Should be able to get compute client
        let result = factory.compute_client().await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_permanent_storage_client_discovery() {
        let registry = Arc::new(DiscoveryRegistry::new("test-primal"));

        // Register a permanent storage service via discovery
        let endpoint = ServiceEndpoint::new(
            "test-loamspine",
            "127.0.0.1:9700".parse().unwrap(),
            vec![Capability::PermanentCommit, Capability::SliceCheckout],
        );
        registry.register_endpoint(endpoint).await;

        let factory = CapabilityClientFactory::new(registry);

        // Should be able to get permanent storage client
        let result = factory.permanent_storage_client().await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_client_caching() {
        let registry = Arc::new(DiscoveryRegistry::new("test-primal"));

        // Register a signing service via discovery
        let endpoint = ServiceEndpoint::new(
            "test-signer",
            "127.0.0.1:9500".parse().unwrap(),
            vec![Capability::Signing],
        );
        registry.register_endpoint(endpoint).await;

        let factory = CapabilityClientFactory::new(registry.clone());

        // Get client twice
        let client1 = factory.signing_client().await;
        let client2 = factory.signing_client().await;

        assert!(client1.is_ok());
        assert!(client2.is_ok());

        // Both should use the same endpoint (discovered, not hardcoded)
        let client1_unwrapped = client1.unwrap();
        let client2_unwrapped = client2.unwrap();
        let endpoint1 = client1_unwrapped.endpoint();
        let endpoint2 = client2_unwrapped.endpoint();
        assert_eq!(endpoint1, endpoint2);
    }

    #[tokio::test]
    async fn test_missing_capability() {
        let registry = Arc::new(DiscoveryRegistry::new("test-primal"));
        let factory = CapabilityClientFactory::new(registry);

        // Try to get signing client when no signing service is registered
        let result = factory.signing_client().await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_multiple_services_same_capability() {
        let registry = Arc::new(DiscoveryRegistry::new("test-primal"));

        // Register multiple signing services - demonstrates capability-based discovery
        let endpoint1 = ServiceEndpoint::new(
            "signer-1",
            "127.0.0.1:9500".parse().unwrap(),
            vec![Capability::Signing],
        );
        registry.register_endpoint(endpoint1).await;

        let endpoint2 = ServiceEndpoint::new(
            "signer-2",
            "127.0.0.1:9501".parse().unwrap(),
            vec![Capability::Signing],
        );
        registry.register_endpoint(endpoint2).await;

        let factory = CapabilityClientFactory::new(registry);

        // Should discover one of the available signing services
        let result = factory.signing_client().await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_factory_clone() {
        let registry = Arc::new(DiscoveryRegistry::new("test-primal"));
        let factory1 = CapabilityClientFactory::new(registry);
        let factory2 = factory1.clone();

        // Both should share the same registry
        assert!(Arc::ptr_eq(&factory1.registry, &factory2.registry));
    }

    #[tokio::test]
    async fn test_concurrent_client_creation() {
        let registry = Arc::new(DiscoveryRegistry::new("test-primal"));

        // Register services via discovery
        let signing_endpoint = ServiceEndpoint::new(
            "signer",
            "127.0.0.1:9500".parse().unwrap(),
            vec![Capability::Signing],
        );
        registry.register_endpoint(signing_endpoint).await;

        let storage_endpoint = ServiceEndpoint::new(
            "storage",
            "127.0.0.1:9600".parse().unwrap(),
            vec![Capability::PayloadStorage],
        );
        registry.register_endpoint(storage_endpoint).await;

        let factory = Arc::new(CapabilityClientFactory::new(registry));

        // Create clients concurrently - tests lock-free discovery
        let factory1 = factory.clone();
        let factory2 = factory.clone();

        let handle1 = tokio::spawn(async move { factory1.signing_client().await });

        let handle2 = tokio::spawn(async move { factory2.storage_client().await });

        let result1 = handle1.await.unwrap();
        let result2 = handle2.await.unwrap();

        assert!(result1.is_ok());
        assert!(result2.is_ok());
    }
}
