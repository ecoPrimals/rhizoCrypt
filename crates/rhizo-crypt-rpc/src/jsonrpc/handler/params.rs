// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2024–2026 ecoPrimals Project

//! Shared JSON parameter parsing and value helpers for JSON-RPC dispatch.

use super::HandlerError;
use rhizo_crypt_core::{SessionId, SliceId, VertexId};
use serde::de::DeserializeOwned;
use serde_json::{Value, json};
use std::borrow::Cow;

pub(super) fn get_obj(params: &Value) -> Result<&serde_json::Map<String, Value>, HandlerError> {
    params.as_object().ok_or(HandlerError::InvalidParams(Cow::Borrowed("params must be an object")))
}

pub(super) fn get_str<'a>(
    obj: &'a serde_json::Map<String, Value>,
    key: &str,
) -> Result<&'a str, HandlerError> {
    obj.get(key)
        .and_then(Value::as_str)
        .ok_or_else(|| HandlerError::InvalidParams(format!("missing or invalid '{key}'").into()))
}

pub(super) fn get_opt_str<'a>(
    obj: &'a serde_json::Map<String, Value>,
    key: &str,
) -> Option<&'a str> {
    obj.get(key).and_then(Value::as_str)
}

/// Deserialize from a `&Value` reference without cloning.
///
/// `&Value` implements `serde::Deserializer`, so owned types (`DeserializeOwned`)
/// can be produced from a borrow — no allocation for the Value itself.
pub(super) fn from_value_ref<T: DeserializeOwned>(value: &Value) -> Result<T, serde_json::Error> {
    serde::Deserialize::deserialize(value)
}

pub(super) fn get_opt_deserialized<T: DeserializeOwned>(
    obj: &serde_json::Map<String, Value>,
    key: &str,
) -> Option<T> {
    obj.get(key).and_then(|v| from_value_ref(v).ok())
}

pub(super) fn get_deserialized<T: DeserializeOwned>(
    obj: &serde_json::Map<String, Value>,
    key: &str,
) -> Result<T, HandlerError> {
    let v = obj.get(key).unwrap_or(&Value::Null);
    from_value_ref(v).map_err(|e| HandlerError::InvalidParams(format!("{key}: {e}").into()))
}

pub(super) fn parse_session_id(s: &str) -> Result<SessionId, HandlerError> {
    uuid::Uuid::parse_str(s)
        .map(SessionId::new)
        .map_err(|e| HandlerError::InvalidParams(format!("invalid session_id: {e}").into()))
}

pub(super) fn parse_vertex_id(s: &str) -> Result<VertexId, HandlerError> {
    let bytes = hex::decode(s)
        .map_err(|e| HandlerError::InvalidParams(format!("invalid vertex_id hex: {e}").into()))?;
    if bytes.len() != 32 {
        return Err(HandlerError::InvalidParams(Cow::Borrowed("vertex_id must be 32 bytes hex")));
    }
    let mut arr = [0u8; 32];
    arr.copy_from_slice(&bytes);
    Ok(VertexId(arr))
}

pub(super) fn parse_slice_id(s: &str) -> Result<SliceId, HandlerError> {
    uuid::Uuid::parse_str(s)
        .map(SliceId::new)
        .map_err(|e| HandlerError::InvalidParams(format!("invalid slice_id: {e}").into()))
}

pub(super) fn parse_did(s: &str) -> rhizo_crypt_core::Did {
    rhizo_crypt_core::Did::new(s)
}

pub(super) fn parse_vertex_id_array(
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

pub(super) fn parse_metadata_array(obj: &serde_json::Map<String, Value>) -> Vec<(String, String)> {
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

pub(super) fn to_json<T: serde::Serialize>(val: &T) -> Result<Value, HandlerError> {
    serde_json::to_value(val).map_err(|e| HandlerError::InvalidParams(e.to_string().into()))
}

pub(super) fn vertex_id_to_value(id: VertexId) -> Value {
    json!(hex::encode(id.as_bytes()))
}

pub(super) fn vertex_ids_to_value(ids: &[VertexId]) -> Result<Value, HandlerError> {
    to_json(&ids.iter().map(|id| hex::encode(id.as_bytes())).collect::<Vec<_>>())
}

pub(super) fn session_id_to_value(id: SessionId) -> Value {
    json!(id.as_uuid().to_string())
}

pub(super) fn slice_id_to_value(id: SliceId) -> Value {
    json!(id.0.to_string())
}
