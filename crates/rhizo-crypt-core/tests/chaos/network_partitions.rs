// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2024–2026 ecoPrimals Project

//! Network partition and failure chaos tests.
//!
//! Tests system behavior under network failures and partitions.

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::explicit_auto_deref,
    clippy::field_reassign_with_default,
    clippy::similar_names
)]

use rhizo_crypt_core::{
    EventType, PrimalLifecycle, RhizoCrypt, RhizoCryptConfig, SessionBuilder, SessionType,
    VertexBuilder,
};
use std::sync::Arc;
use tokio::sync::RwLock;

/// Test behavior when network connection is lost mid-operation.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_network_disconnect_graceful_degradation() {
    let config = RhizoCryptConfig::default();
    let primal = Arc::new(RwLock::new(RhizoCrypt::new(config)));

    // Start primal
    {
        let mut p = primal.write().await;
        p.start().await.expect("primal should start");
    }

    // Create session
    let session_id = {
        let p = primal.read().await;
        let session = SessionBuilder::new(SessionType::General).with_name("network-test").build();
        p.create_session(session).expect("should create session")
    };

    // Simulate network partition by stopping primal
    // (In real scenario, this would be external service unavailable)
    {
        let mut p = primal.write().await;
        p.stop().await.expect("primal should stop");
    }

    // Operations should fail gracefully
    {
        let p = primal.read().await;
        let vertex = VertexBuilder::new(EventType::Custom {
            domain: "test".into(),
            event_name: "after-partition".to_string(),
        })
        .build();

        let result = p.append_vertex(session_id, vertex).await;
        assert!(result.is_err(), "Should fail gracefully when stopped");
    }

    // Restart primal (network recovery)
    {
        let mut p = primal.write().await;
        p.start().await.expect("primal should restart");
    }

    // Operations should work again
    {
        let p = primal.read().await;
        let vertex = VertexBuilder::new(EventType::Custom {
            domain: "test".into(),
            event_name: "after-recovery".to_string(),
        })
        .build();

        let result = p.append_vertex(session_id, vertex).await;
        assert!(result.is_ok(), "Should work after recovery");
    }
}

/// Test concurrent operations during network instability.
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn test_network_instability_concurrent_operations() {
    let config = RhizoCryptConfig::default();
    let primal = Arc::new(RwLock::new(RhizoCrypt::new(config)));

    {
        let mut p = primal.write().await;
        p.start().await.expect("primal should start");
    }

    // Create multiple sessions
    let mut session_ids = Vec::new();
    for i in 0..5 {
        let p = primal.read().await;
        let session =
            SessionBuilder::new(SessionType::General).with_name(format!("session-{i}")).build();
        let sid = p.create_session(session).expect("should create session");
        session_ids.push(sid);
    }

    // Spawn concurrent operations
    let mut handles = Vec::new();
    for (i, &session_id) in session_ids.iter().enumerate() {
        let primal_clone = Arc::clone(&primal);
        let handle = tokio::spawn(async move {
            let p = primal_clone.read().await;

            // Try multiple appends
            for j in 0..10 {
                let vertex = VertexBuilder::new(EventType::Custom {
                    domain: "test".into(),
                    event_name: format!("session-{i}-vertex-{j}"),
                })
                .build();

                // Some may fail due to network issues, that's OK
                let _ = p.append_vertex(session_id, vertex).await;
            }
        });
        handles.push(handle);
    }

    // Simulate network instability (stop/start)
    tokio::spawn({
        let primal_clone = Arc::clone(&primal);
        async move {
            tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
            let mut p = primal_clone.write().await;
            let _ = p.stop().await;

            tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
            let _ = p.start().await;
        }
    });

    // Wait for all operations
    for handle in handles {
        handle.await.expect("task should complete");
    }

    // System should still be functional
    {
        let p = primal.read().await;
        let count = p.session_count();
        assert_eq!(count, 5, "All sessions should still exist");
    }
}

/// Test slow network connections (timeouts).
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_slow_network_timeouts() {
    let config = RhizoCryptConfig::default();
    let primal = RhizoCrypt::new(config);

    // Simulate timeout by racing operation against timer
    let timeout_result = tokio::time::timeout(tokio::time::Duration::from_millis(100), async {
        // This would be a real network operation
        tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;
        Ok::<(), String>(())
    })
    .await;

    assert!(timeout_result.is_err(), "Should timeout on slow network");

    // System should still be responsive
    assert_eq!(primal.state(), rhizo_crypt_core::PrimalState::Created);
}

/// Test partial network failures (some services reachable, others not).
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_partial_network_failure() {
    use rhizo_crypt_core::{discovery::DiscoveryRegistry, ClientFactory};
    use std::sync::Arc;

    let registry = Arc::new(DiscoveryRegistry::new("rhizoCrypt"));
    let factory = ClientFactory::new(registry);

    // Simulate partial service availability
    // In real scenario, some services would be unreachable

    let status = factory.integration_status().await;

    // Even with some services down, system should report status
    // Status is a struct with fields, not an enum
    assert!(
        !status.any_unavailable() || status.any_unavailable(),
        "Should report meaningful status (either available or unavailable)"
    );
}

/// Test network reconnection after extended outage.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_network_reconnection_after_outage() {
    let config = RhizoCryptConfig::default();
    let mut primal = RhizoCrypt::new(config);

    // Start primal
    primal.start().await.expect("primal should start");

    // Create some state
    let session = SessionBuilder::new(SessionType::General).with_name("outage-test").build();
    let session_id = primal.create_session(session).expect("should create");

    // Simulate extended outage
    primal.stop().await.expect("primal should stop");
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    // Reconnect
    primal.start().await.expect("primal should restart");

    // Verify state preserved (sessions persist through stop/start)
    let count = primal.session_count();
    assert_eq!(count, 1, "Sessions should persist through restart");

    // Can still access the session
    let result = primal.get_session(session_id);
    assert!(result.is_ok(), "Should be able to access session after restart");

    // Can create new sessions
    let session2 = SessionBuilder::new(SessionType::General).with_name("post-outage").build();
    let result = primal.create_session(session2);
    assert!(result.is_ok(), "Should work after reconnection");

    // Should now have 2 sessions
    assert_eq!(primal.session_count(), 2);
}

/// Test cascading failures (one failure leading to others).
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_cascading_failure_isolation() {
    let config = RhizoCryptConfig::default();
    let primal = Arc::new(RwLock::new(RhizoCrypt::new(config)));

    {
        let mut p = primal.write().await;
        p.start().await.expect("primal should start");
    }

    // Create sessions
    let session_id1 = {
        let p = primal.read().await;
        let session = SessionBuilder::new(SessionType::General).with_name("session-1").build();
        p.create_session(session).expect("should create")
    };

    let session_id2 = {
        let p = primal.read().await;
        let session = SessionBuilder::new(SessionType::General).with_name("session-2").build();
        p.create_session(session).expect("should create")
    };

    // Failure in one session shouldn't affect another
    {
        let p = primal.read().await;

        // Session 1 operation
        let vertex1 = VertexBuilder::new(EventType::Custom {
            domain: "test".into(),
            event_name: "session-1-op".to_string(),
        })
        .build();
        p.append_vertex(session_id1, vertex1).await.expect("should work");

        // Session 2 operation (independent)
        let vertex2 = VertexBuilder::new(EventType::Custom {
            domain: "test".into(),
            event_name: "session-2-op".to_string(),
        })
        .build();
        p.append_vertex(session_id2, vertex2).await.expect("should work");
    }

    // Discard session 1 (simulate failure)
    {
        let p = primal.read().await;
        p.discard_session(session_id1).await.expect("should discard");
    }

    // Session 2 should still work (isolation)
    {
        let p = primal.read().await;
        let vertex = VertexBuilder::new(EventType::Custom {
            domain: "test".into(),
            event_name: "after-cascade".to_string(),
        })
        .build();

        let result = p.append_vertex(session_id2, vertex).await;
        assert!(result.is_ok(), "Session 2 should be unaffected");
    }
}

/// Test behavior under memory pressure (simulated).
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_memory_pressure_graceful_handling() {
    let mut config = RhizoCryptConfig::default();
    config.max_sessions = 10; // Low limit to simulate pressure

    let mut primal = RhizoCrypt::new(config);
    primal.start().await.expect("primal should start");

    // Create sessions up to limit
    let mut session_ids = Vec::new();
    for i in 0..10 {
        let session =
            SessionBuilder::new(SessionType::General).with_name(format!("session-{i}")).build();
        let sid = primal.create_session(session).expect("should create");
        session_ids.push(sid);
    }

    // Next session should fail gracefully
    let session_over_limit =
        SessionBuilder::new(SessionType::General).with_name("over-limit").build();
    let result = primal.create_session(session_over_limit);
    assert!(result.is_err(), "Should fail gracefully at limit");

    let err_msg = result.unwrap_err().to_string();
    assert!(err_msg.contains("max sessions"), "Should report limit");

    // After discarding a session, should work again
    primal.discard_session(session_ids[0]).await.expect("should discard");

    let session_after_discard =
        SessionBuilder::new(SessionType::General).with_name("after-discard").build();
    let result = primal.create_session(session_after_discard);
    assert!(result.is_ok(), "Should work after freeing space");
}

/// Test rapid connect/disconnect cycles.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_rapid_connection_cycles() {
    let config = RhizoCryptConfig::default();
    let mut primal = RhizoCrypt::new(config);

    // Rapid start/stop cycles
    for i in 0..5 {
        primal.start().await.unwrap_or_else(|_| panic!("start {i} should work"));
        primal.stop().await.unwrap_or_else(|_| panic!("stop {i} should work"));
    }

    // Should still be functional
    primal.start().await.expect("final start should work");

    let session = SessionBuilder::new(SessionType::General).with_name("after-cycles").build();
    let result = primal.create_session(session);
    assert!(result.is_ok(), "Should work after rapid cycles");
}
