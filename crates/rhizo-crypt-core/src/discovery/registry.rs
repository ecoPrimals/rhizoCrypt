// SPDX-License-Identifier: AGPL-3.0-or-later
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

    /// Clear the configured discovery source (e.g., when Songbird is no longer used).
    pub async fn clear_discovery_source(&self) {
        *self.discovery_source.write().await = None;
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
            #[serde(default, deserialize_with = "deserialize_dual_capabilities")]
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
///
/// Accepts PascalCase, snake_case, and colon-delimited (`domain:operation`)
/// formats. This handles the three naming conventions seen across the
/// ecoPrimals ecosystem (groundSpring, neuralSpring, airSpring, wetSpring).
fn parse_capability(name: &str) -> Option<Capability> {
    match name {
        "DidVerification" | "did_verification" | "did:verification" => {
            Some(Capability::DidVerification)
        }
        "Signing" | "signing" | "crypto:signing" => Some(Capability::Signing),
        "SignatureVerification" | "signature_verification" | "crypto:verification" => {
            Some(Capability::SignatureVerification)
        }
        "Attestation" | "attestation" | "attestation:request" => Some(Capability::Attestation),
        "ServiceDiscovery" | "service_discovery" | "discovery:service" => {
            Some(Capability::ServiceDiscovery)
        }
        "PayloadStorage" | "payload_storage" | "payload:storage" => {
            Some(Capability::PayloadStorage)
        }
        "PayloadRetrieval" | "payload_retrieval" | "payload:retrieval" => {
            Some(Capability::PayloadRetrieval)
        }
        "PermanentCommit" | "permanent_commit" | "storage:permanent:commit" => {
            Some(Capability::PermanentCommit)
        }
        "SliceCheckout" | "slice_checkout" | "slice:checkout" => Some(Capability::SliceCheckout),
        "SliceResolution" | "slice_resolution" | "slice:resolution" => {
            Some(Capability::SliceResolution)
        }
        "ComputeOrchestration" | "compute_orchestration" | "compute:orchestration" => {
            Some(Capability::ComputeOrchestration)
        }
        "ComputeEvents" | "compute_events" | "compute:events" => Some(Capability::ComputeEvents),
        "ProvenanceQuery" | "provenance_query" | "provenance:query" => {
            Some(Capability::ProvenanceQuery)
        }
        "Attribution" | "attribution" | "provenance:attribution" => Some(Capability::Attribution),
        other if !other.is_empty() => Some(Capability::custom(other.to_string())),
        _ => None,
    }
}

/// Deserialize capabilities from dual formats: flat string array or nested objects.
///
/// Absorbed from groundSpring/neuralSpring/airSpring/wetSpring dual-format pattern.
/// Songbird may return capabilities as either:
/// - Flat: `["Signing", "did_verification"]`
/// - Nested: `[{"name": "Signing", "version": "1.0"}, ...]`
///
/// This deserializer normalizes both into `Vec<String>`.
/// Extracts capability names from any of the 4 ecosystem response formats.
///
/// - **Format A** (flat): `["dag.session.create", "health.check"]`
/// - **Format B** (nested objects): `[{"name": "dag.session.create", ...}]`
/// - **Format C** (wrapper): `{"capabilities": ["dag.session.create"]}` — biomeOS/neuralSpring
/// - **Format D** (double-nested): `{"capabilities": [{"name": "...", ...}]}` — toadStool S155+
///
/// Absorbed from airSpring v0.8.7 4-format parser. Any unknown shape is
/// silently skipped (graceful degradation).
pub fn extract_capabilities(value: &serde_json::Value) -> Vec<String> {
    match value {
        serde_json::Value::Array(arr) => extract_from_array(arr),
        serde_json::Value::Object(map) => extract_from_object(map),
        serde_json::Value::String(s) => vec![s.clone()],
        _ => Vec::new(),
    }
}

fn extract_from_object(map: &serde_json::Map<String, serde_json::Value>) -> Vec<String> {
    if let Some(inner) = map.get("capabilities").or_else(|| map.get("methods")) {
        return extract_capabilities(inner);
    }
    match map.get("name") {
        Some(serde_json::Value::String(name)) => vec![name.clone()],
        _ => Vec::new(),
    }
}

fn extract_from_array(arr: &[serde_json::Value]) -> Vec<String> {
    let mut caps = Vec::with_capacity(arr.len());
    for item in arr {
        match item {
            serde_json::Value::String(s) => caps.push(s.clone()),
            serde_json::Value::Object(map) => {
                if let Some(serde_json::Value::String(name)) = map.get("name") {
                    caps.push(name.clone());
                }
            }
            _ => {}
        }
    }
    caps
}

fn deserialize_dual_capabilities<'de, D>(
    deserializer: D,
) -> std::result::Result<Vec<String>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    use serde::de;
    use std::fmt;

    struct DualCapVisitor;

    impl<'de> de::Visitor<'de> for DualCapVisitor {
        type Value = Vec<String>;

        fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
            formatter.write_str("a list of capability strings or objects with a 'name' field")
        }

        fn visit_seq<A>(self, mut seq: A) -> std::result::Result<Vec<String>, A::Error>
        where
            A: de::SeqAccess<'de>,
        {
            let mut caps = Vec::with_capacity(seq.size_hint().unwrap_or(0));
            while let Some(item) = seq.next_element::<serde_json::Value>()? {
                match item {
                    serde_json::Value::String(s) => caps.push(s),
                    serde_json::Value::Object(ref map) => {
                        if let Some(serde_json::Value::String(name)) = map.get("name") {
                            caps.push(name.clone());
                        }
                    }
                    _ => {}
                }
            }
            Ok(caps)
        }
    }

    deserializer.deserialize_seq(DualCapVisitor)
}

#[cfg(test)]
#[expect(clippy::unwrap_used, reason = "test code")]
#[path = "registry_tests.rs"]
mod tests;
