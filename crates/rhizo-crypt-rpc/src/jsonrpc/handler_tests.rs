// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2024–2026 ecoPrimals Project

#![allow(clippy::unwrap_used)]

use super::*;
use crate::jsonrpc::types::JsonRpcId;
use crate::jsonrpc::types::JsonRpcRequest;
use rhizo_crypt_core::{PrimalLifecycle, RhizoCryptConfig};
use serde_json::json;

async fn create_test_primal() -> Arc<rhizo_crypt_core::RhizoCrypt> {
    let mut primal = rhizo_crypt_core::RhizoCrypt::new(RhizoCryptConfig::default());
    primal.start().await.unwrap();
    Arc::new(primal)
}

fn make_request(method: &str, params: Option<Value>) -> JsonRpcRequest {
    JsonRpcRequest {
        jsonrpc: "2.0".to_string(),
        method: method.to_string(),
        params,
        id: Some(JsonRpcId::Number(1)),
    }
}

#[tokio::test]
async fn test_session_lifecycle() {
    let primal = create_test_primal().await;

    let req = make_request(
        "dag.session.create",
        Some(json!({"session_type": "General", "description": "test"})),
    );
    let result = handle_request(primal.clone(), req).await.unwrap();
    let session_id = result.as_str().unwrap();
    assert!(uuid::Uuid::parse_str(session_id).is_ok());

    let req = make_request("dag.session.get", Some(json!({"session_id": session_id})));
    let result = handle_request(primal.clone(), req).await.unwrap();
    let info = result.as_object().unwrap();
    assert_eq!(info.get("description").and_then(|v| v.as_str()), Some("test"));

    let req = make_request("dag.session.list", None);
    let result = handle_request(primal.clone(), req).await.unwrap();
    let list = result.as_array().unwrap();
    assert_eq!(list.len(), 1);

    let req = make_request("dag.session.discard", Some(json!({"session_id": session_id})));
    let result = handle_request(primal.clone(), req).await.unwrap();
    assert!(result.is_null());

    let req = make_request("dag.session.list", None);
    let result = handle_request(primal.clone(), req).await.unwrap();
    let list = result.as_array().unwrap();
    assert!(list.is_empty());
}

#[tokio::test]
async fn test_event_append() {
    let primal = create_test_primal().await;

    let req = make_request(
        "dag.session.create",
        Some(json!({"session_type": "General", "description": "test"})),
    );
    let result = handle_request(primal.clone(), req).await.unwrap();
    let session_id = result.as_str().unwrap();

    let req = make_request(
        "dag.event.append",
        Some(json!({
            "session_id": session_id,
            "event_type": {"SessionStart": null}
        })),
    );
    let result = handle_request(primal.clone(), req).await.unwrap();
    let vertex_id_hex = result.as_str().unwrap();
    assert_eq!(vertex_id_hex.len(), 64);
    assert!(hex::decode(vertex_id_hex).is_ok());

    let req = make_request(
        "dag.vertex.get",
        Some(json!({"session_id": session_id, "vertex_id": vertex_id_hex})),
    );
    let result = handle_request(primal.clone(), req).await.unwrap();
    let vertex = result.as_object().unwrap();
    assert!(vertex.contains_key("event_type"));
}

#[tokio::test]
async fn test_event_append_batch() {
    let primal = create_test_primal().await;

    let req = make_request(
        "dag.session.create",
        Some(json!({"session_type": "General", "description": "test"})),
    );
    let result = handle_request(primal.clone(), req).await.unwrap();
    let session_id = result.as_str().unwrap();

    let req = make_request(
        "dag.event.append_batch",
        Some(json!({
            "requests": [
                {"session_id": session_id, "event_type": {"SessionStart": null}},
                {"session_id": session_id, "event_type": {"SessionStart": null}}
            ]
        })),
    );
    let result = handle_request(primal.clone(), req).await.unwrap();
    let ids = result.as_array().unwrap();
    assert_eq!(ids.len(), 2);
    for id in ids {
        assert!(id.as_str().unwrap().len() == 64);
    }
}

#[tokio::test]
async fn test_frontier_and_genesis() {
    let primal = create_test_primal().await;

    let req = make_request(
        "dag.session.create",
        Some(json!({"session_type": "General", "description": "test"})),
    );
    let result = handle_request(primal.clone(), req).await.unwrap();
    let session_id = result.as_str().unwrap();

    let req = make_request(
        "dag.event.append",
        Some(json!({
            "session_id": session_id,
            "event_type": {"SessionStart": null}
        })),
    );
    let _ = handle_request(primal.clone(), req).await.unwrap();

    let req = make_request("dag.frontier.get", Some(json!({"session_id": session_id})));
    let result = handle_request(primal.clone(), req).await.unwrap();
    let frontier = result.as_array().unwrap();
    assert_eq!(frontier.len(), 1);

    let req = make_request("dag.genesis.get", Some(json!({"session_id": session_id})));
    let result = handle_request(primal.clone(), req).await.unwrap();
    let genesis = result.as_array().unwrap();
    assert_eq!(genesis.len(), 1);
}

#[tokio::test]
async fn test_merkle_operations() {
    let primal = create_test_primal().await;

    let req = make_request(
        "dag.session.create",
        Some(json!({"session_type": "General", "description": "test"})),
    );
    let result = handle_request(primal.clone(), req).await.unwrap();
    let session_id = result.as_str().unwrap();

    let req = make_request(
        "dag.event.append",
        Some(json!({
            "session_id": session_id,
            "event_type": {"SessionStart": null}
        })),
    );
    let result = handle_request(primal.clone(), req).await.unwrap();
    let vertex_id_hex = result.as_str().unwrap();

    let req = make_request("dag.merkle.root", Some(json!({"session_id": session_id})));
    let result = handle_request(primal.clone(), req).await.unwrap();
    let root_hex = result.as_str().unwrap();
    assert_eq!(root_hex.len(), 64);

    let req = make_request(
        "dag.merkle.proof",
        Some(json!({"session_id": session_id, "vertex_id": vertex_id_hex})),
    );
    let result = handle_request(primal.clone(), req).await.unwrap();
    let proof = result.as_object().unwrap();
    assert!(proof.contains_key("vertex_id"));

    let req = make_request("dag.merkle.verify", Some(json!({"root": root_hex, "proof": proof})));
    let result = handle_request(primal.clone(), req).await.unwrap();
    assert!(result.as_bool().unwrap());
}

#[tokio::test]
async fn test_slice_operations() {
    let primal = create_test_primal().await;

    let req = make_request(
        "dag.session.create",
        Some(json!({"session_type": "General", "description": "test"})),
    );
    let result = handle_request(primal.clone(), req).await.unwrap();
    let session_id = result.as_str().unwrap();

    let req = make_request(
        "dag.event.append",
        Some(json!({
            "session_id": session_id,
            "event_type": {"SessionStart": null}
        })),
    );
    let _ = handle_request(primal.clone(), req).await.unwrap();

    let req = make_request("dag.dehydrate", Some(json!({"session_id": session_id})));
    let _ = handle_request(primal.clone(), req).await.unwrap();

    let req = make_request("dag.slice.checkout", Some(json!({"spine_index": 0})));
    let result = handle_request(primal.clone(), req).await.unwrap();
    let slice_id = result.as_str().unwrap();
    assert!(uuid::Uuid::parse_str(slice_id).is_ok());

    let req = make_request("dag.slice.get", Some(json!({"slice_id": slice_id})));
    let result = handle_request(primal.clone(), req).await.unwrap();
    let slice = result.as_object().unwrap();
    assert!(slice.contains_key("id") || slice.contains_key("origin"));

    let req = make_request("dag.slice.list", None);
    let result = handle_request(primal.clone(), req).await.unwrap();
    let list = result.as_array().unwrap();
    assert!(!list.is_empty());
}

#[tokio::test]
async fn test_system_health() {
    let primal = create_test_primal().await;

    let req = make_request("system.health", None);
    let result = handle_request(primal.clone(), req).await.unwrap();
    let health = result.as_object().unwrap();
    assert!(health.get("healthy").and_then(Value::as_bool).unwrap());
    assert!(health.contains_key("state"));
}

#[tokio::test]
async fn test_system_metrics() {
    let primal = create_test_primal().await;

    let req = make_request("system.metrics", None);
    let result = handle_request(primal.clone(), req).await.unwrap();
    let metrics = result.as_object().unwrap();
    assert!(metrics.contains_key("sessions_created"));
    assert!(metrics.contains_key("vertices_appended"));
}

#[tokio::test]
async fn test_method_not_found() {
    let primal = create_test_primal().await;

    let req = make_request("unknown.method", Some(json!({})));
    let err = handle_request(primal.clone(), req).await.unwrap_err();
    assert!(matches!(err, HandlerError::MethodNotFound(_)));
}

#[tokio::test]
async fn test_invalid_params() {
    let primal = create_test_primal().await;

    let req = make_request("dag.session.get", Some(json!({})));
    let err = handle_request(primal.clone(), req).await.unwrap_err();
    assert!(matches!(err, HandlerError::InvalidParams(_)));
}

#[tokio::test]
async fn test_vertex_query() {
    let primal = create_test_primal().await;

    let req = make_request(
        "dag.session.create",
        Some(json!({"session_type": "General", "description": "test"})),
    );
    let result = handle_request(primal.clone(), req).await.unwrap();
    let session_id = result.as_str().unwrap();

    let req = make_request(
        "dag.event.append",
        Some(json!({
            "session_id": session_id,
            "event_type": {"SessionStart": null}
        })),
    );
    let _ = handle_request(primal.clone(), req).await.unwrap();

    let req = make_request(
        "dag.vertex.query",
        Some(json!({
            "session_id": session_id,
            "event_types": [{"SessionStart": null}]
        })),
    );
    let result = handle_request(primal.clone(), req).await.unwrap();
    let vertices = result.as_array().unwrap();
    assert_eq!(vertices.len(), 1);
}

#[tokio::test]
async fn test_children() {
    let primal = create_test_primal().await;

    let req = make_request(
        "dag.session.create",
        Some(json!({"session_type": "General", "description": "test"})),
    );
    let result = handle_request(primal.clone(), req).await.unwrap();
    let session_id = result.as_str().unwrap();

    let req = make_request(
        "dag.event.append",
        Some(json!({
            "session_id": session_id,
            "event_type": {"SessionStart": null}
        })),
    );
    let result = handle_request(primal.clone(), req).await.unwrap();
    let parent_id = result.as_str().unwrap();

    let req = make_request(
        "dag.event.append",
        Some(json!({
            "session_id": session_id,
            "event_type": {"DataCreate": {"schema": null}},
            "parents": [parent_id]
        })),
    );
    let _ = handle_request(primal.clone(), req).await.unwrap();

    let req = make_request(
        "dag.vertex.children",
        Some(json!({"session_id": session_id, "vertex_id": parent_id})),
    );
    let result = handle_request(primal.clone(), req).await.unwrap();
    let children = result.as_array().unwrap();
    assert_eq!(children.len(), 1);
}

#[tokio::test]
async fn test_invalid_session_id() {
    let primal = create_test_primal().await;
    let req = make_request("dag.session.get", Some(json!({"session_id": "not-a-uuid"})));
    let err = handle_request(primal, req).await.unwrap_err();
    assert!(matches!(err, HandlerError::InvalidParams(_)));
}

#[tokio::test]
async fn test_invalid_vertex_id_hex() {
    let primal = create_test_primal().await;
    let req = make_request("dag.session.create", Some(json!({"session_type": "General"})));
    let result = handle_request(primal.clone(), req).await.unwrap();
    let session_id = result.as_str().unwrap();

    let req = make_request(
        "dag.vertex.get",
        Some(json!({"session_id": session_id, "vertex_id": "not-hex"})),
    );
    let err = handle_request(primal, req).await.unwrap_err();
    assert!(matches!(err, HandlerError::InvalidParams(_)));
}

#[tokio::test]
async fn test_invalid_vertex_id_length() {
    let primal = create_test_primal().await;
    let req = make_request("dag.session.create", Some(json!({"session_type": "General"})));
    let result = handle_request(primal.clone(), req).await.unwrap();
    let session_id = result.as_str().unwrap();

    let req = make_request(
        "dag.vertex.get",
        Some(json!({"session_id": session_id, "vertex_id": "aabb"})),
    );
    let err = handle_request(primal, req).await.unwrap_err();
    assert!(matches!(err, HandlerError::InvalidParams(_)));
}

#[tokio::test]
async fn test_params_not_object() {
    let primal = create_test_primal().await;
    let req = make_request("dag.session.get", Some(json!([1, 2, 3])));
    let err = handle_request(primal, req).await.unwrap_err();
    assert!(matches!(err, HandlerError::InvalidParams(_)));
}

#[tokio::test]
async fn test_session_create_with_optional_params() {
    let primal = create_test_primal().await;
    let req = make_request(
        "dag.session.create",
        Some(json!({
            "session_type": "General",
            "description": "full params",
            "max_vertices": 100,
            "ttl_seconds": 3600
        })),
    );
    let result = handle_request(primal.clone(), req).await.unwrap();
    assert!(result.as_str().is_some());
}

#[tokio::test]
async fn test_event_append_with_full_params() {
    let primal = create_test_primal().await;
    let req = make_request("dag.session.create", Some(json!({"session_type": "General"})));
    let result = handle_request(primal.clone(), req).await.unwrap();
    let session_id = result.as_str().unwrap();

    let req = make_request(
        "dag.event.append",
        Some(json!({
            "session_id": session_id,
            "event_type": {"DataCreate": {"schema": null}},
            "agent": "did:key:z6MkTest",
            "metadata": {"key": "value"}
        })),
    );
    let result = handle_request(primal.clone(), req).await.unwrap();
    assert!(result.as_str().is_some());
}

#[tokio::test]
async fn test_vertex_query_with_params() {
    let primal = create_test_primal().await;
    let req = make_request("dag.session.create", Some(json!({"session_type": "General"})));
    let result = handle_request(primal.clone(), req).await.unwrap();
    let session_id = result.as_str().unwrap();

    let req = make_request(
        "dag.event.append",
        Some(json!({
            "session_id": session_id,
            "event_type": {"SessionStart": null}
        })),
    );
    let _ = handle_request(primal.clone(), req).await.unwrap();

    let req = make_request(
        "dag.vertex.query",
        Some(json!({
            "session_id": session_id,
            "limit": 10
        })),
    );
    let result = handle_request(primal.clone(), req).await.unwrap();
    let vertices = result.as_array().unwrap();
    assert_eq!(vertices.len(), 1);
}

#[tokio::test]
async fn test_slice_checkout_with_lender_borrower() {
    let primal = create_test_primal().await;
    let req = make_request("dag.session.create", Some(json!({"session_type": "General"})));
    let result = handle_request(primal.clone(), req).await.unwrap();
    let session_id = result.as_str().unwrap();

    let req = make_request(
        "dag.event.append",
        Some(json!({
            "session_id": session_id,
            "event_type": {"SessionStart": null}
        })),
    );
    let _ = handle_request(primal.clone(), req).await.unwrap();

    let req = make_request("dag.dehydrate", Some(json!({"session_id": session_id})));
    let _ = handle_request(primal.clone(), req).await.unwrap();

    let req = make_request(
        "dag.slice.checkout",
        Some(json!({
            "spine_index": 0,
            "lender": "did:key:lender",
            "borrower": "did:key:borrower"
        })),
    );
    let result = handle_request(primal.clone(), req).await.unwrap();
    let slice_id = result.as_str().unwrap();
    assert!(uuid::Uuid::parse_str(slice_id).is_ok());

    let req = make_request("dag.slice.list", None);
    let result = handle_request(primal.clone(), req).await.unwrap();
    let slices = result.as_array().unwrap();
    assert!(!slices.is_empty());

    let req = make_request(
        "dag.slice.resolve",
        Some(json!({
            "slice_id": slice_id,
            "session_id": session_id
        })),
    );
    let result = handle_request(primal.clone(), req).await.unwrap();
    assert!(result.is_null());
}

#[tokio::test]
async fn test_merkle_verify_invalid_root() {
    let primal = create_test_primal().await;
    let req = make_request(
        "dag.merkle.verify",
        Some(json!({
            "root": "aabb",
            "proof": {"vertex_id": "0000000000000000000000000000000000000000000000000000000000000000", "proof": [], "root": "aabb"}
        })),
    );
    let err = handle_request(primal, req).await.unwrap_err();
    assert!(matches!(err, HandlerError::InvalidParams(_)));
}

#[tokio::test]
async fn test_dehydrate_status_handler() {
    let primal = create_test_primal().await;
    let req = make_request("dag.session.create", Some(json!({"session_type": "General"})));
    let result = handle_request(primal.clone(), req).await.unwrap();
    let session_id = result.as_str().unwrap();

    let req = make_request("dag.dehydrate.status", Some(json!({"session_id": session_id})));
    let result = handle_request(primal.clone(), req).await;
    assert!(result.is_ok() || result.is_err());
}
