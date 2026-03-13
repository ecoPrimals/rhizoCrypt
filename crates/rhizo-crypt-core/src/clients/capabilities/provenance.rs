// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2024–2026 ecoPrimals Project

//! Generic provenance client - works with ANY provenance provider.

use crate::clients::adapters::{AdapterFactory, ProtocolAdapter, ProtocolAdapterExt};
use crate::discovery::{Capability, DiscoveryRegistry};
use crate::error::{Result, RhizoCryptError};
use crate::types_ecosystem::provenance::{
    ProvenanceChain, SessionAttribution, VertexQuery, VertexRef,
};
use serde::{Deserialize, Serialize};
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

    /// Get all vertices related to a content hash.
    ///
    /// Queries the provenance provider for any vertices that reference
    /// the given data hash (e.g. via payload references).
    ///
    /// # Errors
    ///
    /// Returns error if the service is unavailable or the query fails.
    pub async fn get_vertices_for_data(&self, data_hash: [u8; 32]) -> Result<Vec<VertexRef>> {
        tracing::debug!("Querying vertices for data hash");
        let request = DataHashQuery {
            data_hash,
        };
        self.adapter.call("provenance.vertices_for_data", request).await
    }

    /// Get the full provenance chain for a vertex.
    ///
    /// Returns the causal ancestry of the given vertex, including all
    /// agents and data hashes involved.
    ///
    /// # Errors
    ///
    /// Returns error if the service is unavailable or the query fails.
    pub async fn get_provenance_chain(
        &self,
        vertex_id: crate::types::VertexId,
    ) -> Result<ProvenanceChain> {
        tracing::debug!(?vertex_id, "Querying provenance chain");
        let request = ProvenanceChainQuery {
            vertex_id: vertex_id.as_bytes().to_vec(),
        };
        self.adapter.call("provenance.chain", request).await
    }

    /// Query vertices by parameters (agent, event types, time range).
    ///
    /// # Errors
    ///
    /// Returns error if the service is unavailable or the query fails.
    pub async fn query_vertices(&self, query: VertexQuery) -> Result<Vec<VertexRef>> {
        tracing::debug!("Querying vertices by parameters");
        self.adapter.call("provenance.query", query).await
    }

    /// Get session attribution for a given session.
    ///
    /// Returns the full attribution breakdown: which agents contributed,
    /// what data was input/output, and the session's merkle root.
    ///
    /// # Errors
    ///
    /// Returns error if the service is unavailable or the query fails.
    pub async fn get_session_attribution(
        &self,
        session_id: crate::types::SessionId,
    ) -> Result<SessionAttribution> {
        tracing::debug!(%session_id, "Querying session attribution");
        let request = SessionAttributionQuery {
            session_id: session_id.to_string(),
        };
        self.adapter.call("provenance.session_attribution", request).await
    }
}

// ============================================================================
// Request DTOs
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
struct DataHashQuery {
    data_hash: [u8; 32],
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ProvenanceChainQuery {
    vertex_id: Vec<u8>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct SessionAttributionQuery {
    session_id: String,
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
