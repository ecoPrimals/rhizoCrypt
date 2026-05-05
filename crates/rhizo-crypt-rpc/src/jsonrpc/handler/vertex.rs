// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2024–2026 ecoPrimals Project

//! JSON-RPC dispatch for vertex and query operations.

use super::HandlerError;
use super::params::{
    get_obj, get_opt_deserialized, get_opt_str, get_str, parse_did, parse_session_id,
    parse_vertex_id_value, to_json, vertex_ids_to_value,
};
use crate::service::{RhizoCryptRpc, RhizoCryptRpcServer};
use crate::service_types::QueryRequest;
use serde_json::Value;
use std::borrow::Cow;

pub async fn dispatch_vertex_get(
    server: &RhizoCryptRpcServer,
    params: Value,
) -> Result<Value, HandlerError> {
    let obj = get_obj(&params)?;
    let session_id = parse_session_id(get_str(obj, "session_id")?)?;
    let vid_val = obj
        .get("vertex_id")
        .ok_or(HandlerError::InvalidParams(Cow::Borrowed("missing 'vertex_id'")))?;
    let vertex_id = parse_vertex_id_value(vid_val)?;
    let vertex =
        server.clone().get_vertex(tarpc::context::current(), session_id, vertex_id).await?;
    to_json(&vertex)
}

pub async fn dispatch_frontier_get(
    server: &RhizoCryptRpcServer,
    params: Value,
) -> Result<Value, HandlerError> {
    let obj = get_obj(&params)?;
    let session_id = parse_session_id(get_str(obj, "session_id")?)?;
    let frontier = server.clone().get_frontier(tarpc::context::current(), session_id).await?;
    vertex_ids_to_value(&frontier)
}

pub async fn dispatch_genesis_get(
    server: &RhizoCryptRpcServer,
    params: Value,
) -> Result<Value, HandlerError> {
    let obj = get_obj(&params)?;
    let session_id = parse_session_id(get_str(obj, "session_id")?)?;
    let genesis = server.clone().get_genesis(tarpc::context::current(), session_id).await?;
    vertex_ids_to_value(&genesis)
}

pub async fn dispatch_vertex_query(
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

pub async fn dispatch_vertex_children(
    server: &RhizoCryptRpcServer,
    params: Value,
) -> Result<Value, HandlerError> {
    let obj = get_obj(&params)?;
    let session_id = parse_session_id(get_str(obj, "session_id")?)?;
    let vid_val = obj
        .get("vertex_id")
        .ok_or(HandlerError::InvalidParams(Cow::Borrowed("missing 'vertex_id'")))?;
    let vertex_id = parse_vertex_id_value(vid_val)?;
    let children =
        server.clone().get_children(tarpc::context::current(), session_id, vertex_id).await?;
    vertex_ids_to_value(&children)
}
