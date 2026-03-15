// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2024–2026 ecoPrimals Project

//! redb persistent storage backend.
//!
//! This module provides a durable storage implementation using redb,
//! a Pure Rust embedded key-value database.
//!
//! ## Features
//!
//! - **Feature-gated** — Only compiled when `redb` feature is enabled
//! - **100% Pure Rust** — No C dependencies, ecoBin compliant
//! - **Persistent storage** — Data survives process restarts
//! - **ACID transactions** — Fully transactional writes
//! - **MVCC** — Concurrent readers without blocking writers
//!
//! ## Usage
//!
//! ```no_run
//! # #[cfg(feature = "redb")]
//! # {
//! # use rhizo_crypt_core::{RedbDagStore, DagStore, event::EventType, vertex::VertexBuilder, types::SessionId};
//! # tokio::runtime::Runtime::new().unwrap().block_on(async {
//! let store = RedbDagStore::open(std::env::temp_dir().join("rhizocrypt-doc-test.db"))?;
//! let session_id = SessionId::now();
//! let vertex = VertexBuilder::new(EventType::SessionStart).build();
//! store.put_vertex(session_id, vertex).await?;
//! # Ok::<(), rhizo_crypt_core::error::RhizoCryptError>(())
//! # });
//! # }
//! ```
//!
//! ## Table Structure
//!
//! | Table     | Key                    | Value                    |
//! |-----------|------------------------|--------------------------|
//! | `vertices`| `session_id:vertex_id` | CBOR-encoded Vertex      |
//! | `children`| `session_id:parent_id` | Packed vertex ID set     |
//! | `frontiers`| `session_id`          | Packed vertex ID set     |
//! | `genesis` | `session_id`           | Packed vertex ID set     |
//! | `metadata`| `session_id`           | Session metadata bytes   |

use crate::error::{Result, RhizoCryptError};
use crate::store::{DagStore, StorageHealth, StorageStats};
use crate::types::{SessionId, VertexId};
use crate::vertex::Vertex;
use redb::{Database, ReadableTableMetadata, TableDefinition};
use std::path::Path;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

/// Table definitions.
const VERTICES: TableDefinition<&[u8], &[u8]> = TableDefinition::new("vertices");
const CHILDREN: TableDefinition<&[u8], &[u8]> = TableDefinition::new("children");
const FRONTIERS: TableDefinition<&[u8], &[u8]> = TableDefinition::new("frontiers");
const GENESIS: TableDefinition<&[u8], &[u8]> = TableDefinition::new("genesis");
const METADATA: TableDefinition<&[u8], &[u8]> = TableDefinition::new("metadata");

/// redb-backed DAG store (100% Pure Rust).
///
/// Provides persistent storage for vertices with table separation
/// for different data types.
#[derive(Clone)]
pub struct RedbDagStore {
    /// redb database instance.
    db: Arc<Database>,
    /// Database path.
    path: Arc<std::path::PathBuf>,
    /// Read operations counter.
    read_ops: Arc<AtomicU64>,
    /// Write operations counter.
    write_ops: Arc<AtomicU64>,
}

impl RedbDagStore {
    /// Open or create a redb store at the given path.
    ///
    /// # Errors
    ///
    /// Returns an error if the database cannot be opened or created.
    pub fn open(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref();
        let path_buf = path.to_path_buf();

        // Create parent directory if it doesn't exist
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| {
                RhizoCryptError::storage(format!("Failed to create database directory: {e}"))
            })?;
        }

        let db = Database::create(path)
            .map_err(|e| RhizoCryptError::storage(format!("Failed to open redb database: {e}")))?;

        // Ensure all tables exist (redb creates tables on first write, but we need them for reads)
        let write_txn = db
            .begin_write()
            .map_err(|e| RhizoCryptError::storage(format!("Failed to begin write: {e}")))?;
        {
            let _ = write_txn.open_table(VERTICES).map_err(|e| {
                RhizoCryptError::storage(format!("Failed to open vertices table: {e}"))
            })?;
            let _ = write_txn.open_table(CHILDREN).map_err(|e| {
                RhizoCryptError::storage(format!("Failed to open children table: {e}"))
            })?;
            let _ = write_txn.open_table(FRONTIERS).map_err(|e| {
                RhizoCryptError::storage(format!("Failed to open frontiers table: {e}"))
            })?;
            let _ = write_txn.open_table(GENESIS).map_err(|e| {
                RhizoCryptError::storage(format!("Failed to open genesis table: {e}"))
            })?;
            let _ = write_txn.open_table(METADATA).map_err(|e| {
                RhizoCryptError::storage(format!("Failed to open metadata table: {e}"))
            })?;
        }
        write_txn
            .commit()
            .map_err(|e| RhizoCryptError::storage(format!("Failed to commit: {e}")))?;

        Ok(Self {
            db: Arc::new(db),
            path: Arc::new(path_buf),
            read_ops: Arc::new(AtomicU64::new(0)),
            write_ops: Arc::new(AtomicU64::new(0)),
        })
    }

    /// Create a key from session and vertex IDs (49 bytes: 16 + 1 + 32).
    fn vertex_key(session_id: SessionId, vertex_id: VertexId) -> Vec<u8> {
        let mut key = Vec::with_capacity(49);
        key.extend_from_slice(session_id.as_bytes());
        key.push(b':');
        key.extend_from_slice(vertex_id.as_bytes());
        key
    }

    /// Create a key from session ID only (16 bytes).
    fn session_key(session_id: SessionId) -> Vec<u8> {
        session_id.as_bytes().to_vec()
    }

    /// Create prefix range for session-scoped keys (vertices, children).
    /// Returns (start, end) where start = session_id + ':' and end = session_id + ';'.
    fn session_prefix_range(session_id: SessionId) -> (Vec<u8>, Vec<u8>) {
        let mut start = session_id.as_bytes().to_vec();
        start.push(b':');
        let mut end = session_id.as_bytes().to_vec();
        end.push(b';'); // ':' + 1 = ';'
        (start, end)
    }

    /// Parse a vertex ID set from stored bytes.
    fn parse_vertex_set(data: &[u8]) -> hashbrown::HashSet<VertexId> {
        if data.is_empty() {
            return hashbrown::HashSet::new();
        }

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

    /// Read a vertex ID set from a table, returning an empty set if no entry exists.
    fn read_vertex_set(
        &self,
        table: TableDefinition<&[u8], &[u8]>,
        key: &[u8],
    ) -> Result<hashbrown::HashSet<VertexId>> {
        let read_txn = self
            .db
            .begin_read()
            .map_err(|e| RhizoCryptError::storage(format!("Failed to begin read: {e}")))?;
        let t = read_txn
            .open_table(table)
            .map_err(|e| RhizoCryptError::storage(format!("Failed to open table: {e}")))?;
        let existing = t
            .get(key)
            .map_err(|e| RhizoCryptError::storage(e.to_string()))?
            .map(|g| g.value().to_vec());
        Ok(existing.as_deref().map(Self::parse_vertex_set).unwrap_or_default())
    }

    /// Insert a serialized vertex set into an open table within a write transaction.
    fn write_vertex_set(
        write_txn: &redb::WriteTransaction,
        table: TableDefinition<&[u8], &[u8]>,
        key: &[u8],
        set: &hashbrown::HashSet<VertexId>,
    ) -> Result<()> {
        let mut t = write_txn
            .open_table(table)
            .map_err(|e| RhizoCryptError::storage(format!("Failed to open table: {e}")))?;
        t.insert(key, Self::serialize_vertex_set(set).as_slice())
            .map_err(|e| RhizoCryptError::storage(e.to_string()))?;
        Ok(())
    }
}

impl DagStore for RedbDagStore {
    async fn put_vertex(&self, session_id: SessionId, mut vertex: Vertex) -> Result<()> {
        self.write_ops.fetch_add(1, Ordering::Relaxed);

        let vertex_id = vertex
            .id()
            .map_err(|e| RhizoCryptError::storage(format!("Failed to compute vertex ID: {e}")))?;
        let value = vertex.to_canonical_bytes()?;

        let write_txn = self.db.begin_write().map_err(|e| {
            RhizoCryptError::storage(format!("Failed to begin write transaction: {e}"))
        })?;

        // Store vertex
        {
            let mut t = write_txn.open_table(VERTICES).map_err(|e| {
                RhizoCryptError::storage(format!("Failed to open vertices table: {e}"))
            })?;
            let key = Self::vertex_key(session_id, vertex_id);
            t.insert(key.as_slice(), &value[..])
                .map_err(|e| RhizoCryptError::storage(e.to_string()))?;
        }

        // Update children index for each parent
        for parent_id in &vertex.parents {
            let parent_key = Self::vertex_key(session_id, *parent_id);
            let mut children_set = self.read_vertex_set(CHILDREN, &parent_key)?;
            children_set.insert(vertex_id);
            Self::write_vertex_set(&write_txn, CHILDREN, &parent_key, &children_set)?;
        }

        // Update genesis if this is a root vertex
        let session_key = Self::session_key(session_id);
        if vertex.is_genesis() {
            let mut genesis_set = self.read_vertex_set(GENESIS, &session_key)?;
            genesis_set.insert(vertex_id);
            Self::write_vertex_set(&write_txn, GENESIS, &session_key, &genesis_set)?;
        }

        // Update frontier: remove parents, add this vertex
        let mut frontier = self.read_vertex_set(FRONTIERS, &session_key)?;
        for parent_id in &vertex.parents {
            frontier.remove(parent_id);
        }
        frontier.insert(vertex_id);
        Self::write_vertex_set(&write_txn, FRONTIERS, &session_key, &frontier)?;

        write_txn.commit().map_err(|e| {
            RhizoCryptError::storage(format!("Failed to commit write transaction: {e}"))
        })?;

        Ok(())
    }

    async fn get_vertex(
        &self,
        session_id: SessionId,
        vertex_id: VertexId,
    ) -> Result<Option<Vertex>> {
        self.read_ops.fetch_add(1, Ordering::Relaxed);

        let read_txn = self
            .db
            .begin_read()
            .map_err(|e| RhizoCryptError::storage(format!("Failed to begin read: {e}")))?;
        let vertices_table = read_txn
            .open_table(VERTICES)
            .map_err(|e| RhizoCryptError::storage(format!("Failed to open vertices table: {e}")))?;

        let key = Self::vertex_key(session_id, vertex_id);
        let value = vertices_table
            .get(key.as_slice())
            .map_err(|e| RhizoCryptError::storage(e.to_string()))?;

        match value {
            Some(guard) => {
                let data = guard.value();
                let vertex = Vertex::from_cbor_bytes(data)?;
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

        let read_txn = self
            .db
            .begin_read()
            .map_err(|e| RhizoCryptError::storage(format!("Failed to begin read: {e}")))?;
        let vertices_table = read_txn
            .open_table(VERTICES)
            .map_err(|e| RhizoCryptError::storage(format!("Failed to open vertices table: {e}")))?;

        let results: Vec<Option<Vertex>> = vertex_ids
            .iter()
            .map(|id| {
                let key = Self::vertex_key(session_id, *id);
                vertices_table
                    .get(key.as_slice())
                    .ok()
                    .flatten()
                    .and_then(|g| Vertex::from_cbor_bytes(g.value()).ok())
            })
            .collect();

        Ok(results)
    }

    async fn exists(&self, session_id: SessionId, vertex_id: VertexId) -> Result<bool> {
        self.read_ops.fetch_add(1, Ordering::Relaxed);

        let read_txn = self
            .db
            .begin_read()
            .map_err(|e| RhizoCryptError::storage(format!("Failed to begin read: {e}")))?;
        let vertices_table = read_txn
            .open_table(VERTICES)
            .map_err(|e| RhizoCryptError::storage(format!("Failed to open vertices table: {e}")))?;

        let key = Self::vertex_key(session_id, vertex_id);
        let exists = vertices_table
            .get(key.as_slice())
            .map_err(|e| RhizoCryptError::storage(e.to_string()))?
            .is_some();

        Ok(exists)
    }

    async fn get_children(
        &self,
        session_id: SessionId,
        parent_id: VertexId,
    ) -> Result<Vec<VertexId>> {
        self.read_ops.fetch_add(1, Ordering::Relaxed);

        let read_txn = self
            .db
            .begin_read()
            .map_err(|e| RhizoCryptError::storage(format!("Failed to begin read: {e}")))?;
        let children_table = read_txn
            .open_table(CHILDREN)
            .map_err(|e| RhizoCryptError::storage(format!("Failed to open children table: {e}")))?;

        let key = Self::vertex_key(session_id, parent_id);
        let value = children_table
            .get(key.as_slice())
            .map_err(|e| RhizoCryptError::storage(e.to_string()))?;

        let children =
            value.as_ref().map(|g| Self::parse_vertex_set(g.value())).unwrap_or_default();
        Ok(children.into_iter().collect())
    }

    async fn get_genesis(&self, session_id: SessionId) -> Result<Vec<VertexId>> {
        self.read_ops.fetch_add(1, Ordering::Relaxed);

        let read_txn = self
            .db
            .begin_read()
            .map_err(|e| RhizoCryptError::storage(format!("Failed to begin read: {e}")))?;
        let genesis_table = read_txn
            .open_table(GENESIS)
            .map_err(|e| RhizoCryptError::storage(format!("Failed to open genesis table: {e}")))?;

        let key = Self::session_key(session_id);
        let value = genesis_table
            .get(key.as_slice())
            .map_err(|e| RhizoCryptError::storage(e.to_string()))?;

        let genesis = value.as_ref().map(|g| Self::parse_vertex_set(g.value())).unwrap_or_default();
        Ok(genesis.into_iter().collect())
    }

    async fn get_frontier(&self, session_id: SessionId) -> Result<Vec<VertexId>> {
        self.read_ops.fetch_add(1, Ordering::Relaxed);

        let read_txn = self
            .db
            .begin_read()
            .map_err(|e| RhizoCryptError::storage(format!("Failed to begin read: {e}")))?;
        let frontiers_table = read_txn.open_table(FRONTIERS).map_err(|e| {
            RhizoCryptError::storage(format!("Failed to open frontiers table: {e}"))
        })?;

        let key = Self::session_key(session_id);
        let value = frontiers_table
            .get(key.as_slice())
            .map_err(|e| RhizoCryptError::storage(e.to_string()))?;

        let frontier =
            value.as_ref().map(|g| Self::parse_vertex_set(g.value())).unwrap_or_default();
        Ok(frontier.into_iter().collect())
    }

    async fn count_vertices(&self, session_id: SessionId) -> Result<u64> {
        self.read_ops.fetch_add(1, Ordering::Relaxed);

        let read_txn = self
            .db
            .begin_read()
            .map_err(|e| RhizoCryptError::storage(format!("Failed to begin read: {e}")))?;
        let vertices_table = read_txn
            .open_table(VERTICES)
            .map_err(|e| RhizoCryptError::storage(format!("Failed to open vertices table: {e}")))?;

        let (start, end) = Self::session_prefix_range(session_id);
        let range = vertices_table
            .range(start.as_slice()..end.as_slice())
            .map_err(|e| RhizoCryptError::storage(e.to_string()))?;

        let count = range.count();
        Ok(count as u64)
    }

    async fn delete_session(&self, session_id: SessionId) -> Result<()> {
        self.write_ops.fetch_add(1, Ordering::Relaxed);

        let (prefix_start, prefix_end) = Self::session_prefix_range(session_id);
        let session_key = Self::session_key(session_id);

        // Collect keys to delete (read phase) - avoid mutating while iterating
        let (vertices_keys, children_keys): (Vec<Vec<u8>>, Vec<Vec<u8>>) = {
            let read_txn = self
                .db
                .begin_read()
                .map_err(|e| RhizoCryptError::storage(format!("Failed to begin read: {e}")))?;
            let vertices_table = read_txn.open_table(VERTICES).map_err(|e| {
                RhizoCryptError::storage(format!("Failed to open vertices table: {e}"))
            })?;
            let children_table = read_txn.open_table(CHILDREN).map_err(|e| {
                RhizoCryptError::storage(format!("Failed to open children table: {e}"))
            })?;
            let vertices_keys = vertices_table
                .range(prefix_start.as_slice()..prefix_end.as_slice())
                .map_err(|e| RhizoCryptError::storage(e.to_string()))?
                .map(|r| {
                    r.map_err(|e| RhizoCryptError::storage(e.to_string()))
                        .map(|(k, _)| k.value().to_vec())
                })
                .collect::<Result<Vec<_>>>()?;
            let children_keys = children_table
                .range(prefix_start.as_slice()..prefix_end.as_slice())
                .map_err(|e| RhizoCryptError::storage(e.to_string()))?
                .map(|r| {
                    r.map_err(|e| RhizoCryptError::storage(e.to_string()))
                        .map(|(k, _)| k.value().to_vec())
                })
                .collect::<Result<Vec<_>>>()?;
            (vertices_keys, children_keys)
        };

        let write_txn = self.db.begin_write().map_err(|e| {
            RhizoCryptError::storage(format!("Failed to begin write transaction: {e}"))
        })?;

        {
            let mut vertices_table = write_txn.open_table(VERTICES).map_err(|e| {
                RhizoCryptError::storage(format!("Failed to open vertices table: {e}"))
            })?;
            for key in vertices_keys {
                vertices_table
                    .remove(key.as_slice())
                    .map_err(|e| RhizoCryptError::storage(e.to_string()))?;
            }
        }

        {
            let mut children_table = write_txn.open_table(CHILDREN).map_err(|e| {
                RhizoCryptError::storage(format!("Failed to open children table: {e}"))
            })?;
            for key in children_keys {
                children_table
                    .remove(key.as_slice())
                    .map_err(|e| RhizoCryptError::storage(e.to_string()))?;
            }
        }

        {
            let mut frontiers_table = write_txn.open_table(FRONTIERS).map_err(|e| {
                RhizoCryptError::storage(format!("Failed to open frontiers table: {e}"))
            })?;
            frontiers_table
                .remove(session_key.as_slice())
                .map_err(|e| RhizoCryptError::storage(e.to_string()))?;
        }

        {
            let mut genesis_table = write_txn.open_table(GENESIS).map_err(|e| {
                RhizoCryptError::storage(format!("Failed to open genesis table: {e}"))
            })?;
            genesis_table
                .remove(session_key.as_slice())
                .map_err(|e| RhizoCryptError::storage(e.to_string()))?;
        }

        {
            let mut metadata_table = write_txn.open_table(METADATA).map_err(|e| {
                RhizoCryptError::storage(format!("Failed to open metadata table: {e}"))
            })?;
            metadata_table
                .remove(session_key.as_slice())
                .map_err(|e| RhizoCryptError::storage(e.to_string()))?;
        }

        write_txn.commit().map_err(|e| {
            RhizoCryptError::storage(format!("Failed to commit write transaction: {e}"))
        })?;

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

        let existing: Option<Vec<u8>> = {
            let read_txn = self
                .db
                .begin_read()
                .map_err(|e| RhizoCryptError::storage(format!("Failed to begin read: {e}")))?;
            let frontiers_table = read_txn.open_table(FRONTIERS).map_err(|e| {
                RhizoCryptError::storage(format!("Failed to open frontiers table: {e}"))
            })?;
            frontiers_table
                .get(key.as_slice())
                .map_err(|e| RhizoCryptError::storage(e.to_string()))?
                .map(|g| g.value().to_vec())
        };

        let mut frontier = existing.as_deref().map(Self::parse_vertex_set).unwrap_or_default();
        for parent in consumed_parents {
            frontier.remove(parent);
        }
        frontier.insert(new_vertex);

        let write_txn = self.db.begin_write().map_err(|e| {
            RhizoCryptError::storage(format!("Failed to begin write transaction: {e}"))
        })?;
        {
            let mut frontiers_table = write_txn.open_table(FRONTIERS).map_err(|e| {
                RhizoCryptError::storage(format!("Failed to open frontiers table: {e}"))
            })?;
            frontiers_table
                .insert(key.as_slice(), Self::serialize_vertex_set(&frontier).as_slice())
                .map_err(|e| RhizoCryptError::storage(e.to_string()))?;
        }
        write_txn.commit().map_err(|e| {
            RhizoCryptError::storage(format!("Failed to commit write transaction: {e}"))
        })?;

        Ok(())
    }

    async fn health(&self) -> StorageHealth {
        StorageHealth::Healthy
    }

    async fn stats(&self) -> StorageStats {
        let read_txn = self.db.begin_read();
        let (session_count, vertex_count, bytes_used) = read_txn.map_or((0, 0, 0), |txn| {
            let genesis_table = txn.open_table(GENESIS);
            let vertices_table = txn.open_table(VERTICES);
            match (genesis_table, vertices_table) {
                (Ok(genesis), Ok(vertices)) => {
                    let session_count = genesis.len().unwrap_or(0);
                    let vertex_count = vertices.len().unwrap_or(0);
                    let bytes_used = self.path.metadata().map(|m| m.len()).unwrap_or(0);
                    (session_count, vertex_count, bytes_used)
                }
                _ => (0, 0, 0),
            }
        });

        StorageStats {
            sessions: session_count,
            vertices: vertex_count,
            bytes_used,
            read_ops: self.read_ops.load(Ordering::Relaxed),
            write_ops: self.write_ops.load(Ordering::Relaxed),
        }
    }
}

impl std::fmt::Debug for RedbDagStore {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RedbDagStore")
            .field("path", &*self.path)
            .field("read_ops", &self.read_ops.load(Ordering::Relaxed))
            .field("write_ops", &self.write_ops.load(Ordering::Relaxed))
            .finish_non_exhaustive()
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
#[path = "store_redb_tests.rs"]
mod tests;

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
#[path = "store_redb_tests_advanced.rs"]
mod tests_advanced;

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
#[path = "store_redb_tests_query.rs"]
mod tests_query;
