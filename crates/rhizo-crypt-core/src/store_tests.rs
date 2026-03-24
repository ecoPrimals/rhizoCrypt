// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2024–2026 ecoPrimals Project

//! Tests for DAG storage traits and in-memory implementations.

use super::*;
use crate::event::EventType;
use crate::vertex::VertexBuilder;

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_in_memory_dag_store() {
    let store = InMemoryDagStore::new();
    let session_id = SessionId::now();

    let vertex = VertexBuilder::new(EventType::SessionStart).build();
    let mut vertex_clone = vertex.clone();
    let vertex_id = vertex_clone.id().unwrap();

    store.put_vertex(session_id, vertex).await.unwrap();

    assert!(store.exists(session_id, vertex_id).await.unwrap());

    let retrieved = store.get_vertex(session_id, vertex_id).await.unwrap();
    assert!(retrieved.is_some());

    assert_eq!(store.count_vertices(session_id).await.unwrap(), 1);
    assert_eq!(store.session_count().await, 1);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_dag_store_genesis_and_frontier() {
    let store = InMemoryDagStore::new();
    let session_id = SessionId::now();

    let v1 = VertexBuilder::new(EventType::SessionStart).build();
    let mut v1_clone = v1.clone();
    let v1_id = v1_clone.id().unwrap();
    store.put_vertex(session_id, v1).await.unwrap();

    let genesis = store.get_genesis(session_id).await.unwrap();
    assert_eq!(genesis.len(), 1);
    assert!(genesis.contains(&v1_id));

    let frontier = store.get_frontier(session_id).await.unwrap();
    assert_eq!(frontier.len(), 1);
    assert!(frontier.contains(&v1_id));

    let v2 = VertexBuilder::new(EventType::DataCreate {
        schema: None,
    })
    .with_parent(v1_id)
    .build();
    let mut v2_clone = v2.clone();
    let v2_id = v2_clone.id().unwrap();
    store.put_vertex(session_id, v2).await.unwrap();

    let frontier = store.get_frontier(session_id).await.unwrap();
    assert_eq!(frontier.len(), 1);
    assert!(frontier.contains(&v2_id));
    assert!(!frontier.contains(&v1_id));

    let genesis = store.get_genesis(session_id).await.unwrap();
    assert!(genesis.contains(&v1_id));
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_dag_store_children() {
    let store = InMemoryDagStore::new();
    let session_id = SessionId::now();

    let parent = VertexBuilder::new(EventType::SessionStart).build();
    let mut parent_clone = parent.clone();
    let parent_id = parent_clone.id().unwrap();
    store.put_vertex(session_id, parent).await.unwrap();

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
async fn test_in_memory_payload_store() {
    let store = InMemoryPayloadStore::new();

    let data = bytes::Bytes::from("test payload data");
    let payload_ref = store.put(data.clone()).await.unwrap();

    assert!(store.exists(&payload_ref).await.unwrap());
    assert_eq!(store.payload_count().await, 1);

    let retrieved = store.get(&payload_ref).await.unwrap();
    assert_eq!(retrieved, Some(data));

    assert!(store.delete(&payload_ref).await.unwrap());
    assert!(!store.exists(&payload_ref).await.unwrap());
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_dag_store_delete_session() {
    let store = InMemoryDagStore::new();
    let session_id = SessionId::now();

    for _ in 0..5 {
        let vertex = VertexBuilder::new(EventType::SessionStart).build();
        store.put_vertex(session_id, vertex).await.unwrap();
    }

    assert_eq!(store.count_vertices(session_id).await.unwrap(), 5);

    store.delete_session(session_id).await.unwrap();
    assert_eq!(store.count_vertices(session_id).await.unwrap(), 0);
    assert_eq!(store.session_count().await, 0);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_dag_store_health_and_stats() {
    let store = InMemoryDagStore::new();
    let session_id = SessionId::now();

    assert_eq!(store.health().await, StorageHealth::Healthy);

    let stats = store.stats().await;
    assert_eq!(stats.sessions, 0);
    assert_eq!(stats.vertices, 0);

    for _ in 0..3 {
        let vertex = VertexBuilder::new(EventType::SessionStart).build();
        store.put_vertex(session_id, vertex).await.unwrap();
    }

    let stats = store.stats().await;
    assert_eq!(stats.sessions, 1);
    assert_eq!(stats.vertices, 3);
    assert!(stats.write_ops >= 3);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_dag_store_update_frontier() {
    let store = InMemoryDagStore::new();
    let session_id = SessionId::now();

    let v1 = VertexBuilder::new(EventType::SessionStart).build();
    let mut v1_clone = v1.clone();
    let v1_id = v1_clone.id().unwrap();
    store.put_vertex(session_id, v1).await.unwrap();

    let new_id = VertexId::from_bytes(&[42; 32]);
    store.update_frontier(session_id, new_id, &[v1_id]).await.unwrap();

    let frontier = store.get_frontier(session_id).await.unwrap();
    assert!(frontier.contains(&new_id));
    assert!(!frontier.contains(&v1_id));
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_payload_store_health_and_stats() {
    let store = InMemoryPayloadStore::new();

    assert_eq!(store.health().await, StorageHealth::Healthy);

    let stats = store.stats().await;
    assert_eq!(stats.vertices, 0);
    assert_eq!(stats.bytes_used, 0);

    let data = bytes::Bytes::from("test payload data");
    let _ref = store.put(data.clone()).await.unwrap();

    let stats = store.stats().await;
    assert_eq!(stats.vertices, 1);
    assert_eq!(stats.bytes_used, data.len() as u64);
    assert!(stats.write_ops >= 1);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_storage_health_variants_and_stats_default() {
    assert_eq!(
        StorageHealth::Degraded("io slow".to_string()),
        StorageHealth::Degraded("io slow".to_string())
    );
    assert_ne!(StorageHealth::Healthy, StorageHealth::Unhealthy("disk full".to_string()));
    let s = StorageStats::default();
    assert_eq!(s.sessions, 0);
    assert_eq!(s.vertices, 0);
    assert_eq!(s.bytes_used, 0);
    assert_eq!(s.read_ops, 0);
    assert_eq!(s.write_ops, 0);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_get_all_vertices_empty_and_diamond() {
    let store = InMemoryDagStore::new();
    let missing = SessionId::now();
    assert!(store.get_all_vertices(missing).await.unwrap().is_empty());

    let session_id = SessionId::now();
    let g = VertexBuilder::new(EventType::SessionStart).build();
    let mut g_c = g.clone();
    let g_id = g_c.id().unwrap();
    store.put_vertex(session_id, g).await.unwrap();

    let a = VertexBuilder::new(EventType::DataCreate {
        schema: Some("a".into()),
    })
    .with_parent(g_id)
    .build();
    let mut a_c = a.clone();
    let a_id = a_c.id().unwrap();
    store.put_vertex(session_id, a).await.unwrap();

    let b = VertexBuilder::new(EventType::DataCreate {
        schema: Some("b".into()),
    })
    .with_parent(g_id)
    .build();
    let mut b_c = b.clone();
    let b_id = b_c.id().unwrap();
    store.put_vertex(session_id, b).await.unwrap();

    let c = VertexBuilder::new(EventType::DataCreate {
        schema: Some("c".into()),
    })
    .with_parent(a_id)
    .with_parent(b_id)
    .build();
    store.put_vertex(session_id, c).await.unwrap();

    let ordered = store.get_all_vertices(session_id).await.unwrap();
    assert_eq!(ordered.len(), 4);
    let ids: Vec<VertexId> = ordered
        .iter()
        .map(|v| {
            let mut c = v.clone();
            c.id().unwrap()
        })
        .collect();
    assert!(ids.contains(&g_id) && ids.contains(&a_id) && ids.contains(&b_id));
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_get_vertices_batch_and_missing_session() {
    let store = InMemoryDagStore::new();
    let session_id = SessionId::now();
    let v = VertexBuilder::new(EventType::SessionStart).build();
    let mut vc = v.clone();
    let vid = vc.id().unwrap();
    store.put_vertex(session_id, v).await.unwrap();

    let other = VertexId::from_bytes(&[7; 32]);
    assert!(store.get_vertices(session_id, &[]).await.unwrap().is_empty());

    let batch = store.get_vertices(session_id, &[vid, other]).await.unwrap();
    assert_eq!(batch.len(), 2);
    assert!(batch[0].is_some());
    assert!(batch[1].is_none());

    let ghost = SessionId::now();
    assert!(store.get_vertex(ghost, vid).await.unwrap().is_none());
    let empty_batch = store.get_vertices(ghost, &[vid]).await.unwrap();
    assert_eq!(empty_batch.len(), 1);
    assert!(empty_batch[0].is_none());
    assert!(!store.exists(ghost, vid).await.unwrap());
    assert!(store.get_children(ghost, vid).await.unwrap().is_empty());
    assert!(store.get_genesis(ghost).await.unwrap().is_empty());
    assert!(store.get_frontier(ghost).await.unwrap().is_empty());
    assert_eq!(store.count_vertices(ghost).await.unwrap(), 0);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_update_frontier_no_session_is_ok() {
    let store = InMemoryDagStore::new();
    let missing = SessionId::now();
    let new_id = VertexId::from_bytes(&[9; 32]);
    store.update_frontier(missing, new_id, &[]).await.unwrap();
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_total_vertex_count_and_session_count_multi_session() {
    let store = InMemoryDagStore::new();
    let s1 = SessionId::now();
    let s2 = SessionId::now();
    for _ in 0..2 {
        store.put_vertex(s1, VertexBuilder::new(EventType::SessionStart).build()).await.unwrap();
    }
    store.put_vertex(s2, VertexBuilder::new(EventType::SessionStart).build()).await.unwrap();

    assert_eq!(store.session_count().await, 2);
    assert_eq!(store.total_vertex_count().await, 3);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_payload_delete_missing_total_bytes_and_read_ops() {
    let store = InMemoryPayloadStore::new();
    let missing_ref =
        crate::types::PayloadRef::from_bytes(&bytes::Bytes::from_static(b"never stored"));
    assert!(!store.delete(&missing_ref).await.unwrap());

    let d1 = bytes::Bytes::from("a");
    let d2 = bytes::Bytes::from("bb");
    let r1 = store.put(d1.clone()).await.unwrap();
    let _r2 = store.put(d2.clone()).await.unwrap();
    assert_eq!(store.payload_count().await, 2);
    assert_eq!(store.total_bytes().await, d1.len() + d2.len());

    assert!(store.delete(&r1).await.unwrap());
    let stats = store.stats().await;
    assert_eq!(stats.vertices, 1);
    assert!(stats.write_ops >= 3);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_dag_backend_memory_debug_and_dispatches() {
    let backend = DagBackend::Memory(InMemoryDagStore::new());
    assert_eq!(format!("{backend:?}"), "DagBackend::Memory");

    let session_id = SessionId::now();
    let v = VertexBuilder::new(EventType::SessionStart).build();
    let mut vc = v.clone();
    let vid = vc.id().unwrap();
    DagStore::put_vertex(&backend, session_id, v).await.unwrap();

    assert_eq!(backend.session_count().await, 1);
    assert_eq!(backend.total_vertex_count().await, 1);
    assert!(DagStore::get_vertex(&backend, session_id, vid).await.unwrap().is_some());
    let all = backend.get_all_vertices(session_id).await.unwrap();
    assert_eq!(all.len(), 1);
    assert_eq!(DagStore::health(&backend).await, StorageHealth::Healthy);
    let st = DagStore::stats(&backend).await;
    assert_eq!(st.sessions, 1);
    assert_eq!(st.vertices, 1);
}

#[cfg(feature = "redb")]
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_dag_backend_redb_dispatch_session_and_vertex_counts() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("dag_redb_test.db");
    let redb_store = crate::RedbDagStore::open(&path).unwrap();
    let backend = DagBackend::Redb(redb_store);

    let session_id = SessionId::now();
    let v = VertexBuilder::new(EventType::SessionStart).build();
    let mut vc = v.clone();
    let vid = vc.id().unwrap();
    DagStore::put_vertex(&backend, session_id, v).await.unwrap();

    assert_eq!(backend.session_count().await, 1);
    assert_eq!(backend.total_vertex_count().await, 1);
    let all = backend.get_all_vertices(session_id).await.unwrap();
    assert_eq!(all.len(), 1);
    assert!(DagStore::get_vertex(&backend, session_id, vid).await.unwrap().is_some());
    assert_eq!(format!("{backend:?}"), "DagBackend::Redb");
    assert_eq!(DagStore::health(&backend).await, StorageHealth::Healthy);
    let st = DagStore::stats(&backend).await;
    assert!(st.vertices >= 1);
}
