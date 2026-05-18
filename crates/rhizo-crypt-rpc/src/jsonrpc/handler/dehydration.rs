// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2024–2026 ecoPrimals Project

//! JSON-RPC dispatch for dehydration operations.

use super::HandlerError;
use super::params::{get_obj, get_str, parse_session_id, parse_vertex_id_value, to_json};
use crate::service::{RhizoCryptRpc, RhizoCryptRpcServer};
use rhizo_crypt_core::VertexId;
use serde_json::{Value, json};

pub async fn dispatch_partial_dehydrate(
    server: &RhizoCryptRpcServer,
    params: Value,
) -> Result<Value, HandlerError> {
    let obj = get_obj(&params)?;
    let session_id = parse_session_id(get_str(obj, "session_id")?)?;

    let vertex_ids: Vec<VertexId> = match obj.get("vertex_ids").and_then(Value::as_array) {
        Some(arr) => arr.iter().map(parse_vertex_id_value).collect::<Result<_, _>>()?,
        None => Vec::new(),
    };

    let resp =
        server.clone().partial_dehydrate(tarpc::context::current(), session_id, vertex_ids).await?;
    to_json(&resp)
}

pub async fn dispatch_dehydrate(
    server: &RhizoCryptRpcServer,
    params: Value,
) -> Result<Value, HandlerError> {
    let obj = get_obj(&params)?;
    let session_id = parse_session_id(get_str(obj, "session_id")?)?;
    let root = server.clone().dehydrate(tarpc::context::current(), session_id).await?;
    Ok(json!(hex::encode(root.0)))
}

pub async fn dispatch_dehydrate_status(
    server: &RhizoCryptRpcServer,
    params: Value,
) -> Result<Value, HandlerError> {
    let obj = get_obj(&params)?;
    let session_id = parse_session_id(get_str(obj, "session_id")?)?;
    let status =
        server.clone().get_dehydration_status(tarpc::context::current(), session_id).await?;
    to_json(&status)
}
