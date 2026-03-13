// SPDX-License-Identifier: AGPL-3.0-only
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
//! ```rust,ignore
//! use rhizo_crypt_core::RedbDagStore;
//!
//! let store = RedbDagStore::open("/path/to/db")?;
//! store.put_vertex(session_id, vertex).await?;
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
}

impl DagStore for RedbDagStore {
    #[allow(clippy::too_many_lines)]
    async fn put_vertex(&self, session_id: SessionId, mut vertex: Vertex) -> Result<()> {
        self.write_ops.fetch_add(1, Ordering::Relaxed);

        let vertex_id = vertex
            .id()
            .map_err(|e| RhizoCryptError::storage(format!("Failed to compute vertex ID: {e}")))?;
        let value = vertex.to_canonical_bytes()?;

        let write_txn = self.db.begin_write().map_err(|e| {
            RhizoCryptError::storage(format!("Failed to begin write transaction: {e}"))
        })?;

        {
            let mut vertices_table = write_txn.open_table(VERTICES).map_err(|e| {
                RhizoCryptError::storage(format!("Failed to open vertices table: {e}"))
            })?;

            let key = Self::vertex_key(session_id, vertex_id);
            vertices_table
                .insert(key.as_slice(), value.as_slice())
                .map_err(|e| RhizoCryptError::storage(e.to_string()))?;
        }

        // Update children index for each parent
        for parent_id in &vertex.parents {
            let parent_key = Self::vertex_key(session_id, *parent_id);

            let existing: Option<Vec<u8>> = {
                let read_txn = self
                    .db
                    .begin_read()
                    .map_err(|e| RhizoCryptError::storage(format!("Failed to begin read: {e}")))?;
                let children_table = read_txn.open_table(CHILDREN).map_err(|e| {
                    RhizoCryptError::storage(format!("Failed to open children table: {e}"))
                })?;
                children_table
                    .get(parent_key.as_slice())
                    .map_err(|e| RhizoCryptError::storage(e.to_string()))?
                    .map(|g| g.value().to_vec())
            };

            let mut children_set =
                existing.as_deref().map(Self::parse_vertex_set).unwrap_or_default();
            children_set.insert(vertex_id);

            let mut children_table = write_txn.open_table(CHILDREN).map_err(|e| {
                RhizoCryptError::storage(format!("Failed to open children table: {e}"))
            })?;
            children_table
                .insert(parent_key.as_slice(), Self::serialize_vertex_set(&children_set).as_slice())
                .map_err(|e| RhizoCryptError::storage(e.to_string()))?;
        }

        // Update genesis if this is a root vertex
        if vertex.is_genesis() {
            let session_key = Self::session_key(session_id);

            let existing: Option<Vec<u8>> = {
                let read_txn = self
                    .db
                    .begin_read()
                    .map_err(|e| RhizoCryptError::storage(format!("Failed to begin read: {e}")))?;
                let genesis_table = read_txn.open_table(GENESIS).map_err(|e| {
                    RhizoCryptError::storage(format!("Failed to open genesis table: {e}"))
                })?;
                genesis_table
                    .get(session_key.as_slice())
                    .map_err(|e| RhizoCryptError::storage(e.to_string()))?
                    .map(|g| g.value().to_vec())
            };

            let mut genesis_set =
                existing.as_deref().map(Self::parse_vertex_set).unwrap_or_default();
            genesis_set.insert(vertex_id);

            let mut genesis_table = write_txn.open_table(GENESIS).map_err(|e| {
                RhizoCryptError::storage(format!("Failed to open genesis table: {e}"))
            })?;
            genesis_table
                .insert(session_key.as_slice(), Self::serialize_vertex_set(&genesis_set).as_slice())
                .map_err(|e| RhizoCryptError::storage(e.to_string()))?;
        }

        // Update frontier
        let session_key = Self::session_key(session_id);

        let existing: Option<Vec<u8>> = {
            let read_txn = self
                .db
                .begin_read()
                .map_err(|e| RhizoCryptError::storage(format!("Failed to begin read: {e}")))?;
            let frontiers_table = read_txn.open_table(FRONTIERS).map_err(|e| {
                RhizoCryptError::storage(format!("Failed to open frontiers table: {e}"))
            })?;
            frontiers_table
                .get(session_key.as_slice())
                .map_err(|e| RhizoCryptError::storage(e.to_string()))?
                .map(|g| g.value().to_vec())
        };

        let mut frontier = existing.as_deref().map(Self::parse_vertex_set).unwrap_or_default();
        for parent_id in &vertex.parents {
            frontier.remove(parent_id);
        }
        frontier.insert(vertex_id);

        {
            let mut frontiers_table = write_txn.open_table(FRONTIERS).map_err(|e| {
                RhizoCryptError::storage(format!("Failed to open frontiers table: {e}"))
            })?;
            frontiers_table
                .insert(session_key.as_slice(), Self::serialize_vertex_set(&frontier).as_slice())
                .map_err(|e| RhizoCryptError::storage(e.to_string()))?;
        }

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

#[allow(clippy::missing_fields_in_debug)]
impl std::fmt::Debug for RedbDagStore {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RedbDagStore")
            .field("path", &*self.path)
            .field("read_ops", &self.read_ops.load(Ordering::Relaxed))
            .field("write_ops", &self.write_ops.load(Ordering::Relaxed))
            .finish()
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;
    use crate::event::EventType;
    use crate::vertex::VertexBuilder;
    use tempfile::TempDir;

    fn create_test_store() -> (RedbDagStore, TempDir) {
        let dir = TempDir::new().expect("Failed to create temp dir");
        let db_path = dir.path().join("db.redb");
        let store = RedbDagStore::open(&db_path).expect("Failed to open store");
        (store, dir)
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn test_put_and_get_vertex() {
        let (store, _dir) = create_test_store();
        let session_id = SessionId::now();

        let vertex = VertexBuilder::new(EventType::SessionStart).build();
        let mut vertex_clone = vertex.clone();
        let vertex_id = vertex_clone.id().unwrap();

        store.put_vertex(session_id, vertex).await.unwrap();

        let retrieved = store.get_vertex(session_id, vertex_id).await.unwrap();
        assert!(retrieved.is_some());

        let mut retrieved_vertex = retrieved.unwrap();
        assert_eq!(retrieved_vertex.id().unwrap(), vertex_id);
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn test_genesis_and_frontier() {
        let (store, _dir) = create_test_store();
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
    async fn test_children() {
        let (store, _dir) = create_test_store();
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
    async fn test_delete_session() {
        let (store, _dir) = create_test_store();
        let session_id = SessionId::now();

        for _ in 0..5 {
            let vertex = VertexBuilder::new(EventType::SessionStart).build();
            store.put_vertex(session_id, vertex).await.unwrap();
        }

        assert_eq!(store.count_vertices(session_id).await.unwrap(), 5);

        store.delete_session(session_id).await.unwrap();
        assert_eq!(store.count_vertices(session_id).await.unwrap(), 0);
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn test_health_and_stats() {
        let (store, _dir) = create_test_store();
        let session_id = SessionId::now();

        assert!(matches!(store.health().await, StorageHealth::Healthy));

        for _ in 0..3 {
            let vertex = VertexBuilder::new(EventType::SessionStart).build();
            store.put_vertex(session_id, vertex).await.unwrap();
        }

        let stats = store.stats().await;
        assert!(stats.vertices >= 3);
        assert!(stats.write_ops >= 3);
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn test_persistence() {
        let dir = TempDir::new().expect("Failed to create temp dir");
        let db_path = dir.path().join("db.redb");
        let session_id = SessionId::now();
        let vertex_id;

        {
            let store = RedbDagStore::open(&db_path).unwrap();
            let vertex = VertexBuilder::new(EventType::SessionStart).build();
            let mut vertex_clone = vertex.clone();
            vertex_id = vertex_clone.id().unwrap();
            store.put_vertex(session_id, vertex).await.unwrap();
        }

        {
            let store = RedbDagStore::open(&db_path).unwrap();
            let retrieved = store.get_vertex(session_id, vertex_id).await.unwrap();
            assert!(retrieved.is_some());
        }
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn test_get_nonexistent_vertex() {
        let (store, _dir) = create_test_store();
        let session_id = SessionId::now();
        let nonexistent_id = VertexId::from_bytes(b"nonexistent vertex id");

        let retrieved = store.get_vertex(session_id, nonexistent_id).await.unwrap();
        assert!(retrieved.is_none());
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn test_get_vertices_batch() {
        let (store, _dir) = create_test_store();
        let session_id = SessionId::now();

        let v1 = VertexBuilder::new(EventType::SessionStart).build();
        let mut v1_clone = v1.clone();
        let v1_id = v1_clone.id().unwrap();
        store.put_vertex(session_id, v1).await.unwrap();

        let nonexistent_id = VertexId::from_bytes(b"missing vertex");
        let ids = vec![v1_id, nonexistent_id, v1_id];

        let results = store.get_vertices(session_id, &ids).await.unwrap();
        assert_eq!(results.len(), 3);
        assert!(results[0].is_some());
        assert!(results[1].is_none());
        assert!(results[2].is_some());
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn test_exists() {
        let (store, _dir) = create_test_store();
        let session_id = SessionId::now();

        let vertex = VertexBuilder::new(EventType::SessionStart).build();
        let mut vertex_clone = vertex.clone();
        let vertex_id = vertex_clone.id().unwrap();
        store.put_vertex(session_id, vertex).await.unwrap();

        assert!(store.exists(session_id, vertex_id).await.unwrap());

        let absent_id = VertexId::from_bytes(b"absent");
        assert!(!store.exists(session_id, absent_id).await.unwrap());
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn test_count_vertices() {
        let (store, _dir) = create_test_store();
        let session_id = SessionId::now();

        assert_eq!(store.count_vertices(session_id).await.unwrap(), 0);

        for _ in 0..5 {
            let vertex = VertexBuilder::new(EventType::SessionStart).build();
            store.put_vertex(session_id, vertex).await.unwrap();
        }
        assert_eq!(store.count_vertices(session_id).await.unwrap(), 5);

        let session_id2 = SessionId::now();
        assert_eq!(store.count_vertices(session_id2).await.unwrap(), 0);
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn test_update_frontier() {
        let (store, _dir) = create_test_store();
        let session_id = SessionId::now();

        let v1 = VertexBuilder::new(EventType::SessionStart).build();
        let mut v1_clone = v1.clone();
        let v1_id = v1_clone.id().unwrap();
        store.put_vertex(session_id, v1).await.unwrap();

        let new_id = VertexId::from_bytes(b"new frontier vertex id 32 bytes!!");
        store.update_frontier(session_id, new_id, &[v1_id]).await.unwrap();

        let frontier = store.get_frontier(session_id).await.unwrap();
        assert!(frontier.contains(&new_id));
        assert!(!frontier.contains(&v1_id));
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn test_multiple_sessions() {
        let (store, _dir) = create_test_store();
        let session1 = SessionId::now();
        let session2 = SessionId::now();

        let v1 = VertexBuilder::new(EventType::SessionStart).build();
        let mut v1_clone = v1.clone();
        let v1_id = v1_clone.id().unwrap();
        store.put_vertex(session1, v1).await.unwrap();

        let v2 = VertexBuilder::new(EventType::DataCreate {
            schema: None,
        })
        .build();
        let mut v2_clone = v2.clone();
        let v2_id = v2_clone.id().unwrap();
        store.put_vertex(session2, v2).await.unwrap();

        assert_eq!(store.count_vertices(session1).await.unwrap(), 1);
        assert_eq!(store.count_vertices(session2).await.unwrap(), 1);

        let got1 = store.get_vertex(session1, v1_id).await.unwrap();
        let got2 = store.get_vertex(session2, v2_id).await.unwrap();
        assert!(got1.is_some());
        assert!(got2.is_some());

        assert!(store.get_vertex(session1, v2_id).await.unwrap().is_none());
        assert!(store.get_vertex(session2, v1_id).await.unwrap().is_none());
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn test_concurrent_reads() {
        let (store, _dir) = create_test_store();
        let session_id = SessionId::now();

        let vertex = VertexBuilder::new(EventType::SessionStart).build();
        let mut vertex_clone = vertex.clone();
        let vertex_id = vertex_clone.id().unwrap();
        store.put_vertex(session_id, vertex).await.unwrap();

        let store_clone = store.clone();
        let handles: Vec<_> = (0..10)
            .map(|_| {
                let s = store_clone.clone();
                let sid = session_id;
                let vid = vertex_id;
                tokio::spawn(async move { s.get_vertex(sid, vid).await })
            })
            .collect();

        for h in handles {
            let result = h.await.unwrap().unwrap();
            assert!(result.is_some());
        }
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn test_large_batch() {
        let (store, _dir) = create_test_store();
        let session_id = SessionId::now();

        let mut ids = Vec::new();
        for i in 0..120 {
            let vertex = VertexBuilder::new(EventType::DataCreate {
                schema: Some(format!("schema_{i}")),
            })
            .build();
            let mut vertex_clone = vertex.clone();
            let vertex_id = vertex_clone.id().unwrap();
            ids.push(vertex_id);
            store.put_vertex(session_id, vertex).await.unwrap();
        }

        assert_eq!(store.count_vertices(session_id).await.unwrap(), 120);

        let results = store.get_vertices(session_id, &ids).await.unwrap();
        assert_eq!(results.len(), 120);
        assert!(results.iter().all(Option::is_some));
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn test_stats_accuracy() {
        let (store, _dir) = create_test_store();
        let session_id = SessionId::now();

        let stats_before = store.stats().await;
        assert_eq!(stats_before.read_ops, 0);
        assert_eq!(stats_before.write_ops, 0);

        for _ in 0..4 {
            let vertex = VertexBuilder::new(EventType::SessionStart).build();
            store.put_vertex(session_id, vertex).await.unwrap();
        }

        let _ = store.get_vertex(session_id, VertexId::from_bytes(b"x")).await;
        let _ = store.count_vertices(session_id).await;

        let stats = store.stats().await;
        assert!(stats.write_ops >= 4);
        assert!(stats.read_ops >= 2);
        assert!(stats.vertices >= 4);
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn test_get_frontier_empty() {
        let (store, _dir) = create_test_store();
        let session_id = SessionId::now();

        let frontier = store.get_frontier(session_id).await.unwrap();
        assert!(frontier.is_empty());
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn test_get_genesis_empty() {
        let (store, _dir) = create_test_store();
        let session_id = SessionId::now();

        let genesis = store.get_genesis(session_id).await.unwrap();
        assert!(genesis.is_empty());
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn test_count_vertices_multiple_sessions() {
        let (store, _dir) = create_test_store();
        let session1 = SessionId::now();
        let session2 = SessionId::now();

        for _ in 0..3 {
            let vertex = VertexBuilder::new(EventType::SessionStart).build();
            store.put_vertex(session1, vertex).await.unwrap();
        }
        for _ in 0..7 {
            let vertex = VertexBuilder::new(EventType::DataCreate {
                schema: None,
            })
            .build();
            store.put_vertex(session2, vertex).await.unwrap();
        }

        assert_eq!(store.count_vertices(session1).await.unwrap(), 3);
        assert_eq!(store.count_vertices(session2).await.unwrap(), 7);
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn test_session_count_via_stats() {
        let (store, _dir) = create_test_store();

        let stats_empty = store.stats().await;
        assert_eq!(stats_empty.sessions, 0);

        let session1 = SessionId::now();
        let session2 = SessionId::now();
        store
            .put_vertex(session1, VertexBuilder::new(EventType::SessionStart).build())
            .await
            .unwrap();
        store
            .put_vertex(session2, VertexBuilder::new(EventType::SessionStart).build())
            .await
            .unwrap();

        let stats = store.stats().await;
        assert_eq!(stats.sessions, 2);
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn test_delete_nonexistent_session() {
        let (store, _dir) = create_test_store();
        let session_id = SessionId::now();

        // Deleting a session that was never used should succeed (no-op)
        store.delete_session(session_id).await.unwrap();
        assert_eq!(store.count_vertices(session_id).await.unwrap(), 0);
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn test_storage_health() {
        let (store, _dir) = create_test_store();

        let health = store.health().await;
        assert!(matches!(health, StorageHealth::Healthy));
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn test_storage_stats_with_data() {
        let (store, _dir) = create_test_store();
        let session1 = SessionId::now();
        let session2 = SessionId::now();

        for i in 0..5 {
            let vertex = VertexBuilder::new(EventType::DataCreate {
                schema: Some(format!("schema_{i}")),
            })
            .build();
            store.put_vertex(session1, vertex).await.unwrap();
        }
        for i in 0..10 {
            let vertex = VertexBuilder::new(EventType::DataCreate {
                schema: Some(format!("other_{i}")),
            })
            .build();
            store.put_vertex(session2, vertex).await.unwrap();
        }

        let stats = store.stats().await;
        assert_eq!(stats.sessions, 2);
        assert_eq!(stats.vertices, 15);
        assert!(stats.bytes_used > 0);
        assert!(stats.write_ops >= 15);
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn test_path() {
        let dir = TempDir::new().expect("Failed to create temp dir");
        let db_path = dir.path().join("db.redb");
        let store = RedbDagStore::open(&db_path).expect("Failed to open store");

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
        let nested_path = dir.path().join("a").join("b").join("c").join("db.redb");
        assert!(!nested_path.parent().unwrap().exists());

        let store = RedbDagStore::open(&nested_path).expect("Failed to open store");
        assert!(store.path().parent().unwrap().exists());
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn test_vertex_with_parents_children_index() {
        let (store, _dir) = create_test_store();
        let session_id = SessionId::now();

        // Create genesis vertex (no parents)
        let genesis = VertexBuilder::new(EventType::SessionStart).build();
        let mut genesis_mut = genesis.clone();
        let genesis_id = genesis_mut.id().unwrap();
        store.put_vertex(session_id, genesis).await.unwrap();

        // Create first child referencing genesis as parent
        let child1 = VertexBuilder::new(EventType::DataCreate {
            schema: Some("schema1".to_string()),
        })
        .with_parent(genesis_id)
        .build();
        let mut child1_mut = child1.clone();
        let child1_id = child1_mut.id().unwrap();
        store.put_vertex(session_id, child1).await.unwrap();

        let children = store.get_children(session_id, genesis_id).await.unwrap();
        assert_eq!(children.len(), 1);
        assert!(children.contains(&child1_id));

        // Create second child to same parent
        let child2 = VertexBuilder::new(EventType::DataCreate {
            schema: Some("schema2".to_string()),
        })
        .with_parent(genesis_id)
        .build();
        let mut child2_mut = child2.clone();
        let child2_id = child2_mut.id().unwrap();
        store.put_vertex(session_id, child2).await.unwrap();

        let children = store.get_children(session_id, genesis_id).await.unwrap();
        assert_eq!(children.len(), 2);
        assert!(children.contains(&child1_id));
        assert!(children.contains(&child2_id));
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn test_complex_dag_structure() {
        let (store, _dir) = create_test_store();
        let session_id = SessionId::now();

        // Diamond DAG: genesis -> A, genesis -> B, A -> C, B -> C (merge)
        let genesis = VertexBuilder::new(EventType::SessionStart).build();
        let mut genesis_mut = genesis.clone();
        let genesis_id = genesis_mut.id().unwrap();
        store.put_vertex(session_id, genesis).await.unwrap();

        let a = VertexBuilder::new(EventType::DataCreate {
            schema: Some("A".to_string()),
        })
        .with_parent(genesis_id)
        .build();
        let mut a_mut = a.clone();
        let a_id = a_mut.id().unwrap();
        store.put_vertex(session_id, a).await.unwrap();

        let b = VertexBuilder::new(EventType::DataCreate {
            schema: Some("B".to_string()),
        })
        .with_parent(genesis_id)
        .build();
        let mut b_mut = b.clone();
        let b_id = b_mut.id().unwrap();
        store.put_vertex(session_id, b).await.unwrap();

        let c = VertexBuilder::new(EventType::DataCreate {
            schema: Some("C".to_string()),
        })
        .with_parents([a_id, b_id])
        .build();
        let mut c_mut = c.clone();
        let c_id = c_mut.id().unwrap();
        store.put_vertex(session_id, c).await.unwrap();

        // Verify frontier is [C]
        let frontier = store.get_frontier(session_id).await.unwrap();
        assert_eq!(frontier.len(), 1);
        assert!(frontier.contains(&c_id));

        // Verify genesis is [genesis]
        let genesis_set = store.get_genesis(session_id).await.unwrap();
        assert_eq!(genesis_set.len(), 1);
        assert!(genesis_set.contains(&genesis_id));

        // Verify children of genesis are [A, B]
        let genesis_children = store.get_children(session_id, genesis_id).await.unwrap();
        assert_eq!(genesis_children.len(), 2);
        assert!(genesis_children.contains(&a_id));
        assert!(genesis_children.contains(&b_id));

        // Verify children of A are [C]
        let a_children = store.get_children(session_id, a_id).await.unwrap();
        assert_eq!(a_children.len(), 1);
        assert!(a_children.contains(&c_id));

        // Verify children of B are [C]
        let b_children = store.get_children(session_id, b_id).await.unwrap();
        assert_eq!(b_children.len(), 1);
        assert!(b_children.contains(&c_id));
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn test_reopen_after_crash() {
        let dir = TempDir::new().expect("Failed to create temp dir");
        let db_path = dir.path().join("db.redb");
        let session_id = SessionId::now();

        let genesis_id;
        let child_id;

        {
            let store = RedbDagStore::open(&db_path).unwrap();
            let genesis = VertexBuilder::new(EventType::SessionStart).build();
            let mut genesis_mut = genesis.clone();
            genesis_id = genesis_mut.id().unwrap();
            store.put_vertex(session_id, genesis).await.unwrap();

            let child = VertexBuilder::new(EventType::DataCreate {
                schema: Some("child".to_string()),
            })
            .with_parent(genesis_id)
            .build();
            let mut child_mut = child.clone();
            child_id = child_mut.id().unwrap();
            store.put_vertex(session_id, child).await.unwrap();
        }

        // Simulate crash: drop store, reopen from same path
        {
            let store = RedbDagStore::open(&db_path).unwrap();

            // Verify vertices persisted
            let retrieved_genesis = store.get_vertex(session_id, genesis_id).await.unwrap();
            assert!(retrieved_genesis.is_some());

            let retrieved_child = store.get_vertex(session_id, child_id).await.unwrap();
            assert!(retrieved_child.is_some());

            // Verify children index persisted
            let children = store.get_children(session_id, genesis_id).await.unwrap();
            assert_eq!(children.len(), 1);
            assert!(children.contains(&child_id));

            // Verify frontier persisted
            let frontier = store.get_frontier(session_id).await.unwrap();
            assert_eq!(frontier.len(), 1);
            assert!(frontier.contains(&child_id));

            // Verify genesis persisted
            let genesis_set = store.get_genesis(session_id).await.unwrap();
            assert_eq!(genesis_set.len(), 1);
            assert!(genesis_set.contains(&genesis_id));
        }
    }
}
