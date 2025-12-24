//! RocksDB persistent storage backend.
//!
//! This module provides a durable storage implementation using RocksDB,
//! suitable for sessions that need to survive restarts.
//!
//! ## Features
//!
//! - **Persistent storage** — Data survives process restarts
//! - **Column families** — Separate namespaces for vertices, children, frontiers
//! - **Atomic writes** — WriteBatch for transactional updates
//! - **Compression** — LZ4 compression for space efficiency
//!
//! ## Usage
//!
//! ```rust,ignore
//! use rhizo_crypt_core::RocksDbDagStore;
//!
//! let store = RocksDbDagStore::open("/path/to/db")?;
//! store.put_vertex(session_id, vertex).await?;
//! ```
//!
//! ## Column Families
//!
//! | Family | Key | Value |
//! |--------|-----|-------|
//! | `vertices` | `session_id:vertex_id` | CBOR-encoded Vertex |
//! | `children` | `session_id:parent_id` | Set of child vertex IDs |
//! | `frontiers` | `session_id` | Set of frontier vertex IDs |
//! | `genesis` | `session_id` | Set of genesis vertex IDs |
//! | `metadata` | `session_id` | Session metadata |

use crate::error::{Result, RhizoCryptError};
use crate::store::{DagStore, StorageHealth, StorageStats};
use crate::types::{SessionId, VertexId};
use crate::vertex::Vertex;

use rocksdb::{ColumnFamily, ColumnFamilyDescriptor, DBCompressionType, Options, WriteBatch, DB};
use std::path::Path;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

/// Column family names.
const CF_VERTICES: &str = "vertices";
const CF_CHILDREN: &str = "children";
const CF_FRONTIERS: &str = "frontiers";
const CF_GENESIS: &str = "genesis";
const CF_METADATA: &str = "metadata";

/// All column families for iteration.
const ALL_CFS: [&str; 5] = [CF_VERTICES, CF_CHILDREN, CF_FRONTIERS, CF_GENESIS, CF_METADATA];

/// RocksDB-backed DAG store.
///
/// Provides persistent storage for vertices with column family separation
/// for different data types.
pub struct RocksDbDagStore {
    /// RocksDB instance.
    db: Arc<DB>,
    /// Read operations counter.
    read_ops: AtomicU64,
    /// Write operations counter.
    write_ops: AtomicU64,
}

impl RocksDbDagStore {
    /// Open or create a RocksDB store at the given path.
    ///
    /// # Errors
    ///
    /// Returns an error if the database cannot be opened or created.
    pub fn open(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref();

        // Create directory if it doesn't exist
        std::fs::create_dir_all(path).map_err(|e| {
            RhizoCryptError::storage(format!("Failed to create database directory: {e}"))
        })?;

        // Configure RocksDB options
        let mut opts = Options::default();
        opts.create_if_missing(true);
        opts.create_missing_column_families(true);
        opts.set_compression_type(DBCompressionType::Lz4);
        opts.set_max_background_jobs(4);
        opts.set_max_open_files(256);
        opts.set_write_buffer_size(64 * 1024 * 1024); // 64MB write buffer
        opts.set_target_file_size_base(64 * 1024 * 1024); // 64MB SST files

        // Column family options (same for all)
        let cf_opts = Options::default();

        // Create column family descriptors
        let cfs: Vec<ColumnFamilyDescriptor> = ALL_CFS
            .iter()
            .map(|name| ColumnFamilyDescriptor::new(*name, cf_opts.clone()))
            .collect();

        // Open database
        let db = DB::open_cf_descriptors(&opts, path, cfs)
            .map_err(|e| RhizoCryptError::storage(format!("Failed to open RocksDB: {e}")))?;

        Ok(Self {
            db: Arc::new(db),
            read_ops: AtomicU64::new(0),
            write_ops: AtomicU64::new(0),
        })
    }

    /// Get a column family handle.
    fn cf(&self, name: &str) -> Result<&ColumnFamily> {
        self.db
            .cf_handle(name)
            .ok_or_else(|| RhizoCryptError::storage(format!("Missing column family: {name}")))
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
            .filter_map(|chunk| {
                let mut arr = [0u8; 32];
                arr.copy_from_slice(chunk);
                Some(VertexId::new(arr))
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
        self.db.path()
    }

    /// Flush all pending writes to disk.
    ///
    /// # Errors
    ///
    /// Returns an error if the flush fails.
    pub fn flush(&self) -> Result<()> {
        for cf_name in ALL_CFS {
            if let Ok(cf) = self.cf(cf_name) {
                self.db.flush_cf(cf).map_err(|e| {
                    RhizoCryptError::storage(format!("Failed to flush {cf_name}: {e}"))
                })?;
            }
        }
        Ok(())
    }

    /// Compact the database to reclaim space.
    ///
    /// # Errors
    ///
    /// Returns an error if compaction fails.
    pub fn compact(&self) -> Result<()> {
        for cf_name in ALL_CFS {
            if let Ok(cf) = self.cf(cf_name) {
                self.db.compact_range_cf(cf, None::<&[u8]>, None::<&[u8]>);
            }
        }
        Ok(())
    }
}

impl DagStore for RocksDbDagStore {
    async fn put_vertex(&self, session_id: SessionId, mut vertex: Vertex) -> Result<()> {
        self.write_ops.fetch_add(1, Ordering::Relaxed);

        let vertex_id = vertex.id();
        let key = Self::vertex_key(session_id, vertex_id);

        // Serialize vertex to CBOR
        let value = vertex.to_canonical_bytes()?;

        let vertices_cf = self.cf(CF_VERTICES)?;
        let children_cf = self.cf(CF_CHILDREN)?;
        let frontiers_cf = self.cf(CF_FRONTIERS)?;
        let genesis_cf = self.cf(CF_GENESIS)?;

        // Use WriteBatch for atomic updates
        let mut batch = WriteBatch::default();

        // Store vertex
        batch.put_cf(vertices_cf, &key, &value);

        // Update children index for each parent
        for parent_id in &vertex.parents {
            let parent_key = Self::vertex_key(session_id, *parent_id);

            // Read existing children
            let existing = self
                .db
                .get_cf(children_cf, &parent_key)
                .map_err(|e| RhizoCryptError::storage(e.to_string()))?;

            let mut children = existing.as_deref().map(Self::parse_vertex_set).unwrap_or_default();

            children.insert(vertex_id);

            batch.put_cf(children_cf, &parent_key, Self::serialize_vertex_set(&children));
        }

        // Update genesis if this is a root vertex
        if vertex.is_genesis() {
            let session_key = Self::session_key(session_id);

            let existing = self
                .db
                .get_cf(genesis_cf, &session_key)
                .map_err(|e| RhizoCryptError::storage(e.to_string()))?;

            let mut genesis = existing.as_deref().map(Self::parse_vertex_set).unwrap_or_default();

            genesis.insert(vertex_id);

            batch.put_cf(genesis_cf, &session_key, Self::serialize_vertex_set(&genesis));
        }

        // Update frontier
        let session_key = Self::session_key(session_id);

        let existing = self
            .db
            .get_cf(frontiers_cf, &session_key)
            .map_err(|e| RhizoCryptError::storage(e.to_string()))?;

        let mut frontier = existing.as_deref().map(Self::parse_vertex_set).unwrap_or_default();

        // Remove parents from frontier
        for parent_id in &vertex.parents {
            frontier.remove(parent_id);
        }

        // Add this vertex to frontier
        frontier.insert(vertex_id);

        batch.put_cf(frontiers_cf, &session_key, Self::serialize_vertex_set(&frontier));

        // Write batch atomically
        self.db.write(batch).map_err(|e| RhizoCryptError::storage(e.to_string()))?;

        Ok(())
    }

    async fn get_vertex(
        &self,
        session_id: SessionId,
        vertex_id: VertexId,
    ) -> Result<Option<Vertex>> {
        self.read_ops.fetch_add(1, Ordering::Relaxed);

        let key = Self::vertex_key(session_id, vertex_id);
        let cf = self.cf(CF_VERTICES)?;

        let value =
            self.db.get_cf(cf, &key).map_err(|e| RhizoCryptError::storage(e.to_string()))?;

        match value {
            Some(data) => {
                let vertex = Vertex::from_canonical_bytes(&data)?;
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

        let cf = self.cf(CF_VERTICES)?;

        let keys: Vec<Vec<u8>> =
            vertex_ids.iter().map(|id| Self::vertex_key(session_id, *id)).collect();

        let results: Vec<Option<Vertex>> = keys
            .iter()
            .map(|key| {
                self.db
                    .get_cf(cf, key)
                    .ok()
                    .flatten()
                    .and_then(|data| Vertex::from_canonical_bytes(&data).ok())
            })
            .collect();

        Ok(results)
    }

    async fn exists(&self, session_id: SessionId, vertex_id: VertexId) -> Result<bool> {
        self.read_ops.fetch_add(1, Ordering::Relaxed);

        let key = Self::vertex_key(session_id, vertex_id);
        let cf = self.cf(CF_VERTICES)?;

        let exists = self
            .db
            .get_cf(cf, &key)
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

        let key = Self::vertex_key(session_id, parent_id);
        let cf = self.cf(CF_CHILDREN)?;

        let value =
            self.db.get_cf(cf, &key).map_err(|e| RhizoCryptError::storage(e.to_string()))?;

        let children = value.as_deref().map(Self::parse_vertex_set).unwrap_or_default();

        Ok(children.into_iter().collect())
    }

    async fn get_genesis(&self, session_id: SessionId) -> Result<Vec<VertexId>> {
        self.read_ops.fetch_add(1, Ordering::Relaxed);

        let key = Self::session_key(session_id);
        let cf = self.cf(CF_GENESIS)?;

        let value =
            self.db.get_cf(cf, &key).map_err(|e| RhizoCryptError::storage(e.to_string()))?;

        let genesis = value.as_deref().map(Self::parse_vertex_set).unwrap_or_default();

        Ok(genesis.into_iter().collect())
    }

    async fn get_frontier(&self, session_id: SessionId) -> Result<Vec<VertexId>> {
        self.read_ops.fetch_add(1, Ordering::Relaxed);

        let key = Self::session_key(session_id);
        let cf = self.cf(CF_FRONTIERS)?;

        let value =
            self.db.get_cf(cf, &key).map_err(|e| RhizoCryptError::storage(e.to_string()))?;

        let frontier = value.as_deref().map(Self::parse_vertex_set).unwrap_or_default();

        Ok(frontier.into_iter().collect())
    }

    async fn count_vertices(&self, session_id: SessionId) -> Result<u64> {
        self.read_ops.fetch_add(1, Ordering::Relaxed);

        let cf = self.cf(CF_VERTICES)?;
        let prefix = Self::session_key(session_id);

        let mut count = 0u64;
        let iter = self.db.prefix_iterator_cf(cf, &prefix);

        for item in iter {
            let (key, _) = item.map_err(|e| RhizoCryptError::storage(e.to_string()))?;

            // Check if key starts with session prefix
            if !key.starts_with(&prefix) {
                break;
            }

            count += 1;
        }

        Ok(count)
    }

    async fn delete_session(&self, session_id: SessionId) -> Result<()> {
        self.write_ops.fetch_add(1, Ordering::Relaxed);

        let session_prefix = Self::session_key(session_id);

        let mut batch = WriteBatch::default();

        // Delete from all column families
        for cf_name in ALL_CFS {
            let cf = self.cf(cf_name)?;

            // For vertices and children, we need to iterate by prefix
            if cf_name == CF_VERTICES || cf_name == CF_CHILDREN {
                let iter = self.db.prefix_iterator_cf(cf, &session_prefix);

                for item in iter {
                    let (key, _) = item.map_err(|e| RhizoCryptError::storage(e.to_string()))?;

                    if !key.starts_with(&session_prefix) {
                        break;
                    }

                    batch.delete_cf(cf, &key);
                }
            } else {
                // For other families, just delete the session key
                batch.delete_cf(cf, &session_prefix);
            }
        }

        self.db.write(batch).map_err(|e| RhizoCryptError::storage(e.to_string()))?;

        Ok(())
    }

    async fn update_frontier(
        &self,
        session_id: SessionId,
        new_vertex: VertexId,
        consumed_parents: &[VertexId],
    ) -> Result<()> {
        self.write_ops.fetch_add(1, Ordering::Relaxed);

        let cf = self.cf(CF_FRONTIERS)?;
        let key = Self::session_key(session_id);

        let existing =
            self.db.get_cf(cf, &key).map_err(|e| RhizoCryptError::storage(e.to_string()))?;

        let mut frontier = existing.as_deref().map(Self::parse_vertex_set).unwrap_or_default();

        // Remove consumed parents
        for parent in consumed_parents {
            frontier.remove(parent);
        }

        // Add new vertex
        frontier.insert(new_vertex);

        self.db
            .put_cf(cf, &key, Self::serialize_vertex_set(&frontier))
            .map_err(|e| RhizoCryptError::storage(e.to_string()))?;

        Ok(())
    }

    async fn health(&self) -> StorageHealth {
        // Check if we can perform a simple operation
        match self.cf(CF_METADATA) {
            Ok(_) => StorageHealth::Healthy,
            Err(e) => StorageHealth::Unhealthy(e.to_string()),
        }
    }

    async fn stats(&self) -> StorageStats {
        // Count sessions from genesis CF
        let session_count = self
            .cf(CF_GENESIS)
            .map(|cf| {
                let iter = self.db.iterator_cf(cf, rocksdb::IteratorMode::Start);
                iter.count() as u64
            })
            .unwrap_or(0);

        // Estimate vertex count (expensive for large DBs)
        let vertex_count = self
            .cf(CF_VERTICES)
            .map(|cf| {
                let iter = self.db.iterator_cf(cf, rocksdb::IteratorMode::Start);
                iter.count() as u64
            })
            .unwrap_or(0);

        // Get disk usage from RocksDB properties
        let bytes_used = self
            .db
            .property_int_value("rocksdb.estimate-live-data-size")
            .ok()
            .flatten()
            .unwrap_or(0);

        StorageStats {
            sessions: session_count,
            vertices: vertex_count,
            bytes_used,
            read_ops: self.read_ops.load(Ordering::Relaxed),
            write_ops: self.write_ops.load(Ordering::Relaxed),
        }
    }
}

impl std::fmt::Debug for RocksDbDagStore {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RocksDbDagStore")
            .field("path", &self.db.path())
            .field("read_ops", &self.read_ops.load(Ordering::Relaxed))
            .field("write_ops", &self.write_ops.load(Ordering::Relaxed))
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::event::EventType;
    use crate::vertex::VertexBuilder;
    use tempfile::TempDir;

    fn create_test_store() -> (RocksDbDagStore, TempDir) {
        let dir = TempDir::new().expect("Failed to create temp dir");
        let store = RocksDbDagStore::open(dir.path()).expect("Failed to open store");
        (store, dir)
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn test_put_and_get_vertex() {
        let (store, _dir) = create_test_store();
        let session_id = SessionId::now();

        let vertex = VertexBuilder::new(EventType::SessionStart).build();
        let mut vertex_clone = vertex.clone();
        let vertex_id = vertex_clone.id();

        store.put_vertex(session_id, vertex).await.unwrap();

        let retrieved = store.get_vertex(session_id, vertex_id).await.unwrap();
        assert!(retrieved.is_some());

        let retrieved_vertex = retrieved.unwrap();
        assert_eq!(retrieved_vertex.id(), vertex_id);
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn test_genesis_and_frontier() {
        let (store, _dir) = create_test_store();
        let session_id = SessionId::now();

        // Add genesis vertex
        let v1 = VertexBuilder::new(EventType::SessionStart).build();
        let mut v1_clone = v1.clone();
        let v1_id = v1_clone.id();
        store.put_vertex(session_id, v1).await.unwrap();

        // Check genesis
        let genesis = store.get_genesis(session_id).await.unwrap();
        assert_eq!(genesis.len(), 1);
        assert!(genesis.contains(&v1_id));

        // Check frontier
        let frontier = store.get_frontier(session_id).await.unwrap();
        assert_eq!(frontier.len(), 1);
        assert!(frontier.contains(&v1_id));

        // Add child vertex
        let v2 = VertexBuilder::new(EventType::DataCreate {
            schema: None,
        })
        .with_parent(v1_id)
        .build();
        let mut v2_clone = v2.clone();
        let v2_id = v2_clone.id();
        store.put_vertex(session_id, v2).await.unwrap();

        // Frontier should now be v2 only
        let frontier = store.get_frontier(session_id).await.unwrap();
        assert_eq!(frontier.len(), 1);
        assert!(frontier.contains(&v2_id));
        assert!(!frontier.contains(&v1_id));

        // Genesis should still be v1
        let genesis = store.get_genesis(session_id).await.unwrap();
        assert!(genesis.contains(&v1_id));
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn test_children() {
        let (store, _dir) = create_test_store();
        let session_id = SessionId::now();

        // Add parent
        let parent = VertexBuilder::new(EventType::SessionStart).build();
        let mut parent_clone = parent.clone();
        let parent_id = parent_clone.id();
        store.put_vertex(session_id, parent).await.unwrap();

        // Add children
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

        // Add some vertices
        for _ in 0..5 {
            let vertex = VertexBuilder::new(EventType::SessionStart).build();
            store.put_vertex(session_id, vertex).await.unwrap();
        }

        assert_eq!(store.count_vertices(session_id).await.unwrap(), 5);

        // Delete session
        store.delete_session(session_id).await.unwrap();
        assert_eq!(store.count_vertices(session_id).await.unwrap(), 0);
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn test_health_and_stats() {
        let (store, _dir) = create_test_store();
        let session_id = SessionId::now();

        // Health should be healthy
        assert!(matches!(store.health().await, StorageHealth::Healthy));

        // Add some data
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
        let session_id = SessionId::now();
        let vertex_id;

        // Create store and add data
        {
            let store = RocksDbDagStore::open(dir.path()).unwrap();
            let vertex = VertexBuilder::new(EventType::SessionStart).build();
            let mut vertex_clone = vertex.clone();
            vertex_id = vertex_clone.id();
            store.put_vertex(session_id, vertex).await.unwrap();
            store.flush().unwrap();
        }

        // Reopen store and verify data persisted
        {
            let store = RocksDbDagStore::open(dir.path()).unwrap();
            let retrieved = store.get_vertex(session_id, vertex_id).await.unwrap();
            assert!(retrieved.is_some());
        }
    }
}
