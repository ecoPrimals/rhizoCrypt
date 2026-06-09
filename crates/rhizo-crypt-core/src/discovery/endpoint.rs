// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2024–2026 ecoPrimals Project

//! Service endpoint types for discovery.
//!
//! Service endpoints are identified by a unique `service_id` and advertise
//! their capabilities. The identity of the providing primal is not exposed -
//! only what capabilities are available at this endpoint.

use super::Capability;
use crate::transport::TransportEndpoint;
use std::borrow::Cow;
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
    /// Transport endpoint (TCP or UDS) for RPC connections.
    pub endpoint: TransportEndpoint,
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
        endpoint: TransportEndpoint,
        capabilities: Vec<Capability>,
    ) -> Self {
        Self {
            service_id: service_id.into(),
            endpoint,
            capabilities,
            last_healthy: std::time::Instant::now(),
            health_interval: crate::constants::DEFAULT_HEALTH_CHECK_INTERVAL,
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
#[expect(clippy::unwrap_used, reason = "test code")]
mod tests {
    use super::*;

    fn tcp_ep(addr: &str) -> TransportEndpoint {
        let (host, port) = addr.rsplit_once(':').unwrap();
        TransportEndpoint::tcp(host, port.parse().unwrap())
    }

    #[test]
    fn test_service_endpoint() {
        let ep = ServiceEndpoint::new(
            "testService",
            tcp_ep("127.0.0.1:9000"),
            vec![Capability::Signing, Capability::DidVerification],
        );

        assert_eq!(ep.service_id.as_ref(), "testService");
        assert!(ep.has_capability(&Capability::Signing));
        assert!(ep.has_capability(&Capability::DidVerification));
        assert!(!ep.has_capability(&Capability::PayloadStorage));
        assert!(ep.is_healthy());
    }

    #[test]
    fn test_service_endpoint_health_interval() {
        let mut ep = ServiceEndpoint::new(
            "test",
            tcp_ep("127.0.0.1:9000"),
            vec![Capability::Signing],
        );

        assert_eq!(ep.health_interval, crate::constants::DEFAULT_HEALTH_CHECK_INTERVAL);

        ep.health_interval = Duration::from_secs(60);
        assert_eq!(ep.health_interval, Duration::from_secs(60));
    }

    #[test]
    fn test_endpoint_no_capabilities() {
        let ep = ServiceEndpoint::new("empty", tcp_ep("127.0.0.1:9001"), vec![]);
        assert!(!ep.has_capability(&Capability::Signing));
        assert!(ep.capabilities.is_empty());
    }

    #[test]
    fn test_endpoint_clone() {
        let ep = ServiceEndpoint::new(
            "cloneable",
            tcp_ep("127.0.0.1:9002"),
            vec![Capability::PermanentCommit],
        );
        let cloned = ep.clone();
        assert_eq!(cloned.service_id, ep.service_id);
        assert_eq!(cloned.endpoint, ep.endpoint);
        assert_eq!(cloned.capabilities, ep.capabilities);
    }

    #[test]
    fn test_endpoint_static_service_id() {
        let ep = ServiceEndpoint::new(
            "static-id",
            tcp_ep("127.0.0.1:9003"),
            vec![Capability::Signing],
        );
        assert_eq!(ep.service_id.as_ref(), "static-id");
    }

    #[test]
    fn test_endpoint_owned_service_id() {
        let id = String::from("dynamic-id");
        let ep = ServiceEndpoint::new(id, tcp_ep("127.0.0.1:9004"), vec![Capability::Signing]);
        assert_eq!(ep.service_id.as_ref(), "dynamic-id");
    }

    #[test]
    fn test_uds_endpoint() {
        let ep = ServiceEndpoint::new(
            "uds-service",
            TransportEndpoint::Uds { path: "/run/biomeos/test.sock".into() },
            vec![Capability::Signing],
        );
        assert!(matches!(ep.endpoint, TransportEndpoint::Uds { .. }));
        assert!(ep.is_healthy());
    }
}
