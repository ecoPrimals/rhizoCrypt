// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2024–2026 ecoPrimals Project

//! JSON-RPC 2.0 method handler.
//!
//! Dispatches semantic method names to RhizoCryptRpcServer operations.

use crate::error::RpcError;
use crate::jsonrpc::types::JsonRpcRequest;
use crate::service::{
    AppendEventRequest, CheckoutSliceRequest, CreateSessionRequest, QueryRequest, RhizoCryptRpc,
    RhizoCryptRpcServer,
};
use rhizo_crypt_core::{MerkleRoot, SessionId, SliceId, SliceMode, VertexId};
use serde_json::{json, Value};
use std::sync::Arc;
use thiserror::Error;
use tracing::debug;

/// Handler error for JSON-RPC.
#[derive(Debug, Error)]
pub enum HandlerError {
    /// Invalid parameters.
    #[error("invalid params: {0}")]
    InvalidParams(String),

    /// Method not found.
    #[error("method not found: {0}")]
    MethodNotFound(String),

    /// RPC error from service.
    #[error("rpc error: {0}")]
    Rpc(#[from] RpcError),
}

/// Handle a single JSON-RPC request.
pub async fn handle_request(
    primal: Arc<rhizo_crypt_core::RhizoCrypt>,
    request: JsonRpcRequest,
) -> Result<Value, HandlerError> {
    let server = RhizoCryptRpcServer::new(primal);
    let method = request.method.as_str();
    let params = request.params.unwrap_or(Value::Null);

    debug!(method = %method, "JSON-RPC request");

    let result = match method {
        "dag.session.create" => dispatch_session_create(&server, params).await,
        "dag.session.get" => dispatch_session_get(&server, params).await,
        "dag.session.list" => dispatch_session_list(&server).await,
        "dag.session.discard" => dispatch_session_discard(&server, params).await,
        "dag.event.append" => dispatch_event_append(&server, params).await,
        "dag.event.append_batch" => dispatch_event_append_batch(&server, params).await,
        "dag.vertex.get" => dispatch_vertex_get(&server, params).await,
        "dag.frontier.get" => dispatch_frontier_get(&server, params).await,
        "dag.genesis.get" => dispatch_genesis_get(&server, params).await,
        "dag.vertex.query" => dispatch_vertex_query(&server, params).await,
        "dag.vertex.children" => dispatch_vertex_children(&server, params).await,
        "dag.merkle.root" => dispatch_merkle_root(&server, params).await,
        "dag.merkle.proof" => dispatch_merkle_proof(&server, params).await,
        "dag.merkle.verify" => dispatch_merkle_verify(&server, params).await,
        "dag.slice.checkout" => dispatch_slice_checkout(&server, params).await,
        "dag.slice.get" => dispatch_slice_get(&server, params).await,
        "dag.slice.list" => dispatch_slice_list(&server).await,
        "dag.slice.resolve" => dispatch_slice_resolve(&server, params).await,
        "dag.dehydrate" => dispatch_dehydrate(&server, params).await,
        "dag.dehydrate.status" => dispatch_dehydrate_status(&server, params).await,
        "system.health" => dispatch_health(&server).await,
        "system.metrics" => dispatch_metrics(&server).await,
        _ => return Err(HandlerError::MethodNotFound(request.method)),
    };

    result
}

fn get_obj(params: &Value) -> Result<&serde_json::Map<String, Value>, HandlerError> {
    params
        .as_object()
        .ok_or_else(|| HandlerError::InvalidParams("params must be an object".to_string()))
}

fn get_str(obj: &serde_json::Map<String, Value>, key: &str) -> Result<String, HandlerError> {
    obj.get(key)
        .and_then(Value::as_str)
        .map(String::from)
        .ok_or_else(|| HandlerError::InvalidParams(format!("missing or invalid '{key}'")))
}

fn get_opt_str(obj: &serde_json::Map<String, Value>, key: &str) -> Option<String> {
    obj.get(key).and_then(Value::as_str).map(String::from)
}

fn parse_session_id(s: &str) -> Result<SessionId, HandlerError> {
    uuid::Uuid::parse_str(s)
        .map(SessionId::new)
        .map_err(|e| HandlerError::InvalidParams(format!("invalid session_id: {e}")))
}

fn parse_vertex_id(s: &str) -> Result<VertexId, HandlerError> {
    let bytes = hex::decode(s)
        .map_err(|e| HandlerError::InvalidParams(format!("invalid vertex_id hex: {e}")))?;
    if bytes.len() != 32 {
        return Err(HandlerError::InvalidParams("vertex_id must be 32 bytes hex".to_string()));
    }
    let mut arr = [0u8; 32];
    arr.copy_from_slice(&bytes);
    Ok(VertexId(arr))
}

fn parse_slice_id(s: &str) -> Result<SliceId, HandlerError> {
    uuid::Uuid::parse_str(s)
        .map(SliceId::new)
        .map_err(|e| HandlerError::InvalidParams(format!("invalid slice_id: {e}")))
}

fn parse_did(s: &str) -> rhizo_crypt_core::Did {
    rhizo_crypt_core::Did::new(s)
}

fn vertex_id_to_value(id: VertexId) -> Value {
    json!(hex::encode(id.as_bytes()))
}

fn session_id_to_value(id: SessionId) -> Value {
    json!(id.as_uuid().to_string())
}

fn slice_id_to_value(id: SliceId) -> Value {
    json!(id.0.to_string())
}

async fn dispatch_session_create(
    server: &RhizoCryptRpcServer,
    params: Value,
) -> Result<Value, HandlerError> {
    let obj = get_obj(&params)?;
    let session_type = obj
        .get("session_type")
        .and_then(|v| serde_json::from_value(v.clone()).ok())
        .unwrap_or_default();
    let description = get_opt_str(obj, "description");
    let parent_session =
        get_opt_str(obj, "parent_session").map(|s| parse_session_id(&s)).transpose()?;
    let max_vertices = obj.get("max_vertices").and_then(Value::as_u64);
    let ttl_seconds = obj.get("ttl_seconds").and_then(Value::as_u64);

    let req = CreateSessionRequest {
        session_type,
        description,
        parent_session,
        max_vertices,
        ttl_seconds,
    };
    let id = server.clone().create_session(tarpc::context::current(), req).await?;
    Ok(session_id_to_value(id))
}

async fn dispatch_session_get(
    server: &RhizoCryptRpcServer,
    params: Value,
) -> Result<Value, HandlerError> {
    let obj = get_obj(&params)?;
    let session_id = parse_session_id(&get_str(obj, "session_id")?)?;
    let info = server.clone().get_session(tarpc::context::current(), session_id).await?;
    serde_json::to_value(&info).map_err(|e| HandlerError::InvalidParams(e.to_string()))
}

async fn dispatch_session_list(server: &RhizoCryptRpcServer) -> Result<Value, HandlerError> {
    let list = server.clone().list_sessions(tarpc::context::current()).await?;
    serde_json::to_value(&list).map_err(|e| HandlerError::InvalidParams(e.to_string()))
}

async fn dispatch_session_discard(
    server: &RhizoCryptRpcServer,
    params: Value,
) -> Result<Value, HandlerError> {
    let obj = get_obj(&params)?;
    let session_id = parse_session_id(&get_str(obj, "session_id")?)?;
    server.clone().discard_session(tarpc::context::current(), session_id).await?;
    Ok(Value::Null)
}

async fn dispatch_event_append(
    server: &RhizoCryptRpcServer,
    params: Value,
) -> Result<Value, HandlerError> {
    let obj = get_obj(&params)?;
    let session_id = parse_session_id(&get_str(obj, "session_id")?)?;
    let event_type = serde_json::from_value(obj.get("event_type").cloned().unwrap_or(Value::Null))
        .map_err(|e| HandlerError::InvalidParams(format!("event_type: {e}")))?;
    let agent = get_opt_str(obj, "agent").as_deref().map(parse_did);
    let parents: Vec<VertexId> = obj
        .get("parents")
        .and_then(Value::as_array)
        .unwrap_or(&vec![])
        .iter()
        .filter_map(|v| v.as_str())
        .map(parse_vertex_id)
        .collect::<Result<Vec<_>, _>>()?;
    let metadata: Vec<(String, String)> = obj
        .get("metadata")
        .and_then(Value::as_array)
        .unwrap_or(&vec![])
        .iter()
        .filter_map(|v| v.as_object())
        .filter_map(|o| {
            let k = o.get("key")?.as_str()?.to_string();
            let v = o.get("value")?.as_str()?.to_string();
            Some((k, v))
        })
        .collect();
    let payload_ref = get_opt_str(obj, "payload_ref");

    let req = AppendEventRequest {
        session_id,
        event_type,
        agent,
        parents,
        metadata,
        payload_ref,
    };
    let id = server.clone().append_event(tarpc::context::current(), req).await?;
    Ok(vertex_id_to_value(id))
}

async fn dispatch_event_append_batch(
    server: &RhizoCryptRpcServer,
    params: Value,
) -> Result<Value, HandlerError> {
    let obj = get_obj(&params)?;
    let requests_arr = obj
        .get("requests")
        .and_then(Value::as_array)
        .ok_or_else(|| HandlerError::InvalidParams("missing 'requests' array".to_string()))?;
    let mut requests = Vec::with_capacity(requests_arr.len());
    for r in requests_arr {
        let r_obj = r.as_object().ok_or_else(|| {
            HandlerError::InvalidParams("each request must be an object".to_string())
        })?;
        let session_id = parse_session_id(&get_str(r_obj, "session_id")?)?;
        let event_type =
            serde_json::from_value(r_obj.get("event_type").cloned().unwrap_or(Value::Null))
                .map_err(|e| HandlerError::InvalidParams(format!("event_type: {e}")))?;
        let agent = get_opt_str(r_obj, "agent").as_deref().map(parse_did);
        let parents: Vec<VertexId> = r_obj
            .get("parents")
            .and_then(Value::as_array)
            .unwrap_or(&vec![])
            .iter()
            .filter_map(|v| v.as_str())
            .map(parse_vertex_id)
            .collect::<Result<Vec<_>, _>>()?;
        let metadata: Vec<(String, String)> = r_obj
            .get("metadata")
            .and_then(Value::as_array)
            .unwrap_or(&vec![])
            .iter()
            .filter_map(|v| v.as_object())
            .filter_map(|o| {
                let k = o.get("key")?.as_str()?.to_string();
                let v = o.get("value")?.as_str()?.to_string();
                Some((k, v))
            })
            .collect();
        let payload_ref = get_opt_str(r_obj, "payload_ref");
        requests.push(AppendEventRequest {
            session_id,
            event_type,
            agent,
            parents,
            metadata,
            payload_ref,
        });
    }
    let ids = server.clone().append_batch(tarpc::context::current(), requests).await?;
    serde_json::to_value(ids.iter().map(|id| vertex_id_to_value(*id)).collect::<Vec<_>>())
        .map_err(|e| HandlerError::InvalidParams(e.to_string()))
}

async fn dispatch_vertex_get(
    server: &RhizoCryptRpcServer,
    params: Value,
) -> Result<Value, HandlerError> {
    let obj = get_obj(&params)?;
    let session_id = parse_session_id(&get_str(obj, "session_id")?)?;
    let vertex_id = parse_vertex_id(&get_str(obj, "vertex_id")?)?;
    let vertex =
        server.clone().get_vertex(tarpc::context::current(), session_id, vertex_id).await?;
    serde_json::to_value(&vertex).map_err(|e| HandlerError::InvalidParams(e.to_string()))
}

async fn dispatch_frontier_get(
    server: &RhizoCryptRpcServer,
    params: Value,
) -> Result<Value, HandlerError> {
    let obj = get_obj(&params)?;
    let session_id = parse_session_id(&get_str(obj, "session_id")?)?;
    let frontier = server.clone().get_frontier(tarpc::context::current(), session_id).await?;
    serde_json::to_value(frontier.iter().map(|id| vertex_id_to_value(*id)).collect::<Vec<_>>())
        .map_err(|e| HandlerError::InvalidParams(e.to_string()))
}

async fn dispatch_genesis_get(
    server: &RhizoCryptRpcServer,
    params: Value,
) -> Result<Value, HandlerError> {
    let obj = get_obj(&params)?;
    let session_id = parse_session_id(&get_str(obj, "session_id")?)?;
    let genesis = server.clone().get_genesis(tarpc::context::current(), session_id).await?;
    serde_json::to_value(genesis.iter().map(|id| vertex_id_to_value(*id)).collect::<Vec<_>>())
        .map_err(|e| HandlerError::InvalidParams(e.to_string()))
}

async fn dispatch_vertex_query(
    server: &RhizoCryptRpcServer,
    params: Value,
) -> Result<Value, HandlerError> {
    let obj = get_obj(&params)?;
    let session_id = parse_session_id(&get_str(obj, "session_id")?)?;
    let event_types = obj.get("event_types").and_then(|v| serde_json::from_value(v.clone()).ok());
    let agent = get_opt_str(obj, "agent").as_deref().map(parse_did);
    let start_time = obj.get("start_time").and_then(|v| serde_json::from_value(v.clone()).ok());
    let end_time = obj.get("end_time").and_then(|v| serde_json::from_value(v.clone()).ok());
    let limit = obj.get("limit").and_then(Value::as_u64).and_then(|u| u32::try_from(u).ok());

    let req = QueryRequest {
        session_id,
        event_types,
        agent,
        start_time,
        end_time,
        limit,
    };
    let vertices = server.clone().query_vertices(tarpc::context::current(), req).await?;
    serde_json::to_value(&vertices).map_err(|e| HandlerError::InvalidParams(e.to_string()))
}

async fn dispatch_vertex_children(
    server: &RhizoCryptRpcServer,
    params: Value,
) -> Result<Value, HandlerError> {
    let obj = get_obj(&params)?;
    let session_id = parse_session_id(&get_str(obj, "session_id")?)?;
    let vertex_id = parse_vertex_id(&get_str(obj, "vertex_id")?)?;
    let children =
        server.clone().get_children(tarpc::context::current(), session_id, vertex_id).await?;
    serde_json::to_value(children.iter().map(|id| vertex_id_to_value(*id)).collect::<Vec<_>>())
        .map_err(|e| HandlerError::InvalidParams(e.to_string()))
}

async fn dispatch_merkle_root(
    server: &RhizoCryptRpcServer,
    params: Value,
) -> Result<Value, HandlerError> {
    let obj = get_obj(&params)?;
    let session_id = parse_session_id(&get_str(obj, "session_id")?)?;
    let root = server.clone().get_merkle_root(tarpc::context::current(), session_id).await?;
    Ok(json!(hex::encode(root.0)))
}

async fn dispatch_merkle_proof(
    server: &RhizoCryptRpcServer,
    params: Value,
) -> Result<Value, HandlerError> {
    let obj = get_obj(&params)?;
    let session_id = parse_session_id(&get_str(obj, "session_id")?)?;
    let vertex_id = parse_vertex_id(&get_str(obj, "vertex_id")?)?;
    let proof =
        server.clone().get_merkle_proof(tarpc::context::current(), session_id, vertex_id).await?;
    serde_json::to_value(&proof).map_err(|e| HandlerError::InvalidParams(e.to_string()))
}

async fn dispatch_merkle_verify(
    server: &RhizoCryptRpcServer,
    params: Value,
) -> Result<Value, HandlerError> {
    let obj = get_obj(&params)?;
    let root_hex = get_str(obj, "root")?;
    let root_bytes =
        hex::decode(&root_hex).map_err(|e| HandlerError::InvalidParams(format!("root: {e}")))?;
    if root_bytes.len() != 32 {
        return Err(HandlerError::InvalidParams("root must be 32 bytes hex".to_string()));
    }
    let mut root_arr = [0u8; 32];
    root_arr.copy_from_slice(&root_bytes);
    let root = MerkleRoot(root_arr);
    let proof = serde_json::from_value(obj.get("proof").cloned().unwrap_or(Value::Null))
        .map_err(|e| HandlerError::InvalidParams(format!("proof: {e}")))?;
    let ok = server.clone().verify_proof(tarpc::context::current(), root, proof).await?;
    Ok(json!(ok))
}

async fn dispatch_slice_checkout(
    server: &RhizoCryptRpcServer,
    params: Value,
) -> Result<Value, HandlerError> {
    let obj = get_obj(&params)?;
    let spine_index = obj
        .get("spine_index")
        .and_then(Value::as_u64)
        .ok_or_else(|| HandlerError::InvalidParams("missing spine_index".to_string()))?;
    let mode = serde_json::from_value(obj.get("mode").cloned().unwrap_or(Value::Null)).unwrap_or(
        SliceMode::Copy {
            allow_recopy: false,
        },
    );
    let lender = get_opt_str(obj, "lender").as_deref().map(parse_did);
    let borrower = get_opt_str(obj, "borrower").as_deref().map(parse_did);
    let duration_seconds = obj.get("duration_seconds").and_then(Value::as_u64);

    let req = CheckoutSliceRequest {
        spine_index,
        mode,
        lender,
        borrower,
        duration_seconds,
    };
    let id = server.clone().checkout_slice(tarpc::context::current(), req).await?;
    Ok(slice_id_to_value(id))
}

async fn dispatch_slice_get(
    server: &RhizoCryptRpcServer,
    params: Value,
) -> Result<Value, HandlerError> {
    let obj = get_obj(&params)?;
    let slice_id = parse_slice_id(&get_str(obj, "slice_id")?)?;
    let slice = server.clone().get_slice(tarpc::context::current(), slice_id).await?;
    serde_json::to_value(&slice).map_err(|e| HandlerError::InvalidParams(e.to_string()))
}

async fn dispatch_slice_list(server: &RhizoCryptRpcServer) -> Result<Value, HandlerError> {
    let list = server.clone().list_slices(tarpc::context::current()).await?;
    serde_json::to_value(&list).map_err(|e| HandlerError::InvalidParams(e.to_string()))
}

async fn dispatch_slice_resolve(
    server: &RhizoCryptRpcServer,
    params: Value,
) -> Result<Value, HandlerError> {
    let obj = get_obj(&params)?;
    let slice_id = parse_slice_id(&get_str(obj, "slice_id")?)?;
    let session_id = parse_session_id(&get_str(obj, "session_id")?)?;
    server.clone().resolve_slice(tarpc::context::current(), slice_id, session_id).await?;
    Ok(Value::Null)
}

async fn dispatch_dehydrate(
    server: &RhizoCryptRpcServer,
    params: Value,
) -> Result<Value, HandlerError> {
    let obj = get_obj(&params)?;
    let session_id = parse_session_id(&get_str(obj, "session_id")?)?;
    let root = server.clone().dehydrate(tarpc::context::current(), session_id).await?;
    Ok(json!(hex::encode(root.0)))
}

async fn dispatch_dehydrate_status(
    server: &RhizoCryptRpcServer,
    params: Value,
) -> Result<Value, HandlerError> {
    let obj = get_obj(&params)?;
    let session_id = parse_session_id(&get_str(obj, "session_id")?)?;
    let status =
        server.clone().get_dehydration_status(tarpc::context::current(), session_id).await?;
    serde_json::to_value(&status).map_err(|e| HandlerError::InvalidParams(e.to_string()))
}

async fn dispatch_health(server: &RhizoCryptRpcServer) -> Result<Value, HandlerError> {
    let status = server.clone().health(tarpc::context::current()).await?;
    serde_json::to_value(&status).map_err(|e| HandlerError::InvalidParams(e.to_string()))
}

async fn dispatch_metrics(server: &RhizoCryptRpcServer) -> Result<Value, HandlerError> {
    let metrics = server.clone().metrics(tarpc::context::current()).await?;
    serde_json::to_value(&metrics).map_err(|e| HandlerError::InvalidParams(e.to_string()))
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
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

        // Create session
        let req = make_request(
            "dag.session.create",
            Some(json!({"session_type": "General", "description": "test"})),
        );
        let result = handle_request(primal.clone(), req).await.unwrap();
        let session_id = result.as_str().unwrap();
        assert!(uuid::Uuid::parse_str(session_id).is_ok());

        // Get session
        let req = make_request("dag.session.get", Some(json!({"session_id": session_id})));
        let result = handle_request(primal.clone(), req).await.unwrap();
        let info = result.as_object().unwrap();
        assert_eq!(info.get("description").and_then(|v| v.as_str()), Some("test"));

        // List sessions
        let req = make_request("dag.session.list", None);
        let result = handle_request(primal.clone(), req).await.unwrap();
        let list = result.as_array().unwrap();
        assert_eq!(list.len(), 1);

        // Discard session
        let req = make_request("dag.session.discard", Some(json!({"session_id": session_id})));
        let result = handle_request(primal.clone(), req).await.unwrap();
        assert!(result.is_null());

        // List should be empty
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

        let req =
            make_request("dag.merkle.verify", Some(json!({"root": root_hex, "proof": proof})));
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
}
