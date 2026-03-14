// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2024–2026 ecoPrimals Project

#![allow(clippy::unwrap_used, clippy::expect_used)]

use rhizo_crypt_core::{
    DagStore, EventType, PrimalLifecycle, RhizoCrypt, RhizoCryptConfig, SessionBuilder,
    SessionType, VertexBuilder,
};

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_merkle_root_determinism() {
    let config = RhizoCryptConfig::default();
    let mut primal = RhizoCrypt::new(config);
    primal.start().await.expect("primal should start");

    let session = SessionBuilder::new(SessionType::General).build();
    let session_id = primal.create_session(session).expect("should create session");

    let v1 = VertexBuilder::new(EventType::SessionStart).build();
    let v2 = VertexBuilder::new(EventType::DataCreate {
        schema: None,
    })
    .with_parent(primal.append_vertex(session_id, v1).await.expect("append v1"))
    .build();
    primal.append_vertex(session_id, v2).await.expect("should append v2");

    let root1 = primal.compute_merkle_root(session_id).await.expect("compute root 1");
    let root2 = primal.compute_merkle_root(session_id).await.expect("compute root 2");

    assert_eq!(root1.as_bytes(), root2.as_bytes());

    primal.stop().await.expect("primal should stop");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_merkle_proof_generation_and_verification() {
    let config = RhizoCryptConfig::default();
    let mut primal = RhizoCrypt::new(config);
    primal.start().await.expect("primal should start");

    let session = SessionBuilder::new(SessionType::General).build();
    let session_id = primal.create_session(session).expect("should create session");

    let v1 = VertexBuilder::new(EventType::SessionStart).build();
    let v1_id = primal.append_vertex(session_id, v1).await.expect("append v1");
    let v2 = VertexBuilder::new(EventType::DataCreate {
        schema: None,
    })
    .with_parent(v1_id)
    .build();
    let v2_id = primal.append_vertex(session_id, v2).await.expect("append v2");

    let proof = primal.generate_merkle_proof(session_id, v2_id).await.expect("generate proof");

    let vertex = primal.get_vertex(session_id, v2_id).await.expect("get vertex");
    assert!(proof.verify(&vertex));

    primal.stop().await.expect("primal should stop");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_merkle_root_changes_on_append() {
    let config = RhizoCryptConfig::default();
    let mut primal = RhizoCrypt::new(config);
    primal.start().await.expect("primal should start");

    let session = SessionBuilder::new(SessionType::General).build();
    let session_id = primal.create_session(session).expect("should create session");

    let v1 = VertexBuilder::new(EventType::SessionStart).build();
    let v1_id = primal.append_vertex(session_id, v1).await.expect("append v1");

    let root_before = primal.compute_merkle_root(session_id).await.expect("root before append");

    let v2 = VertexBuilder::new(EventType::DataCreate {
        schema: None,
    })
    .with_parent(v1_id)
    .build();
    primal.append_vertex(session_id, v2).await.expect("append v2");

    let root_after = primal.compute_merkle_root(session_id).await.expect("root after append");

    assert_ne!(root_before.as_bytes(), root_after.as_bytes());

    primal.stop().await.expect("primal should stop");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_merkle_with_branching_dag() {
    let config = RhizoCryptConfig::default();
    let mut primal = RhizoCrypt::new(config);
    primal.start().await.expect("primal should start");

    let session = SessionBuilder::new(SessionType::General).build();
    let session_id = primal.create_session(session).expect("should create session");

    let v1 = VertexBuilder::new(EventType::SessionStart).build();
    let v1_id = primal.append_vertex(session_id, v1).await.expect("append genesis");

    let v2 = VertexBuilder::new(EventType::Custom {
        domain: "test".into(),
        event_name: "branch_a".into(),
    })
    .with_parent(v1_id)
    .build();
    let v2_id = primal.append_vertex(session_id, v2).await.expect("append branch a");

    let v3 = VertexBuilder::new(EventType::Custom {
        domain: "test".into(),
        event_name: "branch_b".into(),
    })
    .with_parent(v1_id)
    .build();
    let v3_id = primal.append_vertex(session_id, v3).await.expect("append branch b");

    let v4 = VertexBuilder::new(EventType::Custom {
        domain: "test".into(),
        event_name: "merge".into(),
    })
    .with_parent(v2_id)
    .with_parent(v3_id)
    .build();
    let v4_id = primal.append_vertex(session_id, v4).await.expect("append merge");

    let root = primal.compute_merkle_root(session_id).await.expect("compute root");
    assert_ne!(*root.as_bytes(), [0u8; 32]);

    let dag_store = primal.dag_store().await.expect("get dag store");
    let vertices = dag_store.get_all_vertices(session_id).await.expect("get vertices");
    assert_eq!(vertices.len(), 4);

    let proof = primal.generate_merkle_proof(session_id, v4_id).await.expect("generate proof");
    let v4_vertex = primal.get_vertex(session_id, v4_id).await.expect("get v4");
    assert!(proof.verify(&v4_vertex));

    primal.stop().await.expect("primal should stop");
}
