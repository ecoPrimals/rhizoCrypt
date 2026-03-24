# RhizoCrypt — Storage Backends Specification

**Version**: 0.2.0  
**Status**: Draft  
**Last Updated**: March 2026

---

## 1. Overview

RhizoCrypt uses a pluggable storage architecture with Pure Rust backends:

| Backend | Use Case | Durability | Performance |
|---------|----------|------------|-------------|
| **In-Memory** | Short sessions, testing (default for tests) | None | Fastest |
| **redb** | General purpose, production (default) | Full | Fast |

All backends implement the `DagStore` trait and are dispatched at runtime via the `DagBackend` enum.

### 1.1 Design Rationale: Pure Rust Backends

RhizoCrypt targets **ecoBin compliance** — zero C dependencies. RocksDB and LMDB were considered in early design phases but never implemented because they require C/C++ bindings.

- **redb** — 100% Pure Rust embedded key-value store (default persistent backend, ACID, MVCC)
- **In-Memory** — Ephemeral storage for testing and short-lived sessions

The `DagStore` trait uses RPITIT (non-object-safe), so runtime dispatch uses the `DagBackend` enum rather than trait objects.

---

## 2. Storage Traits

### 2.1 DAG Store Trait

```rust
use async_trait::async_trait;
use bytes::Bytes;

/// Primary storage trait for DAG vertices
#[async_trait]
pub trait DagStore: Send + Sync + Clone {
    /// Store a vertex
    async fn put_vertex(
        &self,
        session_id: SessionId,
        vertex: &Vertex,
    ) -> Result<(), StorageError>;
    
    /// Get a vertex by ID
    async fn get_vertex(
        &self,
        session_id: SessionId,
        vertex_id: &VertexId,
    ) -> Result<Option<Vertex>, StorageError>;
    
    /// Get multiple vertices
    async fn get_vertices(
        &self,
        session_id: SessionId,
        vertex_ids: &[VertexId],
    ) -> Result<Vec<Option<Vertex>>, StorageError>;
    
    /// Check if vertex exists
    async fn exists(
        &self,
        session_id: SessionId,
        vertex_id: &VertexId,
    ) -> Result<bool, StorageError>;
    
    /// Get children of a vertex
    async fn get_children(
        &self,
        session_id: SessionId,
        parent_id: &VertexId,
    ) -> Result<Vec<VertexId>, StorageError>;
    
    /// Get genesis vertices for a session
    async fn get_genesis(
        &self,
        session_id: SessionId,
    ) -> Result<Vec<VertexId>, StorageError>;
    
    /// Get frontier vertices for a session
    async fn get_frontier(
        &self,
        session_id: SessionId,
    ) -> Result<Vec<VertexId>, StorageError>;
    
    /// Update frontier after vertex append
    async fn update_frontier(
        &self,
        session_id: SessionId,
        new_vertex: VertexId,
        consumed_parents: &[VertexId],
    ) -> Result<(), StorageError>;
    
    /// Count vertices in a session
    async fn count_vertices(
        &self,
        session_id: SessionId,
    ) -> Result<u64, StorageError>;
    
    /// Iterate all vertices in a session
    fn iter_session(
        &self,
        session_id: SessionId,
    ) -> impl Stream<Item = Result<Vertex, StorageError>> + Send;
    
    /// Delete all vertices in a session
    async fn delete_session(
        &self,
        session_id: SessionId,
    ) -> Result<u64, StorageError>;
    
    /// Health check
    async fn health(&self) -> HealthStatus;
    
    /// Get storage statistics
    async fn stats(&self) -> StorageStats;
}

/// Storage error
#[derive(Debug, thiserror::Error)]
pub enum StorageError {
    #[error("Vertex not found: {0:?}")]
    NotFound(VertexId),
    
    #[error("Session not found: {0}")]
    SessionNotFound(SessionId),
    
    #[error("Serialization error: {0}")]
    Serialization(String),
    
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("Backend error: {0}")]
    Backend(String),
    
    #[error("Corruption detected: {0}")]
    Corruption(String),
}

/// Storage statistics
#[derive(Clone, Debug, Default)]
pub struct StorageStats {
    pub sessions: u64,
    pub vertices: u64,
    pub bytes_used: u64,
    pub read_ops: u64,
    pub write_ops: u64,
}
```

### 2.2 Payload Store Trait

```rust
/// Storage trait for large payloads
#[async_trait]
pub trait PayloadStore: Send + Sync + Clone {
    /// Store a payload
    async fn put(&self, data: Bytes) -> Result<PayloadRef, StorageError>;
    
    /// Store with options
    async fn put_with_options(
        &self,
        data: Bytes,
        options: PayloadOptions,
    ) -> Result<PayloadRef, StorageError>;
    
    /// Get a payload
    async fn get(&self, payload_ref: &PayloadRef) -> Result<Option<Bytes>, StorageError>;
    
    /// Check if payload exists
    async fn exists(&self, payload_ref: &PayloadRef) -> Result<bool, StorageError>;
    
    /// Delete a payload
    async fn delete(&self, payload_ref: &PayloadRef) -> Result<bool, StorageError>;
    
    /// Get payload metadata
    async fn metadata(&self, payload_ref: &PayloadRef) -> Result<Option<PayloadMetadata>, StorageError>;
    
    /// Garbage collect orphaned payloads
    async fn gc(&self, referenced: &HashSet<PayloadRef>) -> Result<GcStats, StorageError>;
    
    /// Health check
    async fn health(&self) -> HealthStatus;
    
    /// Get storage statistics
    async fn stats(&self) -> StorageStats;
}

/// Payload storage options
#[derive(Clone, Debug, Default)]
pub struct PayloadOptions {
    pub mime_type: Option<String>,
    pub compression: Option<CompressionType>,
    pub ttl: Option<Duration>,
}

/// Compression types
#[derive(Clone, Debug, Copy, PartialEq, Eq)]
pub enum CompressionType {
    None,
    Lz4,
    Zstd,
    Snappy,
}
```

### 2.3 Session Store Trait

```rust
/// Storage trait for session metadata
#[async_trait]
pub trait SessionStore: Send + Sync + Clone {
    /// Store session metadata
    async fn put_session(&self, session: &Session) -> Result<(), StorageError>;
    
    /// Get session metadata
    async fn get_session(&self, id: SessionId) -> Result<Option<Session>, StorageError>;
    
    /// Update session state
    async fn update_state(
        &self,
        id: SessionId,
        state: SessionState,
    ) -> Result<(), StorageError>;
    
    /// List sessions by state
    async fn list_by_state(
        &self,
        state: SessionStateFilter,
        limit: usize,
    ) -> Result<Vec<SessionSummary>, StorageError>;
    
    /// Delete session metadata
    async fn delete_session(&self, id: SessionId) -> Result<(), StorageError>;
    
    /// Count sessions by state
    async fn count_by_state(&self, state: SessionStateFilter) -> Result<u64, StorageError>;
}

/// Filter for session states
#[derive(Clone, Debug)]
pub enum SessionStateFilter {
    Active,
    Paused,
    Resolving,
    Committed,
    Discarded,
    Expired,
    ReadyForGc,
    All,
}
```

---

## 3. In-Memory Backend

The fastest backend, suitable for short-lived sessions and testing. **Always available** — no feature flag required. This is the default backend for tests and ephemeral workflows. Implementation: `crates/rhizo-crypt-core/src/store.rs` (`InMemoryDagStore`).

### 3.1 Implementation

```rust
use dashmap::DashMap;
use parking_lot::RwLock;

/// In-memory DAG store
#[derive(Clone)]
pub struct InMemoryDagStore {
    /// Vertices by session and ID
    vertices: Arc<DashMap<SessionId, DashMap<VertexId, Vertex>>>,
    
    /// Parent → Children index
    children: Arc<DashMap<SessionId, DashMap<VertexId, HashSet<VertexId>>>>,
    
    /// Frontier vertices per session
    frontiers: Arc<DashMap<SessionId, RwLock<HashSet<VertexId>>>>,
    
    /// Genesis vertices per session
    genesis: Arc<DashMap<SessionId, RwLock<HashSet<VertexId>>>>,
    
    /// Statistics
    stats: Arc<RwLock<StorageStats>>,
}

impl InMemoryDagStore {
    /// Create a new in-memory store
    pub fn new() -> Self {
        Self {
            vertices: Arc::new(DashMap::new()),
            children: Arc::new(DashMap::new()),
            frontiers: Arc::new(DashMap::new()),
            genesis: Arc::new(DashMap::new()),
            stats: Arc::new(RwLock::new(StorageStats::default())),
        }
    }
    
    /// Create with capacity hints
    pub fn with_capacity(sessions: usize, vertices_per_session: usize) -> Self {
        Self {
            vertices: Arc::new(DashMap::with_capacity(sessions)),
            children: Arc::new(DashMap::with_capacity(sessions)),
            frontiers: Arc::new(DashMap::with_capacity(sessions)),
            genesis: Arc::new(DashMap::with_capacity(sessions)),
            stats: Arc::new(RwLock::new(StorageStats::default())),
        }
    }
}

#[async_trait]
impl DagStore for InMemoryDagStore {
    async fn put_vertex(
        &self,
        session_id: SessionId,
        vertex: &Vertex,
    ) -> Result<(), StorageError> {
        let vertex_id = vertex.compute_id();
        
        // Get or create session maps
        let session_vertices = self.vertices
            .entry(session_id)
            .or_insert_with(DashMap::new);
        
        let session_children = self.children
            .entry(session_id)
            .or_insert_with(DashMap::new);
        
        // Store vertex
        session_vertices.insert(vertex_id, vertex.clone());
        
        // Update parent → child index
        for parent in &vertex.parents {
            session_children
                .entry(*parent)
                .or_insert_with(HashSet::new)
                .insert(vertex_id);
        }
        
        // Track genesis
        if vertex.parents.is_empty() {
            self.genesis
                .entry(session_id)
                .or_insert_with(|| RwLock::new(HashSet::new()))
                .write()
                .insert(vertex_id);
        }
        
        // Update stats
        {
            let mut stats = self.stats.write();
            stats.vertices += 1;
            stats.write_ops += 1;
        }
        
        Ok(())
    }
    
    async fn get_vertex(
        &self,
        session_id: SessionId,
        vertex_id: &VertexId,
    ) -> Result<Option<Vertex>, StorageError> {
        let mut stats = self.stats.write();
        stats.read_ops += 1;
        
        let result = self.vertices
            .get(&session_id)
            .and_then(|session| session.get(vertex_id).map(|v| v.clone()));
        
        if result.is_some() {
            stats.cache_hits += 1;
        } else {
            stats.cache_misses += 1;
        }
        
        Ok(result)
    }
    
    async fn get_children(
        &self,
        session_id: SessionId,
        parent_id: &VertexId,
    ) -> Result<Vec<VertexId>, StorageError> {
        let children = self.children
            .get(&session_id)
            .and_then(|session| {
                session.get(parent_id).map(|c| c.iter().copied().collect())
            })
            .unwrap_or_default();
        
        Ok(children)
    }
    
    async fn get_frontier(
        &self,
        session_id: SessionId,
    ) -> Result<Vec<VertexId>, StorageError> {
        let frontier = self.frontiers
            .get(&session_id)
            .map(|f| f.read().iter().copied().collect())
            .unwrap_or_default();
        
        Ok(frontier)
    }
    
    async fn update_frontier(
        &self,
        session_id: SessionId,
        new_vertex: VertexId,
        consumed_parents: &[VertexId],
    ) -> Result<(), StorageError> {
        let frontier = self.frontiers
            .entry(session_id)
            .or_insert_with(|| RwLock::new(HashSet::new()));
        
        let mut frontier = frontier.write();
        
        // Remove consumed parents
        for parent in consumed_parents {
            frontier.remove(parent);
        }
        
        // Add new vertex
        frontier.insert(new_vertex);
        
        Ok(())
    }
    
    async fn delete_session(&self, session_id: SessionId) -> Result<u64, StorageError> {
        let count = self.vertices
            .get(&session_id)
            .map(|v| v.len() as u64)
            .unwrap_or(0);
        
        self.vertices.remove(&session_id);
        self.children.remove(&session_id);
        self.frontiers.remove(&session_id);
        self.genesis.remove(&session_id);
        
        {
            let mut stats = self.stats.write();
            stats.vertices = stats.vertices.saturating_sub(count);
            stats.sessions = stats.sessions.saturating_sub(1);
        }
        
        Ok(count)
    }
    
    async fn health(&self) -> HealthStatus {
        HealthStatus::Healthy
    }
    
    async fn stats(&self) -> StorageStats {
        self.stats.read().clone()
    }
    
    // ... other trait methods
}
```

### 3.2 In-Memory Payload Store

```rust
/// In-memory payload store
#[derive(Clone)]
pub struct InMemoryPayloadStore {
    payloads: Arc<DashMap<PayloadRef, Bytes>>,
    metadata: Arc<DashMap<PayloadRef, PayloadMetadata>>,
    stats: Arc<RwLock<StorageStats>>,
}

impl InMemoryPayloadStore {
    pub fn new() -> Self {
        Self {
            payloads: Arc::new(DashMap::new()),
            metadata: Arc::new(DashMap::new()),
            stats: Arc::new(RwLock::new(StorageStats::default())),
        }
    }
}

#[async_trait]
impl PayloadStore for InMemoryPayloadStore {
    async fn put(&self, data: Bytes) -> Result<PayloadRef, StorageError> {
        let payload_ref = PayloadRef::from_bytes(&data);
        
        self.payloads.insert(payload_ref.clone(), data.clone());
        self.metadata.insert(payload_ref.clone(), PayloadMetadata {
            hash: payload_ref.hash,
            size: data.len() as u64,
            mime_type: None,
            created_at: current_timestamp_nanos(),
            metadata: HashMap::new(),
        });
        
        {
            let mut stats = self.stats.write();
            stats.bytes_used += data.len() as u64;
            stats.write_ops += 1;
        }
        
        Ok(payload_ref)
    }
    
    async fn get(&self, payload_ref: &PayloadRef) -> Result<Option<Bytes>, StorageError> {
        let mut stats = self.stats.write();
        stats.read_ops += 1;
        
        Ok(self.payloads.get(payload_ref).map(|v| v.clone()))
    }
    
    // ... other trait methods
}
```

---

## 4. redb Backend (Default Persistent)

Persistent storage for general-purpose use. **100% Pure Rust** — no C dependencies, ecoBin compliant. Implementation: `crates/rhizo-crypt-core/src/store_redb.rs` (`RedbDagStore`). Enabled by default via the `redb` feature.

### 4.1 Features

- ACID transactions
- MVCC — concurrent readers without blocking writers
- Table separation: `vertices`, `children`, `frontiers`, `genesis`, `metadata`

### 4.2 Implementation Sketch

```rust
use redb::{Database, TableDefinition};

const VERTICES: TableDefinition<&[u8], &[u8]> = TableDefinition::new("vertices");
const CHILDREN: TableDefinition<&[u8], &[u8]> = TableDefinition::new("children");
const FRONTIERS: TableDefinition<&[u8], &[u8]> = TableDefinition::new("frontiers");
const GENESIS: TableDefinition<&[u8], &[u8]> = TableDefinition::new("genesis");
const METADATA: TableDefinition<&[u8], &[u8]> = TableDefinition::new("metadata");

/// redb-backed DAG store (100% Pure Rust).
#[derive(Clone)]
pub struct RedbDagStore {
    db: Arc<Database>,
    path: Arc<PathBuf>,
    read_ops: Arc<AtomicU64>,
    write_ops: Arc<AtomicU64>,
}

impl RedbDagStore {
    pub fn open(path: impl AsRef<Path>) -> Result<Self> {
        let db = Database::create(path)?;
        // Open tables to ensure they exist
        let write_txn = db.begin_write()?;
        let _ = write_txn.open_table(VERTICES)?;
        let _ = write_txn.open_table(CHILDREN)?;
        // ... other tables
        write_txn.commit()?;
        Ok(Self { db: Arc::new(db), path, read_ops, write_ops })
    }

    fn vertex_key(session_id: SessionId, vertex_id: VertexId) -> Vec<u8> {
        let mut key = Vec::with_capacity(49);
        key.extend_from_slice(session_id.as_bytes());
        key.push(b':');
        key.extend_from_slice(vertex_id.as_bytes());
        key
    }
}

impl DagStore for RedbDagStore {
    async fn put_vertex(&self, session_id: SessionId, vertex: Vertex) -> Result<()> {
        let write_txn = self.db.begin_write()?;
        let mut vertices_table = write_txn.open_table(VERTICES)?;
        let key = Self::vertex_key(session_id, vertex_id);
        vertices_table.insert(key.as_slice(), value.as_slice())?;
        // Update children, genesis, frontiers...
        write_txn.commit()?;
        Ok(())
    }
    // ... other trait methods
}
```

---

## 5. Backend Selection

### 5.1 Configuration

```rust
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum StorageBackend {
    Memory,
    Redb,
}
```

### 5.2 Runtime Dispatch

`DagStore` uses RPITIT (non-object-safe), so runtime dispatch uses `DagBackend` enum:

```rust
pub enum DagBackend {
    Memory(InMemoryDagStore),
    Redb(RedbDagStore),
}
```

`RhizoCrypt::start()` selects the backend based on `StorageConfig::backend`.

---

## 6. Performance Characteristics

| Operation | Memory | redb |
|-----------|--------|------|
| Put vertex | ~1µs | ~50–100µs |
| Get vertex | ~100ns | ~10µs |
| Get children | ~200ns | ~20µs |
| Delete session | O(n) | O(n) |
| Memory usage | High | Medium |
| Persistence | None | Full |
| Concurrent reads | Lock-free | MVCC (excellent) |
| Concurrent writes | Sharded locks | Single writer |
| ecoBin compliant | Yes | Yes (100% Pure Rust) |

---

## 7. References

- [ARCHITECTURE.md](./ARCHITECTURE.md) — System architecture
- [DATA_MODEL.md](./DATA_MODEL.md) — Data structures
- [redb Documentation](https://docs.rs/redb/)

---

*RhizoCrypt: The memory that knows when to forget.*

