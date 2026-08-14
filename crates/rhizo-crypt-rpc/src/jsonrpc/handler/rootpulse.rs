// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2024–2026 ecoPrimals Project

//! rootPulse graph step handlers.
//!
//! These handlers support the rootPulse trio graphs (commit, harvest, diff)
//! defined in wateringHole. They compose existing DAG operations into
//! graph-step-compatible entry points.

use super::HandlerError;
use super::params::{get_obj, parse_session_id};
use crate::error::RpcError;
use crate::service::{RhizoCryptRpc, RhizoCryptRpcServer};
use crate::service_types::AppendEventRequest;
use rhizo_crypt_core::{EventType, SessionBuilder, SessionId, SessionType};
use serde_json::{Value, json};

/// Handle `rootpulse.record_build` — record build provenance in a DAG session.
pub async fn dispatch_record_build(
    server: &RhizoCryptRpcServer,
    params: Value,
) -> Result<Value, HandlerError> {
    let obj = get_obj(&params)?;
    let primal_name = get_rootpulse_str(obj, "PRIMAL_NAME")?;
    let target_triple = get_rootpulse_str(obj, "TARGET_TRIPLE")?;
    let commit_sha = get_rootpulse_str(obj, "COMMIT_SHA")?;
    let blake3_hash = get_rootpulse_str(obj, "BLAKE3_HASH")?;
    let builder_gate = get_rootpulse_str(obj, "BUILDER_GATE")?;
    let built_at = get_rootpulse_str(obj, "BUILT_AT")?;

    let session_id = deterministic_build_session_id(primal_name, target_triple);
    ensure_build_session(server, session_id, primal_name, target_triple).await?;

    let metadata = vec![
        ("primal_name".to_owned(), primal_name.to_owned()),
        ("target_triple".to_owned(), target_triple.to_owned()),
        ("commit_sha".to_owned(), commit_sha.to_owned()),
        ("blake3_hash".to_owned(), blake3_hash.to_owned()),
        ("builder_gate".to_owned(), builder_gate.to_owned()),
        ("built_at".to_owned(), built_at.to_owned()),
    ];

    let req = AppendEventRequest {
        session_id,
        event_type: EventType::Custom {
            domain: "rootpulse".into(),
            event_name: "build_record".into(),
        },
        agent: None,
        parents: Vec::new(),
        metadata,
        payload_ref: Some(blake3_hash.to_owned()),
    };

    let vertex_id = server.clone().append_event(tarpc::context::current(), req).await?;
    let dag_ref = format!("{}:{}", session_id.as_uuid(), hex::encode(vertex_id.as_bytes()));

    Ok(json!({ "dag_ref": dag_ref }))
}

/// Handle `rootpulse.dehydrate_state` — dehydrate a session for the commit pipeline.
pub async fn dispatch_dehydrate_state(
    server: &RhizoCryptRpcServer,
    params: Value,
) -> Result<Value, HandlerError> {
    let obj = get_obj(&params)?;
    let session_id = parse_session_id(get_rootpulse_str(obj, "SESSION_ID")?)?;
    let _agent_did = get_rootpulse_opt_str(obj, "AGENT_DID");
    let _family_id = get_rootpulse_opt_str(obj, "FAMILY_ID");

    let root = server.clone().dehydrate(tarpc::context::current(), session_id).await?;
    let merkle_root_hex = hex::encode(root.0);

    Ok(json!({
        "dehydrated_blob": merkle_root_hex,
        "content_hash": merkle_root_hex,
    }))
}

async fn ensure_build_session(
    server: &RhizoCryptRpcServer,
    session_id: SessionId,
    primal_name: &str,
    target_triple: &str,
) -> Result<(), HandlerError> {
    if server.clone().get_session(tarpc::context::current(), session_id).await.is_ok() {
        return Ok(());
    }

    let mut session = SessionBuilder::new(SessionType::Custom {
        domain: "rootpulse_build".into(),
    })
    .with_name(format!("rootpulse:build:{primal_name}:{target_triple}"))
    .build();
    session.id = session_id;
    server.primal().create_session(session).map_err(RpcError::from)?;
    Ok(())
}

fn deterministic_build_session_id(primal_name: &str, target_triple: &str) -> SessionId {
    let hash = blake3::hash(format!("rootpulse:build:{primal_name}:{target_triple}").as_bytes());
    let mut bytes = [0u8; 16];
    bytes.copy_from_slice(&hash.as_bytes()[..16]);
    bytes[6] = (bytes[6] & 0x0f) | 0x70;
    bytes[8] = (bytes[8] & 0x3f) | 0x80;
    SessionId::new(uuid::Uuid::from_bytes(bytes))
}

fn get_rootpulse_str<'a>(
    obj: &'a serde_json::Map<String, Value>,
    key: &str,
) -> Result<&'a str, HandlerError> {
    let lower = key.to_ascii_lowercase();
    obj.get(key)
        .or_else(|| obj.get(&lower))
        .and_then(Value::as_str)
        .ok_or_else(|| HandlerError::InvalidParams(format!("missing or invalid '{key}'").into()))
}

fn get_rootpulse_opt_str<'a>(
    obj: &'a serde_json::Map<String, Value>,
    key: &str,
) -> Option<&'a str> {
    let lower = key.to_ascii_lowercase();
    obj.get(key).or_else(|| obj.get(&lower)).and_then(Value::as_str)
}

#[cfg(test)]
#[expect(clippy::unwrap_used, reason = "test code")]
mod tests {
    use super::*;

    #[test]
    fn deterministic_session_id_is_stable() {
        let a = deterministic_build_session_id("rhizocrypt", "x86_64-unknown-linux-gnu");
        let b = deterministic_build_session_id("rhizocrypt", "x86_64-unknown-linux-gnu");
        assert_eq!(a, b);
    }

    #[test]
    fn deterministic_session_id_differs_by_triple() {
        let a = deterministic_build_session_id("rhizocrypt", "x86_64-unknown-linux-gnu");
        let b = deterministic_build_session_id("rhizocrypt", "aarch64-unknown-linux-gnu");
        assert_ne!(a, b);
    }

    #[test]
    fn get_rootpulse_str_accepts_upper_and_lower() {
        let mut obj = serde_json::Map::new();
        obj.insert("PRIMAL_NAME".into(), json!("rhizocrypt"));
        assert_eq!(get_rootpulse_str(&obj, "PRIMAL_NAME").unwrap(), "rhizocrypt");

        let mut lower = serde_json::Map::new();
        lower.insert("primal_name".into(), json!("rhizocrypt"));
        assert_eq!(get_rootpulse_str(&lower, "PRIMAL_NAME").unwrap(), "rhizocrypt");
    }

    #[test]
    fn get_rootpulse_str_missing_field_errors() {
        let obj = serde_json::Map::new();
        assert!(get_rootpulse_str(&obj, "PRIMAL_NAME").is_err());
    }
}
