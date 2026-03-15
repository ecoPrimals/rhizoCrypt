// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2024–2026 ecoPrimals Project

//! Failure injection tests for rhizoCrypt.
//!
//! Tests system behavior when operations fail or timeout.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use rhizo_crypt_core::{
    PrimalLifecycle, RhizoCrypt, RhizoCryptConfig, SessionBuilder, SessionId, SessionType,
};

/// Test handling of non-existent session.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_nonexistent_session() {
    let config = RhizoCryptConfig::default();
    let mut primal = RhizoCrypt::new(config);
    primal.start().await.expect("primal should start");

    // Try to get a non-existent session
    let fake_id = SessionId::now();
    let result = primal.get_session(fake_id);
    assert!(result.is_err());

    // Try to discard a non-existent session
    let result = primal.discard_session(fake_id).await;
    assert!(result.is_err());

    primal.stop().await.expect("primal should stop");
}

/// Test operations on stopped primal.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_operations_on_stopped_primal() {
    let config = RhizoCryptConfig::default();
    let mut primal = RhizoCrypt::new(config);
    primal.start().await.expect("primal should start");

    // Create a session while running
    let session = SessionBuilder::new(SessionType::General).build();
    let _session_id = primal.create_session(session).expect("should create session");

    // Stop the primal
    primal.stop().await.expect("primal should stop");

    // Try to create session on stopped primal
    let session = SessionBuilder::new(SessionType::General).build();
    let result = primal.create_session(session);
    assert!(result.is_err());
}

/// Test double start/stop.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_double_start_stop() {
    let config = RhizoCryptConfig::default();
    let mut primal = RhizoCrypt::new(config);

    // Start
    primal.start().await.expect("primal should start");

    // Double start should fail
    let result = primal.start().await;
    assert!(result.is_err());

    // Stop
    primal.stop().await.expect("primal should stop");

    // Double stop should fail
    let result = primal.stop().await;
    assert!(result.is_err());
}

/// Test session creation at limit boundary.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_session_limit_boundary() {
    let config = RhizoCryptConfig {
        max_sessions: 5,
        ..RhizoCryptConfig::default()
    };

    let mut primal = RhizoCrypt::new(config);
    primal.start().await.expect("primal should start");

    // Fill to limit
    for i in 0..5 {
        let session =
            SessionBuilder::new(SessionType::General).with_name(format!("session-{i}")).build();
        primal.create_session(session).expect("should create session within limit");
    }

    // Next should fail
    let session = SessionBuilder::new(SessionType::General).build();
    assert!(primal.create_session(session).is_err());

    // Discard one
    let sessions = primal.list_sessions();
    let first_id = sessions[0].id;
    primal.discard_session(first_id).await.expect("should discard session");

    // Now should succeed
    let session = SessionBuilder::new(SessionType::General).build();
    assert!(primal.create_session(session).is_ok());

    primal.stop().await.expect("primal should stop");
}
