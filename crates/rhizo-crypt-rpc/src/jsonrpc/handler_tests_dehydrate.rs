// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2024–2026 ecoPrimals Project

//! `dag.partial_dehydrate` tests (wetSpring upstream ask).

#![allow(clippy::unwrap_used)]

use super::test_support::{create_test_primal, make_request, test_caller, test_gate};
use super::*;
use serde_json::json;

#[tokio::test]
async fn test_partial_dehydrate_all_vertices() {
    let primal = create_test_primal().await;

    let req = make_request("dag.session.create", Some(json!({"session_type": "General"})));
    let session_id =
        handle_request(primal.clone(), req, &test_gate(), &test_caller()).await.unwrap();
    let session_id = session_id.as_str().unwrap();

    for i in 0..3 {
        let req = make_request(
            "dag.event.append",
            Some(json!({
                "session_id": session_id,
                "event_type": {"DataCreate": {"schema": format!("clone-{i}")}},
                "agent": "did:key:z6MkWetSpring"
            })),
        );
        let _ = handle_request(primal.clone(), req, &test_gate(), &test_caller()).await.unwrap();
    }

    let req = make_request("dag.partial_dehydrate", Some(json!({"session_id": session_id})));
    let resp = handle_request(primal.clone(), req, &test_gate(), &test_caller()).await.unwrap();

    assert_eq!(resp["sealed_count"], 3);
    assert_eq!(resp["open_count"], 0);
    assert!(resp["session_open"].as_bool().unwrap());
    assert_eq!(resp["merkle_root"].as_str().unwrap().len(), 64);

    let req = make_request("dag.merkle.root", Some(json!({"session_id": session_id})));
    let full_root =
        handle_request(primal.clone(), req, &test_gate(), &test_caller()).await.unwrap();
    assert_eq!(
        resp["merkle_root"].as_str().unwrap(),
        full_root.as_str().unwrap(),
        "partial_dehydrate with no filter should match full merkle root"
    );
}

#[tokio::test]
async fn test_partial_dehydrate_subset() {
    let primal = create_test_primal().await;

    let req = make_request("dag.session.create", Some(json!({"session_type": "General"})));
    let session_id =
        handle_request(primal.clone(), req, &test_gate(), &test_caller()).await.unwrap();
    let session_id = session_id.as_str().unwrap();

    let mut vertex_ids = Vec::new();
    for i in 0..3 {
        let req = make_request(
            "dag.event.append",
            Some(json!({
                "session_id": session_id,
                "event_type": {"DataCreate": {"schema": format!("clone-{i}")}},
            })),
        );
        let vid = handle_request(primal.clone(), req, &test_gate(), &test_caller()).await.unwrap();
        vertex_ids.push(vid.as_str().unwrap().to_owned());
    }

    let req = make_request(
        "dag.partial_dehydrate",
        Some(json!({
            "session_id": session_id,
            "vertex_ids": [vertex_ids[0], vertex_ids[1]]
        })),
    );
    let resp = handle_request(primal.clone(), req, &test_gate(), &test_caller()).await.unwrap();

    assert_eq!(resp["sealed_count"], 2);
    assert_eq!(resp["open_count"], 1);
    assert!(resp["session_open"].as_bool().unwrap());
    assert_eq!(resp["merkle_root"].as_str().unwrap().len(), 64);
}

#[tokio::test]
async fn test_partial_dehydrate_does_not_close_session() {
    let primal = create_test_primal().await;

    let req = make_request("dag.session.create", Some(json!({"session_type": "General"})));
    let session_id =
        handle_request(primal.clone(), req, &test_gate(), &test_caller()).await.unwrap();
    let session_id = session_id.as_str().unwrap();

    let req = make_request(
        "dag.event.append",
        Some(json!({"session_id": session_id, "event_type": {"SessionStart": null}})),
    );
    let _ = handle_request(primal.clone(), req, &test_gate(), &test_caller()).await.unwrap();

    let req = make_request("dag.partial_dehydrate", Some(json!({"session_id": session_id})));
    let _ = handle_request(primal.clone(), req, &test_gate(), &test_caller()).await.unwrap();

    let req = make_request(
        "dag.event.append",
        Some(json!({
            "session_id": session_id,
            "event_type": {"DataCreate": {"schema": "post-partial"}},
        })),
    );
    let result = handle_request(primal.clone(), req, &test_gate(), &test_caller()).await;
    assert!(result.is_ok(), "should still append after partial_dehydrate");
}

#[tokio::test]
async fn test_partial_dehydrate_via_provenance_alias() {
    let primal = create_test_primal().await;

    let req = make_request("provenance.session.create", Some(json!({"session_type": "General"})));
    let session_id =
        handle_request(primal.clone(), req, &test_gate(), &test_caller()).await.unwrap();
    let session_id = session_id.as_str().unwrap();

    let req = make_request(
        "provenance.event.append",
        Some(json!({
            "session_id": session_id,
            "event_type": {"DataCreate": {"schema": "aglet-test"}},
        })),
    );
    let _ = handle_request(primal.clone(), req, &test_gate(), &test_caller()).await.unwrap();

    let req = make_request("provenance.partial_dehydrate", Some(json!({"session_id": session_id})));
    let resp = handle_request(primal.clone(), req, &test_gate(), &test_caller()).await;
    assert!(resp.is_ok(), "provenance.partial_dehydrate should alias to dag.partial_dehydrate");
    assert_eq!(resp.unwrap()["sealed_count"], 1);
}

#[tokio::test]
async fn test_partial_dehydrate_empty_session() {
    let primal = create_test_primal().await;

    let req = make_request("dag.session.create", Some(json!({"session_type": "General"})));
    let session_id =
        handle_request(primal.clone(), req, &test_gate(), &test_caller()).await.unwrap();
    let session_id = session_id.as_str().unwrap();

    let req = make_request("dag.partial_dehydrate", Some(json!({"session_id": session_id})));
    let resp = handle_request(primal.clone(), req, &test_gate(), &test_caller()).await.unwrap();
    assert_eq!(resp["sealed_count"], 0);
    assert_eq!(resp["open_count"], 0);
}

#[tokio::test]
async fn test_partial_dehydrate_missing_session_id() {
    let primal = create_test_primal().await;

    let req = make_request("dag.partial_dehydrate", Some(json!({})));
    let resp = handle_request(primal, req, &test_gate(), &test_caller()).await;
    assert!(resp.is_err(), "missing session_id should error");
}

#[tokio::test]
async fn test_partial_dehydrate_nonexistent_session() {
    let primal = create_test_primal().await;

    let fake_id = "00000000-0000-7000-8000-000000000000";
    let req = make_request("dag.partial_dehydrate", Some(json!({"session_id": fake_id})));
    let resp = handle_request(primal, req, &test_gate(), &test_caller()).await;
    assert!(resp.is_err(), "nonexistent session should error");
}

#[tokio::test]
async fn test_partial_dehydrate_idempotent() {
    let primal = create_test_primal().await;

    let req = make_request("dag.session.create", Some(json!({"session_type": "General"})));
    let session_id =
        handle_request(primal.clone(), req, &test_gate(), &test_caller()).await.unwrap();
    let session_id = session_id.as_str().unwrap();

    let req = make_request(
        "dag.event.append",
        Some(json!({
            "session_id": session_id,
            "event_type": {"DataCreate": {"schema": "idempotent-test"}},
            "agent": "did:key:z6MkIdempotent"
        })),
    );
    let _ = handle_request(primal.clone(), req, &test_gate(), &test_caller()).await.unwrap();

    let req1 = make_request("dag.partial_dehydrate", Some(json!({"session_id": session_id})));
    let resp1 = handle_request(primal.clone(), req1, &test_gate(), &test_caller()).await.unwrap();

    let req2 = make_request("dag.partial_dehydrate", Some(json!({"session_id": session_id})));
    let resp2 = handle_request(primal.clone(), req2, &test_gate(), &test_caller()).await.unwrap();

    assert_eq!(resp1["sealed_count"], resp2["sealed_count"]);
}
