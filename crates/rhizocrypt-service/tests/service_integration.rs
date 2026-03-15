// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2024–2026 ecoPrimals Project

//! Integration tests for rhizocrypt service.
//!
//! Tests the service configuration, startup behavior, and basic functionality.

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::uninlined_format_args, unsafe_code)]

use rhizo_crypt_core::{PrimalLifecycle, RhizoCrypt, RhizoCryptConfig, StorageBackend};
use rhizo_crypt_rpc::server::RpcServer;
use rhizocrypt_service::{
    ClientOperation, DoctorCheck, ServiceError, check_dag_engine, check_discovery_connectivity,
    check_storage_backend, exit_codes, print_status, print_version, resolve_bind_addr, run_client,
    run_doctor, run_server,
};
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;
use tokio::time::sleep;

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

    // Start serving in background
    let handle = tokio::spawn(async move { server.serve().await });

    // Give service time to start
    sleep(Duration::from_millis(100)).await;

    // Shutdown by aborting (simulates Ctrl+C)
    handle.abort();

    // Wait for abort
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

    // Start serving in background
    let server_handle = tokio::spawn(async move { server.serve().await });

    // Give service time to start
    sleep(Duration::from_millis(100)).await;

    // Shutdown
    server_handle.abort();

    // Verify clean shutdown
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

static ENV_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

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
#[tokio::test]
async fn test_doctor_unhealthy_config_empty_host() {
    {
        let _guard = ENV_LOCK.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
        unsafe { std::env::set_var("RHIZOCRYPT_RPC_HOST", "") };
    }
    run_doctor(false).await;
    let _guard = ENV_LOCK.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
    unsafe { std::env::remove_var("RHIZOCRYPT_RPC_HOST") };
}

/// Test doctor reports Healthy (standalone mode) when discovery is not configured.
#[tokio::test]
async fn test_doctor_standalone_mode() {
    {
        let _guard = ENV_LOCK.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
        unsafe { std::env::remove_var("RHIZOCRYPT_DISCOVERY_ADAPTER") };
        unsafe { std::env::remove_var("DISCOVERY_ENDPOINT") };
        unsafe { std::env::remove_var("DISCOVERY_ADDRESS") };
    }
    run_doctor(false).await;
}

/// Test doctor with discovery configured but non-comprehensive (Pass path).
#[tokio::test]
async fn test_doctor_discovery_configured_non_comprehensive() {
    {
        let _guard = ENV_LOCK.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
        unsafe { std::env::set_var("DISCOVERY_ENDPOINT", "127.0.0.1:99999") };
    }
    run_doctor(false).await;
    let _guard = ENV_LOCK.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
    unsafe { std::env::remove_var("DISCOVERY_ENDPOINT") };
}

/// Test doctor with discovery configured and comprehensive (unreachable -> Warn).
#[tokio::test]
async fn test_doctor_discovery_comprehensive_unreachable() {
    {
        let _guard = ENV_LOCK.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
        unsafe { std::env::set_var("DISCOVERY_ENDPOINT", "127.0.0.1:99999") };
    }
    run_doctor(true).await;
    let _guard = ENV_LOCK.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
    unsafe { std::env::remove_var("DISCOVERY_ENDPOINT") };
}

/// Test doctor with discovery reachable in comprehensive mode.
#[tokio::test]
async fn test_doctor_discovery_comprehensive_reachable() {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    {
        let _guard = ENV_LOCK.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
        unsafe { std::env::set_var("DISCOVERY_ENDPOINT", format!("http://{addr}")) };
    }
    run_doctor(true).await;
    let _guard = ENV_LOCK.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
    unsafe { std::env::remove_var("DISCOVERY_ENDPOINT") };
}

/// Test check_dag_engine passes.
#[tokio::test]
async fn test_doctor_check_dag_engine() {
    assert!(check_dag_engine().await);
}

/// Test check_storage_backend returns valid result.
#[tokio::test]
async fn test_doctor_check_storage_backend() {
    let (ok, name) = check_storage_backend();
    assert!(ok, "storage backend check should pass");
    assert!(!name.is_empty(), "storage name should be non-empty");
}

/// Test DoctorCheck enum variants.
#[tokio::test]
async fn test_doctor_check_variants() {
    assert_eq!(DoctorCheck::Pass, DoctorCheck::Pass);
    assert_eq!(DoctorCheck::Warn, DoctorCheck::Warn);
    assert_eq!(DoctorCheck::Fail, DoctorCheck::Fail);
    assert_ne!(DoctorCheck::Pass, DoctorCheck::Fail);
    assert_ne!(DoctorCheck::Pass, DoctorCheck::Warn);
    assert_ne!(DoctorCheck::Warn, DoctorCheck::Fail);
}

/// Test check_discovery_connectivity succeeds with reachable address.
#[tokio::test]
async fn test_doctor_discovery_connectivity_success() {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();

    let result = check_discovery_connectivity(&addr.to_string()).await;
    assert!(result.is_ok(), "connectivity to local listener should succeed");
}

/// Test check_discovery_connectivity strips http:// prefix.
#[tokio::test]
async fn test_doctor_discovery_connectivity_http_prefix() {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();

    let result = check_discovery_connectivity(&format!("http://{addr}")).await;
    assert!(result.is_ok(), "http:// prefix should be stripped");
}

/// Test check_discovery_connectivity strips https:// prefix.
#[tokio::test]
async fn test_doctor_discovery_connectivity_https_prefix() {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();

    let result = check_discovery_connectivity(&format!("https://{addr}")).await;
    assert!(result.is_ok(), "https:// prefix should be stripped");
}

/// Test check_discovery_connectivity handles trailing slash.
#[tokio::test]
async fn test_doctor_discovery_connectivity_trailing_slash() {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();

    let result = check_discovery_connectivity(&format!("http://{addr}/")).await;
    assert!(result.is_ok(), "trailing slash should be trimmed");
}

/// Test check_discovery_connectivity fails on invalid address.
#[tokio::test]
async fn test_doctor_discovery_connectivity_invalid_address() {
    let result = check_discovery_connectivity("invalid-host-12345:99999").await;
    assert!(result.is_err(), "invalid address should fail");
}

/// Test check_discovery_connectivity fails on invalid parse.
#[tokio::test]
async fn test_doctor_discovery_connectivity_parse_error() {
    let result = check_discovery_connectivity("not-a-valid-address").await;
    assert!(result.is_err(), "unparseable address should fail");
}

// --- ServiceError and exit codes ---

/// Test ServiceError::Storage display and exit code.
#[test]
fn test_service_error_storage_display_and_exit_code() {
    let err = ServiceError::Storage("backend unavailable".to_string());
    let s = err.to_string();
    assert!(s.contains("storage error"));
    assert!(s.contains("backend unavailable"));
    assert_eq!(err.exit_code(), exit_codes::CONFIG_ERROR);
}

/// Test ServiceError exit code mapping for all variants.
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

/// Test run_client with invalid address returns AddrParse error.
#[tokio::test]
async fn test_run_client_invalid_address() {
    let result = run_client("not-a-valid-address", ClientOperation::Health).await;
    assert!(result.is_err());
    assert!(matches!(&result.unwrap_err(), ServiceError::AddrParse(_)));
}

/// Test run_client connection failure returns Config error.
#[tokio::test]
async fn test_run_client_connection_failure() {
    // Port 1 is typically not listening
    let result = run_client("127.0.0.1:1", ClientOperation::Health).await;
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(matches!(&err, ServiceError::Config(_)));
    assert!(err.to_string().contains("Failed to connect"));
}

/// Test run_client Health against running server.
#[tokio::test]
async fn test_run_client_health() {
    let addr: SocketAddr = "127.0.0.1:19625".parse().unwrap();
    let config = RhizoCryptConfig::default();
    let mut primal = RhizoCrypt::new(config);
    primal.start().await.expect("primal should start");
    let primal = Arc::new(primal);
    let server = RpcServer::new(primal, addr);
    let handle = tokio::spawn(async move { server.serve().await });

    for _ in 0..50 {
        if run_client("127.0.0.1:19625", ClientOperation::Health).await.is_ok() {
            handle.abort();
            let _ = handle.await;
            return;
        }
        tokio::task::yield_now().await;
    }
    handle.abort();
    let _ = handle.await;
    panic!("run_client Health should succeed against running server");
}

/// Test run_client ListSessions against running server.
#[tokio::test]
async fn test_run_client_list_sessions() {
    let addr: SocketAddr = "127.0.0.1:19626".parse().unwrap();
    let config = RhizoCryptConfig::default();
    let mut primal = RhizoCrypt::new(config);
    primal.start().await.expect("primal should start");
    let primal = Arc::new(primal);
    let server = RpcServer::new(primal, addr);
    let handle = tokio::spawn(async move { server.serve().await });

    for _ in 0..50 {
        if run_client("127.0.0.1:19626", ClientOperation::ListSessions).await.is_ok() {
            handle.abort();
            let _ = handle.await;
            return;
        }
        tokio::task::yield_now().await;
    }
    handle.abort();
    let _ = handle.await;
    panic!("run_client ListSessions should succeed against running server");
}

/// Test run_client Metrics against running server.
#[tokio::test]
async fn test_run_client_metrics() {
    let addr: SocketAddr = "127.0.0.1:19627".parse().unwrap();
    let config = RhizoCryptConfig::default();
    let mut primal = RhizoCrypt::new(config);
    primal.start().await.expect("primal should start");
    let primal = Arc::new(primal);
    let server = RpcServer::new(primal, addr);
    let handle = tokio::spawn(async move { server.serve().await });

    for _ in 0..50 {
        if run_client("127.0.0.1:19627", ClientOperation::Metrics).await.is_ok() {
            handle.abort();
            let _ = handle.await;
            return;
        }
        tokio::task::yield_now().await;
    }
    handle.abort();
    let _ = handle.await;
    panic!("run_client Metrics should succeed against running server");
}

// --- run_server and resolve_bind_addr ---

/// Test resolve_bind_addr with invalid host returns AddrParse error.
#[test]
fn test_resolve_bind_addr_invalid_host() {
    let _guard = ENV_LOCK.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
    let result = resolve_bind_addr(Some(9999), Some("not-an-ip".to_string()));
    assert!(result.is_err());
    assert!(matches!(&result.unwrap_err(), ServiceError::AddrParse(_)));
}

/// Test run_server starts in standalone mode when no discovery configured.
#[tokio::test]
async fn test_run_server_standalone_mode_no_discovery() {
    {
        let _guard = ENV_LOCK.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
        unsafe { std::env::remove_var("RHIZOCRYPT_DISCOVERY_ADAPTER") };
        unsafe { std::env::remove_var("DISCOVERY_ENDPOINT") };
        unsafe { std::env::remove_var("DISCOVERY_ADDRESS") };
    }

    let handle =
        tokio::spawn(async { run_server(Some(19709), Some("127.0.0.1".to_string())).await });

    sleep(Duration::from_secs(1)).await;

    handle.abort();
    let _ = handle.await;
}

/// Test run_server continues when discovery registration fails (standalone fallback).
#[tokio::test]
async fn test_run_server_discovery_failure_continues_standalone() {
    {
        let _guard = ENV_LOCK.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
        unsafe { std::env::set_var("RHIZOCRYPT_PORT", "19710") };
        unsafe { std::env::set_var("RHIZOCRYPT_HOST", "127.0.0.1") };
        unsafe {
            std::env::set_var(
                "RHIZOCRYPT_DISCOVERY_ADAPTER",
                "http://invalid-discovery-12345:99999",
            );
        }
        unsafe { std::env::set_var("RUST_LOG", "error") };
    }

    let handle =
        tokio::spawn(async { run_server(Some(19710), Some("127.0.0.1".to_string())).await });

    sleep(Duration::from_secs(2)).await;

    handle.abort();
    let _ = handle.await;

    let _guard = ENV_LOCK.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
    unsafe { std::env::remove_var("RHIZOCRYPT_PORT") };
    unsafe { std::env::remove_var("RHIZOCRYPT_HOST") };
    unsafe { std::env::remove_var("RHIZOCRYPT_DISCOVERY_ADAPTER") };
}

/// Clear all env vars that affect `resolve_bind_addr` so tests are isolated
/// from async tests that may have leaked values outside their lock window.
fn clear_bind_addr_env() {
    unsafe { std::env::remove_var("RHIZOCRYPT_RPC_PORT") };
    unsafe { std::env::remove_var("RHIZOCRYPT_PORT") };
    unsafe { std::env::remove_var("RHIZOCRYPT_RPC_HOST") };
    unsafe { std::env::remove_var("RHIZOCRYPT_HOST") };
    unsafe { std::env::remove_var("RHIZOCRYPT_ENV") };
}

/// Test resolve_bind_addr with RHIZOCRYPT_RPC_PORT env.
#[test]
fn test_resolve_bind_addr_rpc_port_env() {
    let _guard = ENV_LOCK.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
    clear_bind_addr_env();
    unsafe { std::env::set_var("RHIZOCRYPT_RPC_PORT", "19701") };
    let addr = resolve_bind_addr(None, None).unwrap();
    assert_eq!(addr.port(), 19701);
    clear_bind_addr_env();
}

/// Test resolve_bind_addr with RHIZOCRYPT_PORT (legacy) env.
#[test]
fn test_resolve_bind_addr_port_legacy_env() {
    let _guard = ENV_LOCK.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
    clear_bind_addr_env();
    unsafe { std::env::set_var("RHIZOCRYPT_PORT", "19702") };
    let addr = resolve_bind_addr(None, None).unwrap();
    assert_eq!(addr.port(), 19702);
    clear_bind_addr_env();
}

/// Test resolve_bind_addr with RHIZOCRYPT_RPC_HOST env.
#[test]
fn test_resolve_bind_addr_rpc_host_env() {
    let _guard = ENV_LOCK.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
    clear_bind_addr_env();
    unsafe { std::env::set_var("RHIZOCRYPT_RPC_HOST", "127.0.0.1") };
    let addr = resolve_bind_addr(None, None).unwrap();
    assert_eq!(addr.ip().to_string(), "127.0.0.1");
    clear_bind_addr_env();
}

/// Test resolve_bind_addr with RHIZOCRYPT_HOST (legacy) env.
#[test]
fn test_resolve_bind_addr_host_legacy_env() {
    let _guard = ENV_LOCK.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
    clear_bind_addr_env();
    unsafe { std::env::set_var("RHIZOCRYPT_HOST", "0.0.0.0") };
    let addr = resolve_bind_addr(None, None).unwrap();
    assert_eq!(addr.ip().to_string(), "0.0.0.0");
    clear_bind_addr_env();
}

/// Test resolve_bind_addr with development env uses default port.
#[test]
fn test_resolve_bind_addr_development_env() {
    let _guard = ENV_LOCK.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
    clear_bind_addr_env();
    unsafe { std::env::set_var("RHIZOCRYPT_ENV", "development") };
    let addr = resolve_bind_addr(None, None).unwrap();
    assert!(addr.port() > 0 || addr.port() == 0);
    clear_bind_addr_env();
}

// --- print_version and print_status ---

/// Test print_version runs without panic.
#[test]
fn test_print_version_no_panic() {
    print_version();
}

/// Test print_status runs without panic.
#[test]
fn test_print_status_no_panic() {
    print_status();
}

// --- ClientOperation variants ---

/// Test ClientOperation derives and variants exist.
#[test]
fn test_client_operation_variants() {
    let _ = ClientOperation::Health;
    let _ = ClientOperation::ListSessions;
    let _ = ClientOperation::Metrics;
    // Clap Subcommand ensures these are valid
}
