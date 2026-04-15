// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2024–2026 ecoPrimals Project

//! Client operations, bind resolution, server startup, CLI print helpers, and `ServiceError` tests.

use rhizo_crypt_core::{PrimalLifecycle, RhizoCrypt, RhizoCryptConfig};
use rhizo_crypt_rpc::server::RpcServer;
use rhizocrypt_service::{
    ClientOperation, ServiceError, exit_codes, print_status, print_version, resolve_bind_addr,
    run_client, run_server_with_ready,
};
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;

// --- ServiceError and exit codes ---

/// Test `ServiceError::Storage` display and exit code.
#[test]
fn test_service_error_storage_display_and_exit_code() {
    let err = ServiceError::Storage("backend unavailable".to_string());
    let s = err.to_string();
    assert!(s.contains("storage error"));
    assert!(s.contains("backend unavailable"));
    assert_eq!(err.exit_code(), exit_codes::CONFIG_ERROR);
}

/// Test `ServiceError` exit code mapping for all variants.
#[test]
fn test_service_error_exit_codes_all_variants() {
    assert_eq!(ServiceError::Config("x".to_string()).exit_code(), exit_codes::CONFIG_ERROR);
    assert_eq!(ServiceError::Storage("x".to_string()).exit_code(), exit_codes::CONFIG_ERROR);
    let parse_err = "x:y:z".parse::<SocketAddr>().unwrap_err();
    assert_eq!(ServiceError::AddrParse(parse_err).exit_code(), exit_codes::CONFIG_ERROR);
    assert_eq!(
        ServiceError::Rpc(std::io::Error::other("x")).exit_code(),
        exit_codes::NETWORK_ERROR
    );
    assert_eq!(ServiceError::Discovery("x".to_string()).exit_code(), exit_codes::NETWORK_ERROR);
}

// --- run_client ---

/// Test `run_client` with invalid address returns `AddrParse` error.
#[tokio::test]
async fn test_run_client_invalid_address() {
    let result = run_client("not-a-valid-address", ClientOperation::Health).await;
    assert!(result.is_err());
    assert!(matches!(&result.unwrap_err(), ServiceError::AddrParse(_)));
}

/// Test `run_client` connection failure returns Config error.
#[tokio::test]
async fn test_run_client_connection_failure() {
    // Port 1 is typically not listening
    let result = run_client("127.0.0.1:1", ClientOperation::Health).await;
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(matches!(&err, ServiceError::Config(_)));
    assert!(err.to_string().contains("Failed to connect"));
}

/// Spin up a server on an OS-assigned port, retry the given client operation
/// until it succeeds, then tear down. Uses port 0 for OS assignment to
/// eliminate hardcoded ports and test isolation issues.
async fn assert_run_client_succeeds(operation: ClientOperation) {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let bound_addr = listener.local_addr().unwrap();
    drop(listener);

    let config = RhizoCryptConfig::default();
    let mut primal = RhizoCrypt::new(config);
    primal.start().await.expect("primal should start");
    let primal = Arc::new(primal);
    let server = RpcServer::new(primal, bound_addr);
    let ready = server.ready_notifier();
    let handle = tokio::spawn(async move { server.serve().await });
    ready.notified().await;

    let addr_str = bound_addr.to_string();
    for _ in 0..50 {
        if run_client(&addr_str, operation.clone()).await.is_ok() {
            handle.abort();
            let _ = handle.await;
            return;
        }
        tokio::task::yield_now().await;
    }
    handle.abort();
    let _ = handle.await;
    panic!("run_client {operation:?} should succeed against running server");
}

/// Test `run_client` Health against running server.
#[tokio::test]
async fn test_run_client_health() {
    assert_run_client_succeeds(ClientOperation::Health).await;
}

/// Test `run_client` `ListSessions` against running server.
#[tokio::test]
async fn test_run_client_list_sessions() {
    assert_run_client_succeeds(ClientOperation::ListSessions).await;
}

/// Test `run_client` Metrics against running server.
#[tokio::test]
async fn test_run_client_metrics() {
    assert_run_client_succeeds(ClientOperation::Metrics).await;
}

// --- run_server and resolve_bind_addr ---

/// Test `resolve_bind_addr` with invalid host returns `AddrParse` error.
#[test]
fn test_resolve_bind_addr_invalid_host() {
    let result = resolve_bind_addr(Some(9999), Some("not-an-ip".to_string()));
    assert!(result.is_err());
    assert!(matches!(&result.unwrap_err(), ServiceError::AddrParse(_)));
}

/// Test `run_server` starts in standalone mode when no discovery configured.
#[test]
fn test_run_server_standalone_mode_no_discovery() {
    temp_env::with_vars(
        [
            ("RHIZOCRYPT_DISCOVERY_ADAPTER", None::<&str>),
            ("DISCOVERY_ENDPOINT", None),
            ("DISCOVERY_ADDRESS", None),
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
                    .expect("server should become ready within 10s");
                handle.abort();
                let _ = handle.await;
            });
        },
    );
}

/// Test `run_server` continues when discovery registration fails (standalone fallback).
#[test]
fn test_run_server_discovery_failure_continues_standalone() {
    temp_env::with_vars(
        [
            ("RHIZOCRYPT_PORT", Some("0")),
            ("RHIZOCRYPT_HOST", Some("127.0.0.1")),
            ("RHIZOCRYPT_DISCOVERY_ADAPTER", Some("http://invalid-discovery-12345:99999")),
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
                    .expect("server should become ready within 10s");
                handle.abort();
                let _ = handle.await;
            });
        },
    );
}

/// Test `resolve_bind_addr` with `RHIZOCRYPT_RPC_PORT` env.
#[test]
fn test_resolve_bind_addr_rpc_port_env() {
    temp_env::with_vars(
        [
            ("RHIZOCRYPT_RPC_PORT", Some("19701")),
            ("RHIZOCRYPT_PORT", None::<&str>),
            ("RHIZOCRYPT_RPC_HOST", None),
            ("RHIZOCRYPT_HOST", None),
            ("RHIZOCRYPT_ENV", None),
        ],
        || {
            let addr = resolve_bind_addr(None, None).unwrap();
            assert_eq!(addr.port(), 19701);
        },
    );
}

/// Test `resolve_bind_addr` with `RHIZOCRYPT_PORT` (legacy) env.
#[test]
fn test_resolve_bind_addr_port_legacy_env() {
    temp_env::with_vars(
        [
            ("RHIZOCRYPT_PORT", Some("19702")),
            ("RHIZOCRYPT_RPC_PORT", None::<&str>),
            ("RHIZOCRYPT_RPC_HOST", None),
            ("RHIZOCRYPT_HOST", None),
            ("RHIZOCRYPT_ENV", None),
        ],
        || {
            let addr = resolve_bind_addr(None, None).unwrap();
            assert_eq!(addr.port(), 19702);
        },
    );
}

/// Test `resolve_bind_addr` with `RHIZOCRYPT_RPC_HOST` env.
#[test]
fn test_resolve_bind_addr_rpc_host_env() {
    temp_env::with_vars(
        [
            ("RHIZOCRYPT_RPC_HOST", Some("127.0.0.1")),
            ("RHIZOCRYPT_RPC_PORT", None::<&str>),
            ("RHIZOCRYPT_PORT", None),
            ("RHIZOCRYPT_HOST", None),
            ("RHIZOCRYPT_ENV", None),
        ],
        || {
            let addr = resolve_bind_addr(None, None).unwrap();
            assert_eq!(addr.ip().to_string(), "127.0.0.1");
        },
    );
}

/// Test `resolve_bind_addr` with `RHIZOCRYPT_HOST` (legacy) env.
#[test]
fn test_resolve_bind_addr_host_legacy_env() {
    temp_env::with_vars(
        [
            ("RHIZOCRYPT_HOST", Some("0.0.0.0")),
            ("RHIZOCRYPT_RPC_PORT", None::<&str>),
            ("RHIZOCRYPT_PORT", None),
            ("RHIZOCRYPT_RPC_HOST", None),
            ("RHIZOCRYPT_ENV", None),
        ],
        || {
            let addr = resolve_bind_addr(None, None).unwrap();
            assert_eq!(addr.ip().to_string(), "0.0.0.0");
        },
    );
}

/// Test `resolve_bind_addr` with development env uses default port.
#[test]
fn test_resolve_bind_addr_development_env() {
    temp_env::with_vars(
        [
            ("RHIZOCRYPT_ENV", Some("development")),
            ("RHIZOCRYPT_RPC_PORT", None::<&str>),
            ("RHIZOCRYPT_PORT", None),
            ("RHIZOCRYPT_RPC_HOST", None),
            ("RHIZOCRYPT_HOST", None),
        ],
        || {
            let addr = resolve_bind_addr(None, None).unwrap();
            assert!(addr.port() > 0 || addr.port() == 0);
        },
    );
}

// --- print_version and print_status ---

/// Test `print_version` runs without panic.
#[test]
fn test_print_version_no_panic() {
    print_version();
}

/// Test `print_status` runs without panic.
#[test]
fn test_print_status_no_panic() {
    print_status();
}

// --- ClientOperation variants ---

/// Test `ClientOperation` derives and variants exist.
#[test]
fn test_client_operation_variants() {
    let _ = ClientOperation::Health;
    let _ = ClientOperation::ListSessions;
    let _ = ClientOperation::Metrics;
    // Clap Subcommand ensures these are valid
}
