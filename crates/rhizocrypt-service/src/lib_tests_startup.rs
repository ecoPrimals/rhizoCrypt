// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2024–2026 ecoPrimals Project

//! Service startup, TCP/UDS config, and discovery registration tests.

use super::*;
use std::sync::Arc;
use std::time::Duration;

#[test]
fn test_has_explicit_tcp_config_unset() {
    temp_env::with_vars(
        [
            ("RHIZOCRYPT_PORT", None::<&str>),
            ("RHIZOCRYPT_RPC_PORT", None),
            ("RHIZOCRYPT_JSONRPC_PORT", None),
        ],
        || {
            assert!(!has_explicit_tcp_config());
        },
    );
}

#[test]
fn test_has_explicit_tcp_config_port_env() {
    temp_env::with_vars(
        [
            ("RHIZOCRYPT_PORT", Some("9400")),
            ("RHIZOCRYPT_RPC_PORT", None::<&str>),
            ("RHIZOCRYPT_JSONRPC_PORT", None),
        ],
        || {
            assert!(has_explicit_tcp_config());
        },
    );
}

#[test]
fn test_has_explicit_tcp_config_rpc_port_env() {
    temp_env::with_vars(
        [
            ("RHIZOCRYPT_PORT", None::<&str>),
            ("RHIZOCRYPT_RPC_PORT", Some("9401")),
            ("RHIZOCRYPT_JSONRPC_PORT", None),
        ],
        || {
            assert!(has_explicit_tcp_config());
        },
    );
}

#[test]
fn test_has_explicit_tcp_config_jsonrpc_port_env() {
    temp_env::with_vars(
        [
            ("RHIZOCRYPT_PORT", None::<&str>),
            ("RHIZOCRYPT_RPC_PORT", None),
            ("RHIZOCRYPT_JSONRPC_PORT", Some("9402")),
        ],
        || {
            assert!(has_explicit_tcp_config());
        },
    );
}

#[test]
fn test_run_server_btsp_env_conflict_returns_config_error() {
    temp_env::with_vars(
        [
            ("FAMILY_ID", Some("production-family")),
            ("BIOMEOS_INSECURE", Some("1")),
            ("RHIZOCRYPT_PORT", None::<&str>),
            ("RHIZOCRYPT_RPC_PORT", None),
            ("RHIZOCRYPT_JSONRPC_PORT", None),
        ],
        || {
            let rt = tokio::runtime::Builder::new_current_thread().enable_all().build().unwrap();
            let result = rt.block_on(run_server_with_ready(None, None, None, None));
            assert!(matches!(result, Err(ServiceError::Config(_))));
        },
    );
}

#[cfg(unix)]
#[test]
fn test_run_server_uds_only_mode() {
    let dir = tempfile::tempdir().expect("tempdir");
    let sock = dir.path().join("uds-only.sock");
    let sock_str = sock.to_string_lossy().to_string();

    temp_env::with_vars(
        [
            ("RHIZOCRYPT_PORT", None::<&str>),
            ("RHIZOCRYPT_RPC_PORT", None),
            ("RHIZOCRYPT_JSONRPC_PORT", None),
            ("RHIZOCRYPT_HOST", None),
            ("RHIZOCRYPT_RPC_HOST", None),
            ("DISCOVERY_ENDPOINT", None),
            ("RHIZOCRYPT_DISCOVERY_ADAPTER", None),
            ("RUST_LOG", Some("error")),
        ],
        || {
            let rt = tokio::runtime::Builder::new_multi_thread()
                .worker_threads(2)
                .enable_all()
                .build()
                .unwrap();
            let ready = Arc::new(tokio::sync::Notify::new());
            let ready_clone = Arc::clone(&ready);
            let handle = rt.spawn(async move {
                let _ = run_server_with_ready(None, None, Some(sock_str), Some(ready_clone)).await;
            });
            rt.block_on(async {
                tokio::time::timeout(Duration::from_secs(10), ready.notified())
                    .await
                    .expect("UDS-only server should become ready");
                assert!(sock.exists(), "UDS socket should exist in UDS-only mode");
                handle.abort();
                let _ = handle.await;
            });
        },
    );
}

#[cfg(unix)]
#[test]
fn test_run_server_tcp_via_jsonrpc_port_env() {
    temp_env::with_vars(
        [
            ("RHIZOCRYPT_JSONRPC_PORT", Some("0")),
            ("RHIZOCRYPT_PORT", None::<&str>),
            ("RHIZOCRYPT_RPC_PORT", None),
            ("RHIZOCRYPT_HOST", Some("127.0.0.1")),
            ("DISCOVERY_ENDPOINT", None),
            ("RHIZOCRYPT_DISCOVERY_ADAPTER", None),
            ("RUST_LOG", Some("error")),
        ],
        || {
            let rt = tokio::runtime::Builder::new_multi_thread()
                .worker_threads(2)
                .enable_all()
                .build()
                .unwrap();
            let ready = Arc::new(tokio::sync::Notify::new());
            let ready_clone = Arc::clone(&ready);
            let dir = tempfile::tempdir().expect("tempdir");
            let sock = dir.path().join("tcp-env.sock");
            let sock_str = sock.to_string_lossy().to_string();
            let handle = rt.spawn(async move {
                let _ = run_server_with_ready(None, None, Some(sock_str), Some(ready_clone)).await;
            });
            rt.block_on(async {
                tokio::time::timeout(Duration::from_secs(10), ready.notified())
                    .await
                    .expect("server should become ready with JSONRPC_PORT env");
                handle.abort();
                let _ = handle.await;
            });
        },
    );
}

#[test]
fn test_run_server_no_uds_backward_compat() {
    temp_env::with_vars(
        [
            ("RHIZOCRYPT_PORT", None::<&str>),
            ("RHIZOCRYPT_RPC_PORT", None),
            ("RHIZOCRYPT_JSONRPC_PORT", None),
            ("DISCOVERY_ENDPOINT", None),
            ("RHIZOCRYPT_DISCOVERY_ADAPTER", None),
            ("RUST_LOG", Some("error")),
        ],
        || {
            let rt = tokio::runtime::Builder::new_multi_thread()
                .worker_threads(2)
                .enable_all()
                .build()
                .unwrap();
            let ready = Arc::new(tokio::sync::Notify::new());
            let ready_clone = Arc::clone(&ready);
            let handle = rt.spawn(async move {
                let _ = run_server_with_ready(
                    Some(0),
                    Some("127.0.0.1".to_string()),
                    None,
                    Some(ready_clone),
                )
                .await;
            });
            rt.block_on(async {
                tokio::time::timeout(Duration::from_secs(10), ready.notified())
                    .await
                    .expect("TCP-only server should become ready");
                handle.abort();
                let _ = handle.await;
            });
        },
    );
}

#[test]
fn test_run_server_wrapper_delegates() {
    let dir = tempfile::tempdir().expect("tempdir");
    let sock = dir.path().join("wrapper.sock");
    let sock_str = sock.to_string_lossy().to_string();

    temp_env::with_vars(
        [
            ("FAMILY_ID", None::<&str>),
            ("BIOMEOS_INSECURE", Some("1")),
            ("RHIZOCRYPT_PORT", None::<&str>),
            ("RHIZOCRYPT_RPC_PORT", None),
            ("RHIZOCRYPT_JSONRPC_PORT", None),
            ("DISCOVERY_ENDPOINT", None),
            ("RUST_LOG", Some("error")),
        ],
        || {
            let rt = tokio::runtime::Builder::new_multi_thread()
                .worker_threads(2)
                .enable_all()
                .build()
                .unwrap();
            let handle = rt.spawn(async move {
                let _ = run_server(None, None, Some(sock_str)).await;
            });
            rt.block_on(async {
                tokio::time::timeout(Duration::from_secs(5), async {
                    loop {
                        if sock.exists() {
                            break;
                        }
                        tokio::time::sleep(Duration::from_millis(20)).await;
                    }
                })
                .await
                .expect("run_server should start UDS listener");
                handle.abort();
                let _ = handle.await;
            });
        },
    );
}
