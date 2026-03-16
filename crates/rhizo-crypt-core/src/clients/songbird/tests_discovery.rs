// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2024–2026 ecoPrimals Project

//! Discovery, cache, and registry population tests for Songbird client.

use super::{SongbirdClient, SongbirdConfig};
use crate::clients::songbird_types::{ClientState, ServiceInfo};
use crate::discovery::{Capability, DiscoveryRegistry};
use std::collections::HashMap;

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_discover_without_connection() {
    let config = SongbirdConfig::with_address("127.0.0.1:8091");
    let client = SongbirdClient::new(config);
    let result = client.discover("signing").await;
    assert!(result.is_err());
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_cache_operations() {
    let config = SongbirdConfig::with_address("127.0.0.1:8091");
    let client = SongbirdClient::new(config);
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

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_populate_registry() {
    let config = SongbirdConfig::with_address("127.0.0.1:8091");
    let client = SongbirdClient::new(config);
    *client.state.write().await = ClientState::Connected;

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

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_discover_specific_primal() {
    let config = SongbirdConfig::with_address("127.0.0.1:8091");
    let client = SongbirdClient::new(config);
    *client.state.write().await = ClientState::Connected;

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

    let result = client.discover("nonexistent-capability").await;
    assert!(result.is_ok());
    assert!(result.unwrap().is_empty());
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_populate_registry_not_connected() {
    let config = SongbirdConfig::with_address("127.0.0.1:8091");
    let client = SongbirdClient::new(config);
    let registry = DiscoveryRegistry::new("test-service");

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

    assert!(registry.is_available(&Capability::Signing).await);
    assert!(registry.is_available(&Capability::PermanentCommit).await);
    assert!(registry.is_available(&Capability::PayloadStorage).await);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_cache_invalidation() {
    let config = SongbirdConfig::with_address("127.0.0.1:8091");
    let client = SongbirdClient::new(config);
    *client.state.write().await = ClientState::Connected;

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

    let result = client.discover("signing").await.unwrap();
    assert_eq!(result.len(), 1);

    client.clear_cache().await;

    let result = client.discover("signing").await.unwrap();
    assert!(result.is_empty());
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_multiple_capability_discovery() {
    let config = SongbirdConfig::with_address("127.0.0.1:8091");
    let client = SongbirdClient::new(config);
    *client.state.write().await = ClientState::Connected;

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

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_populate_registry_skips_invalid_endpoint() {
    let config = SongbirdConfig::with_address("127.0.0.1:8091");
    let client = SongbirdClient::new(config);
    *client.state.write().await = ClientState::Connected;

    client
        .cache_discovery(
            "signing",
            vec![ServiceInfo {
                id: "bad-1".to_string(),
                name: "bad-endpoint".to_string(),
                endpoint: "not-a-valid-address".to_string(),
                capabilities: vec!["signing".to_string()],
                status: "healthy".to_string(),
                metadata: HashMap::new(),
            }],
        )
        .await;

    let registry = DiscoveryRegistry::new("rhizoCrypt");
    let result = client.populate_registry(&registry).await;
    assert!(result.is_ok());
    assert!(!registry.is_available(&Capability::Signing).await);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_discover_signing_provider_returns_none_when_empty() {
    let config = SongbirdConfig::with_address("127.0.0.1:8091");
    let client = SongbirdClient::new(config);
    *client.state.write().await = ClientState::Connected;

    let signer = client.discover_signing_provider().await.unwrap();
    assert!(signer.is_none());
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_discover_permanent_storage_provider_returns_none_when_empty() {
    let config = SongbirdConfig::with_address("127.0.0.1:8091");
    let client = SongbirdClient::new(config);
    *client.state.write().await = ClientState::Connected;

    let storage = client.discover_permanent_storage_provider().await.unwrap();
    assert!(storage.is_none());
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_discover_payload_storage_provider_returns_none_when_empty() {
    let config = SongbirdConfig::with_address("127.0.0.1:8091");
    let client = SongbirdClient::new(config);
    *client.state.write().await = ClientState::Connected;

    let storage = client.discover_payload_storage_provider().await.unwrap();
    assert!(storage.is_none());
}
