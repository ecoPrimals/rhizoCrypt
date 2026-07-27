// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2024–2026 ecoPrimals Project

//! Branch / Diff / Merge / Federate handler tests (Wave 60).
//!
//! Extracted from `handler_tests.rs` to keep the main test module manageable.

#![expect(clippy::unwrap_used, reason = "test code")]

use super::test_support::{create_test_server, make_request, test_caller, test_gate};
use super::*;
use rhizo_crypt_core::types_ecosystem::provenance::{ProvenanceChain, VertexRef};
use serde_json::json;

// ============================================================================
// Branch / Diff / Merge / Federate (Wave 60)
// ============================================================================

#[tokio::test]
async fn test_dag_branch_creates_forked_session() {
    let server = create_test_server().await;
    let gate = test_gate();
    let caller = test_caller();

    let req = make_request("dag.session.create", Some(json!({"session_type": "General"})));
    let result = handle_request(&server, req, &gate, &caller).await.unwrap();
    let session_id = result.as_str().unwrap().to_string();

    let req = make_request(
        "dag.event.append",
        Some(json!({
            "session_id": session_id,
            "event_type": {"SessionStart": null}
        })),
    );
    let v1 = handle_request(&server, req, &gate, &caller).await.unwrap();
    let v1_hex = v1.as_str().unwrap().to_string();

    let req = make_request(
        "dag.event.append",
        Some(json!({
            "session_id": session_id,
            "event_type": {"DataCreate": {"schema": "test"}},
            "parents": [v1_hex]
        })),
    );
    let v2 = handle_request(&server, req, &gate, &caller).await.unwrap();
    let v2_hex = v2.as_str().unwrap().to_string();

    let req = make_request(
        "dag.branch",
        Some(json!({
            "session_id": session_id,
            "checkout_vertex": v2_hex,
            "name": "feature-branch"
        })),
    );
    let resp = handle_request(&server, req, &gate, &caller).await.unwrap();
    let resp_obj = resp.as_object().unwrap();
    assert!(resp_obj.get("session_id").is_some());
    assert_eq!(resp_obj["vertex_count"], 2);
    assert_eq!(resp_obj["parent_session_id"].as_str().unwrap(), session_id);
}

#[tokio::test]
async fn test_dag_branch_invalid_vertex_returns_error() {
    let server = create_test_server().await;
    let gate = test_gate();
    let caller = test_caller();

    let req = make_request("dag.session.create", Some(json!({"session_type": "General"})));
    let result = handle_request(&server, req, &gate, &caller).await.unwrap();
    let session_id = result.as_str().unwrap().to_string();

    let fake_hex = hex::encode([0u8; 32]);
    let req = make_request(
        "dag.branch",
        Some(json!({
            "session_id": session_id,
            "checkout_vertex": fake_hex,
        })),
    );
    let resp = handle_request(&server, req, &gate, &caller).await;
    assert!(resp.is_err());
}

#[tokio::test]
async fn test_dag_diff_between_sessions() {
    let server = create_test_server().await;
    let gate = test_gate();
    let caller = test_caller();

    let req = make_request("dag.session.create", Some(json!({"session_type": "General"})));
    let s1 = handle_request(&server, req, &gate, &caller).await.unwrap();
    let s1_id = s1.as_str().unwrap().to_string();

    let req = make_request("dag.session.create", Some(json!({"session_type": "General"})));
    let s2 = handle_request(&server, req, &gate, &caller).await.unwrap();
    let s2_id = s2.as_str().unwrap().to_string();

    let req = make_request(
        "dag.event.append",
        Some(json!({
            "session_id": s1_id,
            "event_type": {"SessionStart": null}
        })),
    );
    handle_request(&server, req, &gate, &caller).await.unwrap();

    let req = make_request(
        "dag.event.append",
        Some(json!({
            "session_id": s2_id,
            "event_type": {"DataCreate": {"schema": "other"}}
        })),
    );
    handle_request(&server, req, &gate, &caller).await.unwrap();

    let req = make_request(
        "dag.diff",
        Some(json!({
            "base_session_id": s1_id,
            "other_session_id": s2_id
        })),
    );
    let resp = handle_request(&server, req, &gate, &caller).await.unwrap();
    let obj = resp.as_object().unwrap();
    assert_eq!(obj["only_in_base"].as_array().unwrap().len(), 1);
    assert_eq!(obj["only_in_other"].as_array().unwrap().len(), 1);
    assert_eq!(obj["common_count"], 0);
}

#[tokio::test]
async fn test_dag_merge_collapses_frontier() {
    let server = create_test_server().await;
    let gate = test_gate();
    let caller = test_caller();

    let req = make_request("dag.session.create", Some(json!({"session_type": "General"})));
    let result = handle_request(&server, req, &gate, &caller).await.unwrap();
    let session_id = result.as_str().unwrap().to_string();

    let req = make_request(
        "dag.event.append",
        Some(json!({
            "session_id": session_id,
            "event_type": {"SessionStart": null}
        })),
    );
    let genesis = handle_request(&server, req, &gate, &caller).await.unwrap();
    let genesis_hex = genesis.as_str().unwrap().to_string();

    let req = make_request(
        "dag.event.append",
        Some(json!({
            "session_id": session_id,
            "event_type": {"DataCreate": {"schema": "a"}},
            "parents": [genesis_hex]
        })),
    );
    let branch_a = handle_request(&server, req, &gate, &caller).await.unwrap();
    let branch_a_hex = branch_a.as_str().unwrap().to_string();

    let req = make_request(
        "dag.event.append",
        Some(json!({
            "session_id": session_id,
            "event_type": {"DataCreate": {"schema": "b"}},
            "parents": [genesis_hex]
        })),
    );
    let branch_b = handle_request(&server, req, &gate, &caller).await.unwrap();
    let branch_b_hex = branch_b.as_str().unwrap().to_string();

    let req = make_request("dag.frontier.get", Some(json!({"session_id": session_id})));
    let frontier = handle_request(&server, req, &gate, &caller).await.unwrap();
    assert_eq!(frontier.as_array().unwrap().len(), 2);

    let req = make_request(
        "dag.merge",
        Some(json!({
            "session_id": session_id,
            "parents": [branch_a_hex, branch_b_hex],
            "event_type": {"DataCreate": {"schema": "merge"}}
        })),
    );
    let merge_id = handle_request(&server, req, &gate, &caller).await.unwrap();
    assert!(merge_id.as_str().is_some());

    let req = make_request("dag.frontier.get", Some(json!({"session_id": session_id})));
    let frontier = handle_request(&server, req, &gate, &caller).await.unwrap();
    assert_eq!(frontier.as_array().unwrap().len(), 1, "merge should collapse frontier to 1");
}

#[tokio::test]
async fn test_dag_merge_rejects_single_parent() {
    let server = create_test_server().await;
    let gate = test_gate();
    let caller = test_caller();

    let req = make_request("dag.session.create", Some(json!({"session_type": "General"})));
    let result = handle_request(&server, req, &gate, &caller).await.unwrap();
    let session_id = result.as_str().unwrap().to_string();

    let req = make_request(
        "dag.event.append",
        Some(json!({
            "session_id": session_id,
            "event_type": {"SessionStart": null}
        })),
    );
    let v1 = handle_request(&server, req, &gate, &caller).await.unwrap();
    let v1_hex = v1.as_str().unwrap().to_string();

    let req = make_request(
        "dag.merge",
        Some(json!({
            "session_id": session_id,
            "parents": [v1_hex],
            "event_type": {"DataCreate": {"schema": "nope"}}
        })),
    );
    let resp = handle_request(&server, req, &gate, &caller).await;
    assert!(resp.is_err(), "merge with <2 parents should fail");
}

#[tokio::test]
async fn test_dag_federate_imports_vertices() {
    let server = create_test_server().await;
    let gate = test_gate();
    let caller = test_caller();

    let req = make_request("dag.session.create", Some(json!({"session_type": "General"})));
    let result = handle_request(&server, req, &gate, &caller).await.unwrap();
    let session_id = result.as_str().unwrap().to_string();

    let vertex_json = json!({
        "parents": [],
        "timestamp": 1_000_000_000_u64,
        "event_type": {"SessionStart": null},
        "metadata": {}
    });

    let req = make_request(
        "dag.federate",
        Some(json!({
            "session_id": session_id,
            "vertices": [vertex_json]
        })),
    );
    let resp = handle_request(&server, req, &gate, &caller).await.unwrap();
    let obj = resp.as_object().unwrap();
    assert_eq!(obj["imported"], 1);
    assert_eq!(obj["skipped"], 0);
    assert!(!obj["frontier"].as_array().unwrap().is_empty());
}

#[tokio::test]
async fn test_dag_federate_skips_duplicates() {
    let server = create_test_server().await;
    let gate = test_gate();
    let caller = test_caller();

    let req = make_request("dag.session.create", Some(json!({"session_type": "General"})));
    let result = handle_request(&server, req, &gate, &caller).await.unwrap();
    let session_id = result.as_str().unwrap().to_string();

    let vertex_json = json!({
        "parents": [],
        "timestamp": 1_000_000_000_u64,
        "event_type": {"SessionStart": null},
        "metadata": {}
    });

    let req = make_request(
        "dag.federate",
        Some(json!({
            "session_id": session_id,
            "vertices": [vertex_json.clone()]
        })),
    );
    handle_request(&server, req, &gate, &caller).await.unwrap();

    let req = make_request(
        "dag.federate",
        Some(json!({
            "session_id": session_id,
            "vertices": [vertex_json]
        })),
    );
    let resp = handle_request(&server, req, &gate, &caller).await.unwrap();
    let obj = resp.as_object().unwrap();
    assert_eq!(obj["imported"], 0);
    assert_eq!(obj["skipped"], 1);
}

#[tokio::test]
async fn test_provenance_aliases_for_wave60_methods() {
    let server = create_test_server().await;
    let gate = test_gate();
    let caller = test_caller();

    let req = make_request("dag.session.create", Some(json!({"session_type": "General"})));
    let result = handle_request(&server, req, &gate, &caller).await.unwrap();
    let session_id = result.as_str().unwrap().to_string();

    let req = make_request(
        "dag.event.append",
        Some(json!({
            "session_id": session_id,
            "event_type": {"SessionStart": null}
        })),
    );
    let v1 = handle_request(&server, req, &gate, &caller).await.unwrap();
    let v1_hex = v1.as_str().unwrap().to_string();

    let req = make_request(
        "provenance.branch",
        Some(json!({
            "session_id": session_id,
            "checkout_vertex": v1_hex,
        })),
    );
    let resp = handle_request(&server, req, &gate, &caller).await;
    assert!(resp.is_ok(), "provenance.branch should alias to dag.branch");
}

#[tokio::test]
async fn test_dag_federate_with_source_gate() {
    let server = create_test_server().await;
    let gate = test_gate();
    let caller = test_caller();

    let req = make_request("dag.session.create", Some(json!({"session_type": "General"})));
    let result = handle_request(&server, req, &gate, &caller).await.unwrap();
    let session_id = result.as_str().unwrap().to_string();

    let vertex_json = json!({
        "parents": [],
        "timestamp": 1_000_000_000_u64,
        "event_type": {"SessionStart": null},
        "metadata": {}
    });

    let req = make_request(
        "dag.federate",
        Some(json!({
            "session_id": session_id,
            "vertices": [vertex_json],
            "source_gate": "ironGate"
        })),
    );
    let resp = handle_request(&server, req, &gate, &caller).await.unwrap();
    let obj = resp.as_object().unwrap();
    assert_eq!(obj["imported"], 1);
    assert_eq!(obj["skipped"], 0);
}

#[tokio::test]
async fn test_dag_federate_rejected_field_absent_when_zero() {
    let server = create_test_server().await;
    let gate = test_gate();
    let caller = test_caller();

    let req = make_request("dag.session.create", Some(json!({"session_type": "General"})));
    let result = handle_request(&server, req, &gate, &caller).await.unwrap();
    let session_id = result.as_str().unwrap().to_string();

    let vertex_json = json!({
        "parents": [],
        "timestamp": 1_000_000_000_u64,
        "event_type": {"SessionStart": null},
        "metadata": {}
    });

    let req = make_request(
        "dag.federate",
        Some(json!({
            "session_id": session_id,
            "vertices": [vertex_json]
        })),
    );
    let resp = handle_request(&server, req, &gate, &caller).await.unwrap();
    let obj = resp.as_object().unwrap();
    assert!(
        !obj.contains_key("rejected"),
        "rejected should be absent when zero (skip_serializing_if)"
    );
}

#[tokio::test]
async fn test_dag_federate_verify_signatures_without_provider() {
    let server = create_test_server().await;
    let gate = test_gate();
    let caller = test_caller();

    let req = make_request("dag.session.create", Some(json!({"session_type": "General"})));
    let result = handle_request(&server, req, &gate, &caller).await.unwrap();
    let session_id = result.as_str().unwrap().to_string();

    let vertex_json = json!({
        "parents": [],
        "timestamp": 1_000_000_000_u64,
        "event_type": {"SessionStart": null},
        "metadata": {}
    });

    let req = make_request(
        "dag.federate",
        Some(json!({
            "session_id": session_id,
            "vertices": [vertex_json],
            "verify_signatures": true
        })),
    );
    let resp = handle_request(&server, req, &gate, &caller).await.unwrap();
    let obj = resp.as_object().unwrap();
    assert_eq!(obj["imported"], 1, "unsigned vertices pass when no signing provider");
}

#[test]
fn test_provenance_chain_from_federated_vertices() {
    use rhizo_crypt_core::VertexBuilder;
    use rhizo_crypt_core::event::EventType;
    use rhizo_crypt_core::types::{Did, SessionId};

    let session_id = SessionId::now();
    let vertices = vec![
        VertexBuilder::new(EventType::SessionStart)
            .with_agent(Did::new("did:key:remote-agent"))
            .build(),
        VertexBuilder::new(EventType::DataCreate {
            schema: Some("external-system".into()),
        })
        .build(),
    ];

    let mut chain = ProvenanceChain::new();
    for v in &vertices {
        if let Ok(vid) = v.compute_id() {
            chain.add_vertex(VertexRef {
                session_id,
                vertex_id: vid,
                event_type: v.event_type.name().to_string(),
                agent: v.agent.clone(),
                timestamp: v.timestamp,
                payload_ref: v.payload,
            });
        }
    }

    assert_eq!(chain.len(), 2);
    assert_eq!(chain.agents.len(), 1, "one agent across two vertices");
    assert_eq!(chain.vertices[0].event_type, "session_start");
    assert_eq!(chain.vertices[1].event_type, "data_create");
}

#[test]
fn test_provenance_chain_empty_when_no_vertices() {
    let chain = ProvenanceChain::new();
    assert!(chain.is_empty());
    assert_eq!(chain.agents.len(), 0);
    assert_eq!(chain.data_hashes.len(), 0);
}

#[tokio::test]
async fn test_dag_federate_provenance_notifier_accessible() {
    let server = create_test_server().await;
    let notifier = server.primal.provenance_notifier();
    assert!(
        std::sync::Arc::strong_count(notifier) >= 1,
        "provenance_notifier must be accessible from RPC layer"
    );
}
