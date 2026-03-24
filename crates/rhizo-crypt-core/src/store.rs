// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2024–2026 ecoPrimals Project

//! DAG storage traits and implementations.
//!
//! This module defines the storage interface and provides an in-memory implementation.

use crate::error::Result;
use crate::types::{SessionId, VertexId};
use crate::vertex::Vertex;
use hashbrown::{HashMap, HashSet};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use tokio::sync::RwLock;

/// Health status of a storage backend.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum StorageHealth {
    /// Backend is healthy and operational.
    Healthy,
    /// Backend is degraded but functional.
    Degraded(String),
    /// Backend is unhealthy and may not function correctly.
    Unhealthy(String),
}

/// Storage statistics for monitoring and debugging.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct StorageStats {
    /// Number of active sessions.
    pub sessions: u64,
    /// Total number of vertices across all sessions.
    pub vertices: u64,
    /// Estimated bytes used by stored data.
    pub bytes_used: u64,
    /// Read operations performed.
    pub read_ops: u64,
    /// Write operations performed.
    pub write_ops: u64,
}

/// Primary storage trait for DAG vertices.
pub trait DagStore: Send + Sync {
    /// Store a vertex.
    fn put_vertex(
        &self,
        session_id: SessionId,
        vertex: Vertex,
    ) -> impl std::future::Future<Output = Result<()>> + Send;

    /// Get a vertex by ID.
    fn get_vertex(
        &self,
        session_id: SessionId,
        vertex_id: VertexId,
    ) -> impl std::future::Future<Output = Result<Option<Vertex>>> + Send;

    /// Get multiple vertices.
    fn get_vertices(
        &self,
        session_id: SessionId,
        vertex_ids: &[VertexId],
    ) -> impl std::future::Future<Output = Result<Vec<Option<Vertex>>>> + Send;

    /// Check if vertex exists.
    fn exists(
        &self,
        session_id: SessionId,
        vertex_id: VertexId,
    ) -> impl std::future::Future<Output = Result<bool>> + Send;

    /// Get children of a vertex.
    fn get_children(
        &self,
        session_id: SessionId,
        parent_id: VertexId,
    ) -> impl std::future::Future<Output = Result<Vec<VertexId>>> + Send;

    /// Get genesis vertices for a session.
    fn get_genesis(
        &self,
        session_id: SessionId,
    ) -> impl std::future::Future<Output = Result<Vec<VertexId>>> + Send;

    /// Get frontier vertices for a session.
    fn get_frontier(
        &self,
        session_id: SessionId,
    ) -> impl std::future::Future<Output = Result<Vec<VertexId>>> + Send;

    /// Count vertices in a session.
    fn count_vertices(
        &self,
        session_id: SessionId,
    ) -> impl std::future::Future<Output = Result<u64>> + Send;

    /// Delete a session and all its vertices.
    fn delete_session(
        &self,
        session_id: SessionId,
    ) -> impl std::future::Future<Output = Result<()>> + Send;

    /// Update frontier after vertex append (for backends that need explicit updates).
    ///
    /// # Errors
    ///
    /// Returns an error if the session doesn't exist or the update fails.
    fn update_frontier(
        &self,
        session_id: SessionId,
        new_vertex: VertexId,
        consumed_parents: &[VertexId],
    ) -> impl std::future::Future<Output = Result<()>> + Send;

    /// Get health status of the storage backend.
    fn health(&self) -> impl std::future::Future<Output = StorageHealth> + Send;

    /// Get storage statistics.
    fn stats(&self) -> impl std::future::Future<Output = StorageStats> + Send;
}

/// Session data stored in memory.
#[derive(Clone, Debug, Default)]
struct SessionData {
    /// Vertices indexed by ID.
    vertices: HashMap<VertexId, Vertex>,
    /// Parent to children index.
    children: HashMap<VertexId, HashSet<VertexId>>,
    /// Genesis vertices.
    genesis: HashSet<VertexId>,
    /// Frontier vertices.
    frontier: HashSet<VertexId>,
}

/// In-memory DAG store implementation.
#[derive(Debug, Clone, Default)]
pub struct InMemoryDagStore {
    /// Sessions data.
    sessions: Arc<RwLock<HashMap<SessionId, SessionData>>>,
    /// Read operations counter.
    read_ops: Arc<AtomicU64>,
    /// Write operations counter.
    write_ops: Arc<AtomicU64>,
}

impl InMemoryDagStore {
    /// Create a new in-memory DAG store.
    #[must_use]
    pub fn new() -> Self {
        Self {
            sessions: Arc::new(RwLock::new(HashMap::new())),
            read_ops: Arc::new(AtomicU64::new(0)),
            write_ops: Arc::new(AtomicU64::new(0)),
        }
    }

    /// Get the number of sessions.
    pub async fn session_count(&self) -> usize {
        self.sessions.read().await.len()
    }

    /// Get the total number of vertices across all sessions.
    pub async fn total_vertex_count(&self) -> usize {
        let sessions = self.sessions.read().await;
        sessions.values().map(|s| s.vertices.len()).sum()
    }

    /// Get all vertices for a session in topological order.
    ///
    /// Returns vertices ordered so parents come before children.
    /// Returns an empty vector if the session doesn't exist.
    ///
    /// # Errors
    ///
    /// This function currently doesn't return errors but may in future
    /// storage backend implementations.
    pub async fn get_all_vertices(&self, session_id: SessionId) -> Result<Vec<Vertex>> {
        let sessions = self.sessions.read().await;
        let Some(session) = sessions.get(&session_id) else {
            return Ok(Vec::new());
        };

        // Simple topological sort: BFS from genesis
        let mut result = Vec::new();
        let mut visited = HashSet::new();
        let mut queue: std::collections::VecDeque<VertexId> =
            session.genesis.iter().copied().collect();

        while let Some(vertex_id) = queue.pop_front() {
            if visited.contains(&vertex_id) {
                continue;
            }
            visited.insert(vertex_id);

            if let Some(vertex) = session.vertices.get(&vertex_id) {
                result.push(vertex.clone());
            }

            // Add children to queue
            if let Some(children) = session.children.get(&vertex_id) {
                for child_id in children {
                    if !visited.contains(child_id) {
                        queue.push_back(*child_id);
                    }
                }
            }
        }

        Ok(result)
    }
}

impl DagStore for InMemoryDagStore {
    async fn put_vertex(&self, session_id: SessionId, mut vertex: Vertex) -> Result<()> {
        self.write_ops.fetch_add(1, Ordering::Relaxed);

        // Compute vertex ID if not already computed
        let vertex_id = vertex.id()?;

        let mut sessions = self.sessions.write().await;
        let session = sessions.entry(session_id).or_default();

        // Update children index
        for parent_id in &vertex.parents {
            session.children.entry(*parent_id).or_default().insert(vertex_id);
        }

        // Update genesis/frontier
        if vertex.is_genesis() {
            session.genesis.insert(vertex_id);
        }

        // Remove parents from frontier, add this vertex
        for parent_id in &vertex.parents {
            session.frontier.remove(parent_id);
        }
        session.frontier.insert(vertex_id);

        // Store vertex
        session.vertices.insert(vertex_id, vertex);

        Ok(())
    }

    async fn get_vertex(
        &self,
        session_id: SessionId,
        vertex_id: VertexId,
    ) -> Result<Option<Vertex>> {
        self.read_ops.fetch_add(1, Ordering::Relaxed);
        let sessions = self.sessions.read().await;
        Ok(sessions.get(&session_id).and_then(|s| s.vertices.get(&vertex_id).cloned()))
    }

    async fn get_vertices(
        &self,
        session_id: SessionId,
        vertex_ids: &[VertexId],
    ) -> Result<Vec<Option<Vertex>>> {
        self.read_ops.fetch_add(1, Ordering::Relaxed);
        let session = self.sessions.read().await.get(&session_id).cloned();
        Ok(vertex_ids
            .iter()
            .map(|id| session.as_ref().and_then(|s| s.vertices.get(id).cloned()))
            .collect())
    }

    async fn exists(&self, session_id: SessionId, vertex_id: VertexId) -> Result<bool> {
        self.read_ops.fetch_add(1, Ordering::Relaxed);
        let sessions = self.sessions.read().await;
        Ok(sessions.get(&session_id).is_some_and(|s| s.vertices.contains_key(&vertex_id)))
    }

    async fn get_children(
        &self,
        session_id: SessionId,
        parent_id: VertexId,
    ) -> Result<Vec<VertexId>> {
        self.read_ops.fetch_add(1, Ordering::Relaxed);
        let sessions = self.sessions.read().await;
        Ok(sessions
            .get(&session_id)
            .and_then(|s| s.children.get(&parent_id))
            .map(|c| c.iter().copied().collect())
            .unwrap_or_default())
    }

    async fn get_genesis(&self, session_id: SessionId) -> Result<Vec<VertexId>> {
        self.read_ops.fetch_add(1, Ordering::Relaxed);
        let sessions = self.sessions.read().await;
        Ok(sessions
            .get(&session_id)
            .map(|s| s.genesis.iter().copied().collect())
            .unwrap_or_default())
    }

    async fn get_frontier(&self, session_id: SessionId) -> Result<Vec<VertexId>> {
        self.read_ops.fetch_add(1, Ordering::Relaxed);
        let sessions = self.sessions.read().await;
        Ok(sessions
            .get(&session_id)
            .map(|s| s.frontier.iter().copied().collect())
            .unwrap_or_default())
    }

    async fn count_vertices(&self, session_id: SessionId) -> Result<u64> {
        self.read_ops.fetch_add(1, Ordering::Relaxed);
        let sessions = self.sessions.read().await;
        Ok(sessions
            .get(&session_id)
            .map_or(0, |s| u64::try_from(s.vertices.len()).unwrap_or(u64::MAX)))
    }

    async fn delete_session(&self, session_id: SessionId) -> Result<()> {
        self.write_ops.fetch_add(1, Ordering::Relaxed);
        self.sessions.write().await.remove(&session_id);
        Ok(())
    }

    async fn update_frontier(
        &self,
        session_id: SessionId,
        new_vertex: VertexId,
        consumed_parents: &[VertexId],
    ) -> Result<()> {
        self.write_ops.fetch_add(1, Ordering::Relaxed);
        let mut sessions = self.sessions.write().await;
        if let Some(session) = sessions.get_mut(&session_id) {
            for parent in consumed_parents {
                session.frontier.remove(parent);
            }
            session.frontier.insert(new_vertex);
        }
        Ok(())
    }

    async fn health(&self) -> StorageHealth {
        StorageHealth::Healthy
    }

    async fn stats(&self) -> StorageStats {
        let sessions = self.sessions.read().await;
        let session_count = u64::try_from(sessions.len()).unwrap_or(u64::MAX);
        let vertex_count: u64 =
            sessions.values().map(|s| u64::try_from(s.vertices.len()).unwrap_or(u64::MAX)).sum();

        let bytes_estimate = vertex_count * crate::constants::ESTIMATED_BYTES_PER_VERTEX;

        StorageStats {
            sessions: session_count,
            vertices: vertex_count,
            bytes_used: bytes_estimate,
            read_ops: self.read_ops.load(Ordering::Relaxed),
            write_ops: self.write_ops.load(Ordering::Relaxed),
        }
    }
}

/// Payload storage trait (content-addressed).
pub trait PayloadStore: Send + Sync {
    /// Store a payload, returns content hash.
    fn put(
        &self,
        data: bytes::Bytes,
    ) -> impl std::future::Future<Output = Result<crate::types::PayloadRef>> + Send;

    /// Get a payload by reference.
    fn get(
        &self,
        payload_ref: &crate::types::PayloadRef,
    ) -> impl std::future::Future<Output = Result<Option<bytes::Bytes>>> + Send;

    /// Check if payload exists.
    fn exists(
        &self,
        payload_ref: &crate::types::PayloadRef,
    ) -> impl std::future::Future<Output = Result<bool>> + Send;

    /// Delete a payload.
    fn delete(
        &self,
        payload_ref: &crate::types::PayloadRef,
    ) -> impl std::future::Future<Output = Result<bool>> + Send;

    /// Get health status of the payload store.
    fn health(&self) -> impl std::future::Future<Output = StorageHealth> + Send;

    /// Get storage statistics.
    fn stats(&self) -> impl std::future::Future<Output = StorageStats> + Send;
}

/// In-memory payload store implementation.
#[derive(Debug, Clone, Default)]
pub struct InMemoryPayloadStore {
    /// Payloads indexed by hash.
    payloads: Arc<RwLock<HashMap<[u8; 32], bytes::Bytes>>>,
    /// Read operations counter.
    read_ops: Arc<AtomicU64>,
    /// Write operations counter.
    write_ops: Arc<AtomicU64>,
}

impl InMemoryPayloadStore {
    /// Create a new in-memory payload store.
    #[must_use]
    pub fn new() -> Self {
        Self {
            payloads: Arc::new(RwLock::new(HashMap::new())),
            read_ops: Arc::new(AtomicU64::new(0)),
            write_ops: Arc::new(AtomicU64::new(0)),
        }
    }

    /// Get the number of stored payloads.
    pub async fn payload_count(&self) -> usize {
        self.payloads.read().await.len()
    }

    /// Get total stored bytes.
    pub async fn total_bytes(&self) -> usize {
        let payloads = self.payloads.read().await;
        payloads.values().map(bytes::Bytes::len).sum()
    }
}

impl PayloadStore for InMemoryPayloadStore {
    async fn put(&self, data: bytes::Bytes) -> Result<crate::types::PayloadRef> {
        self.write_ops.fetch_add(1, Ordering::Relaxed);
        let payload_ref = crate::types::PayloadRef::from_bytes(&data);
        self.payloads.write().await.insert(payload_ref.hash, data);
        Ok(payload_ref)
    }

    async fn get(&self, payload_ref: &crate::types::PayloadRef) -> Result<Option<bytes::Bytes>> {
        self.read_ops.fetch_add(1, Ordering::Relaxed);
        let payloads = self.payloads.read().await;
        Ok(payloads.get(&payload_ref.hash).cloned())
    }

    async fn exists(&self, payload_ref: &crate::types::PayloadRef) -> Result<bool> {
        self.read_ops.fetch_add(1, Ordering::Relaxed);
        let payloads = self.payloads.read().await;
        Ok(payloads.contains_key(&payload_ref.hash))
    }

    async fn delete(&self, payload_ref: &crate::types::PayloadRef) -> Result<bool> {
        self.write_ops.fetch_add(1, Ordering::Relaxed);
        let mut payloads = self.payloads.write().await;
        Ok(payloads.remove(&payload_ref.hash).is_some())
    }

    async fn health(&self) -> StorageHealth {
        StorageHealth::Healthy
    }

    async fn stats(&self) -> StorageStats {
        let payloads = self.payloads.read().await;
        let bytes_used: u64 =
            payloads.values().map(|b| u64::try_from(b.len()).unwrap_or(u64::MAX)).sum();

        StorageStats {
            sessions: 0,
            vertices: u64::try_from(payloads.len()).unwrap_or(u64::MAX),
            bytes_used,
            read_ops: self.read_ops.load(Ordering::Relaxed),
            write_ops: self.write_ops.load(Ordering::Relaxed),
        }
    }
}

// ============================================================================
// DagBackend: runtime-dispatched storage backend
// ============================================================================

/// Runtime storage backend dispatch.
///
/// Wraps concrete `DagStore` implementations so `RhizoCrypt` can select
/// the backend at startup without trait objects (RPITIT makes `DagStore`
/// non-object-safe). Each variant delegates to its concrete store.
#[derive(Clone)]
pub enum DagBackend {
    /// In-memory (default for tests and ephemeral workloads).
    Memory(InMemoryDagStore),
    /// redb (Pure Rust, ACID, MVCC — recommended for production).
    #[cfg(feature = "redb")]
    Redb(crate::store_redb::RedbDagStore),
}

impl DagBackend {
    /// Get all vertices for a session in topological order.
    ///
    /// Delegates to the concrete backend's `get_all_vertices`.
    ///
    /// # Errors
    ///
    /// Returns an error if the backend encounters a storage failure.
    pub async fn get_all_vertices(&self, session_id: SessionId) -> Result<Vec<Vertex>> {
        match self {
            Self::Memory(store) => store.get_all_vertices(session_id).await,
            #[cfg(feature = "redb")]
            Self::Redb(store) => store.get_all_vertices(session_id).await,
        }
    }

    /// Get the number of sessions (for testing and metrics).
    pub async fn session_count(&self) -> usize {
        match self {
            Self::Memory(store) => store.session_count().await,
            #[cfg(feature = "redb")]
            Self::Redb(store) => {
                let stats = store.stats().await;
                usize::try_from(stats.sessions).unwrap_or(usize::MAX)
            }
        }
    }

    /// Get the total number of vertices across all sessions.
    pub async fn total_vertex_count(&self) -> usize {
        match self {
            Self::Memory(store) => store.total_vertex_count().await,
            #[cfg(feature = "redb")]
            Self::Redb(store) => {
                let stats = store.stats().await;
                usize::try_from(stats.vertices).unwrap_or(usize::MAX)
            }
        }
    }
}

impl DagStore for DagBackend {
    async fn put_vertex(&self, session_id: SessionId, vertex: Vertex) -> Result<()> {
        match self {
            Self::Memory(s) => s.put_vertex(session_id, vertex).await,
            #[cfg(feature = "redb")]
            Self::Redb(s) => s.put_vertex(session_id, vertex).await,
        }
    }

    async fn get_vertex(
        &self,
        session_id: SessionId,
        vertex_id: VertexId,
    ) -> Result<Option<Vertex>> {
        match self {
            Self::Memory(s) => s.get_vertex(session_id, vertex_id).await,
            #[cfg(feature = "redb")]
            Self::Redb(s) => s.get_vertex(session_id, vertex_id).await,
        }
    }

    async fn get_vertices(
        &self,
        session_id: SessionId,
        vertex_ids: &[VertexId],
    ) -> Result<Vec<Option<Vertex>>> {
        match self {
            Self::Memory(s) => s.get_vertices(session_id, vertex_ids).await,
            #[cfg(feature = "redb")]
            Self::Redb(s) => s.get_vertices(session_id, vertex_ids).await,
        }
    }

    async fn exists(&self, session_id: SessionId, vertex_id: VertexId) -> Result<bool> {
        match self {
            Self::Memory(s) => s.exists(session_id, vertex_id).await,
            #[cfg(feature = "redb")]
            Self::Redb(s) => s.exists(session_id, vertex_id).await,
        }
    }

    async fn get_children(
        &self,
        session_id: SessionId,
        parent_id: VertexId,
    ) -> Result<Vec<VertexId>> {
        match self {
            Self::Memory(s) => s.get_children(session_id, parent_id).await,
            #[cfg(feature = "redb")]
            Self::Redb(s) => s.get_children(session_id, parent_id).await,
        }
    }

    async fn get_genesis(&self, session_id: SessionId) -> Result<Vec<VertexId>> {
        match self {
            Self::Memory(s) => s.get_genesis(session_id).await,
            #[cfg(feature = "redb")]
            Self::Redb(s) => s.get_genesis(session_id).await,
        }
    }

    async fn get_frontier(&self, session_id: SessionId) -> Result<Vec<VertexId>> {
        match self {
            Self::Memory(s) => s.get_frontier(session_id).await,
            #[cfg(feature = "redb")]
            Self::Redb(s) => s.get_frontier(session_id).await,
        }
    }

    async fn count_vertices(&self, session_id: SessionId) -> Result<u64> {
        match self {
            Self::Memory(s) => s.count_vertices(session_id).await,
            #[cfg(feature = "redb")]
            Self::Redb(s) => s.count_vertices(session_id).await,
        }
    }

    async fn delete_session(&self, session_id: SessionId) -> Result<()> {
        match self {
            Self::Memory(s) => s.delete_session(session_id).await,
            #[cfg(feature = "redb")]
            Self::Redb(s) => s.delete_session(session_id).await,
        }
    }

    async fn update_frontier(
        &self,
        session_id: SessionId,
        new_vertex: VertexId,
        consumed_parents: &[VertexId],
    ) -> Result<()> {
        match self {
            Self::Memory(s) => s.update_frontier(session_id, new_vertex, consumed_parents).await,
            #[cfg(feature = "redb")]
            Self::Redb(s) => s.update_frontier(session_id, new_vertex, consumed_parents).await,
        }
    }

    async fn health(&self) -> StorageHealth {
        match self {
            Self::Memory(s) => s.health().await,
            #[cfg(feature = "redb")]
            Self::Redb(s) => s.health().await,
        }
    }

    async fn stats(&self) -> StorageStats {
        match self {
            Self::Memory(s) => s.stats().await,
            #[cfg(feature = "redb")]
            Self::Redb(s) => s.stats().await,
        }
    }
}

impl std::fmt::Debug for DagBackend {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Memory(_) => write!(f, "DagBackend::Memory"),
            #[cfg(feature = "redb")]
            Self::Redb(_) => write!(f, "DagBackend::Redb"),
        }
    }
}

#[cfg(test)]
#[expect(clippy::unwrap_used, reason = "test code")]
#[path = "store_tests.rs"]
mod tests;
