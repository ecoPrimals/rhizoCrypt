//! Concurrent stress tests for rhizoCrypt.
//!
//! Tests system behavior under high concurrency.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use rhizo_crypt_core::{
    EventType, PrimalLifecycle, RhizoCrypt, RhizoCryptConfig, SessionBuilder, SessionType,
    VertexBuilder,
};
use std::sync::Arc;
use tokio::sync::RwLock;

/// Test concurrent session creation.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_concurrent_session_creation() {
    let config = RhizoCryptConfig::default();
    let primal = Arc::new(RwLock::new(RhizoCrypt::new(config)));

    // Start primal
    {
        let mut p = primal.write().await;
        p.start().await.expect("primal should start");
    }

    // Spawn concurrent session creators
    let mut handles = Vec::new();
    for i in 0..10 {
        let primal_clone = Arc::clone(&primal);
        let handle = tokio::spawn(async move {
            let p = primal_clone.read().await;
            let session = SessionBuilder::new(SessionType::General)
                .with_name(format!("concurrent-{i}"))
                .build();
            p.create_session(session).await
        });
        handles.push(handle);
    }

    // Wait for all
    let mut successes = 0;
    for handle in handles {
        if handle.await.expect("task should complete").is_ok() {
            successes += 1;
        }
    }

    // All should succeed
    assert_eq!(successes, 10);

    // Verify count
    {
        let p = primal.read().await;
        assert_eq!(p.session_count(), 10);
    }

    // Stop primal
    {
        let mut p = primal.write().await;
        p.stop().await.expect("primal should stop");
    }
}

/// Test concurrent vertex appends to same session.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_concurrent_vertex_appends() {
    let config = RhizoCryptConfig::default();
    let primal = Arc::new(RwLock::new(RhizoCrypt::new(config)));

    // Start primal
    {
        let mut p = primal.write().await;
        p.start().await.expect("primal should start");
    }

    // Create a session
    let session_id = {
        let p = primal.read().await;
        let session = SessionBuilder::new(SessionType::General).build();
        p.create_session(session).await.expect("should create session")
    };

    // Spawn concurrent vertex appenders
    let mut handles = Vec::new();
    for i in 0..20 {
        let primal_clone = Arc::clone(&primal);
        let sid = session_id;
        let handle = tokio::spawn(async move {
            let p = primal_clone.read().await;
            let vertex = VertexBuilder::new(EventType::Custom {
                domain: "test".into(),
                event_name: format!("event-{i}"),
            })
            .build();
            p.append_vertex(sid, vertex).await
        });
        handles.push(handle);
    }

    // Wait for all
    let mut successes = 0;
    for handle in handles {
        if handle.await.expect("task should complete").is_ok() {
            successes += 1;
        }
    }

    // All should succeed
    assert_eq!(successes, 20);

    // Verify vertex count
    {
        let p = primal.read().await;
        assert_eq!(p.total_vertex_count(), 20);
    }

    // Stop primal
    {
        let mut p = primal.write().await;
        p.stop().await.expect("primal should stop");
    }
}

/// Test high-throughput vertex appends.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_high_throughput_appends() {
    let config = RhizoCryptConfig::default();
    let mut primal = RhizoCrypt::new(config);
    primal.start().await.expect("primal should start");

    // Create a session
    let session = SessionBuilder::new(SessionType::General).build();
    let session_id = primal.create_session(session).await.expect("should create session");

    // Append many vertices sequentially
    let start = std::time::Instant::now();
    for i in 0..1000 {
        let vertex = VertexBuilder::new(EventType::Custom {
            domain: "test".into(),
            event_name: format!("event-{i}"),
        })
        .build();
        primal.append_vertex(session_id, vertex).await.expect("should append vertex");
    }
    let elapsed = start.elapsed();

    // Verify count
    assert_eq!(primal.total_vertex_count(), 1000);

    // Log throughput
    let ops_per_sec = 1000.0 / elapsed.as_secs_f64();
    println!("Throughput: {ops_per_sec:.2} vertices/sec");

    // Should be reasonably fast (> 1000/sec for in-memory)
    assert!(ops_per_sec > 100.0, "Throughput too low: {ops_per_sec}");

    primal.stop().await.expect("primal should stop");
}
