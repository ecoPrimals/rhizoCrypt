// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2024–2026 ecoPrimals Project

//! JSON-RPC dispatch for slice checkout and lifecycle.

use super::HandlerError;
use super::params::{
    get_obj, get_opt_deserialized, get_opt_str, get_str, parse_did, parse_session_id,
    parse_slice_id, parse_vertex_id, slice_id_to_value, to_json,
};
use crate::service::{RhizoCryptRpc, RhizoCryptRpcServer};
use crate::service_types::CheckoutSliceRequest;
use rhizo_crypt_core::SliceMode;
use serde_json::Value;
use std::borrow::Cow;

pub async fn dispatch_slice_checkout(
    server: &RhizoCryptRpcServer,
    params: Value,
) -> Result<Value, HandlerError> {
    let obj = get_obj(&params)?;
    let spine_id = get_str(obj, "spine_id")?.to_string();
    let entry_hash = get_str(obj, "entry_hash")?.to_string();
    let entry_index = obj
        .get("entry_index")
        .and_then(Value::as_u64)
        .ok_or(HandlerError::InvalidParams(Cow::Borrowed("missing entry_index")))?;
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

pub async fn dispatch_slice_get(
    server: &RhizoCryptRpcServer,
    params: Value,
) -> Result<Value, HandlerError> {
    let obj = get_obj(&params)?;
    let slice_id = parse_slice_id(get_str(obj, "slice_id")?)?;
    let slice = server.clone().get_slice(tarpc::context::current(), slice_id).await?;
    to_json(&slice)
}

pub async fn dispatch_slice_list(server: &RhizoCryptRpcServer) -> Result<Value, HandlerError> {
    let list = server.clone().list_slices(tarpc::context::current()).await?;
    to_json(&list)
}

pub async fn dispatch_slice_resolve(
    server: &RhizoCryptRpcServer,
    params: Value,
) -> Result<Value, HandlerError> {
    let obj = get_obj(&params)?;
    let slice_id = parse_slice_id(get_str(obj, "slice_id")?)?;
    let session_id = parse_session_id(get_str(obj, "session_id")?)?;
    server.clone().resolve_slice(tarpc::context::current(), slice_id, session_id).await?;
    Ok(Value::Null)
}
