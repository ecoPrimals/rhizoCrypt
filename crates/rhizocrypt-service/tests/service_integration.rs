//! Integration tests for rhizocrypt-service binary.
//!
//! Tests the service configuration, startup behavior, and basic functionality.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use rhizo_crypt_core::{PrimalLifecycle, RhizoCrypt, RhizoCryptConfig, StorageBackend};
use rhizo_crypt_rpc::server::RpcServer;
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
    let session_id = primal.create_session(session).await.expect("should create session");

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
        let task = primal.create_session(session);
        tasks.push(task);
    }

    // Wait for all to complete
    for task in tasks {
        assert!(task.await.is_ok(), "Session creation should succeed");
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
    let session_id = primal.create_session(session).await.expect("should create session");

    let vertex = VertexBuilder::new(EventType::SessionStart).build();
    primal.append_vertex(session_id, vertex).await.expect("should append vertex");

    // Dehydrate
    let merkle_root = primal.dehydrate(session_id).await;
    assert!(merkle_root.is_ok(), "Dehydration should succeed");

    // Check status
    let status = primal.get_dehydration_status(session_id).await;
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
    let session_id = primal.create_session(session).await.expect("should create session");

    let vertex = VertexBuilder::new(EventType::SessionStart).build();
    let result = primal.append_vertex(session_id, vertex).await;

    assert!(result.is_ok(), "In-memory storage should work");

    primal.stop().await.expect("primal should stop");
}
