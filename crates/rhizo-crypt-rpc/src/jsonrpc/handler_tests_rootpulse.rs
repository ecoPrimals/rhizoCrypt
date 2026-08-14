// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2024–2026 ecoPrimals Project

//! rootPulse graph step handler tests.

#![expect(clippy::unwrap_used, reason = "test code")]

use super::test_support::{create_test_server, make_request, test_caller, test_gate};
use super::*;
use serde_json::json;

fn sample_build_params() -> serde_json::Value {
    json!({
        "PRIMAL_NAME": "rhizocrypt",
        "TARGET_TRIPLE": "x86_64-unknown-linux-gnu",
        "COMMIT_SHA": "abc123def456",
        "BLAKE3_HASH": "a".repeat(64),
        "BUILDER_GATE": "eastGate",
        "BUILT_AT": "2026-08-14T10:00:00Z"
    })
}

#[tokio::test]
async fn test_rootpulse_record_build_success() {
    let server = create_test_server().await;

    let req = make_request("rootpulse.record_build", Some(sample_build_params()));
    let resp = handle_request(&server, req, &test_gate(), &test_caller()).await.unwrap();

    let dag_ref = resp["dag_ref"].as_str().unwrap();
    assert!(dag_ref.contains(':'), "dag_ref should be session_id:vertex_id");
    let (session_id, vertex_id) = dag_ref.split_once(':').unwrap();
    assert_eq!(session_id.len(), 36, "session_id should be UUID format");
    assert_eq!(vertex_id.len(), 64, "vertex_id should be 64-char hex");
}

#[tokio::test]
async fn test_rootpulse_record_build_missing_params() {
    let server = create_test_server().await;

    let req = make_request(
        "rootpulse.record_build",
        Some(json!({
            "PRIMAL_NAME": "rhizocrypt",
            "TARGET_TRIPLE": "x86_64-unknown-linux-gnu"
        })),
    );
    let resp = handle_request(&server, req, &test_gate(), &test_caller()).await;
    assert!(resp.is_err(), "missing build params should return InvalidParams");
    assert!(matches!(resp.unwrap_err(), HandlerError::InvalidParams(_)));
}

#[tokio::test]
async fn test_rootpulse_record_build_appends_to_same_session() {
    let server = create_test_server().await;

    let req = make_request("rootpulse.record_build", Some(sample_build_params()));
    let resp1 = handle_request(&server, req, &test_gate(), &test_caller()).await.unwrap();
    let dag_ref1 = resp1["dag_ref"].as_str().unwrap();
    let session1 = dag_ref1.split(':').next().unwrap();

    let mut second = sample_build_params();
    second["COMMIT_SHA"] = json!("second-commit");
    second["BLAKE3_HASH"] = json!("b".repeat(64));
    let req = make_request("rootpulse.record_build", Some(second));
    let resp2 = handle_request(&server, req, &test_gate(), &test_caller()).await.unwrap();
    let dag_ref2 = resp2["dag_ref"].as_str().unwrap();
    let session2 = dag_ref2.split(':').next().unwrap();

    assert_eq!(session1, session2, "same primal+triple should reuse build session");
    assert_ne!(dag_ref1, dag_ref2, "distinct builds should produce distinct dag_refs");
}

#[tokio::test]
async fn test_rootpulse_dehydrate_state_success() {
    let server = create_test_server().await;

    let req = make_request("dag.session.create", Some(json!({"session_type": "General"})));
    let session_id = handle_request(&server, req, &test_gate(), &test_caller()).await.unwrap();
    let session_id = session_id.as_str().unwrap();

    let req = make_request(
        "dag.event.append",
        Some(json!({
            "session_id": session_id,
            "event_type": {"SessionStart": null}
        })),
    );
    let _ = handle_request(&server, req, &test_gate(), &test_caller()).await.unwrap();

    let req = make_request(
        "rootpulse.dehydrate_state",
        Some(json!({
            "SESSION_ID": session_id,
            "AGENT_DID": "did:key:z6MkRootPulse",
            "FAMILY_ID": "ecoprimal-test"
        })),
    );
    let resp = handle_request(&server, req, &test_gate(), &test_caller()).await.unwrap();

    let blob = resp["dehydrated_blob"].as_str().unwrap();
    let hash = resp["content_hash"].as_str().unwrap();
    assert_eq!(blob.len(), 64);
    assert_eq!(blob, hash);
}

#[tokio::test]
async fn test_rootpulse_dehydrate_state_invalid_session() {
    let server = create_test_server().await;

    let req = make_request(
        "rootpulse.dehydrate_state",
        Some(json!({"SESSION_ID": "00000000-0000-7000-8000-000000000000"})),
    );
    let resp = handle_request(&server, req, &test_gate(), &test_caller()).await;
    assert!(resp.is_err(), "nonexistent session should error");
}

#[tokio::test]
async fn test_dag_append_alias_routes_to_event_append() {
    let server = create_test_server().await;

    let req = make_request("dag.session.create", Some(json!({"session_type": "General"})));
    let session_id = handle_request(&server, req, &test_gate(), &test_caller()).await.unwrap();
    let session_id = session_id.as_str().unwrap();

    let req = make_request(
        "dag.append",
        Some(json!({
            "session_id": session_id,
            "event_type": {"DataCreate": {"schema": "alias-test"}}
        })),
    );
    let resp = handle_request(&server, req, &test_gate(), &test_caller()).await;
    assert!(resp.is_ok(), "dag.append should alias to dag.event.append");
    assert_eq!(resp.unwrap().as_str().unwrap().len(), 64);
}

#[tokio::test]
async fn test_dehydrate_alias_routes_to_dehydration_trigger() {
    let server = create_test_server().await;

    let req = make_request("dag.session.create", Some(json!({"session_type": "General"})));
    let session_id = handle_request(&server, req, &test_gate(), &test_caller()).await.unwrap();
    let session_id = session_id.as_str().unwrap();

    let req = make_request(
        "dag.event.append",
        Some(json!({
            "session_id": session_id,
            "event_type": {"SessionStart": null}
        })),
    );
    let _ = handle_request(&server, req, &test_gate(), &test_caller()).await.unwrap();

    let req = make_request("dehydrate", Some(json!({"session_id": session_id})));
    let resp = handle_request(&server, req, &test_gate(), &test_caller()).await;
    assert!(resp.is_ok(), "dehydrate should alias to dag.dehydration.trigger");
    assert_eq!(resp.unwrap().as_str().unwrap().len(), 64);
}

#[tokio::test]
async fn test_dehydration_execute_alias_routes_to_dehydration_trigger() {
    let server = create_test_server().await;

    let req = make_request("dag.session.create", Some(json!({"session_type": "General"})));
    let session_id = handle_request(&server, req, &test_gate(), &test_caller()).await.unwrap();
    let session_id = session_id.as_str().unwrap();

    let req = make_request(
        "dag.event.append",
        Some(json!({
            "session_id": session_id,
            "event_type": {"SessionStart": null}
        })),
    );
    let _ = handle_request(&server, req, &test_gate(), &test_caller()).await.unwrap();

    let req = make_request("dehydration.execute", Some(json!({"session_id": session_id})));
    let resp = handle_request(&server, req, &test_gate(), &test_caller()).await;
    assert!(resp.is_ok(), "dehydration.execute should alias to dag.dehydration.trigger");
    assert_eq!(resp.unwrap().as_str().unwrap().len(), 64);
}
