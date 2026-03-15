// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2024–2026 ecoPrimals Project

//! E2E tests for DAG operations.
//!
//! Tests vertex linking, frontier tracking, and DAG traversal.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use rhizo_crypt_core::{
    DagStore, EventType, PrimalLifecycle, RhizoCrypt, RhizoCryptConfig, SessionBuilder,
    SessionType, VertexBuilder,
};

/// Test DAG frontier tracking.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_dag_frontier() {
    let config = RhizoCryptConfig::default();
    let mut primal = RhizoCrypt::new(config);
    primal.start().await.expect("primal should start");

    // Create a session
    let session = SessionBuilder::new(SessionType::General).build();
    let session_id = primal.create_session(session).expect("should create session");

    // Append genesis vertex
    let v1 = VertexBuilder::new(EventType::SessionStart).build();
    let v1_id = primal.append_vertex(session_id, v1).await.expect("should append genesis");

    // Check frontier contains v1
    let session = primal.get_session(session_id).expect("should get session");
    assert!(session.frontier.contains(&v1_id));
    assert!(session.genesis.contains(&v1_id));

    // Append child
    let v2 = VertexBuilder::new(EventType::Custom {
        domain: "test".into(),
        event_name: "step".into(),
    })
    .with_parent(v1_id)
    .build();
    let v2_id = primal.append_vertex(session_id, v2).await.expect("should append child");

    // Frontier should now be v2 only
    let session = primal.get_session(session_id).expect("should get session");
    assert!(!session.frontier.contains(&v1_id));
    assert!(session.frontier.contains(&v2_id));

    // Genesis should still be v1
    assert!(session.genesis.contains(&v1_id));

    primal.stop().await.expect("primal should stop");
}

/// Test DAG with multiple branches.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_dag_branching() {
    let config = RhizoCryptConfig::default();
    let mut primal = RhizoCrypt::new(config);
    primal.start().await.expect("primal should start");

    // Create a session
    let session = SessionBuilder::new(SessionType::General).build();
    let session_id = primal.create_session(session).expect("should create session");

    // Create genesis
    let v1 = VertexBuilder::new(EventType::SessionStart).build();
    let v1_id = primal.append_vertex(session_id, v1).await.expect("should append genesis");

    // Create two branches from genesis
    let v2 = VertexBuilder::new(EventType::Custom {
        domain: "test".into(),
        event_name: "branch_a".into(),
    })
    .with_parent(v1_id)
    .build();
    let v2_id = primal.append_vertex(session_id, v2).await.expect("should append branch a");

    let v3 = VertexBuilder::new(EventType::Custom {
        domain: "test".into(),
        event_name: "branch_b".into(),
    })
    .with_parent(v1_id)
    .build();
    let v3_id = primal.append_vertex(session_id, v3).await.expect("should append branch b");

    // Frontier should have both tips
    let session = primal.get_session(session_id).expect("should get session");
    assert_eq!(session.frontier.len(), 2);
    assert!(session.frontier.contains(&v2_id));
    assert!(session.frontier.contains(&v3_id));

    // Merge the branches
    let v4 = VertexBuilder::new(EventType::Custom {
        domain: "test".into(),
        event_name: "merge".into(),
    })
    .with_parent(v2_id)
    .with_parent(v3_id)
    .build();
    let v4_id = primal.append_vertex(session_id, v4).await.expect("should append merge");

    // Frontier should now be just v4
    let session = primal.get_session(session_id).expect("should get session");
    assert_eq!(session.frontier.len(), 1);
    assert!(session.frontier.contains(&v4_id));

    primal.stop().await.expect("primal should stop");
}

/// Test DAG children retrieval.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_dag_children() {
    let config = RhizoCryptConfig::default();
    let mut primal = RhizoCrypt::new(config);
    primal.start().await.expect("primal should start");

    // Create a session
    let session = SessionBuilder::new(SessionType::General).build();
    let session_id = primal.create_session(session).expect("should create session");

    // Create parent
    let parent = VertexBuilder::new(EventType::SessionStart).build();
    let parent_id = primal.append_vertex(session_id, parent).await.expect("should append parent");

    // Create children
    let mut child_ids = Vec::new();
    for i in 0..3 {
        let child = VertexBuilder::new(EventType::Custom {
            domain: "test".into(),
            event_name: format!("child-{i}"),
        })
        .with_parent(parent_id)
        .build();
        let id = primal.append_vertex(session_id, child).await.expect("should append child");
        child_ids.push(id);
    }

    // Verify children via dag store
    let dag_store = primal.dag_store().await.expect("should get dag store");
    let children =
        dag_store.get_children(session_id, parent_id).await.expect("should get children");
    assert_eq!(children.len(), 3);
    for id in &child_ids {
        assert!(children.contains(id));
    }

    primal.stop().await.expect("primal should stop");
}
