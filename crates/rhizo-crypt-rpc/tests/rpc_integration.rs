// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2024–2026 ecoPrimals Project

//! Integration tests for the RPC client/server.
//!
//! Comprehensive tests covering all 24 RPC methods.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use rhizo_crypt_core::{EventType, PrimalLifecycle, RhizoCrypt, RhizoCryptConfig, SessionType};
use rhizo_crypt_rpc::{
    AppendEventRequest, CreateSessionRequest, QueryRequest, RpcClient, RpcServer,
};
use std::sync::Arc;

/// Helper to create a test server and client pair.
///
/// Uses retry logic instead of sleep to wait for server readiness.
async fn setup_server_client(
    port: u16,
) -> (Arc<RhizoCrypt>, RpcClient, tokio::task::JoinHandle<std::result::Result<(), std::io::Error>>)
{
    let mut config = RhizoCryptConfig::default();
    config.rpc.port = port;
    let addr = config.rpc.parse_addr().expect("test config should have valid addr");

    let mut primal = RhizoCrypt::new(config);
    primal.start().await.expect("primal should start");

    let primal = Arc::new(primal);
    let server = RpcServer::new(Arc::clone(&primal), addr);

    let server_handle = tokio::spawn(async move { server.serve().await });

    // Retry connection until server is ready (no sleep, pure async retry)
    let client = async {
        for attempt in 0..50 {
            if let Ok(client) = RpcClient::connect(addr).await {
                return Ok(client);
            }
            if attempt < 49 {
                tokio::task::yield_now().await;
            }
        }
        Err("Failed to connect to server after retries")
    }
    .await
    .expect("client should connect");

    (primal, client, server_handle)
}

/// Test basic RPC server startup and client connection.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_rpc_server_client_connection() {
    // Create and start primal
    let mut config = RhizoCryptConfig::default();
    config.rpc.port = 19501; // Use high port for testing
    let addr = config.rpc.parse_addr().expect("test config should have valid addr");

    let mut primal = RhizoCrypt::new(config.clone());
    primal.start().await.expect("primal should start");

    let primal = Arc::new(primal);
    let server = RpcServer::new(Arc::clone(&primal), addr);

    // Start server in background
    let server_handle = tokio::spawn(async move { server.serve().await });

    // Retry connection until server is ready
    let client = async {
        for attempt in 0..50 {
            if let Ok(client) = RpcClient::connect(addr).await {
                return Ok(client);
            }
            if attempt < 49 {
                tokio::task::yield_now().await;
            }
        }
        Err("Failed to connect")
    }
    .await
    .expect("client should connect");

    // Check health
    let health = client.health().await.expect("should get health");
    assert!(health.healthy);

    // Create a session
    let request = CreateSessionRequest {
        session_type: SessionType::General,
        description: Some("test session".to_string()),
        parent_session: None,
        max_vertices: Some(1000),
        ttl_seconds: None,
    };
    let session_id = client.create_session(request).await.expect("should create session");

    // Get session
    let info = client.get_session(session_id).await.expect("should get session");
    assert_eq!(info.description, Some("test session".to_string()));

    // List sessions
    let sessions = client.list_sessions().await.expect("should list sessions");
    assert_eq!(sessions.len(), 1);

    // Cleanup
    server_handle.abort();
}

/// Test session creation and vertex append via RPC.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_rpc_vertex_operations() {
    // Setup with unique port
    let mut config = RhizoCryptConfig::default();
    config.rpc.port = 19602;
    let addr = config.rpc.parse_addr().expect("test config should have valid addr");

    let mut primal = RhizoCrypt::new(config);
    primal.start().await.expect("primal should start");

    let primal = Arc::new(primal);
    let primal_clone = Arc::clone(&primal);
    let server = RpcServer::new(primal_clone, addr);

    let server_handle = tokio::spawn(async move { server.serve().await });

    // Retry connection
    let client = async {
        for attempt in 0..50 {
            if let Ok(client) = RpcClient::connect(addr).await {
                return Ok(client);
            }
            if attempt < 49 {
                tokio::task::yield_now().await;
            }
        }
        Err("Failed to connect")
    }
    .await
    .expect("client should connect");

    // Create session
    let request = CreateSessionRequest {
        session_type: SessionType::General,
        description: None,
        parent_session: None,
        max_vertices: None,
        ttl_seconds: None,
    };
    let session_id = client.create_session(request).await.expect("should create session");

    // Append vertex
    let append_request = AppendEventRequest {
        session_id,
        event_type: EventType::SessionStart,
        agent: None,
        parents: vec![],
        metadata: vec![],
        payload_ref: None,
    };
    let vertex_id = client.append_event(append_request).await.expect("should append event");

    // Get frontier - should contain our vertex
    let frontier = client.get_frontier(session_id).await.expect("should get frontier");
    assert_eq!(frontier.len(), 1, "frontier should have one vertex");
    assert!(frontier.contains(&vertex_id), "frontier should contain our vertex");

    // Get the full vertex via RPC
    let vertex = client.get_vertex(session_id, vertex_id).await.expect("should get vertex");
    assert_eq!(vertex.event_type, EventType::SessionStart);

    // Cleanup
    server_handle.abort();
}

/// Test RPC health and metrics endpoints.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_rpc_health_metrics() {
    let mut config = RhizoCryptConfig::default();
    config.rpc.port = 19503;
    let addr = config.rpc.parse_addr().expect("test config should have valid addr");

    let mut primal = RhizoCrypt::new(config);
    primal.start().await.expect("primal should start");

    let primal = Arc::new(primal);
    let server = RpcServer::new(Arc::clone(&primal), addr);

    let server_handle = tokio::spawn(async move { server.serve().await });

    // Retry connection
    let client = async {
        for attempt in 0..50 {
            if let Ok(client) = RpcClient::connect(addr).await {
                return Ok(client);
            }
            if attempt < 49 {
                tokio::task::yield_now().await;
            }
        }
        Err("Failed to connect")
    }
    .await
    .expect("client should connect");

    // Check health
    let health = client.health().await.expect("should get health");
    assert!(health.healthy);
    // uptime_seconds is u64, always >= 0
    assert!(health.uptime_seconds < 3600, "uptime should be reasonable");

    // Get metrics
    let metrics = client.metrics().await.expect("should get metrics");
    assert_eq!(metrics.sessions_created, 0);

    // Create a session and check metrics update
    let request = CreateSessionRequest {
        session_type: SessionType::General,
        description: None,
        parent_session: None,
        max_vertices: None,
        ttl_seconds: None,
    };
    let _session_id = client.create_session(request).await.expect("should create session");

    let metrics = client.metrics().await.expect("should get metrics");
    assert_eq!(metrics.sessions_created, 1);

    // Cleanup
    server_handle.abort();
}

/// Test session discard via RPC.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_rpc_discard_session() {
    let (_primal, client, server_handle) = setup_server_client(19604).await;

    // Create session
    let request = CreateSessionRequest {
        session_type: SessionType::General,
        description: Some("to be discarded".to_string()),
        parent_session: None,
        max_vertices: None,
        ttl_seconds: None,
    };
    let session_id = client.create_session(request).await.expect("should create session");

    // Verify it exists
    let sessions = client.list_sessions().await.expect("should list sessions");
    assert_eq!(sessions.len(), 1);

    // Discard it
    client.discard_session(session_id).await.expect("should discard session");

    // Verify it's gone
    let sessions = client.list_sessions().await.expect("should list sessions");
    assert!(sessions.is_empty());

    server_handle.abort();
}

/// Test batch append via RPC.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_rpc_batch_append() {
    let (_primal, client, server_handle) = setup_server_client(19605).await;

    // Create session
    let request = CreateSessionRequest {
        session_type: SessionType::General,
        description: None,
        parent_session: None,
        max_vertices: None,
        ttl_seconds: None,
    };
    let session_id = client.create_session(request).await.expect("should create session");

    // Append batch of events
    let requests: Vec<AppendEventRequest> = (0..5)
        .map(|i| AppendEventRequest {
            session_id,
            event_type: EventType::DataCreate {
                schema: Some(format!("schema-{i}")),
            },
            agent: None,
            parents: vec![],
            metadata: vec![("index".to_string(), i.to_string())],
            payload_ref: None,
        })
        .collect();

    let vertex_ids = client.append_batch(requests).await.expect("should append batch");
    assert_eq!(vertex_ids.len(), 5);

    // Verify session has correct vertex count
    let info = client.get_session(session_id).await.expect("should get session");
    assert_eq!(info.vertex_count, 5);

    server_handle.abort();
}

/// Test query vertices with filters via RPC.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_rpc_query_vertices() {
    let (_primal, client, server_handle) = setup_server_client(19606).await;

    // Create session
    let request = CreateSessionRequest {
        session_type: SessionType::General,
        description: None,
        parent_session: None,
        max_vertices: None,
        ttl_seconds: None,
    };
    let session_id = client.create_session(request).await.expect("should create session");

    // Add mixed events
    let types = [
        EventType::SessionStart,
        EventType::DataCreate {
            schema: None,
        },
        EventType::DataCreate {
            schema: Some("test".to_string()),
        },
        EventType::SessionStart,
    ];
    for event_type in types {
        client
            .append_event(AppendEventRequest {
                session_id,
                event_type,
                agent: None,
                parents: vec![],
                metadata: vec![],
                payload_ref: None,
            })
            .await
            .expect("should append event");
    }

    // Query for SessionStart only
    let query = QueryRequest {
        session_id,
        event_types: Some(vec![EventType::SessionStart]),
        agent: None,
        start_time: None,
        end_time: None,
        limit: None,
    };
    let results = client.query_vertices(query).await.expect("should query");
    assert_eq!(results.len(), 2); // Two SessionStart events

    // Query with limit
    let query = QueryRequest {
        session_id,
        event_types: None,
        agent: None,
        start_time: None,
        end_time: None,
        limit: Some(2),
    };
    let results = client.query_vertices(query).await.expect("should query");
    assert_eq!(results.len(), 2);

    server_handle.abort();
}

/// Test genesis and children via RPC.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_rpc_genesis_and_children() {
    let (_primal, client, server_handle) = setup_server_client(19607).await;

    // Create session
    let request = CreateSessionRequest {
        session_type: SessionType::General,
        description: None,
        parent_session: None,
        max_vertices: None,
        ttl_seconds: None,
    };
    let session_id = client.create_session(request).await.expect("should create session");

    // Add genesis vertex
    let genesis_id = client
        .append_event(AppendEventRequest {
            session_id,
            event_type: EventType::SessionStart,
            agent: None,
            parents: vec![],
            metadata: vec![],
            payload_ref: None,
        })
        .await
        .expect("should append genesis");

    // Add child vertices
    for _ in 0..3 {
        client
            .append_event(AppendEventRequest {
                session_id,
                event_type: EventType::DataCreate {
                    schema: None,
                },
                agent: None,
                parents: vec![genesis_id],
                metadata: vec![],
                payload_ref: None,
            })
            .await
            .expect("should append child");
    }

    // Get genesis
    let genesis = client.get_genesis(session_id).await.expect("should get genesis");
    assert_eq!(genesis.len(), 1);
    assert!(genesis.contains(&genesis_id));

    // Get children
    let children = client.get_children(session_id, genesis_id).await.expect("should get children");
    assert_eq!(children.len(), 3);

    server_handle.abort();
}

/// Test Merkle operations via RPC.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_rpc_merkle_operations() {
    let (_primal, client, server_handle) = setup_server_client(19608).await;

    // Create session
    let request = CreateSessionRequest {
        session_type: SessionType::General,
        description: None,
        parent_session: None,
        max_vertices: None,
        ttl_seconds: None,
    };
    let session_id = client.create_session(request).await.expect("should create session");

    // Add some vertices
    let vertex_id = client
        .append_event(AppendEventRequest {
            session_id,
            event_type: EventType::SessionStart,
            agent: None,
            parents: vec![],
            metadata: vec![],
            payload_ref: None,
        })
        .await
        .expect("should append event");

    // Get Merkle root
    let root = client.get_merkle_root(session_id).await.expect("should get merkle root");
    assert!(!root.as_bytes().iter().all(|&b| b == 0)); // Non-zero root

    // Get Merkle proof
    let proof =
        client.get_merkle_proof(session_id, vertex_id).await.expect("should get merkle proof");
    assert_eq!(proof.vertex_id, vertex_id);

    // Verify proof
    let valid = client.verify_proof(root, proof).await.expect("should verify proof");
    assert!(valid);

    server_handle.abort();
}

/// Test slice operations via RPC.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_rpc_slice_operations() {
    use rhizo_crypt_core::SliceMode;
    use rhizo_crypt_rpc::CheckoutSliceRequest;

    let (_primal, client, server_handle) = setup_server_client(19609).await;

    // Checkout a slice
    let checkout_request = CheckoutSliceRequest {
        spine_index: 0,
        mode: SliceMode::Copy {
            allow_recopy: true,
        },
        lender: None,
        borrower: None,
        duration_seconds: Some(3600),
    };
    let slice_id = client.checkout_slice(checkout_request).await.expect("should checkout slice");

    // Get slice
    let slice = client.get_slice(slice_id).await.expect("should get slice");
    assert_eq!(slice.id, slice_id);

    // List slices
    let slices = client.list_slices().await.expect("should list slices");
    assert!(!slices.is_empty());

    server_handle.abort();
}

/// Test dehydration operations via RPC.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_rpc_dehydration() {
    use rhizo_crypt_core::DehydrationStatus;

    let (_primal, client, server_handle) = setup_server_client(19610).await;

    // Create session with vertices
    let request = CreateSessionRequest {
        session_type: SessionType::General,
        description: None,
        parent_session: None,
        max_vertices: None,
        ttl_seconds: None,
    };
    let session_id = client.create_session(request).await.expect("should create session");

    client
        .append_event(AppendEventRequest {
            session_id,
            event_type: EventType::SessionStart,
            agent: None,
            parents: vec![],
            metadata: vec![],
            payload_ref: None,
        })
        .await
        .expect("should append event");

    // Check initial status
    let status = client.get_dehydration_status(session_id).await.expect("should get status");
    assert!(matches!(status, DehydrationStatus::Pending));

    // Dehydrate
    let root = client.dehydrate(session_id).await.expect("should dehydrate");
    assert!(!root.as_bytes().iter().all(|&b| b == 0));

    // Check final status
    let status = client.get_dehydration_status(session_id).await.expect("should get status");
    assert!(matches!(status, DehydrationStatus::Completed { .. }));

    server_handle.abort();
}
