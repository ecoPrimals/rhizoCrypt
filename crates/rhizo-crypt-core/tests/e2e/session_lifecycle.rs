//! E2E tests for session lifecycle.
//!
//! Tests the complete session flow: create → append → query → resolve.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use rhizo_crypt_core::{
    EventType, PrimalLifecycle, RhizoCrypt, RhizoCryptConfig, SessionBuilder, SessionType,
    VertexBuilder,
};

/// Test basic session creation and cleanup.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_session_create_and_discard() {
    let config = RhizoCryptConfig::default();
    let mut primal = RhizoCrypt::new(config);
    primal.start().await.expect("primal should start");

    // Create a session
    let session = SessionBuilder::new(SessionType::General).with_name("test-session").build();
    let session_id = primal.create_session(session).await.expect("should create session");

    // Verify session exists
    let retrieved = primal.get_session(session_id).expect("should get session");
    assert_eq!(retrieved.name, Some("test-session".to_string()));

    // Discard the session
    primal.discard_session(session_id).await.expect("should discard session");

    // Verify session is gone
    assert!(primal.get_session(session_id).is_err());

    primal.stop().await.expect("primal should stop");
}

/// Test appending vertices to a session.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_session_vertex_append() {
    let config = RhizoCryptConfig::default();
    let mut primal = RhizoCrypt::new(config);
    primal.start().await.expect("primal should start");

    // Create a session
    let session = SessionBuilder::new(SessionType::General).build();
    let session_id = primal.create_session(session).await.expect("should create session");

    // Append a vertex
    let vertex =
        VertexBuilder::new(EventType::SessionStart).with_metadata("test_key", "test_value").build();
    let vertex_id = primal.append_vertex(session_id, vertex).await.expect("should append vertex");

    // Retrieve the vertex
    let retrieved = primal.get_vertex(session_id, vertex_id).await.expect("should get vertex");
    assert_eq!(retrieved.event_type, EventType::SessionStart);

    // Check session vertex count
    let session = primal.get_session(session_id).expect("should get session");
    assert_eq!(session.vertex_count, 1);

    primal.stop().await.expect("primal should stop");
}

/// Test multiple sessions concurrently.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_multiple_sessions() {
    let config = RhizoCryptConfig::default();
    let mut primal = RhizoCrypt::new(config);
    primal.start().await.expect("primal should start");

    // Create multiple sessions
    for i in 0..5 {
        let session =
            SessionBuilder::new(SessionType::General).with_name(format!("session-{i}")).build();
        primal.create_session(session).await.expect("should create session");
    }

    // Verify all exist
    let sessions = primal.list_sessions();
    assert_eq!(sessions.len(), 5);

    // Verify count
    assert_eq!(primal.session_count(), 5);

    primal.stop().await.expect("primal should stop");
}

/// Test session max limit.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_session_limit() {
    let config = RhizoCryptConfig {
        max_sessions: 2,
        ..RhizoCryptConfig::default()
    };

    let mut primal = RhizoCrypt::new(config);
    primal.start().await.expect("primal should start");

    // Create up to limit
    for i in 0..2 {
        let session =
            SessionBuilder::new(SessionType::General).with_name(format!("session-{i}")).build();
        primal.create_session(session).await.expect("should create session within limit");
    }

    // Third should fail
    let session = SessionBuilder::new(SessionType::General).build();
    let result = primal.create_session(session).await;
    assert!(result.is_err());

    primal.stop().await.expect("primal should stop");
}
