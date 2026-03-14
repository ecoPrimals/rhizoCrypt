// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2024–2026 ecoPrimals Project

use super::*;
use crate::event::EventType;
use crate::vertex::VertexBuilder;
use tempfile::TempDir;

fn create_test_store() -> (RedbDagStore, TempDir) {
    let dir = TempDir::new().expect("Failed to create temp dir");
    let db_path = dir.path().join("db.redb");
    let store = RedbDagStore::open(&db_path).expect("Failed to open store");
    (store, dir)
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_put_and_get_vertex() {
    let (store, _dir) = create_test_store();
    let session_id = SessionId::now();
    let vertex = VertexBuilder::new(EventType::SessionStart).build();
    let mut vertex_clone = vertex.clone();
    let vertex_id = vertex_clone.id().expect("Failed to compute vertex ID");
    store.put_vertex(session_id, vertex).await.expect("Failed to put vertex");
    let got = store.get_vertex(session_id, vertex_id).await.expect("Failed to get vertex");
    assert!(got.is_some());
    let got = got.unwrap();
    assert_eq!(got.compute_id().expect("Failed to compute id"), vertex_id);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_genesis_and_frontier() {
    let (store, _dir) = create_test_store();
    let session_id = SessionId::now();
    let genesis = VertexBuilder::new(EventType::SessionStart).build();
    let mut genesis_clone = genesis.clone();
    let genesis_id = genesis_clone.id().expect("Failed to compute genesis ID");
    store.put_vertex(session_id, genesis).await.expect("Failed to put genesis");
    let genesis_list = store.get_genesis(session_id).await.expect("Failed to get genesis");
    assert!(genesis_list.contains(&genesis_id));
    let frontier = store.get_frontier(session_id).await.expect("Failed to get frontier");
    assert!(frontier.contains(&genesis_id));
    let child = VertexBuilder::new(EventType::DataCreate {
        schema: None,
    })
    .with_parent(genesis_id)
    .build();
    let mut child_clone = child.clone();
    let child_id = child_clone.id().expect("Failed to compute child ID");
    store.put_vertex(session_id, child).await.expect("Failed to put child");
    let frontier = store.get_frontier(session_id).await.expect("Failed to get frontier");
    assert!(frontier.contains(&child_id));
    assert!(!frontier.contains(&genesis_id));
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_children() {
    let (store, _dir) = create_test_store();
    let session_id = SessionId::now();
    let parent = VertexBuilder::new(EventType::SessionStart).build();
    let mut parent_clone = parent.clone();
    let parent_id = parent_clone.id().expect("Failed to compute parent ID");
    store.put_vertex(session_id, parent).await.expect("Failed to put parent");
    for i in 0..3 {
        let child = VertexBuilder::new(EventType::DataCreate {
            schema: Some(format!("schema{i}")),
        })
        .with_parent(parent_id)
        .build();
        store.put_vertex(session_id, child).await.expect("Failed to put child");
    }
    let children = store.get_children(session_id, parent_id).await.expect("Failed to get children");
    assert_eq!(children.len(), 3);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_delete_session() {
    let (store, _dir) = create_test_store();
    let session_id = SessionId::now();
    for _ in 0..5 {
        let vertex = VertexBuilder::new(EventType::SessionStart).build();
        store.put_vertex(session_id, vertex).await.expect("Failed to put vertex");
    }
    assert_eq!(store.count_vertices(session_id).await.expect("Failed to count"), 5);
    store.delete_session(session_id).await.expect("Failed to delete session");
    assert_eq!(store.count_vertices(session_id).await.expect("Failed to count"), 0);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_health_and_stats() {
    let (store, _dir) = create_test_store();
    let session_id = SessionId::now();
    assert_eq!(store.health().await, StorageHealth::Healthy);
    for _ in 0..3 {
        let vertex = VertexBuilder::new(EventType::SessionStart).build();
        store.put_vertex(session_id, vertex).await.expect("Failed to put vertex");
    }
    let stats = store.stats().await;
    assert!(stats.vertices >= 3);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_persistence() {
    let (store, dir) = create_test_store();
    let session_id = SessionId::now();
    let vertex = VertexBuilder::new(EventType::SessionStart).build();
    let mut vertex_clone = vertex.clone();
    let vertex_id = vertex_clone.id().expect("Failed to compute vertex ID");
    store.put_vertex(session_id, vertex).await.expect("Failed to put vertex");
    drop(store);
    let db_path = dir.path().join("db.redb");
    let store = RedbDagStore::open(&db_path).expect("Failed to reopen store");
    let got = store.get_vertex(session_id, vertex_id).await.expect("Failed to get vertex");
    assert!(got.is_some());
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_get_nonexistent_vertex() {
    let (store, _dir) = create_test_store();
    let session_id = SessionId::now();
    let vertex_id = VertexId::from_bytes(b"nonexistent vertex id");
    let got = store.get_vertex(session_id, vertex_id).await.expect("Failed to get vertex");
    assert!(got.is_none());
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_get_vertices_batch() {
    let (store, _dir) = create_test_store();
    let session_id = SessionId::now();
    let vertex = VertexBuilder::new(EventType::SessionStart).build();
    let mut vertex_clone = vertex.clone();
    let vertex_id = vertex_clone.id().expect("Failed to compute vertex ID");
    store.put_vertex(session_id, vertex).await.expect("Failed to put vertex");
    let nonexistent = VertexId::from_bytes(b"nonexistent vertex id");
    let batch_ids = [vertex_id, nonexistent, vertex_id];
    let results =
        store.get_vertices(session_id, &batch_ids).await.expect("Failed to get vertices batch");
    assert_eq!(results.len(), 3);
    assert!(results[0].is_some());
    assert!(results[1].is_none());
    assert!(results[2].is_some());
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_exists() {
    let (store, _dir) = create_test_store();
    let session_id = SessionId::now();
    let vertex = VertexBuilder::new(EventType::SessionStart).build();
    let mut vertex_clone = vertex.clone();
    let vertex_id = vertex_clone.id().expect("Failed to compute vertex ID");
    store.put_vertex(session_id, vertex).await.expect("Failed to put vertex");
    assert!(store.exists(session_id, vertex_id).await.expect("Failed to check exists"));
    let absent = VertexId::from_bytes(b"absent");
    assert!(!store.exists(session_id, absent).await.expect("Failed to check exists"));
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_count_vertices() {
    let (store, _dir) = create_test_store();
    let session_id = SessionId::now();
    assert_eq!(store.count_vertices(session_id).await.expect("Failed to count"), 0);
    for _ in 0..5 {
        let vertex = VertexBuilder::new(EventType::SessionStart).build();
        store.put_vertex(session_id, vertex).await.expect("Failed to put vertex");
    }
    assert_eq!(store.count_vertices(session_id).await.expect("Failed to count"), 5);
    let other_session = SessionId::now();
    assert_eq!(store.count_vertices(other_session).await.expect("Failed to count"), 0);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_update_frontier() {
    let (store, _dir) = create_test_store();
    let session_id = SessionId::now();
    let vertex = VertexBuilder::new(EventType::SessionStart).build();
    let mut vertex_clone = vertex.clone();
    let vertex_id = vertex_clone.id().expect("Failed to compute vertex ID");
    store.put_vertex(session_id, vertex).await.expect("Failed to put vertex");
    let new_frontier = VertexId::from_bytes(b"new frontier vertex id 32 bytes!!");
    store
        .update_frontier(session_id, new_frontier, &[vertex_id])
        .await
        .expect("Failed to update frontier");
    let frontier = store.get_frontier(session_id).await.expect("Failed to get frontier");
    assert!(frontier.contains(&new_frontier));
    assert!(!frontier.contains(&vertex_id));
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_multiple_sessions() {
    let (store, _dir) = create_test_store();
    let session1 = SessionId::now();
    let session2 = SessionId::now();
    let v1 = VertexBuilder::new(EventType::SessionStart).build();
    let mut v1_clone = v1.clone();
    let v1_id = v1_clone.id().expect("Failed to compute v1 ID");
    let v2 = VertexBuilder::new(EventType::SessionStart).build();
    let mut v2_clone = v2.clone();
    let v2_id = v2_clone.id().expect("Failed to compute v2 ID");
    store.put_vertex(session1, v1).await.expect("Failed to put v1");
    store.put_vertex(session2, v2).await.expect("Failed to put v2");
    let got1 = store.get_vertex(session1, v1_id).await.expect("Failed to get v1");
    let got2 = store.get_vertex(session2, v2_id).await.expect("Failed to get v2");
    assert!(got1.is_some());
    assert!(got2.is_some());
    assert!(store.get_vertex(session1, v2_id).await.expect("Failed to get").is_none());
    assert!(store.get_vertex(session2, v1_id).await.expect("Failed to get").is_none());
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_concurrent_reads() {
    let (store, _dir) = create_test_store();
    let session_id = SessionId::now();
    let vertex = VertexBuilder::new(EventType::SessionStart).build();
    let mut vertex_clone = vertex.clone();
    let vertex_id = vertex_clone.id().expect("Failed to compute vertex ID");
    store.put_vertex(session_id, vertex).await.expect("Failed to put vertex");
    let store = std::sync::Arc::new(store);
    let mut handles = Vec::new();
    for _ in 0..10 {
        let store_clone = store.clone();
        let sid = session_id;
        let vid = vertex_id;
        handles.push(tokio::spawn(async move {
            store_clone.get_vertex(sid, vid).await.expect("Failed to get")
        }));
    }
    for h in handles {
        let result = h.await.expect("Task panicked");
        assert!(result.is_some());
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_large_batch() {
    let (store, _dir) = create_test_store();
    let session_id = SessionId::now();
    let parent = VertexBuilder::new(EventType::SessionStart).build();
    let mut parent_clone = parent.clone();
    let parent_id = parent_clone.id().expect("Failed to compute parent ID");
    store.put_vertex(session_id, parent).await.expect("Failed to put parent");
    let mut vertex_ids = Vec::with_capacity(120);
    for i in 0..120 {
        let child = VertexBuilder::new(EventType::DataCreate {
            schema: Some(format!("schema{i}")),
        })
        .with_parent(parent_id)
        .build();
        let mut child_clone = child.clone();
        let child_id = child_clone.id().expect("Failed to compute child ID");
        vertex_ids.push(child_id);
        store.put_vertex(session_id, child).await.expect("Failed to put child");
    }
    assert_eq!(store.count_vertices(session_id).await.expect("Failed to count"), 121);
    let results =
        store.get_vertices(session_id, &vertex_ids).await.expect("Failed to get vertices batch");
    assert_eq!(results.len(), 120);
    assert!(results.iter().all(Option::is_some));
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_complex_dag_structure() {
    let (store, _dir) = create_test_store();
    let session_id = SessionId::now();
    let genesis = VertexBuilder::new(EventType::SessionStart).build();
    let mut genesis_clone = genesis.clone();
    let genesis_id = genesis_clone.id().expect("Failed to compute genesis ID");
    store.put_vertex(session_id, genesis).await.expect("Failed to put genesis");
    let a = VertexBuilder::new(EventType::DataCreate {
        schema: None,
    })
    .with_parent(genesis_id)
    .build();
    let mut a_clone = a.clone();
    let a_id = a_clone.id().expect("Failed to compute A ID");
    store.put_vertex(session_id, a).await.expect("Failed to put A");
    let b = VertexBuilder::new(EventType::DataCreate {
        schema: None,
    })
    .with_parent(genesis_id)
    .build();
    let mut b_clone = b.clone();
    let b_id = b_clone.id().expect("Failed to compute B ID");
    store.put_vertex(session_id, b).await.expect("Failed to put B");
    let c = VertexBuilder::new(EventType::DataCreate {
        schema: None,
    })
    .with_parents([a_id, b_id])
    .build();
    let mut c_clone = c.clone();
    let c_id = c_clone.id().expect("Failed to compute C ID");
    store.put_vertex(session_id, c).await.expect("Failed to put C");
    let frontier = store.get_frontier(session_id).await.expect("Failed to get frontier");
    assert!(frontier.contains(&c_id));
    assert!(!frontier.contains(&a_id));
    assert!(!frontier.contains(&b_id));
    assert!(!frontier.contains(&genesis_id));
    let genesis_list = store.get_genesis(session_id).await.expect("Failed to get genesis");
    assert!(genesis_list.contains(&genesis_id));
    let children_a = store.get_children(session_id, a_id).await.expect("Failed to get children");
    assert_eq!(children_a.len(), 1);
    assert!(children_a.contains(&c_id));
    let children_b = store.get_children(session_id, b_id).await.expect("Failed to get children");
    assert_eq!(children_b.len(), 1);
    assert!(children_b.contains(&c_id));
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_reopen_after_crash() {
    let (store, dir) = create_test_store();
    let session_id = SessionId::now();
    let genesis = VertexBuilder::new(EventType::SessionStart).build();
    let mut genesis_clone = genesis.clone();
    let genesis_id = genesis_clone.id().expect("Failed to compute genesis ID");
    store.put_vertex(session_id, genesis).await.expect("Failed to put genesis");
    let child = VertexBuilder::new(EventType::DataCreate {
        schema: None,
    })
    .with_parent(genesis_id)
    .build();
    let mut child_clone = child.clone();
    let child_id = child_clone.id().expect("Failed to compute child ID");
    store.put_vertex(session_id, child).await.expect("Failed to put child");
    drop(store);
    let db_path = dir.path().join("db.redb");
    let store = RedbDagStore::open(&db_path).expect("Failed to reopen store");
    let got_genesis =
        store.get_vertex(session_id, genesis_id).await.expect("Failed to get genesis");
    assert!(got_genesis.is_some());
    let got_child = store.get_vertex(session_id, child_id).await.expect("Failed to get child");
    assert!(got_child.is_some());
    let genesis_list = store.get_genesis(session_id).await.expect("Failed to get genesis");
    assert!(genesis_list.contains(&genesis_id));
    let frontier = store.get_frontier(session_id).await.expect("Failed to get frontier");
    assert!(frontier.contains(&child_id));
    let children =
        store.get_children(session_id, genesis_id).await.expect("Failed to get children");
    assert_eq!(children.len(), 1);
    assert!(children.contains(&child_id));
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_session_lifecycle_create_populate_resolve_cleanup() {
    let (store, _dir) = create_test_store();
    let session_id = SessionId::now();
    let genesis = VertexBuilder::new(EventType::SessionStart).build();
    let mut genesis_clone = genesis.clone();
    let genesis_id = genesis_clone.id().expect("Failed to compute genesis ID");
    store.put_vertex(session_id, genesis).await.expect("Failed to put genesis");
    let child = VertexBuilder::new(EventType::DataCreate {
        schema: None,
    })
    .with_parent(genesis_id)
    .build();
    let mut child_clone = child.clone();
    let child_id = child_clone.id().expect("Failed to compute child ID");
    store.put_vertex(session_id, child).await.expect("Failed to put child");
    assert_eq!(store.count_vertices(session_id).await.expect("Failed to count"), 2);
    let frontier = store.get_frontier(session_id).await.expect("Failed to get frontier");
    assert!(frontier.contains(&child_id));
    store.delete_session(session_id).await.expect("Failed to delete");
    assert_eq!(store.count_vertices(session_id).await.expect("Failed to count"), 0);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_put_duplicate_vertex_idempotent() {
    let (store, _dir) = create_test_store();
    let session_id = SessionId::now();
    let vertex = VertexBuilder::new(EventType::SessionStart).build();
    let mut vertex_clone = vertex.clone();
    let vertex_id = vertex_clone.id().expect("Failed to compute ID");
    store.put_vertex(session_id, vertex.clone()).await.expect("Failed to put");
    store.put_vertex(session_id, vertex).await.expect("Failed to put duplicate");
    let got = store.get_vertex(session_id, vertex_id).await.expect("Failed to get");
    assert!(got.is_some());
    assert_eq!(store.count_vertices(session_id).await.expect("Failed to count"), 1);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_vertex_with_multiple_genesis_in_session() {
    let (store, _dir) = create_test_store();
    let session_id = SessionId::now();
    let g1 = VertexBuilder::new(EventType::SessionStart).build();
    let mut g1_clone = g1.clone();
    let g1_id = g1_clone.id().expect("Failed to compute g1 ID");
    let g2 = VertexBuilder::new(EventType::SessionStart).build();
    let mut g2_clone = g2.clone();
    let g2_id = g2_clone.id().expect("Failed to compute g2 ID");
    store.put_vertex(session_id, g1).await.expect("Failed to put g1");
    store.put_vertex(session_id, g2).await.expect("Failed to put g2");
    let genesis_list = store.get_genesis(session_id).await.expect("Failed to get genesis");
    assert_eq!(genesis_list.len(), 2);
    assert!(genesis_list.contains(&g1_id));
    assert!(genesis_list.contains(&g2_id));
}
