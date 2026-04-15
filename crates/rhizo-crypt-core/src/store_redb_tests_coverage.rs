// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2024–2026 ecoPrimals Project

use super::*;
use crate::event::EventType;
use crate::vertex::VertexBuilder;

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
    let ids: std::collections::HashSet<VertexId> = (0..5u8)
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
    assert_eq!(&key[..], session_id.as_bytes());
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
