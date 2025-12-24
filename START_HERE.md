# 🔐 rhizoCrypt — Start Here

**Version**: 0.9.2  
**Status**: Production Ready

---

## What is rhizoCrypt?

An **ephemeral DAG engine** for Phase 2 of ecoPrimals. Think "git for events" — a content-addressed Directed Acyclic Graph that captures everything during a session, then selectively forgets most of it.

> **Key insight**: rhizoCrypt is designed to be forgotten. Only what's committed to LoamSpine survives.

---

## 🎭 Try the Showcase First

The fastest way to understand rhizoCrypt:

```bash
cd showcase && ./QUICK_START.sh
```

**12 demos** covering sessions, DAG operations, Merkle proofs, slices, discovery, signing, payloads, commits, and live integration.

---

## Quick Start

```bash
# Build
cargo build --workspace

# Test (260 tests)
cargo test --workspace

# Coverage (85%)
cargo llvm-cov --workspace

# Benchmarks
cargo bench -p rhizo-crypt-core

# Docs
cargo doc --workspace --no-deps --open
```

### Feature Flags

```bash
# Persistent storage (RocksDB)
cargo build --features rocksdb

# Live client connections
cargo build -p rhizo-crypt-core --features live-clients
```

---

## Architecture

```
rhizoCrypt
├── tarpc RPC (24 methods)
│   ├── Sessions: create, get, list, discard
│   ├── Events: append, append_batch
│   ├── Queries: get_vertex, get_frontier
│   ├── Merkle: get_root, get_proof, verify
│   ├── Slices: checkout, resolve
│   └── Dehydration: dehydrate, get_status
│
├── Production Hardening
│   ├── RateLimiter (token bucket)
│   ├── MetricsCollector (Prometheus)
│   └── Graceful Shutdown
│
├── Capability Discovery (primal-agnostic)
│   ├── SafeEnv — type-safe environment config
│   ├── CapabilityEnv — capability endpoint resolution
│   └── DiscoveryRegistry — runtime service discovery
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
State checkout from permanent storage:
- **Copy** — Local use only
- **Loan** — Auto-returns
- **Consignment** — Temporary possession
- **Escrow** — Multi-party agreement
- **Waypoint** — Anchors to spine
- **Transfer** — Ownership transfer

### Capability Discovery
```rust
use rhizo_crypt_core::{SafeEnv, CapabilityEnv};

// Type-safe environment config
let port: u16 = SafeEnv::parse("RHIZOCRYPT_PORT", 9400);

// Capability-based endpoint discovery
if let Some(endpoint) = CapabilityEnv::signing_endpoint() {
    // Connect to signing service
}
```

---

## Project Structure

```
rhizoCrypt/
├── Cargo.toml              # Workspace
├── crates/
│   ├── rhizo-crypt-core/   # Core library
│   │   ├── src/
│   │   │   ├── lib.rs      # RhizoCrypt primal
│   │   │   ├── clients/    # Capability clients
│   │   │   ├── discovery.rs
│   │   │   ├── safe_env.rs # Environment config
│   │   │   ├── store.rs
│   │   │   ├── vertex.rs
│   │   │   ├── session.rs
│   │   │   ├── merkle.rs
│   │   │   ├── slice.rs
│   │   │   └── dehydration.rs
│   │   └── tests/
│   │
│   └── rhizo-crypt-rpc/    # RPC layer
│       ├── src/
│       │   ├── service.rs
│       │   ├── server.rs
│       │   ├── client.rs
│       │   ├── rate_limit.rs
│       │   └── metrics.rs
│       └── tests/
│
├── showcase/               # 12 interactive demos
├── specs/                  # Specifications
├── README.md
├── STATUS.md
└── WHATS_NEXT.md
```

---

## Primal-Agnostic Design

rhizoCrypt follows **infant discovery**: it starts with zero knowledge and discovers capabilities at runtime.

| Pattern | Description |
|---------|-------------|
| `service_id` | Agnostic service identifier (not primal name) |
| `Capability` | What a service does, not who provides it |
| `IntegrationStatus` | Uses `signing`, `permanent_storage`, `payload_storage` |
| `SafeEnv` | Type-safe environment configuration |

---

## Testing

| Type | Count | Command |
|------|-------|---------|
| Unit | 183 | `cargo test --lib` |
| Integration | 18 | `cargo test integration::` |
| E2E | 8 | `cargo test --test e2e_tests` |
| Chaos | 18 | `cargo test --test chaos_tests` |
| Property | 17 | `cargo test --test property_tests` |
| RPC | 10 | `cargo test -p rhizo-crypt-rpc` |

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
| SafeEnv | ✅ Type-safe config |
| Tests | ✅ 260 passing |
| Coverage | ✅ 85% |

---

## Further Reading

| Document | Description |
|----------|-------------|
| [showcase/](./showcase/) | Interactive demos |
| [STATUS.md](./STATUS.md) | Implementation status |
| [WHATS_NEXT.md](./WHATS_NEXT.md) | Roadmap |
| [specs/](./specs/) | Full specifications |

---

*rhizoCrypt: The memory that knows when to forget.*
