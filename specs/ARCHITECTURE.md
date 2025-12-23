# RhizoCrypt — Architecture Specification

**Version**: 0.2.0  
**Status**: Draft  
**Last Updated**: December 22, 2025

---

## 1. Overview

RhizoCrypt is the **ephemeral DAG engine** of the ecoPrimals ecosystem. It provides working memory for complex, branching operations that eventually resolve to permanent state in LoamSpine.

### 1.1 Position in the Stack

```
┌─────────────────────────────────────────────────────────────────┐
│                         Applications                             │
│        (Games, Scientific Tools, Collaboration Apps)            │
└─────────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────────┐
│                        SweetGrass 🌾                             │
│                    (Attribution Layer)                           │
│              Queries DAG, builds provenance braids              │
└─────────────────────────────────────────────────────────────────┘
                              │
            ┌─────────────────┼─────────────────┐
            │                 │                 │
            ▼                 ▼                 ▼
┌───────────────────┐ ┌─────────────────┐ ┌─────────────────────┐
│   RhizoCrypt 🔐   │ │  LoamSpine 🦴   │ │     NestGate 🏠     │
│   (Ephemeral DAG) │ │ (Permanent Lin) │ │  (Payload Storage)  │
│                   │ │                 │ │                     │
│ Working memory    │ │ Fossil record   │ │ Large blob storage  │
│ Branching ops     │ │ Certificates    │ │ Content-addressed   │
│ Slice checkout    │ │ Slice anchoring │ │                     │
└─────────┬─────────┘ └────────┬────────┘ └─────────────────────┘
          │                    │
          │  Dehydration       │
          └────────────────────┘
                    │
                    ▼
┌─────────────────────────────────────────────────────────────────┐
│                        BearDog 🐻                                │
│                   (Identity & Security)                          │
│              DIDs, Signatures, Policy Enforcement               │
└─────────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────────┐
│                        Songbird 🐦                               │
│                   (Service Discovery)                            │
│              UPA Registration, Capability Routing               │
└─────────────────────────────────────────────────────────────────┘
```

### 1.2 Core Responsibilities

| Responsibility | Description |
|----------------|-------------|
| **DAG Storage** | Store and index vertices with content-addressing |
| **Session Management** | Lifecycle of scoped DAGs (create, grow, resolve, expire) |
| **Event Ingestion** | High-throughput append path for real-time events |
| **Merkle Proofs** | Generate cryptographic proofs of vertex inclusion |
| **Dehydration** | Commit DAG summaries to LoamSpine |
| **Slice Handling** | Check out, route, and resolve LoamSpine slices |
| **Garbage Collection** | Clean up expired sessions and orphaned data |

---

## 2. Component Architecture

### 2.1 High-Level Components

```
┌─────────────────────────────────────────────────────────────────┐
│                      RhizoCrypt Service                          │
├─────────────────────────────────────────────────────────────────┤
│                                                                  │
│  ┌─────────────────┐  ┌─────────────────┐  ┌─────────────────┐  │
│  │ Session Manager │  │  Event Ingester │  │  Slice Router   │  │
│  │                 │  │                 │  │                 │  │
│  │ Create/Resolve  │  │ High-throughput │  │ Checkout/Commit │  │
│  │ Lifecycle mgmt  │  │ Batch append    │  │ Resolution      │  │
│  └────────┬────────┘  └────────┬────────┘  └────────┬────────┘  │
│           │                    │                    │           │
│           └────────────────────┼────────────────────┘           │
│                                │                                │
│  ┌─────────────────────────────▼─────────────────────────────┐  │
│  │                        DAG Core                            │  │
│  │  ┌─────────────┐  ┌─────────────┐  ┌─────────────────┐    │  │
│  │  │   Vertex    │  │    Index    │  │   Merkle Tree   │    │  │
│  │  │   Store     │  │   Service   │  │    Builder      │    │  │
│  │  └─────────────┘  └─────────────┘  └─────────────────┘    │  │
│  └───────────────────────────────────────────────────────────┘  │
│                                │                                │
│  ┌─────────────────────────────▼─────────────────────────────┐  │
│  │                     Storage Layer                          │  │
│  │  ┌─────────────┐  ┌─────────────┐  ┌─────────────────┐    │  │
│  │  │  In-Memory  │  │   RocksDB   │  │      LMDB       │    │  │
│  │  │   Store     │  │    Store    │  │     Store       │    │  │
│  │  └─────────────┘  └─────────────┘  └─────────────────┘    │  │
│  └───────────────────────────────────────────────────────────┘  │
│                                │                                │
│  ┌─────────────────────────────▼─────────────────────────────┐  │
│  │                  Dehydration Engine                        │  │
│  │                                                            │  │
│  │  Summary generation → LoamSpine commit → GC trigger       │  │
│  └───────────────────────────────────────────────────────────┘  │
│                                                                  │
└──────────────────────────────────────────────────────────────────┘
                                 │
          ┌──────────────────────┼──────────────────────┐
          │                      │                      │
          ▼                      ▼                      ▼
    ┌──────────┐           ┌──────────┐          ┌──────────┐
    │ BearDog  │           │LoamSpine │          │ NestGate │
    │   🐻     │           │   🦴     │          │   🏠     │
    │ Signing  │           │ Commits  │          │ Payloads │
    └──────────┘           └──────────┘          └──────────┘
```

### 2.2 Component Descriptions

#### Session Manager
Manages the lifecycle of RhizoCrypt sessions:
- **Create**: Initialize new DAG with configuration
- **Configure**: Set TTL, max vertices, signature requirements
- **Monitor**: Track session health and resource usage
- **Resolve**: Trigger dehydration and cleanup
- **Expire**: Garbage collect after TTL

#### Event Ingester
High-performance event append path:
- **Single append**: Low-latency single event
- **Batch append**: High-throughput batch operations
- **Parent detection**: Auto-link to frontier if parents not specified
- **Signature validation**: Verify BearDog signatures inline
- **Backpressure**: Handle overload gracefully

#### Slice Router
Manages LoamSpine slice operations:
- **Checkout**: Extract slice from LoamSpine into DAG
- **Track**: Monitor slice through DAG operations
- **Route**: Determine resolution path based on mode
- **Resolve**: Execute COMMIT, ROLLBACK, or WAYPOINT

#### DAG Core
The heart of RhizoCrypt:
- **Vertex Store**: Content-addressed storage of vertices
- **Index Service**: Parent/child lookups, frontier tracking
- **Merkle Tree Builder**: Construct proofs on demand

#### Storage Layer
Pluggable storage backends:
- **In-Memory**: Fast, ephemeral, for short sessions
- **RocksDB**: Persistent, for longer sessions
- **LMDB**: Memory-mapped, balance of speed and durability

#### Dehydration Engine
Commits DAG results to LoamSpine:
- **Summary generation**: Extract key results from DAG
- **Merkle root**: Compute cryptographic summary
- **LoamSpine commit**: Write entry to permanent ledger
- **GC trigger**: Signal session for cleanup

---

## 3. Data Flow

### 3.1 Event Append Flow

```
     Application
          │
          │ AppendEvent(session, event, payload)
          ▼
   ┌──────────────┐
   │Event Ingester│
   └──────┬───────┘
          │
          │ 1. Validate session active
          │ 2. Compute content hash
          │ 3. Detect parents (frontier)
          │ 4. Verify signature (if required)
          ▼
   ┌──────────────┐
   │  DAG Core    │
   └──────┬───────┘
          │
          │ 5. Store vertex
          │ 6. Update indexes
          │ 7. Update frontier
          ▼
   ┌──────────────┐
   │Storage Layer │
   └──────┬───────┘
          │
          │ 8. Persist (backend-specific)
          ▼
       VertexId
```

### 3.2 Dehydration Flow

```
      Resolve Trigger
      (manual or timeout)
             │
             ▼
    ┌────────────────┐
    │ Session Manager│
    └────────┬───────┘
             │
             │ 1. Set state = Resolving
             ▼
    ┌────────────────┐
    │Dehydration Eng │
    └────────┬───────┘
             │
             │ 2. Compute Merkle root
             │ 3. Generate summary
             │ 4. Collect attestations
             ▼
    ┌────────────────┐
    │  Slice Router  │
    └────────┬───────┘
             │
             │ 5. Resolve each slice
             │    (COMMIT/ROLLBACK/WAYPOINT)
             ▼
    ┌────────────────┐
    │   LoamSpine    │
    └────────┬───────┘
             │
             │ 6. Write SessionCommit entry
             │ 7. Write slice resolutions
             ▼
    ┌────────────────┐
    │ Session Manager│
    └────────┬───────┘
             │
             │ 8. Set state = Committed
             │ 9. Schedule GC
             ▼
          Complete
```

### 3.3 Slice Checkout Flow

```
      Application
           │
           │ CheckoutSlice(spine, entry, mode)
           ▼
    ┌─────────────┐
    │Slice Router │
    └──────┬──────┘
           │
           │ 1. Validate ownership/permissions
           │ 2. Create SliceRef
           ▼
    ┌─────────────┐
    │  LoamSpine  │
    └──────┬──────┘
           │
           │ 3. Read slice origin
           │ 4. Lock/mark as checked out
           ▼
    ┌─────────────┐
    │Slice Router │
    └──────┬──────┘
           │
           │ 5. Create SliceCheckout vertex
           │ 6. Attach to session
           ▼
    ┌─────────────┐
    │   Session   │
    └──────┬──────┘
           │
           │ Session now tracks slice
           │ All ops on slice recorded
           ▼
        SliceId
```

---

## 4. Crate Structure

```
rhizoCrypt/
├── Cargo.toml                    # Workspace manifest
├── crates/
│   ├── rhizo-crypt-core/         # Core library
│   │   ├── src/
│   │   │   ├── lib.rs            # Main entry, re-exports
│   │   │   ├── config.rs         # Configuration types
│   │   │   ├── error.rs          # Error types
│   │   │   ├── vertex.rs         # Vertex data structure
│   │   │   ├── session.rs        # Session management
│   │   │   ├── dag.rs            # DAG operations
│   │   │   ├── merkle.rs         # Merkle tree construction
│   │   │   └── slice.rs          # Slice semantics
│   │   └── Cargo.toml
│   │
│   ├── rhizo-crypt-store/        # Storage backends
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── traits.rs         # DagStore, PayloadStore traits
│   │   │   ├── memory.rs         # In-memory implementation
│   │   │   ├── rocksdb.rs        # RocksDB implementation
│   │   │   └── lmdb.rs           # LMDB implementation
│   │   └── Cargo.toml
│   │
│   ├── rhizo-crypt-dehydrate/    # Dehydration engine
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── summary.rs        # Summary generation
│   │   │   ├── commit.rs         # LoamSpine commit
│   │   │   └── gc.rs             # Garbage collection
│   │   └── Cargo.toml
│   │
│   ├── rhizo-crypt-api/          # API layer
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── grpc.rs           # gRPC service
│   │   │   └── rest.rs           # REST handlers
│   │   ├── proto/
│   │   │   └── rhizocrypt.proto  # gRPC definitions
│   │   └── Cargo.toml
│   │
│   └── rhizo-crypt-service/      # Runnable service
│       ├── src/
│       │   └── main.rs           # Service entry point
│       └── Cargo.toml
│
├── specs/                        # Specifications
├── showcase/                     # Demo applications
└── tests/                        # Integration tests
```

---

## 5. Thread Model

### 5.1 Async Runtime

RhizoCrypt uses Tokio as its async runtime:

```
┌───────────────────────────────────────────────────────────────┐
│                     Tokio Runtime                             │
├───────────────────────────────────────────────────────────────┤
│                                                               │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────────────┐   │
│  │ gRPC Server │  │ REST Server │  │  Background Tasks   │   │
│  │   (tonic)   │  │   (axum)    │  │                     │   │
│  └──────┬──────┘  └──────┬──────┘  │  - Session timeout  │   │
│         │                │         │  - GC sweep         │   │
│         │                │         │  - Metrics emit     │   │
│         └────────┬───────┘         └─────────────────────┘   │
│                  │                                            │
│                  ▼                                            │
│  ┌────────────────────────────────────────────────────────┐  │
│  │              Request Handler Pool                       │  │
│  │                                                         │  │
│  │   Each request handled on Tokio task                   │  │
│  │   CPU-bound work spawned to blocking pool              │  │
│  └────────────────────────────────────────────────────────┘  │
│                                                               │
└───────────────────────────────────────────────────────────────┘
```

### 5.2 Concurrency Model

| Component | Concurrency Strategy |
|-----------|---------------------|
| Session Manager | `RwLock<HashMap<SessionId, Session>>` |
| DAG Store | Per-session locks, concurrent reads |
| Event Ingester | Lock-free append with atomic frontier |
| Merkle Builder | Lazy computation, cached results |
| Dehydration | Single-threaded per session |

---

## 6. Error Handling

### 6.1 Error Hierarchy

```rust
#[derive(Debug, thiserror::Error)]
pub enum RhizoCryptError {
    // Session errors
    #[error("Session not found: {0}")]
    SessionNotFound(SessionId),
    
    #[error("Session not active: {0}")]
    SessionNotActive(SessionId),
    
    #[error("Session expired: {0}")]
    SessionExpired(SessionId),
    
    // Vertex errors
    #[error("Vertex not found: {0:?}")]
    VertexNotFound(VertexId),
    
    #[error("Invalid parent reference: {0:?}")]
    InvalidParent(VertexId),
    
    #[error("Signature verification failed")]
    SignatureVerificationFailed,
    
    // Slice errors
    #[error("Slice not found: {0}")]
    SliceNotFound(SliceId),
    
    #[error("Slice already resolved")]
    SliceAlreadyResolved,
    
    #[error("Slice permission denied")]
    SlicePermissionDenied,
    
    // Storage errors
    #[error("Storage error: {0}")]
    Storage(#[from] StorageError),
    
    // Integration errors
    #[error("BearDog error: {0}")]
    BearDog(String),
    
    #[error("LoamSpine error: {0}")]
    LoamSpine(String),
    
    // Internal errors
    #[error("Internal error: {0}")]
    Internal(String),
}
```

### 6.2 Error Recovery

| Error Type | Recovery Strategy |
|------------|-------------------|
| `SessionNotFound` | Return error, client creates new |
| `SessionExpired` | Return error with last known state |
| `VertexNotFound` | Return error, client may retry |
| `SignatureVerificationFailed` | Reject event, log for audit |
| `SlicePermissionDenied` | Return error, no retry |
| `Storage` | Retry with backoff, escalate if persistent |

---

## 7. Observability

### 7.1 Metrics

```rust
// Session metrics
rhizocrypt_sessions_active: Gauge
rhizocrypt_sessions_created_total: Counter
rhizocrypt_sessions_resolved_total: Counter { outcome = "commit|rollback|expire" }

// Event metrics
rhizocrypt_events_appended_total: Counter
rhizocrypt_event_append_latency_seconds: Histogram
rhizocrypt_events_per_session: Histogram

// Storage metrics
rhizocrypt_vertices_stored_total: Counter
rhizocrypt_payload_bytes_stored_total: Counter
rhizocrypt_storage_read_latency_seconds: Histogram
rhizocrypt_storage_write_latency_seconds: Histogram

// Slice metrics
rhizocrypt_slices_checked_out_total: Counter
rhizocrypt_slices_resolved_total: Counter { resolution = "commit|rollback|waypoint" }

// GC metrics
rhizocrypt_gc_sessions_cleaned: Counter
rhizocrypt_gc_bytes_reclaimed: Counter
rhizocrypt_gc_duration_seconds: Histogram
```

### 7.2 Tracing

All operations emit OpenTelemetry spans:

```
rhizocrypt.session.create
rhizocrypt.session.resolve
rhizocrypt.event.append
rhizocrypt.merkle.compute
rhizocrypt.dehydrate.commit
rhizocrypt.slice.checkout
rhizocrypt.slice.resolve
rhizocrypt.gc.sweep
```

### 7.3 Health Checks

```rust
impl PrimalHealth for RhizoCrypt {
    async fn check_health(&self) -> HealthReport {
        HealthReport::new("rhizocrypt")
            .with_status(self.compute_status().await)
            .with_component("session_manager", self.session_manager.health())
            .with_component("dag_store", self.dag_store.health())
            .with_component("loamspine_client", self.loamspine.health())
            .with_metric("active_sessions", self.session_count())
            .with_metric("stored_vertices", self.vertex_count())
    }
}
```

---

## 8. Security Model

### 8.1 Authentication

- All API requests must include BearDog authentication
- Session creation requires valid DID
- Event signing is optional but recommended

### 8.2 Authorization

| Operation | Required Permission |
|-----------|---------------------|
| Create session | `rhizocrypt:session:create` |
| Append event | `rhizocrypt:session:{id}:write` |
| Read session | `rhizocrypt:session:{id}:read` |
| Resolve session | `rhizocrypt:session:{id}:admin` |
| Checkout slice | `loamspine:slice:{id}:checkout` |

### 8.3 Data Protection

- Vertices are tamper-evident (content-addressed)
- Sessions can require signatures for all events
- Payloads can be encrypted at rest
- GC permanently deletes data (no recovery)

---

## 9. Deployment Modes

### 9.1 Embedded Mode

RhizoCrypt runs in-process with the application:

```rust
let rhizo = RhizoCrypt::embedded()
    .with_store(InMemoryStore::new())
    .build()?;

let session = rhizo.create_session(config).await?;
```

### 9.2 Service Mode

RhizoCrypt runs as a standalone service:

```bash
rhizo-crypt-service \
    --config /etc/rhizocrypt/config.toml \
    --grpc-addr 0.0.0.0:50051 \
    --rest-addr 0.0.0.0:8080
```

### 9.3 Federated Mode

Multiple RhizoCrypt instances share sessions:

```
┌─────────────────┐     ┌─────────────────┐
│   RhizoCrypt    │────▶│   RhizoCrypt    │
│    Node A       │◀────│    Node B       │
└────────┬────────┘     └────────┬────────┘
         │                       │
         └───────────┬───────────┘
                     │
                     ▼
              ┌─────────────┐
              │  Songbird   │
              │  Discovery  │
              └─────────────┘
```

---

## 10. References

- [RHIZOCRYPT_SPECIFICATION.md](./RHIZOCRYPT_SPECIFICATION.md) — Full specification
- [DATA_MODEL.md](./DATA_MODEL.md) — Data structures
- [SLICE_SEMANTICS.md](./SLICE_SEMANTICS.md) — Slice protocol
- [DEHYDRATION_PROTOCOL.md](./DEHYDRATION_PROTOCOL.md) — Commit protocol
- [API_SPECIFICATION.md](./API_SPECIFICATION.md) — API definitions

---

*RhizoCrypt: The memory that knows when to forget.*

