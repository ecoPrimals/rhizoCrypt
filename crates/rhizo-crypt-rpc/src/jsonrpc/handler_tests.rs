// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2024–2026 ecoPrimals Project

#![allow(clippy::unwrap_used)]

use super::*;
use crate::jsonrpc::method_gate::{CallerContext, EnforcementMode, MethodGate};
use crate::jsonrpc::types::JsonRpcId;
use crate::jsonrpc::types::JsonRpcRequest;
use rhizo_crypt_core::{PrimalLifecycle, RhizoCryptConfig};
use serde_json::json;

fn test_gate() -> MethodGate {
    MethodGate::with_noop(EnforcementMode::Permissive)
}

fn test_caller() -> CallerContext {
    CallerContext::unix()
}

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
    let result = handle_request(primal.clone(), req, &test_gate(), &test_caller()).await.unwrap();
    let session_id = result.as_str().unwrap();
    assert!(uuid::Uuid::parse_str(session_id).is_ok());

    let req = make_request("dag.session.get", Some(json!({"session_id": session_id})));
    let result = handle_request(primal.clone(), req, &test_gate(), &test_caller()).await.unwrap();
    let info = result.as_object().unwrap();
    assert_eq!(info.get("description").and_then(|v| v.as_str()), Some("test"));

    let req = make_request("dag.session.list", None);
    let result = handle_request(primal.clone(), req, &test_gate(), &test_caller()).await.unwrap();
    let list = result.as_array().unwrap();
    assert_eq!(list.len(), 1);

    let req = make_request("dag.session.discard", Some(json!({"session_id": session_id})));
    let result = handle_request(primal.clone(), req, &test_gate(), &test_caller()).await.unwrap();
    assert!(result.is_null());

    let req = make_request("dag.session.list", None);
    let result = handle_request(primal.clone(), req, &test_gate(), &test_caller()).await.unwrap();
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
    let result = handle_request(primal.clone(), req, &test_gate(), &test_caller()).await.unwrap();
    assert!(result.as_str().is_some());
}

#[tokio::test]
async fn test_session_create_with_parent_session() {
    let primal = create_test_primal().await;

    let req = make_request(
        "dag.session.create",
        Some(json!({"session_type": "General", "description": "parent"})),
    );
    let result = handle_request(primal.clone(), req, &test_gate(), &test_caller()).await.unwrap();
    let parent_id = result.as_str().unwrap().to_string();

    let req = make_request(
        "dag.session.create",
        Some(json!({
            "session_type": "General",
            "description": "child",
            "parent_session": parent_id,
        })),
    );
    let result = handle_request(primal.clone(), req, &test_gate(), &test_caller()).await;
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
    let result = handle_request(primal.clone(), req, &test_gate(), &test_caller()).await;
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
    let result = handle_request(primal.clone(), req, &test_gate(), &test_caller()).await.unwrap();
    let session_id = result.as_str().unwrap();

    let req = make_request(
        "dag.event.append",
        Some(json!({
            "session_id": session_id,
            "event_type": {"SessionStart": null}
        })),
    );
    let result = handle_request(primal.clone(), req, &test_gate(), &test_caller()).await.unwrap();
    let vertex_id_hex = result.as_str().unwrap();
    assert_eq!(vertex_id_hex.len(), 64);
    assert!(hex::decode(vertex_id_hex).is_ok());

    let req = make_request(
        "dag.vertex.get",
        Some(json!({"session_id": session_id, "vertex_id": vertex_id_hex})),
    );
    let result = handle_request(primal.clone(), req, &test_gate(), &test_caller()).await.unwrap();
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
    let result = handle_request(primal.clone(), req, &test_gate(), &test_caller()).await.unwrap();
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
    let result = handle_request(primal.clone(), req, &test_gate(), &test_caller()).await.unwrap();
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
    let result = handle_request(primal.clone(), req, &test_gate(), &test_caller()).await.unwrap();
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
    let result = handle_request(primal.clone(), req, &test_gate(), &test_caller()).await.unwrap();
    assert!(result.as_str().is_some());
}

#[tokio::test]
async fn test_event_append_with_metadata_array() {
    let primal = create_test_primal().await;

    let req = make_request("dag.session.create", Some(json!({"session_type": "General"})));
    let session_id =
        handle_request(primal.clone(), req, &test_gate(), &test_caller()).await.unwrap();
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
    let result = handle_request(primal.clone(), req, &test_gate(), &test_caller()).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_event_append_with_payload_ref() {
    let primal = create_test_primal().await;

    let req = make_request("dag.session.create", Some(json!({"session_type": "General"})));
    let session_id =
        handle_request(primal.clone(), req, &test_gate(), &test_caller()).await.unwrap();
    let session_id = session_id.as_str().unwrap();

    let req = make_request(
        "dag.event.append",
        Some(json!({
            "session_id": session_id,
            "event_type": {"DataCreate": {"schema": "test-schema"}},
            "payload_ref": "ipfs://QmTest123",
        })),
    );
    let result = handle_request(primal.clone(), req, &test_gate(), &test_caller()).await.unwrap();
    let vertex_id = result.as_str().unwrap();

    let req = make_request(
        "dag.vertex.get",
        Some(json!({"session_id": session_id, "vertex_id": vertex_id})),
    );
    let vertex = handle_request(primal.clone(), req, &test_gate(), &test_caller()).await.unwrap();
    assert!(
        vertex.get("payload").is_some(),
        "payload_ref should be applied to vertex, got: {vertex}"
    );
}

#[tokio::test]
async fn test_event_append_with_agent() {
    let primal = create_test_primal().await;

    let req = make_request("dag.session.create", Some(json!({"session_type": "General"})));
    let session_id =
        handle_request(primal.clone(), req, &test_gate(), &test_caller()).await.unwrap();
    let session_id = session_id.as_str().unwrap();

    let req = make_request(
        "dag.event.append",
        Some(json!({
            "session_id": session_id,
            "event_type": {"SessionStart": null},
            "agent": "did:eco:agent:test-001",
        })),
    );
    let result = handle_request(primal.clone(), req, &test_gate(), &test_caller()).await;
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
    let result = handle_request(primal.clone(), req, &test_gate(), &test_caller()).await.unwrap();
    let session_id = result.as_str().unwrap();

    let req = make_request(
        "dag.event.append",
        Some(json!({
            "session_id": session_id,
            "event_type": {"SessionStart": null}
        })),
    );
    let _ = handle_request(primal.clone(), req, &test_gate(), &test_caller()).await.unwrap();

    let req = make_request("dag.frontier.get", Some(json!({"session_id": session_id})));
    let result = handle_request(primal.clone(), req, &test_gate(), &test_caller()).await.unwrap();
    let frontier = result.as_array().unwrap();
    assert_eq!(frontier.len(), 1);

    let req = make_request("dag.genesis.get", Some(json!({"session_id": session_id})));
    let result = handle_request(primal.clone(), req, &test_gate(), &test_caller()).await.unwrap();
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
    let result = handle_request(primal.clone(), req, &test_gate(), &test_caller()).await.unwrap();
    let session_id = result.as_str().unwrap();

    let req = make_request(
        "dag.event.append",
        Some(json!({
            "session_id": session_id,
            "event_type": {"SessionStart": null}
        })),
    );
    let result = handle_request(primal.clone(), req, &test_gate(), &test_caller()).await.unwrap();
    let parent_id = result.as_str().unwrap();

    let req = make_request(
        "dag.event.append",
        Some(json!({
            "session_id": session_id,
            "event_type": {"DataCreate": {"schema": null}},
            "parents": [parent_id]
        })),
    );
    let _ = handle_request(primal.clone(), req, &test_gate(), &test_caller()).await.unwrap();

    let req = make_request(
        "dag.vertex.children",
        Some(json!({"session_id": session_id, "vertex_id": parent_id})),
    );
    let result = handle_request(primal.clone(), req, &test_gate(), &test_caller()).await.unwrap();
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
    let result = handle_request(primal.clone(), req, &test_gate(), &test_caller()).await.unwrap();
    let session_id = result.as_str().unwrap();

    let req = make_request(
        "dag.event.append",
        Some(json!({
            "session_id": session_id,
            "event_type": {"SessionStart": null}
        })),
    );
    let result = handle_request(primal.clone(), req, &test_gate(), &test_caller()).await.unwrap();
    let vertex_id_hex = result.as_str().unwrap();

    let req = make_request("dag.merkle.root", Some(json!({"session_id": session_id})));
    let result = handle_request(primal.clone(), req, &test_gate(), &test_caller()).await.unwrap();
    let root_hex = result.as_str().unwrap();
    assert_eq!(root_hex.len(), 64);

    let req = make_request(
        "dag.merkle.proof",
        Some(json!({"session_id": session_id, "vertex_id": vertex_id_hex})),
    );
    let result = handle_request(primal.clone(), req, &test_gate(), &test_caller()).await.unwrap();
    let proof = result.as_object().unwrap();
    assert!(proof.contains_key("vertex_id"));

    let req = make_request("dag.merkle.verify", Some(json!({"root": root_hex, "proof": proof})));
    let result = handle_request(primal.clone(), req, &test_gate(), &test_caller()).await.unwrap();
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
    let result = handle_request(primal.clone(), req, &test_gate(), &test_caller()).await.unwrap();
    let session_id = result.as_str().unwrap();

    let req = make_request(
        "dag.event.append",
        Some(json!({
            "session_id": session_id,
            "event_type": {"SessionStart": null}
        })),
    );
    let _ = handle_request(primal.clone(), req, &test_gate(), &test_caller()).await.unwrap();

    let req = make_request("dag.dehydration.trigger", Some(json!({"session_id": session_id})));
    let _ = handle_request(primal.clone(), req, &test_gate(), &test_caller()).await.unwrap();

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
    let result = handle_request(primal.clone(), req, &test_gate(), &test_caller()).await.unwrap();
    let slice_id = result.as_str().unwrap();
    assert!(uuid::Uuid::parse_str(slice_id).is_ok());

    let req = make_request("dag.slice.get", Some(json!({"slice_id": slice_id})));
    let result = handle_request(primal.clone(), req, &test_gate(), &test_caller()).await.unwrap();
    let slice = result.as_object().unwrap();
    assert!(slice.contains_key("id") || slice.contains_key("origin"));

    let req = make_request("dag.slice.list", None);
    let result = handle_request(primal.clone(), req, &test_gate(), &test_caller()).await.unwrap();
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
    let result = handle_request(primal.clone(), req, &test_gate(), &test_caller()).await.unwrap();
    let health = result.as_object().unwrap();
    assert!(health.get("healthy").and_then(Value::as_bool).unwrap());
    assert!(health.contains_key("state"));
}

#[tokio::test]
async fn test_health_metrics() {
    let primal = create_test_primal().await;

    let req = make_request("health.metrics", None);
    let result = handle_request(primal.clone(), req, &test_gate(), &test_caller()).await.unwrap();
    let metrics = result.as_object().unwrap();
    assert!(metrics.contains_key("sessions_created"));
    assert!(metrics.contains_key("vertices_appended"));
}

#[tokio::test]
async fn test_capability_list() {
    let primal = create_test_primal().await;

    let req = make_request("capability.list", None);
    let result = handle_request(primal.clone(), req, &test_gate(), &test_caller()).await.unwrap();
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
    let result = handle_request(primal.clone(), req, &test_gate(), &test_caller()).await.unwrap();
    let session_id = result.as_str().unwrap();

    let req = make_request("dag.dehydration.status", Some(json!({"session_id": session_id})));
    let result = handle_request(primal.clone(), req, &test_gate(), &test_caller()).await.unwrap();
    assert!(
        result.as_str().is_some() || result.as_object().is_some(),
        "dehydration.status should return string (unit variant) or object (struct variant)"
    );
}

#[tokio::test]
async fn test_dehydrate_alias_routes_to_trigger() {
    let primal = create_test_primal().await;
    let req = make_request("dag.session.create", Some(json!({"session_type": "General"})));
    let result = handle_request(primal.clone(), req, &test_gate(), &test_caller()).await.unwrap();
    let session_id = result.as_str().unwrap();

    let req_canonical =
        make_request("dag.dehydration.trigger", Some(json!({"session_id": session_id})));
    let result_canonical =
        handle_request(primal.clone(), req_canonical, &test_gate(), &test_caller()).await.unwrap();

    let req_alias = make_request("dag.dehydrate", Some(json!({"session_id": session_id})));
    let result_alias =
        handle_request(primal.clone(), req_alias, &test_gate(), &test_caller()).await.unwrap();

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
    let result = handle_request(primal.clone(), req, &test_gate(), &test_caller()).await.unwrap();
    assert!(uuid::Uuid::parse_str(result.as_str().unwrap()).is_ok());
}

// ============================================================================
// Health endpoint aliases
// ============================================================================

#[tokio::test]
async fn test_health_alias_status() {
    let primal = create_test_primal().await;
    let req = make_request("status", None);
    let result = handle_request(primal.clone(), req, &test_gate(), &test_caller()).await.unwrap();
    assert!(result.get("healthy").is_some());
}

#[tokio::test]
async fn test_health_alias_check() {
    let primal = create_test_primal().await;
    let req = make_request("check", None);
    let result = handle_request(primal.clone(), req, &test_gate(), &test_caller()).await.unwrap();
    assert!(result.get("healthy").is_some());
}

#[tokio::test]
async fn test_health_liveness_alias_ping() {
    let primal = create_test_primal().await;
    let req = make_request("ping", None);
    let result = handle_request(primal.clone(), req, &test_gate(), &test_caller()).await.unwrap();
    assert!(result.get("status").is_some() || result.get("alive").is_some() || result.is_object());
}

#[tokio::test]
async fn test_health_liveness_alias_health() {
    let primal = create_test_primal().await;
    let req = make_request("health", None);
    let result = handle_request(primal.clone(), req, &test_gate(), &test_caller()).await.unwrap();
    assert!(result.is_object());
}

#[tokio::test]
async fn test_health_readiness() {
    let primal = create_test_primal().await;
    let req = make_request("health.readiness", None);
    let result = handle_request(primal.clone(), req, &test_gate(), &test_caller()).await.unwrap();
    assert!(result.is_object());
}

// ============================================================================
// MCP tools.list / tools.call
// ============================================================================

#[tokio::test]
async fn test_mcp_tools_list() {
    let primal = create_test_primal().await;
    let req = make_request("tools.list", None);
    let result = handle_request(primal.clone(), req, &test_gate(), &test_caller()).await.unwrap();
    assert!(result.is_object() || result.is_array());
}

#[tokio::test]
async fn test_mcp_tools_list_alias() {
    let primal = create_test_primal().await;
    let req = make_request("mcp.tools.list", None);
    let result = handle_request(primal.clone(), req, &test_gate(), &test_caller()).await.unwrap();
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
    let result = handle_request(primal.clone(), req, &test_gate(), &test_caller()).await.unwrap();
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
    let result = handle_request(primal.clone(), req, &test_gate(), &test_caller()).await.unwrap();
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
    let result = handle_request(primal.clone(), req, &test_gate(), &test_caller()).await.unwrap();
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
    let err = handle_request(primal.clone(), req, &test_gate(), &test_caller()).await.unwrap_err();
    assert!(matches!(err, HandlerError::MethodNotFound(_)));
}

#[tokio::test]
async fn test_mcp_tools_call_missing_arguments() {
    let primal = create_test_primal().await;
    let req = make_request("tools.call", Some(json!({"name": "status"})));
    let result = handle_request(primal.clone(), req, &test_gate(), &test_caller()).await.unwrap();
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
        let result =
            handle_request(primal.clone(), req, &test_gate(), &test_caller()).await.unwrap();
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
    let result = handle_request(primal.clone(), req, &test_gate(), &test_caller()).await.unwrap();
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
    let err = handle_request(primal.clone(), req, &test_gate(), &test_caller()).await.unwrap_err();
    assert!(matches!(err, HandlerError::Rpc(_)));
}

// ============================================================================
// Readiness gate (PG-60)
// ============================================================================

fn create_unstarted_primal() -> Arc<rhizo_crypt_core::RhizoCrypt> {
    Arc::new(rhizo_crypt_core::RhizoCrypt::new(RhizoCryptConfig::default()))
}

#[tokio::test]
async fn test_readiness_gate_rejects_dag_methods_when_not_running() {
    let primal = create_unstarted_primal();
    assert!(!primal.state().is_running());

    let dag_methods = [
        "dag.session.create",
        "dag.session.get",
        "dag.session.list",
        "dag.event.append",
        "dag.vertex.get",
        "dag.merkle.root",
        "dag.slice.checkout",
        "dag.dehydration.trigger",
    ];

    for method in dag_methods {
        let req = make_request(method, None);
        let err =
            handle_request(primal.clone(), req, &test_gate(), &test_caller()).await.unwrap_err();
        assert!(
            matches!(err, HandlerError::NotReady),
            "{method} should return NotReady when primal is not running"
        );
    }
}

#[tokio::test]
async fn test_readiness_gate_allows_health_probes_when_not_running() {
    let primal = create_unstarted_primal();

    let allowed_methods = [
        "health.liveness",
        "ping",
        "health",
        "health.check",
        "health.readiness",
        "identity.get",
        "capabilities.list",
        "tools.list",
    ];

    for method in allowed_methods {
        let result = handle_request(
            primal.clone(),
            make_request(method, None),
            &test_gate(),
            &test_caller(),
        )
        .await;
        assert!(
            result.is_ok(),
            "{method} should succeed even when primal is not running, got: {result:?}"
        );
    }
}

#[tokio::test]
async fn test_readiness_gate_passes_when_running() {
    let primal = create_test_primal().await;
    assert!(primal.state().is_running());

    let req = make_request(
        "dag.session.create",
        Some(json!({"session_type": "General", "description": "readiness test"})),
    );
    let result = handle_request(primal.clone(), req, &test_gate(), &test_caller()).await;
    assert!(result.is_ok(), "DAG methods should work when primal is running");
}

// =========================================================================
// JH-0 Method Gate Tests
// =========================================================================

#[tokio::test]
async fn test_auth_check_returns_unauthenticated_permissive() {
    let primal = create_test_primal().await;
    let gate = MethodGate::with_noop(EnforcementMode::Permissive);
    let caller = CallerContext::unix();

    let req = make_request("auth.check", None);
    let result = handle_request(primal, req, &gate, &caller).await.unwrap();
    assert_eq!(result["authenticated"], false);
    assert_eq!(result["enforcement"], "permissive");
}

#[tokio::test]
async fn test_auth_mode_returns_current_mode() {
    let primal = create_test_primal().await;
    let gate = MethodGate::with_noop(EnforcementMode::Enforced);
    let caller = CallerContext::unix();

    let req = make_request("auth.mode", None);
    let result = handle_request(primal, req, &gate, &caller).await.unwrap();
    assert_eq!(result["mode"], "enforced");
}

#[tokio::test]
async fn test_auth_peer_info_returns_origin() {
    let primal = create_test_primal().await;
    let gate = test_gate();
    let caller = CallerContext::unix();

    let req = make_request("auth.peer_info", None);
    let result = handle_request(primal, req, &gate, &caller).await.unwrap();
    assert_eq!(result["origin"], "Unix");
    assert_eq!(result["has_token"], false);
}

#[tokio::test]
async fn test_enforced_gate_rejects_protected_without_token() {
    let primal = create_test_primal().await;
    let gate = MethodGate::with_noop(EnforcementMode::Enforced);
    let caller = CallerContext::unix();

    let req = make_request("dag.session.create", Some(json!({"session_type": "General"})));
    let err = handle_request(primal, req, &gate, &caller).await.unwrap_err();
    assert!(
        matches!(err, HandlerError::PermissionDenied(_)),
        "expected PermissionDenied, got: {err:?}"
    );
}

#[tokio::test]
async fn test_enforced_gate_allows_public_without_token() {
    let primal = create_test_primal().await;
    let gate = MethodGate::with_noop(EnforcementMode::Enforced);
    let caller = CallerContext::unix();

    let public_methods = [
        "health.check",
        "health.liveness",
        "identity.get",
        "capabilities.list",
        "auth.check",
        "auth.mode",
        "auth.peer_info",
        "tools.list",
    ];

    for method in public_methods {
        let req = make_request(method, None);
        let result = handle_request(primal.clone(), req, &gate, &caller).await;
        assert!(result.is_ok(), "{method} should be allowed even in enforced mode without a token");
    }
}

#[tokio::test]
async fn test_enforced_gate_allows_protected_with_token() {
    let primal = create_test_primal().await;
    let gate = MethodGate::with_noop(EnforcementMode::Enforced);
    let mut caller = CallerContext::with_bearer_token(
        Some("test-ionic-token".to_owned()),
        crate::jsonrpc::method_gate::ConnectionOrigin::Unix,
    );
    caller.verify_token(gate.verifier());

    let req = make_request("dag.session.create", Some(json!({"session_type": "General"})));
    let result = handle_request(primal, req, &gate, &caller).await;
    assert!(result.is_ok(), "protected method with token should succeed in enforced mode");
}

#[tokio::test]
async fn test_permissive_gate_allows_protected_without_token() {
    let primal = create_test_primal().await;
    let gate = MethodGate::with_noop(EnforcementMode::Permissive);
    let caller = CallerContext::unix();

    let req = make_request("dag.session.create", Some(json!({"session_type": "General"})));
    let result = handle_request(primal, req, &gate, &caller).await;
    assert!(result.is_ok(), "permissive mode should allow unauthenticated protected calls");
}

#[tokio::test]
async fn test_auth_methods_work_when_primal_not_running() {
    let primal = create_unstarted_primal();
    let gate = MethodGate::with_noop(EnforcementMode::Enforced);
    let caller = CallerContext::unix();

    for method in ["auth.check", "auth.mode", "auth.peer_info"] {
        let req = make_request(method, None);
        let result = handle_request(primal.clone(), req, &gate, &caller).await;
        assert!(result.is_ok(), "{method} should work even when primal is not running");
    }
}

// ==========================================================================
// Composition payload tests (provenance trio + JH-5 Phase 3 readiness)
// ==========================================================================

#[tokio::test]
async fn test_composition_skunkbat_security_event_forwarding() {
    let primal = create_test_primal().await;

    let req = make_request(
        "dag.session.create",
        Some(json!({"session_type": "General", "description": "security-audit-pipeline"})),
    );
    let session_id =
        handle_request(primal.clone(), req, &test_gate(), &test_caller()).await.unwrap();
    let session_id = session_id.as_str().unwrap();

    let req = make_request(
        "dag.event.append",
        Some(json!({
            "session_id": session_id,
            "event_type": {"Custom": {"domain": "security", "event_name": "gate_rejection"}},
            "agent": "did:key:z6MkSkunkBat",
            "metadata": [
                {"key": "severity", "value": "high"},
                {"key": "source", "value": "skunkbat"},
                {"key": "method", "value": "dag.session.create"},
                {"key": "correlation_id", "value": "evt-001"}
            ]
        })),
    );
    let result = handle_request(primal.clone(), req, &test_gate(), &test_caller()).await.unwrap();
    assert!(result.as_str().is_some(), "should return vertex_id hex");
    let vertex_id = result.as_str().unwrap();
    assert_eq!(vertex_id.len(), 64);
}

#[tokio::test]
async fn test_composition_security_event_batch_forwarding() {
    let primal = create_test_primal().await;

    let req = make_request(
        "dag.session.create",
        Some(json!({"session_type": "General", "description": "batch-audit"})),
    );
    let session_id =
        handle_request(primal.clone(), req, &test_gate(), &test_caller()).await.unwrap();
    let session_id = session_id.as_str().unwrap();

    let events = vec![
        json!({
            "session_id": session_id,
            "event_type": {"Custom": {"domain": "security", "event_name": "gate_rejection"}},
            "agent": "did:key:z6MkSkunkBat",
            "metadata": [{"key": "severity", "value": "high"}]
        }),
        json!({
            "session_id": session_id,
            "event_type": {"Custom": {"domain": "security", "event_name": "btsp_negotiate"}},
            "agent": "did:key:z6MkSkunkBat",
            "metadata": [{"key": "severity", "value": "info"}]
        }),
        json!({
            "session_id": session_id,
            "event_type": {"Custom": {"domain": "security", "event_name": "threat_detected"}},
            "agent": "did:key:z6MkSkunkBat",
            "metadata": [{"key": "severity", "value": "critical"}]
        }),
    ];

    let req = make_request("dag.event.append_batch", Some(json!({"requests": events})));
    let result = handle_request(primal.clone(), req, &test_gate(), &test_caller()).await.unwrap();
    let ids = result.as_array().unwrap();
    assert_eq!(ids.len(), 3, "batch should return 3 vertex IDs");
    for id in ids {
        assert_eq!(id.as_str().unwrap().len(), 64);
    }
}

#[tokio::test]
async fn test_composition_provenance_trio_full_pipeline() {
    let primal = create_test_primal().await;

    let req = make_request(
        "dag.session.create",
        Some(json!({"session_type": "General", "description": "provenance-trio-pipeline"})),
    );
    let session_id =
        handle_request(primal.clone(), req, &test_gate(), &test_caller()).await.unwrap();
    let session_id = session_id.as_str().unwrap();

    let req = make_request(
        "dag.event.append",
        Some(json!({
            "session_id": session_id,
            "event_type": {"SessionStart": null},
            "agent": "did:key:z6MkOrchestrator"
        })),
    );
    let v1 = handle_request(primal.clone(), req, &test_gate(), &test_caller())
        .await
        .unwrap()
        .as_str()
        .unwrap()
        .to_string();

    let req = make_request(
        "dag.event.append",
        Some(json!({
            "session_id": session_id,
            "event_type": {"DataCreate": {"schema": "rootpulse-commit"}},
            "agent": "did:key:z6MkOrchestrator",
            "parents": [v1],
            "payload_ref": "ipfs://QmRootPulseData123"
        })),
    );
    let v2 = handle_request(primal.clone(), req, &test_gate(), &test_caller())
        .await
        .unwrap()
        .as_str()
        .unwrap()
        .to_string();

    let req = make_request(
        "dag.event.append",
        Some(json!({
            "session_id": session_id,
            "event_type": {"SessionEnd": {"outcome": "Success"}},
            "agent": "did:key:z6MkOrchestrator",
            "parents": [v2]
        })),
    );
    let _ = handle_request(primal.clone(), req, &test_gate(), &test_caller()).await.unwrap();

    let req = make_request("dag.merkle.root", Some(json!({"session_id": session_id})));
    let root = handle_request(primal.clone(), req, &test_gate(), &test_caller()).await.unwrap();
    let root_hex = root.as_str().unwrap();
    assert_eq!(root_hex.len(), 64, "Merkle root should be 64-char hex (32 bytes)");

    let session_req = make_request("dag.session.get", Some(json!({"session_id": session_id})));
    let info =
        handle_request(primal.clone(), session_req, &test_gate(), &test_caller()).await.unwrap();
    assert_eq!(info["vertex_count"], 3);
}

#[tokio::test]
async fn test_composition_payload_ref_hex_hash() {
    let primal = create_test_primal().await;

    let req = make_request("dag.session.create", Some(json!({"session_type": "General"})));
    let session_id =
        handle_request(primal.clone(), req, &test_gate(), &test_caller()).await.unwrap();
    let session_id = session_id.as_str().unwrap();

    let hex_hash = "a".repeat(64);
    let req = make_request(
        "dag.event.append",
        Some(json!({
            "session_id": session_id,
            "event_type": {"DataCreate": {"schema": null}},
            "payload_ref": hex_hash
        })),
    );
    let result = handle_request(primal.clone(), req, &test_gate(), &test_caller()).await.unwrap();
    let vertex_id = result.as_str().unwrap();

    let req = make_request(
        "dag.vertex.get",
        Some(json!({"session_id": session_id, "vertex_id": vertex_id})),
    );
    let vertex = handle_request(primal.clone(), req, &test_gate(), &test_caller()).await.unwrap();
    let payload = vertex.get("payload").unwrap();
    let payload_hash = payload["hash"].as_array().unwrap();
    assert_eq!(payload_hash.len(), 32, "payload hash should be 32 bytes");
    assert_eq!(payload["size"], 0, "hex-parsed payload_ref has unknown size");
}

#[tokio::test]
async fn test_composition_dehydrate_produces_loamspine_compatible_root() {
    let primal = create_test_primal().await;

    let req = make_request("dag.session.create", Some(json!({"session_type": "General"})));
    let session_id =
        handle_request(primal.clone(), req, &test_gate(), &test_caller()).await.unwrap();
    let session_id = session_id.as_str().unwrap();

    let req = make_request(
        "dag.event.append",
        Some(json!({
            "session_id": session_id,
            "event_type": {"DataCreate": {"schema": "test"}},
            "agent": "did:key:z6MkCommitter"
        })),
    );
    let _ = handle_request(primal.clone(), req, &test_gate(), &test_caller()).await.unwrap();

    let req = make_request("dag.dehydration.trigger", Some(json!({"session_id": session_id})));
    let root = handle_request(primal.clone(), req, &test_gate(), &test_caller()).await.unwrap();
    let root_hex = root.as_str().unwrap();
    assert_eq!(root_hex.len(), 64, "dehydration root should be 64-char hex");
    assert!(
        hex::decode(root_hex).is_ok(),
        "dehydration root must be valid hex for loamSpine session_hash"
    );

    let req = make_request("dag.merkle.root", Some(json!({"session_id": session_id})));
    let merkle_root =
        handle_request(primal.clone(), req, &test_gate(), &test_caller()).await.unwrap();
    assert_eq!(
        root_hex,
        merkle_root.as_str().unwrap(),
        "dehydration root should equal Merkle root"
    );
}

// ==========================================================================
// GAP-36: provenance.* wire-name alias tests
// ==========================================================================

#[tokio::test]
async fn test_provenance_session_create_alias() {
    let primal = create_test_primal().await;

    let req = make_request(
        "provenance.session.create",
        Some(json!({"session_type": "General", "description": "alias-test"})),
    );
    let result = handle_request(primal.clone(), req, &test_gate(), &test_caller()).await;
    assert!(result.is_ok(), "provenance.session.create should route to dag.session.create");
    let session_id = result.unwrap();
    assert!(session_id.as_str().is_some(), "should return session_id string");
}

#[tokio::test]
async fn test_provenance_event_append_alias() {
    let primal = create_test_primal().await;

    let req = make_request("provenance.session.create", Some(json!({"session_type": "General"})));
    let session_id =
        handle_request(primal.clone(), req, &test_gate(), &test_caller()).await.unwrap();
    let session_id = session_id.as_str().unwrap();

    let req = make_request(
        "provenance.event.append",
        Some(json!({
            "session_id": session_id,
            "event_type": {"Custom": {"domain": "security", "event_name": "audit"}},
            "agent": "did:key:z6MkTest"
        })),
    );
    let result = handle_request(primal.clone(), req, &test_gate(), &test_caller()).await;
    assert!(result.is_ok(), "provenance.event.append should route to dag.event.append");
    let vertex_id = result.unwrap();
    assert_eq!(vertex_id.as_str().unwrap().len(), 64);
}

#[tokio::test]
async fn test_provenance_dehydrate_alias() {
    let primal = create_test_primal().await;

    let req = make_request("provenance.session.create", Some(json!({"session_type": "General"})));
    let session_id =
        handle_request(primal.clone(), req, &test_gate(), &test_caller()).await.unwrap();
    let session_id = session_id.as_str().unwrap();

    let req = make_request(
        "provenance.event.append",
        Some(json!({
            "session_id": session_id,
            "event_type": {"SessionStart": null}
        })),
    );
    let _ = handle_request(primal.clone(), req, &test_gate(), &test_caller()).await.unwrap();

    let req =
        make_request("provenance.dehydration.trigger", Some(json!({"session_id": session_id})));
    let root = handle_request(primal.clone(), req, &test_gate(), &test_caller()).await;
    assert!(root.is_ok(), "provenance.dehydration.trigger should route to dag.dehydration.trigger");
    assert_eq!(root.unwrap().as_str().unwrap().len(), 64);
}

#[tokio::test]
async fn test_provenance_full_pipeline_via_aliases() {
    let primal = create_test_primal().await;

    let req = make_request("provenance.session.create", Some(json!({"session_type": "General"})));
    let session_id =
        handle_request(primal.clone(), req, &test_gate(), &test_caller()).await.unwrap();
    let session_id = session_id.as_str().unwrap();

    let req = make_request(
        "provenance.event.append",
        Some(json!({
            "session_id": session_id,
            "event_type": {"DataCreate": {"schema": "clinical-data"}},
            "agent": "did:key:z6MkHealthSpring"
        })),
    );
    let _ = handle_request(primal.clone(), req, &test_gate(), &test_caller()).await.unwrap();

    let req = make_request("provenance.merkle.root", Some(json!({"session_id": session_id})));
    let root = handle_request(primal.clone(), req, &test_gate(), &test_caller()).await.unwrap();
    assert_eq!(root.as_str().unwrap().len(), 64);

    let req = make_request("provenance.session.get", Some(json!({"session_id": session_id})));
    let info = handle_request(primal.clone(), req, &test_gate(), &test_caller()).await.unwrap();
    assert_eq!(info["vertex_count"], 1);

    let req = make_request("provenance.session.list", None);
    let list = handle_request(primal.clone(), req, &test_gate(), &test_caller()).await.unwrap();
    assert!(!list.as_array().unwrap().is_empty());
}

#[tokio::test]
async fn test_session_get_returns_summary_fields() {
    let primal = create_test_primal().await;

    let req = make_request(
        "dag.session.create",
        Some(json!({"session_type": "General", "description": "summary-test"})),
    );
    let session_id =
        handle_request(primal.clone(), req, &test_gate(), &test_caller()).await.unwrap();
    let session_id = session_id.as_str().unwrap();

    let req = make_request(
        "dag.event.append",
        Some(json!({
            "session_id": session_id,
            "event_type": {"DataCreate": {"schema": "cathedral"}},
            "agent": "did:key:z6MkCATHEDRAL"
        })),
    );
    let _ = handle_request(primal.clone(), req, &test_gate(), &test_caller()).await.unwrap();

    let req = make_request(
        "dag.event.append",
        Some(json!({
            "session_id": session_id,
            "event_type": {"DataCreate": {"schema": "litho"}},
            "agent": "did:key:z6MkLithoSpore"
        })),
    );
    let _ = handle_request(primal.clone(), req, &test_gate(), &test_caller()).await.unwrap();

    let req = make_request("dag.session.get", Some(json!({"session_id": session_id})));
    let info = handle_request(primal.clone(), req, &test_gate(), &test_caller()).await.unwrap();

    assert_eq!(info["vertex_count"], 2);
    assert_eq!(info["description"], "summary-test");

    let agents = info["agents"].as_array().unwrap();
    assert!(
        agents.iter().any(|a| a.as_str() == Some("did:key:z6MkCATHEDRAL")),
        "agents should include did:key:z6MkCATHEDRAL, got: {agents:?}"
    );
    assert!(
        agents.iter().any(|a| a.as_str() == Some("did:key:z6MkLithoSpore")),
        "agents should include did:key:z6MkLithoSpore, got: {agents:?}"
    );

    let genesis = info["genesis"].as_array().unwrap();
    assert!(!genesis.is_empty(), "genesis should have at least one root vertex");

    let frontier = info["frontier"].as_array().unwrap();
    assert!(!frontier.is_empty(), "frontier should have tip vertices");
}

// ==========================================================================
// dag.partial_dehydrate (wetSpring upstream ask)
// ==========================================================================

#[tokio::test]
async fn test_partial_dehydrate_all_vertices() {
    let primal = create_test_primal().await;

    let req = make_request("dag.session.create", Some(json!({"session_type": "General"})));
    let session_id =
        handle_request(primal.clone(), req, &test_gate(), &test_caller()).await.unwrap();
    let session_id = session_id.as_str().unwrap();

    for i in 0..3 {
        let req = make_request(
            "dag.event.append",
            Some(json!({
                "session_id": session_id,
                "event_type": {"DataCreate": {"schema": format!("clone-{i}")}},
                "agent": "did:key:z6MkWetSpring"
            })),
        );
        let _ = handle_request(primal.clone(), req, &test_gate(), &test_caller()).await.unwrap();
    }

    let req = make_request("dag.partial_dehydrate", Some(json!({"session_id": session_id})));
    let resp = handle_request(primal.clone(), req, &test_gate(), &test_caller()).await.unwrap();

    assert_eq!(resp["sealed_count"], 3);
    assert_eq!(resp["open_count"], 0);
    assert!(resp["session_open"].as_bool().unwrap());
    assert_eq!(resp["merkle_root"].as_str().unwrap().len(), 64);

    let req = make_request("dag.merkle.root", Some(json!({"session_id": session_id})));
    let full_root =
        handle_request(primal.clone(), req, &test_gate(), &test_caller()).await.unwrap();
    assert_eq!(
        resp["merkle_root"].as_str().unwrap(),
        full_root.as_str().unwrap(),
        "partial_dehydrate with no filter should match full merkle root"
    );
}

#[tokio::test]
async fn test_partial_dehydrate_subset() {
    let primal = create_test_primal().await;

    let req = make_request("dag.session.create", Some(json!({"session_type": "General"})));
    let session_id =
        handle_request(primal.clone(), req, &test_gate(), &test_caller()).await.unwrap();
    let session_id = session_id.as_str().unwrap();

    let mut vertex_ids = Vec::new();
    for i in 0..3 {
        let req = make_request(
            "dag.event.append",
            Some(json!({
                "session_id": session_id,
                "event_type": {"DataCreate": {"schema": format!("clone-{i}")}},
            })),
        );
        let vid = handle_request(primal.clone(), req, &test_gate(), &test_caller()).await.unwrap();
        vertex_ids.push(vid.as_str().unwrap().to_owned());
    }

    let req = make_request(
        "dag.partial_dehydrate",
        Some(json!({
            "session_id": session_id,
            "vertex_ids": [vertex_ids[0], vertex_ids[1]]
        })),
    );
    let resp = handle_request(primal.clone(), req, &test_gate(), &test_caller()).await.unwrap();

    assert_eq!(resp["sealed_count"], 2);
    assert_eq!(resp["open_count"], 1);
    assert!(resp["session_open"].as_bool().unwrap());
    assert_eq!(resp["merkle_root"].as_str().unwrap().len(), 64);
}

#[tokio::test]
async fn test_partial_dehydrate_does_not_close_session() {
    let primal = create_test_primal().await;

    let req = make_request("dag.session.create", Some(json!({"session_type": "General"})));
    let session_id =
        handle_request(primal.clone(), req, &test_gate(), &test_caller()).await.unwrap();
    let session_id = session_id.as_str().unwrap();

    let req = make_request(
        "dag.event.append",
        Some(json!({"session_id": session_id, "event_type": {"SessionStart": null}})),
    );
    let _ = handle_request(primal.clone(), req, &test_gate(), &test_caller()).await.unwrap();

    let req = make_request("dag.partial_dehydrate", Some(json!({"session_id": session_id})));
    let _ = handle_request(primal.clone(), req, &test_gate(), &test_caller()).await.unwrap();

    let req = make_request(
        "dag.event.append",
        Some(json!({
            "session_id": session_id,
            "event_type": {"DataCreate": {"schema": "post-partial"}},
        })),
    );
    let result = handle_request(primal.clone(), req, &test_gate(), &test_caller()).await;
    assert!(result.is_ok(), "should still append after partial_dehydrate");
}

#[tokio::test]
async fn test_partial_dehydrate_via_provenance_alias() {
    let primal = create_test_primal().await;

    let req = make_request("provenance.session.create", Some(json!({"session_type": "General"})));
    let session_id =
        handle_request(primal.clone(), req, &test_gate(), &test_caller()).await.unwrap();
    let session_id = session_id.as_str().unwrap();

    let req = make_request(
        "provenance.event.append",
        Some(json!({
            "session_id": session_id,
            "event_type": {"DataCreate": {"schema": "aglet-test"}},
        })),
    );
    let _ = handle_request(primal.clone(), req, &test_gate(), &test_caller()).await.unwrap();

    let req = make_request("provenance.partial_dehydrate", Some(json!({"session_id": session_id})));
    let resp = handle_request(primal.clone(), req, &test_gate(), &test_caller()).await;
    assert!(resp.is_ok(), "provenance.partial_dehydrate should alias to dag.partial_dehydrate");
    assert_eq!(resp.unwrap()["sealed_count"], 1);
}
