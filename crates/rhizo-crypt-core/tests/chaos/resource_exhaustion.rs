// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2024–2026 ecoPrimals Project

#![allow(clippy::unwrap_used, clippy::expect_used)]

use rhizo_crypt_core::{
    EventType, MetadataValue, PrimalLifecycle, RhizoCrypt, RhizoCryptConfig, SessionBuilder,
    SessionType, VertexBuilder,
};

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_max_sessions_boundary() {
    let config = RhizoCryptConfig::default().with_max_sessions(3);
    let mut primal = RhizoCrypt::new(config);
    primal.start().await.expect("primal should start");

    let s1 = SessionBuilder::new(SessionType::General).with_name("s1").build();
    let s2 = SessionBuilder::new(SessionType::General).with_name("s2").build();
    let s3 = SessionBuilder::new(SessionType::General).with_name("s3").build();

    primal.create_session(s1).expect("create session 1");
    primal.create_session(s2).expect("create session 2");
    primal.create_session(s3).expect("create session 3");

    let s4 = SessionBuilder::new(SessionType::General).with_name("s4").build();
    let result = primal.create_session(s4);
    assert!(result.is_err());

    primal.stop().await.expect("primal should stop");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_large_vertex_payload() {
    let config = RhizoCryptConfig::default();
    let mut primal = RhizoCrypt::new(config);
    primal.start().await.expect("primal should start");

    let session = SessionBuilder::new(SessionType::General).build();
    let session_id = primal.create_session(session).expect("should create session");

    let large_metadata = "x".repeat(256 * 1024);
    let vertex = VertexBuilder::new(EventType::DataCreate {
        schema: None,
    })
    .with_metadata("large_field", large_metadata)
    .build();

    let vertex_id =
        primal.append_vertex(session_id, vertex).await.expect("should append large vertex");

    let retrieved = primal.get_vertex(session_id, vertex_id).await.expect("get vertex");
    let meta = retrieved.metadata.get("large_field").expect("large_field");
    assert!(matches!(meta, MetadataValue::String(s) if s.len() == 256 * 1024));

    primal.stop().await.expect("primal should stop");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_rapid_session_create_destroy() {
    let config = RhizoCryptConfig::default().with_max_sessions(50);
    let mut primal = RhizoCrypt::new(config);
    primal.start().await.expect("primal should start");

    for i in 0..30 {
        let session =
            SessionBuilder::new(SessionType::General).with_name(format!("rapid-{i}")).build();
        let session_id = primal.create_session(session).expect("create session");
        primal.discard_session(session_id).await.expect("discard session");
    }

    let session = SessionBuilder::new(SessionType::General).with_name("final").build();
    let session_id = primal.create_session(session).expect("create final session");
    let retrieved = primal.get_session(session_id).expect("get session");
    assert_eq!(retrieved.name, Some("final".to_string()));

    primal.stop().await.expect("primal should stop");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_many_vertices_single_session() {
    let config = RhizoCryptConfig::default();
    let mut primal = RhizoCrypt::new(config);
    primal.start().await.expect("primal should start");

    let session = SessionBuilder::new(SessionType::General).build();
    let session_id = primal.create_session(session).expect("should create session");

    let mut parent_id = None;
    for i in 0..500 {
        let vertex = parent_id.map_or_else(
            || VertexBuilder::new(EventType::SessionStart).build(),
            |pid| {
                VertexBuilder::new(EventType::Custom {
                    domain: "stress".into(),
                    event_name: format!("v{i}"),
                })
                .with_parent(pid)
                .build()
            },
        );
        parent_id = Some(primal.append_vertex(session_id, vertex).await.expect("append vertex"));
    }

    let session = primal.get_session(session_id).expect("get session");
    assert_eq!(session.vertex_count, 500);

    let all = primal.get_all_vertices(session_id).await.expect("get all vertices");
    assert_eq!(all.len(), 500);

    primal.stop().await.expect("primal should stop");
}
