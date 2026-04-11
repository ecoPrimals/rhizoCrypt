// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2024–2026 ecoPrimals Project

use super::*;
use rhizo_crypt_core::{EventType, PrimalLifecycle, RhizoCrypt, RhizoCryptConfig, SessionType};
use std::sync::Arc;

#[test]
fn test_request_serialization() {
    let request = CreateSessionRequest {
        session_type: SessionType::default(),
        description: Some("test".to_string()),
        parent_session: None,
        max_vertices: Some(1000),
        ttl_seconds: Some(3600),
    };

    let json = serde_json::to_string(&request).expect("serialize");
    let parsed: CreateSessionRequest = serde_json::from_str(&json).expect("deserialize");

    assert_eq!(parsed.description, request.description);
    assert_eq!(parsed.max_vertices, request.max_vertices);
}

#[test]
fn test_health_status_serialization() {
    let status = HealthStatus {
        healthy: true,
        state: "running".to_string(),
        active_sessions: 5,
        total_vertices: 1000,
        uptime_seconds: 3600,
    };

    let json = serde_json::to_string(&status).expect("serialize");
    let parsed: HealthStatus = serde_json::from_str(&json).expect("deserialize");

    assert!(parsed.healthy);
    assert_eq!(parsed.active_sessions, 5);
}

async fn make_test_server() -> RhizoCryptRpcServer {
    let mut primal = RhizoCrypt::new(RhizoCryptConfig::default());
    primal.start().await.unwrap();
    let primal = Arc::new(primal);
    RhizoCryptRpcServer::new(primal)
}

#[tokio::test]
async fn test_service_all_methods_accessible() {
    let server = make_test_server().await;

    let req = CreateSessionRequest {
        session_type: SessionType::default(),
        description: Some("test".to_string()),
        parent_session: None,
        max_vertices: Some(1000),
        ttl_seconds: Some(3600),
    };
    let session_id = server.clone().create_session(tarpc::context::current(), req).await.unwrap();

    let _ = server.clone().get_session(tarpc::context::current(), session_id).await.unwrap();
    let _ = server.clone().list_sessions(tarpc::context::current()).await.unwrap();

    let append_req = AppendEventRequest {
        session_id,
        event_type: EventType::SessionStart,
        agent: None,
        parents: vec![],
        metadata: vec![],
        payload_ref: None,
    };
    let vertex_id =
        server.clone().append_event(tarpc::context::current(), append_req).await.unwrap();

    let _ =
        server.clone().get_vertex(tarpc::context::current(), session_id, vertex_id).await.unwrap();
    let _ = server.clone().get_frontier(tarpc::context::current(), session_id).await.unwrap();
    let _ = server.clone().get_genesis(tarpc::context::current(), session_id).await.unwrap();

    let query_req = QueryRequest {
        session_id,
        event_types: None,
        agent: None,
        start_time: None,
        end_time: None,
        limit: None,
    };
    let _ = server.clone().query_vertices(tarpc::context::current(), query_req).await.unwrap();

    let _ = server.clone().get_merkle_root(tarpc::context::current(), session_id).await.unwrap();

    let _ = server.clone().health(tarpc::context::current()).await.unwrap();
    let _ = server.clone().metrics(tarpc::context::current()).await.unwrap();
}

#[tokio::test]
async fn test_dehydration_status() {
    let server = make_test_server().await;

    let req = CreateSessionRequest {
        session_type: SessionType::default(),
        description: None,
        parent_session: None,
        max_vertices: None,
        ttl_seconds: None,
    };
    let session_id = server.clone().create_session(tarpc::context::current(), req).await.unwrap();

    let status =
        server.get_dehydration_status(tarpc::context::current(), session_id).await.unwrap();
    assert!(matches!(status, rhizo_crypt_core::DehydrationStatus::Pending));
}

#[tokio::test]
async fn test_get_slice_not_found() {
    let server = make_test_server().await;

    let nonexistent_id = SliceId::now();
    let result = server.get_slice(tarpc::context::current(), nonexistent_id).await;
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), RpcError::SliceNotFound(_)));
}

#[tokio::test]
async fn test_list_slices_empty() {
    let server = make_test_server().await;

    let slices = server.list_slices(tarpc::context::current()).await.unwrap();
    assert!(slices.is_empty());
}

#[tokio::test]
async fn test_resolve_slice_not_found() {
    let server = make_test_server().await;

    let nonexistent_id = SliceId::now();
    let session_id = SessionId::now();
    let result = server.resolve_slice(tarpc::context::current(), nonexistent_id, session_id).await;
    assert!(result.is_err());
}
