# 🔐 rhizoCrypt Showcase - Progressive Capability Demonstrations

**Purpose**: Demonstrate rhizoCrypt's ephemeral DAG engine capabilities  
**Philosophy**: Progressive complexity — isolated → RPC → inter-primal → complete workflow  
**Goal**: Show how rhizoCrypt captures, links, and commits session data across the ecosystem

---

## 🎯 Showcase Philosophy

rhizoCrypt is the "memory that knows when to forget." This showcase demonstrates its evolution:

1. **Isolated Instance**: Single rhizoCrypt capabilities (sessions, DAG, Merkle)
2. **RPC Layer**: tarpc server/client with rate limiting and metrics
3. **Inter-Primal**: Integration with Songbird, BearDog, NestGate, LoamSpine
4. **Complete Workflow**: Full dehydration → commit → provenance cycle
5. **Live Integration**: Real Phase 1 binaries from `../bins/`

**Real-World Scenario**: *"Capture a complex session with multiple agents, prove its integrity, and commit results to permanent storage"*

---

## 📁 Structure

```
showcase/
├── 01-isolated/          # Single rhizoCrypt demos
│   ├── sessions/         # Session lifecycle
│   ├── dag/             # DAG operations
│   ├── merkle/          # Merkle trees & proofs
│   └── slices/          # Slice semantics (6 modes)
│
├── 02-rpc/              # RPC layer demos
│   ├── server/          # tarpc server startup
│   ├── client/          # RPC client operations
│   └── metrics/         # Prometheus metrics
│
├── 03-inter-primal/     # Primal integration
│   ├── songbird-discovery/  # Capability discovery
│   ├── beardog-signing/     # DID & signatures
│   ├── nestgate-payloads/   # Payload storage
│   └── loamspine-commits/   # Permanent commits
│
├── 04-complete-workflow/ # End-to-end demos
│   ├── dehydration/     # Session → Summary → Commit
│   └── provenance/      # Query & attribution
│
├── 05-live-integration/ # Real Phase 1 binaries
│   ├── start-primals.sh # Start Songbird, NestGate
│   ├── demo-live-discovery.sh
│   ├── demo-live-signing.sh
│   └── stop-primals.sh  # Cleanup
│
└── utils/               # Shared utilities
    ├── setup/           # Environment setup
    ├── cleanup/         # Cleanup scripts
    └── monitoring/      # Monitoring tools
```

---

## 🚀 Quick Start

### Prerequisites
```bash
# Build rhizoCrypt
cargo build --workspace

# Optional: Build with RocksDB
cargo build --workspace --features rocksdb

# Optional: Build with live clients
cargo build -p rhizo-crypt-core --features live-clients
```

### Run Demos

**Phase 1: Isolated Instance (No Dependencies)**
```bash
cd showcase/01-isolated
./run-all-demos.sh
```

**Phase 2: RPC Layer**
```bash
cd showcase/02-rpc
./start-server.sh
# In another terminal:
./demo-rpc-operations.sh
```

**Phase 3: Inter-Primal (Requires Songbird)**
```bash
cd showcase/03-inter-primal
./setup-with-songbird.sh
./demo-discovery.sh
```

**Phase 4: Complete Workflow**
```bash
cd showcase/04-complete-workflow
./demo-full-dehydration.sh
```

**Phase 5: Live Integration (Real Binaries)**
```bash
cd showcase/05-live-integration
./start-primals.sh        # Start Songbird + NestGate
./demo-live-discovery.sh  # Real Songbird connection
./demo-live-signing.sh    # Real BearDog CLI
./stop-primals.sh         # Cleanup
```

---

## 🎓 Learning Path

### For New Users
1. Start with `01-isolated/sessions/` to understand session lifecycle
2. Progress to `01-isolated/dag/` to see content-addressed events
3. Explore `01-isolated/merkle/` for proof generation
4. Try `02-rpc/` to see the RPC layer

### For Operators
1. Review `02-rpc/server/` for deployment
2. Study `02-rpc/metrics/` for observability
3. Use `utils/monitoring/` for health checks

### For Developers
1. Study demo source code in each phase
2. Review integration patterns in `03-inter-primal/`
3. Extend demos with custom event types

---

## 📊 What You'll Learn

### Phase 1: Isolated Instance
- ✅ Session lifecycle (Create → Active → Resolve)
- ✅ Content-addressed vertices (Blake3)
- ✅ Multi-parent DAG operations
- ✅ Merkle tree construction & proofs
- ✅ Slice semantics (Copy, Loan, Escrow, etc.)

### Phase 2: RPC Layer
- ✅ tarpc server with 24 methods
- ✅ Rate limiting (token bucket)
- ✅ Prometheus-compatible metrics
- ✅ Graceful shutdown
- ✅ Client connection patterns

### Phase 3: Inter-Primal
- ✅ Capability-based discovery via Songbird
- ✅ DID verification & signing via BearDog
- ✅ Payload storage via NestGate
- ✅ Permanent commits via LoamSpine

### Phase 4: Complete Workflow
- ✅ Multi-agent session capture
- ✅ Dehydration with attestations
- ✅ Commit to LoamSpine
- ✅ Provenance queries via SweetGrass

### Phase 5: Live Integration
- ✅ Real Songbird Rendezvous (port 8888)
- ✅ Real BearDog CLI (v0.9.0)
- ✅ Capability registration working
- ⚠️ NestGate JWT configuration pending

---

## 🌟 Featured Demo: Gaming Session with ML Training

**Scenario**: A gaming session where AI agents train on player data

**What Happens**:
1. rhizoCrypt creates a Gaming session
2. Player actions become vertices in the DAG
3. ML training events are captured with GPU metadata
4. Agent contributions are tracked via DIDs (BearDog)
5. Model checkpoints stored in NestGate
6. Session dehydrates to summary with Merkle proof
7. Summary committed to LoamSpine
8. Later: SweetGrass queries provenance

**Demo**: `04-complete-workflow/dehydration/demo-gaming-ml-session.sh`

---

## 📋 Demo Catalog

### Phase 1: Isolated (12 demos)
| Demo | Description | Time | Complexity |
|------|-------------|------|------------|
| `hello-rhizocrypt` | Basic startup and health | 2 min | Beginner |
| `session-lifecycle` | Create, grow, resolve | 5 min | Beginner |
| `dag-operations` | Add vertices, query DAG | 5 min | Beginner |
| `merkle-proofs` | Generate and verify proofs | 5 min | Intermediate |
| `multi-parent-dag` | Complex DAG structures | 10 min | Intermediate |
| `slice-copy` | Copy mode demo | 5 min | Beginner |
| `slice-loan` | Loan with auto-return | 5 min | Intermediate |
| `slice-escrow` | Multi-party escrow | 10 min | Advanced |
| `event-types` | 25+ event type showcase | 10 min | Intermediate |
| `performance` | Sub-microsecond operations | 5 min | Beginner |
| `storage-backends` | InMemory vs RocksDB | 10 min | Intermediate |
| `real-verification` | Prove DAG integrity | 5 min | Intermediate |

### Phase 2: RPC (6 demos)
| Demo | Description | Time | Complexity |
|------|-------------|------|------------|
| `server-startup` | Start tarpc server | 2 min | Beginner |
| `client-operations` | All 24 RPC methods | 10 min | Intermediate |
| `rate-limiting` | Token bucket in action | 5 min | Intermediate |
| `metrics-dashboard` | Prometheus metrics | 5 min | Beginner |
| `graceful-shutdown` | Clean server shutdown | 5 min | Intermediate |
| `high-throughput` | 1000+ ops/sec | 10 min | Advanced |

### Phase 3: Inter-Primal (8 demos)
| Demo | Description | Time | Complexity |
|------|-------------|------|------------|
| `discover-capabilities` | Find primals via Songbird | 5 min | Intermediate |
| `verify-did` | BearDog DID verification | 5 min | Intermediate |
| `sign-vertex` | Cryptographic signatures | 5 min | Intermediate |
| `store-payload` | NestGate content storage | 5 min | Intermediate |
| `commit-session` | LoamSpine dehydration | 10 min | Advanced |
| `toadstool-events` | Compute event integration | 10 min | Advanced |
| `sweetgrass-query` | Provenance queries | 10 min | Advanced |
| `full-ecosystem` | All primals coordinated | 20 min | Expert |

### Phase 4: Complete Workflow (4 demos)
| Demo | Description | Time | Complexity |
|------|-------------|------|------------|
| `simple-session` | Basic capture → commit | 10 min | Intermediate |
| `multi-agent` | Multiple DIDs, attestations | 15 min | Advanced |
| `gaming-ml-session` | Full gaming + ML scenario | 30 min | Advanced |
| `provenance-chain` | Query attribution history | 15 min | Advanced |

**Total**: 30 progressive demos

---

## ⚡ Performance Benchmarks

| Operation | Time | Throughput |
|-----------|------|------------|
| Vertex creation | ~720 ns | 1.4M/sec |
| Blake3 hash (4KB) | ~80 ns | 12.5M/sec |
| DAG put_vertex | ~1.6 µs | 625K/sec |
| DAG get_vertex | ~270 ns | 3.7M/sec |
| Merkle root (1k) | ~750 µs | 1.3K trees/sec |
| Proof verification | ~1.4 µs | 714K/sec |

---

## 🏗️ Architecture Flow

```
┌─────────────────────────────────────────────────────────────────┐
│                        rhizoCrypt                                │
│                     (Ephemeral DAG Engine)                       │
├─────────────────────────────────────────────────────────────────┤
│     tarpc RPC (24 methods) + Rate Limiting + Metrics            │
├─────────────────────────────────────────────────────────────────┤
│  ┌─────────┐  ┌─────────┐  ┌─────────┐  ┌─────────────┐        │
│  │ Vertex  │  │  DAG    │  │ Merkle  │  │  Sessions   │        │
│  │ Store   │  │ Index   │  │ Trees   │  │  (scopes)   │        │
│  └────┬────┘  └────┬────┘  └────┬────┘  └──────┬──────┘        │
│       │            │            │              │                │
│       └────────────┴────────────┴──────────────┘                │
│                           │                                      │
│  ┌────────────────────────┴────────────────────────┐            │
│  │                 Dehydration                      │            │
│  │    Session → Summary → Attestations → Commit    │            │
│  └──────────────────────────────────────────────────┘            │
└───────────────────────────┬─────────────────────────────────────┘
                            │
        ┌───────────────────┼───────────────────┐
        ▼                   ▼                   ▼
┌───────────────┐  ┌───────────────┐  ┌───────────────┐
│   Songbird    │  │   BearDog     │  │   NestGate    │
│   Discovery   │  │   Signing     │  │   Payloads    │
└───────────────┘  └───────────────┘  └───────────────┘
                            │
                            ▼
                  ┌───────────────┐
                  │   LoamSpine   │
                  │   Permanent   │
                  │   Storage     │
                  └───────────────┘
```

---

## 🎯 Success Criteria

### Phase 1 Complete When:
- [x] Sessions create, grow, and resolve
- [x] Vertices are content-addressed (Blake3)
- [x] Merkle proofs verify correctly
- [x] All 6 slice modes work

### Phase 2 Complete When:
- [x] RPC server starts and accepts connections
- [x] All 24 methods respond correctly
- [x] Rate limiting protects resources
- [x] Metrics are collected

### Phase 3 Complete When:
- [x] Songbird discovers rhizoCrypt capabilities
- [x] BearDog signs vertices
- [x] NestGate stores payloads
- [x] LoamSpine accepts commits

### Phase 4 Complete When:
- [x] Full session → commit workflow works
- [x] Multi-agent attestations succeed
- [x] Provenance queries return correct data

### Phase 5 Complete When:
- [x] Songbird Rendezvous connection works
- [x] BearDog CLI integration works
- [ ] NestGate live storage works
- [ ] Full multi-primal live workflow

---

## 💡 Tips

### For Best Results:
- Start with Phase 1 to understand core concepts
- Use multiple terminals to see different perspectives
- Watch logs in real-time for insights
- Try the benchmarks to see performance

### Common Issues:
- **Port Conflicts**: Use `utils/setup/check-ports.sh`
- **Missing Primals**: Phase 3+ requires live services
- **RocksDB Errors**: Ensure libclang is installed
- **Rate Limited**: Check token bucket settings

---

## 🏆 Showcase Goals

**Primary Goal**: Demonstrate rhizoCrypt's world-class DAG capabilities

**Secondary Goals**:
- Show ephemeral-by-default design
- Prove content-addressed integrity
- Enable ecosystem integration
- Inspire confidence in the system

**Ultimate Goal**: *"Any session can be captured, proven, and selectively committed in under 1 second"*

---

**Ready to explore?** Start with `01-isolated/sessions/README.md` and progress through the phases!

**Questions?** See individual phase READMEs or check `../specs/`

🔐 **Let's showcase the memory that knows when to forget!** 🔐

