// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2024–2026 ecoPrimals Project

//! Service endpoint types for discovery.
//!
//! Service endpoints are identified by a unique `service_id` and advertise
//! their capabilities. The identity of the providing primal is not exposed -
//! only what capabilities are available at this endpoint.

use super::Capability;
use std::borrow::Cow;
use std::net::SocketAddr;
use std::time::Duration;

// Note: ServiceEndpoint is not Serialize/Deserialize because std::time::Instant
// cannot be serialized. For network transmission, use a DTO pattern.

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

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;

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
}
