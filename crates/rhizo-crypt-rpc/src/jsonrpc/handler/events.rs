// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2024–2026 ecoPrimals Project

//! JSON-RPC dispatch for DAG event operations.

use super::HandlerError;
use super::params::{
    get_deserialized, get_obj, get_opt_str, get_str, parse_did, parse_metadata_array,
    parse_session_id, parse_vertex_id_array, vertex_id_to_value, vertex_ids_to_value,
};
use crate::service::{RhizoCryptRpc, RhizoCryptRpcServer};
use crate::service_types::AppendEventRequest;
use serde_json::Value;
use std::borrow::Cow;

pub async fn dispatch_event_append(
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

pub async fn dispatch_event_append_batch(
    server: &RhizoCryptRpcServer,
    params: Value,
) -> Result<Value, HandlerError> {
    let obj = get_obj(&params)?;
    let requests_arr = obj
        .get("requests")
        .and_then(Value::as_array)
        .ok_or(HandlerError::InvalidParams(Cow::Borrowed("missing 'requests' array")))?;
    let mut requests = Vec::with_capacity(requests_arr.len());
    for r in requests_arr {
        let r_obj = r
            .as_object()
            .ok_or(HandlerError::InvalidParams(Cow::Borrowed("each request must be an object")))?;
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
