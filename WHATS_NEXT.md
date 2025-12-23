# 🔐 RhizoCrypt — What's Next

**Last Updated**: December 22, 2025

---

## 🎯 Implementation Roadmap

### Phase 1: Core Data Structures (Weeks 1-2)

**Goal**: Implement the fundamental types from the specification.

#### Week 1: Vertex & Types
- [ ] Add `blake3` dependency for content addressing
- [ ] Implement `VertexId` type (Blake3 hash wrapper)
- [ ] Implement `Timestamp` with nanosecond precision
- [ ] Implement `PayloadRef` for content-addressed payloads
- [ ] Implement core `Vertex` structure:
  ```rust
  pub struct Vertex {
      pub id: VertexId,
      pub parents: Vec<VertexId>,
      pub timestamp: Timestamp,
      pub agent: Option<Did>,
      pub event_type: EventType,
      pub payload: Option<PayloadRef>,
      pub signature: Option<Signature>,
  }
  ```
- [ ] Add vertex serialization (serde)
- [ ] Add vertex hashing (compute ID from contents)
- [ ] Unit tests for vertex creation and hashing

#### Week 2: Session Management
- [ ] Implement `SessionId` type
- [ ] Implement `SessionConfig` structure
- [ ] Implement `SessionState` enum (Active, Resolving, Committed, Discarded)
- [ ] Implement `Session` structure:
  ```rust
  pub struct Session {
      pub id: SessionId,
      pub session_type: SessionType,
      pub vertices: HashMap<VertexId, Vertex>,
      pub frontier: HashSet<VertexId>,
      pub genesis: HashSet<VertexId>,
      pub state: SessionState,
      pub config: SessionConfig,
  }
  ```
- [ ] Add session lifecycle methods (create, resolve, discard)
- [ ] Unit tests for session management

---

### Phase 2: Event Types & Ingestion (Weeks 3-4)

**Goal**: Define domain events and high-performance append.

#### Week 3: Event Types
- [ ] Implement core `EventType` enum:
  - `SessionStart`, `SessionEnd`
  - `DataCreate`, `DataModify`, `DataDelete`
  - `AgentJoin`, `AgentLeave`
- [ ] Implement gaming event types (from spec):
  - `ItemLoot`, `ItemDrop`, `ItemTransfer`
  - `Combat`, `Extraction`
- [ ] Implement scientific event types:
  - `ExperimentStart`, `Observation`, `Analysis`, `Result`
- [ ] Implement `Custom` event type for extensions
- [ ] Unit tests for event serialization

#### Week 4: Event Ingestion
- [ ] Implement `EventIngester` trait
- [ ] Implement high-performance `append()` method
- [ ] Implement `append_batch()` for throughput
- [ ] Add parent auto-detection (use frontier if not specified)
- [ ] Add signature validation (optional, via BearDog)
- [ ] Benchmarks: target < 1ms per event, > 10k events/sec

---

### Phase 3: DAG Storage (Weeks 5-6)

**Goal**: Implement the storage backend.

#### Week 5: In-Memory Store
- [ ] Implement `DagStore` trait:
  ```rust
  pub trait DagStore: Send + Sync {
      async fn put_vertex(&self, session: SessionId, vertex: Vertex) -> Result<()>;
      async fn get_vertex(&self, session: SessionId, id: VertexId) -> Result<Option<Vertex>>;
      async fn get_children(&self, session: SessionId, parent: VertexId) -> Result<Vec<VertexId>>;
      async fn get_frontier(&self, session: SessionId) -> Result<Vec<VertexId>>;
      async fn delete_session(&self, session: SessionId) -> Result<()>;
  }
  ```
- [ ] Implement `InMemoryDagStore`
- [ ] Add indexing for efficient parent lookups
- [ ] Add frontier tracking
- [ ] Unit tests for store operations

#### Week 6: Payload Store
- [ ] Implement `PayloadStore` trait (content-addressed)
- [ ] Integrate with NestGate for blob storage
- [ ] Add payload reference counting for GC
- [ ] Integration tests with NestGate

---

### Phase 4: Merkle Trees (Weeks 7-8)

**Goal**: Implement cryptographic proofs.

#### Week 7: Tree Construction
- [ ] Implement topological sort for vertices
- [ ] Implement Merkle tree builder
- [ ] Implement `MerkleRoot` computation
- [ ] Add caching for incremental updates
- [ ] Benchmarks: target < 100ms for 100k vertices

#### Week 8: Proof Generation
- [ ] Implement `MerkleProof` structure
- [ ] Implement proof generation for single vertex
- [ ] Implement proof verification
- [ ] Add batch proof generation
- [ ] Unit tests for proof correctness

---

### Phase 5: Dehydration (Weeks 9-10)

**Goal**: Commit to LoamSpine.

#### Week 9: Dehydration Engine
- [ ] Implement `DehydrationSummary` structure
- [ ] Implement summary generation from session
- [ ] Extract key results from session
- [ ] Generate agent participation summary
- [ ] Collect attestations from participants

#### Week 10: LoamSpine Integration
- [ ] Implement `LoamSpineClient` trait
- [ ] Implement commit flow
- [ ] Add session state transition to `Committed`
- [ ] Implement garbage collection for expired sessions
- [ ] Integration tests with LoamSpine

---

### Phase 6: Integration & Hardening (Weeks 11-12)

**Goal**: Production readiness.

#### Week 11: BearDog Integration
- [ ] Implement signature request flow
- [ ] Implement signature verification
- [ ] Add DID resolution
- [ ] Integration tests with BearDog

#### Week 12: Performance & Testing
- [ ] End-to-end integration tests
- [ ] Performance benchmarking
- [ ] Chaos testing (failures, timeouts)
- [ ] Documentation completion
- [ ] Showcase demos

---

## 📊 Success Metrics

| Metric | Target |
|--------|--------|
| Event append latency | < 1ms (p99) |
| Batch throughput | > 10,000 events/sec |
| Merkle root computation | < 100ms (100k vertices) |
| Proof generation | < 1ms |
| Test coverage | > 80% |

---

## 🔗 Dependencies

### External
- `blake3` — Content addressing
- `serde` / `serde_json` — Serialization
- `tokio` — Async runtime

### Gen 1 Primals
- **BearDog** — Signing (Week 11)
- **NestGate** — Payload storage (Week 6)
- **Songbird** — Service discovery (Week 11)

### Phase 2 Siblings
- **LoamSpine** — Commit target (Week 10)

---

## 📚 Reference Documents

- [specs/RHIZOCRYPT_SPECIFICATION.md](./specs/RHIZOCRYPT_SPECIFICATION.md) — Full specification
- [../ARCHITECTURE.md](../ARCHITECTURE.md) — Unified architecture
- [../INTEGRATION_OVERVIEW.md](../INTEGRATION_OVERVIEW.md) — Data flows

---

*RhizoCrypt: Building the memory that knows when to forget.*

