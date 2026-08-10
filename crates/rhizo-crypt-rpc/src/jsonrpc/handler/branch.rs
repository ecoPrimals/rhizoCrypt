// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2024–2026 ecoPrimals Project

//! JSON-RPC dispatch for DAG branch/diff/merge/federate operations (Wave 60).

use super::HandlerError;
use super::params::{
    get_obj, get_opt_str, get_str, parse_session_id, parse_vertex_id_array, parse_vertex_id_value,
    vertex_id_to_value,
};
use crate::service::RhizoCryptRpcServer;
use crate::service_types::{BranchRequest, DiffRequest, FederateRequest, MergeRequest};
use serde_json::Value;
use std::borrow::Cow;

pub async fn dispatch_branch(
    server: &RhizoCryptRpcServer,
    params: Value,
) -> Result<Value, HandlerError> {
    let obj = get_obj(&params)?;
    let session_id = parse_session_id(get_str(obj, "session_id")?)?;
    let cv_val = obj
        .get("checkout_vertex")
        .ok_or(HandlerError::InvalidParams(Cow::Borrowed("missing 'checkout_vertex'")))?;
    let checkout_vertex = parse_vertex_id_value(cv_val)?;
    let name = get_opt_str(obj, "name").map(String::from);
    let description = get_opt_str(obj, "description").map(String::from);

    let req = BranchRequest {
        session_id,
        checkout_vertex,
        name,
        description,
    };
    let resp = server.impl_branch_session(req).await?;
    serde_json::to_value(resp).map_err(|e| HandlerError::InvalidParams(Cow::Owned(e.to_string())))
}

pub async fn dispatch_diff(
    server: &RhizoCryptRpcServer,
    params: Value,
) -> Result<Value, HandlerError> {
    let obj = get_obj(&params)?;
    let base_session_id = parse_session_id(get_str(obj, "base_session_id")?)?;
    let other_session_id = parse_session_id(get_str(obj, "other_session_id")?)?;

    let req = DiffRequest {
        base_session_id,
        other_session_id,
    };
    let resp = server.impl_diff_sessions(req).await?;
    serde_json::to_value(resp).map_err(|e| HandlerError::InvalidParams(Cow::Owned(e.to_string())))
}

pub async fn dispatch_merge(
    server: &RhizoCryptRpcServer,
    params: Value,
) -> Result<Value, HandlerError> {
    let obj = get_obj(&params)?;
    let session_id = parse_session_id(get_str(obj, "session_id")?)?;
    let parents = parse_vertex_id_array(obj, "parents")?;
    let event_type = super::params::get_deserialized(obj, "event_type")?;
    let agent = get_opt_str(obj, "agent").map(super::params::parse_did);
    let metadata = super::params::parse_metadata_array(obj);

    let req = MergeRequest {
        session_id,
        parents,
        event_type,
        agent,
        metadata,
    };
    let id = server.impl_merge_branches(req).await?;
    Ok(vertex_id_to_value(id))
}

pub async fn dispatch_federate(
    server: &RhizoCryptRpcServer,
    mut params: Value,
) -> Result<Value, HandlerError> {
    let obj = params
        .as_object_mut()
        .ok_or(HandlerError::InvalidParams(Cow::Borrowed("params must be an object")))?;
    let session_id = parse_session_id(get_str(obj, "session_id")?)?;

    let Some(Value::Array(vertices_arr)) = obj.remove("vertices") else {
        return Err(HandlerError::InvalidParams(Cow::Borrowed("missing 'vertices' array")));
    };

    let vertices: Vec<rhizo_crypt_core::Vertex> = vertices_arr
        .into_iter()
        .map(|v| {
            serde_json::from_value(v).map_err(|e| {
                HandlerError::InvalidParams(Cow::Owned(format!("invalid vertex: {e}")))
            })
        })
        .collect::<Result<_, _>>()?;

    let source_gate = obj.get("source_gate").and_then(Value::as_str).map(str::to_owned);
    let verify_signatures = obj.get("verify_signatures").and_then(Value::as_bool).unwrap_or(false);

    let req = FederateRequest {
        session_id,
        vertices,
        source_gate,
        verify_signatures,
    };
    let resp = server.impl_federate(req).await?;
    serde_json::to_value(resp).map_err(|e| HandlerError::InvalidParams(Cow::Owned(e.to_string())))
}
