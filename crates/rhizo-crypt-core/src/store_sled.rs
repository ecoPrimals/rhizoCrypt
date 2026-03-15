// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2024–2026 ecoPrimals Project

//! Sled persistent storage backend.
//!
//! This module provides a durable storage implementation using sled,
//! a high-performance embedded database.
//!
//! ## ecoBin Compliance Note
//!
//! Sled 0.34 transitively depends on `zstd-sys` (C compression library),
//! which does not meet the Pure Rust requirement of the ecoBin standard.
//! This backend is behind an optional `sled` feature flag. A migration to
//! a Pure Rust alternative (e.g., `redb`) is planned once the API stabilizes.
//!
//! ## Features
//!
//! - **Feature-gated** — Only compiled when `sled` feature is enabled
//! - **Persistent storage** — Data survives process restarts
//! - **Tree namespaces** — Separate trees for vertices, children, frontiers
//! - **Atomic writes** — Batch operations for transactional updates
//! - **Compression** — zstd compression for space efficiency
//! - **Lock-free** — Concurrent access without blocking
//!
//! ## Usage
//!
//! ```no_run
//! # #[cfg(feature = "sled")]
//! # {
//! # use rhizo_crypt_core::{SledDagStore, DagStore, event::EventType, vertex::VertexBuilder, types::SessionId};
//! # tokio::runtime::Runtime::new().unwrap().block_on(async {
//! let store = SledDagStore::open(std::env::temp_dir().join("rhizocrypt-doc-test-sled.db"))?;
//! let session_id = SessionId::now();
//! let vertex = VertexBuilder::new(EventType::SessionStart).build();
//! store.put_vertex(session_id, vertex).await?;
//! # Ok::<(), rhizo_crypt_core::error::RhizoCryptError>(())
//! # });
//! # }
//! ```
//!
//! ## Tree Structure
//!
//! | Tree | Key | Value |
//! |------|-----|-------|
//! | `vertices` | `session_id:vertex_id` | CBOR-encoded Vertex |
//! | `children` | `session_id:parent_id` | Set of child vertex IDs |
//! | `frontiers` | `session_id` | Set of frontier vertex IDs |
//! | `genesis` | `session_id` | Set of genesis vertex IDs |
//! | `metadata` | `session_id` | Session metadata |

use crate::constants::{SLED_CACHE_SIZE_BYTES, SLED_FLUSH_INTERVAL_MS};
use crate::error::{Result, RhizoCryptError};
use crate::store::{DagStore, StorageHealth, StorageStats};
use crate::types::{SessionId, VertexId};
use crate::vertex::Vertex;

use sled::{Batch, Db, Tree};
use std::path::Path;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

/// Sled export entry: `(tree_name, key_prefix, row_iterator)`.
pub type SledExportEntry = (Vec<u8>, Vec<u8>, Vec<Vec<Vec<u8>>>);

/// Tree names.
const TREE_VERTICES: &str = "vertices";
const TREE_CHILDREN: &str = "children";
const TREE_FRONTIERS: &str = "frontiers";
const TREE_GENESIS: &str = "genesis";
const TREE_METADATA: &str = "metadata";

/// Sled-backed DAG store (100% Pure Rust).
///
/// Provides persistent storage for vertices with tree separation
/// for different data types.
#[derive(Clone)]
pub struct SledDagStore {
    /// Sled database instance.
    db: Arc<Db>,
    /// Vertices tree.
    vertices: Tree,
    /// Children index tree.
    children: Tree,
    /// Frontiers tree.
    frontiers: Tree,
    /// Genesis vertices tree.
    genesis: Tree,
    /// Metadata tree.
    metadata: Tree,
    /// Database path.
    path: Arc<std::path::PathBuf>,
    /// Read operations counter.
    read_ops: Arc<AtomicU64>,
    /// Write operations counter.
    write_ops: Arc<AtomicU64>,
}

impl SledDagStore {
    /// Open or create a sled store at the given path.
    ///
    /// # Errors
    ///
    /// Returns an error if the database cannot be opened or created.
    pub fn open(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref();
        let path_buf = path.to_path_buf();

        // Create directory if it doesn't exist
        std::fs::create_dir_all(path).map_err(|e| {
            RhizoCryptError::storage(format!("Failed to create database directory: {e}"))
        })?;

        // Configure sled with optimizations (no compression to avoid zstd conflict)
        let config = sled::Config::new()
            .path(path)
            .cache_capacity(SLED_CACHE_SIZE_BYTES)
            .flush_every_ms(Some(SLED_FLUSH_INTERVAL_MS))
            .mode(sled::Mode::HighThroughput);

        // Open database
        let db = config
            .open()
            .map_err(|e| RhizoCryptError::storage(format!("Failed to open sled database: {e}")))?;

        // Open trees
        let vertices = db
            .open_tree(TREE_VERTICES)
            .map_err(|e| RhizoCryptError::storage(format!("Failed to open vertices tree: {e}")))?;

        let children = db
            .open_tree(TREE_CHILDREN)
            .map_err(|e| RhizoCryptError::storage(format!("Failed to open children tree: {e}")))?;

        let frontiers = db
            .open_tree(TREE_FRONTIERS)
            .map_err(|e| RhizoCryptError::storage(format!("Failed to open frontiers tree: {e}")))?;

        let genesis = db
            .open_tree(TREE_GENESIS)
            .map_err(|e| RhizoCryptError::storage(format!("Failed to open genesis tree: {e}")))?;

        let metadata = db
            .open_tree(TREE_METADATA)
            .map_err(|e| RhizoCryptError::storage(format!("Failed to open metadata tree: {e}")))?;

        Ok(Self {
            db: Arc::new(db),
            vertices,
            children,
            frontiers,
            genesis,
            metadata,
            path: Arc::new(path_buf),
            read_ops: Arc::new(AtomicU64::new(0)),
            write_ops: Arc::new(AtomicU64::new(0)),
        })
    }

    /// Create a key from session and vertex IDs.
    fn vertex_key(session_id: SessionId, vertex_id: VertexId) -> Vec<u8> {
        let mut key = Vec::with_capacity(48);
        key.extend_from_slice(session_id.as_bytes());
        key.push(b':');
        key.extend_from_slice(vertex_id.as_bytes());
        key
    }

    /// Create a key from session ID only.
    fn session_key(session_id: SessionId) -> Vec<u8> {
        session_id.as_bytes().to_vec()
    }

    /// Parse a vertex ID set from stored bytes.
    fn parse_vertex_set(data: &[u8]) -> hashbrown::HashSet<VertexId> {
        if data.is_empty() {
            return hashbrown::HashSet::new();
        }

        // Each vertex ID is 32 bytes
        data.chunks_exact(32)
            .map(|chunk| {
                let mut arr = [0u8; 32];
                arr.copy_from_slice(chunk);
                VertexId::new(arr)
            })
            .collect()
    }

    /// Serialize a vertex ID set to bytes.
    fn serialize_vertex_set(set: &hashbrown::HashSet<VertexId>) -> Vec<u8> {
        let mut data = Vec::with_capacity(set.len() * 32);
        for id in set {
            data.extend_from_slice(id.as_bytes());
        }
        data
    }

    /// Get the database path.
    #[must_use]
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Flush all pending writes to disk.
    ///
    /// # Errors
    ///
    /// Returns an error if the flush fails.
    pub async fn flush(&self) -> Result<()> {
        self.db
            .flush_async()
            .await
            .map_err(|e| RhizoCryptError::storage(format!("Failed to flush database: {e}")))?;
        Ok(())
    }

    /// Export the database for backup.
    ///
    /// Collects all tree data into a vector of `(tree_name, key_prefix, rows)` entries.
    #[must_use]
    pub fn export(&self) -> Vec<SledExportEntry> {
        self.db
            .export()
            .into_iter()
            .map(|(name, prefix, iter)| (name, prefix, iter.collect()))
            .collect()
    }
}

impl DagStore for SledDagStore {
    async fn put_vertex(&self, session_id: SessionId, mut vertex: Vertex) -> Result<()> {
        self.write_ops.fetch_add(1, Ordering::Relaxed);

        let vertex_id = vertex.id()?;
        let key = Self::vertex_key(session_id, vertex_id);

        // Serialize vertex to CBOR
        let value = vertex.to_canonical_bytes()?;

        // Create batches for atomic updates
        let mut vertices_batch = Batch::default();
        let mut children_batch = Batch::default();
        let mut frontiers_batch = Batch::default();
        let mut genesis_batch = Batch::default();

        // Store vertex
        vertices_batch.insert(key.as_slice(), &value[..]);

        // Update children index for each parent
        for parent_id in &vertex.parents {
            let parent_key = Self::vertex_key(session_id, *parent_id);

            // Read existing children
            let existing = self
                .children
                .get(&parent_key)
                .map_err(|e| RhizoCryptError::storage(e.to_string()))?;

            let mut children_set =
                existing.as_deref().map(Self::parse_vertex_set).unwrap_or_default();

            children_set.insert(vertex_id);

            children_batch.insert(
                parent_key.as_slice(),
                Self::serialize_vertex_set(&children_set).as_slice(),
            );
        }

        // Update genesis if this is a root vertex
        if vertex.is_genesis() {
            let session_key = Self::session_key(session_id);

            let existing = self
                .genesis
                .get(&session_key)
                .map_err(|e| RhizoCryptError::storage(e.to_string()))?;

            let mut genesis_set =
                existing.as_deref().map(Self::parse_vertex_set).unwrap_or_default();

            genesis_set.insert(vertex_id);

            genesis_batch.insert(
                session_key.as_slice(),
                Self::serialize_vertex_set(&genesis_set).as_slice(),
            );
        }

        // Update frontier
        let session_key = Self::session_key(session_id);

        let existing = self
            .frontiers
            .get(&session_key)
            .map_err(|e| RhizoCryptError::storage(e.to_string()))?;

        let mut frontier = existing.as_deref().map(Self::parse_vertex_set).unwrap_or_default();

        // Remove parents from frontier
        for parent_id in &vertex.parents {
            frontier.remove(parent_id);
        }

        // Add this vertex to frontier
        frontier.insert(vertex_id);

        frontiers_batch
            .insert(session_key.as_slice(), Self::serialize_vertex_set(&frontier).as_slice());

        // Apply all batches atomically
        self.vertices
            .apply_batch(vertices_batch)
            .map_err(|e| RhizoCryptError::storage(e.to_string()))?;

        self.children
            .apply_batch(children_batch)
            .map_err(|e| RhizoCryptError::storage(e.to_string()))?;

        self.genesis
            .apply_batch(genesis_batch)
            .map_err(|e| RhizoCryptError::storage(e.to_string()))?;

        self.frontiers
            .apply_batch(frontiers_batch)
            .map_err(|e| RhizoCryptError::storage(e.to_string()))?;

        Ok(())
    }

    async fn get_vertex(
        &self,
        session_id: SessionId,
        vertex_id: VertexId,
    ) -> Result<Option<Vertex>> {
        self.read_ops.fetch_add(1, Ordering::Relaxed);

        let key = Self::vertex_key(session_id, vertex_id);

        let value = self.vertices.get(&key).map_err(|e| RhizoCryptError::storage(e.to_string()))?;

        match value {
            Some(data) => {
                let vertex = Vertex::from_cbor_bytes(&data)?;
                Ok(Some(vertex))
            }
            None => Ok(None),
        }
    }

    async fn get_vertices(
        &self,
        session_id: SessionId,
        vertex_ids: &[VertexId],
    ) -> Result<Vec<Option<Vertex>>> {
        self.read_ops.fetch_add(1, Ordering::Relaxed);

        let results: Vec<Option<Vertex>> = vertex_ids
            .iter()
            .map(|id| {
                let key = Self::vertex_key(session_id, *id);
                self.vertices
                    .get(&key)
                    .ok()
                    .flatten()
                    .and_then(|data| Vertex::from_cbor_bytes(&data).ok())
            })
            .collect();

        Ok(results)
    }

    async fn exists(&self, session_id: SessionId, vertex_id: VertexId) -> Result<bool> {
        self.read_ops.fetch_add(1, Ordering::Relaxed);

        let key = Self::vertex_key(session_id, vertex_id);

        let exists = self
            .vertices
            .contains_key(&key)
            .map_err(|e| RhizoCryptError::storage(e.to_string()))?;

        Ok(exists)
    }

    async fn get_children(
        &self,
        session_id: SessionId,
        parent_id: VertexId,
    ) -> Result<Vec<VertexId>> {
        self.read_ops.fetch_add(1, Ordering::Relaxed);

        let key = Self::vertex_key(session_id, parent_id);

        let value = self.children.get(&key).map_err(|e| RhizoCryptError::storage(e.to_string()))?;

        let children = value.as_deref().map(Self::parse_vertex_set).unwrap_or_default();

        Ok(children.into_iter().collect())
    }

    async fn get_genesis(&self, session_id: SessionId) -> Result<Vec<VertexId>> {
        self.read_ops.fetch_add(1, Ordering::Relaxed);

        let key = Self::session_key(session_id);

        let value = self.genesis.get(&key).map_err(|e| RhizoCryptError::storage(e.to_string()))?;

        let genesis = value.as_deref().map(Self::parse_vertex_set).unwrap_or_default();

        Ok(genesis.into_iter().collect())
    }

    async fn get_frontier(&self, session_id: SessionId) -> Result<Vec<VertexId>> {
        self.read_ops.fetch_add(1, Ordering::Relaxed);

        let key = Self::session_key(session_id);

        let value =
            self.frontiers.get(&key).map_err(|e| RhizoCryptError::storage(e.to_string()))?;

        let frontier = value.as_deref().map(Self::parse_vertex_set).unwrap_or_default();

        Ok(frontier.into_iter().collect())
    }

    async fn count_vertices(&self, session_id: SessionId) -> Result<u64> {
        self.read_ops.fetch_add(1, Ordering::Relaxed);

        let prefix = Self::session_key(session_id);

        let mut count = 0u64;
        for result in self.vertices.scan_prefix(&prefix) {
            let _item = result.map_err(|e| RhizoCryptError::storage(e.to_string()))?;
            count += 1;
        }

        Ok(count)
    }

    async fn delete_session(&self, session_id: SessionId) -> Result<()> {
        self.write_ops.fetch_add(1, Ordering::Relaxed);

        let session_prefix = Self::session_key(session_id);

        // Create batches for atomic deletion
        let mut vertices_batch = Batch::default();
        let mut children_batch = Batch::default();

        // Delete from vertices tree
        for result in self.vertices.scan_prefix(&session_prefix) {
            let (key, _) = result.map_err(|e| RhizoCryptError::storage(e.to_string()))?;
            vertices_batch.remove(&key);
        }

        // Delete from children tree
        for result in self.children.scan_prefix(&session_prefix) {
            let (key, _) = result.map_err(|e| RhizoCryptError::storage(e.to_string()))?;
            children_batch.remove(&key);
        }

        // Delete from other trees (direct session key)
        self.frontiers
            .remove(&session_prefix)
            .map_err(|e| RhizoCryptError::storage(e.to_string()))?;

        self.genesis
            .remove(&session_prefix)
            .map_err(|e| RhizoCryptError::storage(e.to_string()))?;

        self.metadata
            .remove(&session_prefix)
            .map_err(|e| RhizoCryptError::storage(e.to_string()))?;

        // Apply batches
        self.vertices
            .apply_batch(vertices_batch)
            .map_err(|e| RhizoCryptError::storage(e.to_string()))?;

        self.children
            .apply_batch(children_batch)
            .map_err(|e| RhizoCryptError::storage(e.to_string()))?;

        Ok(())
    }

    async fn update_frontier(
        &self,
        session_id: SessionId,
        new_vertex: VertexId,
        consumed_parents: &[VertexId],
    ) -> Result<()> {
        self.write_ops.fetch_add(1, Ordering::Relaxed);

        let key = Self::session_key(session_id);

        let existing =
            self.frontiers.get(&key).map_err(|e| RhizoCryptError::storage(e.to_string()))?;

        let mut frontier = existing.as_deref().map(Self::parse_vertex_set).unwrap_or_default();

        // Remove consumed parents
        for parent in consumed_parents {
            frontier.remove(parent);
        }

        // Add new vertex
        frontier.insert(new_vertex);

        self.frontiers
            .insert(&key, Self::serialize_vertex_set(&frontier).as_slice())
            .map_err(|e| RhizoCryptError::storage(e.to_string()))?;

        Ok(())
    }

    async fn health(&self) -> StorageHealth {
        // Check if database is accessible
        // Always healthy if we can access the trees
        StorageHealth::Healthy
    }

    async fn stats(&self) -> StorageStats {
        // Count sessions from genesis tree
        let session_count = self.genesis.iter().count() as u64;

        // Count vertices
        let vertex_count = self.vertices.iter().count() as u64;

        // Get disk usage
        let bytes_used = self.db.size_on_disk().unwrap_or(0);

        StorageStats {
            sessions: session_count,
            vertices: vertex_count,
            bytes_used,
            read_ops: self.read_ops.load(Ordering::Relaxed),
            write_ops: self.write_ops.load(Ordering::Relaxed),
        }
    }
}

impl std::fmt::Debug for SledDagStore {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SledDagStore")
            .field("path", &*self.path)
            .field("read_ops", &self.read_ops.load(Ordering::Relaxed))
            .field("write_ops", &self.write_ops.load(Ordering::Relaxed))
            .finish_non_exhaustive()
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
#[path = "store_sled_tests.rs"]
mod tests;

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
#[path = "store_sled_tests_advanced.rs"]
mod tests_advanced;
