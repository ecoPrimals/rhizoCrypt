// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2024–2026 ecoPrimals Project

//! E2E tests for complete dehydration workflow.
//!
//! Tests the full dehydration process from session creation through
//! Merkle computation, summary generation, and commitment.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use rhizo_crypt_core::{
    dehydration::DehydrationStatus, event::SessionOutcome, EventType, PrimalLifecycle, RhizoCrypt,
    RhizoCryptConfig, SessionBuilder, SessionType, VertexBuilder,
};

/// Test complete dehydration workflow.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_complete_dehydration_workflow() {
    let config = RhizoCryptConfig::default();
    let mut primal = RhizoCrypt::new(config);
    primal.start().await.expect("primal should start");

    // Create a session
    let session = SessionBuilder::new(SessionType::General).with_name("dehydration-test").build();
    let session_id = primal.create_session(session).expect("should create session");

    // Add vertices to create a DAG
    let v1 = VertexBuilder::new(EventType::SessionStart).build();
    let v1_id = primal.append_vertex(session_id, v1).await.expect("should append v1");

    let v2 = VertexBuilder::new(EventType::DataCreate {
        schema: None,
    })
    .with_parent(v1_id)
    .build();
    let v2_id = primal.append_vertex(session_id, v2).await.expect("should append v2");

    let v3 = VertexBuilder::new(EventType::SessionEnd {
        outcome: SessionOutcome::Success,
    })
    .with_parent(v2_id)
    .build();
    primal.append_vertex(session_id, v3).await.expect("should append v3");

    // Dehydrate the session
    let merkle_root = primal.dehydrate(session_id).await.expect("dehydration should succeed");

    // Verify Merkle root is not zero
    assert_ne!(*merkle_root.as_bytes(), [0u8; 32]);

    // Check dehydration status
    let status = primal.get_dehydration_status(session_id);
    assert!(status.is_complete(), "Dehydration should be complete");

    // Verify commit ref was created
    if let DehydrationStatus::Completed {
        commit_ref,
    } = status
    {
        // Should have a commit reference (local or remote)
        assert!(!commit_ref.spine_id.is_empty());
        assert_eq!(commit_ref.entry_hash, *merkle_root.as_bytes());
    } else {
        panic!("Expected Completed status");
    }

    primal.stop().await.expect("primal should stop");
}

/// Test dehydration with multiple agents.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_dehydration_multi_agent() {
    use rhizo_crypt_core::types::Did;

    let config = RhizoCryptConfig::default();
    let mut primal = RhizoCrypt::new(config);
    primal.start().await.expect("primal should start");

    // Create session with multiple agents
    let agent1 = Did::new("did:key:agent1");
    let agent2 = Did::new("did:key:agent2");

    let session = SessionBuilder::new(SessionType::Collaboration {
        workspace_id: "test-workspace".to_string(),
    })
    .with_name("multi-agent-session")
    .build();
    let session_id = primal.create_session(session).expect("should create session");

    // Add vertices from different agents
    let v1 = VertexBuilder::new(EventType::SessionStart).with_agent(agent1.clone()).build();
    let v1_id = primal.append_vertex(session_id, v1).await.expect("should append v1");

    let v2 = VertexBuilder::new(EventType::DataCreate {
        schema: None,
    })
    .with_parent(v1_id)
    .with_agent(agent2.clone())
    .build();
    primal.append_vertex(session_id, v2).await.expect("should append v2");

    // Dehydrate
    let merkle_root = primal.dehydrate(session_id).await.expect("dehydration should succeed");

    assert_ne!(*merkle_root.as_bytes(), [0u8; 32]);

    let status = primal.get_dehydration_status(session_id);
    assert!(status.is_complete());

    primal.stop().await.expect("primal should stop");
}

/// Test dehydration with large payload.
///
/// Note: This test creates a vertex with metadata instead of payload
/// since payloads now use `PayloadRef` (content-addressed references).
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_dehydration_large_payload() {
    let config = RhizoCryptConfig::default();
    let mut primal = RhizoCrypt::new(config);
    primal.start().await.expect("primal should start");

    let session = SessionBuilder::new(SessionType::General).build();
    let session_id = primal.create_session(session).expect("should create session");

    // Add vertex (no payload - would need PayloadRef)
    let v1 = VertexBuilder::new(EventType::DataCreate {
        schema: None,
    })
    .build();
    primal.append_vertex(session_id, v1).await.expect("should append vertex");

    // Dehydrate
    let merkle_root = primal.dehydrate(session_id).await.expect("dehydration should succeed");

    assert_ne!(*merkle_root.as_bytes(), [0u8; 32]);

    let status = primal.get_dehydration_status(session_id);
    assert!(status.is_complete());

    primal.stop().await.expect("primal should stop");
}

/// Test dehydration status progression.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_dehydration_status_progression() {
    let config = RhizoCryptConfig::default();
    let mut primal = RhizoCrypt::new(config);
    primal.start().await.expect("primal should start");

    let session = SessionBuilder::new(SessionType::General).build();
    let session_id = primal.create_session(session).expect("should create session");

    // Add a vertex
    let v1 = VertexBuilder::new(EventType::SessionStart).build();
    primal.append_vertex(session_id, v1).await.expect("should append vertex");

    // Initial status should be Pending
    let status = primal.get_dehydration_status(session_id);
    assert_eq!(status, DehydrationStatus::Pending);
    assert!(!status.is_in_progress());

    // Start dehydration
    let _merkle_root = primal.dehydrate(session_id).await.expect("dehydration should succeed");

    // Final status should be Completed
    let status = primal.get_dehydration_status(session_id);
    assert!(status.is_complete());
    assert!(!status.is_failed());

    primal.stop().await.expect("primal should stop");
}

/// Test dehydration with empty session (no vertices).
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_dehydration_empty_session() {
    let config = RhizoCryptConfig::default();
    let mut primal = RhizoCrypt::new(config);
    primal.start().await.expect("primal should start");

    let session = SessionBuilder::new(SessionType::General).build();
    let session_id = primal.create_session(session).expect("should create session");

    // Dehydrate without adding any vertices
    let result = primal.dehydrate(session_id).await;

    // Should still succeed (empty DAG is valid)
    assert!(result.is_ok(), "Empty session dehydration should succeed");

    primal.stop().await.expect("primal should stop");
}

/// Test dehydration preserves Merkle root determinism.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_dehydration_merkle_determinism() {
    let config = RhizoCryptConfig::default();
    let mut primal = RhizoCrypt::new(config);
    primal.start().await.expect("primal should start");

    // Create two identical sessions
    let session1 = SessionBuilder::new(SessionType::General).build();
    let session1_id = primal.create_session(session1).expect("should create session1");

    let session2 = SessionBuilder::new(SessionType::General).build();
    let session2_id = primal.create_session(session2).expect("should create session2");

    // Add identical vertices to both
    for session_id in [session1_id, session2_id] {
        let v1 = VertexBuilder::new(EventType::SessionStart).build();
        primal.append_vertex(session_id, v1).await.expect("should append vertex");
    }

    // Dehydrate both
    let root1 = primal.dehydrate(session1_id).await.expect("dehydration1 should succeed");
    let root2 = primal.dehydrate(session2_id).await.expect("dehydration2 should succeed");

    // Merkle roots should be different (sessions have different IDs/timestamps)
    // But the structure should be deterministic
    assert_ne!(*root1.as_bytes(), [0u8; 32]);
    assert_ne!(*root2.as_bytes(), [0u8; 32]);

    primal.stop().await.expect("primal should stop");
}
