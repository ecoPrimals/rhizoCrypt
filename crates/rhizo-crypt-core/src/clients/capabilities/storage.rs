// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2024–2026 ecoPrimals Project

//! Generic storage client - works with ANY payload storage provider.
//!
//! This client provides content-addressed payload storage without knowing
//! or caring about who provides the service.

use crate::clients::adapters::{AdapterFactory, ProtocolAdapter, ProtocolAdapterExt};
use crate::discovery::{Capability, DiscoveryRegistry};
use crate::error::{Result, RhizoCryptError};
use crate::types::PayloadRef;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

/// Generic storage client - works with ANY provider.
///
/// This client is vendor-agnostic. It works with any service that provides
/// payload storage capabilities from any provider.
#[derive(Debug, Clone)]
pub struct StorageClient {
    adapter: Arc<dyn ProtocolAdapter>,
    endpoint: String,
    service_name: Option<String>,
}

impl StorageClient {
    /// Discover and connect to ANY storage provider.
    ///
    /// # Errors
    ///
    /// Returns an error if storage is unavailable, discovery is in progress, discovery
    /// failed, the available list is empty, or adapter creation fails for the resolved address.
    pub async fn discover(registry: &DiscoveryRegistry) -> Result<Self> {
        tracing::info!("🔍 Discovering payload storage capability provider...");

        let status = registry.discover(&Capability::PayloadStorage).await;

        let endpoint = match status {
            crate::discovery::DiscoveryStatus::Available(endpoints) => {
                endpoints.into_iter().next().ok_or_else(|| {
                    RhizoCryptError::integration("No storage providers in available list")
                })?
            }
            crate::discovery::DiscoveryStatus::Unavailable => {
                return Err(RhizoCryptError::integration(
                    "No storage provider available. \
                     Ensure discovery registry has at least one service providing 'payload-storage'.",
                ));
            }
            crate::discovery::DiscoveryStatus::Discovering => {
                return Err(RhizoCryptError::integration("Storage discovery in progress"));
            }
            crate::discovery::DiscoveryStatus::Failed(err) => {
                return Err(RhizoCryptError::integration(format!(
                    "Storage discovery failed: {err}"
                )));
            }
        };

        let service_name = Some(endpoint.service_id.as_ref().to_string());
        let endpoint_addr = endpoint.addr.to_string();

        tracing::info!(
            service = ?service_name,
            endpoint = %endpoint_addr,
            "✅ Discovered storage provider"
        );

        let adapter = AdapterFactory::create(&endpoint_addr)?;

        Ok(Self {
            adapter: Arc::from(adapter),
            endpoint: endpoint_addr,
            service_name,
        })
    }

    /// Create client with explicit endpoint.
    ///
    /// # Errors
    ///
    /// Returns an error if [`AdapterFactory::create`] fails for the given endpoint.
    pub fn with_endpoint(endpoint: &str) -> Result<Self> {
        let adapter = AdapterFactory::create(endpoint)?;

        Ok(Self {
            adapter: Arc::from(adapter),
            endpoint: endpoint.to_string(),
            service_name: None,
        })
    }

    /// Store payload and get content-addressed reference.
    ///
    /// # Arguments
    ///
    /// * `data` - Payload bytes to store
    ///
    /// # Errors
    ///
    /// Returns error if storage fails.
    pub async fn store(&self, data: bytes::Bytes) -> Result<PayloadRef> {
        tracing::debug!(size = data.len(), "Storing payload");

        let request = StoreRequest {
            data,
        };

        let response: StoreResponse = self.adapter.call("store", request).await?;

        let hash: [u8; 32] = response.hash[..]
            .try_into()
            .map_err(|_| RhizoCryptError::integration("Invalid hash length in response"))?;
        Ok(PayloadRef::new(hash, response.size))
    }

    /// Retrieve payload by reference.
    ///
    /// # Arguments
    ///
    /// * `payload_ref` - Content-addressed reference
    ///
    /// # Errors
    ///
    /// Returns error if:
    /// - Payload not found
    /// - Retrieval fails
    pub async fn retrieve(&self, payload_ref: &PayloadRef) -> Result<Option<bytes::Bytes>> {
        tracing::debug!(hash = ?payload_ref.hash, "Retrieving payload");

        let request = RetrieveRequest {
            hash: payload_ref.hash,
        };

        let response: RetrieveResponse = self.adapter.call("retrieve", request).await?;

        Ok(response.data)
    }

    /// Check if payload exists.
    ///
    /// # Arguments
    ///
    /// * `payload_ref` - Content-addressed reference
    ///
    /// # Errors
    ///
    /// Returns error if check fails.
    pub async fn exists(&self, payload_ref: &PayloadRef) -> Result<bool> {
        tracing::debug!(hash = ?payload_ref.hash, "Checking payload existence");

        let request = ExistsRequest {
            hash: payload_ref.hash,
        };

        let response: ExistsResponse = self.adapter.call("exists", request).await?;

        Ok(response.exists)
    }

    /// Delete payload (if supported by provider).
    ///
    /// # Arguments
    ///
    /// * `payload_ref` - Content-addressed reference
    ///
    /// # Errors
    ///
    /// Returns error if deletion fails or is not supported.
    pub async fn delete(&self, payload_ref: &PayloadRef) -> Result<()> {
        tracing::debug!(hash = ?payload_ref.hash, "Deleting payload");

        let request = DeleteRequest {
            hash: payload_ref.hash,
        };

        let _response: DeleteResponse = self.adapter.call("delete", request).await?;

        Ok(())
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
struct StoreRequest {
    data: bytes::Bytes,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct StoreResponse {
    hash: bytes::Bytes,
    size: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct RetrieveRequest {
    hash: [u8; 32],
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct RetrieveResponse {
    data: Option<bytes::Bytes>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ExistsRequest {
    hash: [u8; 32],
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ExistsResponse {
    exists: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct DeleteRequest {
    hash: [u8; 32],
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct DeleteResponse {}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::discovery::{DiscoveryRegistry, ServiceEndpoint};
    use std::net::SocketAddr;

    #[test]
    fn test_storage_client_with_endpoint() {
        let client = StorageClient::with_endpoint("127.0.0.1:9600").unwrap();
        assert_eq!(client.endpoint(), "127.0.0.1:9600");
        assert!(client.service_name().is_none());
    }

    #[test]
    fn test_storage_client_invalid_endpoint() {
        let result = StorageClient::with_endpoint("not a url");
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_storage_client_availability() {
        let client = StorageClient::with_endpoint("127.0.0.1:9999").unwrap();
        let available = client.is_available().await;
        // Just testing that the method doesn't panic
        let _ = available;
    }

    #[tokio::test]
    async fn test_storage_client_discover_no_providers() {
        let registry = DiscoveryRegistry::new("test-rhizocrypt");
        let result = StorageClient::discover(&registry).await;
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("No storage provider available"));
    }

    #[tokio::test]
    async fn test_storage_client_discover_with_provider() {
        let registry = DiscoveryRegistry::new("test-rhizocrypt");

        // Register a mock storage provider
        let addr: SocketAddr = "127.0.0.1:9600".parse().unwrap();
        let endpoint = ServiceEndpoint::new(
            "test-storage".to_string(),
            addr,
            vec![Capability::PayloadStorage],
        );
        registry.register_endpoint(endpoint).await;

        let result = StorageClient::discover(&registry).await;
        assert!(result.is_ok());
        let client = result.unwrap();
        assert!(client.endpoint().contains("127.0.0.1:9600"));
        assert_eq!(client.service_name(), Some("test-storage"));
    }

    #[test]
    fn test_store_request_serialization() {
        let request = StoreRequest {
            data: bytes::Bytes::from_static(&[1, 2, 3, 4, 5]),
        };

        let serialized = serde_json::to_string(&request).unwrap();
        let deserialized: StoreRequest = serde_json::from_str(&serialized).unwrap();
        assert_eq!(&deserialized.data[..], &[1, 2, 3, 4, 5]);
    }

    #[test]
    fn test_store_response_serialization() {
        let hash = bytes::Bytes::from(vec![0u8; 32]);
        let response = StoreResponse {
            hash,
            size: 12345,
        };

        let serialized = serde_json::to_string(&response).unwrap();
        let deserialized: StoreResponse = serde_json::from_str(&serialized).unwrap();
        assert_eq!(deserialized.hash.len(), 32);
        assert_eq!(deserialized.size, 12345);
    }

    #[test]
    fn test_retrieve_request_serialization() {
        let hash = [42u8; 32];
        let request = RetrieveRequest {
            hash,
        };

        let serialized = serde_json::to_string(&request).unwrap();
        let deserialized: RetrieveRequest = serde_json::from_str(&serialized).unwrap();
        assert_eq!(deserialized.hash, hash);
    }

    #[test]
    fn test_retrieve_response_serialization() {
        let response = RetrieveResponse {
            data: Some(bytes::Bytes::from_static(&[1, 2, 3])),
        };

        let serialized = serde_json::to_string(&response).unwrap();
        let deserialized: RetrieveResponse = serde_json::from_str(&serialized).unwrap();
        assert_eq!(deserialized.data.as_deref(), Some([1, 2, 3].as_slice()));

        let response_none = RetrieveResponse {
            data: None,
        };
        let serialized = serde_json::to_string(&response_none).unwrap();
        let deserialized: RetrieveResponse = serde_json::from_str(&serialized).unwrap();
        assert!(deserialized.data.is_none());
    }

    #[test]
    fn test_exists_request_serialization() {
        let hash = [99u8; 32];
        let request = ExistsRequest {
            hash,
        };

        let serialized = serde_json::to_string(&request).unwrap();
        let deserialized: ExistsRequest = serde_json::from_str(&serialized).unwrap();
        assert_eq!(deserialized.hash, hash);
    }

    #[test]
    fn test_exists_response_serialization() {
        let response_true = ExistsResponse {
            exists: true,
        };
        let serialized = serde_json::to_string(&response_true).unwrap();
        let deserialized: ExistsResponse = serde_json::from_str(&serialized).unwrap();
        assert!(deserialized.exists);

        let response_false = ExistsResponse {
            exists: false,
        };
        let serialized = serde_json::to_string(&response_false).unwrap();
        let deserialized: ExistsResponse = serde_json::from_str(&serialized).unwrap();
        assert!(!deserialized.exists);
    }

    #[test]
    fn test_delete_request_serialization() {
        let hash = [7u8; 32];
        let request = DeleteRequest {
            hash,
        };

        let serialized = serde_json::to_string(&request).unwrap();
        let deserialized: DeleteRequest = serde_json::from_str(&serialized).unwrap();
        assert_eq!(deserialized.hash, hash);
    }

    #[test]
    fn test_delete_response_serialization() {
        let response = DeleteResponse {};
        let serialized = serde_json::to_string(&response).unwrap();
        let _deserialized: DeleteResponse = serde_json::from_str(&serialized).unwrap();
        // Just verify it doesn't panic
    }

    #[test]
    fn test_storage_client_clone() {
        let client = StorageClient::with_endpoint("127.0.0.1:9600").unwrap();
        let cloned = client.clone();
        assert_eq!(client.endpoint(), cloned.endpoint());
        assert_eq!(client.service_name(), cloned.service_name());
    }

    #[test]
    fn test_storage_client_debug() {
        let client = StorageClient::with_endpoint("127.0.0.1:9600").unwrap();
        let debug_str = format!("{client:?}");
        assert!(debug_str.contains("StorageClient"));
    }

    #[tokio::test]
    async fn test_storage_client_multiple_providers() {
        let registry = DiscoveryRegistry::new("test-rhizocrypt");

        // Register multiple storage providers
        let addr1: SocketAddr = "127.0.0.1:9600".parse().unwrap();
        registry
            .register_endpoint(ServiceEndpoint::new(
                "storage-primary".to_string(),
                addr1,
                vec![Capability::PayloadStorage],
            ))
            .await;

        let addr2: SocketAddr = "127.0.0.1:9601".parse().unwrap();
        registry
            .register_endpoint(ServiceEndpoint::new(
                "storage-secondary".to_string(),
                addr2,
                vec![Capability::PayloadStorage],
            ))
            .await;

        // Discovery should return the first available
        let result = StorageClient::discover(&registry).await;
        assert!(result.is_ok());
        let client = result.unwrap();
        // Should get one of the registered endpoints
        assert!(
            client.endpoint().contains("127.0.0.1:9600")
                || client.endpoint().contains("127.0.0.1:9601")
        );
    }

    #[tokio::test]
    async fn test_storage_client_service_name_tracking() {
        let registry = DiscoveryRegistry::new("test-rhizocrypt");

        let addr: SocketAddr = "127.0.0.1:9600".parse().unwrap();
        let endpoint =
            ServiceEndpoint::new("storage-zfs".to_string(), addr, vec![Capability::PayloadStorage]);
        registry.register_endpoint(endpoint).await;

        let client = StorageClient::discover(&registry).await.unwrap();
        assert_eq!(client.service_name(), Some("storage-zfs"));
        assert!(client.endpoint().contains("127.0.0.1:9600"));
    }

    #[test]
    #[cfg(feature = "http-clients")]
    fn test_storage_client_endpoint_formats() {
        // Test various endpoint formats (http/https only when http-clients is enabled)
        let http_client = StorageClient::with_endpoint("http://localhost:9600").unwrap();
        assert_eq!(http_client.endpoint(), "http://localhost:9600");

        let https_client = StorageClient::with_endpoint("https://storage.example.com:443").unwrap();
        assert_eq!(https_client.endpoint(), "https://storage.example.com:443");

        // AdapterFactory auto-adds http:// for addresses without protocol
        let auto_http = StorageClient::with_endpoint("localhost:9600").unwrap();
        assert!(auto_http.endpoint().contains("localhost:9600"));
    }

    #[test]
    fn test_payload_ref_creation() {
        let hash = [123u8; 32];
        let payload_ref = PayloadRef::new(hash, 5000);
        assert_eq!(payload_ref.hash, hash);
        assert_eq!(payload_ref.size, 5000);
    }

    #[test]
    fn test_large_payload_serialization() {
        let large_data = bytes::Bytes::from(vec![42u8; 1024 * 1024]);
        let request = StoreRequest {
            data: large_data,
        };

        let serialized = serde_json::to_string(&request).unwrap();
        let deserialized: StoreRequest = serde_json::from_str(&serialized).unwrap();
        assert_eq!(deserialized.data.len(), 1024 * 1024);
        assert_eq!(deserialized.data[0], 42);
    }

    #[tokio::test]
    async fn test_storage_client_discover_with_multiple_capabilities() {
        let registry = DiscoveryRegistry::new("test-rhizocrypt");

        // Register endpoint that provides both PayloadStorage and Signing
        let addr: SocketAddr = "127.0.0.1:9600".parse().unwrap();
        let endpoint = ServiceEndpoint::new(
            "multi-cap-storage".to_string(),
            addr,
            vec![Capability::PayloadStorage, Capability::Signing],
        );
        registry.register_endpoint(endpoint).await;

        let result = StorageClient::discover(&registry).await;
        assert!(result.is_ok());
        let client = result.unwrap();
        assert_eq!(client.service_name(), Some("multi-cap-storage"));
        assert!(client.endpoint().contains("127.0.0.1:9600"));
    }

    #[test]
    fn test_store_request_with_empty_data() {
        let request = StoreRequest {
            data: bytes::Bytes::new(),
        };
        let serialized = serde_json::to_string(&request).unwrap();
        let deserialized: StoreRequest = serde_json::from_str(&serialized).unwrap();
        assert!(deserialized.data.is_empty());
    }

    #[test]
    fn test_retrieve_request_roundtrip() {
        let hash = [99u8; 32];
        let request = RetrieveRequest {
            hash,
        };
        let serialized = serde_json::to_string(&request).unwrap();
        let deserialized: RetrieveRequest = serde_json::from_str(&serialized).unwrap();
        assert_eq!(deserialized.hash, hash);
    }

    #[test]
    fn test_payload_ref_creation_roundtrip() {
        let data = b"test payload for roundtrip";
        let payload_ref = PayloadRef::from_bytes(data);
        let reconstructed = PayloadRef::new(payload_ref.hash, payload_ref.size);
        assert_eq!(payload_ref.hash, reconstructed.hash);
        assert_eq!(payload_ref.size, reconstructed.size);
    }

    #[test]
    fn test_payload_ref_from_hash() {
        let hash = [42u8; 32];
        let payload_ref = PayloadRef::from_hash(&hash);
        assert_eq!(payload_ref.hash, hash);
        assert_eq!(payload_ref.size, 0);
    }
}
