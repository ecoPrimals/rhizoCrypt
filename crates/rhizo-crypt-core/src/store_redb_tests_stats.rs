// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2024–2026 ecoPrimals Project

use super::*;
use crate::event::EventType;
use crate::vertex::VertexBuilder;

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_stats_accuracy() {
    let (store, _dir) = create_test_store();
    let session_id = SessionId::now();
    let stats_before = store.stats().await;
    let vertex = VertexBuilder::new(EventType::SessionStart).build();
    let mut vertex_clone = vertex.clone();
    let vertex_id = vertex_clone.id().expect("Failed to compute vertex ID");
    store.put_vertex(session_id, vertex).await.expect("Failed to put vertex");
    store.get_vertex(session_id, vertex_id).await.expect("Failed to get vertex");
    let stats_after = store.stats().await;
    assert!(stats_after.write_ops > stats_before.write_ops);
    assert!(stats_after.read_ops > stats_before.read_ops);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_count_vertices_multiple_sessions() {
    let (store, _dir) = create_test_store();
    let session1 = SessionId::now();
    let session2 = SessionId::now();
    for _ in 0..3 {
        let vertex = VertexBuilder::new(EventType::SessionStart).build();
        store.put_vertex(session1, vertex).await.expect("Failed to put vertex");
    }
    for _ in 0..7 {
        let vertex = VertexBuilder::new(EventType::SessionStart).build();
        store.put_vertex(session2, vertex).await.expect("Failed to put vertex");
    }
    assert_eq!(store.count_vertices(session1).await.expect("Failed to count"), 3);
    assert_eq!(store.count_vertices(session2).await.expect("Failed to count"), 7);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_session_count_via_stats() {
    let (store, _dir) = create_test_store();
    let stats_empty = store.stats().await;
    assert_eq!(stats_empty.sessions, 0);
    let session1 = SessionId::now();
    let session2 = SessionId::now();
    let v1 = VertexBuilder::new(EventType::SessionStart).build();
    let v2 = VertexBuilder::new(EventType::SessionStart).build();
    store.put_vertex(session1, v1).await.expect("Failed to put v1");
    store.put_vertex(session2, v2).await.expect("Failed to put v2");
    let stats_after = store.stats().await;
    assert_eq!(stats_after.sessions, 2);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_storage_health() {
    let (store, _dir) = create_test_store();
    assert_eq!(store.health().await, StorageHealth::Healthy);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_storage_stats_with_data() {
    let (store, _dir) = create_test_store();
    let session1 = SessionId::now();
    let session2 = SessionId::now();
    for _ in 0..5 {
        let vertex = VertexBuilder::new(EventType::SessionStart).build();
        store.put_vertex(session1, vertex).await.expect("Failed to put vertex");
    }
    for _ in 0..10 {
        let vertex = VertexBuilder::new(EventType::SessionStart).build();
        store.put_vertex(session2, vertex).await.expect("Failed to put vertex");
    }
    let stats = store.stats().await;
    assert_eq!(stats.sessions, 2);
    assert_eq!(stats.vertices, 15);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_stats_read_write_ops_tracked() {
    let (store, _dir) = create_test_store();
    let session_id = SessionId::now();
    let stats_before = store.stats().await;
    let vertex = VertexBuilder::new(EventType::SessionStart).build();
    let mut vertex_clone = vertex.clone();
    let vertex_id = vertex_clone.id().expect("Failed to compute ID");
    store.put_vertex(session_id, vertex).await.expect("Failed to put");
    store.get_vertex(session_id, vertex_id).await.expect("Failed to get");
    store.exists(session_id, vertex_id).await.expect("Failed to exists");
    let stats_after = store.stats().await;
    assert!(stats_after.write_ops > stats_before.write_ops);
    assert!(stats_after.read_ops > stats_before.read_ops + 1);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_stats_bytes_used_reflects_database_file() {
    let (store, _dir) = create_test_store();
    let stats_empty = store.stats().await;
    let session_id = SessionId::now();
    for _ in 0..10 {
        let vertex = VertexBuilder::new(EventType::SessionStart).build();
        store.put_vertex(session_id, vertex).await.expect("Failed to put");
    }
    let stats_after = store.stats().await;
    assert!(stats_after.bytes_used >= stats_empty.bytes_used);
    assert!(stats_after.vertices >= 10);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_count_vertices_increments_read_ops() {
    let (store, _dir) = create_test_store();
    let session_id = SessionId::now();
    store
        .put_vertex(session_id, VertexBuilder::new(EventType::SessionStart).build())
        .await
        .expect("Failed to put");
    let stats_before = store.stats().await;
    store.count_vertices(session_id).await.expect("Failed to count");
    let stats_after = store.stats().await;
    assert!(stats_after.read_ops > stats_before.read_ops);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_stats_full_structure_all_fields() {
    let (store, _dir) = create_test_store();
    let stats = store.stats().await;
    assert_eq!(stats.sessions, 0);
    assert_eq!(stats.vertices, 0);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_stats_bytes_used_nonzero_with_data() {
    let (store, _dir) = create_test_store();
    let session_id = SessionId::now();
    for _ in 0..20 {
        let vertex = VertexBuilder::new(EventType::SessionStart).build();
        store.put_vertex(session_id, vertex).await.expect("Failed to put");
    }
    let stats = store.stats().await;
    assert!(stats.bytes_used > 0, "bytes_used should be non-zero with data");
    assert_eq!(stats.vertices, 20);
    assert_eq!(stats.sessions, 1);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_stats_consistency_with_multiple_sessions() {
    let (store, _dir) = create_test_store();
    let session1 = SessionId::now();
    let session2 = SessionId::now();
    for _ in 0..4 {
        store
            .put_vertex(session1, VertexBuilder::new(EventType::SessionStart).build())
            .await
            .expect("Failed to put");
    }
    for _ in 0..6 {
        store
            .put_vertex(session2, VertexBuilder::new(EventType::SessionStart).build())
            .await
            .expect("Failed to put");
    }
    let stats = store.stats().await;
    assert_eq!(stats.sessions, 2);
    assert_eq!(stats.vertices, 10);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_stats_read_ops_from_get_children_genesis_frontier() {
    let (store, _dir) = create_test_store();
    let session_id = SessionId::now();
    let genesis = VertexBuilder::new(EventType::SessionStart).build();
    let mut genesis_clone = genesis.clone();
    let genesis_id = genesis_clone.id().expect("Failed to compute genesis ID");
    store.put_vertex(session_id, genesis).await.expect("Failed to put genesis");
    let stats_before = store.stats().await;
    store.get_children(session_id, genesis_id).await.expect("Failed to get children");
    store.get_genesis(session_id).await.expect("Failed to get genesis");
    store.get_frontier(session_id).await.expect("Failed to get frontier");
    let stats_after = store.stats().await;
    assert!(stats_after.read_ops >= stats_before.read_ops + 3);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_delete_session_increments_write_ops() {
    let (store, _dir) = create_test_store();
    let session_id = SessionId::now();
    store
        .put_vertex(session_id, VertexBuilder::new(EventType::SessionStart).build())
        .await
        .expect("Failed to put");
    let stats_before = store.stats().await;
    store.delete_session(session_id).await.expect("Failed to delete");
    let stats_after = store.stats().await;
    assert!(stats_after.write_ops > stats_before.write_ops);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_update_frontier_increments_write_ops() {
    let (store, _dir) = create_test_store();
    let session_id = SessionId::now();
    let stats_before = store.stats().await;
    store
        .update_frontier(
            session_id,
            VertexId::from_bytes(b"frontier vertex id 32 bytes!!!!!!"),
            &[],
        )
        .await
        .expect("Failed to update frontier");
    let stats_after = store.stats().await;
    assert!(stats_after.write_ops > stats_before.write_ops);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_get_vertices_increments_read_ops() {
    let (store, _dir) = create_test_store();
    let session_id = SessionId::now();
    let vertex = VertexBuilder::new(EventType::SessionStart).build();
    let mut vertex_clone = vertex.clone();
    let vertex_id = vertex_clone.id().expect("Failed to compute ID");
    store.put_vertex(session_id, vertex).await.expect("Failed to put");
    let stats_before = store.stats().await;
    store.get_vertices(session_id, &[vertex_id]).await.expect("Failed to get vertices");
    let stats_after = store.stats().await;
    assert!(stats_after.read_ops > stats_before.read_ops);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_stats_after_delete_session() {
    let (store, _dir) = create_test_store();
    let session_id = SessionId::now();
    for _ in 0..5 {
        store
            .put_vertex(session_id, VertexBuilder::new(EventType::SessionStart).build())
            .await
            .expect("Failed to put");
    }
    let stats_before = store.stats().await;
    store.delete_session(session_id).await.expect("Failed to delete");
    let stats_after = store.stats().await;
    assert!(stats_after.vertices < stats_before.vertices);
    assert!(stats_after.sessions <= stats_before.sessions);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_count_vertices_empty_range() {
    let (store, _dir) = create_test_store();
    let session_id = SessionId::now();
    let count = store.count_vertices(session_id).await.expect("Failed to count");
    assert_eq!(count, 0);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_count_vertices_session_prefix_range() {
    let (store, _dir) = create_test_store();
    let session1 = SessionId::now();
    let session2 = SessionId::now();
    for _ in 0..4 {
        store
            .put_vertex(session1, VertexBuilder::new(EventType::SessionStart).build())
            .await
            .expect("Failed to put");
    }
    for _ in 0..6 {
        store
            .put_vertex(session2, VertexBuilder::new(EventType::SessionStart).build())
            .await
            .expect("Failed to put");
    }
    assert_eq!(store.count_vertices(session1).await.expect("count"), 4);
    assert_eq!(store.count_vertices(session2).await.expect("count"), 6);
}

#[cfg(unix)]
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_stats_bytes_used_when_path_deleted() {
    let (store, _dir) = create_test_store();
    let session_id = SessionId::now();
    store
        .put_vertex(session_id, VertexBuilder::new(EventType::SessionStart).build())
        .await
        .expect("Failed to put");
    let db_path = store.path().to_path_buf();
    let _ = std::fs::remove_file(&db_path);
    let stats = store.stats().await;
    assert_eq!(stats.bytes_used, 0, "bytes_used should be 0 when path metadata fails");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_storage_stats_debug_format() {
    let (store, _dir) = create_test_store();
    let stats = store.stats().await;
    let debug_str = format!("{stats:?}");
    assert!(debug_str.contains("sessions"));
    assert!(debug_str.contains("vertices"));
    assert!(debug_str.contains("bytes_used"));
    assert!(debug_str.contains("read_ops"));
    assert!(debug_str.contains("write_ops"));
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_multiple_rapid_calls_same_operation() {
    let (store, _dir) = create_test_store();
    let session_id = SessionId::now();
    let vertex = VertexBuilder::new(EventType::SessionStart).build();
    let mut vertex_clone = vertex.clone();
    let vertex_id = vertex_clone.id().expect("Failed to compute ID");
    store.put_vertex(session_id, vertex).await.expect("Failed to put");
    for _ in 0..5 {
        let _ = store.get_vertex(session_id, vertex_id).await.expect("get");
        let _ = store.count_vertices(session_id).await.expect("count");
        let _ = store.stats().await;
    }
    let stats = store.stats().await;
    assert!(stats.read_ops >= 10);
}
