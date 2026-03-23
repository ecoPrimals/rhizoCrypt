// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2024–2026 ecoPrimals Project

//! JSON-RPC 2.0 method handler.
//!
//! Dispatches semantic method names to `RhizoCryptRpcServer` operations.

use crate::error::RpcError;
use crate::jsonrpc::types::JsonRpcRequest;
use crate::service::{
    AppendEventRequest, CheckoutSliceRequest, CreateSessionRequest, QueryRequest, RhizoCryptRpc,
    RhizoCryptRpcServer,
};
use rhizo_crypt_core::{MerkleRoot, SessionId, SliceId, SliceMode, VertexId};
use serde::de::DeserializeOwned;
use serde_json::{Value, json};
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

    match method {
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
        "dag.dehydration.trigger" => dispatch_dehydrate(&server, params).await,
        "dag.dehydration.status" => dispatch_dehydrate_status(&server, params).await,
        "health.check" | "status" | "check" => dispatch_health(&server).await,
        "health.liveness" | "ping" | "health" => Ok(rhizo_crypt_core::niche::health_liveness()),
        "health.readiness" => dispatch_readiness(&server).await,
        "health.metrics" => dispatch_metrics(&server).await,
        "capabilities.list" | "capability.list" | "primal.capabilities" => {
            dispatch_capability_list(&server).await
        }
        _ => Err(HandlerError::MethodNotFound(request.method)),
    }
}

fn get_obj(params: &Value) -> Result<&serde_json::Map<String, Value>, HandlerError> {
    params
        .as_object()
        .ok_or_else(|| HandlerError::InvalidParams("params must be an object".to_string()))
}

fn get_str<'a>(
    obj: &'a serde_json::Map<String, Value>,
    key: &str,
) -> Result<&'a str, HandlerError> {
    obj.get(key)
        .and_then(Value::as_str)
        .ok_or_else(|| HandlerError::InvalidParams(format!("missing or invalid '{key}'")))
}

fn get_opt_str<'a>(obj: &'a serde_json::Map<String, Value>, key: &str) -> Option<&'a str> {
    obj.get(key).and_then(Value::as_str)
}

/// Deserialize from a `&Value` reference without cloning.
///
/// `&Value` implements `serde::Deserializer`, so owned types (`DeserializeOwned`)
/// can be produced from a borrow — no allocation for the Value itself.
fn from_value_ref<T: DeserializeOwned>(value: &Value) -> Result<T, serde_json::Error> {
    T::deserialize(value)
}

fn get_opt_deserialized<T: DeserializeOwned>(
    obj: &serde_json::Map<String, Value>,
    key: &str,
) -> Option<T> {
    obj.get(key).and_then(|v| from_value_ref(v).ok())
}

fn get_deserialized<T: DeserializeOwned>(
    obj: &serde_json::Map<String, Value>,
    key: &str,
) -> Result<T, HandlerError> {
    let v = obj.get(key).unwrap_or(&Value::Null);
    from_value_ref(v).map_err(|e| HandlerError::InvalidParams(format!("{key}: {e}")))
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

fn parse_vertex_id_array(
    obj: &serde_json::Map<String, Value>,
    key: &str,
) -> Result<Vec<VertexId>, HandlerError> {
    obj.get(key)
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(|v| v.as_str())
        .map(parse_vertex_id)
        .collect()
}

fn parse_metadata_array(obj: &serde_json::Map<String, Value>) -> Vec<(String, String)> {
    obj.get("metadata")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(|v| v.as_object())
        .filter_map(|o| {
            let k = o.get("key")?.as_str()?.to_string();
            let val = o.get("value")?.as_str()?.to_string();
            Some((k, val))
        })
        .collect()
}

fn to_json<T: serde::Serialize>(val: &T) -> Result<Value, HandlerError> {
    serde_json::to_value(val).map_err(|e| HandlerError::InvalidParams(e.to_string()))
}

fn vertex_id_to_value(id: VertexId) -> Value {
    json!(hex::encode(id.as_bytes()))
}

fn vertex_ids_to_value(ids: &[VertexId]) -> Result<Value, HandlerError> {
    to_json(&ids.iter().map(|id| hex::encode(id.as_bytes())).collect::<Vec<_>>())
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
    let session_type = get_opt_deserialized(obj, "session_type").unwrap_or_default();
    let description = get_opt_str(obj, "description").map(String::from);
    let parent_session = get_opt_str(obj, "parent_session").map(parse_session_id).transpose()?;
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
    let session_id = parse_session_id(get_str(obj, "session_id")?)?;
    let info = server.clone().get_session(tarpc::context::current(), session_id).await?;
    to_json(&info)
}

async fn dispatch_session_list(server: &RhizoCryptRpcServer) -> Result<Value, HandlerError> {
    let list = server.clone().list_sessions(tarpc::context::current()).await?;
    to_json(&list)
}

async fn dispatch_session_discard(
    server: &RhizoCryptRpcServer,
    params: Value,
) -> Result<Value, HandlerError> {
    let obj = get_obj(&params)?;
    let session_id = parse_session_id(get_str(obj, "session_id")?)?;
    server.clone().discard_session(tarpc::context::current(), session_id).await?;
    Ok(Value::Null)
}

async fn dispatch_event_append(
    server: &RhizoCryptRpcServer,
    params: Value,
) -> Result<Value, HandlerError> {
    let obj = get_obj(&params)?;
    let session_id = parse_session_id(get_str(obj, "session_id")?)?;
    let event_type = get_deserialized(obj, "event_type")?;
    let agent = get_opt_str(obj, "agent").map(parse_did);
    let parents = parse_vertex_id_array(obj, "parents")?;
    let metadata = parse_metadata_array(obj);
    let payload_ref = get_opt_str(obj, "payload_ref").map(String::from);

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
        let session_id = parse_session_id(get_str(r_obj, "session_id")?)?;
        let event_type = get_deserialized(r_obj, "event_type")?;
        let agent = get_opt_str(r_obj, "agent").map(parse_did);
        let parents = parse_vertex_id_array(r_obj, "parents")?;
        let metadata = parse_metadata_array(r_obj);
        let payload_ref = get_opt_str(r_obj, "payload_ref").map(String::from);
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
    vertex_ids_to_value(&ids)
}

async fn dispatch_vertex_get(
    server: &RhizoCryptRpcServer,
    params: Value,
) -> Result<Value, HandlerError> {
    let obj = get_obj(&params)?;
    let session_id = parse_session_id(get_str(obj, "session_id")?)?;
    let vertex_id = parse_vertex_id(get_str(obj, "vertex_id")?)?;
    let vertex =
        server.clone().get_vertex(tarpc::context::current(), session_id, vertex_id).await?;
    to_json(&vertex)
}

async fn dispatch_frontier_get(
    server: &RhizoCryptRpcServer,
    params: Value,
) -> Result<Value, HandlerError> {
    let obj = get_obj(&params)?;
    let session_id = parse_session_id(get_str(obj, "session_id")?)?;
    let frontier = server.clone().get_frontier(tarpc::context::current(), session_id).await?;
    vertex_ids_to_value(&frontier)
}

async fn dispatch_genesis_get(
    server: &RhizoCryptRpcServer,
    params: Value,
) -> Result<Value, HandlerError> {
    let obj = get_obj(&params)?;
    let session_id = parse_session_id(get_str(obj, "session_id")?)?;
    let genesis = server.clone().get_genesis(tarpc::context::current(), session_id).await?;
    vertex_ids_to_value(&genesis)
}

async fn dispatch_vertex_query(
    server: &RhizoCryptRpcServer,
    params: Value,
) -> Result<Value, HandlerError> {
    let obj = get_obj(&params)?;
    let session_id = parse_session_id(get_str(obj, "session_id")?)?;
    let event_types = get_opt_deserialized(obj, "event_types");
    let agent = get_opt_str(obj, "agent").map(parse_did);
    let start_time = get_opt_deserialized(obj, "start_time");
    let end_time = get_opt_deserialized(obj, "end_time");
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
    to_json(&vertices)
}

async fn dispatch_vertex_children(
    server: &RhizoCryptRpcServer,
    params: Value,
) -> Result<Value, HandlerError> {
    let obj = get_obj(&params)?;
    let session_id = parse_session_id(get_str(obj, "session_id")?)?;
    let vertex_id = parse_vertex_id(get_str(obj, "vertex_id")?)?;
    let children =
        server.clone().get_children(tarpc::context::current(), session_id, vertex_id).await?;
    vertex_ids_to_value(&children)
}

async fn dispatch_merkle_root(
    server: &RhizoCryptRpcServer,
    params: Value,
) -> Result<Value, HandlerError> {
    let obj = get_obj(&params)?;
    let session_id = parse_session_id(get_str(obj, "session_id")?)?;
    let root = server.clone().get_merkle_root(tarpc::context::current(), session_id).await?;
    Ok(json!(hex::encode(root.0)))
}

async fn dispatch_merkle_proof(
    server: &RhizoCryptRpcServer,
    params: Value,
) -> Result<Value, HandlerError> {
    let obj = get_obj(&params)?;
    let session_id = parse_session_id(get_str(obj, "session_id")?)?;
    let vertex_id = parse_vertex_id(get_str(obj, "vertex_id")?)?;
    let proof =
        server.clone().get_merkle_proof(tarpc::context::current(), session_id, vertex_id).await?;
    to_json(&proof)
}

async fn dispatch_merkle_verify(
    server: &RhizoCryptRpcServer,
    params: Value,
) -> Result<Value, HandlerError> {
    let obj = get_obj(&params)?;
    let root_hex = get_str(obj, "root")?;
    let root_bytes =
        hex::decode(root_hex).map_err(|e| HandlerError::InvalidParams(format!("root: {e}")))?;
    if root_bytes.len() != 32 {
        return Err(HandlerError::InvalidParams("root must be 32 bytes hex".to_string()));
    }
    let mut root_arr = [0u8; 32];
    root_arr.copy_from_slice(&root_bytes);
    let root = MerkleRoot(root_arr);
    let proof = get_deserialized(obj, "proof")?;
    let ok = server.clone().verify_proof(tarpc::context::current(), root, proof).await?;
    Ok(json!(ok))
}

async fn dispatch_slice_checkout(
    server: &RhizoCryptRpcServer,
    params: Value,
) -> Result<Value, HandlerError> {
    let obj = get_obj(&params)?;
    let spine_id = get_str(obj, "spine_id")?.to_string();
    let entry_hash = get_str(obj, "entry_hash")?.to_string();
    let entry_index = obj
        .get("entry_index")
        .and_then(Value::as_u64)
        .ok_or_else(|| HandlerError::InvalidParams("missing entry_index".to_string()))?;
    let mode = get_opt_deserialized(obj, "mode").unwrap_or(SliceMode::Copy {
        allow_recopy: false,
    });
    let owner = parse_did(get_str(obj, "owner")?);
    let holder = parse_did(get_str(obj, "holder")?);
    let session_id = parse_session_id(get_str(obj, "session_id")?)?;
    let checkout_vertex = parse_vertex_id(get_str(obj, "checkout_vertex")?)?;
    let certificate_id = get_opt_str(obj, "certificate_id").map(String::from);
    let duration_seconds = obj.get("duration_seconds").and_then(Value::as_u64);

    let req = CheckoutSliceRequest {
        spine_id,
        entry_hash,
        entry_index,
        mode,
        owner,
        holder,
        session_id,
        checkout_vertex,
        certificate_id,
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
    let slice_id = parse_slice_id(get_str(obj, "slice_id")?)?;
    let slice = server.clone().get_slice(tarpc::context::current(), slice_id).await?;
    to_json(&slice)
}

async fn dispatch_slice_list(server: &RhizoCryptRpcServer) -> Result<Value, HandlerError> {
    let list = server.clone().list_slices(tarpc::context::current()).await?;
    to_json(&list)
}

async fn dispatch_slice_resolve(
    server: &RhizoCryptRpcServer,
    params: Value,
) -> Result<Value, HandlerError> {
    let obj = get_obj(&params)?;
    let slice_id = parse_slice_id(get_str(obj, "slice_id")?)?;
    let session_id = parse_session_id(get_str(obj, "session_id")?)?;
    server.clone().resolve_slice(tarpc::context::current(), slice_id, session_id).await?;
    Ok(Value::Null)
}

async fn dispatch_dehydrate(
    server: &RhizoCryptRpcServer,
    params: Value,
) -> Result<Value, HandlerError> {
    let obj = get_obj(&params)?;
    let session_id = parse_session_id(get_str(obj, "session_id")?)?;
    let root = server.clone().dehydrate(tarpc::context::current(), session_id).await?;
    Ok(json!(hex::encode(root.0)))
}

async fn dispatch_dehydrate_status(
    server: &RhizoCryptRpcServer,
    params: Value,
) -> Result<Value, HandlerError> {
    let obj = get_obj(&params)?;
    let session_id = parse_session_id(get_str(obj, "session_id")?)?;
    let status =
        server.clone().get_dehydration_status(tarpc::context::current(), session_id).await?;
    to_json(&status)
}

async fn dispatch_health(server: &RhizoCryptRpcServer) -> Result<Value, HandlerError> {
    let status = server.clone().health(tarpc::context::current()).await?;
    to_json(&status)
}

async fn dispatch_readiness(server: &RhizoCryptRpcServer) -> Result<Value, HandlerError> {
    let status = server.clone().health(tarpc::context::current()).await?;
    Ok(rhizo_crypt_core::niche::health_readiness(status.healthy))
}

async fn dispatch_metrics(server: &RhizoCryptRpcServer) -> Result<Value, HandlerError> {
    let metrics = server.clone().metrics(tarpc::context::current()).await?;
    to_json(&metrics)
}

async fn dispatch_capability_list(server: &RhizoCryptRpcServer) -> Result<Value, HandlerError> {
    let capabilities = server.clone().list_capabilities(tarpc::context::current()).await?;
    to_json(&capabilities)
}

#[cfg(test)]
#[path = "handler_tests.rs"]
mod tests;

#[cfg(test)]
#[path = "handler_tests_validation.rs"]
mod tests_validation;
