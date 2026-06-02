// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2024–2026 ecoPrimals Project

//! Composition payload tests (provenance trio + JH-5 Phase 3 readiness).

#![allow(clippy::unwrap_used)]

use super::test_support::{create_test_primal, make_request, test_caller, test_gate};
use super::*;
use serde_json::json;

#[tokio::test]
async fn test_composition_skunkbat_security_event_forwarding() {
    let primal = create_test_primal().await;

    let req = make_request(
        "dag.session.create",
        Some(json!({"session_type": "General", "description": "security-audit-pipeline"})),
    );
    let session_id =
        handle_request(primal.clone(), req, &test_gate(), &test_caller()).await.unwrap();
    let session_id = session_id.as_str().unwrap();

    let req = make_request(
        "dag.event.append",
        Some(json!({
            "session_id": session_id,
            "event_type": {"Custom": {"domain": "security", "event_name": "gate_rejection"}},
            "agent": "did:key:z6MkSkunkBat",
            "metadata": [
                {"key": "severity", "value": "high"},
                {"key": "source", "value": "skunkbat"},
                {"key": "method", "value": "dag.session.create"},
                {"key": "correlation_id", "value": "evt-001"}
            ]
        })),
    );
    let result = handle_request(primal.clone(), req, &test_gate(), &test_caller()).await.unwrap();
    assert!(result.as_str().is_some(), "should return vertex_id hex");
    let vertex_id = result.as_str().unwrap();
    assert_eq!(vertex_id.len(), 64);
}

#[tokio::test]
async fn test_composition_security_event_batch_forwarding() {
    let primal = create_test_primal().await;

    let req = make_request(
        "dag.session.create",
        Some(json!({"session_type": "General", "description": "batch-audit"})),
    );
    let session_id =
        handle_request(primal.clone(), req, &test_gate(), &test_caller()).await.unwrap();
    let session_id = session_id.as_str().unwrap();

    let events = vec![
        json!({
            "session_id": session_id,
            "event_type": {"Custom": {"domain": "security", "event_name": "gate_rejection"}},
            "agent": "did:key:z6MkSkunkBat",
            "metadata": [{"key": "severity", "value": "high"}]
        }),
        json!({
            "session_id": session_id,
            "event_type": {"Custom": {"domain": "security", "event_name": "btsp_negotiate"}},
            "agent": "did:key:z6MkSkunkBat",
            "metadata": [{"key": "severity", "value": "info"}]
        }),
        json!({
            "session_id": session_id,
            "event_type": {"Custom": {"domain": "security", "event_name": "threat_detected"}},
            "agent": "did:key:z6MkSkunkBat",
            "metadata": [{"key": "severity", "value": "critical"}]
        }),
    ];

    let req = make_request("dag.event.append_batch", Some(json!({"requests": events})));
    let result = handle_request(primal.clone(), req, &test_gate(), &test_caller()).await.unwrap();
    let ids = result.as_array().unwrap();
    assert_eq!(ids.len(), 3, "batch should return 3 vertex IDs");
    for id in ids {
        assert_eq!(id.as_str().unwrap().len(), 64);
    }
}

#[tokio::test]
async fn test_composition_provenance_trio_full_pipeline() {
    let primal = create_test_primal().await;

    let req = make_request(
        "dag.session.create",
        Some(json!({"session_type": "General", "description": "provenance-trio-pipeline"})),
    );
    let session_id =
        handle_request(primal.clone(), req, &test_gate(), &test_caller()).await.unwrap();
    let session_id = session_id.as_str().unwrap();

    let req = make_request(
        "dag.event.append",
        Some(json!({
            "session_id": session_id,
            "event_type": {"SessionStart": null},
            "agent": "did:key:z6MkOrchestrator"
        })),
    );
    let v1 = handle_request(primal.clone(), req, &test_gate(), &test_caller())
        .await
        .unwrap()
        .as_str()
        .unwrap()
        .to_string();

    let req = make_request(
        "dag.event.append",
        Some(json!({
            "session_id": session_id,
            "event_type": {"DataCreate": {"schema": "rootpulse-commit"}},
            "agent": "did:key:z6MkOrchestrator",
            "parents": [v1],
            "payload_ref": "ipfs://QmRootPulseData123"
        })),
    );
    let v2 = handle_request(primal.clone(), req, &test_gate(), &test_caller())
        .await
        .unwrap()
        .as_str()
        .unwrap()
        .to_string();

    let req = make_request(
        "dag.event.append",
        Some(json!({
            "session_id": session_id,
            "event_type": {"SessionEnd": {"outcome": "Success"}},
            "agent": "did:key:z6MkOrchestrator",
            "parents": [v2]
        })),
    );
    let _ = handle_request(primal.clone(), req, &test_gate(), &test_caller()).await.unwrap();

    let req = make_request("dag.merkle.root", Some(json!({"session_id": session_id})));
    let root = handle_request(primal.clone(), req, &test_gate(), &test_caller()).await.unwrap();
    let root_hex = root.as_str().unwrap();
    assert_eq!(root_hex.len(), 64, "Merkle root should be 64-char hex (32 bytes)");

    let session_req = make_request("dag.session.get", Some(json!({"session_id": session_id})));
    let info =
        handle_request(primal.clone(), session_req, &test_gate(), &test_caller()).await.unwrap();
    assert_eq!(info["vertex_count"], 3);
}

#[tokio::test]
async fn test_composition_payload_ref_hex_hash() {
    let primal = create_test_primal().await;

    let req = make_request("dag.session.create", Some(json!({"session_type": "General"})));
    let session_id =
        handle_request(primal.clone(), req, &test_gate(), &test_caller()).await.unwrap();
    let session_id = session_id.as_str().unwrap();

    let hex_hash = "a".repeat(64);
    let req = make_request(
        "dag.event.append",
        Some(json!({
            "session_id": session_id,
            "event_type": {"DataCreate": {"schema": null}},
            "payload_ref": hex_hash
        })),
    );
    let result = handle_request(primal.clone(), req, &test_gate(), &test_caller()).await.unwrap();
    let vertex_id = result.as_str().unwrap();

    let req = make_request(
        "dag.vertex.get",
        Some(json!({"session_id": session_id, "vertex_id": vertex_id})),
    );
    let vertex = handle_request(primal.clone(), req, &test_gate(), &test_caller()).await.unwrap();
    let payload = vertex.get("payload").unwrap();
    let payload_hash = payload["hash"].as_array().unwrap();
    assert_eq!(payload_hash.len(), 32, "payload hash should be 32 bytes");
    assert_eq!(payload["size"], 0, "hex-parsed payload_ref has unknown size");
}

#[tokio::test]
async fn test_composition_dehydrate_produces_loamspine_compatible_root() {
    let primal = create_test_primal().await;

    let req = make_request("dag.session.create", Some(json!({"session_type": "General"})));
    let session_id =
        handle_request(primal.clone(), req, &test_gate(), &test_caller()).await.unwrap();
    let session_id = session_id.as_str().unwrap();

    let req = make_request(
        "dag.event.append",
        Some(json!({
            "session_id": session_id,
            "event_type": {"DataCreate": {"schema": "test"}},
            "agent": "did:key:z6MkCommitter"
        })),
    );
    let _ = handle_request(primal.clone(), req, &test_gate(), &test_caller()).await.unwrap();

    let req = make_request("dag.dehydration.trigger", Some(json!({"session_id": session_id})));
    let root = handle_request(primal.clone(), req, &test_gate(), &test_caller()).await.unwrap();
    let root_hex = root.as_str().unwrap();
    assert_eq!(root_hex.len(), 64, "dehydration root should be 64-char hex");
    assert!(
        hex::decode(root_hex).is_ok(),
        "dehydration root must be valid hex for loamSpine session_hash"
    );

    let req = make_request("dag.merkle.root", Some(json!({"session_id": session_id})));
    let merkle_root =
        handle_request(primal.clone(), req, &test_gate(), &test_caller()).await.unwrap();
    assert_eq!(
        root_hex,
        merkle_root.as_str().unwrap(),
        "dehydration root should equal Merkle root"
    );
}
