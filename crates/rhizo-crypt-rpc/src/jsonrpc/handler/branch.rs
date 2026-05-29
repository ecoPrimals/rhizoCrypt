// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2024–2026 ecoPrimals Project

//! JSON-RPC dispatch for DAG branch/diff/merge/federate operations (Wave 60).

use super::HandlerError;
use super::params::{
    get_obj, get_opt_str, get_str, parse_session_id, parse_vertex_id_array, parse_vertex_id_value,
    vertex_id_to_value,
};
use crate::service::{RhizoCryptRpc, RhizoCryptRpcServer};
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
    let resp = server.clone().branch_session(tarpc::context::current(), req).await?;
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
    let resp = server.clone().diff_sessions(tarpc::context::current(), req).await?;
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
    let id = server.clone().merge_branches(tarpc::context::current(), req).await?;
    Ok(vertex_id_to_value(id))
}

pub async fn dispatch_federate(
    server: &RhizoCryptRpcServer,
    params: Value,
) -> Result<Value, HandlerError> {
    let obj = get_obj(&params)?;
    let session_id = parse_session_id(get_str(obj, "session_id")?)?;
    let vertices_arr = obj
        .get("vertices")
        .and_then(Value::as_array)
        .ok_or(HandlerError::InvalidParams(Cow::Borrowed("missing 'vertices' array")))?;

    let mut vertices = Vec::with_capacity(vertices_arr.len());
    for v in vertices_arr {
        let vertex: rhizo_crypt_core::Vertex = serde_json::from_value(v.clone())
            .map_err(|e| HandlerError::InvalidParams(Cow::Owned(format!("invalid vertex: {e}"))))?;
        vertices.push(vertex);
    }

    let req = FederateRequest {
        session_id,
        vertices,
    };
    let resp = server.clone().federate(tarpc::context::current(), req).await?;
    serde_json::to_value(resp).map_err(|e| HandlerError::InvalidParams(Cow::Owned(e.to_string())))
}
