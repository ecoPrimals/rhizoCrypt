// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2024–2026 ecoPrimals Project

//! JSON-RPC dispatch for cross-gate mesh trust event recording.
//!
//! Handles `mesh.events.record` — accepts a trust event payload from
//! bearDog (or any cross-gate auth provider) and records it via the
//! `MeshEventListener`. The event is mapped to an `EventType` mesh
//! variant suitable for DAG append.

use super::HandlerError;
use super::params::get_obj;
use crate::service::RhizoCryptRpcServer;
use rhizo_crypt_core::MeshTrustEvent;
use serde_json::Value;
use std::borrow::Cow;

/// Handle `mesh.events.record`.
///
/// Accepts a `MeshTrustEvent` JSON payload and records it in the
/// mesh event listener. Returns the mapped `EventType` that would
/// be appended to a DAG session.
///
/// ## Request
///
/// ```json
/// {
///   "kind": { "type": "TrustIssuerRegistered", "payload": { "issuer_fingerprint": "abc" } },
///   "source_gate": "eastGate",
///   "timestamp": 1717444800
/// }
/// ```
///
/// ## Response
///
/// ```json
/// {
///   "recorded": true,
///   "event_type": "trust_issuer_registered",
///   "source_gate": "eastGate",
///   "event_count": 1
/// }
/// ```
pub async fn dispatch_mesh_events_record(
    server: &RhizoCryptRpcServer,
    params: Value,
) -> Result<Value, HandlerError> {
    let obj = get_obj(&params)?;

    let event: MeshTrustEvent = serde_json::from_value(Value::Object(obj.clone()))
        .map_err(|e| HandlerError::InvalidParams(Cow::Owned(format!("invalid mesh event: {e}"))))?;

    let source_gate = event.source_gate.clone();
    let event_type = server.primal.mesh_listener().record_event(event).await;
    let event_count = server.primal.mesh_listener().event_count().await;

    Ok(serde_json::json!({
        "recorded": true,
        "event_type": event_type.name(),
        "source_gate": source_gate,
        "event_count": event_count,
    }))
}
