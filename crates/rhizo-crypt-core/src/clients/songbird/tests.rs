// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2024–2026 ecoPrimals Project

//! Core client lifecycle tests for Songbird client.
//!
//! Connection, registration, heartbeat, and state transition tests.
//! Discovery tests are in `tests_discovery.rs`, config/type tests in
//! `tests_config.rs`, and tarpc integration tests in `tests_tarpc.rs`.

use super::{SongbirdClient, SongbirdConfig};
use crate::clients::songbird_types::ClientState;

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_client_initial_state() {
    let config = SongbirdConfig::with_address("127.0.0.1:8091");
    let client = SongbirdClient::new(config);
    assert_eq!(client.state().await, ClientState::Disconnected);
    assert!(!client.is_connected().await);
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
async fn test_heartbeat_requires_registration() {
    let config = SongbirdConfig::with_address("127.0.0.1:8091");
    let client = SongbirdClient::new(config);

    let result = client.start_heartbeat().await;
    assert!(result.is_err());
    let err_msg = result.unwrap_err().to_string();
    assert!(err_msg.contains("Not registered"));
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_heartbeat_lifecycle() {
    let config = SongbirdConfig::with_address("127.0.0.1:8091");
    let client = SongbirdClient::new(config);

    *client.state.write().await = ClientState::Registered;
    *client.service_id.write().await = Some("test-id".to_string());
    *client.our_endpoint.write().await = Some("127.0.0.1:9400".to_string());

    let result = client.start_heartbeat().await;
    assert!(result.is_ok(), "Heartbeat should start: {result:?}");

    assert!(client.heartbeat_handle.read().await.is_some());

    let result2 = client.start_heartbeat().await;
    assert!(result2.is_err());
    assert!(result2.unwrap_err().to_string().contains("already running"));

    client.stop_heartbeat().await;

    assert!(client.heartbeat_handle.read().await.is_none());

    client.stop_heartbeat().await;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_heartbeat_stops_when_unregistered() {
    let config = SongbirdConfig::with_address("127.0.0.1:8091");
    let client = SongbirdClient::new(config);

    *client.state.write().await = ClientState::Registered;
    *client.service_id.write().await = Some("test-id".to_string());
    *client.our_endpoint.write().await = Some("127.0.0.1:9400".to_string());

    client.start_heartbeat().await.unwrap();

    *client.state.write().await = ClientState::Connected;

    tokio::time::sleep(std::time::Duration::from_millis(100)).await;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_client_clone() {
    let config = SongbirdConfig::with_address("127.0.0.1:8091");
    let client1 = SongbirdClient::new(config);
    *client1.state.write().await = ClientState::Connected;

    let client2 = client1.clone();
    assert_eq!(client2.state().await, ClientState::Connected);

    *client1.state.write().await = ClientState::Registered;
    assert_eq!(client2.state().await, ClientState::Registered);
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

    *client.state.write().await = ClientState::Connected;

    let result = client.connect().await;
    assert!(result.is_ok());
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_connect_already_registered() {
    let config = SongbirdConfig::with_address("127.0.0.1:8091");
    let client = SongbirdClient::new(config);

    *client.state.write().await = ClientState::Registered;

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

    let result = client.register("127.0.0.1:9400").await;
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.to_string().contains("not connected") || err.to_string().contains("Not connected"));
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
async fn test_federation_status_requires_live_clients() {
    let config = SongbirdConfig::with_address("127.0.0.1:8091");
    let client = SongbirdClient::new(config);
    *client.state.write().await = ClientState::Connected;

    #[cfg(not(feature = "live-clients"))]
    {
        let result = client.federation_status().await;
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("live-clients"), "expected live-clients error, got: {err}");
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_unregister_without_registration() {
    let config = SongbirdConfig::with_address("127.0.0.1:8091");
    let client = SongbirdClient::new(config);

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

    assert_eq!(client.state().await, ClientState::Disconnected);
    assert!(!client.is_connected().await);
    assert!(client.service_id().await.is_none());
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_state_transitions() {
    let config = SongbirdConfig::with_address("127.0.0.1:8091");
    let client = SongbirdClient::new(config);

    assert_eq!(client.state().await, ClientState::Disconnected);

    *client.state.write().await = ClientState::Connected;
    assert!(client.is_connected().await);

    *client.state.write().await = ClientState::Registered;
    assert!(client.is_connected().await);

    *client.state.write().await = ClientState::Disconnected;
    assert!(!client.is_connected().await);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_disconnect_idempotent_when_already_disconnected() {
    let config = SongbirdConfig::with_address("127.0.0.1:8091");
    let client = SongbirdClient::new(config);

    client.disconnect().await;

    assert_eq!(client.state().await, ClientState::Disconnected);
    assert!(client.endpoint().await.is_none());
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_endpoint_none_when_disconnected() {
    let config = SongbirdConfig::with_address("127.0.0.1:8091");
    let client = SongbirdClient::new(config);

    assert!(client.endpoint().await.is_none());
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_service_id_some_when_registered() {
    let config = SongbirdConfig::with_address("127.0.0.1:8091");
    let client = SongbirdClient::new(config);
    *client.state.write().await = ClientState::Registered;
    *client.service_id.write().await = Some("my-service-123".to_string());

    let id = client.service_id().await;
    assert_eq!(id, Some("my-service-123".to_string()));
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_register_fails_when_disconnected() {
    let config = SongbirdConfig::with_address("127.0.0.1:8091");
    let client = SongbirdClient::new(config);

    let result = client.register("127.0.0.1:9400").await;
    assert!(result.is_err(), "register should fail when not connected");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_stop_heartbeat_when_not_running() {
    let config = SongbirdConfig::with_address("127.0.0.1:8091");
    let client = SongbirdClient::new(config);

    client.stop_heartbeat().await;
}
