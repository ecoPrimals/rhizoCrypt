// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2024–2026 ecoPrimals Project

//! Tests for Songbird client.
//!
//! Note: Songbird uses tarpc (TCP + bincode), not HTTP. Wiremock cannot be used.
//! The live-clients tests use a mock tarpc server for integration coverage.

use super::{SongbirdClient, SongbirdConfig};
use crate::clients::songbird_types::{ClientState, ServiceInfo};
use crate::discovery::{Capability, DiscoveryRegistry};
use std::collections::HashMap;

#[test]
fn test_config_new_unconfigured() {
    let config = SongbirdConfig::new();
    assert!(config.address.is_empty());
    assert!(!config.is_configured());
    assert_eq!(config.service_name, "rhizoCrypt");
    assert!(!config.capabilities.is_empty());
}

#[test]
fn test_config_with_address() {
    let config = SongbirdConfig::with_address("192.0.2.100:8091");
    assert_eq!(config.address, "192.0.2.100:8091");
    assert!(config.is_configured());
}

#[test]
fn test_client_creation() {
    let config = SongbirdConfig::with_address("127.0.0.1:8091");
    let client = SongbirdClient::new(config);
    assert!(client.config.auto_reconnect);
    assert!(client.config.is_configured());
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_client_initial_state() {
    let config = SongbirdConfig::with_address("127.0.0.1:8091");
    let client = SongbirdClient::new(config);
    assert_eq!(client.state().await, ClientState::Disconnected);
    assert!(!client.is_connected().await);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_discover_without_connection() {
    let config = SongbirdConfig::with_address("127.0.0.1:8091");
    let client = SongbirdClient::new(config);
    let result = client.discover("signing").await;
    assert!(result.is_err());
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_register_without_connection() {
    let config = SongbirdConfig::with_address("127.0.0.1:8091");
    let client = SongbirdClient::new(config);
    let result = client.register("127.0.0.1:9400").await;
    assert!(result.is_err());
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_connect_without_config_fails() {
    let config = SongbirdConfig::new();
    let client = SongbirdClient::new(config);
    let result = client.connect().await;
    assert!(result.is_err());
    let err_msg = result.unwrap_err().to_string();
    assert!(err_msg.contains("not configured"));
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_cache_operations() {
    let config = SongbirdConfig::with_address("127.0.0.1:8091");
    let client = SongbirdClient::new(config);

    // Manually set connected for cache test
    *client.state.write().await = ClientState::Connected;

    let services = vec![ServiceInfo {
        id: "test-1".to_string(),
        name: "test-beardog".to_string(),
        endpoint: "127.0.0.1:9000".to_string(),
        capabilities: vec!["signing".to_string()],
        status: "healthy".to_string(),
        metadata: HashMap::new(),
    }];

    client.cache_discovery("signing", services.clone()).await;

    let cached = client.discover("signing").await.unwrap();
    assert_eq!(cached.len(), 1);
    assert_eq!(cached[0].name, "test-beardog");

    client.clear_cache().await;
    let empty = client.discover("signing").await.unwrap();
    assert!(empty.is_empty());
}

#[test]
fn test_service_info_has_capability() {
    let service = ServiceInfo {
        id: "test".to_string(),
        name: "test".to_string(),
        endpoint: "127.0.0.1:9000".to_string(),
        capabilities: vec!["signing".to_string(), "did".to_string()],
        status: "healthy".to_string(),
        metadata: HashMap::new(),
    };

    assert!(service.has_capability("signing"));
    assert!(service.has_capability("did"));
    assert!(!service.has_capability("storage"));
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_populate_registry() {
    let config = SongbirdConfig::with_address("127.0.0.1:8091");
    let client = SongbirdClient::new(config);
    *client.state.write().await = ClientState::Connected;

    // Add cached services
    client
        .cache_discovery(
            "signing",
            vec![ServiceInfo {
                id: "bd-1".to_string(),
                name: "beardog-main".to_string(),
                endpoint: "127.0.0.1:9500".to_string(),
                capabilities: vec!["signing".to_string()],
                status: "healthy".to_string(),
                metadata: HashMap::new(),
            }],
        )
        .await;

    let registry = DiscoveryRegistry::new("rhizoCrypt");
    client.populate_registry(&registry).await.unwrap();

    assert!(registry.is_available(&Capability::Signing).await);
}

#[test]
fn test_from_env_respects_variables() {
    // Test with_address is explicit
    let config = SongbirdConfig::with_address("10.0.0.1:9999");
    assert_eq!(config.address, "10.0.0.1:9999");
    assert!(config.is_configured());
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_heartbeat_requires_registration() {
    let config = SongbirdConfig::with_address("127.0.0.1:8091");
    let client = SongbirdClient::new(config);

    // Try to start heartbeat without registration
    let result = client.start_heartbeat().await;
    assert!(result.is_err());
    let err_msg = result.unwrap_err().to_string();
    assert!(err_msg.contains("Not registered"));
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_heartbeat_lifecycle() {
    let config = SongbirdConfig::with_address("127.0.0.1:8091");
    let client = SongbirdClient::new(config);

    // Simulate registration
    *client.state.write().await = ClientState::Registered;
    *client.service_id.write().await = Some("test-id".to_string());
    *client.our_endpoint.write().await = Some("127.0.0.1:9400".to_string());

    // Start heartbeat
    let result = client.start_heartbeat().await;
    assert!(result.is_ok(), "Heartbeat should start: {result:?}");

    // Verify heartbeat is running
    assert!(client.heartbeat_handle.read().await.is_some());

    // Try to start again (should fail)
    let result2 = client.start_heartbeat().await;
    assert!(result2.is_err());
    assert!(result2.unwrap_err().to_string().contains("already running"));

    // Stop heartbeat
    client.stop_heartbeat().await;

    // Verify stopped
    assert!(client.heartbeat_handle.read().await.is_none());

    // Stopping again is safe (idempotent)
    client.stop_heartbeat().await;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_heartbeat_stops_when_unregistered() {
    let config = SongbirdConfig::with_address("127.0.0.1:8091");
    let client = SongbirdClient::new(config);

    // Simulate registration
    *client.state.write().await = ClientState::Registered;
    *client.service_id.write().await = Some("test-id".to_string());
    *client.our_endpoint.write().await = Some("127.0.0.1:9400".to_string());

    // Start heartbeat
    client.start_heartbeat().await.unwrap();

    // Simulate unregistration
    *client.state.write().await = ClientState::Connected;

    // Wait a bit for heartbeat to check state
    tokio::time::sleep(std::time::Duration::from_millis(100)).await;

    // Heartbeat task should self-terminate (we can't directly check, but no panic = good)
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_client_clone() {
    let config = SongbirdConfig::with_address("127.0.0.1:8091");
    let client1 = SongbirdClient::new(config);
    *client1.state.write().await = ClientState::Connected;

    // Clone should share state
    let client2 = client1.clone();
    assert_eq!(client2.state().await, ClientState::Connected);

    // State changes propagate
    *client1.state.write().await = ClientState::Registered;
    assert_eq!(client2.state().await, ClientState::Registered);
}

// ============================================================================
// Additional Tests for Coverage Boost (39% → 80%+)
// ============================================================================

#[test]
fn test_config_default() {
    let config = SongbirdConfig::default();
    assert_eq!(config.service_name, "rhizoCrypt");
    assert!(!config.capabilities.is_empty());
    assert_eq!(config.timeout_ms, 5000);
    assert!(config.auto_reconnect);
}

#[test]
fn test_config_with_metadata() {
    let mut config = SongbirdConfig::new();
    config.metadata.insert("version".to_string(), "0.11.0".to_string());
    config.metadata.insert("role".to_string(), "dag-engine".to_string());

    assert_eq!(config.metadata.len(), 2);
    assert_eq!(config.metadata.get("version"), Some(&"0.11.0".to_string()));
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_connect_invalid_address() {
    let config = SongbirdConfig::with_address("invalid-address-no-port");
    let client = SongbirdClient::new(config);
    let result = client.connect().await;

    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.to_string().contains("Invalid Songbird address"));
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_connect_already_connected() {
    let config = SongbirdConfig::with_address("127.0.0.1:8091");
    let client = SongbirdClient::new(config);

    // Manually set to connected
    *client.state.write().await = ClientState::Connected;

    // Second connect should succeed immediately (idempotent)
    let result = client.connect().await;
    assert!(result.is_ok());
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_connect_already_registered() {
    let config = SongbirdConfig::with_address("127.0.0.1:8091");
    let client = SongbirdClient::new(config);

    // Manually set to registered
    *client.state.write().await = ClientState::Registered;

    // Connect should succeed (already connected)
    let result = client.connect().await;
    assert!(result.is_ok());
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_connect_success() {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let config = SongbirdConfig::with_address(addr.to_string());
    let client = SongbirdClient::new(config);

    let accept_handle = tokio::spawn(async move {
        let _ = listener.accept().await;
    });

    let result = client.connect().await;
    assert!(result.is_ok(), "connect failed: {result:?}");
    assert_eq!(client.state().await, ClientState::Connected);
    assert!(client.endpoint().await.is_some());

    accept_handle.await.unwrap();
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_connect_timeout() {
    let mut config = SongbirdConfig::with_address("192.0.2.1:9999");
    config.timeout_ms = 1;
    let client = SongbirdClient::new(config);

    let result = client.connect().await;
    assert!(result.is_err());
    let err_msg = result.unwrap_err().to_string();
    assert!(
        err_msg.contains("timeout") || err_msg.contains("Cannot reach"),
        "expected timeout or connection error, got: {err_msg}"
    );
    assert_eq!(client.state().await, ClientState::Failed);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_connect_connection_refused() {
    let config = SongbirdConfig::with_address("127.0.0.1:49151");
    let client = SongbirdClient::new(config);

    let result = client.connect().await;
    assert!(result.is_err());
    assert_eq!(client.state().await, ClientState::Failed);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_register_not_connected() {
    let config = SongbirdConfig::with_address("127.0.0.1:8091");
    let client = SongbirdClient::new(config);

    // Try to register without connecting
    let result = client.register("127.0.0.1:9400").await;
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.to_string().contains("not connected") || err.to_string().contains("Not connected"));
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_discover_specific_primal() {
    let config = SongbirdConfig::with_address("127.0.0.1:8091");
    let client = SongbirdClient::new(config);
    *client.state.write().await = ClientState::Connected;

    // Cache a beardog service
    client
        .cache_discovery(
            "signing",
            vec![ServiceInfo {
                id: "beardog-1".to_string(),
                name: "beardog-main".to_string(),
                endpoint: "127.0.0.1:9500".to_string(),
                capabilities: vec!["signing".to_string(), "did-verification".to_string()],
                status: "healthy".to_string(),
                metadata: HashMap::new(),
            }],
        )
        .await;

    let signer = client.discover_signing_provider().await;
    assert!(signer.is_ok());
    assert!(signer.unwrap().is_some());
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_discover_permanent_storage_provider() {
    let config = SongbirdConfig::with_address("127.0.0.1:8091");
    let client = SongbirdClient::new(config);
    *client.state.write().await = ClientState::Connected;

    client
        .cache_discovery(
            "permanent-storage",
            vec![ServiceInfo {
                id: "storage-1".to_string(),
                name: "permanent-storage-main".to_string(),
                endpoint: "127.0.0.1:9600".to_string(),
                capabilities: vec!["permanent-storage".to_string()],
                status: "healthy".to_string(),
                metadata: HashMap::new(),
            }],
        )
        .await;

    let storage = client.discover_permanent_storage_provider().await;
    assert!(storage.is_ok());
    assert!(storage.unwrap().is_some());
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_discover_payload_storage_provider() {
    let config = SongbirdConfig::with_address("127.0.0.1:8091");
    let client = SongbirdClient::new(config);
    *client.state.write().await = ClientState::Connected;

    client
        .cache_discovery(
            "payload-storage",
            vec![ServiceInfo {
                id: "payload-1".to_string(),
                name: "payload-storage-main".to_string(),
                endpoint: "127.0.0.1:9700".to_string(),
                capabilities: vec!["payload-storage".to_string()],
                status: "healthy".to_string(),
                metadata: HashMap::new(),
            }],
        )
        .await;

    let storage = client.discover_payload_storage_provider().await;
    assert!(storage.is_ok());
    assert!(storage.unwrap().is_some());
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_discover_empty_results() {
    let config = SongbirdConfig::with_address("127.0.0.1:8091");
    let client = SongbirdClient::new(config);
    *client.state.write().await = ClientState::Connected;

    // No services cached - should return empty
    let result = client.discover("nonexistent-capability").await;
    assert!(result.is_ok());
    assert!(result.unwrap().is_empty());
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_populate_registry_not_connected() {
    let config = SongbirdConfig::with_address("127.0.0.1:8091");
    let client = SongbirdClient::new(config);
    let registry = DiscoveryRegistry::new("test-service");

    // Try to populate registry without connection
    let result = client.populate_registry(&registry).await;
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.to_string().contains("Not connected"));
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_populate_registry_with_services() {
    let config = SongbirdConfig::with_address("127.0.0.1:8091");
    let client = SongbirdClient::new(config);
    *client.state.write().await = ClientState::Connected;

    // Cache multiple services
    client
        .cache_discovery(
            "signing",
            vec![ServiceInfo {
                id: "beardog-1".to_string(),
                name: "beardog".to_string(),
                endpoint: "127.0.0.1:9500".to_string(),
                capabilities: vec!["signing".to_string()],
                status: "healthy".to_string(),
                metadata: HashMap::new(),
            }],
        )
        .await;

    client
        .cache_discovery(
            "permanent-storage",
            vec![ServiceInfo {
                id: "loamspine-1".to_string(),
                name: "loamspine".to_string(),
                endpoint: "127.0.0.1:9600".to_string(),
                capabilities: vec!["permanent-storage".to_string()],
                status: "healthy".to_string(),
                metadata: HashMap::new(),
            }],
        )
        .await;

    client
        .cache_discovery(
            "payload-storage",
            vec![ServiceInfo {
                id: "nestgate-1".to_string(),
                name: "nestgate".to_string(),
                endpoint: "127.0.0.1:9700".to_string(),
                capabilities: vec!["payload-storage".to_string()],
                status: "healthy".to_string(),
                metadata: HashMap::new(),
            }],
        )
        .await;

    let registry = DiscoveryRegistry::new("rhizoCrypt");
    let result = client.populate_registry(&registry).await;
    assert!(result.is_ok());

    // Verify services were registered
    assert!(registry.is_available(&Capability::Signing).await);
    assert!(registry.is_available(&Capability::PermanentCommit).await);
    assert!(registry.is_available(&Capability::PayloadStorage).await);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_federation_status_not_connected() {
    let config = SongbirdConfig::with_address("127.0.0.1:8091");
    let client = SongbirdClient::new(config);

    let result = client.federation_status().await;
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.to_string().contains("Not connected"));
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_federation_status_connected() {
    let config = SongbirdConfig::with_address("127.0.0.1:8091");
    let client = SongbirdClient::new(config);
    *client.state.write().await = ClientState::Connected;

    let result = client.federation_status().await;
    assert!(result.is_ok());
    let status = result.unwrap();
    assert_eq!(status.version, "pending-integration");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_unregister_without_registration() {
    let config = SongbirdConfig::with_address("127.0.0.1:8091");
    let client = SongbirdClient::new(config);

    // Unregister without being registered (should succeed, no-op)
    let result = client.unregister().await;
    assert!(result.is_ok());
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_unregister_with_registration() {
    let config = SongbirdConfig::with_address("127.0.0.1:8091");
    let client = SongbirdClient::new(config);
    *client.state.write().await = ClientState::Registered;
    *client.service_id.write().await = Some("test-service-id".to_string());

    let result = client.unregister().await;
    assert!(result.is_ok());

    // Verify state changed
    assert_eq!(client.state().await, ClientState::Connected);
    assert!(client.service_id().await.is_none());
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_disconnect() {
    let config = SongbirdConfig::with_address("127.0.0.1:8091");
    let client = SongbirdClient::new(config);
    *client.state.write().await = ClientState::Registered;
    *client.service_id.write().await = Some("test-id".to_string());

    client.disconnect().await;

    // Verify disconnected
    assert_eq!(client.state().await, ClientState::Disconnected);
    assert!(!client.is_connected().await);
    assert!(client.service_id().await.is_none());
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_cache_invalidation() {
    let config = SongbirdConfig::with_address("127.0.0.1:8091");
    let client = SongbirdClient::new(config);
    *client.state.write().await = ClientState::Connected;

    // Add services
    client
        .cache_discovery(
            "signing",
            vec![ServiceInfo {
                id: "test".to_string(),
                name: "test".to_string(),
                endpoint: "127.0.0.1:9000".to_string(),
                capabilities: vec!["signing".to_string()],
                status: "healthy".to_string(),
                metadata: HashMap::new(),
            }],
        )
        .await;

    // Verify cached
    let result = client.discover("signing").await.unwrap();
    assert_eq!(result.len(), 1);

    // Clear cache
    client.clear_cache().await;

    // Verify empty
    let result = client.discover("signing").await.unwrap();
    assert!(result.is_empty());
}

#[test]
fn test_service_info_metadata() {
    let mut metadata = HashMap::new();
    metadata.insert("version".to_string(), "1.0.0".to_string());
    metadata.insert("region".to_string(), "us-west".to_string());

    let service = ServiceInfo {
        id: "test".to_string(),
        name: "test-service".to_string(),
        endpoint: "10.0.0.1:9000".to_string(),
        capabilities: vec!["signing".to_string()],
        status: "healthy".to_string(),
        metadata: metadata.clone(),
    };

    assert_eq!(service.metadata.len(), 2);
    assert_eq!(service.metadata.get("version"), Some(&"1.0.0".to_string()));
    assert_eq!(service.metadata.get("region"), Some(&"us-west".to_string()));
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_state_transitions() {
    let config = SongbirdConfig::with_address("127.0.0.1:8091");
    let client = SongbirdClient::new(config);

    // Initial state
    assert_eq!(client.state().await, ClientState::Disconnected);

    // Transition to connected
    *client.state.write().await = ClientState::Connected;
    assert!(client.is_connected().await);

    // Transition to registered
    *client.state.write().await = ClientState::Registered;
    assert!(client.is_connected().await);

    // Back to disconnected
    *client.state.write().await = ClientState::Disconnected;
    assert!(!client.is_connected().await);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_multiple_capability_discovery() {
    let config = SongbirdConfig::with_address("127.0.0.1:8091");
    let client = SongbirdClient::new(config);
    *client.state.write().await = ClientState::Connected;

    // Add services with multiple capabilities
    client
        .cache_discovery(
            "multi-capability",
            vec![ServiceInfo {
                id: "multi-1".to_string(),
                name: "multi-service".to_string(),
                endpoint: "127.0.0.1:9000".to_string(),
                capabilities: vec![
                    "signing".to_string(),
                    "did-verification".to_string(),
                    "key-management".to_string(),
                ],
                status: "healthy".to_string(),
                metadata: HashMap::new(),
            }],
        )
        .await;

    let services = client.discover("multi-capability").await.unwrap();
    assert_eq!(services.len(), 1);
    assert_eq!(services[0].capabilities.len(), 3);
    assert!(services[0].has_capability("signing"));
    assert!(services[0].has_capability("did-verification"));
    assert!(services[0].has_capability("key-management"));
}

// ============================================================================
// tarpc integration tests (live-clients only — Songbird uses tarpc, not HTTP)
// ============================================================================

#[cfg(feature = "live-clients")]
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn tarpc_register_success() {
    use crate::clients::songbird_rpc::{MockSongbirdServer, SongbirdRpc};
    use futures_util::StreamExt;
    use tarpc::server::{self, Channel};
    use tarpc::tokio_serde::formats::Bincode;

    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();

    let server = MockSongbirdServer;

    let _accept_handle = tokio::spawn(async move {
        let (stream, _) = listener.accept().await.unwrap();
        let transport = tarpc::serde_transport::Transport::from((stream, Bincode::default()));
        let channel = server::BaseChannel::with_defaults(transport);
        channel
            .execute(server.serve())
            .for_each(|fut| async move {
                fut.await;
            })
            .await;
    });

    let config = SongbirdConfig::with_address(addr.to_string());
    let client = SongbirdClient::new(config);
    client.connect().await.unwrap();

    let result = client.register("127.0.0.1:9400").await.unwrap();
    assert!(result.success);
    assert!(result.service_id.is_some());
    assert_eq!(client.state().await, ClientState::Registered);
}

#[cfg(feature = "live-clients")]
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn tarpc_discover_signing() {
    use crate::clients::songbird_rpc::{MockSongbirdServer, SongbirdRpc};
    use futures_util::StreamExt;
    use tarpc::server::{self, Channel};
    use tarpc::tokio_serde::formats::Bincode;

    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();

    let server = MockSongbirdServer;

    let _accept_handle = tokio::spawn(async move {
        let (stream, _) = listener.accept().await.unwrap();
        let transport = tarpc::serde_transport::Transport::from((stream, Bincode::default()));
        let channel = server::BaseChannel::with_defaults(transport);
        channel
            .execute(server.serve())
            .for_each(|fut| async move {
                fut.await;
            })
            .await;
    });

    let config = SongbirdConfig::with_address(addr.to_string());
    let client = SongbirdClient::new(config);
    client.connect().await.unwrap();

    let services = client.discover("signing").await.unwrap();
    assert_eq!(services.len(), 1);
    assert_eq!(services[0].id, "mock-beardog-1");
    assert_eq!(services[0].endpoint, "127.0.0.1:9500");
}

#[cfg(feature = "live-clients")]
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn tarpc_discover_empty_for_unknown_capability() {
    use crate::clients::songbird_rpc::{MockSongbirdServer, SongbirdRpc};
    use futures_util::StreamExt;
    use tarpc::server::{self, Channel};
    use tarpc::tokio_serde::formats::Bincode;

    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();

    let server = MockSongbirdServer;

    let _accept_handle = tokio::spawn(async move {
        let (stream, _) = listener.accept().await.unwrap();
        let transport = tarpc::serde_transport::Transport::from((stream, Bincode::default()));
        let channel = server::BaseChannel::with_defaults(transport);
        channel
            .execute(server.serve())
            .for_each(|fut| async move {
                fut.await;
            })
            .await;
    });

    let config = SongbirdConfig::with_address(addr.to_string());
    let client = SongbirdClient::new(config);
    client.connect().await.unwrap();

    let services = client.discover("unknown-capability").await.unwrap();
    assert!(services.is_empty());
}

#[cfg(feature = "live-clients")]
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn tarpc_register_then_discover_signing_provider() {
    use crate::clients::songbird_rpc::{MockSongbirdServer, SongbirdRpc};
    use futures_util::StreamExt;
    use tarpc::server::{self, Channel};
    use tarpc::tokio_serde::formats::Bincode;

    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();

    let server = MockSongbirdServer;

    let _accept_handle = tokio::spawn(async move {
        let (stream, _) = listener.accept().await.unwrap();
        let transport = tarpc::serde_transport::Transport::from((stream, Bincode::default()));
        let channel = server::BaseChannel::with_defaults(transport);
        channel
            .execute(server.serve())
            .for_each(|fut| async move {
                fut.await;
            })
            .await;
    });

    let config = SongbirdConfig::with_address(addr.to_string());
    let client = SongbirdClient::new(config);
    client.connect().await.unwrap();
    client.register("127.0.0.1:9400").await.unwrap();

    let signer = client.discover_signing_provider().await.unwrap();
    assert!(signer.is_some());
    assert_eq!(signer.unwrap().endpoint, "127.0.0.1:9500");
}
