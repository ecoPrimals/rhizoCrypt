//! Generic compute client - works with ANY compute provider.

use crate::clients::adapters::{AdapterFactory, ProtocolAdapter};
use crate::discovery::{Capability, DiscoveryRegistry};
use crate::error::{Result, RhizoCryptError};
use std::sync::Arc;

/// Generic compute client - works with ANY provider.
#[derive(Debug, Clone)]
pub struct ComputeClient {
    adapter: Arc<Box<dyn ProtocolAdapter>>,
    endpoint: String,
    service_name: Option<String>,
}

impl ComputeClient {
    /// Discover and connect to ANY compute provider.
    pub async fn discover(registry: &DiscoveryRegistry) -> Result<Self> {
        tracing::info!("🔍 Discovering compute capability provider...");

        let status = registry.discover(&Capability::ComputeOrchestration).await;

        let endpoint = match status {
            crate::discovery::DiscoveryStatus::Available(endpoints) => {
                endpoints.into_iter().next().ok_or_else(|| {
                    RhizoCryptError::integration("No compute providers in available list")
                })?
            }
            _ => return Err(RhizoCryptError::integration("No compute provider available.")),
        };

        let service_name = Some(endpoint.service_id.as_ref().to_string());
        let endpoint_addr = endpoint.addr.to_string();

        tracing::info!(
            service = ?service_name,
            endpoint = %endpoint_addr,
            "✅ Discovered compute provider"
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::discovery::{DiscoveryRegistry, ServiceEndpoint};
    use std::net::SocketAddr;

    #[test]
    fn test_compute_client_with_endpoint() {
        let client = ComputeClient::with_endpoint("http://localhost:9800").unwrap();
        assert_eq!(client.endpoint(), "http://localhost:9800");
        assert!(client.service_name().is_none());
    }

    #[test]
    fn test_compute_client_invalid_endpoint() {
        let result = ComputeClient::with_endpoint("not a url");
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_compute_client_availability() {
        let client = ComputeClient::with_endpoint("http://localhost:9999").unwrap();
        let available = client.is_available().await;
        // Just testing that the method doesn't panic
        let _ = available;
    }

    #[tokio::test]
    async fn test_compute_client_discover_no_providers() {
        let registry = DiscoveryRegistry::new("test-rhizocrypt");
        let result = ComputeClient::discover(&registry).await;
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("No compute provider available"));
    }

    #[tokio::test]
    async fn test_compute_client_discover_with_provider() {
        let registry = DiscoveryRegistry::new("test-rhizocrypt");

        // Register a mock compute provider
        let addr: SocketAddr = "127.0.0.1:9800".parse().unwrap();
        let endpoint = ServiceEndpoint::new(
            "test-compute".to_string(),
            addr,
            vec![Capability::ComputeOrchestration],
        );
        registry.register_endpoint(endpoint).await;

        let result = ComputeClient::discover(&registry).await;
        assert!(result.is_ok());
        let client = result.unwrap();
        // AdapterFactory adds http:// prefix
        assert!(client.endpoint().contains("127.0.0.1:9800"));
        assert_eq!(client.service_name(), Some("test-compute"));
    }

    #[test]
    fn test_compute_client_clone() {
        let client = ComputeClient::with_endpoint("http://localhost:9800").unwrap();
        let cloned = client.clone();
        assert_eq!(client.endpoint(), cloned.endpoint());
        assert_eq!(client.service_name(), cloned.service_name());
    }

    #[test]
    fn test_compute_client_debug() {
        let client = ComputeClient::with_endpoint("http://localhost:9800").unwrap();
        let debug_str = format!("{:?}", client);
        assert!(debug_str.contains("ComputeClient"));
    }

    #[tokio::test]
    async fn test_compute_client_multiple_providers() {
        let registry = DiscoveryRegistry::new("test-rhizocrypt");

        // Register multiple compute providers
        let addr1: SocketAddr = "127.0.0.1:9800".parse().unwrap();
        registry
            .register_endpoint(ServiceEndpoint::new(
                "toadstool-1".to_string(),
                addr1,
                vec![Capability::ComputeOrchestration],
            ))
            .await;

        let addr2: SocketAddr = "127.0.0.1:9801".parse().unwrap();
        registry
            .register_endpoint(ServiceEndpoint::new(
                "toadstool-2".to_string(),
                addr2,
                vec![Capability::ComputeOrchestration],
            ))
            .await;

        // Discovery should return the first available
        let result = ComputeClient::discover(&registry).await;
        assert!(result.is_ok());
        let client = result.unwrap();
        // Should get one of the registered endpoints (AdapterFactory adds http:// prefix)
        assert!(
            client.endpoint().contains("127.0.0.1:9800")
                || client.endpoint().contains("127.0.0.1:9801")
        );
    }

    #[tokio::test]
    async fn test_compute_client_service_name_tracking() {
        let registry = DiscoveryRegistry::new("test-rhizocrypt");

        let addr: SocketAddr = "127.0.0.1:9800".parse().unwrap();
        let endpoint = ServiceEndpoint::new(
            "toadstool-gpu".to_string(),
            addr,
            vec![Capability::ComputeOrchestration],
        );
        registry.register_endpoint(endpoint).await;

        let client = ComputeClient::discover(&registry).await.unwrap();
        assert_eq!(client.service_name(), Some("toadstool-gpu"));
        assert!(client.endpoint().contains("127.0.0.1:9800"));
    }

    #[test]
    fn test_compute_client_endpoint_formats() {
        // Test various endpoint formats
        let http_client = ComputeClient::with_endpoint("http://localhost:9800").unwrap();
        assert_eq!(http_client.endpoint(), "http://localhost:9800");

        let https_client = ComputeClient::with_endpoint("https://compute.example.com:443").unwrap();
        assert_eq!(https_client.endpoint(), "https://compute.example.com:443");

        // AdapterFactory auto-adds http:// for addresses without protocol
        let auto_http = ComputeClient::with_endpoint("localhost:9800").unwrap();
        assert!(auto_http.endpoint().contains("localhost:9800"));
    }

    #[tokio::test]
    async fn test_compute_client_discovery_different_capabilities() {
        let registry = DiscoveryRegistry::new("test-rhizocrypt");

        // Register provider with ComputeOrchestration
        let addr1: SocketAddr = "127.0.0.1:9800".parse().unwrap();
        registry
            .register_endpoint(ServiceEndpoint::new(
                "compute-orchestrator".to_string(),
                addr1,
                vec![Capability::ComputeOrchestration],
            ))
            .await;

        // Should discover orchestration provider
        let result = ComputeClient::discover(&registry).await;
        assert!(result.is_ok());

        // Register provider with different capability (should not be discovered)
        let addr2: SocketAddr = "127.0.0.1:9900".parse().unwrap();
        registry
            .register_endpoint(ServiceEndpoint::new(
                "signing-service".to_string(),
                addr2,
                vec![Capability::Signing],
            ))
            .await;

        // Should still find compute provider, not signing
        let client = ComputeClient::discover(&registry).await.unwrap();
        assert!(client.service_name().unwrap().contains("compute"));
    }

    #[test]
    fn test_compute_client_various_addresses() {
        // IPv4
        let ipv4 = ComputeClient::with_endpoint("http://192.168.1.1:9800").unwrap();
        assert_eq!(ipv4.endpoint(), "http://192.168.1.1:9800");

        // IPv6
        let ipv6 = ComputeClient::with_endpoint("http://[::1]:9800").unwrap();
        assert_eq!(ipv6.endpoint(), "http://[::1]:9800");

        // Domain
        let domain = ComputeClient::with_endpoint("http://toadstool.example.com:9800").unwrap();
        assert_eq!(domain.endpoint(), "http://toadstool.example.com:9800");
    }

    #[tokio::test]
    async fn test_compute_client_concurrent_discovery() {
        use std::sync::Arc;

        let registry = Arc::new(DiscoveryRegistry::new("test-rhizocrypt"));

        // Register compute provider
        let addr: SocketAddr = "127.0.0.1:9800".parse().unwrap();
        registry
            .register_endpoint(ServiceEndpoint::new(
                "shared-compute".to_string(),
                addr,
                vec![Capability::ComputeOrchestration],
            ))
            .await;

        // Discover from multiple tasks concurrently
        let mut handles = vec![];
        for _ in 0..5 {
            let reg = Arc::clone(&registry);
            let handle = tokio::spawn(async move { ComputeClient::discover(&*reg).await });
            handles.push(handle);
        }

        // All should succeed
        for handle in handles {
            let result = handle.await.unwrap();
            assert!(result.is_ok());
        }
    }
}
