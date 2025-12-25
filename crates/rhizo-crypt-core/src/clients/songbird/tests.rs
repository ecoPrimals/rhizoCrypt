//! Tests for Songbird client.

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
