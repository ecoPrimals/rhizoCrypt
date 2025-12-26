# 🔐 rhizoCrypt

**Ephemeral DAG Engine** — Phase 2 Working Memory  
**🦀 100% Pure Rust** — Zero C/C++ dependencies, zero unsafe code  
**🌱 Infant Discovery** — Zero hardcoding, capability-based architecture

---

## 🎯 Status: Production Ready

| Metric | Value |
|--------|-------|
| **Version** | 0.11.0 |
| **Pure Rust** | 🦀 **100%** (zero C/C++ deps) |
| **Tests** | ✅ 271 passing |
| **Coverage** | ✅ 64% core |
| **Clippy** | ✅ Zero warnings |
| **Unsafe** | ✅ 0 blocks (forbidden) |
| **Storage** | 🦀 Sled (Pure Rust) |
| **Architecture** | 🌱 **Capability-Based** |
| **Status** | 🚀 **Production Ready** |

---

## 🌟 What's New in v0.11.0

### **Zero-Hardcoding Architecture** 🎉

rhizoCrypt now uses **capability-based discovery** instead of hardcoded service names:

- ✅ **Vendor Neutral** — Works with ANY provider (not just BearDog, NestGate, etc.)
- ✅ **Runtime Discovery** — Services found dynamically, not hardcoded
- ✅ **Minimal Config** — One environment variable (or zero!)
- ✅ **Infant Discovery** — Starts with zero knowledge, learns at runtime

**Before (v0.10.x)**:
```rust
// ❌ Hardcoded to specific primal
use rhizo_crypt_core::clients::BearDogClient;
let client = BearDogClient::connect("http://beardog:9500").await?;
```

**After (v0.11.x)**:
```rust
// ✅ Works with ANY signing provider
use rhizo_crypt_core::clients::capabilities::SigningClient;
let signer = SigningClient::discover(&registry).await?;
// BearDog, YubiKey, CloudKMS, HSM, etc.
```

**Configuration**:
```bash
# Before: 6+ hardcoded variables
BEARDOG_ADDRESS=beardog.prod:9500
NESTGATE_ADDRESS=nestgate.prod:8080
# ... more ...

# After: 1 variable (or zero!)
export RHIZOCRYPT_DISCOVERY_ADAPTER=songbird.prod:7500
# Everything else discovered at runtime!
```

See [INFANT_DISCOVERY.md](INFANT_DISCOVERY.md) for details.

---

## 🚀 Quick Start

### As a Library

```bash
# Clone and build
git clone <repo>
cd rhizoCrypt
cargo build --workspace

# Run tests (271 passing)
cargo test --workspace

# Try showcase demos
cd showcase/00-local-primal/04-sessions
./demo-session-lifecycle.sh
```

### As a Standalone Service

```bash
# Build standalone service
cargo build --release --bin rhizocrypt-service

# Run service (default port 9400)
./target/release/rhizocrypt-service

# Or with discovery registration
RHIZOCRYPT_DISCOVERY_ADAPTER=songbird.local:7500 \
./target/release/rhizocrypt-service
```

See [crates/rhizocrypt-service/README.md](crates/rhizocrypt-service/README.md) for details.

---

## 📚 What is rhizoCrypt?

rhizoCrypt is the **ephemeral working memory** of the ecoPrimals ecosystem:

- **DAG Engine**: Content-addressed directed acyclic graph
- **Session Management**: Scoped workflows with lifecycle
- **Merkle Proofs**: Cryptographic integrity for every vertex
- **Dehydration**: Commit ephemeral results to permanent storage
- **Slice Semantics**: Checkout immutable snapshots for computation
- **Capability Discovery**: Zero hardcoding, runtime service discovery

### Philosophy

> **"Ephemeral by default, persistent by consent."**

rhizoCrypt forgets by design. Only explicit dehydration creates permanence.

> **"Like an infant, we start with zero knowledge and discover."**

rhizoCrypt starts with no hardcoded services. Everything is discovered at runtime based on capabilities, not vendor names.

---

## 🏗️ Architecture

### **High-Level Overview**

```
┌─────────────────────────────────────────────────────────────┐
│                        rhizoCrypt                            │
│                     (Ephemeral DAG Engine)                   │
│                                                              │
│  ┌─────────┐  ┌─────────┐  ┌─────────┐  ┌─────────────┐   │
│  │ Vertex  │  │  DAG    │  │ Merkle  │  │  Sessions   │   │
│  │ Store   │  │ Index   │  │ Trees   │  │  (scopes)   │   │
│  └─────────┘  └─────────┘  └─────────┘  └─────────────┘   │
│                                                              │
│  ┌────────────────────────────────────────────────────┐     │
│  │         Capability-Based Service Discovery         │     │
│  │  ┌─────────┐  ┌─────────┐  ┌──────────────────┐  │     │
│  │  │ Signing │  │ Storage │  │ Permanent Commit │  │     │
│  │  └─────────┘  └─────────┘  └──────────────────┘  │     │
│  └────────────────────────────────────────────────────┘     │
└─────────────────────────────────────────────────────────────┘
        │                                              │
        │ Slice Checkout                               │ Dehydration
        ▼                                              ▼
┌──────────────┐                              ┌──────────────┐
│ ANY Provider │                              │ ANY Provider │
│ (LoamSpine,  │                              │ (LoamSpine,  │
│  PostgreSQL, │                              │  S3, etc.)   │
│  etc.)       │                              │              │
└──────────────┘                              └──────────────┘
```

### **Capability-Based Architecture**

rhizoCrypt discovers services by **what they can do**, not **who they are**:

```
Discovery Registry
    ├─ Signing Capability        → (ANY signing provider)
    ├─ Storage Capability        → (ANY storage provider)
    ├─ Permanent Commit          → (ANY permanent storage)
    ├─ Compute Orchestration     → (ANY compute provider)
    └─ Provenance Tracking       → (ANY provenance provider)
```

**Zero Vendor Lock-In**: Swap providers with configuration changes only.

---

## 🎓 Core Concepts

### Sessions
Scoped DAGs with defined lifecycle:
- **Create** → Initialize session
- **Grow** → Add vertices
- **Resolve** → Finalize DAG
- **Expire** → Garbage collect

### Vertices
Content-addressed events in the DAG:
- Blake3 hash as identifier
- Parent references (DAG structure)
- Event type and payload
- Optional agent signature

### Merkle Trees
Cryptographic integrity:
- Merkle root for entire session
- Proofs for individual vertices
- Tamper detection
- Efficient verification

### Dehydration
Commit to permanent storage:
- Extract frontier (results)
- Compute Merkle root
- Send to LoamSpine
- Forget working memory

### Slices
Checkout from permanent storage:
- Immutable snapshot
- Copy/Loan/Consignment modes
- Compute over permanent data
- Dehydrate results back

---

## 📦 Crates

### `rhizo-crypt-core`
Core DAG engine library:
- Session management
- Vertex operations
- Merkle tree computation
- Storage backends (Sled)
- Client integrations

### `rhizo-crypt-rpc`
tarpc-based RPC layer:
- Type-safe remote calls
- Metrics and monitoring
- Rate limiting
- Service health

---

## 🎪 Showcase

Progressive learning path with **21 working demos**:

### Level 1: Hello rhizoCrypt (3 demos)
```bash
cd showcase/00-local-primal/01-hello-rhizocrypt
./demo-first-session.sh
./demo-first-vertex.sh
./demo-query-dag.sh
```

### Level 2: DAG Engine (4 demos)
```bash
cd showcase/00-local-primal/02-dag-engine
./demo-genesis.sh
./demo-frontier.sh
./demo-multi-parent.sh
./demo-topological-sort.sh
```

### Level 3: Merkle Proofs (4 demos)
```bash
cd showcase/00-local-primal/03-merkle-proofs
./demo-content-addressing.sh
./demo-merkle-tree.sh
./demo-merkle-proof.sh
./demo-tamper-detection.sh
```

### Level 4: Sessions ✨ (4 demos)
```bash
cd showcase/00-local-primal/04-sessions
./demo-session-lifecycle.sh       # Create → Grow → Resolve → Expire
./demo-ephemeral-persistent.sh    # Session management
./demo-slices.sh                   # Rhizo-Loam pattern
./demo-dehydration.sh              # Commit to permanent storage
```

### Inter-Primal Integration 🔗 (8 demos)
```bash
cd showcase/01-inter-primal-live

# Phase 1: Songbird (Discovery)
cd 01-songbird-discovery
./start-songbird.sh
./demo-register.sh
./demo-discover.sh
./demo-health.sh

# Phase 2: BearDog (Signing)
cd ../02-beardog-signing
./demo-hsm-discover.sh
./demo-generate-keys.sh
./demo-sign-vertex.sh
./demo-multi-agent.sh
```

**Progress**: 21/48 demos complete (44%)

---

## 🔗 Integration

rhizoCrypt integrates with Phase 1 primals:

### Songbird (Discovery) ✅
- Capability-based discovery
- Service registration
- Heartbeat mechanism
- **Status**: 4/4 demos working

### BearDog (Signing) ✅
- DID verification
- Ed25519 signatures
- Multi-agent sessions
- **Status**: 4/4 demos working

### NestGate (Storage) 📋
- Content-addressed payloads
- ZFS snapshots
- Compression coordination
- **Status**: Planned

### ToadStool (Compute) 📋
- GPU event tracking
- ML session capture
- Biome lifecycle
- **Status**: Planned

### Squirrel (AI) 📋
- MCP session routing
- Provider metadata
- Privacy preservation
- **Status**: Planned

---

## 🛠️ Development

### Build
```bash
cargo build --workspace
cargo build --release
```

### Test
```bash
# All tests (228)
cargo test --workspace

# With coverage
cargo llvm-cov --workspace --summary-only

# Specific crate
cargo test -p rhizo-crypt-core
```

### Lint
```bash
# Check code quality
cargo clippy --workspace -- -D warnings

# Format
cargo fmt --all
```

### Benchmark
```bash
cargo bench -p rhizo-crypt-core
```

---

## 📖 Documentation

### Getting Started
- [START_HERE.md](START_HERE.md) - New user onboarding
- [STATUS.md](STATUS.md) - Current project status
- [CHANGELOG.md](CHANGELOG.md) - Version history

### Specifications
- [specs/RHIZOCRYPT_SPECIFICATION.md](specs/RHIZOCRYPT_SPECIFICATION.md) - Core spec
- [specs/ARCHITECTURE.md](specs/ARCHITECTURE.md) - System design
- [specs/DEHYDRATION_PROTOCOL.md](specs/DEHYDRATION_PROTOCOL.md) - Commit protocol
- [specs/SLICE_SEMANTICS.md](specs/SLICE_SEMANTICS.md) - Checkout semantics

### Showcase
- [showcase/README.md](showcase/README.md) - Demo overview
- [showcase/00_SHOWCASE_INDEX.md](showcase/00_SHOWCASE_INDEX.md) - Full index

### Archives
- [../archive/rhizoCrypt/](../archive/rhizoCrypt/) - Historical session docs

---

## 🏆 Recent Achievements

### December 25, 2025
- ✅ **Pure Rust Evolution**: Removed RocksDB, achieved 100% Pure Rust
- ✅ **Showcase Level 4**: Created 4 session demos (all working)
- ✅ **Smart Refactoring**: songbird.rs 1159 → 864 lines (-25%)
- ✅ **BearDog Integration**: 4 signing demos (multi-agent working!)

### December 24, 2025
- ✅ **Songbird Integration**: 4 discovery demos (all working)
- ✅ **Comprehensive Audit**: Identified and fixed gaps
- ✅ **Test Coverage**: Achieved 64% core coverage

---

## 🎯 Roadmap

### Immediate (This Week)
- [ ] NestGate integration (payload storage)
- [ ] ToadStool integration (compute tracking)
- [ ] Complete showcase Level 5 (performance)

### Short Term (Next 2 Weeks)
- [ ] Squirrel integration (AI routing)
- [ ] Complete workflow demos
- [ ] Increase test coverage to 70%+

### Medium Term (Next Month)
- [ ] LoamSpine integration (permanent storage)
- [ ] Production hardening
- [ ] Performance optimization
- [ ] Security audit

---

## 🤝 Contributing

rhizoCrypt follows the ecoPrimals philosophy:

- **Primal Sovereignty**: Each primal has self-knowledge only
- **Pure Infant Discovery**: No hardcoded addresses, runtime discovery
- **Capability-Based**: Services discovered by what they can do
- **Ephemeral by Default**: Privacy through forgetting
- **Cryptographic Provenance**: Merkle proofs + signatures

---

## 📄 License

[License details here]

---

## 🔗 Links

- **ecoPrimals Ecosystem**: [Phase 1 Primals](../../phase1/)
- **Specifications**: [specs/](specs/)
- **Showcase**: [showcase/](showcase/)
- **Archives**: [../archive/rhizoCrypt/](../archive/rhizoCrypt/)

---

**rhizoCrypt** — *Ephemeral working memory for the sovereign data ecosystem* 🔐🌱
