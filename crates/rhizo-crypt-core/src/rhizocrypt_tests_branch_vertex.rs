// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2024–2026 ecoPrimals Project

//! Unit tests for branch operations (branch, diff, merge, federate) and vertex operations (query, merkle).

#![expect(clippy::unwrap_used, clippy::expect_used, reason = "test code")]

use super::*;
use crate::event::EventType;
use crate::session::{SessionBuilder, SessionType};
use crate::types::Did;
use crate::vertex::VertexBuilder;

async fn running_primal() -> RhizoCrypt {
    let config = RhizoCryptConfig::default();
    let mut primal = RhizoCrypt::new(config);
    primal.start().await.unwrap();
    primal
}

async fn append_chain(primal: &RhizoCrypt, session_id: SessionId, count: usize) -> Vec<VertexId> {
    let mut ids = Vec::with_capacity(count);
    for i in 0..count {
        let mut builder = VertexBuilder::new(EventType::DataCreate {
            schema: Some(format!("vertex-{i}")),
        });
        if let Some(parent) = ids.last() {
            builder = builder.with_parent(*parent);
        }
        let vertex = builder.build();
        let id = primal.append_vertex(session_id, vertex).await.unwrap();
        ids.push(id);
    }
    ids
}

// ============================================================================
// Branch operations
// ============================================================================

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_branch_session_basic() {
    let primal = running_primal().await;

    let session = SessionBuilder::new(SessionType::General).build();
    let session_id = primal.create_session(session).unwrap();

    let ids = append_chain(&primal, session_id, 3).await;
    let tip = *ids.last().unwrap();

    let (branch_id, copied) =
        primal.branch_session(session_id, tip, Some("feature".into())).await.unwrap();
    assert_eq!(copied, 3);

    let branch_vertices = primal.get_all_vertices(branch_id).await.unwrap();
    assert_eq!(branch_vertices.len(), 3);

    let original = primal.get_all_vertices(session_id).await.unwrap();
    let original_ids: std::collections::HashSet<_> =
        original.iter().map(|v| v.compute_id().unwrap()).collect();
    for v in &branch_vertices {
        assert!(original_ids.contains(&v.compute_id().unwrap()));
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_branch_session_from_mid_graph() {
    let primal = running_primal().await;

    let session = SessionBuilder::new(SessionType::General).build();
    let session_id = primal.create_session(session).unwrap();

    let ids = append_chain(&primal, session_id, 3).await;
    let mid = ids[1];

    let (branch_id, copied) = primal.branch_session(session_id, mid, None).await.unwrap();
    assert_eq!(copied, 2);

    let branch_vertices = primal.get_all_vertices(branch_id).await.unwrap();
    assert_eq!(branch_vertices.len(), 2);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_branch_session_invalid_checkout() {
    let primal = running_primal().await;

    let session = SessionBuilder::new(SessionType::General).build();
    let session_id = primal.create_session(session).unwrap();

    let _ = append_chain(&primal, session_id, 1).await;

    let fake = VertexId::from_bytes(b"nonexistent vertex id 32 bytes!!");
    let err = primal.branch_session(session_id, fake, None).await.unwrap_err();
    assert!(err.to_string().contains("not found") || err.to_string().contains("Vertex"));
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_branch_session_not_running() {
    let config = RhizoCryptConfig::default();
    let primal = RhizoCrypt::new(config);

    let session_id = SessionId::now();
    let checkout = VertexId::from_bytes(b"checkout vertex id 32 bytes!!!!!");
    let err = primal.branch_session(session_id, checkout, None).await.unwrap_err();
    assert!(err.to_string().contains("not running") || err.to_string().contains("primal"));
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_diff_sessions_identical() {
    let primal = running_primal().await;

    let session = SessionBuilder::new(SessionType::General).build();
    let session_id = primal.create_session(session).unwrap();

    let ids = append_chain(&primal, session_id, 3).await;
    let tip = *ids.last().unwrap();

    let (branch_id, _) = primal.branch_session(session_id, tip, None).await.unwrap();

    let (only_in_base, only_in_other, common) =
        primal.diff_sessions(session_id, branch_id).await.unwrap();

    assert!(only_in_base.is_empty());
    assert!(only_in_other.is_empty());
    assert_eq!(common, 3);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_diff_sessions_disjoint() {
    let primal = running_primal().await;

    let s1 = SessionBuilder::new(SessionType::General).build();
    let s1_id = primal.create_session(s1).unwrap();
    let v1 = VertexBuilder::new(EventType::SessionStart).build();
    primal.append_vertex(s1_id, v1).await.unwrap();

    let s2 = SessionBuilder::new(SessionType::General).build();
    let s2_id = primal.create_session(s2).unwrap();
    let v2 = VertexBuilder::new(EventType::DataCreate {
        schema: Some("other".into()),
    })
    .build();
    primal.append_vertex(s2_id, v2).await.unwrap();

    let (only_in_base, only_in_other, common) = primal.diff_sessions(s1_id, s2_id).await.unwrap();

    assert_eq!(only_in_base.len(), 1);
    assert_eq!(only_in_other.len(), 1);
    assert_eq!(common, 0);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_merge_branches_basic() {
    let primal = running_primal().await;

    let session = SessionBuilder::new(SessionType::General).build();
    let session_id = primal.create_session(session).unwrap();

    let genesis = VertexBuilder::new(EventType::SessionStart).build();
    let genesis_id = primal.append_vertex(session_id, genesis).await.unwrap();

    let branch_a = VertexBuilder::new(EventType::DataCreate {
        schema: Some("a".into()),
    })
    .with_parent(genesis_id)
    .build();
    let branch_a_id = primal.append_vertex(session_id, branch_a).await.unwrap();

    let branch_b = VertexBuilder::new(EventType::DataCreate {
        schema: Some("b".into()),
    })
    .with_parent(genesis_id)
    .build();
    let branch_b_id = primal.append_vertex(session_id, branch_b).await.unwrap();

    let session = primal.get_session(session_id).unwrap();
    assert_eq!(session.frontier.len(), 2);

    let merge_vertex = VertexBuilder::new(EventType::DataCreate {
        schema: Some("merge".into()),
    })
    .with_parents([branch_a_id, branch_b_id])
    .build();

    primal.merge_branches(session_id, vec![branch_a_id, branch_b_id], merge_vertex).await.unwrap();

    let session = primal.get_session(session_id).unwrap();
    assert_eq!(session.frontier.len(), 1);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_merge_branches_needs_two_parents() {
    let primal = running_primal().await;

    let session = SessionBuilder::new(SessionType::General).build();
    let session_id = primal.create_session(session).unwrap();

    let v1 = VertexBuilder::new(EventType::SessionStart).build();
    let v1_id = primal.append_vertex(session_id, v1).await.unwrap();

    let merge_vertex = VertexBuilder::new(EventType::DataCreate {
        schema: Some("nope".into()),
    })
    .with_parent(v1_id)
    .build();

    let err = primal.merge_branches(session_id, vec![v1_id], merge_vertex).await.unwrap_err();
    assert!(err.to_string().contains("at least 2 parent"));
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_merge_branches_parent_not_in_frontier() {
    let primal = running_primal().await;

    let session = SessionBuilder::new(SessionType::General).build();
    let session_id = primal.create_session(session).unwrap();

    let genesis = VertexBuilder::new(EventType::SessionStart).build();
    let genesis_id = primal.append_vertex(session_id, genesis).await.unwrap();

    let child = VertexBuilder::new(EventType::DataCreate {
        schema: Some("child".into()),
    })
    .with_parent(genesis_id)
    .build();
    let child_id = primal.append_vertex(session_id, child).await.unwrap();

    let merge_vertex = VertexBuilder::new(EventType::DataCreate {
        schema: Some("merge".into()),
    })
    .with_parents([genesis_id, child_id])
    .build();

    let err = primal
        .merge_branches(session_id, vec![genesis_id, child_id], merge_vertex)
        .await
        .unwrap_err();
    assert!(err.to_string().contains("not in the session frontier"));
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_federate_vertices_skip_existing() {
    let primal = running_primal().await;

    let session = SessionBuilder::new(SessionType::General).build();
    let session_id = primal.create_session(session).unwrap();

    let vertex = VertexBuilder::new(EventType::SessionStart).build();
    let vertex_id = primal.append_vertex(session_id, vertex).await.unwrap();
    let existing = primal.get_vertex(session_id, vertex_id).await.unwrap();

    let (imported, skipped, _) =
        primal.federate_vertices(session_id, vec![existing]).await.unwrap();

    assert_eq!(imported, 0);
    assert_eq!(skipped, 1);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_federate_vertices_import_new() {
    let primal = running_primal().await;

    let session = SessionBuilder::new(SessionType::General).build();
    let session_id = primal.create_session(session).unwrap();

    let remote = VertexBuilder::new(EventType::SessionStart).build();
    let (imported, skipped, frontier) =
        primal.federate_vertices(session_id, vec![remote]).await.unwrap();

    assert_eq!(imported, 1);
    assert_eq!(skipped, 0);
    assert_eq!(frontier.len(), 1);
}

// ============================================================================
// Vertex operations (query, merkle)
// ============================================================================

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_query_vertices_by_event_type() {
    let primal = running_primal().await;

    let session = SessionBuilder::new(SessionType::General).build();
    let session_id = primal.create_session(session).unwrap();

    let v1 = VertexBuilder::new(EventType::SessionStart).build();
    let v2 = VertexBuilder::new(EventType::DataCreate {
        schema: None,
    })
    .build();
    let v3 = VertexBuilder::new(EventType::AgentJoin {
        role: crate::event::AgentRole::Participant,
    })
    .build();

    primal.append_vertex(session_id, v1).await.unwrap();
    primal.append_vertex(session_id, v2).await.unwrap();
    primal.append_vertex(session_id, v3).await.unwrap();

    let filtered = primal
        .query_vertices(
            session_id,
            Some(&[EventType::DataCreate {
                schema: None,
            }]),
            None,
            None,
        )
        .await
        .unwrap();

    assert_eq!(filtered.len(), 1);
    assert_eq!(
        filtered[0].event_type,
        EventType::DataCreate {
            schema: None
        }
    );
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_query_vertices_by_agent() {
    let primal = running_primal().await;

    let session = SessionBuilder::new(SessionType::General).build();
    let session_id = primal.create_session(session).unwrap();

    let agent_a = Did::new("did:key:agent-a");
    let agent_b = Did::new("did:key:agent-b");

    let v1 = VertexBuilder::new(EventType::DataCreate {
        schema: None,
    })
    .with_agent(agent_a.clone())
    .build();
    let v2 = VertexBuilder::new(EventType::DataCreate {
        schema: Some("other".into()),
    })
    .with_agent(agent_b)
    .build();

    primal.append_vertex(session_id, v1).await.unwrap();
    primal.append_vertex(session_id, v2).await.unwrap();

    let filtered = primal.query_vertices(session_id, None, Some(&agent_a), None).await.unwrap();

    assert_eq!(filtered.len(), 1);
    assert_eq!(filtered[0].agent, Some(agent_a));
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_query_vertices_with_limit() {
    let primal = running_primal().await;

    let session = SessionBuilder::new(SessionType::General).build();
    let session_id = primal.create_session(session).unwrap();

    for i in 0..5 {
        let vertex = VertexBuilder::new(EventType::DataCreate {
            schema: Some(format!("item-{i}")),
        })
        .build();
        primal.append_vertex(session_id, vertex).await.unwrap();
    }

    let limited = primal.query_vertices(session_id, None, None, Some(2)).await.unwrap();
    assert_eq!(limited.len(), 2);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_compute_merkle_root() {
    let primal = running_primal().await;

    let session = SessionBuilder::new(SessionType::General).build();
    let session_id = primal.create_session(session).unwrap();

    let v1 = VertexBuilder::new(EventType::SessionStart).build();
    let v2 = VertexBuilder::new(EventType::DataCreate {
        schema: None,
    })
    .build();
    primal.append_vertex(session_id, v1).await.unwrap();
    primal.append_vertex(session_id, v2).await.unwrap();

    let root = primal.compute_merkle_root(session_id).await.unwrap();
    assert!(!root.as_bytes().iter().all(|&b| b == 0));
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_generate_merkle_proof_success() {
    let primal = running_primal().await;

    let session = SessionBuilder::new(SessionType::General).build();
    let session_id = primal.create_session(session).unwrap();

    let v1 = VertexBuilder::new(EventType::SessionStart).build();
    let v2 = VertexBuilder::new(EventType::DataCreate {
        schema: None,
    })
    .build();
    primal.append_vertex(session_id, v1).await.unwrap();
    let vertex_id = primal.append_vertex(session_id, v2).await.unwrap();

    let proof = primal.generate_merkle_proof(session_id, vertex_id).await.unwrap();
    let vertex = primal.get_vertex(session_id, vertex_id).await.unwrap();
    assert!(proof.verify(&vertex));
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_generate_merkle_proof_not_found() {
    let primal = running_primal().await;

    let session = SessionBuilder::new(SessionType::General).build();
    let session_id = primal.create_session(session).unwrap();

    let v1 = VertexBuilder::new(EventType::SessionStart).build();
    primal.append_vertex(session_id, v1).await.unwrap();

    let fake = VertexId::from_bytes(b"nosuch vertex id 32 bytes!!!!!!");
    let err = primal.generate_merkle_proof(session_id, fake).await.unwrap_err();
    assert!(err.to_string().contains("not found") || err.to_string().contains("Vertex"));
}
