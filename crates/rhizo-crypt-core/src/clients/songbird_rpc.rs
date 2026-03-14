// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2024–2026 ecoPrimals Project

//! Songbird tarpc RPC types and client.
//!
//! This module contains the tarpc service definition and related types
//! for connecting to the Songbird orchestrator.
//!
//! ## Feature Gate
//!
//! This module is only compiled when the `live-clients` feature is enabled.
//! Without the feature, the Songbird client operates in scaffolded mode.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ============================================================================
// tarpc Service Definition
// ============================================================================

/// tarpc service trait for Songbird discovery operations.
///
/// This trait mirrors the Songbird orchestrator's RPC interface.
#[tarpc::service]
pub trait SongbirdRpc {
    /// Discover services by capability.
    async fn discover(capability: String) -> Vec<RpcServiceInfo>;

    /// Discover all available services.
    async fn discover_all() -> Vec<RpcServiceInfo>;

    /// Register a service with the mesh.
    async fn register(registration: RpcServiceRegistration) -> RpcRegistrationResult;

    /// Unregister a service from the mesh.
    async fn unregister(service_id: String) -> RpcRegistrationResult;

    /// Get health status of the orchestrator.
    async fn health() -> RpcHealthStatus;

    /// Get version information.
    async fn version() -> RpcVersionInfo;
}

// ============================================================================
// RPC Types
// ============================================================================

/// Service info from tarpc discovery.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RpcServiceInfo {
    /// Service identifier.
    pub id: String,
    /// Service capability.
    pub capability: String,
    /// Service endpoint address.
    pub endpoint: String,
    /// Service status.
    pub status: String,
    /// Optional metadata.
    pub metadata: Option<serde_json::Value>,
}

/// Service registration for tarpc.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RpcServiceRegistration {
    /// Service ID.
    pub service_id: String,
    /// Service name.
    pub service_name: String,
    /// Primary capability.
    pub capability: String,
    /// Endpoint address.
    pub endpoint: String,
    /// Additional metadata.
    #[serde(default)]
    pub metadata: HashMap<String, String>,
}

/// Registration result from tarpc.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RpcRegistrationResult {
    /// Whether registration succeeded.
    pub success: bool,
    /// Result message.
    pub message: String,
}

/// Health status from tarpc.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RpcHealthStatus {
    /// Overall status.
    pub status: String,
    /// Orchestrator version.
    pub version: String,
    /// Uptime in seconds.
    pub uptime_seconds: u64,
    /// Number of registered services.
    pub services_count: usize,
}

/// Version info from tarpc.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RpcVersionInfo {
    /// Version string.
    pub version: String,
    /// Protocol version.
    pub protocol: String,
    /// Supported capabilities.
    pub capabilities: Vec<String>,
}

// ============================================================================
// Mock server for integration tests (live-clients only)
// ============================================================================

/// Mock Songbird RPC server for integration tests.
///
/// Implements `SongbirdRpc` with canned responses for register, discover, etc.
#[cfg(all(test, feature = "live-clients"))]
#[derive(Clone)]
pub struct MockSongbirdServer;

#[cfg(all(test, feature = "live-clients"))]
impl SongbirdRpc for MockSongbirdServer {
    async fn discover(self, _: tarpc::context::Context, capability: String) -> Vec<RpcServiceInfo> {
        if capability == "signing" {
            vec![RpcServiceInfo {
                id: "mock-beardog-1".to_string(),
                capability: "signing".to_string(),
                endpoint: "127.0.0.1:9500".to_string(),
                status: "healthy".to_string(),
                metadata: None,
            }]
        } else {
            vec![]
        }
    }

    async fn discover_all(self, _: tarpc::context::Context) -> Vec<RpcServiceInfo> {
        vec![]
    }

    async fn register(
        self,
        _: tarpc::context::Context,
        registration: RpcServiceRegistration,
    ) -> RpcRegistrationResult {
        RpcRegistrationResult {
            success: true,
            message: format!("Registered {}", registration.service_id),
        }
    }

    async fn unregister(
        self,
        _: tarpc::context::Context,
        _service_id: String,
    ) -> RpcRegistrationResult {
        RpcRegistrationResult {
            success: true,
            message: "Unregistered".to_string(),
        }
    }

    async fn health(self, _: tarpc::context::Context) -> RpcHealthStatus {
        RpcHealthStatus {
            status: "healthy".to_string(),
            version: "0.1.0-test".to_string(),
            uptime_seconds: 0,
            services_count: 1,
        }
    }

    async fn version(self, _: tarpc::context::Context) -> RpcVersionInfo {
        RpcVersionInfo {
            version: "0.1.0-test".to_string(),
            protocol: "tarpc-1.0".to_string(),
            capabilities: vec!["discovery".to_string()],
        }
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn test_service_info_roundtrip() {
        let info = RpcServiceInfo {
            id: "svc-001".to_string(),
            capability: "dag-engine".to_string(),
            endpoint: "127.0.0.1:9400".to_string(),
            status: "healthy".to_string(),
            metadata: Some(serde_json::json!({"version": "0.13.0"})),
        };

        let json = serde_json::to_string(&info).unwrap();
        let decoded: RpcServiceInfo = serde_json::from_str(&json).unwrap();
        assert_eq!(decoded.id, "svc-001");
        assert_eq!(decoded.capability, "dag-engine");
        assert!(decoded.metadata.is_some());
    }

    #[test]
    fn test_registration_roundtrip() {
        let reg = RpcServiceRegistration {
            service_id: "rhizocrypt-001".to_string(),
            service_name: "rhizoCrypt".to_string(),
            capability: "dag-engine".to_string(),
            endpoint: "127.0.0.1:9400".to_string(),
            metadata: HashMap::from([("version".to_string(), "0.13.0".to_string())]),
        };

        let json = serde_json::to_string(&reg).unwrap();
        let decoded: RpcServiceRegistration = serde_json::from_str(&json).unwrap();
        assert_eq!(decoded.service_name, "rhizoCrypt");
        assert_eq!(decoded.metadata.get("version").unwrap(), "0.13.0");
    }

    #[test]
    fn test_registration_default_metadata() {
        let json = r#"{"service_id":"s1","service_name":"svc","capability":"cap","endpoint":"ep"}"#;
        let decoded: RpcServiceRegistration = serde_json::from_str(json).unwrap();
        assert!(decoded.metadata.is_empty());
    }

    #[test]
    fn test_registration_result_roundtrip() {
        let result = RpcRegistrationResult {
            success: true,
            message: "Registered".to_string(),
        };

        let json = serde_json::to_string(&result).unwrap();
        let decoded: RpcRegistrationResult = serde_json::from_str(&json).unwrap();
        assert!(decoded.success);
        assert_eq!(decoded.message, "Registered");
    }

    #[test]
    fn test_health_status_roundtrip() {
        let health = RpcHealthStatus {
            status: "healthy".to_string(),
            version: "0.5.0".to_string(),
            uptime_seconds: 3600,
            services_count: 12,
        };

        let json = serde_json::to_string(&health).unwrap();
        let decoded: RpcHealthStatus = serde_json::from_str(&json).unwrap();
        assert_eq!(decoded.uptime_seconds, 3600);
        assert_eq!(decoded.services_count, 12);
    }

    #[test]
    fn test_version_info_roundtrip() {
        let version = RpcVersionInfo {
            version: "0.5.0".to_string(),
            protocol: "tarpc-1.0".to_string(),
            capabilities: vec!["discovery".to_string(), "mesh".to_string()],
        };

        let json = serde_json::to_string(&version).unwrap();
        let decoded: RpcVersionInfo = serde_json::from_str(&json).unwrap();
        assert_eq!(decoded.protocol, "tarpc-1.0");
        assert_eq!(decoded.capabilities.len(), 2);
    }

    // ------------------------------------------------------------------------
    // MockSongbirdServer direct-call tests (live-clients only)
    // ------------------------------------------------------------------------

    #[cfg(all(test, feature = "live-clients"))]
    #[tokio::test]
    async fn mock_discover_signing_returns_service() {
        use super::{MockSongbirdServer, SongbirdRpc};

        let server = MockSongbirdServer;
        let services = server.discover(tarpc::context::current(), "signing".to_string()).await;
        assert_eq!(services.len(), 1);
        assert_eq!(services[0].id, "mock-beardog-1");
        assert_eq!(services[0].capability, "signing");
        assert_eq!(services[0].endpoint, "127.0.0.1:9500");
        assert_eq!(services[0].status, "healthy");
    }

    #[cfg(all(test, feature = "live-clients"))]
    #[tokio::test]
    async fn mock_discover_unknown_capability_returns_empty() {
        use super::{MockSongbirdServer, SongbirdRpc};

        let server = MockSongbirdServer;
        let services = server.discover(tarpc::context::current(), "unknown".to_string()).await;
        assert!(services.is_empty());
    }

    #[cfg(all(test, feature = "live-clients"))]
    #[tokio::test]
    async fn mock_discover_all_returns_empty() {
        use super::{MockSongbirdServer, SongbirdRpc};

        let server = MockSongbirdServer;
        let services = server.discover_all(tarpc::context::current()).await;
        assert!(services.is_empty());
    }

    #[cfg(all(test, feature = "live-clients"))]
    #[tokio::test]
    async fn mock_register_success() {
        use super::{MockSongbirdServer, RpcServiceRegistration, SongbirdRpc};

        let server = MockSongbirdServer;
        let reg = RpcServiceRegistration {
            service_id: "test-svc-1".to_string(),
            service_name: "TestService".to_string(),
            capability: "dag-engine".to_string(),
            endpoint: "127.0.0.1:9400".to_string(),
            metadata: HashMap::new(),
        };
        let result = server.register(tarpc::context::current(), reg).await;
        assert!(result.success);
        assert_eq!(result.message, "Registered test-svc-1");
    }

    #[cfg(all(test, feature = "live-clients"))]
    #[tokio::test]
    async fn mock_unregister_success() {
        use super::{MockSongbirdServer, SongbirdRpc};

        let server = MockSongbirdServer;
        let result = server.unregister(tarpc::context::current(), "test-svc-1".to_string()).await;
        assert!(result.success);
        assert_eq!(result.message, "Unregistered");
    }

    #[cfg(all(test, feature = "live-clients"))]
    #[tokio::test]
    async fn mock_health_returns_status() {
        use super::{MockSongbirdServer, SongbirdRpc};

        let server = MockSongbirdServer;
        let health = server.health(tarpc::context::current()).await;
        assert_eq!(health.status, "healthy");
        assert_eq!(health.version, "0.1.0-test");
        assert_eq!(health.services_count, 1);
    }

    #[cfg(all(test, feature = "live-clients"))]
    #[tokio::test]
    async fn mock_version_returns_info() {
        use super::{MockSongbirdServer, SongbirdRpc};

        let server = MockSongbirdServer;
        let version = server.version(tarpc::context::current()).await;
        assert_eq!(version.version, "0.1.0-test");
        assert_eq!(version.protocol, "tarpc-1.0");
        assert_eq!(version.capabilities, vec!["discovery"]);
    }
}
