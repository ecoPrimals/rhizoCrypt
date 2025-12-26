//! Generic storage client - works with ANY payload storage provider.
//!
//! This client provides content-addressed payload storage without knowing
//! or caring about who provides the service (NestGate, S3, IPFS, etc.).

use crate::clients::adapters::{AdapterFactory, ProtocolAdapter, ProtocolAdapterExt};
use crate::discovery::{Capability, DiscoveryRegistry};
use crate::error::{Result, RhizoCryptError};
use crate::types::PayloadRef;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

/// Generic storage client - works with ANY provider.
///
/// This client is vendor-agnostic. It works with any service that provides
/// payload storage capabilities: NestGate, S3, IPFS, etc.
#[derive(Debug, Clone)]
pub struct StorageClient {
    adapter: Arc<Box<dyn ProtocolAdapter>>,
    endpoint: String,
    service_name: Option<String>,
}

impl StorageClient {
    /// Discover and connect to ANY storage provider.
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
                return Err(RhizoCryptError::integration(format!("Storage discovery failed: {}", err)));
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
            adapter: Arc::new(adapter),
            endpoint: endpoint_addr,
            service_name,
        })
    }

    /// Create client with explicit endpoint.
    pub fn with_endpoint(endpoint: &str) -> Result<Self> {
        let adapter = AdapterFactory::create(endpoint)?;

        Ok(Self {
            adapter: Arc::new(adapter),
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
            data: data.to_vec(),
        };

        let response: StoreResponse = self.adapter.call("store", request).await?;

        Ok(PayloadRef::new(
            response.hash.try_into().map_err(|_| {
                RhizoCryptError::integration("Invalid hash length in response")
            })?,
            response.size,
        ))
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

        Ok(response.data.map(bytes::Bytes::from))
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
    data: Vec<u8>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct StoreResponse {
    hash: Vec<u8>,
    size: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct RetrieveRequest {
    hash: [u8; 32],
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct RetrieveResponse {
    data: Option<Vec<u8>>,
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

    #[test]
    fn test_storage_client_with_endpoint() {
        let client = StorageClient::with_endpoint("http://localhost:9600").unwrap();
        assert_eq!(client.endpoint(), "http://localhost:9600");
        assert!(client.service_name().is_none());
    }

    #[test]
    fn test_storage_client_invalid_endpoint() {
        let result = StorageClient::with_endpoint("not a url");
        assert!(result.is_err());
    }
}

