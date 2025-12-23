# RhizoCrypt — Storage Backends Specification

**Version**: 0.2.0  
**Status**: Draft  
**Last Updated**: December 22, 2025

---

## 1. Overview

RhizoCrypt uses a pluggable storage architecture with three primary backends:

| Backend | Use Case | Durability | Performance |
|---------|----------|------------|-------------|
| **In-Memory** | Short sessions, testing | None | Fastest |
| **RocksDB** | General purpose, longer sessions | Full | Fast |
| **LMDB** | Memory-mapped, high read workloads | Full | Very Fast |

All backends implement the same trait interface, allowing seamless swapping.

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
    pub bytes_available: u64,
    pub read_ops: u64,
    pub write_ops: u64,
    pub cache_hits: u64,
    pub cache_misses: u64,
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

The fastest backend, suitable for short-lived sessions and testing.

### 3.1 Implementation

```rust
use dashmap::DashMap;
use parking_lot::RwLock;

/// In-memory DAG store
#[derive(Clone)]
pub struct MemoryDagStore {
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

impl MemoryDagStore {
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
impl DagStore for MemoryDagStore {
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
pub struct MemoryPayloadStore {
    payloads: Arc<DashMap<PayloadRef, Bytes>>,
    metadata: Arc<DashMap<PayloadRef, PayloadMetadata>>,
    stats: Arc<RwLock<StorageStats>>,
}

impl MemoryPayloadStore {
    pub fn new() -> Self {
        Self {
            payloads: Arc::new(DashMap::new()),
            metadata: Arc::new(DashMap::new()),
            stats: Arc::new(RwLock::new(StorageStats::default())),
        }
    }
}

#[async_trait]
impl PayloadStore for MemoryPayloadStore {
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

## 4. RocksDB Backend

Persistent storage for general-purpose use.

### 4.1 Implementation

```rust
use rocksdb::{DB, Options, ColumnFamilyDescriptor};

/// Column families
const CF_VERTICES: &str = "vertices";
const CF_CHILDREN: &str = "children";
const CF_FRONTIERS: &str = "frontiers";
const CF_GENESIS: &str = "genesis";
const CF_SESSIONS: &str = "sessions";

/// RocksDB DAG store
#[derive(Clone)]
pub struct RocksDbDagStore {
    db: Arc<DB>,
    stats: Arc<RwLock<StorageStats>>,
}

impl RocksDbDagStore {
    /// Open or create a RocksDB store
    pub fn open(path: impl AsRef<Path>) -> Result<Self, StorageError> {
        let mut opts = Options::default();
        opts.create_if_missing(true);
        opts.create_missing_column_families(true);
        opts.set_compression_type(rocksdb::DBCompressionType::Lz4);
        opts.set_max_background_jobs(4);
        
        // Column family options
        let cf_opts = Options::default();
        
        let cfs = vec![
            ColumnFamilyDescriptor::new(CF_VERTICES, cf_opts.clone()),
            ColumnFamilyDescriptor::new(CF_CHILDREN, cf_opts.clone()),
            ColumnFamilyDescriptor::new(CF_FRONTIERS, cf_opts.clone()),
            ColumnFamilyDescriptor::new(CF_GENESIS, cf_opts.clone()),
            ColumnFamilyDescriptor::new(CF_SESSIONS, cf_opts),
        ];
        
        let db = DB::open_cf_descriptors(&opts, path, cfs)
            .map_err(|e| StorageError::Backend(e.to_string()))?;
        
        Ok(Self {
            db: Arc::new(db),
            stats: Arc::new(RwLock::new(StorageStats::default())),
        })
    }
    
    /// Compose key from session and vertex ID
    fn vertex_key(session_id: SessionId, vertex_id: &VertexId) -> Vec<u8> {
        let mut key = Vec::with_capacity(32 + 32);
        key.extend_from_slice(session_id.as_bytes());
        key.extend_from_slice(vertex_id);
        key
    }
}

#[async_trait]
impl DagStore for RocksDbDagStore {
    async fn put_vertex(
        &self,
        session_id: SessionId,
        vertex: &Vertex,
    ) -> Result<(), StorageError> {
        let vertex_id = vertex.compute_id();
        let key = Self::vertex_key(session_id, &vertex_id);
        let value = vertex.to_cbor();
        
        let cf = self.db.cf_handle(CF_VERTICES)
            .ok_or_else(|| StorageError::Backend("Missing CF".into()))?;
        
        self.db.put_cf(&cf, &key, &value)
            .map_err(|e| StorageError::Backend(e.to_string()))?;
        
        // Update children index
        let children_cf = self.db.cf_handle(CF_CHILDREN)
            .ok_or_else(|| StorageError::Backend("Missing CF".into()))?;
        
        for parent in &vertex.parents {
            let parent_key = Self::vertex_key(session_id, parent);
            
            // Read existing children
            let mut children: HashSet<VertexId> = self.db
                .get_cf(&children_cf, &parent_key)
                .map_err(|e| StorageError::Backend(e.to_string()))?
                .map(|v| bincode::deserialize(&v).unwrap_or_default())
                .unwrap_or_default();
            
            children.insert(vertex_id);
            
            self.db.put_cf(
                &children_cf,
                &parent_key,
                &bincode::serialize(&children).unwrap(),
            ).map_err(|e| StorageError::Backend(e.to_string()))?;
        }
        
        Ok(())
    }
    
    async fn get_vertex(
        &self,
        session_id: SessionId,
        vertex_id: &VertexId,
    ) -> Result<Option<Vertex>, StorageError> {
        let key = Self::vertex_key(session_id, vertex_id);
        
        let cf = self.db.cf_handle(CF_VERTICES)
            .ok_or_else(|| StorageError::Backend("Missing CF".into()))?;
        
        let value = self.db.get_cf(&cf, &key)
            .map_err(|e| StorageError::Backend(e.to_string()))?;
        
        match value {
            Some(bytes) => {
                let vertex = Vertex::from_cbor(&bytes)?;
                Ok(Some(vertex))
            }
            None => Ok(None),
        }
    }
    
    async fn delete_session(&self, session_id: SessionId) -> Result<u64, StorageError> {
        let prefix = session_id.as_bytes();
        let mut count = 0u64;
        
        // Delete from all column families
        for cf_name in [CF_VERTICES, CF_CHILDREN, CF_FRONTIERS, CF_GENESIS] {
            let cf = self.db.cf_handle(cf_name)
                .ok_or_else(|| StorageError::Backend("Missing CF".into()))?;
            
            let iter = self.db.prefix_iterator_cf(&cf, prefix);
            
            for item in iter {
                let (key, _) = item.map_err(|e| StorageError::Backend(e.to_string()))?;
                
                if !key.starts_with(prefix) {
                    break;
                }
                
                self.db.delete_cf(&cf, &key)
                    .map_err(|e| StorageError::Backend(e.to_string()))?;
                
                if cf_name == CF_VERTICES {
                    count += 1;
                }
            }
        }
        
        Ok(count)
    }
    
    // ... other trait methods
}
```

### 4.2 RocksDB Payload Store

```rust
/// RocksDB payload store
#[derive(Clone)]
pub struct RocksDbPayloadStore {
    db: Arc<DB>,
    compression: CompressionType,
}

impl RocksDbPayloadStore {
    pub fn open(path: impl AsRef<Path>) -> Result<Self, StorageError> {
        let mut opts = Options::default();
        opts.create_if_missing(true);
        opts.set_compression_type(rocksdb::DBCompressionType::Zstd);
        
        let db = DB::open(&opts, path)
            .map_err(|e| StorageError::Backend(e.to_string()))?;
        
        Ok(Self {
            db: Arc::new(db),
            compression: CompressionType::Zstd,
        })
    }
}

#[async_trait]
impl PayloadStore for RocksDbPayloadStore {
    async fn put(&self, data: Bytes) -> Result<PayloadRef, StorageError> {
        let payload_ref = PayloadRef::from_bytes(&data);
        
        self.db.put(&payload_ref.hash, &data)
            .map_err(|e| StorageError::Backend(e.to_string()))?;
        
        Ok(payload_ref)
    }
    
    async fn get(&self, payload_ref: &PayloadRef) -> Result<Option<Bytes>, StorageError> {
        let value = self.db.get(&payload_ref.hash)
            .map_err(|e| StorageError::Backend(e.to_string()))?;
        
        Ok(value.map(Bytes::from))
    }
    
    // ... other trait methods
}
```

---

## 5. LMDB Backend

Memory-mapped for maximum read performance.

### 5.1 Implementation

```rust
use heed::{Database, EnvOpenOptions, types::*};

/// LMDB DAG store
#[derive(Clone)]
pub struct LmdbDagStore {
    env: Arc<heed::Env>,
    vertices: Database<Bytes, Bytes>,
    children: Database<Bytes, Bytes>,
    frontiers: Database<Bytes, Bytes>,
    genesis: Database<Bytes, Bytes>,
}

impl LmdbDagStore {
    pub fn open(path: impl AsRef<Path>) -> Result<Self, StorageError> {
        std::fs::create_dir_all(&path)
            .map_err(|e| StorageError::Io(e))?;
        
        let env = EnvOpenOptions::new()
            .map_size(10 * 1024 * 1024 * 1024) // 10GB
            .max_dbs(10)
            .open(path)
            .map_err(|e| StorageError::Backend(e.to_string()))?;
        
        let mut wtxn = env.write_txn()
            .map_err(|e| StorageError::Backend(e.to_string()))?;
        
        let vertices = env.create_database(&mut wtxn, Some("vertices"))
            .map_err(|e| StorageError::Backend(e.to_string()))?;
        
        let children = env.create_database(&mut wtxn, Some("children"))
            .map_err(|e| StorageError::Backend(e.to_string()))?;
        
        let frontiers = env.create_database(&mut wtxn, Some("frontiers"))
            .map_err(|e| StorageError::Backend(e.to_string()))?;
        
        let genesis = env.create_database(&mut wtxn, Some("genesis"))
            .map_err(|e| StorageError::Backend(e.to_string()))?;
        
        wtxn.commit().map_err(|e| StorageError::Backend(e.to_string()))?;
        
        Ok(Self {
            env: Arc::new(env),
            vertices,
            children,
            frontiers,
            genesis,
        })
    }
}

#[async_trait]
impl DagStore for LmdbDagStore {
    async fn put_vertex(
        &self,
        session_id: SessionId,
        vertex: &Vertex,
    ) -> Result<(), StorageError> {
        let vertex_id = vertex.compute_id();
        let key = compose_key(session_id, &vertex_id);
        let value = vertex.to_cbor();
        
        let mut wtxn = self.env.write_txn()
            .map_err(|e| StorageError::Backend(e.to_string()))?;
        
        self.vertices.put(&mut wtxn, &key, &value)
            .map_err(|e| StorageError::Backend(e.to_string()))?;
        
        wtxn.commit().map_err(|e| StorageError::Backend(e.to_string()))?;
        
        Ok(())
    }
    
    async fn get_vertex(
        &self,
        session_id: SessionId,
        vertex_id: &VertexId,
    ) -> Result<Option<Vertex>, StorageError> {
        let key = compose_key(session_id, vertex_id);
        
        let rtxn = self.env.read_txn()
            .map_err(|e| StorageError::Backend(e.to_string()))?;
        
        let value = self.vertices.get(&rtxn, &key)
            .map_err(|e| StorageError::Backend(e.to_string()))?;
        
        match value {
            Some(bytes) => {
                let vertex = Vertex::from_cbor(bytes)?;
                Ok(Some(vertex))
            }
            None => Ok(None),
        }
    }
    
    // ... other trait methods
}

fn compose_key(session_id: SessionId, vertex_id: &VertexId) -> Vec<u8> {
    let mut key = Vec::with_capacity(32 + 32);
    key.extend_from_slice(session_id.as_bytes());
    key.extend_from_slice(vertex_id);
    key
}
```

---

## 6. Backend Selection

### 6.1 Configuration

```rust
/// Storage backend configuration
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct StorageConfig {
    /// Backend type
    pub backend: StorageBackendType,
    
    /// Path for persistent backends
    pub path: Option<PathBuf>,
    
    /// Memory limit for in-memory backend
    pub memory_limit: Option<usize>,
    
    /// RocksDB-specific options
    pub rocksdb: Option<RocksDbOptions>,
    
    /// LMDB-specific options
    pub lmdb: Option<LmdbOptions>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum StorageBackendType {
    Memory,
    RocksDb,
    Lmdb,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RocksDbOptions {
    pub compression: CompressionType,
    pub cache_size: usize,
    pub max_open_files: i32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LmdbOptions {
    pub map_size: usize,
    pub max_readers: u32,
}
```

### 6.2 Factory

```rust
/// Create storage backends from configuration
pub fn create_dag_store(config: &StorageConfig) -> Result<Box<dyn DagStore>, StorageError> {
    match config.backend {
        StorageBackendType::Memory => {
            Ok(Box::new(MemoryDagStore::new()))
        }
        StorageBackendType::RocksDb => {
            let path = config.path.as_ref()
                .ok_or_else(|| StorageError::Backend("Path required for RocksDB".into()))?;
            Ok(Box::new(RocksDbDagStore::open(path)?))
        }
        StorageBackendType::Lmdb => {
            let path = config.path.as_ref()
                .ok_or_else(|| StorageError::Backend("Path required for LMDB".into()))?;
            Ok(Box::new(LmdbDagStore::open(path)?))
        }
    }
}

pub fn create_payload_store(config: &StorageConfig) -> Result<Box<dyn PayloadStore>, StorageError> {
    match config.backend {
        StorageBackendType::Memory => {
            Ok(Box::new(MemoryPayloadStore::new()))
        }
        StorageBackendType::RocksDb => {
            let path = config.path.as_ref()
                .ok_or_else(|| StorageError::Backend("Path required for RocksDB".into()))?;
            let payload_path = path.join("payloads");
            Ok(Box::new(RocksDbPayloadStore::open(payload_path)?))
        }
        StorageBackendType::Lmdb => {
            let path = config.path.as_ref()
                .ok_or_else(|| StorageError::Backend("Path required for LMDB".into()))?;
            let payload_path = path.join("payloads");
            Ok(Box::new(LmdbPayloadStore::open(payload_path)?))
        }
    }
}
```

---

## 7. Performance Characteristics

| Operation | Memory | RocksDB | LMDB |
|-----------|--------|---------|------|
| Put vertex | ~1µs | ~50µs | ~100µs |
| Get vertex | ~100ns | ~10µs | ~1µs |
| Get children | ~200ns | ~20µs | ~2µs |
| Delete session | O(n) | O(n) | O(n) |
| Memory usage | High | Medium | Low (mmap) |
| Persistence | None | Full | Full |
| Concurrent reads | Lock-free | Good | Excellent |
| Concurrent writes | Sharded locks | Single writer | Single writer |

---

## 8. References

- [ARCHITECTURE.md](./ARCHITECTURE.md) — System architecture
- [DATA_MODEL.md](./DATA_MODEL.md) — Data structures
- [RocksDB Documentation](https://rocksdb.org/docs/)
- [LMDB Documentation](http://www.lmdb.tech/doc/)

---

*RhizoCrypt: The memory that knows when to forget.*

