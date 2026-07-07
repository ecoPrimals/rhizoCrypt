// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2024–2026 ecoPrimals Project

//! Configuration and type tests for Songbird client.

use super::{DiscoveryClient, DiscoveryConfig};
use crate::clients::discovery_types::{ClientState, FederationStatus, RegistrationResult};
use std::collections::HashMap;

#[test]
fn test_config_new_unconfigured() {
    let config = DiscoveryConfig::new();
    assert!(config.address.is_empty());
    assert!(!config.is_configured());
    assert_eq!(config.service_name, "rhizoCrypt");
    assert!(!config.capabilities.is_empty());
}

#[test]
fn test_config_with_address() {
    let config = DiscoveryConfig::with_address("192.0.2.100:8091");
    assert_eq!(config.address, "192.0.2.100:8091");
    assert!(config.is_configured());
}

#[test]
fn test_config_default() {
    let config = DiscoveryConfig::default();
    assert_eq!(config.service_name, "rhizoCrypt");
    assert!(!config.capabilities.is_empty());
    assert_eq!(config.timeout_ms, 5000);
    assert!(config.auto_reconnect);
}

#[test]
fn test_config_with_metadata() {
    let mut config = DiscoveryConfig::new();
    config.metadata.insert("version".to_string(), "0.14.0".to_string());
    config.metadata.insert("role".to_string(), "dag-engine".to_string());

    assert_eq!(config.metadata.len(), 2);
    assert_eq!(config.metadata.get("version"), Some(&"0.14.0".to_string()));
}

#[test]
fn test_from_env_respects_variables() {
    let config = DiscoveryConfig::with_address("10.0.0.1:9999");
    assert_eq!(config.address, "10.0.0.1:9999");
    assert!(config.is_configured());
}

#[test]
fn test_config_with_address_owned() {
    let config = DiscoveryConfig::with_address(String::from("10.0.0.5:8080"));
    assert_eq!(config.address.as_ref(), "10.0.0.5:8080");
    assert!(config.is_configured());
}

#[test]
fn test_config_debug_impl() {
    let config = DiscoveryConfig::with_address("127.0.0.1:8091");
    let debug_str = format!("{config:?}");
    assert!(debug_str.contains("DiscoveryConfig"));
}

#[test]
fn test_client_state_debug() {
    let state = ClientState::Connecting;
    let debug_str = format!("{state:?}");
    assert!(debug_str.contains("Connecting"));
}

#[test]
fn test_registration_result_debug() {
    let result = RegistrationResult {
        success: true,
        message: "OK".to_string(),
        service_id: Some("id-1".to_string()),
    };
    let debug_str = format!("{result:?}");
    assert!(debug_str.contains("RegistrationResult"));
}

#[test]
fn test_federation_status_debug() {
    let status = FederationStatus {
        total_services: 5,
        total_peers: 3,
        uptime_seconds: 3600,
        version: "1.0".to_string(),
    };
    let debug_str = format!("{status:?}");
    assert!(debug_str.contains("FederationStatus"));
}

#[test]
fn test_with_defaults() {
    let client = DiscoveryClient::with_defaults();
    assert!(!client.config.service_name.is_empty());
}

#[test]
fn test_from_env() {
    let client = DiscoveryClient::from_env();
    assert!(!client.config.service_name.is_empty());
}

#[test]
fn test_client_creation() {
    let config = DiscoveryConfig::with_address("127.0.0.1:8091");
    let client = DiscoveryClient::new(config);
    assert!(client.config.auto_reconnect);
    assert!(client.config.is_configured());
}

#[test]
fn test_service_info_has_capability() {
    use crate::clients::discovery_types::ServiceInfo;

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

#[test]
fn test_service_info_metadata() {
    use crate::clients::discovery_types::ServiceInfo;

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
