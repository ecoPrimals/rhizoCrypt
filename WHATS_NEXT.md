# 🔐 rhizoCrypt — What's Next

**Last Updated**: December 23, 2025

---

## ✅ Completed

### Phase 1: Core Data Structures
- Core types: `VertexId`, `SessionId`, `SliceId`, `PayloadRef`, `Did`, `Timestamp`
- Vertex structure with builder pattern
- Session management with lifecycle states
- 25+ event types across 7 domains

### Phase 2: DAG & Merkle
- `DagStore` and `PayloadStore` traits
- `InMemoryDagStore` and `InMemoryPayloadStore`
- Merkle tree builder with topological sort
- Proof generation and verification

### Phase 3A: Advanced Features
- Slice semantics (6 modes)
- Resolution routing and constraints
- Dehydration protocol with attestations
- Integration client traits

### Phase 3B: Live Client Wiring ✅
- `live-clients` feature flag
- Songbird client (tarpc)
- BearDog client (HTTP)
- NestGate client (HTTP)
- LoamSpine client (tarpc)

### Phase 3C: Full Ecosystem Coverage ✅ (NEW)
- ToadStool client (`toadstool.rs`, 556 lines)
  - `TaskId`, `ComputeEvent` types
  - Event subscription for compute tasks
  - Integration with ML pipelines
- SweetGrass client (`sweetgrass.rs`)
  - `ProvenanceChain`, `SessionAttribution` types
  - `SweetGrassQueryable` trait
  - Provenance and attribution queries
- New discovery capabilities:
  - `ComputeOrchestration`, `ComputeEvents` (ToadStool)
  - `ProvenanceQuery`, `Attribution` (SweetGrass)

### Phase 4: Hardening
- tarpc RPC (24 methods)
- Rate limiting (token bucket)
- Prometheus metrics
- Graceful shutdown
- RocksDB backend
- 263 tests, 86%+ coverage

### Phase 4B: Showcase ✅ (COMPLETE)
Progressive demo structure (following Songbird pattern):

**Phase 01: Isolated (4 demos)** — Core capabilities, no dependencies
- `demo-session-lifecycle.sh` — Create, grow, query, resolve sessions
- `demo-dag-operations.sh` — Multi-parent DAG, content-addressing
- `demo-merkle-proofs.sh` — Tree construction, O(log n) proofs
- `demo-slice-semantics.sh` — Copy/Loan/Consignment modes

**Phase 02: RPC (1 demo)** — tarpc access
- `start-server.sh` — RPC server startup

**Phase 03: Inter-Primal (4 demos)** — Ecosystem integration
- `demo-discovery.sh` — Runtime capability-based discovery
- `demo-signing.sh` — BearDog DID verification, signatures
- `demo-payload-storage.sh` — NestGate content-addressed payloads
- `demo-loamspine-commit.sh` — Permanent storage, checkout

**Phase 04: Complete Workflows (1 demo)** — End-to-end
- `demo-simple-dehydration.sh` — Session → Merkle → Commit

**Phase 05: Live Integration (2 demos)** — Real Phase 1 binaries ✅ NEW
- `demo-live-discovery.sh` — Real Songbird Rendezvous connection
- `demo-live-signing.sh` — Real BearDog CLI v0.9.0

**Total: 12 demos (11 verified)** | `QUICK_START.sh` for interactive launcher

---

## 🎯 Current: Live Integration Complete ✅

### Phase 1 Binaries (from `../bins/`)
| Binary | Type | Port | Status |
|--------|------|------|--------|
| `songbird-rendezvous` | HTTP | 8888 | ✅ Tested |
| `songbird-orchestrator` | tarpc | 8080 | ✅ Tested |
| `beardog` | CLI | — | ✅ v0.9.0 |
| `nestgate` | HTTP | 8092 | ⚠️ Needs JWT |

### Quick Start
```bash
cd showcase/05-live-integration
./start-primals.sh        # Start Songbird + NestGate
./demo-live-discovery.sh  # Connect to real Songbird
./demo-live-signing.sh    # Use real BearDog CLI
./stop-primals.sh         # Cleanup
```

### Integration Checklist
- [x] Start Songbird orchestrator + rendezvous
- [x] Test HTTP discovery via Rendezvous
- [x] BearDog CLI integration (v0.9.0)
- [x] Capability registration working
- [ ] NestGate JWT configuration
- [ ] LoamSpine integration

---

## 📋 Phase 5: Extended Hardening

### Testing
- [ ] Network failure chaos tests
- [ ] Load testing (sustained pressure)
- [ ] Memory profiling

### Storage
- [ ] LMDB backend (optional)
- [ ] Migration tooling
- [ ] Backup/restore

### Observability
- [ ] Structured logging improvements
- [ ] Distributed tracing

---

## 📋 Phase 6: Production

- [ ] Configuration validation
- [ ] Extended health checks
- [ ] Graceful degradation
- [ ] Deployment docs
- [ ] Operational runbooks

---

## 📊 Success Metrics

| Metric | Target | Current |
|--------|--------|---------|
| Event append | < 1ms | ✅ ~1.6µs |
| Vertex lookup | < 1ms | ✅ ~270ns |
| Merkle root (1k) | < 100ms | ✅ ~750µs |
| Proof verify | < 1ms | ✅ ~1.4µs |
| Coverage | > 80% | ✅ 86.16% |
| RPC methods | 24 | ✅ 24 |
| Tests | Full suite | ✅ 263 |
| Client modules | 6 | ✅ 6 |

---

## 🔗 Client Status

| Client | Protocol | Status |
|--------|----------|--------|
| **Songbird** | tarpc | ✅ Wired |
| **BearDog** | HTTP | ✅ Wired |
| **NestGate** | HTTP | ✅ Wired |
| **LoamSpine** | tarpc | ✅ Wired |
| **ToadStool** | HTTP | ✅ Wired (`toadstool_http.rs`) |
| **SweetGrass** | Provider | ✅ Verified (rhizoCrypt exposes API) |

---

## 🏁 Definition of Done

rhizoCrypt is **production-ready** when:

1. ✅ All 24 RPC methods implemented
2. ✅ E2E, chaos, RPC tests passing
3. ✅ 80%+ test coverage
4. ✅ Rate limiting + metrics
5. ✅ Multiple storage backends
6. ✅ Live primal connections tested (Songbird, BearDog)
7. ✅ Performance targets met
8. ✅ Extended chaos testing (18 tests)
9. ✅ Observability operational
10. ✅ Documentation complete
11. ✅ Pure Rust (no protobuf)
12. ✅ Showcase demos (12 total, Songbird pattern)

---

*rhizoCrypt: Building the memory that knows when to forget.*
