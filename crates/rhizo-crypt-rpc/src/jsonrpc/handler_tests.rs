// SPDX-License-Identifier: AGPL-3.0-or-later
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

// ============================================================================
// Session lifecycle
// ============================================================================

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
async fn test_session_create_with_parent_session() {
    let primal = create_test_primal().await;

    let req = make_request(
        "dag.session.create",
        Some(json!({"session_type": "General", "description": "parent"})),
    );
    let result = handle_request(primal.clone(), req).await.unwrap();
    let parent_id = result.as_str().unwrap().to_string();

    let req = make_request(
        "dag.session.create",
        Some(json!({
            "session_type": "General",
            "description": "child",
            "parent_session": parent_id,
        })),
    );
    let result = handle_request(primal.clone(), req).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_session_create_with_limits() {
    let primal = create_test_primal().await;

    let req = make_request(
        "dag.session.create",
        Some(json!({
            "session_type": "General",
            "max_vertices": 1000,
            "ttl_seconds": 3600,
        })),
    );
    let result = handle_request(primal.clone(), req).await;
    assert!(result.is_ok());
}

// ============================================================================
// Event operations
// ============================================================================

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
async fn test_event_append_with_metadata_array() {
    let primal = create_test_primal().await;

    let req = make_request("dag.session.create", Some(json!({"session_type": "General"})));
    let session_id = handle_request(primal.clone(), req).await.unwrap();
    let session_id = session_id.as_str().unwrap();

    let req = make_request(
        "dag.event.append",
        Some(json!({
            "session_id": session_id,
            "event_type": {"SessionStart": null},
            "metadata": [
                {"key": "source", "value": "test"},
                {"key": "version", "value": "1.0"},
            ],
        })),
    );
    let result = handle_request(primal.clone(), req).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_event_append_with_payload_ref() {
    let primal = create_test_primal().await;

    let req = make_request("dag.session.create", Some(json!({"session_type": "General"})));
    let session_id = handle_request(primal.clone(), req).await.unwrap();
    let session_id = session_id.as_str().unwrap();

    let req = make_request(
        "dag.event.append",
        Some(json!({
            "session_id": session_id,
            "event_type": {"DataCreate": {"schema": "test-schema"}},
            "payload_ref": "ipfs://QmTest123",
        })),
    );
    let result = handle_request(primal.clone(), req).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_event_append_with_agent() {
    let primal = create_test_primal().await;

    let req = make_request("dag.session.create", Some(json!({"session_type": "General"})));
    let session_id = handle_request(primal.clone(), req).await.unwrap();
    let session_id = session_id.as_str().unwrap();

    let req = make_request(
        "dag.event.append",
        Some(json!({
            "session_id": session_id,
            "event_type": {"SessionStart": null},
            "agent": "did:eco:agent:test-001",
        })),
    );
    let result = handle_request(primal.clone(), req).await;
    assert!(result.is_ok());
}

// ============================================================================
// DAG topology: frontier, genesis, children
// ============================================================================

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

// ============================================================================
// Merkle operations
// ============================================================================

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

// ============================================================================
// Slice operations
// ============================================================================

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

// ============================================================================
// Health and introspection
// ============================================================================

#[tokio::test]
async fn test_health_check() {
    let primal = create_test_primal().await;

    let req = make_request("health.check", None);
    let result = handle_request(primal.clone(), req).await.unwrap();
    let health = result.as_object().unwrap();
    assert!(health.get("healthy").and_then(Value::as_bool).unwrap());
    assert!(health.contains_key("state"));
}

#[tokio::test]
async fn test_health_metrics() {
    let primal = create_test_primal().await;

    let req = make_request("health.metrics", None);
    let result = handle_request(primal.clone(), req).await.unwrap();
    let metrics = result.as_object().unwrap();
    assert!(metrics.contains_key("sessions_created"));
    assert!(metrics.contains_key("vertices_appended"));
}

#[tokio::test]
async fn test_capability_list() {
    let primal = create_test_primal().await;

    let req = make_request("capability.list", None);
    let result = handle_request(primal.clone(), req).await.unwrap();
    let obj = result.as_object().unwrap();

    assert_eq!(obj["primal"].as_str().unwrap(), "rhizocrypt", "L2: primal");
    assert!(obj.contains_key("version"), "L2: version");

    let methods = obj["methods"].as_array().unwrap();
    assert!(methods.len() >= 20, "L2: flat methods array");
    let method_strs: Vec<&str> = methods.iter().filter_map(Value::as_str).collect();
    assert!(method_strs.contains(&"dag.session.create"), "L2: dag method present");
    assert!(method_strs.contains(&"health.liveness"), "L2: health method present");
    assert!(method_strs.contains(&"identity.get"), "L2: identity method present");
    assert!(method_strs.contains(&"capabilities.list"), "L2: meta method present");

    let provided = obj["provided_capabilities"].as_array().unwrap();
    assert!(
        provided.iter().filter_map(|c| c.get("type").and_then(Value::as_str)).any(|t| t == "dag"),
        "L3: provided_capabilities dag group",
    );

    let consumed = obj["consumed_capabilities"].as_array().unwrap();
    assert!(!consumed.is_empty(), "L3: consumed_capabilities");
    assert!(
        consumed.iter().filter_map(Value::as_str).any(|s| s == "crypto.sign"),
        "L3: consumes crypto.sign",
    );

    let costs = obj["cost_estimates"].as_object().unwrap();
    assert!(costs.contains_key("dag.dehydration.trigger"), "L3: cost_estimates");

    let deps = obj["operation_dependencies"].as_object().unwrap();
    assert!(deps.contains_key("dag.event.append"), "L3: operation_dependencies");

    assert!(obj.contains_key("descriptors"), "detailed descriptors preserved");
}

// ============================================================================
// Dehydration
// ============================================================================

#[tokio::test]
async fn test_dehydrate_status_handler() {
    let primal = create_test_primal().await;
    let req = make_request("dag.session.create", Some(json!({"session_type": "General"})));
    let result = handle_request(primal.clone(), req).await.unwrap();
    let session_id = result.as_str().unwrap();

    let req = make_request("dag.dehydration.status", Some(json!({"session_id": session_id})));
    let result = handle_request(primal.clone(), req).await.unwrap();
    assert!(
        result.as_str().is_some() || result.as_object().is_some(),
        "dehydration.status should return string (unit variant) or object (struct variant)"
    );
}

#[tokio::test]
async fn test_dehydrate_alias_routes_to_trigger() {
    let primal = create_test_primal().await;
    let req = make_request("dag.session.create", Some(json!({"session_type": "General"})));
    let result = handle_request(primal.clone(), req).await.unwrap();
    let session_id = result.as_str().unwrap();

    let req_canonical =
        make_request("dag.dehydration.trigger", Some(json!({"session_id": session_id})));
    let result_canonical = handle_request(primal.clone(), req_canonical).await.unwrap();

    let req_alias = make_request("dag.dehydrate", Some(json!({"session_id": session_id})));
    let result_alias = handle_request(primal.clone(), req_alias).await.unwrap();

    assert_eq!(result_canonical, result_alias, "dag.dehydrate alias should route identically");
}

#[tokio::test]
async fn test_extra_fields_ignored() {
    let primal = create_test_primal().await;
    let req = make_request(
        "dag.session.create",
        Some(json!({
            "session_type": "General",
            "description": "test",
            "extra_field": "ignored",
            "another_extra": 123
        })),
    );
    let result = handle_request(primal.clone(), req).await.unwrap();
    assert!(uuid::Uuid::parse_str(result.as_str().unwrap()).is_ok());
}

// ============================================================================
// Health endpoint aliases
// ============================================================================

#[tokio::test]
async fn test_health_alias_status() {
    let primal = create_test_primal().await;
    let req = make_request("status", None);
    let result = handle_request(primal.clone(), req).await.unwrap();
    assert!(result.get("healthy").is_some());
}

#[tokio::test]
async fn test_health_alias_check() {
    let primal = create_test_primal().await;
    let req = make_request("check", None);
    let result = handle_request(primal.clone(), req).await.unwrap();
    assert!(result.get("healthy").is_some());
}

#[tokio::test]
async fn test_health_liveness_alias_ping() {
    let primal = create_test_primal().await;
    let req = make_request("ping", None);
    let result = handle_request(primal.clone(), req).await.unwrap();
    assert!(result.get("status").is_some() || result.get("alive").is_some() || result.is_object());
}

#[tokio::test]
async fn test_health_liveness_alias_health() {
    let primal = create_test_primal().await;
    let req = make_request("health", None);
    let result = handle_request(primal.clone(), req).await.unwrap();
    assert!(result.is_object());
}

#[tokio::test]
async fn test_health_readiness() {
    let primal = create_test_primal().await;
    let req = make_request("health.readiness", None);
    let result = handle_request(primal.clone(), req).await.unwrap();
    assert!(result.is_object());
}

// ============================================================================
// MCP tools.list / tools.call
// ============================================================================

#[tokio::test]
async fn test_mcp_tools_list() {
    let primal = create_test_primal().await;
    let req = make_request("tools.list", None);
    let result = handle_request(primal.clone(), req).await.unwrap();
    assert!(result.is_object() || result.is_array());
}

#[tokio::test]
async fn test_mcp_tools_list_alias() {
    let primal = create_test_primal().await;
    let req = make_request("mcp.tools.list", None);
    let result = handle_request(primal.clone(), req).await.unwrap();
    assert!(result.is_object() || result.is_array());
}

#[tokio::test]
async fn test_mcp_tools_call_session_create() {
    let primal = create_test_primal().await;
    let req = make_request(
        "tools.call",
        Some(json!({
            "name": "dag.session.create",
            "arguments": { "session_type": "General" }
        })),
    );
    let result = handle_request(primal.clone(), req).await.unwrap();
    assert!(uuid::Uuid::parse_str(result.as_str().unwrap()).is_ok());
}

#[tokio::test]
async fn test_mcp_tools_call_health() {
    let primal = create_test_primal().await;
    let req = make_request(
        "tools.call",
        Some(json!({
            "name": "health.check"
        })),
    );
    let result = handle_request(primal.clone(), req).await.unwrap();
    assert!(result.get("healthy").is_some());
}

#[tokio::test]
async fn test_mcp_tools_call_capabilities() {
    let primal = create_test_primal().await;
    let req = make_request(
        "tools.call",
        Some(json!({
            "name": "capabilities.list"
        })),
    );
    let result = handle_request(primal.clone(), req).await.unwrap();
    assert!(result.is_object(), "Format E wrapper is an object");
    assert!(result.get("provided_capabilities").is_some());
}

#[tokio::test]
async fn test_mcp_tools_call_unknown_tool() {
    let primal = create_test_primal().await;
    let req = make_request(
        "tools.call",
        Some(json!({
            "name": "nonexistent.tool"
        })),
    );
    let err = handle_request(primal.clone(), req).await.unwrap_err();
    assert!(matches!(err, HandlerError::MethodNotFound(_)));
}

#[tokio::test]
async fn test_mcp_tools_call_missing_arguments() {
    let primal = create_test_primal().await;
    let req = make_request("tools.call", Some(json!({"name": "status"})));
    let result = handle_request(primal.clone(), req).await.unwrap();
    assert!(result.get("healthy").is_some());
}

// ============================================================================
// Capability aliases
// ============================================================================

#[tokio::test]
async fn test_capability_list_aliases() {
    let primal = create_test_primal().await;
    for method in &["capabilities.list", "capability.list", "primal.capabilities"] {
        let req = make_request(method, None);
        let result = handle_request(primal.clone(), req).await.unwrap();
        assert!(
            result.get("provided_capabilities").is_some(),
            "capabilities.list alias '{method}' should return Format E wrapper"
        );
    }
}

#[tokio::test]
async fn test_identity_get() {
    let primal = create_test_primal().await;
    let req = make_request("identity.get", None);
    let result = handle_request(primal.clone(), req).await.unwrap();
    let obj = result.as_object().unwrap();
    assert_eq!(obj["primal"].as_str().unwrap(), "rhizocrypt");
    assert!(obj.contains_key("version"));
    assert_eq!(obj["domain"].as_str().unwrap(), "dag");
    assert!(obj.contains_key("description"));
}

// ============================================================================
// HandlerError::Rpc propagation
// ============================================================================

#[tokio::test]
async fn test_handler_rpc_error_session_not_found() {
    let primal = create_test_primal().await;
    let fake_id = "00000000-0000-0000-0000-000000000099";
    let req = make_request("dag.session.get", Some(json!({"session_id": fake_id})));
    let err = handle_request(primal.clone(), req).await.unwrap_err();
    assert!(matches!(err, HandlerError::Rpc(_)));
}
