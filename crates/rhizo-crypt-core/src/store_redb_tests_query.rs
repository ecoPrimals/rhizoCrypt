// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2024–2026 ecoPrimals Project
//
//! Query format and serialization tests for `RedbDagStore`.
//! Tests `parse_vertex_set` behavior, raw table format, and vertex serialization edge cases.

use super::*;
use crate::event::EventType;
use crate::vertex::VertexBuilder;
use redb::TableDefinition;
use tempfile::TempDir;

const CHILDREN_TABLE: TableDefinition<&[u8], &[u8]> = TableDefinition::new("children");
const FRONTIERS_TABLE: TableDefinition<&[u8], &[u8]> = TableDefinition::new("frontiers");

fn create_test_store() -> (RedbDagStore, TempDir) {
    let dir = TempDir::new().expect("Failed to create temp dir");
    let db_path = dir.path().join("db.redb");
    let store = RedbDagStore::open(&db_path).expect("Failed to open store");
    (store, dir)
}

// === Serialization / parse_vertex_set edge cases ===

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_parse_vertex_set_empty_via_get_frontier_empty_session() {
    let (store, _dir) = create_test_store();
    let session_id = SessionId::now();
    let frontier = store.get_frontier(session_id).await.expect("Failed to get frontier");
    assert!(frontier.is_empty());
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_parse_vertex_set_empty_via_get_genesis_empty_session() {
    let (store, _dir) = create_test_store();
    let session_id = SessionId::now();
    let genesis = store.get_genesis(session_id).await.expect("Failed to get genesis");
    assert!(genesis.is_empty());
}

// === Vertex serialization edge cases: large vertices, empty metadata, unicode ===

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_vertex_large_schema_serialization() {
    let (store, _dir) = create_test_store();
    let session_id = SessionId::now();
    let large_schema = "x".repeat(16 * 1024);
    let vertex = VertexBuilder::new(EventType::DataCreate {
        schema: Some(large_schema),
    })
    .build();
    let mut vertex_clone = vertex.clone();
    let vertex_id = vertex_clone.id().expect("Failed to compute ID");
    store.put_vertex(session_id, vertex).await.expect("Failed to put large vertex");
    let got = store.get_vertex(session_id, vertex_id).await.expect("Failed to get");
    assert!(got.is_some());
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_vertex_empty_metadata() {
    let (store, _dir) = create_test_store();
    let session_id = SessionId::now();
    let vertex = VertexBuilder::new(EventType::SessionStart).build();
    let mut vertex_clone = vertex.clone();
    let vertex_id = vertex_clone.id().expect("Failed to compute ID");
    store.put_vertex(session_id, vertex).await.expect("Failed to put");
    let got = store.get_vertex(session_id, vertex_id).await.expect("Failed to get");
    assert!(got.is_some());
    assert!(got.unwrap().metadata.is_empty());
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_vertex_unicode_metadata_keys() {
    let (store, _dir) = create_test_store();
    let session_id = SessionId::now();
    let vertex = VertexBuilder::new(EventType::DataCreate {
        schema: Some("schema_日本語_emoji_🎉".to_string()),
    })
    .with_metadata("key_日本語", "value")
    .with_metadata("emoji_🔑", 42i64)
    .build();
    let mut vertex_clone = vertex.clone();
    let vertex_id = vertex_clone.id().expect("Failed to compute ID");
    store.put_vertex(session_id, vertex).await.expect("Failed to put");
    let got = store.get_vertex(session_id, vertex_id).await.expect("Failed to get");
    assert!(got.is_some());
}

// === Table operations / parse_vertex_set raw format ===

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_parse_vertex_set_partial_chunk_via_raw_insert() {
    let (store, dir) = create_test_store();
    let session_id = SessionId::now();
    let parent_id = VertexId::from_bytes(b"parent_for_partial_chunk_32bytes!");
    let child_id = VertexId::from_bytes(b"child_id_for_partial_chunk_32b!!");
    drop(store);
    let db_path = dir.path().join("db.redb");
    let db = redb::Database::create(&db_path).expect("Failed to create db");
    let write_txn = db.begin_write().expect("Failed to begin write");
    {
        let mut children_table = write_txn.open_table(CHILDREN_TABLE).expect("open");
        let mut key = session_id.as_bytes().to_vec();
        key.push(b':');
        key.extend_from_slice(parent_id.as_bytes());
        let mut value = child_id.as_bytes().to_vec();
        value.push(0);
        children_table.insert(key.as_slice(), value.as_slice()).expect("insert");
    }
    write_txn.commit().expect("commit");
    drop(db);
    let store = RedbDagStore::open(&db_path).expect("Failed to reopen store");
    let children = store.get_children(session_id, parent_id).await.expect("Failed to get children");
    assert_eq!(children.len(), 1);
    assert!(children.contains(&child_id));
}

// === parse_vertex_set malformed data edge cases ===

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_parse_vertex_set_truncated_31_bytes_returns_empty() {
    let (store, dir) = create_test_store();
    let session_id = SessionId::now();
    let parent_id = VertexId::from_bytes(b"parent_truncated_31_bytes________");
    drop(store);
    let db_path = dir.path().join("db.redb");
    let db = redb::Database::create(&db_path).expect("Failed to create db");
    let write_txn = db.begin_write().expect("Failed to begin write");
    {
        let mut children_table = write_txn.open_table(CHILDREN_TABLE).expect("open");
        let mut key = session_id.as_bytes().to_vec();
        key.push(b':');
        key.extend_from_slice(parent_id.as_bytes());
        let truncated = [0u8; 31];
        children_table.insert(key.as_slice(), truncated.as_slice()).expect("insert");
    }
    write_txn.commit().expect("commit");
    drop(db);
    let store = RedbDagStore::open(&db_path).expect("Failed to reopen store");
    let children = store.get_children(session_id, parent_id).await.expect("Failed to get children");
    assert!(children.is_empty(), "chunks_exact(32) yields nothing for 31 bytes");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_parse_vertex_set_64_bytes_two_vertices() {
    let (store, dir) = create_test_store();
    let session_id = SessionId::now();
    let v1 = VertexId::from_bytes(b"vertex1_________________________");
    let v2 = VertexId::from_bytes(b"vertex2_________________________");
    drop(store);
    let db_path = dir.path().join("db.redb");
    let db = redb::Database::create(&db_path).expect("Failed to create db");
    let write_txn = db.begin_write().expect("Failed to begin write");
    {
        let mut frontiers_table = write_txn.open_table(FRONTIERS_TABLE).expect("open");
        let key = session_id.as_bytes().to_vec();
        let mut value = v1.as_bytes().to_vec();
        value.extend_from_slice(v2.as_bytes());
        frontiers_table.insert(key.as_slice(), value.as_slice()).expect("insert");
    }
    write_txn.commit().expect("commit");
    drop(db);
    let store = RedbDagStore::open(&db_path).expect("Failed to reopen store");
    let frontier = store.get_frontier(session_id).await.expect("Failed to get frontier");
    assert_eq!(frontier.len(), 2);
    assert!(frontier.contains(&v1));
    assert!(frontier.contains(&v2));
}
