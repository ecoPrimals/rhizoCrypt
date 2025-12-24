//! Integration traits for external primals.
//!
//! This module defines the client interfaces for integrating with
//! BearDog (identity/signing), LoamSpine (permanent storage), and NestGate (payloads).
//!
//! ## Design Philosophy
//!
//! - **Traits define capabilities** — What operations are available
//! - **Discovery provides endpoints** — Where to find implementations
//! - **Mocks are test-only** — Production uses runtime-discovered clients
//!
//! ## Usage
//!
//! ```rust,ignore
//! // In production: Use discovery to find BearDog
//! let provider = ClientFactory::new(registry);
//! if provider.has_beardog().await {
//!     let endpoint = provider.beardog_endpoint().await?;
//!     // Connect to endpoint.addr via tarpc
//! }
//!
//! // In tests: Use mock implementations
//! #[cfg(test)]
//! let client = mocks::MockBearDogClient::permissive();
//! ```

use crate::dehydration::{Attestation, DehydrationSummary};
use crate::discovery::{Capability, DiscoveryRegistry};
use crate::error::Result;
use crate::session::LoamCommitRef;
use crate::slice::{ResolutionOutcome, Slice, SliceOrigin};
use crate::types::{Did, PayloadRef, Signature};
use crate::vertex::Vertex;

use std::borrow::Cow;
use std::net::SocketAddr;
use std::sync::Arc;

// Test-only mock implementations
#[cfg(any(test, feature = "test-utils"))]
pub mod mocks;

// Re-export mocks for test convenience
#[cfg(any(test, feature = "test-utils"))]
pub use mocks::{MockBearDogClient, MockLoamSpineClient, MockNestGateClient};

// ============================================================================
// BearDog Integration (Identity & Signing)
// ============================================================================

/// BearDog client interface for identity and signing.
///
/// BearDog provides:
/// - DID verification and resolution
/// - Cryptographic signing operations
/// - Signature verification
/// - Attestation requests
pub trait BearDogClient: Send + Sync {
    /// Resolve a DID to verify it exists and is active.
    fn verify_did(&self, did: &Did) -> impl std::future::Future<Output = Result<bool>> + Send;

    /// Sign data with a specific DID.
    fn sign(
        &self,
        data: &[u8],
        signer: &Did,
    ) -> impl std::future::Future<Output = Result<Signature>> + Send;

    /// Sign a vertex.
    fn sign_vertex(
        &self,
        vertex: &Vertex,
        signer: &Did,
    ) -> impl std::future::Future<Output = Result<Signature>> + Send;

    /// Verify a signature.
    fn verify_signature(
        &self,
        data: &[u8],
        signature: &Signature,
        signer: &Did,
    ) -> impl std::future::Future<Output = Result<bool>> + Send;

    /// Verify a vertex signature.
    fn verify_vertex_signature(
        &self,
        vertex: &Vertex,
    ) -> impl std::future::Future<Output = Result<bool>> + Send;

    /// Request an attestation from a party.
    fn request_attestation(
        &self,
        attester: &Did,
        summary: &DehydrationSummary,
    ) -> impl std::future::Future<Output = Result<Attestation>> + Send;
}

// ============================================================================
// LoamSpine Integration (Permanent Storage)
// ============================================================================

/// LoamSpine client interface for permanent storage.
///
/// LoamSpine provides:
/// - Permanent commit of dehydration summaries
/// - Commit verification
/// - Slice checkout and resolution
pub trait LoamSpineClient: Send + Sync {
    /// Commit a dehydration summary to the spine.
    fn commit(
        &self,
        summary: &DehydrationSummary,
    ) -> impl std::future::Future<Output = Result<LoamCommitRef>> + Send;

    /// Verify a commit exists.
    fn verify_commit(
        &self,
        commit_ref: &LoamCommitRef,
    ) -> impl std::future::Future<Output = Result<bool>> + Send;

    /// Get a commit by reference.
    fn get_commit(
        &self,
        commit_ref: &LoamCommitRef,
    ) -> impl std::future::Future<Output = Result<Option<DehydrationSummary>>> + Send;

    /// Check out a slice from the spine.
    fn checkout_slice(
        &self,
        spine_id: &str,
        entry_hash: &[u8; 32],
        holder: &Did,
    ) -> impl std::future::Future<Output = Result<SliceOrigin>> + Send;

    /// Resolve a slice back to the spine.
    fn resolve_slice(
        &self,
        slice: &Slice,
        outcome: &ResolutionOutcome,
    ) -> impl std::future::Future<Output = Result<()>> + Send;
}

// ============================================================================
// NestGate Integration (Payload Storage)
// ============================================================================

/// NestGate client interface for payload storage.
///
/// NestGate provides:
/// - Content-addressed payload storage
/// - Payload retrieval
/// - Existence checks
pub trait NestGateClient: Send + Sync {
    /// Store a payload.
    fn put_payload(
        &self,
        data: bytes::Bytes,
    ) -> impl std::future::Future<Output = Result<PayloadRef>> + Send;

    /// Get a payload.
    fn get_payload(
        &self,
        payload_ref: &PayloadRef,
    ) -> impl std::future::Future<Output = Result<Option<bytes::Bytes>>> + Send;

    /// Check if payload exists.
    fn payload_exists(
        &self,
        payload_ref: &PayloadRef,
    ) -> impl std::future::Future<Output = Result<bool>> + Send;
}

// ============================================================================
// Integration Status - Runtime capability checking
// ============================================================================

/// Status of integration with discovered capabilities.
///
/// Uses capability-based naming rather than primal names. Each primal
/// knows only itself and discovers capabilities, not specific primals.
#[derive(Debug, Clone, Default)]
pub struct IntegrationStatus {
    /// Signing capability status (identity & cryptography).
    pub signing: ServiceStatus,
    /// Permanent storage capability status.
    pub permanent_storage: ServiceStatus,
    /// Payload storage capability status.
    pub payload_storage: ServiceStatus,
}

/// Status of a single service integration.
#[derive(Debug, Clone, Default)]
pub enum ServiceStatus {
    /// Service not discovered yet.
    #[default]
    NotDiscovered,
    /// Service discovered and healthy.
    Healthy {
        /// Service endpoint address.
        endpoint: Cow<'static, str>,
        /// Last successful health check.
        last_healthy: std::time::Instant,
    },
    /// Service discovered but unhealthy.
    Unhealthy {
        /// Service endpoint address.
        endpoint: Cow<'static, str>,
        /// Reason for unhealthy status.
        reason: Cow<'static, str>,
    },
    /// Service explicitly unavailable (graceful degradation).
    Unavailable {
        /// Reason for unavailability.
        reason: Cow<'static, str>,
    },
}

impl ServiceStatus {
    /// Check if service is healthy.
    #[inline]
    #[must_use]
    pub const fn is_healthy(&self) -> bool {
        matches!(self, Self::Healthy { .. })
    }

    /// Check if service is available (healthy or not yet discovered).
    #[inline]
    #[must_use]
    pub const fn is_available(&self) -> bool {
        !matches!(self, Self::Unavailable { .. })
    }

    /// Create a healthy status.
    #[must_use]
    pub fn healthy(endpoint: impl Into<Cow<'static, str>>) -> Self {
        Self::Healthy {
            endpoint: endpoint.into(),
            last_healthy: std::time::Instant::now(),
        }
    }

    /// Create an unhealthy status.
    #[must_use]
    pub fn unhealthy(
        endpoint: impl Into<Cow<'static, str>>,
        reason: impl Into<Cow<'static, str>>,
    ) -> Self {
        Self::Unhealthy {
            endpoint: endpoint.into(),
            reason: reason.into(),
        }
    }

    /// Create an unavailable status.
    #[must_use]
    pub fn unavailable(reason: impl Into<Cow<'static, str>>) -> Self {
        Self::Unavailable {
            reason: reason.into(),
        }
    }
}

impl IntegrationStatus {
    /// Create a new integration status with all services not discovered.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Check if all services are healthy.
    #[must_use]
    pub const fn all_healthy(&self) -> bool {
        self.signing.is_healthy()
            && self.permanent_storage.is_healthy()
            && self.payload_storage.is_healthy()
    }

    /// Check if any service is unavailable.
    #[must_use]
    pub const fn any_unavailable(&self) -> bool {
        !self.signing.is_available()
            || !self.permanent_storage.is_available()
            || !self.payload_storage.is_available()
    }

    /// Get a human-readable summary.
    #[must_use]
    pub fn summary(&self) -> String {
        format!(
            "signing: {}, permanent_storage: {}, payload_storage: {}",
            status_str(&self.signing),
            status_str(&self.permanent_storage),
            status_str(&self.payload_storage),
        )
    }
}

const fn status_str(status: &ServiceStatus) -> &'static str {
    match status {
        ServiceStatus::NotDiscovered => "not discovered",
        ServiceStatus::Healthy {
            ..
        } => "healthy",
        ServiceStatus::Unhealthy {
            ..
        } => "unhealthy",
        ServiceStatus::Unavailable {
            ..
        } => "unavailable",
    }
}

// ============================================================================
// Client Factory (Production Use)
// ============================================================================

/// Factory for creating connected clients via discovery.
///
/// This factory provides a production-ready way to create clients for
/// sibling primals, using the discovery registry to find endpoints.
///
/// ## Usage
///
/// ```rust,ignore
/// let registry = Arc::new(DiscoveryRegistry::new("rhizoCrypt"));
/// let factory = ClientFactory::new(registry);
///
/// // Check if a service is available
/// if factory.has_signing_capability().await {
///     let endpoint = factory.signing_endpoint().await?;
///     // Connect to endpoint via tarpc
/// }
/// ```
#[derive(Clone)]
pub struct ClientFactory {
    registry: Arc<DiscoveryRegistry>,
}

impl ClientFactory {
    /// Create a new client factory with the given discovery registry.
    #[must_use]
    pub const fn new(registry: Arc<DiscoveryRegistry>) -> Self {
        Self {
            registry,
        }
    }

    /// Check if signing capability (BearDog) is available.
    pub async fn has_signing_capability(&self) -> bool {
        self.registry.is_available(&Capability::Signing).await
    }

    /// Check if permanent commit capability (LoamSpine) is available.
    pub async fn has_commit_capability(&self) -> bool {
        self.registry.is_available(&Capability::PermanentCommit).await
    }

    /// Check if payload storage capability (NestGate) is available.
    pub async fn has_storage_capability(&self) -> bool {
        self.registry.is_available(&Capability::PayloadStorage).await
    }

    /// Get the endpoint for signing capability.
    ///
    /// # Errors
    ///
    /// Returns error if no signing service is registered.
    pub async fn signing_endpoint(&self) -> Result<SocketAddr> {
        self.registry
            .get_endpoint(&Capability::Signing)
            .await
            .ok_or_else(|| crate::error::RhizoCryptError::integration("No signing service found"))
            .map(|e| e.addr)
    }

    /// Get the endpoint for permanent commit capability.
    ///
    /// # Errors
    ///
    /// Returns error if no commit service is registered.
    pub async fn commit_endpoint(&self) -> Result<SocketAddr> {
        self.registry
            .get_endpoint(&Capability::PermanentCommit)
            .await
            .ok_or_else(|| {
                crate::error::RhizoCryptError::integration("No permanent commit service found")
            })
            .map(|e| e.addr)
    }

    /// Get the endpoint for payload storage capability.
    ///
    /// # Errors
    ///
    /// Returns error if no storage service is registered.
    pub async fn storage_endpoint(&self) -> Result<SocketAddr> {
        self.registry
            .get_endpoint(&Capability::PayloadStorage)
            .await
            .ok_or_else(|| {
                crate::error::RhizoCryptError::integration("No payload storage service found")
            })
            .map(|e| e.addr)
    }

    /// Get the discovery registry for direct access.
    #[must_use]
    pub fn registry(&self) -> &DiscoveryRegistry {
        &self.registry
    }

    /// Get current integration status based on discovered capabilities.
    pub async fn integration_status(&self) -> IntegrationStatus {
        let mut status = IntegrationStatus::new();

        if let Some(endpoint) = self.registry.get_endpoint(&Capability::Signing).await {
            status.signing = ServiceStatus::healthy(endpoint.addr.to_string());
        }

        if let Some(endpoint) = self.registry.get_endpoint(&Capability::PermanentCommit).await {
            status.permanent_storage = ServiceStatus::healthy(endpoint.addr.to_string());
        }

        if let Some(endpoint) = self.registry.get_endpoint(&Capability::PayloadStorage).await {
            status.payload_storage = ServiceStatus::healthy(endpoint.addr.to_string());
        }

        status
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;
    use crate::discovery::ServiceEndpoint;

    #[test]
    fn test_service_status() {
        let status = ServiceStatus::healthy("127.0.0.1:9000");
        assert!(status.is_healthy());
        assert!(status.is_available());

        let status = ServiceStatus::unavailable("no provider");
        assert!(!status.is_healthy());
        assert!(!status.is_available());
    }

    #[test]
    fn test_integration_status() {
        let mut status = IntegrationStatus::new();
        assert!(!status.all_healthy());
        assert!(!status.any_unavailable());

        status.signing = ServiceStatus::healthy("127.0.0.1:9000");
        status.permanent_storage = ServiceStatus::healthy("127.0.0.1:9001");
        status.payload_storage = ServiceStatus::healthy("127.0.0.1:9002");
        assert!(status.all_healthy());
    }

    #[test]
    fn test_integration_status_summary() {
        let mut status = IntegrationStatus::new();
        let summary = status.summary();
        assert!(summary.contains("not discovered"));

        status.signing = ServiceStatus::healthy("127.0.0.1:9000");
        status.permanent_storage = ServiceStatus::unhealthy("127.0.0.1:9001", "connection refused");
        status.payload_storage = ServiceStatus::unavailable("no provider");

        let summary = status.summary();
        assert!(summary.contains("healthy"));
        assert!(summary.contains("unhealthy"));
        assert!(summary.contains("unavailable"));
    }

    #[test]
    fn test_service_status_unhealthy() {
        let status = ServiceStatus::unhealthy("127.0.0.1:9000", "timeout");
        assert!(!status.is_healthy());
        assert!(status.is_available()); // unhealthy != unavailable

        match status {
            ServiceStatus::Unhealthy {
                endpoint,
                reason,
            } => {
                assert_eq!(endpoint.as_ref(), "127.0.0.1:9000");
                assert_eq!(reason.as_ref(), "timeout");
            }
            _ => panic!("Expected Unhealthy status"),
        }
    }

    #[test]
    fn test_integration_status_any_unavailable() {
        let mut status = IntegrationStatus::new();
        assert!(!status.any_unavailable()); // NotDiscovered is available

        status.signing = ServiceStatus::unavailable("no provider");
        assert!(status.any_unavailable());

        // Even with other healthy services, any_unavailable is true
        status.permanent_storage = ServiceStatus::healthy("127.0.0.1:9001");
        status.payload_storage = ServiceStatus::healthy("127.0.0.1:9002");
        assert!(status.any_unavailable());
    }

    #[tokio::test]
    async fn test_client_factory() {
        let registry = Arc::new(DiscoveryRegistry::new("rhizoCrypt"));
        let factory = ClientFactory::new(Arc::clone(&registry));

        // Initially nothing is available
        assert!(!factory.has_signing_capability().await);
        assert!(!factory.has_commit_capability().await);
        assert!(!factory.has_storage_capability().await);

        // Errors when trying to get endpoints
        assert!(factory.signing_endpoint().await.is_err());
        assert!(factory.commit_endpoint().await.is_err());
        assert!(factory.storage_endpoint().await.is_err());
    }

    #[tokio::test]
    async fn test_client_factory_with_services() {
        let registry = Arc::new(DiscoveryRegistry::new("rhizoCrypt"));
        let factory = ClientFactory::new(Arc::clone(&registry));

        // Register services
        registry
            .register_endpoint(ServiceEndpoint::new(
                "bearDog",
                "127.0.0.1:9000".parse().unwrap(),
                vec![Capability::Signing, Capability::DidVerification],
            ))
            .await;

        registry
            .register_endpoint(ServiceEndpoint::new(
                "loamSpine",
                "127.0.0.1:9001".parse().unwrap(),
                vec![Capability::PermanentCommit],
            ))
            .await;

        registry
            .register_endpoint(ServiceEndpoint::new(
                "nestGate",
                "127.0.0.1:9002".parse().unwrap(),
                vec![Capability::PayloadStorage],
            ))
            .await;

        // Now everything should be available
        assert!(factory.has_signing_capability().await);
        assert!(factory.has_commit_capability().await);
        assert!(factory.has_storage_capability().await);

        // Endpoints should be retrievable
        let signing_addr = factory.signing_endpoint().await.unwrap();
        assert_eq!(signing_addr.port(), 9000);

        let commit_addr = factory.commit_endpoint().await.unwrap();
        assert_eq!(commit_addr.port(), 9001);

        let storage_addr = factory.storage_endpoint().await.unwrap();
        assert_eq!(storage_addr.port(), 9002);
    }

    #[tokio::test]
    async fn test_client_factory_integration_status() {
        let registry = Arc::new(DiscoveryRegistry::new("rhizoCrypt"));
        let factory = ClientFactory::new(Arc::clone(&registry));

        // Initially all not discovered
        let status = factory.integration_status().await;
        assert!(!status.all_healthy());
        assert!(!status.any_unavailable()); // NotDiscovered != Unavailable

        // Register services
        registry
            .register_endpoint(ServiceEndpoint::new(
                "bearDog",
                "127.0.0.1:9000".parse().unwrap(),
                vec![Capability::Signing],
            ))
            .await;

        registry
            .register_endpoint(ServiceEndpoint::new(
                "loamSpine",
                "127.0.0.1:9001".parse().unwrap(),
                vec![Capability::PermanentCommit],
            ))
            .await;

        registry
            .register_endpoint(ServiceEndpoint::new(
                "nestGate",
                "127.0.0.1:9002".parse().unwrap(),
                vec![Capability::PayloadStorage],
            ))
            .await;

        let status = factory.integration_status().await;
        assert!(status.all_healthy());
    }

    #[test]
    fn test_client_factory_registry_access() {
        let registry = Arc::new(DiscoveryRegistry::new("rhizoCrypt"));
        let factory = ClientFactory::new(Arc::clone(&registry));

        assert_eq!(factory.registry().local_name(), "rhizoCrypt");
    }

    #[test]
    fn test_service_status_default() {
        let status = ServiceStatus::default();
        matches!(status, ServiceStatus::NotDiscovered);
    }
}
