# 🔐 rhizoCrypt — What's Next

**Last Updated**: December 24, 2025  
**Version**: 0.9.2

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

### Phase 3: Advanced Features
- Slice semantics (6 modes)
- Resolution routing and constraints
- Dehydration protocol with attestations
- Integration client traits

### Phase 4: Hardening
- tarpc RPC (24 methods)
- Rate limiting (token bucket)
- Prometheus metrics
- Graceful shutdown
- RocksDB backend
- 260 tests, 85% coverage

### Phase 5: Showcase
- 12 progressive demos (Songbird pattern)
- Live Songbird Rendezvous connection
- Live BearDog CLI integration (v0.9.0)

### Phase 6: Primal-Agnostic Architecture ✅ NEW
- `SafeEnv` module for type-safe environment config
- `CapabilityEnv` for capability endpoint resolution
- `service_id` replaces `primal_name` (agnostic)
- `IntegrationStatus` uses capability-based fields
- Removed primal names from `Capability` enum comments
- Debug logs use capability descriptions

---

## 📊 Current Metrics

| Metric | Target | Current |
|--------|--------|---------|
| Tests | 200+ | ✅ 260 |
| Coverage | 40%+ | ✅ 85% |
| Clippy | Clean | ✅ Clean |
| Unsafe | 0 | ✅ 0 |
| Max file | <1000 | ✅ 923 |
| Primal names in code | 0 | ✅ 0 (production) |

---

## 📋 Future Work

### Extended Hardening (Optional)
- [ ] Network partition chaos tests
- [ ] Load testing (sustained pressure)
- [ ] Memory profiling
- [ ] LMDB backend

### Production Deployment
- [ ] Kubernetes deployment manifests
- [ ] Configuration validation
- [ ] Extended health checks
- [ ] Operational runbooks

### Ecosystem Integration
- [ ] NestGate JWT configuration
- [ ] LoamSpine live integration
- [ ] SweetGrass live queries

---

## 🏁 Definition of Done

rhizoCrypt is **production-ready** ✅

| Criteria | Status |
|----------|--------|
| All 24 RPC methods implemented | ✅ |
| E2E, chaos, RPC tests passing | ✅ |
| 80%+ test coverage | ✅ 85% |
| Rate limiting + metrics | ✅ |
| Multiple storage backends | ✅ |
| Live primal connections tested | ✅ |
| Performance targets met | ✅ |
| Chaos testing (18 tests) | ✅ |
| Observability operational | ✅ |
| Documentation complete | ✅ |
| Pure Rust (no protobuf) | ✅ |
| Showcase demos (12 total) | ✅ |
| Primal-agnostic architecture | ✅ |

---

## 📚 Documentation

| Document | Description |
|----------|-------------|
| [README.md](./README.md) | Project overview |
| [START_HERE.md](./START_HERE.md) | Developer guide |
| [STATUS.md](./STATUS.md) | Implementation status |
| [showcase/](./showcase/) | Interactive demos |
| [specs/](./specs/) | Full specifications |

---

*rhizoCrypt: The memory that knows when to forget.*
