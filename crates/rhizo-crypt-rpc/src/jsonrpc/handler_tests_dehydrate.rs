// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2024–2026 ecoPrimals Project

//! `dag.partial_dehydrate` tests (wetSpring upstream ask).

#![expect(clippy::unwrap_used, reason = "test code")]

use super::test_support::{create_test_server, make_request, test_caller, test_gate};
use super::*;
use serde_json::json;

#[tokio::test]
async fn test_partial_dehydrate_all_vertices() {
    let server = create_test_server().await;

    let req = make_request("dag.session.create", Some(json!({"session_type": "General"})));
    let session_id = handle_request(&server, req, &test_gate(), &test_caller()).await.unwrap();
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
        let _ = handle_request(&server, req, &test_gate(), &test_caller()).await.unwrap();
    }

    let req = make_request("dag.partial_dehydrate", Some(json!({"session_id": session_id})));
    let resp = handle_request(&server, req, &test_gate(), &test_caller()).await.unwrap();

    assert_eq!(resp["sealed_count"], 3);
    assert_eq!(resp["open_count"], 0);
    assert!(resp["session_open"].as_bool().unwrap());
    assert_eq!(resp["merkle_root"].as_str().unwrap().len(), 64);

    let req = make_request("dag.merkle.root", Some(json!({"session_id": session_id})));
    let full_root = handle_request(&server, req, &test_gate(), &test_caller()).await.unwrap();
    assert_eq!(
        resp["merkle_root"].as_str().unwrap(),
        full_root.as_str().unwrap(),
        "partial_dehydrate with no filter should match full merkle root"
    );
}

#[tokio::test]
async fn test_partial_dehydrate_subset() {
    let server = create_test_server().await;

    let req = make_request("dag.session.create", Some(json!({"session_type": "General"})));
    let session_id = handle_request(&server, req, &test_gate(), &test_caller()).await.unwrap();
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
        let vid = handle_request(&server, req, &test_gate(), &test_caller()).await.unwrap();
        vertex_ids.push(vid.as_str().unwrap().to_owned());
    }

    let req = make_request(
        "dag.partial_dehydrate",
        Some(json!({
            "session_id": session_id,
            "vertex_ids": [vertex_ids[0], vertex_ids[1]]
        })),
    );
    let resp = handle_request(&server, req, &test_gate(), &test_caller()).await.unwrap();

    assert_eq!(resp["sealed_count"], 2);
    assert_eq!(resp["open_count"], 1);
    assert!(resp["session_open"].as_bool().unwrap());
    assert_eq!(resp["merkle_root"].as_str().unwrap().len(), 64);
}

#[tokio::test]
async fn test_partial_dehydrate_does_not_close_session() {
    let server = create_test_server().await;

    let req = make_request("dag.session.create", Some(json!({"session_type": "General"})));
    let session_id = handle_request(&server, req, &test_gate(), &test_caller()).await.unwrap();
    let session_id = session_id.as_str().unwrap();

    let req = make_request(
        "dag.event.append",
        Some(json!({"session_id": session_id, "event_type": {"SessionStart": null}})),
    );
    let _ = handle_request(&server, req, &test_gate(), &test_caller()).await.unwrap();

    let req = make_request("dag.partial_dehydrate", Some(json!({"session_id": session_id})));
    let _ = handle_request(&server, req, &test_gate(), &test_caller()).await.unwrap();

    let req = make_request(
        "dag.event.append",
        Some(json!({
            "session_id": session_id,
            "event_type": {"DataCreate": {"schema": "post-partial"}},
        })),
    );
    let result = handle_request(&server, req, &test_gate(), &test_caller()).await;
    assert!(result.is_ok(), "should still append after partial_dehydrate");
}

#[tokio::test]
async fn test_partial_dehydrate_via_provenance_alias() {
    let server = create_test_server().await;

    let req = make_request("provenance.session.create", Some(json!({"session_type": "General"})));
    let session_id = handle_request(&server, req, &test_gate(), &test_caller()).await.unwrap();
    let session_id = session_id.as_str().unwrap();

    let req = make_request(
        "provenance.event.append",
        Some(json!({
            "session_id": session_id,
            "event_type": {"DataCreate": {"schema": "aglet-test"}},
        })),
    );
    let _ = handle_request(&server, req, &test_gate(), &test_caller()).await.unwrap();

    let req = make_request("provenance.partial_dehydrate", Some(json!({"session_id": session_id})));
    let resp = handle_request(&server, req, &test_gate(), &test_caller()).await;
    assert!(resp.is_ok(), "provenance.partial_dehydrate should alias to dag.partial_dehydrate");
    assert_eq!(resp.unwrap()["sealed_count"], 1);
}

#[tokio::test]
async fn test_partial_dehydrate_empty_session() {
    let server = create_test_server().await;

    let req = make_request("dag.session.create", Some(json!({"session_type": "General"})));
    let session_id = handle_request(&server, req, &test_gate(), &test_caller()).await.unwrap();
    let session_id = session_id.as_str().unwrap();

    let req = make_request("dag.partial_dehydrate", Some(json!({"session_id": session_id})));
    let resp = handle_request(&server, req, &test_gate(), &test_caller()).await.unwrap();
    assert_eq!(resp["sealed_count"], 0);
    assert_eq!(resp["open_count"], 0);
}

#[tokio::test]
async fn test_partial_dehydrate_missing_session_id() {
    let server = create_test_server().await;

    let req = make_request("dag.partial_dehydrate", Some(json!({})));
    let resp = handle_request(&server, req, &test_gate(), &test_caller()).await;
    assert!(resp.is_err(), "missing session_id should error");
}

#[tokio::test]
async fn test_partial_dehydrate_nonexistent_session() {
    let server = create_test_server().await;

    let fake_id = "00000000-0000-7000-8000-000000000000";
    let req = make_request("dag.partial_dehydrate", Some(json!({"session_id": fake_id})));
    let resp = handle_request(&server, req, &test_gate(), &test_caller()).await;
    assert!(resp.is_err(), "nonexistent session should error");
}

// ============================================================================
// G31 Batch / Pipeline Tests
// ============================================================================

#[tokio::test]
async fn test_dehydrate_batch_multiple_sessions() {
    let server = create_test_server().await;

    let mut session_ids = Vec::new();
    for _ in 0..3 {
        let req = make_request("dag.session.create", Some(json!({"session_type": "General"})));
        let sid = handle_request(&server, req, &test_gate(), &test_caller()).await.unwrap();
        let sid_str = sid.as_str().unwrap().to_owned();

        let req = make_request(
            "dag.event.append",
            Some(json!({
                "session_id": sid_str,
                "event_type": {"DataCreate": {"schema": "batch-dehydrate-test"}},
            })),
        );
        let _ = handle_request(&server, req, &test_gate(), &test_caller()).await.unwrap();
        session_ids.push(sid_str);
    }

    let req =
        make_request("dag.dehydration.trigger_batch", Some(json!({"session_ids": session_ids})));
    let resp = handle_request(&server, req, &test_gate(), &test_caller()).await.unwrap();
    let results = resp.as_array().unwrap();
    assert_eq!(results.len(), 3);
    for result in results {
        assert!(result["success"].as_bool().unwrap(), "each session should dehydrate");
        assert_eq!(result["merkle_root"].as_str().unwrap().len(), 64);
    }
}

#[tokio::test]
async fn test_dehydrate_batch_empty() {
    let server = create_test_server().await;

    let req = make_request("dag.dehydration.trigger_batch", Some(json!({"session_ids": []})));
    let resp = handle_request(&server, req, &test_gate(), &test_caller()).await.unwrap();
    assert!(resp.as_array().unwrap().is_empty());
}

#[tokio::test]
async fn test_dehydrate_batch_mixed_success_and_failure() {
    let server = create_test_server().await;

    let req = make_request("dag.session.create", Some(json!({"session_type": "General"})));
    let good_sid = handle_request(&server, req, &test_gate(), &test_caller()).await.unwrap();
    let good_sid = good_sid.as_str().unwrap().to_owned();

    let req = make_request(
        "dag.event.append",
        Some(json!({
            "session_id": good_sid,
            "event_type": {"SessionStart": null},
        })),
    );
    let _ = handle_request(&server, req, &test_gate(), &test_caller()).await.unwrap();

    let fake_sid = "00000000-0000-7000-8000-000000dead00";
    let req = make_request(
        "dag.dehydration.trigger_batch",
        Some(json!({"session_ids": [good_sid, fake_sid]})),
    );
    let resp = handle_request(&server, req, &test_gate(), &test_caller()).await.unwrap();
    let results = resp.as_array().unwrap();
    assert_eq!(results.len(), 2);

    let successes = results.iter().filter(|r| r["success"].as_bool().unwrap()).count();
    let failures: Vec<_> = results.iter().filter(|r| !r["success"].as_bool().unwrap()).collect();
    assert_eq!(successes, 1, "good session should succeed");
    assert_eq!(failures.len(), 1, "fake session should fail");
    assert!(failures[0]["error"].as_str().is_some(), "failure should have error message");
}

#[tokio::test]
async fn test_dehydrate_batch_via_provenance_alias() {
    let server = create_test_server().await;

    let req = make_request("dag.session.create", Some(json!({"session_type": "General"})));
    let sid = handle_request(&server, req, &test_gate(), &test_caller()).await.unwrap();
    let sid = sid.as_str().unwrap().to_owned();

    let req = make_request(
        "dag.event.append",
        Some(json!({"session_id": sid, "event_type": {"SessionStart": null}})),
    );
    let _ = handle_request(&server, req, &test_gate(), &test_caller()).await.unwrap();

    let req =
        make_request("provenance.dehydration.trigger_batch", Some(json!({"session_ids": [sid]})));
    let resp = handle_request(&server, req, &test_gate(), &test_caller()).await;
    assert!(resp.is_ok(), "provenance alias should work for batch dehydrate");
}

#[tokio::test]
async fn test_pipeline_ingest_creates_session_and_appends() {
    let server = create_test_server().await;

    let req = make_request(
        "dag.pipeline.ingest",
        Some(json!({
            "session_type": "General",
            "description": "pipeline-test",
            "events": [
                {"session_id": "00000000-0000-7000-8000-000000000000", "event_type": {"DataCreate": {"schema": "pdb-1abc"}}, "parents": [], "metadata": {}},
                {"session_id": "00000000-0000-7000-8000-000000000000", "event_type": {"DataCreate": {"schema": "pdb-2def"}}, "parents": [], "metadata": {}},
            ],
            "dehydrate": false,
        })),
    );
    let resp = handle_request(&server, req, &test_gate(), &test_caller()).await.unwrap();

    assert!(resp["session_id"].as_str().is_some(), "should create session");
    assert_eq!(resp["appended"].as_u64().unwrap(), 2);
    assert_eq!(resp["vertex_ids"].as_array().unwrap().len(), 2);
    assert!(resp.get("merkle_root").is_none() || resp["merkle_root"].is_null());
}

#[tokio::test]
async fn test_pipeline_ingest_with_dehydrate() {
    let server = create_test_server().await;

    let req = make_request(
        "dag.pipeline.ingest",
        Some(json!({
            "session_type": "General",
            "events": [
                {"session_id": "00000000-0000-7000-8000-000000000000", "event_type": {"DataCreate": {"schema": "bulk-ingest"}}, "parents": [], "metadata": {}},
            ],
            "dehydrate": true,
        })),
    );
    let resp = handle_request(&server, req, &test_gate(), &test_caller()).await.unwrap();

    assert_eq!(resp["appended"].as_u64().unwrap(), 1);
    assert!(resp["merkle_root"].as_str().is_some(), "should have merkle_root when dehydrate=true");
    assert_eq!(resp["merkle_root"].as_str().unwrap().len(), 64);
}

#[tokio::test]
async fn test_pipeline_ingest_via_provenance_alias() {
    let server = create_test_server().await;

    let req = make_request(
        "provenance.pipeline.ingest",
        Some(json!({
            "session_type": "General",
            "events": [
                {"session_id": "00000000-0000-7000-8000-000000000000", "event_type": {"SessionStart": null}, "parents": [], "metadata": {}},
            ],
            "dehydrate": false,
        })),
    );
    let resp = handle_request(&server, req, &test_gate(), &test_caller()).await;
    assert!(resp.is_ok(), "provenance.pipeline.ingest alias should work");
}

#[tokio::test]
async fn test_pipeline_ingest_empty_events() {
    let server = create_test_server().await;

    let req = make_request(
        "dag.pipeline.ingest",
        Some(json!({
            "session_type": "General",
            "events": [],
            "dehydrate": false,
        })),
    );
    let resp = handle_request(&server, req, &test_gate(), &test_caller()).await.unwrap();
    assert_eq!(resp["appended"].as_u64().unwrap(), 0);
    assert!(resp["vertex_ids"].as_array().unwrap().is_empty());
}

#[tokio::test]
async fn test_partial_dehydrate_idempotent() {
    let server = create_test_server().await;

    let req = make_request("dag.session.create", Some(json!({"session_type": "General"})));
    let session_id = handle_request(&server, req, &test_gate(), &test_caller()).await.unwrap();
    let session_id = session_id.as_str().unwrap();

    let req = make_request(
        "dag.event.append",
        Some(json!({
            "session_id": session_id,
            "event_type": {"DataCreate": {"schema": "idempotent-test"}},
            "agent": "did:key:z6MkIdempotent"
        })),
    );
    let _ = handle_request(&server, req, &test_gate(), &test_caller()).await.unwrap();

    let req1 = make_request("dag.partial_dehydrate", Some(json!({"session_id": session_id})));
    let resp1 = handle_request(&server, req1, &test_gate(), &test_caller()).await.unwrap();

    let req2 = make_request("dag.partial_dehydrate", Some(json!({"session_id": session_id})));
    let resp2 = handle_request(&server, req2, &test_gate(), &test_caller()).await.unwrap();

    assert_eq!(resp1["sealed_count"], resp2["sealed_count"]);
}
