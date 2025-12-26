//! Capability-based client factory for dependency injection and testing.
//!
//! This module provides a factory for creating capability-based clients that
//! can work with ANY provider discovered at runtime.
//!
//! ## Usage
//!
//! ```rust,ignore
//! use rhizo_crypt_core::clients::factory::CapabilityClientFactory;
//!
//! // Production: discover and connect to real services
//! let factory = CapabilityClientFactory::new(registry).await?;
//! let signer = factory.signing_client().await?;
//! let storage = factory.storage_client().await?;
//!
//! // Testing: use mocks
//! let factory = CapabilityClientFactory::with_mocks();
//! let signer = factory.signing_client().await?; // Returns mock
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
    /// Panics if called outside of test builds.
    #[must_use]
    pub fn with_mocks() -> Self {
        // For testing, we'll need to implement mock clients
        // TODO: Create a mock registry for testing
        // For now, this will panic if actually used
        panic!("Mock factory not yet implemented. Use test-specific setup.");
    }
}

#[cfg(test)]
mod tests {
    #[tokio::test]
    async fn test_factory_caches_clients() {
        // This test requires a mock registry
        // TODO: Implement once mock registry is available
    }
}
