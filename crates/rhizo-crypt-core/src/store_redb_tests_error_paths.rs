// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2024–2026 ecoPrimals Project

use super::*;
use crate::event::EventType;
use crate::store::DagStore;
use crate::vertex::VertexBuilder;
use tempfile::TempDir;

fn create_test_store() -> (RedbDagStore, TempDir) {
    let dir = TempDir::new().expect("temp dir");
    let db_path = dir.path().join("db.redb");
    let store = RedbDagStore::open(&db_path).expect("open");
    (store, dir)
}

#[test]
fn test_open_with_non_database_file() {
    let dir = TempDir::new().expect("temp dir");
    let bad_path = dir.path().join("not-a-db.redb");
    std::fs::write(&bad_path, b"this is not a valid redb database file contents").expect("write");
    let result = RedbDagStore::open(&bad_path);
    assert!(result.is_err());
    let err = result.unwrap_err().to_string();
    assert!(
        err.contains("storage") || err.contains("database") || err.contains("redb"),
        "expected storage error, got: {err}"
    );
}

#[cfg(unix)]
#[test]
fn test_open_with_unwritable_directory() {
    use std::os::unix::fs::PermissionsExt;
    let dir = TempDir::new().expect("temp dir");
    let readonly_dir = dir.path().join("readonly");
    std::fs::create_dir(&readonly_dir).expect("create dir");
    std::fs::set_permissions(&readonly_dir, std::fs::Permissions::from_mode(0o444))
        .expect("set permissions");
    let bad_path = readonly_dir.join("nested").join("db.redb");
    let result = RedbDagStore::open(&bad_path);
    assert!(result.is_err());
    std::fs::set_permissions(&readonly_dir, std::fs::Permissions::from_mode(0o755))
        .expect("restore permissions");
}

#[test]
fn test_parse_vertex_set_empty_data() {
    let result = RedbDagStore::parse_vertex_set(&[]);
    assert!(result.is_empty());
}

#[test]
fn test_parse_vertex_set_exact_one_vertex() {
    let id_bytes = [42u8; crate::constants::VERTEX_ID_BYTES];
    let result = RedbDagStore::parse_vertex_set(&id_bytes);
    assert_eq!(result.len(), 1);
    assert!(result.contains(&VertexId::new(id_bytes)));
}

#[test]
fn test_parse_vertex_set_partial_chunk_ignored() {
    let mut data = vec![0u8; crate::constants::VERTEX_ID_BYTES + 5];
    data[..crate::constants::VERTEX_ID_BYTES].fill(1);
    let result = RedbDagStore::parse_vertex_set(&data);
    assert_eq!(result.len(), 1, "trailing 5 bytes should be ignored by chunks_exact");
}

#[test]
fn test_serialize_vertex_set_empty() {
    let set = std::collections::HashSet::new();
    let data = RedbDagStore::serialize_vertex_set(&set);
    assert!(data.is_empty());
}

#[test]
fn test_serialize_deserialize_round_trip() {
    let mut set = std::collections::HashSet::new();
    set.insert(VertexId::new([1u8; 32]));
    set.insert(VertexId::new([2u8; 32]));
    set.insert(VertexId::new([3u8; 32]));
    let data = RedbDagStore::serialize_vertex_set(&set);
    assert_eq!(data.len(), 3 * crate::constants::VERTEX_ID_BYTES);
    let parsed = RedbDagStore::parse_vertex_set(&data);
    assert_eq!(parsed, set);
}

#[test]
fn test_vertex_key_construction() {
    let session_id = SessionId::now();
    let vertex_id = VertexId::new([0xAB; 32]);
    let key = RedbDagStore::vertex_key(session_id, vertex_id);
    assert_eq!(key.len(), crate::constants::VERTEX_KEY_SIZE);
    assert_eq!(key[crate::constants::SESSION_ID_BYTES], crate::constants::VERTEX_KEY_SEPARATOR);
}

#[test]
fn test_session_key_construction() {
    let session_id = SessionId::now();
    let key = RedbDagStore::session_key(session_id);
    assert_eq!(key.len(), crate::constants::SESSION_ID_BYTES);
}

#[test]
fn test_session_prefix_range() {
    let session_id = SessionId::now();
    let (start, end) = RedbDagStore::session_prefix_range(session_id);
    assert_eq!(start.len(), crate::constants::SESSION_ID_BYTES + 1);
    assert_eq!(end.len(), crate::constants::SESSION_ID_BYTES + 1);
    assert_eq!(*start.last().unwrap(), crate::constants::VERTEX_KEY_SEPARATOR);
    assert_eq!(*end.last().unwrap(), crate::constants::VERTEX_KEY_SEPARATOR + 1);
}

#[test]
fn test_debug_format() {
    let (store, _dir) = create_test_store();
    let debug_str = format!("{store:?}");
    assert!(debug_str.contains("RedbDagStore"));
    assert!(debug_str.contains("path"));
    assert!(debug_str.contains("read_ops"));
    assert!(debug_str.contains("write_ops"));
}

#[test]
fn test_path_accessor() {
    let dir = TempDir::new().expect("temp dir");
    let db_path = dir.path().join("db.redb");
    let store = RedbDagStore::open(&db_path).expect("open");
    assert_eq!(store.path(), db_path.as_path());
}

#[test]
fn test_clone() {
    let (store1, _dir) = create_test_store();
    let store2 = store1.clone();
    assert_eq!(store1.path(), store2.path());
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_delete_empty_session() {
    let (store, _dir) = create_test_store();
    let session_id = SessionId::now();
    store.delete_session(session_id).await.expect("delete empty session should succeed");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_update_frontier_empty_session() {
    let (store, _dir) = create_test_store();
    let session_id = SessionId::now();
    let vertex_id = VertexId::new([99u8; 32]);
    store
        .update_frontier(session_id, vertex_id, &[])
        .await
        .expect("update frontier on empty session should succeed");
    let frontier = store.get_frontier(session_id).await.expect("get frontier");
    assert_eq!(frontier.len(), 1);
    assert!(frontier.contains(&vertex_id));
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_get_children_nonexistent_parent() {
    let (store, _dir) = create_test_store();
    let session_id = SessionId::now();
    let parent_id = VertexId::new([77u8; 32]);
    let children = store.get_children(session_id, parent_id).await.expect("get children");
    assert!(children.is_empty());
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_get_vertices_empty_batch() {
    let (store, _dir) = create_test_store();
    let session_id = SessionId::now();
    let results = store.get_vertices(session_id, &[]).await.expect("get vertices");
    assert!(results.is_empty());
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_read_vertex_set_for_nonexistent_key() {
    let (store, _dir) = create_test_store();
    let result = store
        .read_vertex_set(FRONTIERS, b"nonexistent_key_that_does_not_exist")
        .expect("read vertex set should not error for missing key");
    assert!(result.is_empty());
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_stats_on_fresh_database() {
    let (store, _dir) = create_test_store();
    let stats = store.stats().await;
    assert_eq!(stats.sessions, 0);
    assert_eq!(stats.vertices, 0);
    assert_eq!(stats.read_ops, 0);
    assert_eq!(stats.write_ops, 0);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_put_vertex_with_metadata() {
    let (store, _dir) = create_test_store();
    let session_id = SessionId::now();
    let vertex = VertexBuilder::new(EventType::DataCreate {
        schema: Some("test-schema".to_string()),
    })
    .with_metadata("key1", "value1")
    .with_metadata("key2", 42i64)
    .build();
    let mut vertex_clone = vertex.clone();
    let vertex_id = vertex_clone.id().expect("compute ID");
    store.put_vertex(session_id, vertex).await.expect("put");
    let got = store.get_vertex(session_id, vertex_id).await.expect("get");
    assert!(got.is_some());
    let v = got.unwrap();
    assert_eq!(v.metadata.len(), 2);
}
