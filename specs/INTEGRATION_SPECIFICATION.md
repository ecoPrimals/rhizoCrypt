# RhizoCrypt — Integration Specification

**Version**: 0.2.0  
**Status**: Draft  
**Last Updated**: December 22, 2025

---

## 1. Overview

RhizoCrypt integrates with the ecoPrimals ecosystem through well-defined interfaces. This document specifies how RhizoCrypt interacts with each primal.

```
┌─────────────────────────────────────────────────────────────────┐
│                        RhizoCrypt                                │
├─────────────────────────────────────────────────────────────────┤
│                                                                  │
│                    ┌─────────────────┐                          │
│                    │  RhizoCrypt     │                          │
│                    │    Core         │                          │
│                    └────────┬────────┘                          │
│                             │                                    │
│         ┌───────────────────┼───────────────────┐               │
│         │                   │                   │               │
│    ┌────▼────┐        ┌────▼────┐        ┌────▼────┐           │
│    │BearDog  │        │LoamSpine│        │NestGate │           │
│    │Adapter  │        │ Adapter │        │ Adapter │           │
│    └────┬────┘        └────┬────┘        └────┬────┘           │
│         │                   │                   │               │
└─────────┼───────────────────┼───────────────────┼───────────────┘
          │                   │                   │
          ▼                   ▼                   ▼
     ┌─────────┐         ┌─────────┐        ┌─────────┐
     │ BearDog │         │LoamSpine│        │ NestGate│
     │   🐻    │         │   🦴    │        │   🏠    │
     └─────────┘         └─────────┘        └─────────┘
```

---

## 2. BearDog Integration

BearDog provides identity, signing, and policy enforcement.

### 2.1 Client Interface

```rust
/// BearDog client for RhizoCrypt
#[async_trait]
pub trait SigningProvider: Send + Sync {
    // ==================== Identity ====================
    
    /// Resolve a DID to its document
    async fn resolve_did(&self, did: &Did) -> Result<DidDocument, BearDogError>;
    
    /// Get the public key for a DID
    async fn get_public_key(&self, did: &Did) -> Result<PublicKey, BearDogError>;
    
    /// Verify that a DID is valid and active
    async fn verify_did(&self, did: &Did) -> Result<bool, BearDogError>;
    
    // ==================== Signing ====================
    
    /// Sign data with a specific key
    async fn sign(
        &self,
        data: &[u8],
        key_id: &KeyId,
    ) -> Result<Signature, BearDogError>;
    
    /// Sign a vertex
    async fn sign_vertex(
        &self,
        vertex: &Vertex,
        key_id: &KeyId,
    ) -> Result<Signature, BearDogError>;
    
    /// Verify a signature
    async fn verify_signature(
        &self,
        data: &[u8],
        signature: &Signature,
        did: &Did,
    ) -> Result<bool, BearDogError>;
    
    /// Verify a vertex signature
    async fn verify_vertex_signature(
        &self,
        vertex: &Vertex,
    ) -> Result<bool, BearDogError>;
    
    // ==================== Attestations ====================
    
    /// Request an attestation from another party
    async fn request_attestation(
        &self,
        attester: &Did,
        request: &AttestationRequest,
    ) -> Result<Attestation, BearDogError>;
    
    /// Create an attestation
    async fn create_attestation(
        &self,
        subject: &AttestationSubject,
        key_id: &KeyId,
    ) -> Result<Attestation, BearDogError>;
    
    /// Verify an attestation
    async fn verify_attestation(
        &self,
        attestation: &Attestation,
    ) -> Result<bool, BearDogError>;
    
    // ==================== Permissions ====================
    
    /// Check if a DID has permission for an action
    async fn check_permission(
        &self,
        did: &Did,
        resource: &str,
        action: &str,
    ) -> Result<PermissionResult, BearDogError>;
    
    /// Check multiple permissions at once
    async fn check_permissions_batch(
        &self,
        did: &Did,
        checks: &[PermissionCheck],
    ) -> Result<Vec<PermissionResult>, BearDogError>;
}

/// DID Document
#[derive(Clone, Debug)]
pub struct DidDocument {
    pub id: Did,
    pub verification_methods: Vec<VerificationMethod>,
    pub authentication: Vec<KeyId>,
    pub assertion_method: Vec<KeyId>,
    pub created: u64,
    pub updated: u64,
}

/// Permission check result
#[derive(Clone, Debug)]
pub struct PermissionResult {
    pub allowed: bool,
    pub reason: Option<String>,
    pub expires_at: Option<u64>,
}

/// Permission check request
#[derive(Clone, Debug)]
pub struct PermissionCheck {
    pub resource: String,
    pub action: String,
}
```

### 2.2 Integration Points

| RhizoCrypt Action | BearDog Integration |
|-------------------|---------------------|
| Create session | Verify owner DID |
| Append event | Optional signature verification |
| Checkout slice | Permission check for slice access |
| Resolve session | Create attestations |
| Dehydration | Sign commit summary |

### 2.3 Permission Model

```rust
/// RhizoCrypt permission resources
pub mod permissions {
    // Session permissions
    pub const SESSION_CREATE: &str = "rhizocrypt:session:create";
    pub const SESSION_READ: &str = "rhizocrypt:session:{id}:read";
    pub const SESSION_WRITE: &str = "rhizocrypt:session:{id}:write";
    pub const SESSION_ADMIN: &str = "rhizocrypt:session:{id}:admin";
    
    // Slice permissions
    pub const SLICE_CHECKOUT: &str = "loamspine:{spine}:slice:checkout";
    pub const SLICE_OPERATE: &str = "rhizocrypt:session:{id}:slice:{slice_id}:operate";
    pub const SLICE_RECALL: &str = "loamspine:{spine}:slice:{slice_id}:recall";
}
```

---

## 3. LoamSpine Integration

LoamSpine provides permanent storage for dehydrated sessions.

### 3.1 Client Interface

```rust
/// LoamSpine client for RhizoCrypt
#[async_trait]
pub trait PermanentStorageProvider: Send + Sync {
    // ==================== Entry Operations ====================
    
    /// Append an entry to a spine
    async fn append_entry(
        &self,
        spine_id: &SpineId,
        entry_type: EntryType,
        signer: &impl Signer,
    ) -> Result<LoamCommitRef, LoamError>;
    
    /// Get an entry by hash
    async fn get_entry(
        &self,
        spine_id: &SpineId,
        entry_hash: &EntryHash,
    ) -> Result<Option<LoamEntry>, LoamError>;
    
    /// Get entry by index
    async fn get_entry_by_index(
        &self,
        spine_id: &SpineId,
        index: u64,
    ) -> Result<Option<LoamEntry>, LoamError>;
    
    /// Get the tip entry
    async fn get_tip(&self, spine_id: &SpineId) -> Result<Option<LoamEntry>, LoamError>;
    
    // ==================== Slice Operations ====================
    
    /// Mark an entry as sliced (checked out)
    async fn mark_sliced(
        &self,
        spine_id: &SpineId,
        entry_hash: &EntryHash,
        slice_id: SliceId,
    ) -> Result<(), LoamError>;
    
    /// Clear slice mark
    async fn clear_slice_mark(
        &self,
        spine_id: &SpineId,
        entry_hash: &EntryHash,
        slice_id: SliceId,
    ) -> Result<(), LoamError>;
    
    /// Check if entry is sliced
    async fn is_sliced(
        &self,
        spine_id: &SpineId,
        entry_hash: &EntryHash,
    ) -> Result<Option<SliceId>, LoamError>;
    
    // ==================== Verification ====================
    
    /// Verify a commit exists
    async fn verify_commit(
        &self,
        commit_ref: &LoamCommitRef,
    ) -> Result<bool, LoamError>;
    
    /// Generate inclusion proof
    async fn generate_inclusion_proof(
        &self,
        spine_id: &SpineId,
        entry_hash: &EntryHash,
    ) -> Result<InclusionProof, LoamError>;
    
    // ==================== Spine Management ====================
    
    /// Get spine info
    async fn get_spine(&self, spine_id: &SpineId) -> Result<Option<SpineInfo>, LoamError>;
    
    /// Check if spine exists
    async fn spine_exists(&self, spine_id: &SpineId) -> Result<bool, LoamError>;
}

/// LoamSpine entry (simplified view)
#[derive(Clone, Debug)]
pub struct LoamEntry {
    pub index: u64,
    pub previous: Option<EntryHash>,
    pub timestamp: u64,
    pub committer: Did,
    pub entry_type: EntryType,
    pub payload: Option<PayloadRef>,
    pub signature: Signature,
    pub hash: EntryHash,
}

/// Spine information
#[derive(Clone, Debug)]
pub struct SpineInfo {
    pub id: SpineId,
    pub owner: Did,
    pub height: u64,
    pub tip: EntryHash,
    pub created_at: u64,
}
```

### 3.2 Entry Types

RhizoCrypt creates these entry types in LoamSpine:

```rust
/// Entry types created by RhizoCrypt
pub enum RhizoCryptEntryType {
    /// Dehydrated session commit
    SessionCommit {
        session_id: SessionId,
        session_type: SessionType,
        merkle_root: MerkleRoot,
        summary: DehydrationSummary,
    },
    
    /// Slice checkout record
    SliceCheckout {
        slice_id: SliceId,
        session_id: SessionId,
        mode: SliceMode,
        terms: Option<LoanTerms>,
    },
    
    /// Slice return record
    SliceReturn {
        slice_id: SliceId,
        checkout_entry: EntryHash,
        summary: Option<WaypointSummary>,
    },
    
    /// Slice transfer record
    SliceTransfer {
        slice_id: SliceId,
        from: Did,
        to: Did,
        session_id: SessionId,
    },
}
```

### 3.3 Dehydration Commit Flow

```
RhizoCrypt                              LoamSpine
    │                                       │
    │  1. Compute Merkle root               │
    │  2. Generate summary                  │
    │  3. Resolve slices                    │
    │                                       │
    ├──────── append_entry() ──────────────►│
    │         (SessionCommit)               │
    │                                       ├── Validate entry
    │                                       ├── Append to spine
    │◄─────── LoamCommitRef ───────────────┤
    │                                       │
    │  4. For each resolved slice:          │
    │                                       │
    ├──────── append_entry() ──────────────►│
    │         (SliceReturn/Transfer)        │
    │                                       │
    │  5. Clear slice marks                 │
    │                                       │
    ├──────── clear_slice_mark() ──────────►│
    │                                       │
```

---

## 4. NestGate Integration

NestGate provides content-addressed storage for large payloads.

### 4.1 Client Interface

```rust
/// NestGate client for RhizoCrypt
#[async_trait]
pub trait PayloadStorageProvider: Send + Sync {
    // ==================== Payload Operations ====================
    
    /// Store a payload
    async fn put(
        &self,
        data: Bytes,
        options: PutOptions,
    ) -> Result<PayloadRef, NestGateError>;
    
    /// Get a payload
    async fn get(&self, payload_ref: &PayloadRef) -> Result<Option<Bytes>, NestGateError>;
    
    /// Check if payload exists
    async fn exists(&self, payload_ref: &PayloadRef) -> Result<bool, NestGateError>;
    
    /// Delete a payload (if unreferenced)
    async fn delete(&self, payload_ref: &PayloadRef) -> Result<bool, NestGateError>;
    
    // ==================== Streaming ====================
    
    /// Stream upload for large payloads
    async fn put_stream(
        &self,
        stream: impl Stream<Item = Bytes> + Send,
        options: PutOptions,
    ) -> Result<PayloadRef, NestGateError>;
    
    /// Stream download for large payloads
    async fn get_stream(
        &self,
        payload_ref: &PayloadRef,
    ) -> Result<Option<impl Stream<Item = Bytes>>, NestGateError>;
    
    // ==================== Batch Operations ====================
    
    /// Store multiple payloads
    async fn put_batch(
        &self,
        payloads: Vec<Bytes>,
        options: PutOptions,
    ) -> Result<Vec<PayloadRef>, NestGateError>;
    
    /// Get multiple payloads
    async fn get_batch(
        &self,
        refs: &[PayloadRef],
    ) -> Result<Vec<Option<Bytes>>, NestGateError>;
    
    // ==================== Metadata ====================
    
    /// Get payload metadata
    async fn get_metadata(
        &self,
        payload_ref: &PayloadRef,
    ) -> Result<Option<PayloadMetadata>, NestGateError>;
}

/// Options for storing payloads
#[derive(Clone, Debug, Default)]
pub struct PutOptions {
    /// MIME type
    pub mime_type: Option<String>,
    
    /// Custom metadata
    pub metadata: HashMap<String, String>,
    
    /// Encryption options
    pub encryption: Option<EncryptionOptions>,
    
    /// Replication policy
    pub replication: Option<ReplicationPolicy>,
}

/// Payload metadata
#[derive(Clone, Debug)]
pub struct PayloadMetadata {
    pub hash: ContentHash,
    pub size: u64,
    pub mime_type: Option<String>,
    pub created_at: u64,
    pub metadata: HashMap<String, String>,
}
```

### 4.2 Usage Pattern

```rust
// When appending an event with a large payload

// 1. Store payload in NestGate
let payload_data = large_payload_bytes;
let payload_ref = nestgate.put(payload_data, PutOptions::default()).await?;

// 2. Reference in vertex
let vertex = VertexBuilder::new(EventType::DataCreate { schema: None })
    .with_payload(payload_ref)
    .with_agent(agent_did)
    .build();

// 3. Append vertex to session
let vertex_id = session_manager.append_vertex(session_id, vertex).await?;

// Payload is stored in NestGate, only hash is in DAG
```

### 4.3 GC Coordination

When RhizoCrypt GCs a session, it must coordinate with NestGate:

```rust
/// Coordinate GC with NestGate
async fn gc_payloads(
    session: &Session,
    dag_store: &impl DagStore,
    nestgate: &impl PayloadStorageProvider,
) -> Result<GcPayloadStats, RhizoCryptError> {
    // Collect all payload refs in the session
    let payload_refs = collect_payload_refs(session, dag_store).await?;
    
    // Check which payloads are only referenced by this session
    let orphaned = find_orphaned_payloads(&payload_refs, nestgate).await?;
    
    // Delete orphaned payloads
    let mut deleted = 0;
    let mut bytes_freed = 0;
    
    for payload_ref in orphaned {
        if let Some(meta) = nestgate.get_metadata(&payload_ref).await? {
            bytes_freed += meta.size;
        }
        if nestgate.delete(&payload_ref).await? {
            deleted += 1;
        }
    }
    
    Ok(GcPayloadStats { deleted, bytes_freed })
}
```

---

## 5. Songbird Integration

Songbird provides service discovery and routing.

### 5.1 UPA Registration

```rust
/// Register RhizoCrypt with Songbird UPA
pub async fn register_with_songbird(
    rhizocrypt: &RhizoCrypt,
    songbird: &impl SongbirdClient,
) -> Result<RegistrationReceipt, SongbirdError> {
    let capabilities = vec![
        Capability::new("rhizocrypt:session:create"),
        Capability::new("rhizocrypt:session:read"),
        Capability::new("rhizocrypt:session:write"),
        Capability::new("rhizocrypt:session:admin"),
        Capability::new("rhizocrypt:slice:checkout"),
        Capability::new("rhizocrypt:merkle:compute"),
        Capability::new("rhizocrypt:merkle:prove"),
    ];
    
    let service_info = ServiceInfo {
        name: "rhizocrypt".to_string(),
        version: rhizocrypt.version().to_string(),
        capabilities,
        endpoints: vec![
            Endpoint::Grpc {
                host: rhizocrypt.grpc_host(),
                port: rhizocrypt.grpc_port(),
            },
            Endpoint::Rest {
                base_url: rhizocrypt.rest_url(),
            },
        ],
        health_check: Some(HealthCheck {
            endpoint: "/health".to_string(),
            interval: Duration::from_secs(30),
        }),
    };
    
    songbird.register(service_info).await
}
```

### 5.2 Service Discovery

```rust
/// Songbird client for RhizoCrypt
#[async_trait]
pub trait SongbirdClient: Send + Sync {
    /// Register a service
    async fn register(&self, service: ServiceInfo) -> Result<RegistrationReceipt, SongbirdError>;
    
    /// Deregister a service
    async fn deregister(&self, service_id: &str) -> Result<(), SongbirdError>;
    
    /// Discover services by capability
    async fn discover(
        &self,
        capability: &str,
    ) -> Result<Vec<ServiceEndpoint>, SongbirdError>;
    
    /// Get a specific service
    async fn get_service(&self, name: &str) -> Result<Option<ServiceInfo>, SongbirdError>;
}
```

---

## 6. ToadStool Integration

ToadStool provides compute orchestration and event sourcing.

### 6.1 Event Source

```rust
/// ToadStool event source for RhizoCrypt
#[async_trait]
pub trait ToadStoolEventSource: Send + Sync {
    /// Subscribe to compute task events
    fn subscribe_task(&self, task_id: TaskId) -> impl Stream<Item = ComputeEvent>;
    
    /// Subscribe to all events for an agent
    fn subscribe_agent(&self, agent: &Did) -> impl Stream<Item = ComputeEvent>;
    
    /// Get events in a time range
    async fn get_events(
        &self,
        task_id: TaskId,
        start: u64,
        end: u64,
    ) -> Result<Vec<ComputeEvent>, ToadStoolError>;
}

/// Compute events from ToadStool
#[derive(Clone, Debug)]
pub enum ComputeEvent {
    TaskCreated {
        task_id: TaskId,
        task_type: String,
        requester: Did,
    },
    TaskStarted {
        task_id: TaskId,
        worker: Did,
        started_at: u64,
    },
    TaskProgress {
        task_id: TaskId,
        progress: f32,
        message: Option<String>,
    },
    TaskCompleted {
        task_id: TaskId,
        result_ref: PayloadRef,
        completed_at: u64,
    },
    TaskFailed {
        task_id: TaskId,
        error: String,
        failed_at: u64,
    },
}
```

### 6.2 Compute Session Binding

```rust
/// Bind a compute task to a RhizoCrypt session
pub async fn bind_compute_task(
    session_id: SessionId,
    task_id: TaskId,
    toadstool: &impl ToadStoolEventSource,
    session_manager: &SessionManager,
) -> Result<(), RhizoCryptError> {
    // Subscribe to task events
    let mut events = toadstool.subscribe_task(task_id);
    
    // Forward events to session
    while let Some(event) = events.next().await {
        let vertex = match &event {
            ComputeEvent::TaskCreated { task_type, requester, .. } => {
                VertexBuilder::new(EventType::Custom {
                    domain: "toadstool".to_string(),
                    event_name: "task.created".to_string(),
                })
                .with_metadata("task_type", task_type.clone())
                .with_agent(requester.clone())
                .build()
            }
            ComputeEvent::TaskCompleted { result_ref, .. } => {
                VertexBuilder::new(EventType::Custom {
                    domain: "toadstool".to_string(),
                    event_name: "task.completed".to_string(),
                })
                .with_payload(*result_ref)
                .build()
            }
            // ... other event types
            _ => continue,
        };
        
        session_manager.append_vertex(session_id, vertex).await?;
    }
    
    Ok(())
}
```

---

## 7. SweetGrass Integration

SweetGrass queries RhizoCrypt for provenance information.

### 7.1 Query Interface

```rust
/// SweetGrass query interface for RhizoCrypt
#[async_trait]
pub trait SweetGrassQueryable: Send + Sync {
    /// Get all vertices related to a data hash
    async fn get_vertices_for_data(
        &self,
        data_hash: ContentHash,
    ) -> Result<Vec<VertexRef>, RhizoCryptError>;
    
    /// Get provenance chain for a vertex
    async fn get_provenance_chain(
        &self,
        vertex_id: VertexId,
    ) -> Result<ProvenanceChain, RhizoCryptError>;
    
    /// Query vertices by agent and event type
    async fn query_by_agent(
        &self,
        agent: &Did,
        event_types: Option<Vec<String>>,
        limit: usize,
    ) -> Result<Vec<VertexRef>, RhizoCryptError>;
    
    /// Get session summary for attribution
    async fn get_session_attribution(
        &self,
        session_id: SessionId,
    ) -> Result<SessionAttribution, RhizoCryptError>;
}

/// Reference to a vertex (for SweetGrass)
#[derive(Clone, Debug)]
pub struct VertexRef {
    pub session_id: SessionId,
    pub vertex_id: VertexId,
    pub event_type: EventType,
    pub agent: Option<Did>,
    pub timestamp: u64,
    pub payload_ref: Option<PayloadRef>,
}

/// Provenance chain
#[derive(Clone, Debug)]
pub struct ProvenanceChain {
    pub vertices: Vec<VertexRef>,
    pub agents: HashSet<Did>,
    pub data_hashes: HashSet<ContentHash>,
}

/// Session attribution info for SweetGrass
#[derive(Clone, Debug)]
pub struct SessionAttribution {
    pub session_id: SessionId,
    pub session_type: SessionType,
    pub agents: Vec<AgentContribution>,
    pub data_inputs: Vec<ContentHash>,
    pub data_outputs: Vec<ContentHash>,
    pub merkle_root: MerkleRoot,
}

/// Agent contribution to a session
#[derive(Clone, Debug)]
pub struct AgentContribution {
    pub agent: Did,
    pub event_count: u64,
    pub event_types: Vec<String>,
    pub first_event: u64,
    pub last_event: u64,
}
```

---

## 8. Adapter Pattern

All integrations use a common adapter pattern:

```rust
/// Generic primal adapter
pub struct PrimalAdapter<C> {
    client: C,
    config: AdapterConfig,
    metrics: AdapterMetrics,
}

impl<C> PrimalAdapter<C> {
    pub fn new(client: C, config: AdapterConfig) -> Self {
        Self {
            client,
            config,
            metrics: AdapterMetrics::default(),
        }
    }
}

/// Adapter configuration
#[derive(Clone, Debug)]
pub struct AdapterConfig {
    /// Retry configuration
    pub retry: RetryConfig,
    
    /// Circuit breaker configuration
    pub circuit_breaker: CircuitBreakerConfig,
    
    /// Timeout
    pub timeout: Duration,
    
    /// Enable caching
    pub cache_enabled: bool,
    
    /// Cache TTL
    pub cache_ttl: Duration,
}

/// Retry configuration
#[derive(Clone, Debug)]
pub struct RetryConfig {
    pub max_attempts: u32,
    pub base_delay: Duration,
    pub max_delay: Duration,
    pub exponential_base: f64,
}

/// Circuit breaker configuration
#[derive(Clone, Debug)]
pub struct CircuitBreakerConfig {
    pub failure_threshold: u32,
    pub success_threshold: u32,
    pub timeout: Duration,
}
```

---

## 9. Error Handling

### 9.1 Integration Errors

```rust
/// Errors from primal integrations
#[derive(Debug, thiserror::Error)]
pub enum IntegrationError {
    #[error("BearDog error: {0}")]
    BearDog(#[from] BearDogError),
    
    #[error("LoamSpine error: {0}")]
    LoamSpine(#[from] LoamError),
    
    #[error("NestGate error: {0}")]
    NestGate(#[from] NestGateError),
    
    #[error("Songbird error: {0}")]
    Songbird(#[from] SongbirdError),
    
    #[error("ToadStool error: {0}")]
    ToadStool(#[from] ToadStoolError),
    
    #[error("Connection failed: {0}")]
    Connection(String),
    
    #[error("Timeout after {0:?}")]
    Timeout(Duration),
    
    #[error("Circuit breaker open")]
    CircuitBreakerOpen,
}
```

---

## 10. References

- [ARCHITECTURE.md](./ARCHITECTURE.md) — System architecture
- [API_SPECIFICATION.md](./API_SPECIFICATION.md) — API definitions
- [BearDog Specification](../../../beardog/specs/) — Identity primal
- [LoamSpine Specification](../../loamSpine/specs/) — Permanence primal
- [NestGate Specification](../../../nestgate/specs/) — Storage primal
- [Songbird Specification](../../../songbird/specs/) — Discovery primal
- [ToadStool Specification](../../../toadstool/specs/) — Compute primal

---

*RhizoCrypt: The memory that knows when to forget.*

