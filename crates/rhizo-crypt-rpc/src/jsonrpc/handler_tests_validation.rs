// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2024–2026 ecoPrimals Project

//! Validation and error-path tests for the JSON-RPC handler.

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

// ============================================================================
// Method dispatch errors
// ============================================================================

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
async fn test_params_not_object() {
    let primal = create_test_primal().await;
    let req = make_request("dag.session.get", Some(json!([1, 2, 3])));
    let err = handle_request(primal, req).await.unwrap_err();
    assert!(matches!(err, HandlerError::InvalidParams(_)));
}

#[tokio::test]
async fn test_params_null() {
    let primal = create_test_primal().await;
    let req = make_request("dag.session.get", None);
    let err = handle_request(primal, req).await.unwrap_err();
    assert!(matches!(err, HandlerError::InvalidParams(_)));
}

// ============================================================================
// Session validation
// ============================================================================

#[tokio::test]
async fn test_invalid_session_id() {
    let primal = create_test_primal().await;
    let req = make_request("dag.session.get", Some(json!({"session_id": "not-a-uuid"})));
    let err = handle_request(primal, req).await.unwrap_err();
    assert!(matches!(err, HandlerError::InvalidParams(_)));
}

#[tokio::test]
async fn test_session_create_with_invalid_parent_session() {
    let primal = create_test_primal().await;

    let req = make_request(
        "dag.session.create",
        Some(json!({
            "session_type": "General",
            "parent_session": "not-a-uuid",
        })),
    );
    let result = handle_request(primal.clone(), req).await;
    assert!(result.is_err());
}

// ============================================================================
// Vertex ID validation
// ============================================================================

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
async fn test_vertex_children_invalid_vertex_id() {
    let primal = create_test_primal().await;
    let req = make_request("dag.session.create", Some(json!({"session_type": "General"})));
    let result = handle_request(primal.clone(), req).await.unwrap();
    let session_id = result.as_str().unwrap();

    let req = make_request(
        "dag.vertex.children",
        Some(json!({"session_id": session_id, "vertex_id": "zzzz"})),
    );
    let err = handle_request(primal, req).await.unwrap_err();
    assert!(matches!(err, HandlerError::InvalidParams(_)));
}

#[tokio::test]
async fn test_vertex_children_empty() {
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
    let vertex_id = result.as_str().unwrap();

    let req = make_request(
        "dag.vertex.children",
        Some(json!({"session_id": session_id, "vertex_id": vertex_id})),
    );
    let result = handle_request(primal.clone(), req).await.unwrap();
    let children = result.as_array().unwrap();
    assert!(children.is_empty());
}

// ============================================================================
// Event validation
// ============================================================================

#[tokio::test]
async fn test_event_append_invalid_event_type() {
    let primal = create_test_primal().await;

    let req = make_request("dag.session.create", Some(json!({"session_type": "General"})));
    let session_id = handle_request(primal.clone(), req).await.unwrap();
    let session_id = session_id.as_str().unwrap();

    let req = make_request(
        "dag.event.append",
        Some(json!({
            "session_id": session_id,
            "event_type": {"CompletelyInvalidType": {}},
        })),
    );
    let result = handle_request(primal.clone(), req).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_event_append_missing_event_type() {
    let primal = create_test_primal().await;

    let req = make_request("dag.session.create", Some(json!({"session_type": "General"})));
    let session_id = handle_request(primal.clone(), req).await.unwrap();
    let session_id = session_id.as_str().unwrap();

    let req = make_request("dag.event.append", Some(json!({"session_id": session_id})));
    let result = handle_request(primal.clone(), req).await;
    assert!(result.is_err());
}

// ============================================================================
// Batch validation
// ============================================================================

#[tokio::test]
async fn test_event_append_batch_empty_array() {
    let primal = create_test_primal().await;
    let req = make_request(
        "dag.session.create",
        Some(json!({"session_type": "General", "description": "test"})),
    );
    let _ = handle_request(primal.clone(), req).await.unwrap();

    let req = make_request("dag.event.append_batch", Some(json!({"requests": []})));
    let result = handle_request(primal.clone(), req).await.unwrap();
    let ids = result.as_array().unwrap();
    assert!(ids.is_empty());
}

#[tokio::test]
async fn test_event_append_batch_single_with_metadata() {
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
            "requests": [{
                "session_id": session_id,
                "event_type": {"DataCreate": {"schema": null}},
                "metadata": [{"key": "k1", "value": "v1"}],
                "agent": "did:key:z6MkTest"
            }]
        })),
    );
    let result = handle_request(primal.clone(), req).await.unwrap();
    let ids = result.as_array().unwrap();
    assert_eq!(ids.len(), 1);
    assert!(ids[0].as_str().unwrap().len() == 64);
}

#[tokio::test]
async fn test_event_append_batch_missing_requests() {
    let primal = create_test_primal().await;
    let req = make_request("dag.event.append_batch", Some(json!({})));
    let err = handle_request(primal, req).await.unwrap_err();
    assert!(matches!(err, HandlerError::InvalidParams(_)));
}

#[tokio::test]
async fn test_event_append_batch_requests_not_array() {
    let primal = create_test_primal().await;
    let req = make_request("dag.event.append_batch", Some(json!({"requests": "not-array"})));
    let err = handle_request(primal, req).await.unwrap_err();
    assert!(matches!(err, HandlerError::InvalidParams(_)));
}

#[tokio::test]
async fn test_event_append_batch_request_not_object() {
    let primal = create_test_primal().await;
    let req = make_request("dag.session.create", Some(json!({"session_type": "General"})));
    let result = handle_request(primal.clone(), req).await.unwrap();
    let session_id = result.as_str().unwrap();

    let req = make_request(
        "dag.event.append_batch",
        Some(json!({
            "requests": [session_id]
        })),
    );
    let err = handle_request(primal, req).await.unwrap_err();
    assert!(matches!(err, HandlerError::InvalidParams(_)));
}

#[tokio::test]
async fn test_event_append_batch_invalid_session_id() {
    let primal = create_test_primal().await;
    let req = make_request(
        "dag.event.append_batch",
        Some(json!({
            "requests": [{"session_id": "not-a-uuid", "event_type": {"SessionStart": null}}]
        })),
    );
    let err = handle_request(primal, req).await.unwrap_err();
    assert!(matches!(err, HandlerError::InvalidParams(_)));
}

#[tokio::test]
async fn test_event_append_batch_invalid_parents_hex() {
    let primal = create_test_primal().await;
    let req = make_request("dag.session.create", Some(json!({"session_type": "General"})));
    let result = handle_request(primal.clone(), req).await.unwrap();
    let session_id = result.as_str().unwrap();

    let req = make_request(
        "dag.event.append_batch",
        Some(json!({
            "requests": [{
                "session_id": session_id,
                "event_type": {"SessionStart": null},
                "parents": ["not-valid-hex"]
            }]
        })),
    );
    let err = handle_request(primal, req).await.unwrap_err();
    assert!(matches!(err, HandlerError::InvalidParams(_)));
}

#[tokio::test]
async fn test_event_append_batch_params_not_object() {
    let primal = create_test_primal().await;
    let req = make_request("dag.event.append_batch", Some(json!([1, 2, 3])));
    let err = handle_request(primal, req).await.unwrap_err();
    assert!(matches!(err, HandlerError::InvalidParams(_)));
}

// ============================================================================
// Merkle validation
// ============================================================================

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

// ============================================================================
// Slice validation
// ============================================================================

#[tokio::test]
async fn test_slice_checkout_missing_required_fields() {
    let primal = create_test_primal().await;
    let req = make_request("dag.slice.checkout", Some(json!({})));
    let err = handle_request(primal, req).await.unwrap_err();
    assert!(matches!(err, HandlerError::InvalidParams(_)));
}

#[tokio::test]
async fn test_slice_get_invalid_slice_id() {
    let primal = create_test_primal().await;
    let req = make_request("dag.slice.get", Some(json!({"slice_id": "not-a-uuid"})));
    let err = handle_request(primal, req).await.unwrap_err();
    assert!(matches!(err, HandlerError::InvalidParams(_)));
}

#[tokio::test]
async fn test_slice_get_missing_slice_id() {
    let primal = create_test_primal().await;
    let req = make_request("dag.slice.get", Some(json!({})));
    let err = handle_request(primal, req).await.unwrap_err();
    assert!(matches!(err, HandlerError::InvalidParams(_)));
}

#[tokio::test]
async fn test_slice_resolve_invalid_slice_id() {
    let primal = create_test_primal().await;
    let req = make_request("dag.session.create", Some(json!({"session_type": "General"})));
    let result = handle_request(primal.clone(), req).await.unwrap();
    let session_id = result.as_str().unwrap();

    let req = make_request(
        "dag.slice.resolve",
        Some(json!({"slice_id": "bad-uuid", "session_id": session_id})),
    );
    let err = handle_request(primal, req).await.unwrap_err();
    assert!(matches!(err, HandlerError::InvalidParams(_)));
}

#[tokio::test]
async fn test_slice_resolve_invalid_session_id() {
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

    let req = make_request("dag.dehydration.trigger", Some(json!({"session_id": session_id})));
    let _ = handle_request(primal.clone(), req).await.unwrap();

    let zero_vertex = "0".repeat(64);
    let req = make_request(
        "dag.slice.checkout",
        Some(json!({
            "spine_id": "spine-0",
            "entry_hash": "00".repeat(32),
            "entry_index": 0,
            "owner": "did:eco:owner",
            "holder": "did:eco:holder",
            "session_id": session_id,
            "checkout_vertex": zero_vertex,
        })),
    );
    let result = handle_request(primal.clone(), req).await.unwrap();
    let slice_id = result.as_str().unwrap();

    let req = make_request(
        "dag.slice.resolve",
        Some(json!({"slice_id": slice_id, "session_id": "not-a-uuid"})),
    );
    let err = handle_request(primal, req).await.unwrap_err();
    assert!(matches!(err, HandlerError::InvalidParams(_)));
}

#[tokio::test]
async fn test_slice_list_with_null_params() {
    let primal = create_test_primal().await;
    let req = make_request("dag.slice.list", None);
    let result = handle_request(primal, req).await.unwrap();
    let list = result.as_array().unwrap();
    assert!(list.is_empty());
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

    let req = make_request("dag.dehydration.trigger", Some(json!({"session_id": session_id})));
    let _ = handle_request(primal.clone(), req).await.unwrap();

    let zero_vertex = "0".repeat(64);
    let req = make_request(
        "dag.slice.checkout",
        Some(json!({
            "spine_id": "spine-0",
            "entry_hash": "00".repeat(32),
            "entry_index": 0,
            "owner": "did:key:lender",
            "holder": "did:key:borrower",
            "session_id": session_id,
            "checkout_vertex": zero_vertex,
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
async fn test_slice_checkout_with_mode_and_duration() {
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

    let req = make_request("dag.dehydration.trigger", Some(json!({"session_id": session_id})));
    let _ = handle_request(primal.clone(), req).await.unwrap();

    let zero_vertex = "0".repeat(64);
    let req = make_request(
        "dag.slice.checkout",
        Some(json!({
            "spine_id": "spine-0",
            "entry_hash": "00".repeat(32),
            "entry_index": 0,
            "mode": {"Copy": {"allow_recopy": true}},
            "owner": "did:eco:owner",
            "holder": "did:eco:holder",
            "session_id": session_id,
            "checkout_vertex": zero_vertex,
            "duration_seconds": 3600
        })),
    );
    let result = handle_request(primal.clone(), req).await.unwrap();
    assert!(uuid::Uuid::parse_str(result.as_str().unwrap()).is_ok());
}

#[tokio::test]
async fn test_slice_checkout_missing_entry_index() {
    let primal = create_test_primal().await;

    let req = make_request("dag.session.create", Some(json!({"session_type": "General"})));
    let session_id = handle_request(primal.clone(), req).await.unwrap();
    let session_id = session_id.as_str().unwrap();

    let req = make_request(
        "dag.event.append",
        Some(json!({
            "session_id": session_id,
            "event_type": {"SessionStart": null}
        })),
    );
    let _ = handle_request(primal.clone(), req).await.unwrap();

    let req = make_request(
        "dag.slice.checkout",
        Some(json!({
            "spine_id": "spine-0",
            "entry_hash": "00".repeat(32),
            "mode": {"Copy": {"allow_recopy": true}},
            "owner": "did:eco:owner",
            "holder": "did:eco:holder",
            "session_id": session_id,
            "checkout_vertex": "0".repeat(64),
            "duration_seconds": 3600
        })),
    );
    let result = handle_request(primal.clone(), req).await;
    assert!(result.is_err(), "should fail without entry_index");
}

// ============================================================================
// Dehydration validation
// ============================================================================

#[tokio::test]
async fn test_dehydration_status_invalid_session_id() {
    let primal = create_test_primal().await;
    let req = make_request("dag.dehydration.status", Some(json!({"session_id": "bad-uuid"})));
    let err = handle_request(primal, req).await.unwrap_err();
    assert!(matches!(err, HandlerError::InvalidParams(_)));
}

// ============================================================================
// Vertex query
// ============================================================================

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
