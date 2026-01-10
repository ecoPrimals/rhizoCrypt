//! Runtime primal discovery - capability-based service location.
//!
//! This module implements the ecoPrimals principle that primals have only self-knowledge
//! and discover other primals at runtime through capability-based discovery.
//!
//! ## Philosophy
//!
//! - **No hardcoded addresses** — Services are discovered, not configured
//! - **Capability-based** — Request what you need, not who provides it
//! - **Runtime resolution** — Bindings happen at runtime, not compile time
//! - **Graceful degradation** — Missing services don't crash, they report unavailability

use crate::error::{Result, RhizoCryptError};
use serde::{Deserialize, Serialize};
use std::borrow::Cow;
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;

// Note: ServiceEndpoint is not Serialize/Deserialize because std::time::Instant
// cannot be serialized. For network transmission, use a DTO pattern.

// ============================================================================
// Capability Definitions
// ============================================================================

/// Capability identifier - what a service can do.
///
/// Capabilities are primal-agnostic: they describe WHAT a service provides,
/// not WHO provides it. Any primal may implement any capability.
///
/// # Infant Discovery
///
/// Primals start with zero knowledge and discover capabilities at runtime.
/// There is no hardcoding of which primal provides which capability.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[non_exhaustive]
pub enum Capability {
    // === Identity & Cryptography ===
    /// DID resolution and verification.
    DidVerification,
    /// Cryptographic signing operations.
    Signing,
    /// Signature verification.
    SignatureVerification,
    /// Attestation and credential requests.
    Attestation,

    // === Discovery & Mesh ===
    /// Service discovery and registration.
    ServiceDiscovery,

    // === Payload Storage ===
    /// Content-addressed payload storage.
    PayloadStorage,
    /// Content-addressed payload retrieval.
    PayloadRetrieval,

    // === Permanent Storage ===
    /// Permanent/immutable storage commits.
    PermanentCommit,
    /// Slice checkout from permanent storage.
    SliceCheckout,
    /// Slice resolution back to permanent storage.
    SliceResolution,

    // === Compute ===
    /// Compute task orchestration.
    ComputeOrchestration,
    /// Compute task event streaming.
    ComputeEvents,

    // === Provenance ===
    /// Provenance chain queries.
    ProvenanceQuery,
    /// Attribution and contribution tracking.
    Attribution,

    /// Custom capability for extensibility.
    Custom(Cow<'static, str>),
}

impl Capability {
    /// Create a custom capability.
    #[inline]
    #[must_use]
    pub fn custom(name: impl Into<Cow<'static, str>>) -> Self {
        Self::Custom(name.into())
    }
}

impl std::fmt::Display for Capability {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            // Identity & Cryptography
            Self::DidVerification => write!(f, "did:verification"),
            Self::Signing => write!(f, "crypto:signing"),
            Self::SignatureVerification => write!(f, "crypto:verification"),
            Self::Attestation => write!(f, "attestation:request"),
            // Discovery & Mesh
            Self::ServiceDiscovery => write!(f, "discovery:service"),
            // Payload Storage
            Self::PayloadStorage => write!(f, "payload:storage"),
            Self::PayloadRetrieval => write!(f, "payload:retrieval"),
            // Permanent Storage
            Self::PermanentCommit => write!(f, "storage:permanent:commit"),
            Self::SliceCheckout => write!(f, "slice:checkout"),
            Self::SliceResolution => write!(f, "slice:resolution"),
            // Compute
            Self::ComputeOrchestration => write!(f, "compute:orchestration"),
            Self::ComputeEvents => write!(f, "compute:events"),
            // Provenance
            Self::ProvenanceQuery => write!(f, "provenance:query"),
            Self::Attribution => write!(f, "provenance:attribution"),
            // Custom
            Self::Custom(name) => write!(f, "custom:{name}"),
        }
    }
}

// ============================================================================
// Service Endpoint
// ============================================================================

/// A discovered service endpoint.
///
/// Service endpoints are identified by a unique `service_id` and advertise
/// their capabilities. The identity of the providing primal is not exposed -
/// only what capabilities are available at this endpoint.
#[derive(Debug, Clone)]
pub struct ServiceEndpoint {
    /// Unique service instance identifier (primal-agnostic).
    pub service_id: Cow<'static, str>,
    /// Socket address for RPC connections.
    pub addr: SocketAddr,
    /// Capabilities provided by this endpoint.
    pub capabilities: Vec<Capability>,
    /// When this endpoint was last seen healthy (not serialized).
    pub last_healthy: std::time::Instant,
    /// Health check interval.
    pub health_interval: Duration,
}

impl ServiceEndpoint {
    /// Create a new service endpoint.
    #[must_use]
    pub fn new(
        service_id: impl Into<Cow<'static, str>>,
        addr: SocketAddr,
        capabilities: Vec<Capability>,
    ) -> Self {
        Self {
            service_id: service_id.into(),
            addr,
            capabilities,
            last_healthy: std::time::Instant::now(),
            health_interval: Duration::from_secs(30),
        }
    }

    /// Check if this endpoint provides a capability.
    #[inline]
    #[must_use]
    pub fn has_capability(&self, cap: &Capability) -> bool {
        self.capabilities.contains(cap)
    }

    /// Check if the endpoint is considered healthy (last check within interval).
    #[inline]
    #[must_use]
    pub fn is_healthy(&self) -> bool {
        self.last_healthy.elapsed() < self.health_interval * 2
    }
}

// ============================================================================
// Discovery Registry
// ============================================================================

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
    /// This will:
    /// 1. Check local cache first
    /// 2. Query discovery source (Songbird) if not cached
    /// 3. Return unavailable if discovery fails
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
        if source.is_none() {
            return DiscoveryStatus::Unavailable;
        }

        // In a real implementation, this would query Songbird
        // For now, return unavailable to indicate runtime discovery is needed
        DiscoveryStatus::Unavailable
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

// ============================================================================
// Client Provider - Runtime-resolved clients
// ============================================================================

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
    /// regardless of which service implements it (could be NestGate, S3, Azure, etc.).
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

    // ============================================================================
    // Deprecated Vendor-Specific Methods (Backward Compatibility)
    // ============================================================================

    /// **DEPRECATED**: Use `has_signing()` and `has_did_verification()` instead.
    ///
    /// Check if BearDog capabilities are available.
    ///
    /// **Migration:** Replace with capability-based discovery:
    /// ```rust,ignore
    /// // OLD:
    /// if provider.has_beardog().await { ... }
    ///
    /// // NEW:
    /// if provider.has_signing().await && provider.has_did_verification().await { ... }
    /// ```
    #[deprecated(
        since = "0.15.0",
        note = "Use has_signing() and has_did_verification() instead - vendor agnostic"
    )]
    pub async fn has_beardog(&self) -> bool {
        self.has_signing().await && self.has_did_verification().await
    }

    /// **DEPRECATED**: Use `has_permanent_storage()` instead.
    ///
    /// Check if LoamSpine capabilities are available.
    ///
    /// **Migration:** Replace with capability-based discovery:
    /// ```rust,ignore
    /// // OLD:
    /// if provider.has_loamspine().await { ... }
    ///
    /// // NEW:
    /// if provider.has_permanent_storage().await { ... }
    /// ```
    #[deprecated(since = "0.15.0", note = "Use has_permanent_storage() instead - vendor agnostic")]
    pub async fn has_loamspine(&self) -> bool {
        self.has_permanent_storage().await
    }

    /// **DEPRECATED**: Use `has_payload_storage()` instead.
    ///
    /// Check if NestGate capabilities are available.
    ///
    /// **Migration:** Replace with capability-based discovery:
    /// ```rust,ignore
    /// // OLD:
    /// if provider.has_nestgate().await { ... }
    ///
    /// // NEW:
    /// if provider.has_payload_storage().await { ... }
    /// ```
    #[deprecated(since = "0.15.0", note = "Use has_payload_storage() instead - vendor agnostic")]
    pub async fn has_nestgate(&self) -> bool {
        self.has_payload_storage().await
    }

    /// **DEPRECATED**: Use `signing_endpoint()` or `did_verification_endpoint()` instead.
    ///
    /// Get the BearDog endpoint if available.
    ///
    /// # Errors
    ///
    /// Returns an error if BearDog has not been discovered.
    ///
    /// **Migration:** Replace with capability-based discovery:
    /// ```rust,ignore
    /// // OLD:
    /// let endpoint = provider.beardog_endpoint().await?;
    ///
    /// // NEW:
    /// let endpoint = provider.signing_endpoint().await?;
    /// ```
    #[deprecated(
        since = "0.15.0",
        note = "Use signing_endpoint() or did_verification_endpoint() instead - vendor agnostic"
    )]
    pub async fn beardog_endpoint(&self) -> Result<ServiceEndpoint> {
        self.signing_endpoint().await
    }

    /// **DEPRECATED**: Use `permanent_storage_endpoint()` instead.
    ///
    /// Get the LoamSpine endpoint if available.
    ///
    /// # Errors
    ///
    /// Returns an error if LoamSpine has not been discovered.
    ///
    /// **Migration:** Replace with capability-based discovery:
    /// ```rust,ignore
    /// // OLD:
    /// let endpoint = provider.loamspine_endpoint().await?;
    ///
    /// // NEW:
    /// let endpoint = provider.permanent_storage_endpoint().await?;
    /// ```
    #[deprecated(
        since = "0.15.0",
        note = "Use permanent_storage_endpoint() instead - vendor agnostic"
    )]
    pub async fn loamspine_endpoint(&self) -> Result<ServiceEndpoint> {
        self.permanent_storage_endpoint().await
    }

    /// **DEPRECATED**: Use `payload_storage_endpoint()` instead.
    ///
    /// Get the NestGate endpoint if available.
    ///
    /// # Errors
    ///
    /// Returns an error if NestGate has not been discovered.
    ///
    /// **Migration:** Replace with capability-based discovery:
    /// ```rust,ignore
    /// // OLD:
    /// let endpoint = provider.nestgate_endpoint().await?;
    ///
    /// // NEW:
    /// let endpoint = provider.payload_storage_endpoint().await?;
    /// ```
    #[deprecated(
        since = "0.15.0",
        note = "Use payload_storage_endpoint() instead - vendor agnostic"
    )]
    pub async fn nestgate_endpoint(&self) -> Result<ServiceEndpoint> {
        self.payload_storage_endpoint().await
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;

    #[test]
    fn test_capability_display() {
        assert_eq!(Capability::DidVerification.to_string(), "did:verification");
        assert_eq!(Capability::custom("myapp:feature").to_string(), "custom:myapp:feature");
    }

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

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn test_client_provider() {
        let registry = Arc::new(DiscoveryRegistry::new("rhizoCrypt"));
        let provider = ClientProvider::new(Arc::clone(&registry));

        // Nothing registered yet
        #[allow(deprecated)]
        {
            assert!(!provider.has_beardog().await);
        }

        // Register BearDog
        registry
            .register_endpoint(ServiceEndpoint::new(
                "bearDog",
                "127.0.0.1:9000".parse().unwrap(),
                vec![Capability::DidVerification, Capability::Signing],
            ))
            .await;

        #[allow(deprecated)]
        {
            assert!(provider.has_beardog().await);
            assert!(provider.beardog_endpoint().await.is_ok());
        }
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
    async fn test_client_provider_loamspine() {
        let registry = Arc::new(DiscoveryRegistry::new("rhizoCrypt"));
        let provider = ClientProvider::new(Arc::clone(&registry));

        // Nothing registered yet
        assert!(!provider.has_loamspine().await);
        assert!(provider.loamspine_endpoint().await.is_err());

        // Register LoamSpine
        registry
            .register_endpoint(ServiceEndpoint::new(
                "loamSpine",
                "127.0.0.1:9001".parse().unwrap(),
                vec![Capability::PermanentCommit, Capability::SliceCheckout],
            ))
            .await;

        assert!(provider.has_loamspine().await);
        assert!(provider.loamspine_endpoint().await.is_ok());
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn test_client_provider_nestgate() {
        let registry = Arc::new(DiscoveryRegistry::new("rhizoCrypt"));
        let provider = ClientProvider::new(Arc::clone(&registry));

        // Nothing registered yet
        assert!(!provider.has_nestgate().await);
        assert!(provider.nestgate_endpoint().await.is_err());

        // Register NestGate
        registry
            .register_endpoint(ServiceEndpoint::new(
                "nestGate",
                "127.0.0.1:9002".parse().unwrap(),
                vec![Capability::PayloadStorage, Capability::PayloadRetrieval],
            ))
            .await;

        assert!(provider.has_nestgate().await);
        assert!(provider.nestgate_endpoint().await.is_ok());
    }

    #[test]
    fn test_capability_display_all() {
        // Test all capability display strings
        assert_eq!(Capability::DidVerification.to_string(), "did:verification");
        assert_eq!(Capability::Signing.to_string(), "crypto:signing");
        assert_eq!(Capability::SignatureVerification.to_string(), "crypto:verification");
        assert_eq!(Capability::Attestation.to_string(), "attestation:request");
        assert_eq!(Capability::ServiceDiscovery.to_string(), "discovery:service");
        assert_eq!(Capability::PayloadStorage.to_string(), "payload:storage");
        assert_eq!(Capability::PayloadRetrieval.to_string(), "payload:retrieval");
        assert_eq!(Capability::PermanentCommit.to_string(), "storage:permanent:commit");
        assert_eq!(Capability::SliceCheckout.to_string(), "slice:checkout");
        assert_eq!(Capability::SliceResolution.to_string(), "slice:resolution");
        assert_eq!(Capability::ComputeOrchestration.to_string(), "compute:orchestration");
        assert_eq!(Capability::ComputeEvents.to_string(), "compute:events");
        assert_eq!(Capability::ProvenanceQuery.to_string(), "provenance:query");
        assert_eq!(Capability::Attribution.to_string(), "provenance:attribution");
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

    #[test]
    fn test_service_endpoint() {
        let endpoint = ServiceEndpoint::new(
            "testService",
            "127.0.0.1:9000".parse().unwrap(),
            vec![Capability::Signing, Capability::DidVerification],
        );

        assert_eq!(endpoint.service_id.as_ref(), "testService");
        assert!(endpoint.has_capability(&Capability::Signing));
        assert!(endpoint.has_capability(&Capability::DidVerification));
        assert!(!endpoint.has_capability(&Capability::PayloadStorage));
        assert!(endpoint.is_healthy());
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
    async fn test_client_provider_toadstool() {
        let registry = Arc::new(DiscoveryRegistry::new("rhizoCrypt"));
        let _provider = ClientProvider::new(Arc::clone(&registry));

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
    async fn test_client_provider_sweetgrass() {
        let registry = Arc::new(DiscoveryRegistry::new("rhizoCrypt"));

        // Register SweetGrass
        registry
            .register_endpoint(ServiceEndpoint::new(
                "sweetGrass",
                "127.0.0.1:9004".parse().unwrap(),
                vec![Capability::ProvenanceQuery, Capability::Attribution],
            ))
            .await;

        // SweetGrass capabilities should be discoverable
        assert!(registry.is_available(&Capability::ProvenanceQuery).await);
        assert!(registry.is_available(&Capability::Attribution).await);

        // Verify endpoint details
        let endpoint = registry.get_endpoint(&Capability::ProvenanceQuery).await;
        assert!(endpoint.is_some());
        let ep = endpoint.unwrap();
        assert_eq!(ep.service_id.as_ref(), "sweetGrass");
        assert_eq!(ep.addr.port(), 9004);
    }

    #[test]
    fn test_capability_equality() {
        assert_eq!(Capability::Signing, Capability::Signing);
        assert_ne!(Capability::Signing, Capability::Attestation);
        assert_eq!(Capability::Custom("test".into()), Capability::Custom("test".into()));
        assert_ne!(Capability::Custom("test1".into()), Capability::Custom("test2".into()));
    }

    #[test]
    fn test_capability_hash() {
        use std::collections::HashSet;
        let mut set = HashSet::new();
        set.insert(Capability::Signing);
        set.insert(Capability::Signing); // duplicate
        set.insert(Capability::Attestation);
        assert_eq!(set.len(), 2);
    }

    #[test]
    fn test_service_endpoint_health_interval() {
        let mut endpoint = ServiceEndpoint::new(
            "test",
            "127.0.0.1:9000".parse().unwrap(),
            vec![Capability::Signing],
        );

        // Default health interval
        assert_eq!(endpoint.health_interval, Duration::from_secs(30));

        // Customize health interval
        endpoint.health_interval = Duration::from_secs(60);
        assert_eq!(endpoint.health_interval, Duration::from_secs(60));
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
}
