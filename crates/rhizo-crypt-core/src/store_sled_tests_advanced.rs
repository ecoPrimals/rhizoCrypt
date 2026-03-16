// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2024–2026 ecoPrimals Project
//
//! Advanced tests for SledDagStore: edge cases, parse_vertex_set, corrupt data, open failures.

use super::*;
use crate::event::EventType;
use crate::store::StorageHealth;
use crate::vertex::VertexBuilder;
use tempfile::TempDir;

fn create_test_store() -> (SledDagStore, TempDir) {
    let dir = TempDir::new().expect("Failed to create temp dir");
    let store = SledDagStore::open(dir.path()).expect("Failed to open store");
    (store, dir)
}

// === Open failure tests ===

#[test]
fn test_open_fails_when_parent_is_file() {
    let dir = TempDir::new().expect("Failed to create temp dir");
    let file_path = dir.path().join("blocker");
    std::fs::File::create(&file_path).expect("Failed to create file");
    let db_path = file_path.join("db");
    let result = SledDagStore::open(&db_path);
    assert!(result.is_err());
    let err_msg = result.unwrap_err().to_string();
    assert!(
        err_msg.contains("directory") || err_msg.contains("Failed to create"),
        "Expected directory/create error, got: {err_msg}"
    );
}

// === Corrupt data tests ===

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_get_vertex_fails_on_corrupt_data() {
    let (store, _dir) = create_test_store();
    let session_id = SessionId::now();
    let vertex_id = VertexId::from_bytes(b"corrupt vertex id 32 bytes!!!!!!");

    let mut key = session_id.as_bytes().to_vec();
    key.push(b':');
    key.extend_from_slice(vertex_id.as_bytes());
    store.vertices.insert(key.as_slice(), b"not valid cbor").expect("Failed to insert");

    let result = store.get_vertex(session_id, vertex_id).await;
    assert!(result.is_err());
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_get_vertices_returns_none_for_invalid_cbor() {
    let (store, _dir) = create_test_store();
    let session_id = SessionId::now();
    let valid_vertex = VertexBuilder::new(EventType::SessionStart).build();
    let mut valid_clone = valid_vertex.clone();
    let valid_id = valid_clone.id().expect("Failed to compute ID");
    store.put_vertex(session_id, valid_vertex).await.expect("Failed to put");

    let corrupt_id = VertexId::from_bytes(b"corrupt vertex id 32 bytes!!!!!!");
    let mut key = session_id.as_bytes().to_vec();
    key.push(b':');
    key.extend_from_slice(corrupt_id.as_bytes());
    store.vertices.insert(key.as_slice(), b"invalid cbor bytes").expect("Failed to insert");

    let results = store
        .get_vertices(session_id, &[valid_id, corrupt_id])
        .await
        .expect("Failed to get vertices");
    assert_eq!(results.len(), 2);
    assert!(results[0].is_some());
    assert!(results[1].is_none());
}

// === get_vertices batch edge cases ===

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_get_vertices_single_id() {
    let (store, _dir) = create_test_store();
    let session_id = SessionId::now();
    let vertex = VertexBuilder::new(EventType::SessionStart).build();
    let mut vc = vertex.clone();
    let vid = vc.id().unwrap();
    store.put_vertex(session_id, vertex).await.unwrap();
    let results = store.get_vertices(session_id, &[vid]).await.unwrap();
    assert_eq!(results.len(), 1);
    assert!(results[0].is_some());
    assert_eq!(results[0].clone().unwrap().id().unwrap(), vid);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_get_vertices_all_missing() {
    let (store, _dir) = create_test_store();
    let session_id = SessionId::now();
    let ids = [VertexId::from_bytes(&[1u8; 32]), VertexId::from_bytes(&[2u8; 32])];
    let results = store.get_vertices(session_id, &ids).await.unwrap();
    assert_eq!(results.len(), 2);
    assert!(results[0].is_none());
    assert!(results[1].is_none());
}

// === exists edge cases ===

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_exists_false_for_empty_session() {
    let (store, _dir) = create_test_store();
    let session_id = SessionId::now();
    let fake_id = VertexId::from_bytes(&[0u8; 32]);
    assert!(!store.exists(session_id, fake_id).await.unwrap());
}

// === count_vertices edge cases ===

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_count_vertices_after_put() {
    let (store, _dir) = create_test_store();
    let session_id = SessionId::now();
    for i in 0..7 {
        let vertex = VertexBuilder::new(EventType::DataCreate {
            schema: Some(format!("schema{i}")),
        })
        .build();
        store.put_vertex(session_id, vertex).await.unwrap();
    }
    assert_eq!(store.count_vertices(session_id).await.unwrap(), 7);
}

// === delete_session full cleanup ===

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_delete_session_cleans_frontiers_genesis_metadata() {
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
    store.put_vertex(session_id, v2).await.unwrap();

    let genesis = store.get_genesis(session_id).await.unwrap();
    assert_eq!(genesis.len(), 1);
    let frontier = store.get_frontier(session_id).await.unwrap();
    assert_eq!(frontier.len(), 1);

    store.delete_session(session_id).await.unwrap();

    let genesis_after = store.get_genesis(session_id).await.unwrap();
    assert!(genesis_after.is_empty());
    let frontier_after = store.get_frontier(session_id).await.unwrap();
    assert!(frontier_after.is_empty());
    assert_eq!(store.count_vertices(session_id).await.unwrap(), 0);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_delete_session_with_children_index() {
    let (store, _dir) = create_test_store();
    let session_id = SessionId::now();

    let parent = VertexBuilder::new(EventType::SessionStart).build();
    let mut pc = parent.clone();
    let parent_id = pc.id().unwrap();
    store.put_vertex(session_id, parent).await.unwrap();

    let child = VertexBuilder::new(EventType::DataCreate {
        schema: None,
    })
    .with_parent(parent_id)
    .build();
    store.put_vertex(session_id, child).await.unwrap();

    let children = store.get_children(session_id, parent_id).await.unwrap();
    assert_eq!(children.len(), 1);

    store.delete_session(session_id).await.unwrap();

    let children_after = store.get_children(session_id, parent_id).await.unwrap();
    assert!(children_after.is_empty());
}

// === update_frontier scenarios ===

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_update_frontier_multiple_consumed_parents() {
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

    let new_id = VertexId::from_bytes(&[0u8; 32]);
    store.update_frontier(session_id, new_id, &[v1_id, v2_id]).await.unwrap();

    let frontier = store.get_frontier(session_id).await.unwrap();
    assert!(frontier.contains(&new_id));
    assert!(!frontier.contains(&v1_id));
    assert!(!frontier.contains(&v2_id));
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_update_frontier_adds_to_existing() {
    let (store, _dir) = create_test_store();
    let session_id = SessionId::now();

    let v1 = VertexBuilder::new(EventType::SessionStart).build();
    let mut v1c = v1.clone();
    let v1_id = v1c.id().unwrap();
    store.put_vertex(session_id, v1).await.unwrap();

    let v2_id = VertexId::from_bytes(&[2u8; 32]);
    store.update_frontier(session_id, v2_id, &[]).await.unwrap();

    let frontier = store.get_frontier(session_id).await.unwrap();
    assert!(frontier.contains(&v1_id));
    assert!(frontier.contains(&v2_id));
}

// === stats method ===

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_stats_session_count() {
    let (store, _dir) = create_test_store();
    let stats_empty = store.stats().await;
    assert_eq!(stats_empty.sessions, 0);

    let s1 = SessionId::now();
    let s2 = SessionId::now();
    store.put_vertex(s1, VertexBuilder::new(EventType::SessionStart).build()).await.unwrap();
    store.put_vertex(s2, VertexBuilder::new(EventType::SessionStart).build()).await.unwrap();

    let stats = store.stats().await;
    assert_eq!(stats.sessions, 2);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_stats_vertex_count() {
    let (store, _dir) = create_test_store();
    let session_id = SessionId::now();
    for _ in 0..12 {
        store
            .put_vertex(session_id, VertexBuilder::new(EventType::SessionStart).build())
            .await
            .unwrap();
    }
    let stats = store.stats().await;
    assert_eq!(stats.vertices, 12);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_stats_disk_usage() {
    let (store, _dir) = create_test_store();
    let stats_empty = store.stats().await;
    let session_id = SessionId::now();
    for _ in 0..15 {
        store
            .put_vertex(session_id, VertexBuilder::new(EventType::SessionStart).build())
            .await
            .unwrap();
    }
    store.flush().await.unwrap();
    let stats = store.stats().await;
    assert!(stats.bytes_used >= stats_empty.bytes_used, "bytes_used should increase with data");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_stats_all_fields_populated() {
    let (store, _dir) = create_test_store();
    let session_id = SessionId::now();
    store
        .put_vertex(session_id, VertexBuilder::new(EventType::SessionStart).build())
        .await
        .unwrap();
    store.get_vertex(session_id, VertexId::from_bytes(&[0u8; 32])).await.ok();

    let stats = store.stats().await;
    assert!(stats.sessions >= 1);
    assert!(stats.vertices >= 1);
    assert!(stats.read_ops >= 1);
    assert!(stats.write_ops >= 1);
}

// === health method ===

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_health_returns_healthy() {
    let (store, _dir) = create_test_store();
    let health = store.health().await;
    assert!(matches!(health, StorageHealth::Healthy));
}

// === Debug impl ===

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_debug_includes_path() {
    let (store, dir) = create_test_store();
    let debug_str = format!("{store:?}");
    assert!(debug_str.contains("SledDagStore"));
    assert!(debug_str.contains("path"));
    let path_str = dir.path().to_string_lossy();
    assert!(debug_str.contains(path_str.as_ref()));
}

// === path() method ===

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_path_returns_db_directory() {
    let (store, dir) = create_test_store();
    assert_eq!(store.path(), dir.path());
}

// === parse_vertex_set: empty, partial, truncated, 64 bytes ===

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_parse_vertex_set_empty_via_get_frontier() {
    let (store, _dir) = create_test_store();
    let session_id = SessionId::now();
    let frontier = store.get_frontier(session_id).await.unwrap();
    assert!(frontier.is_empty());
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_parse_vertex_set_empty_via_get_genesis() {
    let (store, _dir) = create_test_store();
    let session_id = SessionId::now();
    let genesis = store.get_genesis(session_id).await.unwrap();
    assert!(genesis.is_empty());
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_parse_vertex_set_truncated_31_bytes() {
    let (store, _dir) = create_test_store();
    let session_id = SessionId::now();
    let parent_id = VertexId::from_bytes(b"parent_truncated_31_bytes________");

    let mut key = session_id.as_bytes().to_vec();
    key.push(b':');
    key.extend_from_slice(parent_id.as_bytes());
    let truncated = [0u8; 31];
    store.children.insert(key.as_slice(), truncated.as_slice()).expect("insert");

    let result = store.get_children(session_id, parent_id).await.unwrap();
    assert!(result.is_empty(), "chunks_exact(32) yields nothing for 31 bytes");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_parse_vertex_set_64_bytes_two_vertices() {
    let (store, _dir) = create_test_store();
    let session_id = SessionId::now();
    let v1 = VertexId::from_bytes(b"vertex1_________________________");
    let v2 = VertexId::from_bytes(b"vertex2_________________________");

    let key = session_id.as_bytes().to_vec();
    let mut value = v1.as_bytes().to_vec();
    value.extend_from_slice(v2.as_bytes());
    store.frontiers.insert(key.as_slice(), value.as_slice()).expect("insert");

    let frontier = store.get_frontier(session_id).await.unwrap();
    assert_eq!(frontier.len(), 2);
    assert!(frontier.contains(&v1));
    assert!(frontier.contains(&v2));
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_parse_vertex_set_partial_chunk_via_children() {
    let (store, _dir) = create_test_store();
    let session_id = SessionId::now();
    let parent_id = VertexId::from_bytes(b"parent_for_partial_chunk_32bytes!");
    let child_id = VertexId::from_bytes(b"child_id_for_partial_chunk_32b!!");

    let mut key = session_id.as_bytes().to_vec();
    key.push(b':');
    key.extend_from_slice(parent_id.as_bytes());
    let value = child_id.as_bytes().to_vec();
    store.children.insert(key.as_slice(), value.as_slice()).expect("insert");

    let result = store.get_children(session_id, parent_id).await.unwrap();
    assert_eq!(result.len(), 1);
    assert!(result.contains(&child_id));
}

// === flush ===

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_flush_succeeds() {
    let (store, _dir) = create_test_store();
    store.flush().await.unwrap();
}

// === export ===

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_export_empty_store() {
    let (store, _dir) = create_test_store();
    let export_data = store.export();
    assert!(!export_data.is_empty(), "export returns tree metadata even when empty");
}

// === Additional coverage tests ===

#[test]
fn test_open_creates_nested_dirs() {
    let dir = TempDir::new().expect("temp dir");
    let nested_path = dir.path().join("a/b/c/sled_db");
    let store = SledDagStore::open(&nested_path).expect("open");
    assert!(store.path().exists());
}

#[test]
fn test_parse_vertex_set_empty() {
    let set = SledDagStore::parse_vertex_set(&[]);
    assert!(set.is_empty());
}

#[test]
fn test_serialize_then_parse_roundtrips() {
    let ids: hashbrown::HashSet<VertexId> = (0..5u8)
        .map(|i| {
            let buf = [i; 32];
            VertexId::new(buf)
        })
        .collect();
    let serialized = SledDagStore::serialize_vertex_set(&ids);
    let parsed = SledDagStore::parse_vertex_set(&serialized);
    assert_eq!(ids, parsed);
}

#[test]
fn test_parse_vertex_set_trailing_bytes_ignored() {
    let id = VertexId::from_bytes(b"exactly 32 bytes vertex id here!");
    let mut data = id.as_bytes().to_vec();
    data.extend_from_slice(&[0xFF; 10]);
    let set = SledDagStore::parse_vertex_set(&data);
    assert_eq!(set.len(), 1);
    assert!(set.contains(&id));
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_delete_session_complex_dag() {
    let (store, _dir) = create_test_store();
    let session_id = SessionId::now();

    let genesis = VertexBuilder::new(EventType::SessionStart).build();
    let mut gc = genesis.clone();
    let gid = gc.id().expect("gid");
    store.put_vertex(session_id, genesis).await.expect("put genesis");

    let a = VertexBuilder::new(EventType::DataCreate {
        schema: None,
    })
    .with_parent(gid)
    .build();
    store.put_vertex(session_id, a).await.expect("put a");

    let b = VertexBuilder::new(EventType::DataCreate {
        schema: None,
    })
    .with_parent(gid)
    .build();
    store.put_vertex(session_id, b).await.expect("put b");

    assert_eq!(store.count_vertices(session_id).await.expect("count"), 3);
    store.delete_session(session_id).await.expect("delete");
    assert_eq!(store.count_vertices(session_id).await.expect("count"), 0);
    assert!(store.get_genesis(session_id).await.expect("genesis").is_empty());
    assert!(store.get_frontier(session_id).await.expect("frontier").is_empty());
    assert!(store.get_children(session_id, gid).await.expect("children").is_empty());
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_delete_session_twice_idempotent() {
    let (store, _dir) = create_test_store();
    let session_id = SessionId::now();
    let v = VertexBuilder::new(EventType::SessionStart).build();
    store.put_vertex(session_id, v).await.expect("put");
    store.delete_session(session_id).await.expect("first delete");
    store.delete_session(session_id).await.expect("second delete");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_clone_shares_state() {
    let (store, _dir) = create_test_store();
    let store2 = store.clone();
    let session_id = SessionId::now();
    let v = VertexBuilder::new(EventType::SessionStart).build();
    let mut vc = v.clone();
    let vid = vc.id().expect("id");
    store.put_vertex(session_id, v).await.expect("put");

    let got = store2.get_vertex(session_id, vid).await.expect("get");
    assert!(got.is_some());
    assert_eq!(store.stats().await.write_ops, store2.stats().await.write_ops);
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
    store.update_frontier(session_id, child, &[p1, p2]).await.expect("merge");

    let frontier = store.get_frontier(session_id).await.expect("frontier");
    assert!(frontier.contains(&child));
    assert!(!frontier.contains(&p1));
    assert!(!frontier.contains(&p2));
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_exists_wrong_session() {
    let (store, _dir) = create_test_store();
    let s1 = SessionId::now();
    let s2 = SessionId::now();
    let v = VertexBuilder::new(EventType::SessionStart).build();
    let mut vc = v.clone();
    let vid = vc.id().expect("id");
    store.put_vertex(s1, v).await.expect("put");

    assert!(store.exists(s1, vid).await.expect("exists"));
    assert!(!store.exists(s2, vid).await.expect("exists"));
}
