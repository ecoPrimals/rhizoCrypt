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
    assert!(
        store
            .get_children(session_id, genesis_id)
            .await
            .expect("Failed to get children")
            .is_empty()
    );
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

// === Additional coverage ===

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_exists_nonexistent_vertex() {
    let (store, _dir) = create_test_store();
    let session_id = SessionId::now();
    let vertex_id = VertexId::from_bytes(b"nonexistent vertex id 32 bytes!!!!!!");
    let exists = store.exists(session_id, vertex_id).await.expect("Failed to check exists");
    assert!(!exists);
}

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
async fn test_parse_vertex_set_empty_data() {
    let (store, _dir) = create_test_store();
    let session_id = SessionId::now();
    let frontier = store.get_frontier(session_id).await.expect("Failed to get frontier");
    assert!(frontier.is_empty(), "parse_vertex_set should return empty set for empty session");
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

// === Additional coverage tests: complex delete_session with children, parse/serialize roundtrips ===

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_delete_session_complex_dag_with_children_and_metadata() {
    let (store, _dir) = create_test_store();
    let session_id = SessionId::now();

    let genesis = VertexBuilder::new(EventType::SessionStart).build();
    let mut genesis_clone = genesis.clone();
    let genesis_id = genesis_clone.id().expect("genesis ID");
    store.put_vertex(session_id, genesis).await.expect("put genesis");

    let a = VertexBuilder::new(EventType::DataCreate {
        schema: None,
    })
    .with_parent(genesis_id)
    .build();
    let mut a_clone = a.clone();
    let a_id = a_clone.id().expect("a ID");
    store.put_vertex(session_id, a).await.expect("put a");

    let b = VertexBuilder::new(EventType::DataCreate {
        schema: None,
    })
    .with_parent(genesis_id)
    .build();
    let mut b_clone = b.clone();
    let b_id = b_clone.id().expect("b ID");
    store.put_vertex(session_id, b).await.expect("put b");

    let merge = VertexBuilder::new(EventType::DataCreate {
        schema: None,
    })
    .with_parents([a_id, b_id])
    .build();
    let mut merge_clone = merge.clone();
    let merge_id = merge_clone.id().expect("merge ID");
    store.put_vertex(session_id, merge).await.expect("put merge");

    assert_eq!(store.count_vertices(session_id).await.expect("count"), 4);
    assert!(!store.get_children(session_id, genesis_id).await.expect("children").is_empty());
    assert!(!store.get_genesis(session_id).await.expect("genesis").is_empty());
    assert!(!store.get_frontier(session_id).await.expect("frontier").is_empty());

    store.delete_session(session_id).await.expect("delete");

    assert_eq!(store.count_vertices(session_id).await.expect("count"), 0);
    assert!(store.get_children(session_id, genesis_id).await.expect("children").is_empty());
    assert!(store.get_genesis(session_id).await.expect("genesis").is_empty());
    assert!(store.get_frontier(session_id).await.expect("frontier").is_empty());
    assert!(store.get_vertex(session_id, merge_id).await.expect("get").is_none());
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_delete_session_twice_is_idempotent() {
    let (store, _dir) = create_test_store();
    let session_id = SessionId::now();
    let vertex = VertexBuilder::new(EventType::SessionStart).build();
    store.put_vertex(session_id, vertex).await.expect("put");

    store.delete_session(session_id).await.expect("first delete");
    store.delete_session(session_id).await.expect("second delete");

    assert_eq!(store.count_vertices(session_id).await.expect("count"), 0);
}

#[test]
fn test_parse_vertex_set_empty_returns_empty() {
    let set = RedbDagStore::parse_vertex_set(&[]);
    assert!(set.is_empty());
}

#[test]
fn test_parse_vertex_set_single_vertex() {
    let id = VertexId::from_bytes(b"a single vertex id in 32 bytes!!");
    let data = id.as_bytes().to_vec();
    let set = RedbDagStore::parse_vertex_set(&data);
    assert_eq!(set.len(), 1);
    assert!(set.contains(&id));
}

#[test]
fn test_serialize_then_parse_roundtrips() {
    let ids: hashbrown::HashSet<VertexId> = (0..5u8)
        .map(|i| {
            let mut buf = [i; 32];
            buf[0] = i;
            VertexId::new(buf)
        })
        .collect();
    let serialized = RedbDagStore::serialize_vertex_set(&ids);
    let parsed = RedbDagStore::parse_vertex_set(&serialized);
    assert_eq!(ids, parsed);
}

#[test]
fn test_parse_vertex_set_ignores_trailing_bytes() {
    let id = VertexId::from_bytes(b"exactly 32 bytes vertex id here!");
    let mut data = id.as_bytes().to_vec();
    data.extend_from_slice(&[0xFF; 15]); // 15 trailing bytes (not a full 32-byte chunk)
    let set = RedbDagStore::parse_vertex_set(&data);
    assert_eq!(set.len(), 1);
    assert!(set.contains(&id));
}

#[test]
fn test_vertex_key_format() {
    let session_id = SessionId::now();
    let vertex_id = VertexId::from_bytes(b"32 bytes of vertex id data here!");
    let key = RedbDagStore::vertex_key(session_id, vertex_id);
    assert_eq!(key.len(), 49); // 16 (session) + 1 (':') + 32 (vertex)
    assert_eq!(key[16], b':');
}

#[test]
fn test_session_key_format() {
    let session_id = SessionId::now();
    let key = RedbDagStore::session_key(session_id);
    assert_eq!(key.len(), 16);
    assert_eq!(key, session_id.as_bytes().to_vec());
}

#[test]
fn test_session_prefix_range_format() {
    let session_id = SessionId::now();
    let (start, end) = RedbDagStore::session_prefix_range(session_id);
    assert_eq!(start.len(), 17); // 16 + ':'
    assert_eq!(end.len(), 17); // 16 + ';'
    assert_eq!(*start.last().unwrap(), b':');
    assert_eq!(*end.last().unwrap(), b';');
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_clone_shares_counters() {
    let (store, _dir) = create_test_store();
    let store2 = store.clone();

    let session_id = SessionId::now();
    let vertex = VertexBuilder::new(EventType::SessionStart).build();
    store.put_vertex(session_id, vertex).await.expect("put");

    let stats1 = store.stats().await;
    let stats2 = store2.stats().await;
    assert_eq!(stats1.write_ops, stats2.write_ops);
    assert_eq!(stats1.read_ops, stats2.read_ops);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_update_frontier_replaces_multiple_parents() {
    let (store, _dir) = create_test_store();
    let session_id = SessionId::now();

    let p1 = VertexId::from_bytes(b"parent_1________________________");
    let p2 = VertexId::from_bytes(b"parent_2________________________");
    let child = VertexId::from_bytes(b"child___________________________");

    store.update_frontier(session_id, p1, &[]).await.expect("add p1");
    store.update_frontier(session_id, p2, &[]).await.expect("add p2");

    let frontier = store.get_frontier(session_id).await.expect("frontier");
    assert!(frontier.contains(&p1));
    assert!(frontier.contains(&p2));

    store.update_frontier(session_id, child, &[p1, p2]).await.expect("merge");

    let frontier = store.get_frontier(session_id).await.expect("frontier after merge");
    assert!(frontier.contains(&child));
    assert!(!frontier.contains(&p1));
    assert!(!frontier.contains(&p2));
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_exists_returns_false_for_wrong_session() {
    let (store, _dir) = create_test_store();
    let session1 = SessionId::now();
    let session2 = SessionId::now();

    let vertex = VertexBuilder::new(EventType::SessionStart).build();
    let mut vertex_clone = vertex.clone();
    let vertex_id = vertex_clone.id().expect("ID");
    store.put_vertex(session1, vertex).await.expect("put");

    assert!(store.exists(session1, vertex_id).await.expect("exists"));
    assert!(!store.exists(session2, vertex_id).await.expect("exists"));
}
