// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2024–2026 ecoPrimals Project

//! Tests for `mesh.events.record` handler dispatch.

#![expect(clippy::unwrap_used, reason = "test code")]

use super::mesh;
use super::test_support::{create_test_server, make_request, test_caller, test_gate};
use super::*;
use serde_json::json;

fn trust_issuer_event(source_gate: &str) -> serde_json::Value {
    json!({
        "kind": {
            "type": "TrustIssuerRegistered",
            "payload": { "issuer_fingerprint": "a1b2c3d4deadbeef" }
        },
        "source_gate": source_gate,
        "timestamp": 1_717_444_800
    })
}

fn key_exchange_event(source_gate: &str) -> serde_json::Value {
    json!({
        "kind": {
            "type": "KeyExchangeCompleted",
            "payload": {
                "remote_gate": "ironGate",
                "method": "ed25519_dh"
            }
        },
        "source_gate": source_gate,
        "timestamp": 1_717_444_801
    })
}

fn mesh_join_event(source_gate: &str) -> serde_json::Value {
    json!({
        "kind": {
            "type": "MeshJoin",
            "payload": { "mesh_id": "glacial-mesh-v1" }
        },
        "source_gate": source_gate,
        "timestamp": 1_717_444_802
    })
}

fn family_enrollment_event(source_gate: &str) -> serde_json::Value {
    json!({
        "kind": {
            "type": "FamilyEnrollment",
            "payload": {
                "family_id": "ecoPrimals",
                "primal_count": 5
            }
        },
        "source_gate": source_gate,
        "timestamp": 1_717_444_803
    })
}

fn mesh_leave_event(source_gate: &str) -> serde_json::Value {
    json!({
        "kind": {
            "type": "MeshLeave",
            "payload": {
                "mesh_id": "glacial-mesh-v1",
                "reason": "Graceful"
            }
        },
        "source_gate": source_gate,
        "timestamp": 1_717_444_804
    })
}

// ============================================================================
// Valid mesh event recording
// ============================================================================

#[tokio::test]
async fn test_mesh_events_record_trust_issuer_registered() {
    let server = create_test_server().await;
    let params = trust_issuer_event("eastGate");

    let result = mesh::dispatch_mesh_events_record(&server, params).await.unwrap();
    assert_eq!(result["recorded"], true);
    assert_eq!(result["event_type"], "trust_issuer_registered");
    assert_eq!(result["source_gate"], "eastGate");
    assert_eq!(result["event_count"], 1);
}

#[tokio::test]
async fn test_mesh_events_record_key_exchange_completed() {
    let server = create_test_server().await;
    let params = key_exchange_event("strandGate");

    let result = mesh::dispatch_mesh_events_record(&server, params).await.unwrap();
    assert_eq!(result["recorded"], true);
    assert_eq!(result["event_type"], "key_exchange_completed");
    assert_eq!(result["source_gate"], "strandGate");
    assert_eq!(result["event_count"], 1);
}

#[tokio::test]
async fn test_mesh_events_record_mesh_join() {
    let server = create_test_server().await;
    let params = mesh_join_event("westGate");

    let result = mesh::dispatch_mesh_events_record(&server, params).await.unwrap();
    assert_eq!(result["recorded"], true);
    assert_eq!(result["event_type"], "mesh_join");
    assert_eq!(result["source_gate"], "westGate");
    assert_eq!(result["event_count"], 1);
}

#[tokio::test]
async fn test_mesh_events_record_family_enrollment() {
    let server = create_test_server().await;
    let params = family_enrollment_event("biomeGate");

    let result = mesh::dispatch_mesh_events_record(&server, params).await.unwrap();
    assert_eq!(result["recorded"], true);
    assert_eq!(result["event_type"], "family_enrollment");
    assert_eq!(result["source_gate"], "biomeGate");
    assert_eq!(result["event_count"], 1);
}

#[tokio::test]
async fn test_mesh_events_record_mesh_leave() {
    let server = create_test_server().await;
    let params = mesh_leave_event("southGate");

    let result = mesh::dispatch_mesh_events_record(&server, params).await.unwrap();
    assert_eq!(result["recorded"], true);
    assert_eq!(result["event_type"], "mesh_leave");
    assert_eq!(result["source_gate"], "southGate");
    assert_eq!(result["event_count"], 1);
}

#[tokio::test]
async fn test_mesh_events_record_via_handle_request() {
    let server = create_test_server().await;
    let req = make_request("mesh.events.record", Some(trust_issuer_event("eastGate")));

    let result = handle_request(&server, req, &test_gate(), &test_caller()).await.unwrap();
    assert_eq!(result["recorded"], true);
    assert_eq!(result["event_type"], "trust_issuer_registered");
    assert_eq!(result["source_gate"], "eastGate");
    assert_eq!(result["event_count"], 1);
}

// ============================================================================
// Multiple event recording with event_count tracking
// ============================================================================

#[tokio::test]
async fn test_mesh_events_record_multiple_increments_event_count() {
    let server = create_test_server().await;

    let r1 =
        mesh::dispatch_mesh_events_record(&server, trust_issuer_event("eastGate")).await.unwrap();
    assert_eq!(r1["event_count"], 1);

    let r2 =
        mesh::dispatch_mesh_events_record(&server, key_exchange_event("strandGate")).await.unwrap();
    assert_eq!(r2["event_count"], 2);

    let r3 = mesh::dispatch_mesh_events_record(&server, mesh_join_event("westGate")).await.unwrap();
    assert_eq!(r3["event_count"], 3);

    let r4 = mesh::dispatch_mesh_events_record(&server, family_enrollment_event("biomeGate"))
        .await
        .unwrap();
    assert_eq!(r4["event_count"], 4);

    let r5 =
        mesh::dispatch_mesh_events_record(&server, mesh_leave_event("southGate")).await.unwrap();
    assert_eq!(r5["event_count"], 5);

    assert_eq!(server.primal.mesh_listener().event_count().await, 5);
}

// ============================================================================
// Invalid params
// ============================================================================

#[tokio::test]
async fn test_mesh_events_record_missing_source_gate() {
    let server = create_test_server().await;
    let params = json!({
        "kind": {
            "type": "TrustIssuerRegistered",
            "payload": { "issuer_fingerprint": "abc" }
        },
        "timestamp": 1_717_444_800
    });

    let err = mesh::dispatch_mesh_events_record(&server, params).await.unwrap_err();
    assert!(matches!(err, HandlerError::InvalidParams(_)));
}

#[tokio::test]
async fn test_mesh_events_record_missing_kind() {
    let server = create_test_server().await;
    let params = json!({
        "source_gate": "eastGate",
        "timestamp": 1_717_444_800
    });

    let err = mesh::dispatch_mesh_events_record(&server, params).await.unwrap_err();
    assert!(matches!(err, HandlerError::InvalidParams(_)));
}

#[tokio::test]
async fn test_mesh_events_record_wrong_timestamp_type() {
    let server = create_test_server().await;
    let params = json!({
        "kind": {
            "type": "TrustIssuerRegistered",
            "payload": { "issuer_fingerprint": "abc" }
        },
        "source_gate": "eastGate",
        "timestamp": "not-a-number"
    });

    let err = mesh::dispatch_mesh_events_record(&server, params).await.unwrap_err();
    assert!(matches!(err, HandlerError::InvalidParams(_)));
}

#[tokio::test]
async fn test_mesh_events_record_unknown_event_kind() {
    let server = create_test_server().await;
    let params = json!({
        "kind": {
            "type": "UnknownMeshEvent",
            "payload": {}
        },
        "source_gate": "eastGate",
        "timestamp": 1_717_444_800
    });

    let err = mesh::dispatch_mesh_events_record(&server, params).await.unwrap_err();
    assert!(matches!(err, HandlerError::InvalidParams(_)));
}

#[tokio::test]
async fn test_mesh_events_record_params_not_object() {
    let server = create_test_server().await;
    let params = json!([1, 2, 3]);

    let err = mesh::dispatch_mesh_events_record(&server, params).await.unwrap_err();
    assert!(matches!(err, HandlerError::InvalidParams(_)));
}

#[tokio::test]
async fn test_mesh_events_record_params_null_via_handle_request() {
    let server = create_test_server().await;
    let req = make_request("mesh.events.record", None);

    let err = handle_request(&server, req, &test_gate(), &test_caller()).await.unwrap_err();
    assert!(matches!(err, HandlerError::InvalidParams(_)));
}

#[tokio::test]
async fn test_mesh_events_record_invalid_mesh_leave_reason() {
    let server = create_test_server().await;
    let params = json!({
        "kind": {
            "type": "MeshLeave",
            "payload": {
                "mesh_id": "mesh-1",
                "reason": "NotAValidReason"
            }
        },
        "source_gate": "eastGate",
        "timestamp": 1_717_444_800
    });

    let err = mesh::dispatch_mesh_events_record(&server, params).await.unwrap_err();
    assert!(matches!(err, HandlerError::InvalidParams(_)));
}

#[tokio::test]
async fn test_mesh_events_record_missing_payload_field() {
    let server = create_test_server().await;
    let params = json!({
        "kind": {
            "type": "KeyExchangeCompleted",
            "payload": { "remote_gate": "ironGate" }
        },
        "source_gate": "strandGate",
        "timestamp": 1_717_444_800
    });

    let err = mesh::dispatch_mesh_events_record(&server, params).await.unwrap_err();
    assert!(matches!(err, HandlerError::InvalidParams(_)));
}
