// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2024–2026 ecoPrimals Project

//! Integration traits for external services.
//!
//! This module provides integration interfaces for connecting with external
//! services in the ecosystem.
//!
//! ## Architecture Evolution
//!
//! ### ✅ NEW: Capability-Based Integration (Recommended)
//!
//! Use generic capability clients that work with ANY provider:
//!
//! ```no_run
//! # use rhizo_crypt_core::clients::capabilities::SigningClient;
//! # use rhizo_crypt_core::types::Did;
//! # use std::sync::Arc;
//! # tokio::runtime::Runtime::new().unwrap().block_on(async {
//! # let registry = Arc::new(rhizo_crypt_core::discovery::DiscoveryRegistry::new("doc-test"));
//! # registry.register_endpoint(rhizo_crypt_core::discovery::ServiceEndpoint::new(
//! #     "test-signer", "127.0.0.1:9500".parse().unwrap(),
//! #     vec![rhizo_crypt_core::discovery::Capability::Signing],
//! # )).await;
//! // Discover ANY signing provider
//! let signer = SigningClient::discover(&registry).await?;
//! let did = Did::new("did:key:test");
//! let signature = signer.sign(b"data", &did).await?;
//! # Ok::<(), rhizo_crypt_core::error::RhizoCryptError>(())
//! # });
//! ```
//!
//! ### ⚠️ LEGACY: Primal-Specific Traits (Deprecated)
//!
//! These traits are vendor-specific and create lock-in. Use capability clients instead.
//!
//! ## Design Philosophy
//!
//! - **Capabilities over vendors** — Request what you need, not who provides it
//! - **Discovery at runtime** — Services found dynamically, not hardcoded
//! - **Mocks for testing** — Generic mocks that work with all providers

mod traits;
pub use traits::{PayloadStorageProvider, PermanentStorageProvider, SigningProvider};

use crate::discovery::{Capability, DiscoveryRegistry};
use crate::error::Result;

use std::borrow::Cow;
use std::net::SocketAddr;
use std::sync::Arc;

// Test-only mock implementations
#[cfg(any(test, feature = "test-utils"))]
pub mod mocks;

#[cfg(any(test, feature = "test-utils"))]
pub use mocks::{
    MockCapabilityFactory, MockPayloadStorageProvider, MockPermanentStorageProvider,
    MockProtocolAdapter, MockSigningProvider,
};

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
/// ```no_run
/// # use rhizo_crypt_core::integration::ClientFactory;
/// # use rhizo_crypt_core::discovery::{DiscoveryRegistry, ServiceEndpoint, Capability};
/// # use std::sync::Arc;
/// # tokio::runtime::Runtime::new().unwrap().block_on(async {
/// # let registry = Arc::new(DiscoveryRegistry::new("rhizoCrypt"));
/// # registry.register_endpoint(ServiceEndpoint::new(
/// #     "test-signer", "127.0.0.1:9500".parse().unwrap(),
/// #     vec![Capability::Signing],
/// # )).await;
/// let factory = ClientFactory::new(registry);
///
/// // Check if a service is available
/// if factory.has_signing_capability().await {
///     let _endpoint = factory.signing_endpoint().await?;
///     // Connect to endpoint via tarpc
/// }
/// # Ok::<(), rhizo_crypt_core::error::RhizoCryptError>(())
/// # });
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

    /// Check if signing capability is available.
    pub async fn has_signing_capability(&self) -> bool {
        self.registry.is_available(&Capability::Signing).await
    }

    /// Check if permanent commit capability is available.
    pub async fn has_commit_capability(&self) -> bool {
        self.registry.is_available(&Capability::PermanentCommit).await
    }

    /// Check if payload storage capability is available.
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
#[expect(clippy::unwrap_used, reason = "test code")]
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

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
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

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
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

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
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
