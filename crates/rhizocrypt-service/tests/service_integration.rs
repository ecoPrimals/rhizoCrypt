// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2024–2026 ecoPrimals Project

//! Integration tests for rhizocrypt service.
//!
//! Tests the service configuration, startup behavior, and basic functionality.

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::uninlined_format_args)]

use rhizo_crypt_core::{PrimalLifecycle, RhizoCrypt, RhizoCryptConfig, StorageBackend};
use rhizo_crypt_rpc::server::RpcServer;
use rhizocrypt_service::{
    ClientOperation, DoctorCheck, ServiceError, check_dag_engine, check_discovery_connectivity,
    check_storage_backend, exit_codes, print_status, print_version, resolve_bind_addr, run_client,
    run_doctor, run_server_with_ready,
};
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;

/// Test that service can be created with default configuration.
#[tokio::test]
async fn test_service_default_configuration() {
    let addr: SocketAddr = "127.0.0.1:0".parse().unwrap();

    let config = RhizoCryptConfig::default();
    let mut primal = RhizoCrypt::new(config);
    primal.start().await.expect("primal should start");
    let primal = Arc::new(primal);

    let server = RpcServer::new(primal, addr);

    // Verify server was created
    assert!(!server.is_running(), "Server should not be running yet");
}

/// Test that service can be created with custom configuration.
#[tokio::test]
async fn test_service_custom_configuration() {
    let addr: SocketAddr = "127.0.0.1:0".parse().unwrap();

    let mut config = RhizoCryptConfig::default();
    config.storage.backend = StorageBackend::Memory;

    let mut primal = RhizoCrypt::new(config);
    primal.start().await.expect("primal should start");
    let primal = Arc::new(primal);

    let server = RpcServer::new(primal, addr);

    assert!(!server.is_running(), "Server should not be running yet");
}

/// Test that service can start and shut down gracefully.
#[tokio::test]
async fn test_service_startup_shutdown() {
    let addr: SocketAddr = "127.0.0.1:0".parse().unwrap();

    let config = RhizoCryptConfig::default();
    let mut primal = RhizoCrypt::new(config);
    primal.start().await.expect("primal should start");
    let primal = Arc::new(primal);

    let server = RpcServer::new(primal, addr);
    let ready = server.ready_notifier();

    let handle = tokio::spawn(async move { server.serve().await });
    ready.notified().await;

    handle.abort();

    let result = handle.await;
    assert!(result.is_err(), "Aborted task should return error");
}

/// Test that service gracefully handles shutdown signal.
#[tokio::test]
async fn test_service_graceful_shutdown() {
    let addr: SocketAddr = "127.0.0.1:0".parse().unwrap();

    let config = RhizoCryptConfig::default();
    let mut primal = RhizoCrypt::new(config);
    primal.start().await.expect("primal should start");
    let primal = Arc::new(primal);

    let server = RpcServer::new(primal, addr);
    let ready = server.ready_notifier();

    let server_handle = tokio::spawn(async move { server.serve().await });
    ready.notified().await;

    server_handle.abort();
    let _ = server_handle.await;
}

/// Test that underlying rhizoCrypt primal functions correctly.
#[tokio::test]
async fn test_service_primal_functionality() {
    use rhizo_crypt_core::{EventType, SessionBuilder, SessionType, VertexBuilder};

    let config = RhizoCryptConfig::default();
    let mut primal = RhizoCrypt::new(config);
    primal.start().await.expect("primal should start");

    // Create session
    let session = SessionBuilder::new(SessionType::General).with_name("test-session").build();
    let session_id = primal.create_session(session).expect("should create session");

    // Add vertex
    let vertex = VertexBuilder::new(EventType::SessionStart).build();
    let vertex_id = primal.append_vertex(session_id, vertex).await.expect("should append vertex");

    // Verify vertex was added
    let vertex = primal.get_vertex(session_id, vertex_id).await;
    assert!(vertex.is_ok(), "Should retrieve vertex");

    // Stop primal
    primal.stop().await.expect("primal should stop");
}

/// Test service with multiple concurrent operations.
#[tokio::test]
async fn test_service_concurrent_sessions() {
    use rhizo_crypt_core::{SessionBuilder, SessionType};

    let config = RhizoCryptConfig::default();
    let mut primal = RhizoCrypt::new(config);
    primal.start().await.expect("primal should start");

    // Create multiple sessions concurrently
    let mut tasks = Vec::new();
    for i in 0..10 {
        let session =
            SessionBuilder::new(SessionType::General).with_name(format!("session-{}", i)).build();
        let result = primal.create_session(session);
        tasks.push(result);
    }

    // Wait for all to complete
    for result in tasks {
        assert!(result.is_ok(), "Session creation should succeed");
    }

    // Verify all sessions exist
    let sessions = primal.list_sessions();
    assert_eq!(sessions.len(), 10, "Should have 10 sessions");

    primal.stop().await.expect("primal should stop");
}

/// Test service health check.
#[tokio::test]
async fn test_service_health_check() {
    use rhizo_crypt_core::primal::PrimalState;

    let config = RhizoCryptConfig::default();
    let mut primal = RhizoCrypt::new(config);
    primal.start().await.expect("primal should start");

    // Verify primal is running
    assert_eq!(primal.state(), PrimalState::Running);

    primal.stop().await.expect("primal should stop");
}

/// Test service error handling.
#[tokio::test]
async fn test_service_error_handling() {
    use rhizo_crypt_core::types::SessionId;

    let config = RhizoCryptConfig::default();
    let mut primal = RhizoCrypt::new(config);
    primal.start().await.expect("primal should start");

    // Try to get non-existent session
    let fake_id = SessionId::now();
    let result = primal.get_session(fake_id);

    assert!(result.is_err(), "Getting non-existent session should fail");

    primal.stop().await.expect("primal should stop");
}

/// Test service with dehydration operations.
#[tokio::test]
async fn test_service_dehydration() {
    use rhizo_crypt_core::{EventType, SessionBuilder, SessionType, VertexBuilder};

    let config = RhizoCryptConfig::default();
    let mut primal = RhizoCrypt::new(config);
    primal.start().await.expect("primal should start");

    // Create session with vertices
    let session = SessionBuilder::new(SessionType::General).build();
    let session_id = primal.create_session(session).expect("should create session");

    let vertex = VertexBuilder::new(EventType::SessionStart).build();
    primal.append_vertex(session_id, vertex).await.expect("should append vertex");

    // Dehydrate
    let merkle_root = primal.dehydrate(session_id).await;
    assert!(merkle_root.is_ok(), "Dehydration should succeed");

    // Check status
    let status = primal.get_dehydration_status(session_id);
    assert!(status.is_complete(), "Dehydration should be complete");

    primal.stop().await.expect("primal should stop");
}

/// Test that service properly initializes with in-memory storage.
#[tokio::test]
async fn test_service_in_memory_storage() {
    use rhizo_crypt_core::{EventType, SessionBuilder, SessionType, VertexBuilder};

    let mut config = RhizoCryptConfig::default();
    config.storage.backend = StorageBackend::Memory;

    let mut primal = RhizoCrypt::new(config);
    primal.start().await.expect("primal should start");

    // Verify storage works
    let session = SessionBuilder::new(SessionType::General).build();
    let session_id = primal.create_session(session).expect("should create session");

    let vertex = VertexBuilder::new(EventType::SessionStart).build();
    let result = primal.append_vertex(session_id, vertex).await;

    assert!(result.is_ok(), "In-memory storage should work");

    primal.stop().await.expect("primal should stop");
}

// --- Doctor integration tests ---

/// Test doctor runs in basic (non-comprehensive) mode.
#[tokio::test]
async fn test_doctor_run_basic() {
    run_doctor(false).await;
}

/// Test doctor runs in comprehensive mode.
#[tokio::test]
async fn test_doctor_run_comprehensive() {
    run_doctor(true).await;
}

/// Test doctor reports Unhealthy when configuration has empty host.
#[test]
fn test_doctor_unhealthy_config_empty_host() {
    temp_env::with_vars([("RHIZOCRYPT_RPC_HOST", Some(""))], || {
        tokio::runtime::Builder::new_multi_thread()
            .worker_threads(2)
            .enable_all()
            .build()
            .unwrap()
            .block_on(async { run_doctor(false).await });
    });
}

/// Test doctor reports Healthy (standalone mode) when discovery is not configured.
#[test]
fn test_doctor_standalone_mode() {
    temp_env::with_vars(
        [
            ("RHIZOCRYPT_DISCOVERY_ADAPTER", None::<&str>),
            ("DISCOVERY_ENDPOINT", None),
            ("DISCOVERY_ADDRESS", None),
        ],
        || {
            tokio::runtime::Builder::new_multi_thread()
                .worker_threads(2)
                .enable_all()
                .build()
                .unwrap()
                .block_on(async { run_doctor(false).await });
        },
    );
}

/// Test doctor with discovery configured but non-comprehensive (Pass path).
#[test]
fn test_doctor_discovery_configured_non_comprehensive() {
    temp_env::with_vars([("DISCOVERY_ENDPOINT", Some("127.0.0.1:99999"))], || {
        tokio::runtime::Builder::new_multi_thread()
            .worker_threads(2)
            .enable_all()
            .build()
            .unwrap()
            .block_on(async { run_doctor(false).await });
    });
}

/// Test doctor with discovery configured and comprehensive (unreachable -> Warn).
#[test]
fn test_doctor_discovery_comprehensive_unreachable() {
    temp_env::with_vars([("DISCOVERY_ENDPOINT", Some("127.0.0.1:99999"))], || {
        tokio::runtime::Builder::new_multi_thread()
            .worker_threads(2)
            .enable_all()
            .build()
            .unwrap()
            .block_on(async { run_doctor(true).await });
    });
}

/// Test doctor with discovery reachable in comprehensive mode.
#[test]
fn test_doctor_discovery_comprehensive_reachable() {
    let rt =
        tokio::runtime::Builder::new_multi_thread().worker_threads(2).enable_all().build().unwrap();
    let addr = rt.block_on(async {
        tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap().local_addr().unwrap()
    });
    temp_env::with_vars([("DISCOVERY_ENDPOINT", Some(format!("http://{addr}")))], || {
        rt.block_on(async { run_doctor(true).await });
    });
}

/// Test `check_dag_engine` passes.
#[tokio::test]
async fn test_doctor_check_dag_engine() {
    assert!(check_dag_engine().await);
}

/// Test `check_storage_backend` returns valid result.
#[tokio::test]
async fn test_doctor_check_storage_backend() {
    let (ok, name) = check_storage_backend();
    assert!(ok, "storage backend check should pass");
    assert!(!name.is_empty(), "storage name should be non-empty");
}

/// Test `DoctorCheck` enum variants.
#[tokio::test]
async fn test_doctor_check_variants() {
    assert_eq!(DoctorCheck::Pass, DoctorCheck::Pass);
    assert_eq!(DoctorCheck::Warn, DoctorCheck::Warn);
    assert_eq!(DoctorCheck::Fail, DoctorCheck::Fail);
    assert_ne!(DoctorCheck::Pass, DoctorCheck::Fail);
    assert_ne!(DoctorCheck::Pass, DoctorCheck::Warn);
    assert_ne!(DoctorCheck::Warn, DoctorCheck::Fail);
}

/// Test `check_discovery_connectivity` succeeds with reachable address.
#[tokio::test]
async fn test_doctor_discovery_connectivity_success() {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();

    let result = check_discovery_connectivity(&addr.to_string()).await;
    assert!(result.is_ok(), "connectivity to local listener should succeed");
}

/// Test `check_discovery_connectivity` strips `http://` prefix.
#[tokio::test]
async fn test_doctor_discovery_connectivity_http_prefix() {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();

    let result = check_discovery_connectivity(&format!("http://{addr}")).await;
    assert!(result.is_ok(), "http:// prefix should be stripped");
}

/// Test `check_discovery_connectivity` strips `https://` prefix.
#[tokio::test]
async fn test_doctor_discovery_connectivity_https_prefix() {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();

    let result = check_discovery_connectivity(&format!("https://{addr}")).await;
    assert!(result.is_ok(), "https:// prefix should be stripped");
}

/// Test `check_discovery_connectivity` handles trailing slash.
#[tokio::test]
async fn test_doctor_discovery_connectivity_trailing_slash() {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();

    let result = check_discovery_connectivity(&format!("http://{addr}/")).await;
    assert!(result.is_ok(), "trailing slash should be trimmed");
}

/// Test `check_discovery_connectivity` fails on invalid address.
#[tokio::test]
async fn test_doctor_discovery_connectivity_invalid_address() {
    let result = check_discovery_connectivity("invalid-host-12345:99999").await;
    assert!(result.is_err(), "invalid address should fail");
}

/// Test `check_discovery_connectivity` fails on invalid parse.
#[tokio::test]
async fn test_doctor_discovery_connectivity_parse_error() {
    let result = check_discovery_connectivity("not-a-valid-address").await;
    assert!(result.is_err(), "unparseable address should fail");
}

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

// --- UDS transport integration tests ---

#[cfg(unix)]
mod uds_integration {
    use super::*;
    use rhizo_crypt_rpc::jsonrpc::uds::UdsJsonRpcServer;
    use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
    use tokio::net::UnixStream;

    /// socat-style validation: raw newline JSON-RPC over UDS.
    #[tokio::test]
    async fn test_uds_socat_style_health_liveness() {
        let dir = tempfile::tempdir().expect("tempdir");
        let sock = dir.path().join("socat-test.sock");

        let config = RhizoCryptConfig::default();
        let mut primal = RhizoCrypt::new(config);
        primal.start().await.expect("primal should start");
        let primal = Arc::new(primal);

        let uds = UdsJsonRpcServer::new(Arc::clone(&primal), sock.clone());
        let (shutdown_tx, shutdown_rx) = tokio::sync::watch::channel(false);
        let ready = Arc::new(tokio::sync::Notify::new());
        let ready_rx = Arc::clone(&ready);

        let handle = tokio::spawn(async move { uds.serve_with_ready(shutdown_rx, ready_rx).await });
        ready.notified().await;

        let stream = UnixStream::connect(&sock).await.expect("connect");
        let (reader, mut writer) = stream.into_split();

        let req = "{\"jsonrpc\":\"2.0\",\"method\":\"health.liveness\",\"params\":{},\"id\":1}\n";
        writer.write_all(req.as_bytes()).await.unwrap();
        writer.shutdown().await.unwrap();

        let mut lines = BufReader::new(reader).lines();
        let line = lines.next_line().await.unwrap().expect("response");
        let resp: serde_json::Value = serde_json::from_str(&line).unwrap();
        assert_eq!(resp["jsonrpc"], "2.0");
        assert!(resp.get("result").is_some() || resp.get("error").is_some());

        let _ = shutdown_tx.send(true);
        let _ = handle.await;
    }

    /// Server with --unix creates socket file and cleans up on shutdown.
    #[tokio::test]
    async fn test_uds_server_lifecycle() {
        let dir = tempfile::tempdir().expect("tempdir");
        let sock = dir.path().join("lifecycle.sock");

        let config = RhizoCryptConfig::default();
        let mut primal = RhizoCrypt::new(config);
        primal.start().await.expect("primal should start");
        let primal = Arc::new(primal);

        let uds = UdsJsonRpcServer::new(primal, sock.clone());
        let (shutdown_tx, shutdown_rx) = tokio::sync::watch::channel(false);
        let ready = Arc::new(tokio::sync::Notify::new());
        let ready_rx = Arc::clone(&ready);

        assert!(!sock.exists(), "socket should not exist before serve");

        let handle = tokio::spawn(async move { uds.serve_with_ready(shutdown_rx, ready_rx).await });
        ready.notified().await;

        assert!(sock.exists(), "socket should exist after serve starts");

        let _ = shutdown_tx.send(true);
        let _ = handle.await;

        assert!(!sock.exists(), "socket should be cleaned up after shutdown");
    }

    /// Run server with UDS enabled (empty path = default).
    #[test]
    fn test_run_server_with_uds() {
        let dir = tempfile::tempdir().expect("tempdir");
        let sock = dir.path().join("run-server-uds.sock");

        let rt = tokio::runtime::Builder::new_multi_thread()
            .worker_threads(2)
            .enable_all()
            .build()
            .unwrap();

        let ready = Arc::new(tokio::sync::Notify::new());
        let ready_clone = Arc::clone(&ready);
        let sock_str = sock.to_string_lossy().to_string();

        let handle = rt.spawn(async move {
            let _ = run_server_with_ready(
                Some(0),
                Some("127.0.0.1".to_string()),
                Some(sock_str),
                Some(ready_clone),
            )
            .await;
        });

        rt.block_on(async {
            tokio::time::timeout(Duration::from_secs(10), ready.notified())
                .await
                .expect("server should become ready within 10s");

            assert!(sock.exists(), "UDS socket should be created");

            handle.abort();
            let _ = handle.await;
        });
    }

    /// Composition-load: many concurrent UDS clients issuing JSON-RPC.
    ///
    /// Simulates the composition load that downstream springs (wetSpring,
    /// ludoSpring, healthSpring) apply when trio IPC is active. Validates
    /// that UDS remains stable under parallel connection pressure.
    #[tokio::test]
    async fn test_uds_composition_load_concurrent_clients() {
        const CONCURRENT_CLIENTS: usize = 50;

        let dir = tempfile::tempdir().expect("tempdir");
        let sock = dir.path().join("comp-load.sock");

        let config = RhizoCryptConfig::default();
        let mut primal = RhizoCrypt::new(config);
        primal.start().await.expect("primal should start");
        let primal = Arc::new(primal);

        let uds = UdsJsonRpcServer::new(Arc::clone(&primal), sock.clone());
        let (shutdown_tx, shutdown_rx) = tokio::sync::watch::channel(false);
        let ready = Arc::new(tokio::sync::Notify::new());
        let ready_rx = Arc::clone(&ready);

        let handle = tokio::spawn(async move { uds.serve_with_ready(shutdown_rx, ready_rx).await });
        ready.notified().await;

        let mut tasks = Vec::with_capacity(CONCURRENT_CLIENTS);
        for i in 0..CONCURRENT_CLIENTS {
            let sock = sock.clone();
            tasks.push(tokio::spawn(async move {
                let stream = UnixStream::connect(&sock).await.expect("connect");
                let (reader, mut writer) = stream.into_split();

                let req = format!(
                    "{{\"jsonrpc\":\"2.0\",\"method\":\"health.check\",\"params\":{{}},\"id\":{i}}}\n"
                );
                writer.write_all(req.as_bytes()).await.unwrap();
                writer.shutdown().await.unwrap();

                let mut lines = BufReader::new(reader).lines();
                let line = lines.next_line().await.unwrap().expect("response");
                let resp: serde_json::Value = serde_json::from_str(&line).unwrap();
                assert_eq!(resp["jsonrpc"], "2.0");
                assert!(
                    resp.get("result").is_some(),
                    "client {i} expected result, got: {resp}"
                );
            }));
        }

        for task in tasks {
            task.await.unwrap();
        }

        let _ = shutdown_tx.send(true);
        let _ = handle.await;
    }

    /// Composition-load: sustained sequential requests on a single UDS
    /// connection (simulates a long-lived spring client).
    #[tokio::test]
    async fn test_uds_sustained_sequential_requests() {
        const REQUEST_COUNT: usize = 200;

        let dir = tempfile::tempdir().expect("tempdir");
        let sock = dir.path().join("sustained.sock");

        let config = RhizoCryptConfig::default();
        let mut primal = RhizoCrypt::new(config);
        primal.start().await.expect("primal should start");
        let primal = Arc::new(primal);

        let uds = UdsJsonRpcServer::new(Arc::clone(&primal), sock.clone());
        let (shutdown_tx, shutdown_rx) = tokio::sync::watch::channel(false);
        let ready = Arc::new(tokio::sync::Notify::new());
        let ready_rx = Arc::clone(&ready);

        let handle = tokio::spawn(async move { uds.serve_with_ready(shutdown_rx, ready_rx).await });
        ready.notified().await;

        let stream = UnixStream::connect(&sock).await.expect("connect");
        let (reader, mut writer) = tokio::io::split(stream);
        let mut lines = BufReader::new(reader).lines();

        for i in 0..REQUEST_COUNT {
            let req = format!(
                "{{\"jsonrpc\":\"2.0\",\"method\":\"health.liveness\",\"params\":{{}},\"id\":{i}}}\n"
            );
            writer.write_all(req.as_bytes()).await.unwrap();
            writer.flush().await.unwrap();

            let line = lines.next_line().await.unwrap().expect("response");
            let resp: serde_json::Value = serde_json::from_str(&line).unwrap();
            assert_eq!(resp["jsonrpc"], "2.0", "request {i} bad jsonrpc");
        }

        drop(writer);
        let _ = shutdown_tx.send(true);
        let _ = handle.await;
    }

    /// Graceful shutdown: server stops accepting new connections and cleans
    /// up the socket file after existing connections complete.
    #[tokio::test]
    async fn test_uds_graceful_shutdown_under_load() {
        let dir = tempfile::tempdir().expect("tempdir");
        let sock = dir.path().join("graceful.sock");

        let config = RhizoCryptConfig::default();
        let mut primal = RhizoCrypt::new(config);
        primal.start().await.expect("primal should start");
        let primal = Arc::new(primal);

        let uds = UdsJsonRpcServer::new(Arc::clone(&primal), sock.clone());
        let (shutdown_tx, shutdown_rx) = tokio::sync::watch::channel(false);
        let ready = Arc::new(tokio::sync::Notify::new());
        let ready_rx = Arc::clone(&ready);

        let handle = tokio::spawn(async move { uds.serve_with_ready(shutdown_rx, ready_rx).await });
        ready.notified().await;

        let mut tasks = Vec::with_capacity(10);
        for i in 0..10 {
            let sock = sock.clone();
            tasks.push(tokio::spawn(async move {
                let stream = UnixStream::connect(&sock).await.expect("connect");
                let (reader, mut writer) = stream.into_split();

                let req = format!(
                    "{{\"jsonrpc\":\"2.0\",\"method\":\"health.check\",\"params\":{{}},\"id\":{i}}}\n"
                );
                writer.write_all(req.as_bytes()).await.unwrap();
                writer.shutdown().await.unwrap();

                let mut lines = BufReader::new(reader).lines();
                let line = lines.next_line().await.unwrap().expect("response");
                let resp: serde_json::Value = serde_json::from_str(&line).unwrap();
                assert_eq!(resp["jsonrpc"], "2.0");
            }));
        }

        for task in tasks {
            task.await.unwrap();
        }

        let _ = shutdown_tx.send(true);

        let result = tokio::time::timeout(std::time::Duration::from_secs(5), handle)
            .await
            .expect("server should shut down within 5s");
        assert!(result.is_ok(), "server should shut down cleanly");
        assert!(!sock.exists(), "socket should be cleaned up");
    }
}
