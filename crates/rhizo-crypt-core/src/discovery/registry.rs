// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2024–2026 ecoPrimals Project

//! Discovery registry - central point for capability-based service discovery.
//!
//! This implements the pattern where primals discover other primals at runtime
//! rather than having hardcoded knowledge of addresses.

use super::{Capability, ServiceEndpoint};
use crate::constants::{DISCOVERY_QUERY_TIMEOUT, DISCOVERY_RESPONSE_BUFFER_SIZE};
use std::borrow::Cow;
use std::collections::HashMap;
use std::net::SocketAddr;
use tokio::sync::RwLock;

/// Discovery status for a capability.
#[derive(Debug, Clone)]
pub enum DiscoveryStatus {
    /// Capability is available at one or more endpoints.
    Available(Vec<ServiceEndpoint>),
    /// Capability is being discovered.
    Discovering,
    /// Capability is unavailable (no providers found).
    Unavailable,
    /// Discovery failed with error.
    Failed(String),
}

impl DiscoveryStatus {
    /// Check if capability is available.
    #[inline]
    #[must_use]
    pub const fn is_available(&self) -> bool {
        matches!(self, Self::Available(_))
    }

    /// Get the first available endpoint, if any.
    #[must_use]
    pub fn first_endpoint(&self) -> Option<&ServiceEndpoint> {
        match self {
            Self::Available(endpoints) => endpoints.first(),
            _ => None,
        }
    }
}

/// The discovery registry - central point for capability-based service discovery.
///
/// This implements the pattern where primals discover other primals at runtime
/// rather than having hardcoded knowledge of addresses.
#[derive(Debug)]
pub struct DiscoveryRegistry {
    /// Known endpoints by capability.
    endpoints: RwLock<HashMap<Capability, Vec<ServiceEndpoint>>>,
    /// Discovery source (e.g., Songbird address).
    discovery_source: RwLock<Option<SocketAddr>>,
    /// Local primal name (self-knowledge only).
    local_primal: Cow<'static, str>,
}

impl DiscoveryRegistry {
    /// Create a new discovery registry with only self-knowledge.
    #[must_use]
    pub fn new(local_primal: impl Into<Cow<'static, str>>) -> Self {
        Self {
            endpoints: RwLock::new(HashMap::new()),
            discovery_source: RwLock::new(None),
            local_primal: local_primal.into(),
        }
    }

    /// Set the discovery source (e.g., Songbird endpoint).
    ///
    /// This is the only "configured" address - everything else is discovered.
    pub async fn set_discovery_source(&self, addr: SocketAddr) {
        *self.discovery_source.write().await = Some(addr);
    }

    /// Register a known endpoint (for bootstrap or testing).
    pub async fn register_endpoint(&self, endpoint: ServiceEndpoint) {
        let mut endpoints = self.endpoints.write().await;
        for cap in &endpoint.capabilities {
            endpoints.entry(cap.clone()).or_default().push(endpoint.clone());
        }
    }

    /// Discover endpoints for a capability.
    ///
    /// Strategy:
    /// 1. Check local cache for healthy endpoints
    /// 2. If cache miss and a discovery source (Songbird) is configured,
    ///    attempt a live `discovery.resolve` JSON-RPC query via Unix socket
    /// 3. Cache any discovered endpoints for future lookups
    /// 4. Return `Unavailable` when no endpoints can be found
    pub async fn discover(&self, capability: &Capability) -> DiscoveryStatus {
        // Check cache first
        {
            let endpoints = self.endpoints.read().await;
            if let Some(eps) = endpoints.get(capability) {
                let healthy: Vec<_> = eps.iter().filter(|e| e.is_healthy()).cloned().collect();
                if !healthy.is_empty() {
                    return DiscoveryStatus::Available(healthy);
                }
            }
        }

        // Try to discover via discovery source
        let source = self.discovery_source.read().await;
        let Some(source_addr) = *source else {
            return DiscoveryStatus::Unavailable;
        };
        drop(source);

        // Query Songbird for the capability
        match self.query_discovery_source(source_addr, capability).await {
            Ok(endpoints) if !endpoints.is_empty() => {
                // Cache the discovered endpoints
                {
                    let mut cached = self.endpoints.write().await;
                    for ep in &endpoints {
                        for cap in &ep.capabilities {
                            cached.entry(cap.clone()).or_default().push(ep.clone());
                        }
                    }
                }
                DiscoveryStatus::Available(endpoints)
            }
            Ok(_) => DiscoveryStatus::Unavailable,
            Err(e) => {
                tracing::debug!(
                    error = %e,
                    capability = ?capability,
                    "Discovery query failed"
                );
                DiscoveryStatus::Failed(e)
            }
        }
    }

    /// Query the discovery source (Songbird) for endpoints providing a capability.
    ///
    /// Uses a lightweight TCP JSON-RPC call to the Songbird discovery endpoint.
    async fn query_discovery_source(
        &self,
        source_addr: SocketAddr,
        capability: &Capability,
    ) -> std::result::Result<Vec<ServiceEndpoint>, String> {
        #[derive(serde::Deserialize)]
        struct DiscoveryResponse {
            #[serde(default)]
            result: Option<Vec<DiscoveredEndpoint>>,
        }

        #[derive(serde::Deserialize)]
        struct DiscoveredEndpoint {
            service_id: String,
            address: String,
            #[serde(default)]
            capabilities: Vec<String>,
        }

        use tokio::io::{AsyncReadExt, AsyncWriteExt};
        use tokio::net::TcpStream;

        let capability_name = format!("{capability:?}");

        let request_body = serde_json::json!({
            "jsonrpc": "2.0",
            "method": "discovery.resolve",
            "params": {
                "capability": capability_name,
                "requester": self.local_primal.as_ref()
            },
            "id": 1
        });

        let body_bytes = serde_json::to_vec(&request_body).map_err(|e| e.to_string())?;

        let header = format!(
            "POST /rpc HTTP/1.1\r\n\
             Host: {source_addr}\r\n\
             Content-Type: application/json\r\n\
             Content-Length: {}\r\n\
             Connection: close\r\n\
             \r\n",
            body_bytes.len()
        );

        let timeout = DISCOVERY_QUERY_TIMEOUT;

        let mut stream = tokio::time::timeout(timeout, TcpStream::connect(source_addr))
            .await
            .map_err(|_| format!("Discovery source connection timed out: {source_addr}"))?
            .map_err(|e| format!("Discovery source connection failed: {e}"))?;

        stream.write_all(header.as_bytes()).await.map_err(|e| e.to_string())?;
        stream.write_all(&body_bytes).await.map_err(|e| e.to_string())?;

        let mut response_buf = Vec::with_capacity(DISCOVERY_RESPONSE_BUFFER_SIZE);
        tokio::time::timeout(timeout, stream.read_to_end(&mut response_buf))
            .await
            .map_err(|_| "Discovery source read timed out".to_string())?
            .map_err(|e| format!("Discovery source read failed: {e}"))?;

        let body_start =
            response_buf.windows(4).position(|w| w == b"\r\n\r\n").map_or(0, |pos| pos + 4);

        let body = &response_buf[body_start..];

        let parsed: DiscoveryResponse = serde_json::from_slice(body)
            .map_err(|e| format!("Failed to parse discovery response: {e}"))?;

        let Some(discovered) = parsed.result else {
            return Ok(Vec::new());
        };

        let mut endpoints = Vec::with_capacity(discovered.len());
        for d in discovered {
            let Ok(addr) = d.address.parse::<SocketAddr>() else {
                continue;
            };
            let caps: Vec<Capability> =
                d.capabilities.iter().filter_map(|c| parse_capability(c)).collect();
            if !caps.is_empty() {
                endpoints.push(ServiceEndpoint::new(d.service_id, addr, caps));
            }
        }

        Ok(endpoints)
    }

    /// Get all known endpoints.
    pub async fn all_endpoints(&self) -> Vec<ServiceEndpoint> {
        let endpoints = self.endpoints.read().await;
        endpoints.values().flatten().cloned().collect()
    }

    /// Get the local primal name (self-knowledge).
    #[inline]
    #[must_use]
    pub fn local_name(&self) -> &str {
        &self.local_primal
    }

    /// Check if a capability is available.
    pub async fn is_available(&self, capability: &Capability) -> bool {
        self.discover(capability).await.is_available()
    }

    /// Get the first endpoint for a capability.
    pub async fn get_endpoint(&self, capability: &Capability) -> Option<ServiceEndpoint> {
        match self.discover(capability).await {
            DiscoveryStatus::Available(mut endpoints) => endpoints.pop(),
            _ => None,
        }
    }
}

/// Parse a capability name string into a `Capability` variant.
fn parse_capability(name: &str) -> Option<Capability> {
    match name {
        "DidVerification" | "did_verification" => Some(Capability::DidVerification),
        "Signing" | "signing" => Some(Capability::Signing),
        "SignatureVerification" | "signature_verification" => {
            Some(Capability::SignatureVerification)
        }
        "Attestation" | "attestation" => Some(Capability::Attestation),
        "ServiceDiscovery" | "service_discovery" => Some(Capability::ServiceDiscovery),
        "PayloadStorage" | "payload_storage" => Some(Capability::PayloadStorage),
        "PayloadRetrieval" | "payload_retrieval" => Some(Capability::PayloadRetrieval),
        "PermanentCommit" | "permanent_commit" => Some(Capability::PermanentCommit),
        "SliceCheckout" | "slice_checkout" => Some(Capability::SliceCheckout),
        "SliceResolution" | "slice_resolution" => Some(Capability::SliceResolution),
        "ComputeOrchestration" | "compute_orchestration" => Some(Capability::ComputeOrchestration),
        "ComputeEvents" | "compute_events" => Some(Capability::ComputeEvents),
        "ProvenanceQuery" | "provenance_query" => Some(Capability::ProvenanceQuery),
        "Attribution" | "attribution" => Some(Capability::Attribution),
        other if !other.is_empty() => Some(Capability::custom(other.to_string())),
        _ => None,
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn test_registry_self_knowledge() {
        let registry = DiscoveryRegistry::new("rhizoCrypt");

        // Without any registration, nothing is available
        assert!(!registry.is_available(&Capability::DidVerification).await);
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn test_registry_registration() {
        let registry = DiscoveryRegistry::new("rhizoCrypt");

        let endpoint = ServiceEndpoint::new(
            "bearDog",
            "127.0.0.1:9000".parse().unwrap(),
            vec![Capability::DidVerification, Capability::Signing],
        );

        registry.register_endpoint(endpoint).await;

        assert!(registry.is_available(&Capability::DidVerification).await);
        assert!(registry.is_available(&Capability::Signing).await);
        assert!(!registry.is_available(&Capability::PayloadStorage).await);
    }

    #[test]
    fn test_discovery_status() {
        let unavailable = DiscoveryStatus::Unavailable;
        assert!(!unavailable.is_available());
        assert!(unavailable.first_endpoint().is_none());

        let discovering = DiscoveryStatus::Discovering;
        assert!(!discovering.is_available());

        let failed = DiscoveryStatus::Failed("test error".to_string());
        assert!(!failed.is_available());

        let endpoint = ServiceEndpoint::new(
            "test",
            "127.0.0.1:9000".parse().unwrap(),
            vec![Capability::Signing],
        );
        let available = DiscoveryStatus::Available(vec![endpoint]);
        assert!(available.is_available());
        assert!(available.first_endpoint().is_some());
    }

    #[test]
    fn test_discovery_status_clone() {
        let status = DiscoveryStatus::Failed("error".to_string());
        let cloned = status;
        match cloned {
            DiscoveryStatus::Failed(msg) => assert_eq!(msg, "error"),
            _ => panic!("Clone failed"),
        }
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn test_registry_discover() {
        let registry = DiscoveryRegistry::new("rhizoCrypt");

        // Initially nothing available
        let status = registry.discover(&Capability::Signing).await;
        assert!(!status.is_available());

        // Register a service
        registry
            .register_endpoint(ServiceEndpoint::new(
                "bearDog",
                "127.0.0.1:9000".parse().unwrap(),
                vec![Capability::Signing],
            ))
            .await;

        // Now should be available
        let status = registry.discover(&Capability::Signing).await;
        assert!(status.is_available());
        assert!(status.first_endpoint().is_some());
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn test_registry_get_endpoint() {
        let registry = DiscoveryRegistry::new("rhizoCrypt");

        // Initially returns None
        assert!(registry.get_endpoint(&Capability::PayloadStorage).await.is_none());

        // Register services
        registry
            .register_endpoint(ServiceEndpoint::new(
                "nestGate",
                "127.0.0.1:9001".parse().unwrap(),
                vec![Capability::PayloadStorage],
            ))
            .await;

        // Now should return endpoint
        let endpoint = registry.get_endpoint(&Capability::PayloadStorage).await;
        assert!(endpoint.is_some());
        assert_eq!(endpoint.unwrap().service_id.as_ref(), "nestGate");
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn test_registry_local_name() {
        let registry = DiscoveryRegistry::new("myPrimal");
        assert_eq!(registry.local_name(), "myPrimal");

        // Test with Cow::Owned
        let owned_name = String::from("dynamicPrimal");
        let registry2 = DiscoveryRegistry::new(owned_name);
        assert_eq!(registry2.local_name(), "dynamicPrimal");
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn test_registry_set_discovery_source() {
        let registry = DiscoveryRegistry::new("rhizoCrypt");

        // Set discovery source
        let addr: SocketAddr = "127.0.0.1:8091".parse().unwrap();
        registry.set_discovery_source(addr).await;

        // Verify it's set by attempting discovery (will still return Unavailable
        // since we don't have a real Songbird, but the code path is exercised)
        let status = registry.discover(&Capability::ServiceDiscovery).await;
        // With source set but no cache, returns Unavailable (pending real discovery)
        assert!(!status.is_available());
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn test_registry_all_endpoints() {
        let registry = DiscoveryRegistry::new("rhizoCrypt");

        // Initially empty
        let all = registry.all_endpoints().await;
        assert!(all.is_empty());

        // Register multiple endpoints
        registry
            .register_endpoint(ServiceEndpoint::new(
                "bearDog",
                "127.0.0.1:9000".parse().unwrap(),
                vec![Capability::DidVerification, Capability::Signing],
            ))
            .await;
        registry
            .register_endpoint(ServiceEndpoint::new(
                "nestGate",
                "127.0.0.1:9001".parse().unwrap(),
                vec![Capability::PayloadStorage],
            ))
            .await;

        let all = registry.all_endpoints().await;
        // Each capability stores the endpoint, so we get duplicates in the flat list
        assert!(all.len() >= 2);
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn test_registry_multiple_endpoints_for_capability() {
        let registry = DiscoveryRegistry::new("rhizoCrypt");

        // Register two endpoints with the same capability
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

        // Discover should return both
        let status = registry.discover(&Capability::Signing).await;
        match status {
            DiscoveryStatus::Available(endpoints) => {
                assert_eq!(endpoints.len(), 2);
            }
            _ => panic!("Expected Available status"),
        }
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn test_registry_toadstool() {
        let registry = DiscoveryRegistry::new("rhizoCrypt");

        // Register ToadStool
        registry
            .register_endpoint(ServiceEndpoint::new(
                "toadStool",
                "127.0.0.1:9003".parse().unwrap(),
                vec![Capability::ComputeOrchestration, Capability::ComputeEvents],
            ))
            .await;

        // ToadStool capabilities should be discoverable
        assert!(registry.is_available(&Capability::ComputeOrchestration).await);
        assert!(registry.is_available(&Capability::ComputeEvents).await);

        // Verify endpoint
        let endpoint = registry.get_endpoint(&Capability::ComputeOrchestration).await;
        assert!(endpoint.is_some());
        assert_eq!(endpoint.unwrap().service_id.as_ref(), "toadStool");
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn test_registry_sweetgrass() {
        let registry = DiscoveryRegistry::new("rhizoCrypt");

        registry
            .register_endpoint(ServiceEndpoint::new(
                "sweetGrass",
                "127.0.0.1:9004".parse().unwrap(),
                vec![Capability::ProvenanceQuery, Capability::Attribution],
            ))
            .await;

        assert!(registry.is_available(&Capability::ProvenanceQuery).await);
        assert!(registry.is_available(&Capability::Attribution).await);

        let endpoint = registry.get_endpoint(&Capability::ProvenanceQuery).await;
        assert!(endpoint.is_some());
        let ep = endpoint.unwrap();
        assert_eq!(ep.service_id.as_ref(), "sweetGrass");
        assert_eq!(ep.addr.port(), 9004);
    }

    #[test]
    fn test_parse_capability_all_variants() {
        assert!(matches!(parse_capability("DidVerification"), Some(Capability::DidVerification)));
        assert!(matches!(parse_capability("did_verification"), Some(Capability::DidVerification)));
        assert!(matches!(parse_capability("Signing"), Some(Capability::Signing)));
        assert!(matches!(parse_capability("signing"), Some(Capability::Signing)));
        assert!(matches!(
            parse_capability("SignatureVerification"),
            Some(Capability::SignatureVerification)
        ));
        assert!(matches!(
            parse_capability("signature_verification"),
            Some(Capability::SignatureVerification)
        ));
        assert!(matches!(parse_capability("Attestation"), Some(Capability::Attestation)));
        assert!(matches!(parse_capability("attestation"), Some(Capability::Attestation)));
        assert!(matches!(parse_capability("ServiceDiscovery"), Some(Capability::ServiceDiscovery)));
        assert!(matches!(
            parse_capability("service_discovery"),
            Some(Capability::ServiceDiscovery)
        ));
        assert!(matches!(parse_capability("PayloadStorage"), Some(Capability::PayloadStorage)));
        assert!(matches!(parse_capability("payload_storage"), Some(Capability::PayloadStorage)));
        assert!(matches!(parse_capability("PayloadRetrieval"), Some(Capability::PayloadRetrieval)));
        assert!(matches!(
            parse_capability("payload_retrieval"),
            Some(Capability::PayloadRetrieval)
        ));
        assert!(matches!(parse_capability("PermanentCommit"), Some(Capability::PermanentCommit)));
        assert!(matches!(parse_capability("permanent_commit"), Some(Capability::PermanentCommit)));
        assert!(matches!(parse_capability("SliceCheckout"), Some(Capability::SliceCheckout)));
        assert!(matches!(parse_capability("slice_checkout"), Some(Capability::SliceCheckout)));
        assert!(matches!(parse_capability("SliceResolution"), Some(Capability::SliceResolution)));
        assert!(matches!(parse_capability("slice_resolution"), Some(Capability::SliceResolution)));
        assert!(matches!(
            parse_capability("ComputeOrchestration"),
            Some(Capability::ComputeOrchestration)
        ));
        assert!(matches!(
            parse_capability("compute_orchestration"),
            Some(Capability::ComputeOrchestration)
        ));
        assert!(matches!(parse_capability("ComputeEvents"), Some(Capability::ComputeEvents)));
        assert!(matches!(parse_capability("compute_events"), Some(Capability::ComputeEvents)));
        assert!(matches!(parse_capability("ProvenanceQuery"), Some(Capability::ProvenanceQuery)));
        assert!(matches!(parse_capability("provenance_query"), Some(Capability::ProvenanceQuery)));
        assert!(matches!(parse_capability("Attribution"), Some(Capability::Attribution)));
        assert!(matches!(parse_capability("attribution"), Some(Capability::Attribution)));
    }

    #[test]
    fn test_parse_capability_custom_and_empty() {
        let custom = parse_capability("MyCustomCapability");
        assert!(custom.is_some());
        match custom.unwrap() {
            Capability::Custom(name) => assert_eq!(name, "MyCustomCapability"),
            _ => panic!("Expected Custom variant"),
        }

        assert!(parse_capability("").is_none());
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn test_discover_unhealthy_endpoints_filtered() {
        let registry = DiscoveryRegistry::new("rhizoCrypt");

        let mut endpoint = ServiceEndpoint::new(
            "bearDog",
            "127.0.0.1:9000".parse().unwrap(),
            vec![Capability::Signing],
        );
        endpoint.last_healthy =
            std::time::Instant::now().checked_sub(std::time::Duration::from_secs(300)).unwrap();

        registry.register_endpoint(endpoint).await;

        let status = registry.discover(&Capability::Signing).await;
        assert!(!status.is_available(), "Unhealthy endpoints should be filtered out");
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn test_discover_with_source_connection_refused() {
        let registry = DiscoveryRegistry::new("rhizoCrypt");
        registry.set_discovery_source("127.0.0.1:1".parse().unwrap()).await;

        let status = registry.discover(&Capability::Signing).await;
        match status {
            DiscoveryStatus::Failed(msg) => {
                assert!(
                    msg.contains("connection")
                        || msg.contains("Connection")
                        || msg.contains("timed out"),
                    "Expected connection error, got: {msg}"
                );
            }
            _ => panic!("Expected Failed status from unreachable discovery source"),
        }
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn test_get_endpoint_with_multiple() {
        let registry = DiscoveryRegistry::new("rhizoCrypt");

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

        let ep = registry.get_endpoint(&Capability::Signing).await;
        assert!(ep.is_some());
    }
}
