# 🔐 rhizoCrypt

**Core DAG Engine** — Ephemeral Working Memory for Phase 2

---

## Status

| Metric | Value |
|--------|-------|
| Tests | ✅ **263 passing** |
| Coverage | ✅ **86.16%** |
| Clippy | ✅ Clean |
| Unsafe | ✅ 0 blocks |
| Clients | ✅ All 6 wired |
| Showcase | ✅ **12 demos** (11 verified) |
| LOC | ~17,600 |

---

## 🎭 Showcase

Experience rhizoCrypt capabilities through interactive demos:

```bash
cd showcase && ./QUICK_START.sh
```

| Phase | Demos | Highlights |
|-------|-------|------------|
| **01-isolated** | 4 | Sessions, DAG, Merkle proofs, Slices |
| **02-rpc** | 1 | tarpc server |
| **03-inter-primal** | 4 | Discovery, Signing, Payloads, Commits |
| **04-workflow** | 1 | Complete dehydration |
| **05-live-integration** | 2 | Real Songbird + BearDog binaries |

---

## Quick Start

```bash
# Build
cargo build --workspace

# Test
cargo test --workspace

# Coverage
cargo llvm-cov --workspace

# Benchmarks
cargo bench -p rhizo-crypt-core

# Docs
cargo doc --workspace --no-deps --open
```

### Features

```bash
# Persistent storage (RocksDB)
cargo build --features rocksdb

# Live primal connections
cargo build -p rhizo-crypt-core --features live-clients
```

### Live Integration (Phase 1 Binaries)

```bash
cd showcase/05-live-integration
./start-primals.sh      # Start Songbird, NestGate
./demo-live-discovery.sh # Connect to real Songbird
./demo-live-signing.sh   # Use real BearDog CLI
./stop-primals.sh        # Cleanup
```

---

## Crates

| Crate | Purpose | LOC |
|-------|---------|-----|
| `rhizo-crypt-core` | DAG, sessions, storage, clients | ~13,700 |
| `rhizo-crypt-rpc` | tarpc RPC, rate limiting, metrics | ~3,300 |

---

## Pure Rust RPC

24 methods via **tarpc** — no protobuf, no code generation:

```rust
#[tarpc::service]
pub trait RhizoCryptRpc {
    async fn create_session(request: CreateSessionRequest) -> Result<SessionId, RpcError>;
    async fn append_event(request: AppendEventRequest) -> Result<VertexId, RpcError>;
    async fn get_vertex(session_id: SessionId, vertex_id: VertexId) -> Result<Vertex, RpcError>;
    async fn health() -> Result<HealthStatus, RpcError>;
    // ... 20 more methods
}
```

---

## Storage Backends

| Backend | Status | Flag |
|---------|--------|------|
| In-Memory | ✅ Default | — |
| RocksDB | ✅ Complete | `--features rocksdb` |
| LMDB | 📋 Planned | — |

---

## Live Clients

Capability-based discovery via Songbird:

| Client | Protocol | Module | Status |
|--------|----------|--------|--------|
| **Songbird** | tarpc | `songbird_rpc.rs` | ✅ Wired |
| **BearDog** | HTTP | `beardog_http.rs` | ✅ Wired |
| **NestGate** | HTTP | `nestgate_http.rs` | ✅ Wired |
| **LoamSpine** | tarpc | `loamspine_rpc.rs` | ✅ Wired |
| **ToadStool** | HTTP | `toadstool_http.rs` | ✅ Wired (BYOB API) |
| **SweetGrass** | Provider | `sweetgrass.rs` | ✅ Verified |

```rust
use rhizo_crypt_core::clients::SongbirdClient;

let songbird = SongbirdClient::from_env();
songbird.connect().await?;
songbird.register("127.0.0.1:9400").await?;

let beardog = songbird.discover_beardog().await?;
```

Without `live-clients` feature, clients operate in **scaffolded mode**.

---

## Core Concepts

### Vertex
Content-addressed DAG node with Blake3 hash, parent links, event type, optional signature.

### Session
Scoped DAG: `Created → Active → Resolving → Resolved/Rolled Back`

### Slice
LoamSpine checkout: Copy, Loan, Consignment, Escrow, Waypoint, Transfer

### Dehydration
Commit session to LoamSpine with Merkle root and attestations.

---

## Performance

| Operation | Time |
|-----------|------|
| Vertex creation | ~720 ns |
| Blake3 hash (4KB) | ~80 ns |
| DAG put_vertex | ~1.6 µs |
| DAG get_vertex | ~270 ns |
| Merkle root (1k) | ~750 µs |

---

## Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                       rhizoCrypt                             │
├─────────────────────────────────────────────────────────────┤
│     tarpc RPC (24 methods) + Rate Limiting + Metrics        │
├─────────────────────────────────────────────────────────────┤
│  ┌─────────┐  ┌─────────┐  ┌─────────┐  ┌─────────────┐    │
│  │ Vertex  │  │  DAG    │  │ Merkle  │  │  Sessions   │    │
│  │ Store   │  │ Index   │  │ Trees   │  │  (scopes)   │    │
│  └─────────┘  └─────────┘  └─────────┘  └─────────────┘    │
│  ┌─────────┐  ┌─────────────┐  ┌───────────────────────┐    │
│  │ Slices  │  │ Dehydration │  │    Live Clients       │    │
│  └─────────┘  └─────────────┘  └───────────────────────┘    │
│  ┌─────────────────────────────────────────────────────┐    │
│  │    Storage: InMemory | RocksDB | (LMDB planned)     │    │
│  └─────────────────────────────────────────────────────┘    │
└─────────────────────────────────────────────────────────────┘
```

---

## Testing

| Type | Count |
|------|-------|
| Unit | 190 |
| Discovery | 21 |
| Integration | 21 |
| E2E | 8 |
| Chaos | 18 |
| RPC | 10 |
| Property | 17 |
| **Total** | **263** |

---

## Integration

### Gen 1 Primals
- **BearDog** — DIDs, signatures ✅
- **Songbird** — Service discovery ✅
- **NestGate** — Payload storage ✅
- **ToadStool** — Compute events ✅ (BYOB HTTP API)

### Phase 2 Siblings
- **LoamSpine** — Receives commits ✅
- **SweetGrass** — Provenance queries ✅ (rhizoCrypt is provider)

---

## Key Principles

1. **Ephemeral by default** — Designed to be forgotten
2. **Content-addressed** — Blake3 hashes
3. **Multi-parent DAG** — Not just a chain
4. **Selective permanence** — Only commits survive
5. **Pure Rust** — No protobuf
6. **Capability-based** — Runtime discovery

---

## Documentation

| Document | Description |
|----------|-------------|
| [showcase/](./showcase/) | **Interactive demos** — start here! |
| [STATUS.md](./STATUS.md) | Implementation status |
| [START_HERE.md](./START_HERE.md) | Developer guide |
| [WHATS_NEXT.md](./WHATS_NEXT.md) | Roadmap |
| [specs/](./specs/) | Full specifications |

---

## License

AGPL-3.0

---

*rhizoCrypt: The memory that knows when to forget.*
