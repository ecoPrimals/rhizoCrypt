// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2024–2026 ecoPrimals Project

//! JSON-RPC dispatch for session operations.

use super::HandlerError;
use super::params::{
    get_obj, get_opt_deserialized, get_opt_str, get_str, parse_session_id, session_id_to_value,
    to_json,
};
use crate::service::{RhizoCryptRpc, RhizoCryptRpcServer};
use crate::service_types::CreateSessionRequest;
use serde_json::Value;

pub async fn dispatch_session_create(
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

pub async fn dispatch_session_get(
    server: &RhizoCryptRpcServer,
    params: Value,
) -> Result<Value, HandlerError> {
    let obj = get_obj(&params)?;
    let session_id = parse_session_id(get_str(obj, "session_id")?)?;
    let info = server.clone().get_session(tarpc::context::current(), session_id).await?;
    to_json(&info)
}

pub async fn dispatch_session_list(server: &RhizoCryptRpcServer) -> Result<Value, HandlerError> {
    let list = server.clone().list_sessions(tarpc::context::current()).await?;
    to_json(&list)
}

pub async fn dispatch_session_discard(
    server: &RhizoCryptRpcServer,
    params: Value,
) -> Result<Value, HandlerError> {
    let obj = get_obj(&params)?;
    let session_id = parse_session_id(get_str(obj, "session_id")?)?;
    server.clone().discard_session(tarpc::context::current(), session_id).await?;
    Ok(Value::Null)
}
