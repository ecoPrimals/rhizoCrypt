// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2024–2026 ecoPrimals Project

//! GAP-36: provenance.* wire-name alias tests + session summary fields.

#![allow(clippy::unwrap_used)]

use super::test_support::{create_test_primal, make_request, test_caller, test_gate};
use super::*;
use serde_json::json;

#[tokio::test]
async fn test_provenance_session_create_alias() {
    let primal = create_test_primal().await;

    let req = make_request(
        "provenance.session.create",
        Some(json!({"session_type": "General", "description": "alias-test"})),
    );
    let result = handle_request(primal.clone(), req, &test_gate(), &test_caller()).await;
    assert!(result.is_ok(), "provenance.session.create should route to dag.session.create");
    let session_id = result.unwrap();
    assert!(session_id.as_str().is_some(), "should return session_id string");
}

#[tokio::test]
async fn test_provenance_event_append_alias() {
    let primal = create_test_primal().await;

    let req = make_request("provenance.session.create", Some(json!({"session_type": "General"})));
    let session_id =
        handle_request(primal.clone(), req, &test_gate(), &test_caller()).await.unwrap();
    let session_id = session_id.as_str().unwrap();

    let req = make_request(
        "provenance.event.append",
        Some(json!({
            "session_id": session_id,
            "event_type": {"Custom": {"domain": "security", "event_name": "audit"}},
            "agent": "did:key:z6MkTest"
        })),
    );
    let result = handle_request(primal.clone(), req, &test_gate(), &test_caller()).await;
    assert!(result.is_ok(), "provenance.event.append should route to dag.event.append");
    let vertex_id = result.unwrap();
    assert_eq!(vertex_id.as_str().unwrap().len(), 64);
}

#[tokio::test]
async fn test_provenance_dehydrate_alias() {
    let primal = create_test_primal().await;

    let req = make_request("provenance.session.create", Some(json!({"session_type": "General"})));
    let session_id =
        handle_request(primal.clone(), req, &test_gate(), &test_caller()).await.unwrap();
    let session_id = session_id.as_str().unwrap();

    let req = make_request(
        "provenance.event.append",
        Some(json!({
            "session_id": session_id,
            "event_type": {"SessionStart": null}
        })),
    );
    let _ = handle_request(primal.clone(), req, &test_gate(), &test_caller()).await.unwrap();

    let req =
        make_request("provenance.dehydration.trigger", Some(json!({"session_id": session_id})));
    let root = handle_request(primal.clone(), req, &test_gate(), &test_caller()).await;
    assert!(root.is_ok(), "provenance.dehydration.trigger should route to dag.dehydration.trigger");
    assert_eq!(root.unwrap().as_str().unwrap().len(), 64);
}

#[tokio::test]
async fn test_provenance_full_pipeline_via_aliases() {
    let primal = create_test_primal().await;

    let req = make_request("provenance.session.create", Some(json!({"session_type": "General"})));
    let session_id =
        handle_request(primal.clone(), req, &test_gate(), &test_caller()).await.unwrap();
    let session_id = session_id.as_str().unwrap();

    let req = make_request(
        "provenance.event.append",
        Some(json!({
            "session_id": session_id,
            "event_type": {"DataCreate": {"schema": "clinical-data"}},
            "agent": "did:key:z6MkHealthSpring"
        })),
    );
    let _ = handle_request(primal.clone(), req, &test_gate(), &test_caller()).await.unwrap();

    let req = make_request("provenance.merkle.root", Some(json!({"session_id": session_id})));
    let root = handle_request(primal.clone(), req, &test_gate(), &test_caller()).await.unwrap();
    assert_eq!(root.as_str().unwrap().len(), 64);

    let req = make_request("provenance.session.get", Some(json!({"session_id": session_id})));
    let info = handle_request(primal.clone(), req, &test_gate(), &test_caller()).await.unwrap();
    assert_eq!(info["vertex_count"], 1);

    let req = make_request("provenance.session.list", None);
    let list = handle_request(primal.clone(), req, &test_gate(), &test_caller()).await.unwrap();
    assert!(!list.as_array().unwrap().is_empty());
}

#[tokio::test]
async fn test_session_get_returns_summary_fields() {
    let primal = create_test_primal().await;

    let req = make_request(
        "dag.session.create",
        Some(json!({"session_type": "General", "description": "summary-test"})),
    );
    let session_id =
        handle_request(primal.clone(), req, &test_gate(), &test_caller()).await.unwrap();
    let session_id = session_id.as_str().unwrap();

    let req = make_request(
        "dag.event.append",
        Some(json!({
            "session_id": session_id,
            "event_type": {"DataCreate": {"schema": "cathedral"}},
            "agent": "did:key:z6MkCATHEDRAL"
        })),
    );
    let _ = handle_request(primal.clone(), req, &test_gate(), &test_caller()).await.unwrap();

    let req = make_request(
        "dag.event.append",
        Some(json!({
            "session_id": session_id,
            "event_type": {"DataCreate": {"schema": "litho"}},
            "agent": "did:key:z6MkLithoSpore"
        })),
    );
    let _ = handle_request(primal.clone(), req, &test_gate(), &test_caller()).await.unwrap();

    let req = make_request("dag.session.get", Some(json!({"session_id": session_id})));
    let info = handle_request(primal.clone(), req, &test_gate(), &test_caller()).await.unwrap();

    assert_eq!(info["vertex_count"], 2);
    assert_eq!(info["description"], "summary-test");

    let agents = info["agents"].as_array().unwrap();
    assert!(
        agents.iter().any(|a| a.as_str() == Some("did:key:z6MkCATHEDRAL")),
        "agents should include did:key:z6MkCATHEDRAL, got: {agents:?}"
    );
    assert!(
        agents.iter().any(|a| a.as_str() == Some("did:key:z6MkLithoSpore")),
        "agents should include did:key:z6MkLithoSpore, got: {agents:?}"
    );

    let genesis = info["genesis"].as_array().unwrap();
    assert!(!genesis.is_empty(), "genesis should have at least one root vertex");

    let frontier = info["frontier"].as_array().unwrap();
    assert!(!frontier.is_empty(), "frontier should have tip vertices");
}
