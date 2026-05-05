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

/// Parse a 32-byte hash from either a hex string (`"a1b2..."`) or a JSON
/// byte array (`[161, 178, ...]`).  Provenance trio interop: loamSpine and
/// sweetGrass may send `[u8; 32]` arrays where rhizoCrypt historically
/// expected hex strings.
pub(super) fn parse_hash32(value: &Value, field: &str) -> Result<[u8; 32], HandlerError> {
    match value {
        Value::String(s) => {
            let bytes = hex::decode(s).map_err(|e| {
                HandlerError::InvalidParams(format!("invalid {field} hex: {e}").into())
            })?;
            if bytes.len() != 32 {
                return Err(HandlerError::InvalidParams(
                    format!("{field} hex must decode to 32 bytes").into(),
                ));
            }
            let mut arr = [0u8; 32];
            arr.copy_from_slice(&bytes);
            Ok(arr)
        }
        Value::Array(arr) => {
            if arr.len() != 32 {
                return Err(HandlerError::InvalidParams(
                    format!("{field} byte array must have exactly 32 elements").into(),
                ));
            }
            let mut out = [0u8; 32];
            for (i, v) in arr.iter().enumerate() {
                out[i] = v.as_u64().and_then(|n| u8::try_from(n).ok()).ok_or_else(|| {
                    HandlerError::InvalidParams(
                        format!("{field}[{i}]: expected integer 0-255").into(),
                    )
                })?;
            }
            Ok(out)
        }
        _ => Err(HandlerError::InvalidParams(
            format!("{field} must be a hex string or byte array").into(),
        )),
    }
}

/// Parse a vertex ID from a `Value` — accepts hex string or byte array.
pub(super) fn parse_vertex_id_value(value: &Value) -> Result<VertexId, HandlerError> {
    parse_hash32(value, "vertex_id").map(VertexId)
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
        .map(parse_vertex_id_value)
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

#[cfg(test)]
#[expect(clippy::unwrap_used, clippy::cast_possible_truncation, reason = "test code")]
mod tests {
    use super::*;

    #[test]
    fn parse_hash32_hex_string() {
        let hex = hex::encode([42u8; 32]);
        let val = Value::String(hex);
        let arr = parse_hash32(&val, "test").unwrap();
        assert_eq!(arr, [42u8; 32]);
    }

    #[test]
    fn parse_hash32_byte_array() {
        let bytes: Vec<Value> = (0u8..32).map(|b| Value::Number(b.into())).collect();
        let val = Value::Array(bytes);
        let arr = parse_hash32(&val, "test").unwrap();
        assert_eq!(arr, core::array::from_fn::<u8, 32, _>(|i| i as u8));
    }

    #[test]
    fn parse_hash32_hex_and_bytes_equivalent() {
        let raw = [7u8; 32];
        let from_hex = parse_hash32(&Value::String(hex::encode(raw)), "t").unwrap();
        let from_arr = parse_hash32(
            &Value::Array(raw.iter().map(|&b| Value::Number(b.into())).collect()),
            "t",
        )
        .unwrap();
        assert_eq!(from_hex, from_arr);
    }

    #[test]
    fn parse_hash32_rejects_short_hex() {
        let val = Value::String("abcd".to_string());
        assert!(parse_hash32(&val, "test").is_err());
    }

    #[test]
    fn parse_hash32_rejects_wrong_length_array() {
        let bytes: Vec<Value> = (0u8..16).map(|b| Value::Number(b.into())).collect();
        assert!(parse_hash32(&Value::Array(bytes), "test").is_err());
    }

    #[test]
    fn parse_hash32_rejects_out_of_range_byte() {
        let mut bytes: Vec<Value> = (0u8..32).map(|b| Value::Number(b.into())).collect();
        bytes[5] = Value::Number(256.into());
        assert!(parse_hash32(&Value::Array(bytes), "test").is_err());
    }

    #[test]
    fn parse_hash32_rejects_non_string_non_array() {
        assert!(parse_hash32(&Value::Bool(true), "test").is_err());
        assert!(parse_hash32(&Value::Null, "test").is_err());
    }

    #[test]
    fn parse_vertex_id_value_hex() {
        let raw = [99u8; 32];
        let vid = parse_vertex_id_value(&Value::String(hex::encode(raw))).unwrap();
        assert_eq!(vid.0, raw);
    }

    #[test]
    fn parse_vertex_id_value_byte_array() {
        let raw = [13u8; 32];
        let bytes: Vec<Value> = raw.iter().map(|&b| Value::Number(b.into())).collect();
        let vid = parse_vertex_id_value(&Value::Array(bytes)).unwrap();
        assert_eq!(vid.0, raw);
    }

    #[test]
    fn parse_vertex_id_array_mixed() {
        let hex_id = Value::String(hex::encode([1u8; 32]));
        let byte_id = Value::Array((0u8..32).map(|b| Value::Number(b.into())).collect());
        let mut map = serde_json::Map::new();
        map.insert("parents".to_string(), Value::Array(vec![hex_id, byte_id]));
        let ids = parse_vertex_id_array(&map, "parents").unwrap();
        assert_eq!(ids.len(), 2);
        assert_eq!(ids[0].0, [1u8; 32]);
        assert_eq!(ids[1].0, core::array::from_fn::<u8, 32, _>(|i| i as u8));
    }
}
