// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2024–2026 ecoPrimals Project

use super::*;
use crate::event::EventType;
use crate::vertex::VertexBuilder;
use tempfile::TempDir;

fn create_test_store() -> (SledDagStore, TempDir) {
    let dir = TempDir::new().expect("Failed to create temp dir");
    let store = SledDagStore::open(dir.path()).expect("Failed to open store");
    (store, dir)
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_put_and_get_vertex() {
    let (store, _dir) = create_test_store();
    let session_id = SessionId::now();

    let vertex = VertexBuilder::new(EventType::SessionStart).build();
    let mut vertex_clone = vertex.clone();
    let vertex_id = vertex_clone.id().unwrap();

    store.put_vertex(session_id, vertex).await.unwrap();

    let retrieved = store.get_vertex(session_id, vertex_id).await.unwrap();
    assert!(retrieved.is_some());

    let mut retrieved_vertex = retrieved.unwrap();
    assert_eq!(retrieved_vertex.id().unwrap(), vertex_id);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_genesis_and_frontier() {
    let (store, _dir) = create_test_store();
    let session_id = SessionId::now();

    // Add genesis vertex
    let v1 = VertexBuilder::new(EventType::SessionStart).build();
    let mut v1_clone = v1.clone();
    let v1_id = v1_clone.id().unwrap();
    store.put_vertex(session_id, v1).await.unwrap();

    // Check genesis
    let genesis = store.get_genesis(session_id).await.unwrap();
    assert_eq!(genesis.len(), 1);
    assert!(genesis.contains(&v1_id));

    // Check frontier
    let frontier = store.get_frontier(session_id).await.unwrap();
    assert_eq!(frontier.len(), 1);
    assert!(frontier.contains(&v1_id));

    // Add child vertex
    let v2 = VertexBuilder::new(EventType::DataCreate {
        schema: None,
    })
    .with_parent(v1_id)
    .build();
    let mut v2_clone = v2.clone();
    let v2_id = v2_clone.id().unwrap();
    store.put_vertex(session_id, v2).await.unwrap();

    // Frontier should now be v2 only
    let frontier = store.get_frontier(session_id).await.unwrap();
    assert_eq!(frontier.len(), 1);
    assert!(frontier.contains(&v2_id));
    assert!(!frontier.contains(&v1_id));

    // Genesis should still be v1
    let genesis = store.get_genesis(session_id).await.unwrap();
    assert!(genesis.contains(&v1_id));
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_children() {
    let (store, _dir) = create_test_store();
    let session_id = SessionId::now();

    // Add parent
    let parent = VertexBuilder::new(EventType::SessionStart).build();
    let mut parent_clone = parent.clone();
    let parent_id = parent_clone.id().unwrap();
    store.put_vertex(session_id, parent).await.unwrap();

    // Add children
    for i in 0..3 {
        let child = VertexBuilder::new(EventType::DataCreate {
            schema: Some(format!("schema{i}")),
        })
        .with_parent(parent_id)
        .build();
        store.put_vertex(session_id, child).await.unwrap();
    }

    let children = store.get_children(session_id, parent_id).await.unwrap();
    assert_eq!(children.len(), 3);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_delete_session() {
    let (store, _dir) = create_test_store();
    let session_id = SessionId::now();

    // Add some vertices
    for _ in 0..5 {
        let vertex = VertexBuilder::new(EventType::SessionStart).build();
        store.put_vertex(session_id, vertex).await.unwrap();
    }

    assert_eq!(store.count_vertices(session_id).await.unwrap(), 5);

    // Delete session
    store.delete_session(session_id).await.unwrap();
    assert_eq!(store.count_vertices(session_id).await.unwrap(), 0);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_health_and_stats() {
    let (store, _dir) = create_test_store();
    let session_id = SessionId::now();

    // Health should be healthy
    assert!(matches!(store.health().await, StorageHealth::Healthy));

    // Add some data
    for _ in 0..3 {
        let vertex = VertexBuilder::new(EventType::SessionStart).build();
        store.put_vertex(session_id, vertex).await.unwrap();
    }

    let stats = store.stats().await;
    assert!(stats.vertices >= 3);
    assert!(stats.write_ops >= 3);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_persistence() {
    let dir = TempDir::new().expect("Failed to create temp dir");
    let session_id = SessionId::now();
    let vertex_id;

    // Create store and add data
    {
        let store = SledDagStore::open(dir.path()).unwrap();
        let vertex = VertexBuilder::new(EventType::SessionStart).build();
        let mut vertex_clone = vertex.clone();
        vertex_id = vertex_clone.id().unwrap();
        store.put_vertex(session_id, vertex).await.unwrap();
        store.flush().await.unwrap();
    }

    // Reopen store and verify data persisted
    {
        let store = SledDagStore::open(dir.path()).unwrap();
        let retrieved = store.get_vertex(session_id, vertex_id).await.unwrap();
        assert!(retrieved.is_some());
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_pure_rust_excellence() {
    let (store, _dir) = create_test_store();
    let session_id = SessionId::now();
    let vertex = VertexBuilder::new(EventType::SessionStart).build();
    store.put_vertex(session_id, vertex).await.unwrap();
    assert!(matches!(store.health().await, StorageHealth::Healthy));
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_path() {
    let (store, dir) = create_test_store();
    assert_eq!(store.path(), dir.path());
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_debug_impl() {
    let (store, _dir) = create_test_store();
    let debug = format!("{store:?}");
    assert!(debug.contains("SledDagStore"));
    assert!(debug.contains("read_ops"));
    assert!(debug.contains("write_ops"));
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_export() {
    let (store, _dir) = create_test_store();
    let session_id = SessionId::now();

    let vertex = VertexBuilder::new(EventType::SessionStart).build();
    store.put_vertex(session_id, vertex).await.unwrap();
    store.flush().await.unwrap();

    let export_data = store.export();
    assert!(!export_data.is_empty(), "export should return tree data");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_exists() {
    let (store, _dir) = create_test_store();
    let session_id = SessionId::now();

    let vertex = VertexBuilder::new(EventType::SessionStart).build();
    let mut vc = vertex.clone();
    let vid = vc.id().unwrap();
    store.put_vertex(session_id, vertex).await.unwrap();

    assert!(store.exists(session_id, vid).await.unwrap());

    let fake_id = VertexId::from_bytes(&[0u8; 32]);
    assert!(!store.exists(session_id, fake_id).await.unwrap());
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_get_vertices_batch() {
    let (store, _dir) = create_test_store();
    let session_id = SessionId::now();

    let v1 = VertexBuilder::new(EventType::SessionStart).build();
    let mut v1c = v1.clone();
    let v1_id = v1c.id().unwrap();
    store.put_vertex(session_id, v1).await.unwrap();

    let v2 = VertexBuilder::new(EventType::DataCreate {
        schema: None,
    })
    .with_parent(v1_id)
    .build();
    let mut v2c = v2.clone();
    let v2_id = v2c.id().unwrap();
    store.put_vertex(session_id, v2).await.unwrap();

    let fake_id = VertexId::from_bytes(&[0u8; 32]);
    let results = store.get_vertices(session_id, &[v1_id, v2_id, fake_id]).await.unwrap();
    assert_eq!(results.len(), 3);
    assert!(results[0].is_some());
    assert!(results[1].is_some());
    assert!(results[2].is_none());
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_get_vertices_empty() {
    let (store, _dir) = create_test_store();
    let session_id = SessionId::now();
    let results = store.get_vertices(session_id, &[]).await.unwrap();
    assert!(results.is_empty());
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_get_vertex_not_found() {
    let (store, _dir) = create_test_store();
    let session_id = SessionId::now();
    let result = store.get_vertex(session_id, VertexId::from_bytes(&[0u8; 32])).await.unwrap();
    assert!(result.is_none());
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_update_frontier() {
    let (store, _dir) = create_test_store();
    let session_id = SessionId::now();

    let v1 = VertexBuilder::new(EventType::SessionStart).build();
    let mut v1c = v1.clone();
    let v1_id = v1c.id().unwrap();
    store.put_vertex(session_id, v1).await.unwrap();

    let frontier = store.get_frontier(session_id).await.unwrap();
    assert!(frontier.contains(&v1_id));

    let new_id = VertexId::from_bytes(&[0u8; 32]);
    store.update_frontier(session_id, new_id, &[v1_id]).await.unwrap();

    let frontier = store.get_frontier(session_id).await.unwrap();
    assert!(frontier.contains(&new_id));
    assert!(!frontier.contains(&v1_id));
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_update_frontier_empty_consumed() {
    let (store, _dir) = create_test_store();
    let session_id = SessionId::now();

    let new_id = VertexId::from_bytes(&[0u8; 32]);
    store.update_frontier(session_id, new_id, &[]).await.unwrap();

    let frontier = store.get_frontier(session_id).await.unwrap();
    assert!(frontier.contains(&new_id));
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_get_children_empty() {
    let (store, _dir) = create_test_store();
    let session_id = SessionId::now();
    let children = store.get_children(session_id, VertexId::from_bytes(&[0u8; 32])).await.unwrap();
    assert!(children.is_empty());
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_get_genesis_empty() {
    let (store, _dir) = create_test_store();
    let session_id = SessionId::now();
    let genesis = store.get_genesis(session_id).await.unwrap();
    assert!(genesis.is_empty());
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_get_frontier_empty() {
    let (store, _dir) = create_test_store();
    let session_id = SessionId::now();
    let frontier = store.get_frontier(session_id).await.unwrap();
    assert!(frontier.is_empty());
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_count_vertices_empty() {
    let (store, _dir) = create_test_store();
    let session_id = SessionId::now();
    assert_eq!(store.count_vertices(session_id).await.unwrap(), 0);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_delete_session_nonexistent() {
    let (store, _dir) = create_test_store();
    let session_id = SessionId::now();
    store.delete_session(session_id).await.unwrap();
    assert_eq!(store.count_vertices(session_id).await.unwrap(), 0);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_multiple_sessions() {
    let (store, _dir) = create_test_store();
    let s1 = SessionId::now();
    let s2 = SessionId::now();

    let v1 = VertexBuilder::new(EventType::SessionStart).build();
    let v2 = VertexBuilder::new(EventType::SessionStart).build();
    store.put_vertex(s1, v1).await.unwrap();
    store.put_vertex(s2, v2).await.unwrap();

    assert_eq!(store.count_vertices(s1).await.unwrap(), 1);
    assert_eq!(store.count_vertices(s2).await.unwrap(), 1);

    store.delete_session(s1).await.unwrap();
    assert_eq!(store.count_vertices(s1).await.unwrap(), 0);
    assert_eq!(store.count_vertices(s2).await.unwrap(), 1);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_stats_after_operations() {
    let (store, _dir) = create_test_store();
    let session_id = SessionId::now();

    let initial_stats = store.stats().await;
    let initial_reads = initial_stats.read_ops;
    let initial_writes = initial_stats.write_ops;

    let vertex = VertexBuilder::new(EventType::SessionStart).build();
    store.put_vertex(session_id, vertex).await.unwrap();

    let _ = store.get_vertex(session_id, VertexId::from_bytes(&[0u8; 32])).await;

    let stats = store.stats().await;
    assert!(stats.write_ops > initial_writes);
    assert!(stats.read_ops > initial_reads);
    assert!(stats.bytes_used > 0);
    assert_eq!(stats.vertices, 1);
    assert_eq!(stats.sessions, 1);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_concurrent_reads() {
    let (store, _dir) = create_test_store();
    let session_id = SessionId::now();

    let vertex = VertexBuilder::new(EventType::SessionStart).build();
    let mut vc = vertex.clone();
    let vid = vc.id().unwrap();
    store.put_vertex(session_id, vertex).await.unwrap();

    let store = std::sync::Arc::new(store);
    let mut handles = vec![];
    for _ in 0..10 {
        let s = std::sync::Arc::clone(&store);
        handles.push(tokio::spawn(async move {
            s.get_vertex(session_id, vid).await.unwrap().is_some()
        }));
    }
    for h in handles {
        assert!(h.await.unwrap());
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_open_creates_parent_dirs() {
    let dir = TempDir::new().unwrap();
    let nested = dir.path().join("deep").join("nested").join("path");
    let store = SledDagStore::open(&nested).unwrap();
    assert!(store.path().exists());
}
