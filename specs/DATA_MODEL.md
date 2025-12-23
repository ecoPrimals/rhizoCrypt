# RhizoCrypt — Data Model Specification

**Version**: 0.2.0  
**Status**: Draft  
**Last Updated**: December 22, 2025

---

## 1. Overview

This document defines the core data structures of RhizoCrypt: the Vertex, Session, and DAG types that form the foundation of the ephemeral working memory layer.

---

## 2. Content Addressing

All RhizoCrypt data structures use Blake3 for content addressing:

```rust
use blake3::Hasher;

/// 32-byte content hash
pub type ContentHash = [u8; 32];

/// Vertex identifier (Blake3 hash of canonical vertex)
pub type VertexId = ContentHash;

/// Session identifier (UUID v7 for time-ordering)
pub type SessionId = uuid::Uuid;

/// Payload reference (Blake3 hash of payload bytes)
pub type PayloadRef = ContentHash;

/// Compute Blake3 hash of bytes
pub fn hash_bytes(data: &[u8]) -> ContentHash {
    blake3::hash(data).into()
}

/// Compute Blake3 hash of two hashes (for Merkle trees)
pub fn hash_pair(left: &ContentHash, right: &ContentHash) -> ContentHash {
    let mut hasher = Hasher::new();
    hasher.update(left);
    hasher.update(right);
    hasher.finalize().into()
}
```

---

## 3. Vertex Structure

A Vertex is a single event in the RhizoCrypt DAG.

### 3.1 Core Definition

```rust
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

/// A single event in the RhizoCrypt DAG
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Vertex {
    // === Identity ===
    
    /// Content-addressed identifier (computed, not serialized)
    #[serde(skip)]
    id: Option<VertexId>,
    
    // === Structure ===
    
    /// References to parent vertices (empty for genesis)
    pub parents: Vec<VertexId>,
    
    /// Timestamp of vertex creation (nanoseconds since epoch)
    pub timestamp: u64,
    
    // === Agent ===
    
    /// The agent that created this vertex (BearDog DID)
    pub agent: Option<Did>,
    
    /// Optional cryptographic signature from agent
    pub signature: Option<Signature>,
    
    // === Content ===
    
    /// Event type identifier
    pub event_type: EventType,
    
    /// Event payload reference (content-addressed)
    pub payload: Option<PayloadRef>,
    
    /// Inline metadata (small key-value pairs)
    pub metadata: HashMap<String, Value>,
}

impl Vertex {
    /// Compute the vertex ID (Blake3 hash of canonical form)
    pub fn compute_id(&self) -> VertexId {
        let canonical = self.to_canonical_bytes();
        hash_bytes(&canonical)
    }
    
    /// Get or compute the vertex ID
    pub fn id(&mut self) -> VertexId {
        if let Some(id) = self.id {
            id
        } else {
            let id = self.compute_id();
            self.id = Some(id);
            id
        }
    }
    
    /// Serialize to canonical bytes (for hashing)
    pub fn to_canonical_bytes(&self) -> Vec<u8> {
        // Use deterministic CBOR encoding
        let mut buf = Vec::new();
        ciborium::into_writer(self, &mut buf)
            .expect("Vertex serialization cannot fail");
        buf
    }
    
    /// Check if this is a genesis vertex
    pub fn is_genesis(&self) -> bool {
        self.parents.is_empty()
    }
    
    /// Check if this vertex is signed
    pub fn is_signed(&self) -> bool {
        self.signature.is_some()
    }
}
```

### 3.2 Event Types

```rust
/// Event type identifier
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum EventType {
    // === Session Lifecycle ===
    SessionStart,
    SessionEnd { outcome: SessionOutcome },
    
    // === Agent Events ===
    AgentJoin { role: AgentRole },
    AgentLeave { reason: LeaveReason },
    AgentAction { action: String },
    
    // === Data Events ===
    DataCreate { schema: Option<String> },
    DataModify { delta_type: String },
    DataDelete,
    DataTransfer { to: Did },
    
    // === Slice Events ===
    SliceCheckout { slice_id: SliceId, mode: SliceMode },
    SliceOperation { slice_id: SliceId, operation: String },
    SliceResolve { slice_id: SliceId, resolution: ResolutionRoute },
    
    // === Domain-Specific (Gaming) ===
    GameEvent { game_type: String, event_name: String },
    ItemLoot { item_type: String },
    ItemDrop,
    ItemTransfer { to: Did },
    Combat { target: Did, outcome: String },
    Extraction { success: bool },
    
    // === Domain-Specific (Science) ===
    ExperimentStart { protocol: String },
    Observation { instrument: String },
    Analysis { method: String },
    Result { confidence: f64 },
    
    // === Domain-Specific (Collaboration) ===
    DocumentEdit { operation: String },
    CommentAdd,
    ApprovalGrant,
    ApprovalRevoke,
    
    // === Custom ===
    Custom { domain: String, event_name: String },
}

impl EventType {
    /// Get the domain for this event type
    pub fn domain(&self) -> &'static str {
        match self {
            Self::SessionStart | Self::SessionEnd { .. } => "session",
            Self::AgentJoin { .. } | Self::AgentLeave { .. } | Self::AgentAction { .. } => "agent",
            Self::DataCreate { .. } | Self::DataModify { .. } | Self::DataDelete | Self::DataTransfer { .. } => "data",
            Self::SliceCheckout { .. } | Self::SliceOperation { .. } | Self::SliceResolve { .. } => "slice",
            Self::GameEvent { .. } | Self::ItemLoot { .. } | Self::ItemDrop | Self::ItemTransfer { .. } | Self::Combat { .. } | Self::Extraction { .. } => "gaming",
            Self::ExperimentStart { .. } | Self::Observation { .. } | Self::Analysis { .. } | Self::Result { .. } => "science",
            Self::DocumentEdit { .. } | Self::CommentAdd | Self::ApprovalGrant | Self::ApprovalRevoke => "collaboration",
            Self::Custom { domain, .. } => domain.as_str(),
        }
    }
}
```

### 3.3 Vertex Builder

```rust
/// Builder for creating vertices
pub struct VertexBuilder {
    parents: Vec<VertexId>,
    timestamp: Option<u64>,
    agent: Option<Did>,
    event_type: EventType,
    payload: Option<PayloadRef>,
    metadata: HashMap<String, Value>,
}

impl VertexBuilder {
    /// Create a new builder with required event type
    pub fn new(event_type: EventType) -> Self {
        Self {
            parents: Vec::new(),
            timestamp: None,
            agent: None,
            event_type,
            payload: None,
            metadata: HashMap::new(),
        }
    }
    
    /// Add a parent vertex
    pub fn with_parent(mut self, parent: VertexId) -> Self {
        self.parents.push(parent);
        self
    }
    
    /// Add multiple parents
    pub fn with_parents(mut self, parents: impl IntoIterator<Item = VertexId>) -> Self {
        self.parents.extend(parents);
        self
    }
    
    /// Set the agent
    pub fn with_agent(mut self, agent: Did) -> Self {
        self.agent = Some(agent);
        self
    }
    
    /// Set the payload reference
    pub fn with_payload(mut self, payload: PayloadRef) -> Self {
        self.payload = Some(payload);
        self
    }
    
    /// Add metadata
    pub fn with_metadata(mut self, key: impl Into<String>, value: impl Into<Value>) -> Self {
        self.metadata.insert(key.into(), value.into());
        self
    }
    
    /// Build the vertex
    pub fn build(self) -> Vertex {
        Vertex {
            id: None,
            parents: self.parents,
            timestamp: self.timestamp.unwrap_or_else(|| current_timestamp_nanos()),
            agent: self.agent,
            signature: None,
            event_type: self.event_type,
            payload: self.payload,
            metadata: self.metadata,
        }
    }
}
```

---

## 4. Session Structure

A Session is a scoped DAG with its own lifecycle.

### 4.1 Core Definition

```rust
/// A RhizoCrypt session (scoped DAG with lifecycle)
#[derive(Clone, Debug)]
pub struct Session {
    // === Identity ===
    
    /// Unique session identifier (UUID v7)
    pub id: SessionId,
    
    /// Human-readable session name
    pub name: Option<String>,
    
    // === Configuration ===
    
    /// Session type (determines event types, policies)
    pub session_type: SessionType,
    
    /// Session configuration
    pub config: SessionConfig,
    
    // === Lifecycle ===
    
    /// Genesis timestamp
    pub created_at: u64,
    
    /// Session state
    pub state: SessionState,
    
    // === DAG Structure ===
    
    /// Genesis vertices (roots with no parents)
    pub genesis: HashSet<VertexId>,
    
    /// Frontier vertices (tips with no children)
    pub frontier: HashSet<VertexId>,
    
    /// Total vertex count
    pub vertex_count: u64,
    
    // === Slices ===
    
    /// Active slices in this session
    pub slices: HashMap<SliceId, SliceRef>,
    
    // === Agents ===
    
    /// Agents participating in this session
    pub agents: HashSet<Did>,
    
    // === Cached Computations ===
    
    /// Cached Merkle root (invalidated on vertex append)
    merkle_root: Option<MerkleRoot>,
}

/// Session type
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum SessionType {
    /// Gaming session (raid, match, etc.)
    Gaming { game_id: String },
    
    /// Scientific experiment
    Experiment { protocol_id: String },
    
    /// Collaborative document editing
    Collaboration { workspace_id: String },
    
    /// General-purpose session
    General,
    
    /// Custom domain
    Custom { domain: String },
}

/// Session state
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SessionState {
    /// Actively accepting events
    Active,
    
    /// Paused (no new events, can resume)
    Paused { reason: String },
    
    /// Preparing for resolution
    Resolving { started_at: u64 },
    
    /// Committed to LoamSpine
    Committed { 
        loam_ref: LoamCommitRef,
        committed_at: u64,
    },
    
    /// Discarded without commit
    Discarded { 
        reason: DiscardReason,
        discarded_at: u64,
    },
    
    /// Garbage collected
    Expired { expired_at: u64 },
}

impl SessionState {
    /// Check if session is accepting events
    pub fn is_active(&self) -> bool {
        matches!(self, Self::Active)
    }
    
    /// Check if session is terminal
    pub fn is_terminal(&self) -> bool {
        matches!(self, Self::Committed { .. } | Self::Discarded { .. } | Self::Expired { .. })
    }
}
```

### 4.2 Session Configuration

```rust
/// Session configuration
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SessionConfig {
    // === Limits ===
    
    /// Maximum session duration
    pub max_duration: Duration,
    
    /// Maximum vertices before forced resolution
    pub max_vertices: u64,
    
    /// Maximum payload bytes
    pub max_payload_bytes: u64,
    
    // === Signatures ===
    
    /// Require signatures for all events
    pub require_all_signatures: bool,
    
    /// Event types that require signatures
    pub signature_required_events: HashSet<String>,
    
    // === Dehydration ===
    
    /// Automatic dehydration on resolve
    pub auto_dehydrate: bool,
    
    /// Dehydration configuration
    pub dehydration: DehydrationConfig,
    
    // === Storage ===
    
    /// Preferred storage backend
    pub storage_backend: StorageBackend,
    
    /// Payload storage configuration
    pub payload_store: PayloadStoreConfig,
    
    // === Access Control ===
    
    /// Session owner
    pub owner: Did,
    
    /// Agents allowed to append
    pub allowed_agents: Option<HashSet<Did>>,
    
    /// Agents allowed to read
    pub read_access: AccessPolicy,
}

impl Default for SessionConfig {
    fn default() -> Self {
        Self {
            max_duration: Duration::from_secs(3600), // 1 hour
            max_vertices: 100_000,
            max_payload_bytes: 1024 * 1024 * 1024, // 1 GB
            require_all_signatures: false,
            signature_required_events: HashSet::new(),
            auto_dehydrate: true,
            dehydration: DehydrationConfig::default(),
            storage_backend: StorageBackend::Memory,
            payload_store: PayloadStoreConfig::default(),
            owner: Did::default(),
            allowed_agents: None,
            read_access: AccessPolicy::Owner,
        }
    }
}

/// Dehydration configuration
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DehydrationConfig {
    /// Include full vertices in summary
    pub include_vertices: bool,
    
    /// Include payloads in summary
    pub include_payloads: bool,
    
    /// Generate Merkle proofs for key vertices
    pub generate_proofs_for: Vec<VertexId>,
    
    /// Require attestations before commit
    pub required_attestations: Vec<Did>,
}

impl Default for DehydrationConfig {
    fn default() -> Self {
        Self {
            include_vertices: false,
            include_payloads: false,
            generate_proofs_for: Vec::new(),
            required_attestations: Vec::new(),
        }
    }
}
```

### 4.3 Session Builder

```rust
/// Builder for creating sessions
pub struct SessionBuilder {
    name: Option<String>,
    session_type: SessionType,
    config: SessionConfig,
}

impl SessionBuilder {
    /// Create a new session builder
    pub fn new(session_type: SessionType) -> Self {
        Self {
            name: None,
            session_type,
            config: SessionConfig::default(),
        }
    }
    
    /// Set session name
    pub fn with_name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }
    
    /// Set session owner
    pub fn with_owner(mut self, owner: Did) -> Self {
        self.config.owner = owner;
        self
    }
    
    /// Set max duration
    pub fn with_max_duration(mut self, duration: Duration) -> Self {
        self.config.max_duration = duration;
        self
    }
    
    /// Set max vertices
    pub fn with_max_vertices(mut self, max: u64) -> Self {
        self.config.max_vertices = max;
        self
    }
    
    /// Require all signatures
    pub fn require_all_signatures(mut self) -> Self {
        self.config.require_all_signatures = true;
        self
    }
    
    /// Build the session
    pub fn build(self) -> Session {
        Session {
            id: SessionId::now_v7(),
            name: self.name,
            session_type: self.session_type,
            config: self.config,
            created_at: current_timestamp_nanos(),
            state: SessionState::Active,
            genesis: HashSet::new(),
            frontier: HashSet::new(),
            vertex_count: 0,
            slices: HashMap::new(),
            agents: HashSet::new(),
            merkle_root: None,
        }
    }
}
```

---

## 5. DAG Structure

### 5.1 DAG Index

The DAG index provides efficient lookups:

```rust
/// DAG index for efficient traversal
#[derive(Clone, Debug, Default)]
pub struct DagIndex {
    /// Parent → Children mapping
    children: HashMap<VertexId, HashSet<VertexId>>,
    
    /// Child → Parents mapping (redundant for fast lookup)
    parents: HashMap<VertexId, Vec<VertexId>>,
    
    /// Vertices by event type
    by_event_type: HashMap<EventType, HashSet<VertexId>>,
    
    /// Vertices by agent
    by_agent: HashMap<Did, HashSet<VertexId>>,
    
    /// Vertices by timestamp (BTreeMap for range queries)
    by_timestamp: BTreeMap<u64, HashSet<VertexId>>,
    
    /// Vertices with specific payload
    by_payload: HashMap<PayloadRef, HashSet<VertexId>>,
}

impl DagIndex {
    /// Create a new empty index
    pub fn new() -> Self {
        Self::default()
    }
    
    /// Index a vertex
    pub fn index_vertex(&mut self, vertex: &Vertex) {
        let id = vertex.id().expect("Vertex must have ID");
        
        // Index parent relationships
        for parent in &vertex.parents {
            self.children.entry(*parent).or_default().insert(id);
        }
        self.parents.insert(id, vertex.parents.clone());
        
        // Index by event type
        self.by_event_type
            .entry(vertex.event_type.clone())
            .or_default()
            .insert(id);
        
        // Index by agent
        if let Some(agent) = &vertex.agent {
            self.by_agent.entry(agent.clone()).or_default().insert(id);
        }
        
        // Index by timestamp
        self.by_timestamp
            .entry(vertex.timestamp)
            .or_default()
            .insert(id);
        
        // Index by payload
        if let Some(payload) = &vertex.payload {
            self.by_payload.entry(*payload).or_default().insert(id);
        }
    }
    
    /// Get children of a vertex
    pub fn get_children(&self, parent: &VertexId) -> Option<&HashSet<VertexId>> {
        self.children.get(parent)
    }
    
    /// Get parents of a vertex
    pub fn get_parents(&self, child: &VertexId) -> Option<&Vec<VertexId>> {
        self.parents.get(child)
    }
    
    /// Get vertices by event type
    pub fn get_by_event_type(&self, event_type: &EventType) -> Option<&HashSet<VertexId>> {
        self.by_event_type.get(event_type)
    }
    
    /// Get vertices by agent
    pub fn get_by_agent(&self, agent: &Did) -> Option<&HashSet<VertexId>> {
        self.by_agent.get(agent)
    }
    
    /// Get vertices in timestamp range
    pub fn get_by_timestamp_range(&self, start: u64, end: u64) -> Vec<VertexId> {
        self.by_timestamp
            .range(start..end)
            .flat_map(|(_, ids)| ids.iter().copied())
            .collect()
    }
}
```

### 5.2 DAG Traversal

```rust
/// DAG traversal utilities
pub struct DagTraversal<'a, S: DagStore> {
    store: &'a S,
    index: &'a DagIndex,
}

impl<'a, S: DagStore> DagTraversal<'a, S> {
    /// Create a new traversal context
    pub fn new(store: &'a S, index: &'a DagIndex) -> Self {
        Self { store, index }
    }
    
    /// Topological sort of all vertices
    pub async fn topological_sort(&self, session: SessionId) -> Result<Vec<Vertex>, RhizoCryptError> {
        let mut visited = HashSet::new();
        let mut result = Vec::new();
        
        // Start from genesis vertices
        let genesis = self.store.get_genesis(session).await?;
        
        for vertex_id in genesis {
            self.topological_visit(&vertex_id, &mut visited, &mut result).await?;
        }
        
        Ok(result)
    }
    
    /// Recursive topological visit
    async fn topological_visit(
        &self,
        vertex_id: &VertexId,
        visited: &mut HashSet<VertexId>,
        result: &mut Vec<Vertex>,
    ) -> Result<(), RhizoCryptError> {
        if visited.contains(vertex_id) {
            return Ok(());
        }
        
        visited.insert(*vertex_id);
        
        let vertex = self.store.get_vertex(vertex_id).await?
            .ok_or_else(|| RhizoCryptError::VertexNotFound(*vertex_id))?;
        
        // Visit children first (for reverse topological order)
        if let Some(children) = self.index.get_children(vertex_id) {
            for child_id in children {
                self.topological_visit(child_id, visited, result).await?;
            }
        }
        
        result.push(vertex);
        Ok(())
    }
    
    /// Find all ancestors of a vertex
    pub async fn ancestors(&self, vertex_id: &VertexId) -> Result<HashSet<VertexId>, RhizoCryptError> {
        let mut ancestors = HashSet::new();
        let mut queue = VecDeque::new();
        
        queue.push_back(*vertex_id);
        
        while let Some(current) = queue.pop_front() {
            if let Some(parents) = self.index.get_parents(&current) {
                for parent in parents {
                    if ancestors.insert(*parent) {
                        queue.push_back(*parent);
                    }
                }
            }
        }
        
        Ok(ancestors)
    }
    
    /// Find all descendants of a vertex
    pub async fn descendants(&self, vertex_id: &VertexId) -> Result<HashSet<VertexId>, RhizoCryptError> {
        let mut descendants = HashSet::new();
        let mut queue = VecDeque::new();
        
        queue.push_back(*vertex_id);
        
        while let Some(current) = queue.pop_front() {
            if let Some(children) = self.index.get_children(&current) {
                for child in children {
                    if descendants.insert(*child) {
                        queue.push_back(*child);
                    }
                }
            }
        }
        
        Ok(descendants)
    }
    
    /// Find lowest common ancestors of two vertices
    pub async fn lowest_common_ancestors(
        &self,
        a: &VertexId,
        b: &VertexId,
    ) -> Result<HashSet<VertexId>, RhizoCryptError> {
        let ancestors_a = self.ancestors(a).await?;
        let ancestors_b = self.ancestors(b).await?;
        
        let common: HashSet<_> = ancestors_a.intersection(&ancestors_b).copied().collect();
        
        // Filter to only lowest (no descendants in common set)
        let mut lowest = HashSet::new();
        for candidate in &common {
            let descendants = self.descendants(candidate).await?;
            if descendants.intersection(&common).next().is_none() {
                lowest.insert(*candidate);
            }
        }
        
        Ok(lowest)
    }
}
```

---

## 6. Merkle Tree Structure

### 6.1 Merkle Root

```rust
/// Merkle root of a session
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct MerkleRoot(pub ContentHash);

impl MerkleRoot {
    /// Compute Merkle root from vertices in topological order
    pub fn compute(vertices: &[Vertex]) -> Self {
        if vertices.is_empty() {
            return Self([0u8; 32]);
        }
        
        // Compute leaf hashes
        let mut hashes: Vec<ContentHash> = vertices
            .iter()
            .map(|v| v.compute_id())
            .collect();
        
        // Pad to power of 2 if needed
        while !hashes.len().is_power_of_two() {
            hashes.push([0u8; 32]);
        }
        
        // Build tree bottom-up
        while hashes.len() > 1 {
            hashes = hashes
                .chunks(2)
                .map(|chunk| hash_pair(&chunk[0], &chunk[1]))
                .collect();
        }
        
        Self(hashes[0])
    }
}
```

### 6.2 Merkle Proof

```rust
/// Merkle proof for a vertex
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MerkleProof {
    /// The vertex being proven
    pub vertex_id: VertexId,
    
    /// Position in topological order
    pub position: usize,
    
    /// Total vertices in session
    pub total_vertices: usize,
    
    /// Sibling hashes from leaf to root
    pub siblings: Vec<(Direction, ContentHash)>,
    
    /// The Merkle root
    pub root: MerkleRoot,
}

/// Direction in Merkle tree
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum Direction {
    Left,
    Right,
}

impl MerkleProof {
    /// Verify this proof
    pub fn verify(&self, vertex: &Vertex) -> bool {
        let vertex_hash = vertex.compute_id();
        if vertex_hash != self.vertex_id {
            return false;
        }
        
        let mut current = vertex_hash;
        
        for (direction, sibling) in &self.siblings {
            current = match direction {
                Direction::Left => hash_pair(sibling, &current),
                Direction::Right => hash_pair(&current, sibling),
            };
        }
        
        current == self.root.0
    }
    
    /// Generate proof for a vertex at given position
    pub fn generate(
        vertices: &[Vertex],
        position: usize,
        root: MerkleRoot,
    ) -> Result<Self, RhizoCryptError> {
        if position >= vertices.len() {
            return Err(RhizoCryptError::Internal("Position out of bounds".into()));
        }
        
        let vertex = &vertices[position];
        let vertex_id = vertex.compute_id();
        
        // Compute leaf hashes
        let mut hashes: Vec<ContentHash> = vertices
            .iter()
            .map(|v| v.compute_id())
            .collect();
        
        // Pad to power of 2
        let original_len = hashes.len();
        while !hashes.len().is_power_of_two() {
            hashes.push([0u8; 32]);
        }
        
        // Build proof
        let mut siblings = Vec::new();
        let mut idx = position;
        
        while hashes.len() > 1 {
            let sibling_idx = if idx % 2 == 0 { idx + 1 } else { idx - 1 };
            let direction = if idx % 2 == 0 { Direction::Right } else { Direction::Left };
            
            siblings.push((direction, hashes[sibling_idx]));
            
            // Compute next level
            hashes = hashes
                .chunks(2)
                .map(|chunk| hash_pair(&chunk[0], &chunk[1]))
                .collect();
            
            idx /= 2;
        }
        
        Ok(Self {
            vertex_id,
            position,
            total_vertices: original_len,
            siblings,
            root,
        })
    }
}
```

---

## 7. Payload Storage

### 7.1 Payload Reference

```rust
/// Reference to an externally stored payload
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct PayloadRef {
    /// Blake3 hash of payload content
    pub hash: ContentHash,
    
    /// Payload size in bytes
    pub size: u64,
    
    /// MIME type
    pub mime_type: Option<String>,
}

impl PayloadRef {
    /// Create a new payload reference from bytes
    pub fn from_bytes(data: &[u8]) -> Self {
        Self {
            hash: hash_bytes(data),
            size: data.len() as u64,
            mime_type: None,
        }
    }
    
    /// Create with MIME type
    pub fn from_bytes_with_mime(data: &[u8], mime_type: impl Into<String>) -> Self {
        Self {
            hash: hash_bytes(data),
            size: data.len() as u64,
            mime_type: Some(mime_type.into()),
        }
    }
}
```

### 7.2 Payload Store Trait

```rust
/// Trait for payload storage backends
#[async_trait]
pub trait PayloadStore: Send + Sync {
    /// Store a payload
    async fn put(&self, data: bytes::Bytes) -> Result<PayloadRef, StorageError>;
    
    /// Get a payload by reference
    async fn get(&self, payload_ref: &PayloadRef) -> Result<Option<bytes::Bytes>, StorageError>;
    
    /// Check if payload exists
    async fn exists(&self, payload_ref: &PayloadRef) -> Result<bool, StorageError>;
    
    /// Delete a payload
    async fn delete(&self, payload_ref: &PayloadRef) -> Result<bool, StorageError>;
    
    /// Get total stored bytes
    async fn total_bytes(&self) -> Result<u64, StorageError>;
}
```

---

## 8. Type Aliases and Utilities

```rust
/// BearDog DID (Decentralized Identifier)
pub type Did = String; // Full: did:key:z6Mk...

/// BearDog signature
pub type Signature = Vec<u8>;

/// LoamSpine commit reference
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LoamCommitRef {
    pub spine_id: String,
    pub entry_hash: ContentHash,
    pub index: u64,
}

/// Slice identifier
pub type SliceId = uuid::Uuid;

/// Generic value type for metadata
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Value {
    Null,
    Bool(bool),
    Int(i64),
    Float(f64),
    String(String),
    Bytes(Vec<u8>),
    Array(Vec<Value>),
    Object(HashMap<String, Value>),
}

/// Get current timestamp in nanoseconds
pub fn current_timestamp_nanos() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("Time went backwards")
        .as_nanos() as u64
}
```

---

## 9. Serialization

All data structures use deterministic CBOR serialization for content addressing:

```rust
use ciborium;

impl Vertex {
    /// Serialize to CBOR bytes
    pub fn to_cbor(&self) -> Vec<u8> {
        let mut buf = Vec::new();
        ciborium::into_writer(self, &mut buf).expect("Serialization cannot fail");
        buf
    }
    
    /// Deserialize from CBOR bytes
    pub fn from_cbor(data: &[u8]) -> Result<Self, RhizoCryptError> {
        ciborium::from_reader(data)
            .map_err(|e| RhizoCryptError::Internal(format!("Deserialization failed: {}", e)))
    }
}

impl Session {
    /// Serialize session metadata (not vertices)
    pub fn to_cbor(&self) -> Vec<u8> {
        let mut buf = Vec::new();
        ciborium::into_writer(self, &mut buf).expect("Serialization cannot fail");
        buf
    }
}
```

---

## 10. References

- [ARCHITECTURE.md](./ARCHITECTURE.md) — System architecture
- [SLICE_SEMANTICS.md](./SLICE_SEMANTICS.md) — Slice data structures
- [STORAGE_BACKENDS.md](./STORAGE_BACKENDS.md) — Store implementations
- [RHIZOCRYPT_SPECIFICATION.md](./RHIZOCRYPT_SPECIFICATION.md) — Full specification

---

*RhizoCrypt: The memory that knows when to forget.*

