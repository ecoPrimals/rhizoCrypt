# 🔐 rhizoCrypt — Start Here

Welcome to rhizoCrypt! This guide gets you up to speed quickly.

---

## What is rhizoCrypt?

**Core DAG Engine** for Phase 2 of ecoPrimals. Think "git for events" — a content-addressed Directed Acyclic Graph that captures everything during a session, then selectively forgets most of it.

> **Key insight**: rhizoCrypt is designed to be forgotten. Only what's committed to LoamSpine survives.

---

## 🎭 Try the Showcase First!

The fastest way to understand rhizoCrypt is through the interactive demos:

```bash
cd showcase && ./QUICK_START.sh
```

**12 demos** covering:
- Session lifecycle (create, grow, query, resolve)
- DAG operations (multi-parent, content-addressing)
- Merkle proofs (O(log n) verification)
- Slice semantics (Copy, Loan, Consignment)
- Capability discovery (Songbird)
- DID signing (BearDog)
- Payload storage (NestGate)
- Permanent commits (LoamSpine)
- Complete dehydration workflow
- **Live Songbird connection** (real binary)
- **Live BearDog CLI** (v0.9.0)

---

## Quick Start (Development)

```bash
# Build
cargo build --workspace

# Test (254 tests)
cargo test --workspace

# Coverage (86%+)
cargo llvm-cov --workspace

# Benchmarks
cargo bench -p rhizo-crypt-core

# Docs
cargo doc --workspace --no-deps --open
```

### Feature Flags

```bash
# With persistent storage
cargo build --features rocksdb

# With live client connections
cargo build -p rhizo-crypt-core --features live-clients
```

---

## Architecture

```
rhizoCrypt
│
├── tarpc RPC (24 methods)
│   ├── Sessions: create, get, list, discard
│   ├── Events: append, append_batch
│   ├── Queries: get_vertex, get_frontier, query
│   ├── Merkle: get_root, get_proof, verify
│   ├── Slices: checkout, get, list, resolve
│   └── Dehydration: dehydrate, get_status
│
├── Production Hardening
│   ├── RateLimiter (token bucket)
│   ├── MetricsCollector (Prometheus)
│   └── Graceful Shutdown
│
├── Live Clients (capability discovery)
│   ├── SongbirdClient (tarpc) — service mesh
│   ├── BearDogClient (HTTP) — signing
│   ├── NestGateClient (HTTP) — payloads
│   ├── LoamSpineClient (tarpc) — commits
│   ├── ToadStoolClient (tarpc) — compute events
│   └── SweetGrassQueryable — provenance
│
├── Storage Backends
│   ├── InMemoryDagStore (default)
│   └── RocksDbDagStore (--features rocksdb)
│
└── Core Engine
    ├── Vertices (content-addressed events)
    ├── Sessions (scoped DAGs)
    ├── Merkle Trees (proofs)
    ├── Slices (6 modes)
    └── Dehydration (→ LoamSpine)
```

---

## Key Concepts

### Vertex
Content-addressed DAG node:
- Blake3 hash as ID
- Parent links (multi-parent DAG)
- Event type (25+ types)
- Optional DID signature

### Session
Scoped DAG with lifecycle:
```
Created → Active → Resolving → Resolved
                            ↘ Rolled Back
```

### Slice
LoamSpine state "checkout":
- **Copy** — Local use only
- **Loan** — Auto-returns
- **Consignment** — Temporary possession
- **Escrow** — Multi-party agreement
- **Waypoint** — Anchors to spine
- **Transfer** — Ownership transfer

### Storage
```rust
// In-memory (default)
let store = InMemoryDagStore::new();

// RocksDB (persistent)
#[cfg(feature = "rocksdb")]
let store = RocksDbDagStore::open("/path/to/db")?;
```

### Live Clients
```rust
use rhizo_crypt_core::clients::SongbirdClient;

let songbird = SongbirdClient::from_env();
songbird.connect().await?;
songbird.register("127.0.0.1:9400").await?;

let beardog = songbird.discover_beardog().await?;
```

### Pure Rust RPC
```rust
#[tarpc::service]
pub trait RhizoCryptRpc {
    async fn create_session(request: CreateSessionRequest) -> Result<SessionId, RpcError>;
    async fn append_event(request: AppendEventRequest) -> Result<VertexId, RpcError>;
    async fn health() -> Result<HealthStatus, RpcError>;
}
```

---

## Project Structure

```
rhizoCrypt/
├── Cargo.toml           # Workspace
├── crates/
│   ├── rhizo-crypt-core/    # Core library (~13.7k LOC)
│   │   ├── src/
│   │   │   ├── lib.rs       # Entry + RhizoCrypt primal
│   │   │   ├── clients/     # Live primal clients
│   │   │   ├── store.rs     # In-memory storage
│   │   │   ├── store_rocksdb.rs
│   │   │   ├── vertex.rs
│   │   │   ├── session.rs
│   │   │   ├── merkle.rs
│   │   │   ├── slice.rs
│   │   │   └── dehydration.rs
│   │   ├── benches/
│   │   └── tests/
│   │
│   └── rhizo-crypt-rpc/     # RPC (~3.3k LOC)
│       ├── src/
│       │   ├── service.rs   # tarpc trait
│       │   ├── server.rs
│       │   ├── client.rs
│       │   ├── rate_limit.rs
│       │   └── metrics.rs
│       └── tests/
│
├── showcase/            # Interactive demos (10 total)
│   ├── QUICK_START.sh
│   ├── 01-isolated/     # Sessions, DAG, Merkle, Slices
│   ├── 02-rpc/          # Server demos
│   ├── 03-inter-primal/ # Discovery, Signing, Payloads, Commits
│   └── 04-complete-workflow/
│
├── specs/               # Specifications
├── README.md
├── STATUS.md
└── WHATS_NEXT.md
```

---

## Testing

| Type | Count | Command |
|------|-------|---------|
| Unit | 181 | `cargo test --lib` |
| Discovery | 21 | `cargo test discovery::` |
| Integration | 21 | `cargo test integration::` |
| E2E | 8 | `cargo test -p rhizo-crypt-core --test e2e_tests` |
| Chaos | 18 | `cargo test -p rhizo-crypt-core --test chaos_tests` |
| Property | 17 | `cargo test -p rhizo-crypt-core --test property_tests` |
| RPC | 10 | `cargo test -p rhizo-crypt-rpc` |

---

## Integration

### Depends On (Gen 1)
| Primal | Purpose | Client |
|--------|---------|--------|
| **BearDog** | DIDs, signatures | ✅ Wired |
| **Songbird** | Service discovery | ✅ Wired |
| **NestGate** | Payload storage | ✅ Wired |
| **ToadStool** | Compute events | ✅ Scaffolded |

### Phase 2 Siblings
| Primal | Relationship | Client |
|--------|--------------|--------|
| **LoamSpine** | Receives commits | ✅ Wired |
| **SweetGrass** | Provenance queries | ✅ Scaffolded |

---

## Current Status

| Aspect | Status |
|--------|--------|
| Core Types | ✅ Complete |
| Sessions | ✅ Complete |
| Storage | ✅ InMemory + RocksDB |
| Merkle Trees | ✅ Complete |
| Slices | ✅ Complete |
| Dehydration | ✅ Complete |
| tarpc RPC | ✅ 24 methods |
| Rate Limiting | ✅ Token bucket |
| Metrics | ✅ Prometheus |
| Discovery | ✅ Capability-based |
| Live Clients | ✅ All 6 (4 wired + 2 scaffolded) |
| Tests | ✅ 254 passing |
| Coverage | ✅ 86%+ |

---

## Further Reading

| Document | Description |
|----------|-------------|
| [showcase/](./showcase/) | **Interactive demos** — start here! |
| [STATUS.md](./STATUS.md) | Implementation status |
| [WHATS_NEXT.md](./WHATS_NEXT.md) | Roadmap |
| [specs/](./specs/) | Full specifications |

---

*rhizoCrypt: The memory that knows when to forget.*
