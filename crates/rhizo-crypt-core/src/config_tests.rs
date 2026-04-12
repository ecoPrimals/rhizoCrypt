// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2024–2026 ecoPrimals Project

use super::*;

#[test]
fn test_config_default() {
    let config = RhizoCryptConfig::default();
    assert_eq!(config.name, constants::PRIMAL_NAME);
    assert_eq!(config.max_sessions, constants::DEFAULT_MAX_SESSIONS);
    assert_eq!(config.storage.backend, StorageBackend::Memory);
}

#[test]
fn test_config_builder() {
    let config = RhizoCryptConfig::new("TestRhizo")
        .with_max_sessions(500)
        .with_gc_interval(Duration::from_secs(120));

    assert_eq!(config.name, "TestRhizo");
    assert_eq!(config.max_sessions, 500);
    assert_eq!(config.gc_interval, Duration::from_secs(120));
}

#[test]
fn test_config_serialization() {
    let config = RhizoCryptConfig::default();
    let json = serde_json::to_string(&config).unwrap();
    let parsed: RhizoCryptConfig = serde_json::from_str(&json).unwrap();
    assert_eq!(config.name, parsed.name);
}

#[test]
fn test_storage_backend_default() {
    assert_eq!(StorageBackend::default(), StorageBackend::Memory);
}

#[test]
fn test_config_with_all_options() {
    let config = RhizoCryptConfig::new("FullConfig")
        .with_max_sessions(2000)
        .with_gc_interval(Duration::from_secs(90))
        .with_storage(StorageConfig {
            backend: StorageBackend::Redb,
            path: Some("/tmp/rhizo".to_string()),
            max_memory_bytes: Some(2 * 1024 * 1024 * 1024),
        });

    assert_eq!(config.name, "FullConfig");
    assert_eq!(config.max_sessions, 2000);
    assert_eq!(config.gc_interval, Duration::from_secs(90));
    assert_eq!(config.storage.backend, StorageBackend::Redb);
    assert_eq!(config.storage.path.as_deref(), Some("/tmp/rhizo"));
    assert_eq!(config.storage.max_memory_bytes, Some(2 * 1024 * 1024 * 1024));
}

#[test]
fn test_config_from_env() {
    temp_env::with_vars(
        [
            ("RHIZOCRYPT_RPC_HOST", Some("0.0.0.0")),
            ("RHIZOCRYPT_RPC_PORT", Some("9090")),
            ("RHIZOCRYPT_RPC_ENABLED", Some("false")),
        ],
        || {
            let rpc = RpcConfig::from_env_or_default();
            assert_eq!(rpc.host.as_ref(), "0.0.0.0");
            assert_eq!(rpc.port, 9090);
            assert!(!rpc.enabled);
        },
    );
}

#[test]
fn test_config_from_env_reader_di_pattern() {
    use std::collections::HashMap;
    let env: HashMap<&str, &str> = [
        ("RHIZOCRYPT_RPC_HOST", "10.0.0.1"),
        ("RHIZOCRYPT_RPC_PORT", "8888"),
        ("RHIZOCRYPT_RPC_ENABLED", "true"),
    ]
    .into_iter()
    .collect();

    let rpc = RpcConfig::from_env_reader(|key| {
        env.get(key).map(|v| (*v).to_string()).ok_or(std::env::VarError::NotPresent)
    });
    assert_eq!(rpc.host.as_ref(), "10.0.0.1");
    assert_eq!(rpc.port, 8888);
    assert!(rpc.enabled);
}

#[test]
fn test_config_from_env_reader_defaults() {
    let rpc = RpcConfig::from_env_reader(|_| Err(std::env::VarError::NotPresent));
    assert_eq!(rpc.host.as_ref(), constants::DEFAULT_RPC_HOST);
    assert_eq!(rpc.port, constants::DEFAULT_RPC_PORT);
    assert!(rpc.enabled);
    assert_eq!(rpc.max_connections, constants::DEFAULT_MAX_CONNECTIONS);
}

#[test]
fn test_config_from_env_reader_partial_override() {
    let rpc = RpcConfig::from_env_reader(|key| {
        if key == "RHIZOCRYPT_RPC_PORT" {
            Ok("7777".to_string())
        } else {
            Err(std::env::VarError::NotPresent)
        }
    });
    assert_eq!(rpc.host.as_ref(), constants::DEFAULT_RPC_HOST);
    assert_eq!(rpc.port, 7777);
    assert!(rpc.enabled);
}

#[test]
fn test_storage_backend_variants() {
    assert_eq!(StorageBackend::Memory, StorageBackend::Memory);
    assert_eq!(StorageBackend::Redb, StorageBackend::Redb);
    assert_ne!(StorageBackend::Memory, StorageBackend::Redb);
}

#[test]
fn test_config_validation() {
    let config = RhizoCryptConfig::default();
    assert!(!config.name.is_empty());
    assert!(config.max_sessions > 0);
    assert!(config.gc_interval.as_secs() > 0);
}

#[test]
fn test_config_clone() {
    let config = RhizoCryptConfig::new("CloneTest").with_max_sessions(100);
    let cloned = config.clone();
    assert_eq!(config.name, cloned.name);
    assert_eq!(config.max_sessions, cloned.max_sessions);
    assert_eq!(config.storage.backend, cloned.storage.backend);
}
