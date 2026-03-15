// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2024–2026 ecoPrimals Project

use super::*;
use crate::event::EventType;
use crate::vertex::VertexBuilder;
use redb::TableDefinition;
use tempfile::TempDir;

const VERTICES_TABLE: TableDefinition<&[u8], &[u8]> = TableDefinition::new("vertices");

fn create_test_store() -> (RedbDagStore, TempDir) {
    let dir = TempDir::new().expect("Failed to create temp dir");
    let db_path = dir.path().join("db.redb");
    let store = RedbDagStore::open(&db_path).expect("Failed to open store");
    (store, dir)
}

// === Error handling tests ===

#[test]
fn test_open_fails_when_parent_is_file() {
    let dir = TempDir::new().expect("Failed to create temp dir");
    let file_path = dir.path().join("blocker");
    std::fs::File::create(&file_path).expect("Failed to create file");
    let db_path = file_path.join("db.redb");
    let result = RedbDagStore::open(&db_path);
    assert!(result.is_err());
    let err_msg = result.unwrap_err().to_string();
    assert!(err_msg.contains("directory") || err_msg.contains("Failed to create"));
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_get_vertex_fails_on_corrupt_data() {
    let (store, dir) = create_test_store();
    let db_path = dir.path().join("db.redb");
    let session_id = SessionId::now();
    let vertex_id = VertexId::from_bytes(b"corrupt vertex id 32 bytes!!!!!!");

    drop(store);
    let db = redb::Database::create(&db_path).expect("Failed to create db");
    let write_txn = db.begin_write().expect("Failed to begin write");
    {
        let mut table = write_txn.open_table(VERTICES_TABLE).expect("Failed to open table");
        let mut key = session_id.as_bytes().to_vec();
        key.push(b':');
        key.extend_from_slice(vertex_id.as_bytes());
        table.insert(key.as_slice(), b"not valid cbor".as_slice()).expect("Failed to insert");
    }
    write_txn.commit().expect("Failed to commit");
    drop(db);

    let store = RedbDagStore::open(&db_path).expect("Failed to reopen store");
    let result = store.get_vertex(session_id, vertex_id).await;
    assert!(result.is_err());
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_get_vertices_returns_none_for_invalid_cbor() {
    let (store, dir) = create_test_store();
    let session_id = SessionId::now();
    let valid_vertex = VertexBuilder::new(EventType::SessionStart).build();
    let mut valid_clone = valid_vertex.clone();
    let valid_id = valid_clone.id().expect("Failed to compute ID");
    store.put_vertex(session_id, valid_vertex).await.expect("Failed to put");

    let corrupt_id = VertexId::from_bytes(b"corrupt vertex id 32 bytes!!!!!!");
    let db_path = dir.path().join("db.redb");
    drop(store);
    let db = redb::Database::create(&db_path).expect("Failed to create db");
    let write_txn = db.begin_write().expect("Failed to begin write");
    {
        let mut table = write_txn.open_table(VERTICES_TABLE).expect("Failed to open table");
        let mut key = session_id.as_bytes().to_vec();
        key.push(b':');
        key.extend_from_slice(corrupt_id.as_bytes());
        table.insert(key.as_slice(), b"invalid cbor bytes".as_slice()).expect("Failed to insert");
    }
    write_txn.commit().expect("Failed to commit");
    drop(db);

    let store = RedbDagStore::open(&db_path).expect("Failed to reopen store");
    let results = store
        .get_vertices(session_id, &[valid_id, corrupt_id])
        .await
        .expect("Failed to get vertices");
    assert_eq!(results.len(), 2);
    assert!(results[0].is_some());
    assert!(results[1].is_none());
}

#[test]
fn test_open_fails_when_path_is_directory() {
    let dir = TempDir::new().expect("Failed to create temp dir");
    let result = RedbDagStore::open(dir.path());
    assert!(result.is_err());
    let err_msg = result.unwrap_err().to_string();
    assert!(
        err_msg.contains("Failed to open") || err_msg.contains("database"),
        "Expected storage/database error, got: {err_msg}"
    );
}

// === Edge case tests ===

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_get_vertices_empty_slice() {
    let (store, _dir) = create_test_store();
    let session_id = SessionId::now();
    let results = store.get_vertices(session_id, &[]).await.expect("Failed to get vertices");
    assert!(results.is_empty());
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_get_children_empty_for_nonexistent_parent() {
    let (store, _dir) = create_test_store();
    let session_id = SessionId::now();
    let parent_id = VertexId::from_bytes(b"parent with no children stored");
    let children = store.get_children(session_id, parent_id).await.expect("Failed to get children");
    assert!(children.is_empty());
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_get_children_empty_for_parent_with_no_children() {
    let (store, _dir) = create_test_store();
    let session_id = SessionId::now();
    let genesis = VertexBuilder::new(EventType::SessionStart).build();
    let mut genesis_clone = genesis.clone();
    let genesis_id = genesis_clone.id().expect("Failed to compute ID");
    store.put_vertex(session_id, genesis).await.expect("Failed to put genesis");
    let child = VertexBuilder::new(EventType::DataCreate {
        schema: None,
    })
    .with_parent(genesis_id)
    .build();
    let mut child_clone = child.clone();
    let child_id = child_clone.id().expect("Failed to compute child ID");
    store.put_vertex(session_id, child).await.expect("Failed to put child");
    let leaf_parent_id = child_id;
    let children =
        store.get_children(session_id, leaf_parent_id).await.expect("Failed to get children");
    assert!(children.is_empty());
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_update_frontier_empty_consumed_parents() {
    let (store, _dir) = create_test_store();
    let session_id = SessionId::now();
    let new_vertex = VertexId::from_bytes(b"new frontier vertex id 32 bytes!!");
    store.update_frontier(session_id, new_vertex, &[]).await.expect("Failed to update frontier");
    let frontier = store.get_frontier(session_id).await.expect("Failed to get frontier");
    assert!(frontier.contains(&new_vertex));
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_update_frontier_on_empty_session() {
    let (store, _dir) = create_test_store();
    let session_id = SessionId::now();
    let new_vertex = VertexId::from_bytes(b"new frontier vertex id 32 bytes!!");
    store.update_frontier(session_id, new_vertex, &[]).await.expect("Failed to update frontier");
    let frontier = store.get_frontier(session_id).await.expect("Failed to get frontier");
    assert_eq!(frontier.len(), 1);
    assert!(frontier.contains(&new_vertex));
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_get_frontier_empty() {
    let (store, _dir) = create_test_store();
    let session_id = SessionId::now();
    let frontier = store.get_frontier(session_id).await.expect("Failed to get frontier");
    assert!(frontier.is_empty());
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_get_genesis_empty() {
    let (store, _dir) = create_test_store();
    let session_id = SessionId::now();
    let genesis = store.get_genesis(session_id).await.expect("Failed to get genesis");
    assert!(genesis.is_empty());
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_path() {
    let (store, dir) = create_test_store();
    let db_path = dir.path().join("db.redb");
    assert_eq!(store.path(), db_path);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_debug_impl() {
    let (store, _dir) = create_test_store();
    let debug_str = format!("{store:?}");
    assert!(debug_str.contains("RedbDagStore"));
    assert!(debug_str.contains("path"));
    assert!(debug_str.contains("read_ops"));
    assert!(debug_str.contains("write_ops"));
}

#[test]
fn test_open_creates_parent_dirs() {
    let dir = TempDir::new().expect("Failed to create temp dir");
    let nested_path = dir.path().join("a/b/c/db.redb");
    let store = RedbDagStore::open(&nested_path).expect("Failed to open store");
    assert!(nested_path.parent().unwrap().exists());
    assert!(store.path().parent().unwrap().exists());
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_vertex_with_parents_children_index() {
    let (store, _dir) = create_test_store();
    let session_id = SessionId::now();
    let genesis = VertexBuilder::new(EventType::SessionStart).build();
    let mut genesis_clone = genesis.clone();
    let genesis_id = genesis_clone.id().expect("Failed to compute genesis ID");
    store.put_vertex(session_id, genesis).await.expect("Failed to put genesis");
    let child1 = VertexBuilder::new(EventType::DataCreate {
        schema: None,
    })
    .with_parent(genesis_id)
    .build();
    let child2 = VertexBuilder::new(EventType::DataCreate {
        schema: None,
    })
    .with_parent(genesis_id)
    .build();
    store.put_vertex(session_id, child1).await.expect("Failed to put child1");
    store.put_vertex(session_id, child2).await.expect("Failed to put child2");
    let children =
        store.get_children(session_id, genesis_id).await.expect("Failed to get children");
    assert_eq!(children.len(), 2);
}

// === Stats/metrics tests ===

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

// === Batch operations ===

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_get_vertices_batch_all_missing() {
    let (store, _dir) = create_test_store();
    let session_id = SessionId::now();
    let ids = [VertexId::from_bytes(b"missing1"), VertexId::from_bytes(b"missing2")];
    let results = store.get_vertices(session_id, &ids).await.expect("Failed to get vertices");
    assert_eq!(results.len(), 2);
    assert!(results[0].is_none());
    assert!(results[1].is_none());
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_get_vertices_batch_single_element() {
    let (store, _dir) = create_test_store();
    let session_id = SessionId::now();
    let vertex = VertexBuilder::new(EventType::SessionStart).build();
    let mut vertex_clone = vertex.clone();
    let vertex_id = vertex_clone.id().expect("Failed to compute ID");
    store.put_vertex(session_id, vertex).await.expect("Failed to put");
    let results =
        store.get_vertices(session_id, &[vertex_id]).await.expect("Failed to get vertices");
    assert_eq!(results.len(), 1);
    assert!(results[0].is_some());
}

// === Advanced session/delete tests ===

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_delete_nonexistent_session() {
    let (store, _dir) = create_test_store();
    let session_id = SessionId::now();
    store.delete_session(session_id).await.expect("Failed to delete nonexistent session");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_delete_session_isolates_sessions() {
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
    store.delete_session(session1).await.expect("Failed to delete session1");
    assert!(store.get_vertex(session1, v1_id).await.expect("Failed to get").is_none());
    assert!(store.get_vertex(session2, v2_id).await.expect("Failed to get").is_some());
    assert_eq!(store.count_vertices(session1).await.expect("Failed to count"), 0);
    assert_eq!(store.count_vertices(session2).await.expect("Failed to count"), 1);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_delete_session_with_children_and_frontiers() {
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
    store.put_vertex(session_id, child).await.expect("Failed to put child");
    store.delete_session(session_id).await.expect("Failed to delete");
    assert!(store.get_genesis(session_id).await.expect("Failed to get genesis").is_empty());
    assert!(store.get_frontier(session_id).await.expect("Failed to get frontier").is_empty());
    assert_eq!(store.count_vertices(session_id).await.expect("Failed to count"), 0);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_delete_session_multi_table_cleanup() {
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
    store.delete_session(session_id).await.expect("Failed to delete");
    assert!(store.get_vertex(session_id, genesis_id).await.expect("Failed to get").is_none());
    assert!(store.get_vertex(session_id, child_id).await.expect("Failed to get").is_none());
    assert!(store.get_genesis(session_id).await.expect("Failed to get genesis").is_empty());
    assert!(store.get_frontier(session_id).await.expect("Failed to get frontier").is_empty());
    assert!(store
        .get_children(session_id, genesis_id)
        .await
        .expect("Failed to get children")
        .is_empty());
    assert_eq!(store.count_vertices(session_id).await.expect("Failed to count"), 0);
}

// === Merge vertex, update frontier variants ===

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_merge_vertex_updates_both_parent_children() {
    let (store, _dir) = create_test_store();
    let session_id = SessionId::now();
    let a = VertexBuilder::new(EventType::SessionStart).build();
    let mut a_clone = a.clone();
    let a_id = a_clone.id().expect("Failed to compute A ID");
    let b = VertexBuilder::new(EventType::SessionStart).build();
    let mut b_clone = b.clone();
    let b_id = b_clone.id().expect("Failed to compute B ID");
    store.put_vertex(session_id, a).await.expect("Failed to put A");
    store.put_vertex(session_id, b).await.expect("Failed to put B");
    let merge = VertexBuilder::new(EventType::DataCreate {
        schema: None,
    })
    .with_parents([a_id, b_id])
    .build();
    let mut merge_clone = merge.clone();
    let merge_id = merge_clone.id().expect("Failed to compute merge ID");
    store.put_vertex(session_id, merge).await.expect("Failed to put merge");
    let children_a = store.get_children(session_id, a_id).await.expect("Failed to get children");
    let children_b = store.get_children(session_id, b_id).await.expect("Failed to get children");
    assert_eq!(children_a.len(), 1);
    assert_eq!(children_b.len(), 1);
    assert!(children_a.contains(&merge_id));
    assert!(children_b.contains(&merge_id));
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_update_frontier_multiple_consumed_parents() {
    let (store, _dir) = create_test_store();
    let session_id = SessionId::now();
    let v1 = VertexId::from_bytes(b"parent1_________________________");
    let v2 = VertexId::from_bytes(b"parent2_________________________");
    store.update_frontier(session_id, v1, &[]).await.expect("Failed to add v1");
    store.update_frontier(session_id, v2, &[]).await.expect("Failed to add v2");
    let merged = VertexId::from_bytes(b"merged vertex id 32 bytes!!!!!!");
    store.update_frontier(session_id, merged, &[v1, v2]).await.expect("Failed to update frontier");
    let frontier = store.get_frontier(session_id).await.expect("Failed to get frontier");
    assert!(frontier.contains(&merged));
    assert!(!frontier.contains(&v1));
    assert!(!frontier.contains(&v2));
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_update_frontier_with_existing_frontier() {
    let (store, _dir) = create_test_store();
    let session_id = SessionId::now();
    let v1 = VertexBuilder::new(EventType::SessionStart).build();
    let mut v1_clone = v1.clone();
    let v1_id = v1_clone.id().expect("Failed to compute v1 ID");
    store.put_vertex(session_id, v1).await.expect("Failed to put v1");
    let v2 = VertexId::from_bytes(b"manual frontier id 32 bytes!!!!!!");
    store.update_frontier(session_id, v2, &[v1_id]).await.expect("Failed to update frontier");
    let frontier = store.get_frontier(session_id).await.expect("Failed to get frontier");
    assert!(frontier.contains(&v2));
    assert!(!frontier.contains(&v1_id));
}

// === Concurrent write operations ===

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_concurrent_writes_multiple_sessions() {
    let (store, _dir) = create_test_store();
    let store = std::sync::Arc::new(store);
    let mut handles = Vec::new();
    for i in 0..8 {
        let store_clone = store.clone();
        handles.push(tokio::spawn(async move {
            let session_id = SessionId::now();
            for j in 0..5 {
                let vertex = VertexBuilder::new(EventType::DataCreate {
                    schema: Some(format!("session{i}_vertex{j}")),
                })
                .build();
                store_clone.put_vertex(session_id, vertex).await.expect("put");
            }
        }));
    }
    for h in handles {
        h.await.expect("task panicked");
    }
    let stats = store.stats().await;
    assert!(stats.sessions >= 8);
    assert!(stats.vertices >= 40);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_concurrent_writes_same_session() {
    let (store, _dir) = create_test_store();
    let store = std::sync::Arc::new(store);
    let session_id = SessionId::now();
    let genesis = VertexBuilder::new(EventType::SessionStart).build();
    let mut genesis_clone = genesis.clone();
    let genesis_id = genesis_clone.id().expect("Failed to compute genesis ID");
    store.put_vertex(session_id, genesis).await.expect("Failed to put genesis");
    let mut handles = Vec::new();
    for i in 0..10 {
        let store_clone = store.clone();
        let sid = session_id;
        let gid = genesis_id;
        handles.push(tokio::spawn(async move {
            let child = VertexBuilder::new(EventType::DataCreate {
                schema: Some(format!("concurrent_child_{i}")),
            })
            .with_parent(gid)
            .build();
            store_clone.put_vertex(sid, child).await.expect("put");
        }));
    }
    for h in handles {
        h.await.expect("task panicked");
    }
    let children =
        store.get_children(session_id, genesis_id).await.expect("Failed to get children");
    assert_eq!(children.len(), 10);
}

// === Database recovery paths ===

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_reopen_after_incomplete_write() {
    let (store, dir) = create_test_store();
    let session_id = SessionId::now();
    let vertex = VertexBuilder::new(EventType::SessionStart).build();
    let mut vertex_clone = vertex.clone();
    let vertex_id = vertex_clone.id().expect("Failed to compute ID");
    store.put_vertex(session_id, vertex).await.expect("Failed to put");
    drop(store);
    let db_path = dir.path().join("db.redb");
    {
        let db = redb::Database::create(&db_path).expect("Failed to create db");
        let write_txn = db.begin_write().expect("Failed to begin write");
        {
            let mut table = write_txn.open_table(VERTICES_TABLE).expect("Failed to open table");
            let mut key = session_id.as_bytes().to_vec();
            key.push(b':');
            key.extend_from_slice(vertex_id.as_bytes());
            table.insert(key.as_slice(), b"incomplete".as_slice()).expect("insert");
        }
        drop(write_txn);
    }
    let store = RedbDagStore::open(&db_path).expect("Failed to reopen store");
    let got = store.get_vertex(session_id, vertex_id).await.expect("Failed to get");
    assert!(got.is_some(), "Original vertex should persist after uncommitted write rollback");
}

// === Stats/metrics accuracy ===

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

// === Table operations / count / exists edge cases ===

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_count_vertices_empty_range() {
    let (store, _dir) = create_test_store();
    let session_id = SessionId::now();
    let count = store.count_vertices(session_id).await.expect("Failed to count");
    assert_eq!(count, 0);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_exists_nonexistent_vertex() {
    let (store, _dir) = create_test_store();
    let session_id = SessionId::now();
    let vertex_id = VertexId::from_bytes(b"nonexistent vertex id 32 bytes!!!!!!");
    let exists = store.exists(session_id, vertex_id).await.expect("Failed to check exists");
    assert!(!exists);
}

// === Additional coverage ===

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_put_vertex_with_parent_not_in_db() {
    let (store, _dir) = create_test_store();
    let session_id = SessionId::now();
    let phantom_parent = VertexId::from_bytes(b"phantom parent id 32 bytes!!!!!!");
    let child = VertexBuilder::new(EventType::DataCreate {
        schema: None,
    })
    .with_parent(phantom_parent)
    .build();
    let mut child_clone = child.clone();
    let child_id = child_clone.id().expect("Failed to compute child ID");
    store.put_vertex(session_id, child).await.expect("Failed to put");
    let children =
        store.get_children(session_id, phantom_parent).await.expect("Failed to get children");
    assert!(children.contains(&child_id));
    let frontier = store.get_frontier(session_id).await.expect("Failed to get frontier");
    assert!(frontier.contains(&child_id));
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_exists_after_delete_session() {
    let (store, _dir) = create_test_store();
    let session_id = SessionId::now();
    let vertex = VertexBuilder::new(EventType::SessionStart).build();
    let mut vertex_clone = vertex.clone();
    let vertex_id = vertex_clone.id().expect("Failed to compute ID");
    store.put_vertex(session_id, vertex).await.expect("Failed to put");
    assert!(store.exists(session_id, vertex_id).await.expect("Failed to exists"));
    store.delete_session(session_id).await.expect("Failed to delete");
    assert!(!store.exists(session_id, vertex_id).await.expect("Failed to exists"));
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_serialize_vertex_set_with_many_children() {
    let (store, _dir) = create_test_store();
    let session_id = SessionId::now();
    let parent = VertexBuilder::new(EventType::SessionStart).build();
    let mut parent_clone = parent.clone();
    let parent_id = parent_clone.id().expect("Failed to compute parent ID");
    store.put_vertex(session_id, parent).await.expect("Failed to put parent");
    for i in 0..50 {
        let child = VertexBuilder::new(EventType::DataCreate {
            schema: Some(format!("schema{i}")),
        })
        .with_parent(parent_id)
        .build();
        store.put_vertex(session_id, child).await.expect("Failed to put child");
    }
    let children = store.get_children(session_id, parent_id).await.expect("Failed to get children");
    assert_eq!(children.len(), 50);
}

// === Clone, multiple calls, Display/Debug ===

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_clone_store_works_independently() {
    let (store, _dir) = create_test_store();
    let session_id = SessionId::now();
    let vertex = VertexBuilder::new(EventType::SessionStart).build();
    let mut vertex_clone = vertex.clone();
    let vertex_id = vertex_clone.id().expect("Failed to compute ID");
    store.put_vertex(session_id, vertex).await.expect("Failed to put");
    let store2 = store.clone();
    let got1 = store.get_vertex(session_id, vertex_id).await.expect("Failed to get");
    let got2 = store2.get_vertex(session_id, vertex_id).await.expect("Failed to get");
    assert!(got1.is_some());
    assert!(got2.is_some());
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

// === Additional coverage: parse_vertex_set empty, stats error paths, concurrent ops ===

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_parse_vertex_set_empty_data() {
    let (store, _dir) = create_test_store();
    let session_id = SessionId::now();
    let frontier = store.get_frontier(session_id).await.expect("Failed to get frontier");
    assert!(frontier.is_empty(), "parse_vertex_set should return empty set for empty session");
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
async fn test_concurrent_update_frontier() {
    let (store, _dir) = create_test_store();
    let store = std::sync::Arc::new(store);
    let session_id = SessionId::now();
    let v1 = VertexId::from_bytes(b"concurrent_frontier_1__________");
    store.update_frontier(session_id, v1, &[]).await.expect("Failed to add v1");
    let mut handles = Vec::new();
    for i in 0u32..5 {
        let store_clone = store.clone();
        let sid = session_id;
        let new_id = VertexId::from_bytes(&{
            let mut b = [0u8; 32];
            b[..4].copy_from_slice(&i.to_le_bytes());
            b
        });
        handles.push(tokio::spawn(async move {
            store_clone.update_frontier(sid, new_id, &[v1]).await.expect("update");
        }));
    }
    for h in handles {
        h.await.expect("task panicked");
    }
    let frontier = store.get_frontier(session_id).await.expect("Failed to get frontier");
    assert!(!frontier.is_empty());
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_very_large_vertex_set_serialization() {
    let (store, _dir) = create_test_store();
    let session_id = SessionId::now();
    let parent = VertexBuilder::new(EventType::SessionStart).build();
    let mut parent_clone = parent.clone();
    let parent_id = parent_clone.id().expect("Failed to compute parent ID");
    store.put_vertex(session_id, parent).await.expect("Failed to put parent");
    for i in 0..150 {
        let child = VertexBuilder::new(EventType::DataCreate {
            schema: Some(format!("large_set_{i}")),
        })
        .with_parent(parent_id)
        .build();
        store.put_vertex(session_id, child).await.expect("Failed to put child");
    }
    let children = store.get_children(session_id, parent_id).await.expect("Failed to get children");
    assert_eq!(children.len(), 150);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_delete_session_empty_tables_no_panic() {
    let (store, _dir) = create_test_store();
    let session_id = SessionId::now();
    store.delete_session(session_id).await.expect("Failed to delete empty session");
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

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_put_vertex_with_multiple_parents_children_index() {
    let (store, _dir) = create_test_store();
    let session_id = SessionId::now();
    let a = VertexBuilder::new(EventType::SessionStart).build();
    let mut a_clone = a.clone();
    let a_id = a_clone.id().expect("Failed to compute A ID");
    let b = VertexBuilder::new(EventType::SessionStart).build();
    let mut b_clone = b.clone();
    let b_id = b_clone.id().expect("Failed to compute B ID");
    let c = VertexBuilder::new(EventType::SessionStart).build();
    let mut c_clone = c.clone();
    let c_id = c_clone.id().expect("Failed to compute C ID");
    store.put_vertex(session_id, a).await.expect("Failed to put A");
    store.put_vertex(session_id, b).await.expect("Failed to put B");
    store.put_vertex(session_id, c).await.expect("Failed to put C");
    let merge = VertexBuilder::new(EventType::DataCreate {
        schema: None,
    })
    .with_parents([a_id, b_id, c_id])
    .build();
    let mut merge_clone = merge.clone();
    let merge_id = merge_clone.id().expect("Failed to compute merge ID");
    store.put_vertex(session_id, merge).await.expect("Failed to put merge");
    let children_a = store.get_children(session_id, a_id).await.expect("Failed to get children");
    let children_b = store.get_children(session_id, b_id).await.expect("Failed to get children");
    let children_c = store.get_children(session_id, c_id).await.expect("Failed to get children");
    assert_eq!(children_a.len(), 1);
    assert_eq!(children_b.len(), 1);
    assert_eq!(children_c.len(), 1);
    assert!(children_a.contains(&merge_id));
    assert!(children_b.contains(&merge_id));
    assert!(children_c.contains(&merge_id));
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_update_frontier_consumed_parents_not_in_frontier() {
    let (store, _dir) = create_test_store();
    let session_id = SessionId::now();
    let v1 = VertexId::from_bytes(b"frontier_v1____________________");
    let v2 = VertexId::from_bytes(b"frontier_v2____________________");
    store.update_frontier(session_id, v1, &[]).await.expect("Failed to add v1");
    store
        .update_frontier(session_id, v2, &[VertexId::from_bytes(b"nonexistent_______________")])
        .await
        .expect("Failed to update frontier");
    let frontier = store.get_frontier(session_id).await.expect("Failed to get frontier");
    assert!(frontier.contains(&v1));
    assert!(frontier.contains(&v2));
}
