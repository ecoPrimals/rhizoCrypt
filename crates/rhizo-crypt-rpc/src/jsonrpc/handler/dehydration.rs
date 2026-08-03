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

pub async fn dispatch_dehydrate_batch(
    server: &RhizoCryptRpcServer,
    params: Value,
) -> Result<Value, HandlerError> {
    let obj = get_obj(&params)?;
    let raw_ids = obj
        .get("session_ids")
        .and_then(Value::as_array)
        .ok_or_else(|| HandlerError::InvalidParams("session_ids array required".into()))?;

    let session_ids: Vec<rhizo_crypt_core::SessionId> = raw_ids
        .iter()
        .map(|v| {
            v.as_str()
                .ok_or_else(|| HandlerError::InvalidParams("session_ids must be strings".into()))
                .and_then(parse_session_id)
        })
        .collect::<Result<_, _>>()?;

    let results = server.clone().dehydrate_batch(tarpc::context::current(), session_ids).await?;
    to_json(&results)
}

pub async fn dispatch_pipeline_ingest(
    server: &RhizoCryptRpcServer,
    params: Value,
) -> Result<Value, HandlerError> {
    use super::params::{
        get_deserialized, get_opt_str, parse_did, parse_metadata_array, parse_vertex_id_array,
    };
    use std::borrow::Cow;

    let obj = get_obj(&params)?;
    let session_type = get_deserialized(obj, "session_type")?;
    let description = get_opt_str(obj, "description").map(String::from);
    let dehydrate = obj.get("dehydrate").and_then(Value::as_bool).unwrap_or(false);

    let events_arr = obj
        .get("events")
        .and_then(Value::as_array)
        .ok_or(HandlerError::InvalidParams(Cow::Borrowed("missing 'events' array")))?;

    let mut events = Vec::with_capacity(events_arr.len());
    for ev in events_arr {
        let ev_obj = ev
            .as_object()
            .ok_or(HandlerError::InvalidParams(Cow::Borrowed("each event must be an object")))?;
        let session_id = parse_session_id(
            get_opt_str(ev_obj, "session_id").unwrap_or("00000000-0000-7000-8000-000000000000"),
        )?;
        let event_type = get_deserialized(ev_obj, "event_type")?;
        let agent = get_opt_str(ev_obj, "agent").map(parse_did);
        let parents = parse_vertex_id_array(ev_obj, "parents")?;
        let metadata = parse_metadata_array(ev_obj);
        let payload_ref = get_opt_str(ev_obj, "payload_ref").map(String::from);
        events.push(crate::service_types::AppendEventRequest {
            session_id,
            event_type,
            agent,
            parents,
            metadata,
            payload_ref,
        });
    }

    let request = crate::service_types::PipelineIngestRequest {
        session_type,
        description,
        events,
        dehydrate,
    };
    let result = server.clone().pipeline_ingest(tarpc::context::current(), request).await?;
    to_json(&result)
}
