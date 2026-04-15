// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2024–2026 ecoPrimals Project

use super::*;
use crate::event::EventType;
use crate::session::{SessionBuilder, SessionType};
use crate::slice::{ResolutionOutcome, SliceBuilder, SliceMode, SliceOrigin};
use crate::vertex::VertexBuilder;

async fn running_primal() -> RhizoCrypt {
    let config = RhizoCryptConfig::default();
    let mut primal = RhizoCrypt::new(config);
    primal.start().await.unwrap();
    primal
}

fn test_slice_origin(owner: Did) -> SliceOrigin {
    SliceOrigin {
        spine_id: "spine-test".to_string(),
        entry_hash: [0u8; 32],
        entry_index: 0,
        certificate_id: None,
        owner,
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_session_operations() {
    let primal = running_primal().await;

    let session = SessionBuilder::new(SessionType::General).with_name("test").build();
    let session_id = primal.create_session(session).unwrap();

    let got = primal.get_session(session_id).unwrap();
    assert_eq!(got.name, Some("test".to_string()));

    let sessions = primal.list_sessions();
    assert_eq!(sessions.len(), 1);
    assert_eq!(primal.session_count(), 1);
    assert_eq!(primal.total_vertex_count(), 0);

    primal.discard_session(session_id).await.unwrap();
    assert!(primal.get_session(session_id).is_err());
    assert_eq!(primal.session_count(), 0);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_session_not_running() {
    let config = RhizoCryptConfig::default();
    let primal = RhizoCrypt::new(config);

    let session = SessionBuilder::new(SessionType::General).build();
    assert!(primal.create_session(session).is_err());

    let session = SessionBuilder::new(SessionType::General).build();
    let session_id = session.id;
    let mut primal = running_primal().await;
    primal.create_session(session).unwrap();
    primal.stop().await.unwrap();
    assert!(primal.discard_session(session_id).await.is_err());
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_max_sessions_exceeded() {
    let config = RhizoCryptConfig::default().with_max_sessions(2);
    let mut primal = RhizoCrypt::new(config);
    primal.start().await.unwrap();

    let s1 = SessionBuilder::new(SessionType::General).build();
    let s2 = SessionBuilder::new(SessionType::General).build();
    primal.create_session(s1).unwrap();
    primal.create_session(s2).unwrap();

    let s3 = SessionBuilder::new(SessionType::General).build();
    assert!(primal.create_session(s3).is_err());
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_vertex_operations() {
    let primal = running_primal().await;

    let session = SessionBuilder::new(SessionType::General).build();
    let session_id = primal.create_session(session).unwrap();

    let vertex = VertexBuilder::new(EventType::SessionStart).build();
    let vertex_id = primal.append_vertex(session_id, vertex).await.unwrap();

    let got = primal.get_vertex(session_id, vertex_id).await.unwrap();
    assert_eq!(got.event_type, EventType::SessionStart);

    let all = primal.get_all_vertices(session_id).await.unwrap();
    assert_eq!(all.len(), 1);
    assert_eq!(all[0].event_type, EventType::SessionStart);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_vertex_not_running() {
    let config = RhizoCryptConfig::default();
    let primal = RhizoCrypt::new(config);
    let session_id = SessionId::now();
    let vertex = VertexBuilder::new(EventType::SessionStart).build();
    assert!(primal.append_vertex(session_id, vertex).await.is_err());
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_vertex_session_not_found() {
    let primal = running_primal().await;
    let session_id = SessionId::now();
    let vertex = VertexBuilder::new(EventType::SessionStart).build();
    assert!(primal.append_vertex(session_id, vertex).await.is_err());
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_query_vertices() {
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

    let data_only = primal
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
    assert_eq!(data_only.len(), 1);
    assert_eq!(
        data_only[0].event_type,
        EventType::DataCreate {
            schema: None
        }
    );

    let limited = primal.query_vertices(session_id, None, None, Some(2)).await.unwrap();
    assert_eq!(limited.len(), 2);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_merkle_operations() {
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

    let root = primal.compute_merkle_root(session_id).await.unwrap();
    assert!(!root.as_bytes().iter().all(|&b| b == 0));

    let proof = primal.generate_merkle_proof(session_id, vertex_id).await.unwrap();
    let vertex = primal.get_vertex(session_id, vertex_id).await.unwrap();
    assert!(proof.verify(&vertex));
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_slice_operations() {
    let primal = running_primal().await;

    let session = SessionBuilder::new(SessionType::General).build();
    let session_id = primal.create_session(session).unwrap();

    let vertex = VertexBuilder::new(EventType::SessionStart).build();
    let vertex_id = primal.append_vertex(session_id, vertex).await.unwrap();

    let owner = Did::new("did:test:owner");
    let holder = Did::new("did:test:user");
    let slice = SliceBuilder::new(
        test_slice_origin(owner),
        holder,
        SliceMode::Copy {
            allow_recopy: false,
        },
        session_id,
        vertex_id,
    )
    .build();

    let slice_id = primal.checkout_slice(slice).unwrap();

    let got = primal.get_slice(slice_id).unwrap();
    assert_eq!(got.session_id, session_id);

    let slices = primal.list_slices();
    assert_eq!(slices.len(), 1);

    primal.resolve_slice(slice_id, ResolutionOutcome::ReturnedUnchanged).unwrap();
    let resolved = primal.get_slice(slice_id).unwrap();
    assert!(resolved.is_resolved());
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_slice_not_running() {
    let config = RhizoCryptConfig::default();
    let primal = RhizoCrypt::new(config);
    let owner = Did::new("did:test:owner");
    let slice = SliceBuilder::new(
        test_slice_origin(owner),
        Did::new("did:test:user"),
        SliceMode::Copy {
            allow_recopy: false,
        },
        SessionId::now(),
        VertexId::from_bytes(b"checkout"),
    )
    .build();
    assert!(primal.checkout_slice(slice).is_err());
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_dehydrate() {
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

    let root = primal.dehydrate(session_id).await.unwrap();
    assert!(!root.as_bytes().iter().all(|&b| b == 0));
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_dehydration_status() {
    let primal = running_primal().await;

    let session = SessionBuilder::new(SessionType::General).build();
    let session_id = primal.create_session(session).unwrap();

    let status_before = primal.get_dehydration_status(session_id);
    assert!(matches!(status_before, crate::dehydration::DehydrationStatus::Pending));

    primal.dehydrate(session_id).await.unwrap();

    let status_after = primal.get_dehydration_status(session_id);
    assert!(matches!(status_after, crate::dehydration::DehydrationStatus::Completed { .. }));
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_health_report() {
    let primal = running_primal().await;

    let report = primal.health_check().await.unwrap();
    assert!(report.status.is_healthy());
    assert!(report.uptime_secs.is_some());
    assert_eq!(report.name, crate::constants::PRIMAL_NAME);
    assert!(!report.version.is_empty());
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_restart() {
    let config = RhizoCryptConfig::default();
    let mut primal = RhizoCrypt::new(config);

    primal.start().await.unwrap();
    assert_eq!(primal.state(), PrimalState::Running);

    primal.stop().await.unwrap();
    assert_eq!(primal.state(), PrimalState::Stopped);

    primal.start().await.unwrap();
    assert_eq!(primal.state(), PrimalState::Running);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_rhizocrypt_lifecycle() {
    let config = RhizoCryptConfig::default();
    let mut primal = RhizoCrypt::new(config);

    assert_eq!(primal.state(), PrimalState::Created);

    primal.start().await.unwrap();
    assert_eq!(primal.state(), PrimalState::Running);
    assert!(primal.uptime_secs().is_some());

    let report = primal.health_check().await.unwrap();
    assert!(report.status.is_healthy());

    primal.stop().await.unwrap();
    assert_eq!(primal.state(), PrimalState::Stopped);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_rhizocrypt_stores() {
    let config = RhizoCryptConfig::default();
    let mut primal = RhizoCrypt::new(config);

    assert!(primal.dag_store().await.is_err());

    primal.start().await.unwrap();

    let dag_store = primal.dag_store().await.unwrap();
    assert_eq!(dag_store.session_count().await, 0);

    let payload_store = primal.payload_store().await.unwrap();
    assert_eq!(payload_store.payload_count().await, 0);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_rhizocrypt_invalid_transitions() {
    let config = RhizoCryptConfig::default();
    let mut primal = RhizoCrypt::new(config);

    assert!(primal.stop().await.is_err());

    primal.start().await.unwrap();

    assert!(primal.start().await.is_err());
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_get_slice_not_found() {
    let primal = running_primal().await;
    let fake_id = SliceId::new(uuid::Uuid::now_v7());
    let err = primal.get_slice(fake_id).unwrap_err();
    assert!(err.to_string().contains("not found"));
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_resolve_slice_already_resolved() {
    use crate::types::VertexId;

    let primal = running_primal().await;
    let session = SessionBuilder::new(SessionType::General).with_name("resolve-test").build();
    let session_id = primal.create_session(session).unwrap();

    let origin = test_slice_origin(Did::new("did:key:resolver"));
    let holder = Did::new("did:key:holder");
    let mode = SliceMode::Copy {
        allow_recopy: false,
    };
    let slice = SliceBuilder::new(origin, holder, mode, session_id, VertexId::ZERO).build();
    let slice_id = primal.checkout_slice(slice).unwrap();

    primal.resolve_slice(slice_id, ResolutionOutcome::ReturnedUnchanged).unwrap();
    let err = primal.resolve_slice(slice_id, ResolutionOutcome::ReturnedUnchanged).unwrap_err();
    assert!(err.to_string().contains("already resolved"));
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_list_slices_filters_resolved() {
    use crate::types::VertexId;

    let primal = running_primal().await;
    let session = SessionBuilder::new(SessionType::General).with_name("filter-test").build();
    let session_id = primal.create_session(session).unwrap();

    let origin = test_slice_origin(Did::new("did:key:lister"));
    let mode = SliceMode::Copy {
        allow_recopy: false,
    };
    let s1 = SliceBuilder::new(
        origin.clone(),
        Did::new("did:key:h1"),
        mode.clone(),
        session_id,
        VertexId::ZERO,
    )
    .build();
    let s2 =
        SliceBuilder::new(origin, Did::new("did:key:h2"), mode, session_id, VertexId::ZERO).build();
    let id1 = primal.checkout_slice(s1).unwrap();
    let _id2 = primal.checkout_slice(s2).unwrap();

    assert_eq!(primal.list_slices().len(), 2);

    primal.resolve_slice(id1, ResolutionOutcome::ReturnedUnchanged).unwrap();
    assert_eq!(primal.list_slices().len(), 1);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_append_vertex_with_agent() {
    let primal = running_primal().await;
    let session = SessionBuilder::new(SessionType::General).with_name("agent-test").build();
    let session_id = primal.create_session(session).unwrap();

    let agent = Did::new("did:key:agent123");
    let vertex = VertexBuilder::new(EventType::DataCreate {
        schema: None,
    })
    .with_agent(agent.clone())
    .build();

    let vid = primal.append_vertex(session_id, vertex).await.unwrap();
    let got = primal.get_vertex(session_id, vid).await.unwrap();
    assert_eq!(got.agent, Some(agent));
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_health_status_before_start() {
    let config = RhizoCryptConfig::default();
    let primal = RhizoCrypt::new(config);
    assert_eq!(primal.state(), PrimalState::Created);
    assert!(primal.uptime_secs().is_none());
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_get_session_not_found() {
    let primal = running_primal().await;
    let err = primal.get_session(SessionId::now()).unwrap_err();
    assert!(err.to_string().contains("not found") || err.to_string().contains("Session"));
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_discard_session_not_found() {
    let primal = running_primal().await;
    let err = primal.discard_session(SessionId::now()).await.unwrap_err();
    assert!(err.to_string().contains("not found") || err.to_string().contains("Session"));
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_get_vertex_not_found() {
    let primal = running_primal().await;
    let session = SessionBuilder::new(SessionType::General).build();
    let session_id = primal.create_session(session).unwrap();
    let err = primal.get_vertex(session_id, VertexId::from_bytes(b"nosuchvx")).await.unwrap_err();
    assert!(err.to_string().contains("not found") || err.to_string().contains("Vertex"));
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_get_all_vertices_not_running() {
    let config = RhizoCryptConfig::default();
    let primal = RhizoCrypt::new(config);
    let err = primal.get_all_vertices(SessionId::now()).await.unwrap_err();
    assert!(err.to_string().contains("not running") || err.to_string().contains("primal"));
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_payload_store_not_running() {
    let primal = RhizoCrypt::new(RhizoCryptConfig::default());
    let err = primal.payload_store().await.unwrap_err();
    assert!(err.to_string().contains("not running") || err.to_string().contains("primal"));
}
