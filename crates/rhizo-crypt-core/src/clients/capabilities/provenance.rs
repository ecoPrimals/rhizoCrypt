// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2024–2026 ecoPrimals Project

//! Generic provenance client - works with ANY provenance provider.

use crate::clients::adapters::{AdapterFactory, ProtocolAdapter};
use crate::discovery::{Capability, DiscoveryRegistry};
use crate::error::{Result, RhizoCryptError};
use std::sync::Arc;

/// Generic provenance client - works with ANY provider.
#[derive(Debug, Clone)]
pub struct ProvenanceClient {
    adapter: Arc<Box<dyn ProtocolAdapter>>,
    endpoint: String,
    service_name: Option<String>,
}

impl ProvenanceClient {
    /// Discover and connect to ANY provenance provider.
    pub async fn discover(registry: &DiscoveryRegistry) -> Result<Self> {
        tracing::info!("🔍 Discovering provenance capability provider...");

        let status = registry.discover(&Capability::ProvenanceQuery).await;

        let endpoint = match status {
            crate::discovery::DiscoveryStatus::Available(endpoints) => {
                endpoints.into_iter().next().ok_or_else(|| {
                    RhizoCryptError::integration("No provenance providers in available list")
                })?
            }
            _ => return Err(RhizoCryptError::integration("No provenance provider available.")),
        };

        let service_name = Some(endpoint.service_id.as_ref().to_string());
        let endpoint_addr = endpoint.addr.to_string();

        tracing::info!(
            service = ?service_name,
            endpoint = %endpoint_addr,
            "✅ Discovered provenance provider"
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
    use crate::discovery::{Capability, DiscoveryRegistry};
    use std::net::SocketAddr;

    #[test]
    fn test_provenance_client_with_endpoint() {
        let client = ProvenanceClient::with_endpoint("127.0.0.1:9900").unwrap();
        assert_eq!(client.endpoint(), "127.0.0.1:9900");
        assert!(client.service_name().is_none());
    }

    #[test]
    fn test_provenance_client_with_tarpc_endpoint() {
        let client = ProvenanceClient::with_endpoint("tarpc://127.0.0.1:9901").unwrap();
        assert_eq!(client.endpoint(), "tarpc://127.0.0.1:9901");
    }

    #[test]
    fn test_provenance_client_with_unix_endpoint() {
        let client = ProvenanceClient::with_endpoint("/tmp/provenance.sock").unwrap();
        assert_eq!(client.endpoint(), "/tmp/provenance.sock");
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn test_provenance_client_discover_unavailable() {
        let registry = DiscoveryRegistry::new("rhizoCrypt");
        let result = ProvenanceClient::discover(&registry).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("No provenance provider"));
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn test_provenance_client_discover_success() {
        let registry = DiscoveryRegistry::new("rhizoCrypt");
        let addr: SocketAddr = "127.0.0.1:19900".parse().unwrap();
        registry
            .register_endpoint(crate::discovery::ServiceEndpoint::new(
                "provenance-test",
                addr,
                vec![Capability::ProvenanceQuery],
            ))
            .await;

        let client = ProvenanceClient::discover(&registry).await.unwrap();
        assert_eq!(client.endpoint(), "127.0.0.1:19900");
        assert_eq!(client.service_name(), Some("provenance-test"));
    }
}
