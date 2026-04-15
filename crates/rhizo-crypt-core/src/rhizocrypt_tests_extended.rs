// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2024–2026 ecoPrimals Project

use super::*;
use crate::config::{StorageBackend, StorageConfig};
use crate::event::EventType;
use crate::session::{CommitRef, SessionBuilder, SessionState, SessionType};
use crate::slice::ResolutionOutcome;
use crate::vertex::VertexBuilder;
use std::sync::Arc;
use std::time::Duration;

async fn running_primal() -> RhizoCrypt {
    let config = RhizoCryptConfig::default();
    let mut primal = RhizoCrypt::new(config);
    primal.start().await.unwrap();
    primal
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_compute_merkle_root_session_not_found() {
    let primal = running_primal().await;
    let err = primal.compute_merkle_root(SessionId::now()).await.unwrap_err();
    assert!(err.to_string().contains("not found") || err.to_string().contains("Session"));
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_compute_merkle_root_empty_session() {
    let primal = running_primal().await;
    let session = SessionBuilder::new(SessionType::General).build();
    let session_id = primal.create_session(session).unwrap();
    let root = primal.compute_merkle_root(session_id).await.unwrap();
    assert_eq!(root, crate::merkle::MerkleRoot::ZERO);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_generate_merkle_proof_empty_session() {
    let primal = running_primal().await;
    let session = SessionBuilder::new(SessionType::General).build();
    let session_id = primal.create_session(session).unwrap();
    let err = primal.generate_merkle_proof(session_id, VertexId::ZERO).await.unwrap_err();
    assert!(err.to_string().contains("not found") || err.to_string().contains("Vertex"));
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_generate_merkle_proof_vertex_missing_from_dag() {
    let primal = running_primal().await;
    let session = SessionBuilder::new(SessionType::General).build();
    let session_id = primal.create_session(session).unwrap();
    let v = VertexBuilder::new(EventType::SessionStart).build();
    primal.append_vertex(session_id, v).await.unwrap();
    let err = primal
        .generate_merkle_proof(session_id, VertexId::from_bytes(b"notintree"))
        .await
        .unwrap_err();
    assert!(err.to_string().contains("not found") || err.to_string().contains("Vertex"));
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_query_vertices_filter_by_agent() {
    let primal = running_primal().await;
    let session = SessionBuilder::new(SessionType::General).build();
    let session_id = primal.create_session(session).unwrap();

    let alice = Did::new("did:key:alice");
    let bob = Did::new("did:key:bob");

    let v_alice = VertexBuilder::new(EventType::DataCreate {
        schema: None,
    })
    .with_agent(alice.clone())
    .build();
    let v_bob = VertexBuilder::new(EventType::DataCreate {
        schema: None,
    })
    .with_agent(bob.clone())
    .build();
    primal.append_vertex(session_id, v_alice).await.unwrap();
    primal.append_vertex(session_id, v_bob).await.unwrap();

    let for_alice = primal.query_vertices(session_id, None, Some(&alice), None).await.unwrap();
    assert_eq!(for_alice.len(), 1);
    assert_eq!(for_alice[0].agent, Some(alice.clone()));

    let for_bob = primal.query_vertices(session_id, None, Some(&bob), None).await.unwrap();
    assert_eq!(for_bob.len(), 1);
    assert_eq!(for_bob[0].agent, Some(bob));

    let narrow = primal
        .query_vertices(
            session_id,
            Some(&[EventType::DataCreate {
                schema: None,
            }]),
            Some(&alice),
            None,
        )
        .await
        .unwrap();
    assert_eq!(narrow.len(), 1);
    assert_eq!(narrow[0].agent, Some(alice));
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_append_vertex_session_not_active() {
    let primal = running_primal().await;
    let mut session = SessionBuilder::new(SessionType::General).with_name("paused").build();
    session.state = SessionState::Paused {
        reason: "test pause".to_string(),
    };
    let session_id = primal.create_session(session).unwrap();
    let vertex = VertexBuilder::new(EventType::SessionStart).build();
    let err = primal.append_vertex(session_id, vertex).await.unwrap_err();
    assert!(err.to_string().contains("not active") || err.to_string().contains("session"));
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_session_for_vertex() {
    let primal = running_primal().await;
    let session = SessionBuilder::new(SessionType::General).build();
    let session_id = primal.create_session(session).unwrap();
    let vertex = VertexBuilder::new(EventType::SessionStart).build();
    let vertex_id = primal.append_vertex(session_id, vertex).await.unwrap();
    assert_eq!(primal.session_for_vertex(vertex_id), Some(session_id));
    assert!(primal.session_for_vertex(VertexId::from_bytes(b"unknown!")).is_none());
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_dehydrate_session_not_found() {
    let primal = running_primal().await;
    let err = primal.dehydrate(SessionId::now()).await.unwrap_err();
    assert!(err.to_string().contains("not found") || err.to_string().contains("Session"));
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_dehydrate_skips_missing_frontier_vertex_in_summary() {
    let primal = running_primal().await;
    let mut session = SessionBuilder::new(SessionType::General).build();
    session.frontier.insert(VertexId::from_bytes(b"orphan01"));
    let session_id = primal.create_session(session).unwrap();
    primal.dehydrate(session_id).await.unwrap();
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_health_unhealthy_when_stopped() {
    let config = RhizoCryptConfig::default();
    let mut primal = RhizoCrypt::new(config);
    primal.start().await.unwrap();
    primal.stop().await.unwrap();

    let status = primal.health_status();
    assert!(matches!(status, crate::primal::HealthStatus::Unhealthy { .. }));
    let report = primal.health_check().await.unwrap();
    assert!(!report.status.is_healthy());
    assert!(report.uptime_secs.is_none());
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_stop_when_not_running() {
    let config = RhizoCryptConfig::default();
    let mut primal = RhizoCrypt::new(config);
    assert!(primal.stop().await.is_err());

    primal.start().await.unwrap();
    primal.stop().await.unwrap();
    assert!(primal.stop().await.is_err());
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_resolve_slice_not_found() {
    let primal = running_primal().await;
    let fake = SliceId::new(uuid::Uuid::now_v7());
    let err = primal.resolve_slice(fake, ResolutionOutcome::ReturnedUnchanged).unwrap_err();
    assert!(err.to_string().contains("not found") || err.to_string().contains("Slice"));
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_gc_sweep_reaps_zero_ttl_session() {
    let primal = running_primal().await;
    let session =
        SessionBuilder::new(SessionType::General).with_max_duration(Duration::ZERO).build();
    let session_id = primal.create_session(session).unwrap();
    assert_eq!(primal.session_count(), 1);

    let reaped = primal.gc_sweep().await;
    assert_eq!(reaped, 1);
    assert_eq!(primal.session_count(), 0);
    assert!(primal.get_session(session_id).is_err());
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_gc_sweep_skips_terminal_session() {
    let primal = running_primal().await;
    let mut session =
        SessionBuilder::new(SessionType::General).with_max_duration(Duration::ZERO).build();
    session.state = SessionState::Committed {
        commit_ref: CommitRef {
            spine_id: "committed".to_string(),
            entry_hash: [7u8; 32],
            index: 1,
        },
        committed_at: crate::types::Timestamp::now(),
    };
    primal.create_session(session).unwrap();
    assert_eq!(primal.gc_sweep().await, 0);
    assert_eq!(primal.session_count(), 1);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_spawn_gc_sweeper_returns_join_handle() {
    let config = RhizoCryptConfig::default().with_gc_interval(Duration::from_millis(200));
    let mut primal = Arc::new(RhizoCrypt::new(config));
    Arc::get_mut(&mut primal).unwrap().start().await.unwrap();
    let handle = primal.spawn_gc_sweeper();
    handle.abort();
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_dehydrate_with_agent_join_leave_events() {
    use crate::event::{AgentRole, LeaveReason};
    use crate::types::Did;

    let primal = running_primal().await;
    let agent_a = Did::new("did:key:agentA");
    let agent_b = Did::new("did:key:agentB");

    let mut session = SessionBuilder::new(SessionType::Collaboration {
        workspace_id: "ws".to_string(),
    })
    .with_name("agent-events")
    .build();
    session.agents.insert(agent_a.clone());
    session.agents.insert(agent_b.clone());
    let session_id = primal.create_session(session).unwrap();

    let v1 = crate::vertex::VertexBuilder::new(EventType::AgentJoin {
        role: AgentRole::Owner,
    })
    .with_agent(agent_a.clone())
    .build();
    let v1_id = primal.append_vertex(session_id, v1).await.unwrap();

    let v2 = crate::vertex::VertexBuilder::new(EventType::AgentJoin {
        role: AgentRole::Custom("researcher".to_string()),
    })
    .with_agent(agent_b.clone())
    .with_parent(v1_id)
    .build();
    let v2_id = primal.append_vertex(session_id, v2).await.unwrap();

    let v3 = crate::vertex::VertexBuilder::new(EventType::DataCreate {
        schema: None,
    })
    .with_agent(agent_a.clone())
    .with_parent(v2_id)
    .build();
    let v3_id = primal.append_vertex(session_id, v3).await.unwrap();

    let v4 = crate::vertex::VertexBuilder::new(EventType::AgentLeave {
        reason: LeaveReason::Normal,
    })
    .with_agent(agent_b)
    .with_parent(v3_id)
    .build();
    primal.append_vertex(session_id, v4).await.unwrap();

    let root = primal.dehydrate(session_id).await.unwrap();
    assert!(!root.as_bytes().iter().all(|&b| b == 0));

    let status = primal.get_dehydration_status(session_id);
    assert!(status.is_complete());
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_dehydrate_with_observer_role() {
    use crate::event::AgentRole;
    use crate::types::Did;

    let primal = running_primal().await;
    let observer = Did::new("did:key:observer");

    let session = SessionBuilder::new(SessionType::General).with_name("observer-test").build();
    let session_id = primal.create_session(session).unwrap();

    let v1 = crate::vertex::VertexBuilder::new(EventType::AgentJoin {
        role: AgentRole::Observer,
    })
    .with_agent(observer)
    .build();
    primal.append_vertex(session_id, v1).await.unwrap();

    let root = primal.dehydrate(session_id).await.unwrap();
    assert!(!root.as_bytes().iter().all(|&b| b == 0));
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_dehydrate_with_payload_store() {
    use crate::store::PayloadStore;

    let primal = running_primal().await;

    let session = SessionBuilder::new(SessionType::General).build();
    let session_id = primal.create_session(session).unwrap();

    let store = primal.payload_store().await.unwrap();
    let payload = bytes::Bytes::from("test payload data for dehydration");
    let payload_ref = store.put(payload).await.unwrap();

    let v1 = crate::vertex::VertexBuilder::new(EventType::DataCreate {
        schema: None,
    })
    .with_payload(payload_ref)
    .build();
    primal.append_vertex(session_id, v1).await.unwrap();

    let root = primal.dehydrate(session_id).await.unwrap();
    assert!(!root.as_bytes().iter().all(|&b| b == 0));

    let status = primal.get_dehydration_status(session_id);
    assert!(status.is_complete());
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_dehydrate_with_registered_agents_no_vertices() {
    use crate::types::Did;

    let primal = running_primal().await;

    let mut session = SessionBuilder::new(SessionType::General).build();
    session.agents.insert(Did::new("did:key:silent-agent"));
    let session_id = primal.create_session(session).unwrap();

    let v = crate::vertex::VertexBuilder::new(EventType::SessionStart).build();
    primal.append_vertex(session_id, v).await.unwrap();

    let root = primal.dehydrate(session_id).await.unwrap();
    assert!(!root.as_bytes().iter().all(|&b| b == 0));
}

#[cfg(feature = "redb")]
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_start_with_redb_backend() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("gc.redb");
    let config = RhizoCryptConfig::default().with_storage(StorageConfig {
        backend: StorageBackend::Redb,
        path: Some(path.to_string_lossy().into_owned()),
        max_memory_bytes: None,
    });
    let mut primal = RhizoCrypt::new(config);
    primal.start().await.unwrap();
    assert_eq!(primal.state(), PrimalState::Running);
    primal.dag_store().await.unwrap();
    primal.stop().await.unwrap();
}
