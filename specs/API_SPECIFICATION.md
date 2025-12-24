# RhizoCrypt — API Specification

**Version**: 0.3.0  
**Status**: Complete  
**Last Updated**: December 22, 2025

---

## 1. Overview

RhizoCrypt exposes a **pure Rust RPC** interface using tarpc. This follows the ecoPrimals commitment to:
- **Compile-time type safety** — The Rust compiler verifies all RPC calls
- **No code generation** — No `.proto` files, no `protoc`
- **Lean into Rust** — Traits define the API, macros generate client/server

### 1.1 Why tarpc, Not gRPC?

| Aspect | gRPC | tarpc |
|--------|------|-------|
| Schema | External `.proto` files | Rust traits |
| Codegen | `protoc` + plugins | Rust macros |
| Type safety | Runtime | Compile-time |
| Ecosystem | Language-agnostic | Rust-native |
| Overhead | Schema parsing | Zero |
| Debugging | Opaque errors | Rust stack traces |

For a pure Rust ecosystem like ecoPrimals, tarpc provides better ergonomics and safety.

---

## 2. tarpc Service Definition

The complete RPC interface is defined as a Rust trait:

```rust
/// RhizoCrypt RPC service.
///
/// The `#[tarpc::service]` macro generates:
/// - `RhizoCryptRpcClient` — async client stub
/// - Server trait to implement
///
/// All types are checked at compile time.
#[tarpc::service]
pub trait RhizoCryptRpc {
    // ========================================================================
    // Session Operations
    // ========================================================================

    /// Create a new session.
    async fn create_session(request: CreateSessionRequest) -> Result<SessionId, RpcError>;

    /// Get session info.
    async fn get_session(session_id: SessionId) -> Result<SessionInfo, RpcError>;

    /// List all active sessions.
    async fn list_sessions() -> Result<Vec<SessionInfo>, RpcError>;

    /// Discard a session (delete without committing).
    async fn discard_session(session_id: SessionId) -> Result<(), RpcError>;

    // ========================================================================
    // Event Operations
    // ========================================================================

    /// Append an event to a session.
    async fn append_event(request: AppendEventRequest) -> Result<VertexId, RpcError>;

    /// Append multiple events in a batch.
    async fn append_batch(requests: Vec<AppendEventRequest>) -> Result<Vec<VertexId>, RpcError>;

    // ========================================================================
    // Query Operations
    // ========================================================================

    /// Get a specific vertex by ID.
    async fn get_vertex(session_id: SessionId, vertex_id: VertexId) -> Result<Vertex, RpcError>;

    /// Get the current frontier (DAG tips).
    async fn get_frontier(session_id: SessionId) -> Result<Vec<VertexId>, RpcError>;

    /// Get genesis vertices (DAG roots).
    async fn get_genesis(session_id: SessionId) -> Result<Vec<VertexId>, RpcError>;

    /// Query vertices with filters.
    async fn query_vertices(request: QueryRequest) -> Result<Vec<Vertex>, RpcError>;

    /// Get children of a vertex.
    async fn get_children(session_id: SessionId, vertex_id: VertexId) -> Result<Vec<VertexId>, RpcError>;

    // ========================================================================
    // Merkle Operations
    // ========================================================================

    /// Get the Merkle root for a session.
    async fn get_merkle_root(session_id: SessionId) -> Result<MerkleRoot, RpcError>;

    /// Generate inclusion proof for a vertex.
    async fn get_merkle_proof(session_id: SessionId, vertex_id: VertexId) -> Result<MerkleProof, RpcError>;

    /// Verify a Merkle proof.
    async fn verify_proof(root: MerkleRoot, proof: MerkleProof) -> Result<bool, RpcError>;

    // ========================================================================
    // Slice Operations
    // ========================================================================

    /// Checkout a slice from LoamSpine.
    async fn checkout_slice(request: CheckoutSliceRequest) -> Result<SliceId, RpcError>;

    /// Get slice info.
    async fn get_slice(slice_id: SliceId) -> Result<Slice, RpcError>;

    /// List active slices.
    async fn list_slices() -> Result<Vec<Slice>, RpcError>;

    /// Resolve a slice (commit back to LoamSpine).
    async fn resolve_slice(slice_id: SliceId, session_id: SessionId) -> Result<(), RpcError>;

    // ========================================================================
    // Dehydration Operations
    // ========================================================================

    /// Trigger dehydration of a session to LoamSpine.
    async fn dehydrate(session_id: SessionId) -> Result<MerkleRoot, RpcError>;

    /// Get dehydration status.
    async fn get_dehydration_status(session_id: SessionId) -> Result<DehydrationStatus, RpcError>;

    // ========================================================================
    // Health & Metrics
    // ========================================================================

    /// Health check.
    async fn health() -> Result<HealthStatus, RpcError>;

    /// Get service metrics.
    async fn metrics() -> Result<ServiceMetrics, RpcError>;
}
```

---

## 3. Request/Response Types

All types are defined in pure Rust with serde derives for serialization:

### 3.1 Session Types

```rust
/// Session creation request.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateSessionRequest {
    /// Session type.
    pub session_type: SessionType,
    /// Optional description.
    pub description: Option<String>,
    /// Optional parent session.
    pub parent_session: Option<SessionId>,
    /// Maximum vertices allowed.
    pub max_vertices: Option<u64>,
    /// TTL in seconds.
    pub ttl_seconds: Option<u64>,
}

/// Session info response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionInfo {
    /// Session ID.
    pub id: SessionId,
    /// Session type.
    pub session_type: SessionType,
    /// Current state.
    pub state: SessionState,
    /// Vertex count.
    pub vertex_count: u64,
    /// Creation time.
    pub created_at: Timestamp,
    /// Description.
    pub description: Option<String>,
}
```

### 3.2 Event Types

```rust
/// Event append request.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppendEventRequest {
    /// Target session.
    pub session_id: SessionId,
    /// Event type.
    pub event_type: EventType,
    /// Agent DID.
    pub agent: Option<Did>,
    /// Parent vertices (empty = use frontier).
    pub parents: Vec<VertexId>,
    /// Metadata key-value pairs.
    pub metadata: Vec<(String, String)>,
    /// Optional payload reference.
    pub payload_ref: Option<String>,
}
```

### 3.3 Query Types

```rust
/// Query request for vertices.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryRequest {
    /// Session to query.
    pub session_id: SessionId,
    /// Filter by event types.
    pub event_types: Option<Vec<EventType>>,
    /// Filter by agent.
    pub agent: Option<Did>,
    /// Start time filter.
    pub start_time: Option<Timestamp>,
    /// End time filter.
    pub end_time: Option<Timestamp>,
    /// Maximum results.
    pub limit: Option<u32>,
}
```

### 3.4 Slice Types

```rust
/// Slice checkout request.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CheckoutSliceRequest {
    /// Source spine index.
    pub spine_index: u64,
    /// Slice mode.
    pub mode: SliceMode,
    /// Lender DID (for loans).
    pub lender: Option<Did>,
    /// Borrower DID (for loans).
    pub borrower: Option<Did>,
    /// Duration in seconds.
    pub duration_seconds: Option<u64>,
}
```

### 3.5 Health Types

```rust
/// Health status response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthStatus {
    /// Whether the service is healthy.
    pub healthy: bool,
    /// Current state description.
    pub state: String,
    /// Active session count.
    pub active_sessions: u64,
    /// Total vertices in memory.
    pub total_vertices: u64,
    /// Uptime in seconds.
    pub uptime_seconds: u64,
}

/// Service metrics.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceMetrics {
    /// Sessions created.
    pub sessions_created: u64,
    /// Sessions resolved.
    pub sessions_resolved: u64,
    /// Vertices appended.
    pub vertices_appended: u64,
    /// Queries executed.
    pub queries_executed: u64,
    /// Slices checked out.
    pub slices_checked_out: u64,
    /// Dehydrations completed.
    pub dehydrations_completed: u64,
}
```

---

## 4. Error Types

RPC errors are strongly typed:

```rust
/// RPC-specific errors.
#[derive(Debug, Clone, Error, Serialize, Deserialize)]
pub enum RpcError {
    /// Core library error.
    #[error("core error: {0}")]
    Core(String),

    /// Transport error.
    #[error("transport error: {0}")]
    Transport(String),

    /// Connection error.
    #[error("connection error: {0}")]
    Connection(String),

    /// Timeout error.
    #[error("timeout: {0}")]
    Timeout(String),

    /// Session not found.
    #[error("session not found: {0}")]
    SessionNotFound(String),

    /// Vertex not found.
    #[error("vertex not found: {0}")]
    VertexNotFound(String),

    /// Internal error.
    #[error("internal error: {0}")]
    Internal(String),
}
```

---

## 5. Client Usage

The generated client provides an ergonomic async API:

```rust
use rhizo_crypt_rpc::RpcClient;

async fn example() -> Result<(), Box<dyn std::error::Error>> {
    // Connect to server
    let client = RpcClient::connect("127.0.0.1:9400".parse()?).await?;

    // Create a session
    let session_id = client.create_session(CreateSessionRequest {
        session_type: SessionType::Gaming { game_id: "raid-42".into() },
        description: Some("Boss raid".into()),
        ..Default::default()
    }).await?;

    // Append an event
    let vertex_id = client.append_event(AppendEventRequest {
        session_id: session_id.clone(),
        event_type: EventType::ItemLoot { item_id: "ak-47".into() },
        agent: Some(Did::new("did:key:player1")),
        parents: vec![],
        metadata: vec![("location".into(), "warehouse".into())],
        payload_ref: None,
    }).await?;

    // Query the frontier
    let frontier = client.get_frontier(session_id.clone()).await?;

    // Get Merkle root
    let root = client.get_merkle_root(session_id.clone()).await?;

    // Dehydrate to LoamSpine
    let commit_root = client.dehydrate(session_id).await?;

    Ok(())
}
```

---

## 6. Server Implementation

Servers implement the trait:

```rust
use rhizo_crypt_rpc::{RhizoCryptRpc, RpcServer};
use rhizo_crypt_core::RhizoCrypt;

impl RhizoCryptRpc for MyServer {
    async fn create_session(
        self,
        _: tarpc::context::Context,
        request: CreateSessionRequest,
    ) -> Result<SessionId, RpcError> {
        // Implementation
        self.primal.create_session(request).await.map_err(Into::into)
    }

    // ... other methods
}

// Start the server
let primal = Arc::new(RhizoCrypt::new(config));
let server = RpcServer::new(primal, "127.0.0.1:9400".parse()?);
server.serve().await?;
```

---

## 7. Transport

RhizoCrypt uses:
- **TCP** for transport
- **bincode** for serialization (compact, fast)
- **TLS optional** (via BearDog tunnels for production)

```rust
// Server listens with bincode serialization
let listener = tarpc::serde_transport::tcp::listen(&addr, Bincode::default).await?;

// Client connects
let transport = tarpc::serde_transport::tcp::connect(&addr, Bincode::default).await?;
let client = RhizoCryptRpcClient::new(client::Config::default(), transport).spawn();
```

---

## 8. REST API (Optional)

For compatibility with non-Rust clients, an optional REST/JSON wrapper can be provided via axum:

```yaml
openapi: 3.0.3
info:
  title: RhizoCrypt REST API
  version: 0.3.0

paths:
  /sessions:
    post:
      summary: Create a new session
      requestBody:
        content:
          application/json:
            schema:
              $ref: '#/components/schemas/CreateSessionRequest'
      responses:
        '201':
          description: Session created

  /sessions/{session_id}:
    get:
      summary: Get session info
    delete:
      summary: Discard session

  /sessions/{session_id}/events:
    post:
      summary: Append event
    get:
      summary: Query events

  /sessions/{session_id}/merkle:
    get:
      summary: Get Merkle root

  /health:
    get:
      summary: Health check
```

**Note**: The REST API is a convenience wrapper. The primary interface is tarpc.

---

## 9. Authentication

All RPC calls require BearDog authentication:

```rust
// Context carries authentication
let ctx = tarpc::context::current();

// For production, use BearDog-signed tokens
let token = beardog.sign_request(&request).await?;
client.with_auth(token).create_session(request).await?;
```

---

## 10. Rate Limiting

| Endpoint Category | Rate Limit |
|-------------------|------------|
| Session management | 100 req/min |
| Event append | 10,000 req/min |
| Query operations | 1,000 req/min |
| Slice operations | 100 req/min |
| Health/metrics | 1,000 req/min |

---

## 11. Sovereignty Considerations

The RPC API respects primal sovereignty:

| Aspect | Implementation |
|--------|----------------|
| **Data ownership** | Session creator owns all data |
| **Consent** | Agent DID required for events |
| **Audit** | All operations logged with DIDs |
| **Deletion** | Sessions can be discarded by owner |
| **Export** | Full DAG can be exported |

---

## 12. References

- [ARCHITECTURE.md](./ARCHITECTURE.md) — System architecture
- [DATA_MODEL.md](./DATA_MODEL.md) — Data structures
- [INTEGRATION_SPECIFICATION.md](./INTEGRATION_SPECIFICATION.md) — Primal integrations
- [tarpc documentation](https://docs.rs/tarpc) — RPC framework

---

*RhizoCrypt: Pure Rust RPC for the memory that knows when to forget.*
