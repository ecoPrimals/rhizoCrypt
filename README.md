# 🔐 rhizoCrypt

**Ephemeral DAG Engine** — Phase 2 Working Memory

---

## Status

| Metric | Value |
|--------|-------|
| **Version** | 0.9.2 |
| **Tests** | ✅ 260 passing |
| **Coverage** | ✅ 85% |
| **Clippy** | ✅ Clean |
| **Unsafe** | ✅ 0 blocks |
| **Architecture** | ✅ Primal-agnostic |

---

## Quick Start

```bash
# Build
cargo build --workspace

# Test
cargo test --workspace

# Showcase (12 interactive demos)
cd showcase && ./QUICK_START.sh
```

### Features

```bash
# Persistent storage
cargo build --features rocksdb

# Live capability discovery
cargo build -p rhizo-crypt-core --features live-clients
```

---

## What is rhizoCrypt?

A content-addressed DAG engine designed to **forget**. Sessions capture events, explore branches, then either commit results to permanent storage (LoamSpine) or gracefully expire.

```
         ┌──○──┐                    
         │     │                    ○ = Event vertex
    ○────┼──○──┼────○               │ = DAG edge
         │     │                    
    ○────┼──○──┼────○              Branches, explores,
         │     │                   then resolves
         └──○──┘                    
             │
             ▼
    ═══════════════════            → LoamSpine (permanent)
```

---

## Core Concepts

| Concept | Description |
|---------|-------------|
| **Vertex** | Content-addressed event (Blake3 hash) |
| **Session** | Scoped DAG with lifecycle |
| **Slice** | Checkout from permanent storage |
| **Dehydration** | Commit session to LoamSpine |

---

## Crates

| Crate | Purpose |
|-------|---------|
| `rhizo-crypt-core` | DAG, sessions, storage, clients |
| `rhizo-crypt-rpc` | tarpc RPC, rate limiting, metrics |

---

## Architecture

### Primal-Agnostic Design

rhizoCrypt follows **infant discovery**: it starts with zero knowledge of other primals and discovers capabilities at runtime.

```rust
use rhizo_crypt_core::{SafeEnv, CapabilityEnv};

// Discover by capability, not primal name
let signing_endpoint = CapabilityEnv::signing_endpoint();
let storage_endpoint = CapabilityEnv::permanent_commit_endpoint();
```

### Capability Discovery

| Capability | Description |
|------------|-------------|
| `crypto:signing` | DID signing operations |
| `payload:storage` | Content-addressed payloads |
| `storage:permanent:commit` | Immutable commits |
| `compute:orchestration` | Compute task scheduling |
| `provenance:query` | Attribution tracking |

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

## Testing

| Type | Count |
|------|-------|
| Unit | 183 |
| Integration | 18 |
| Chaos | 18 |
| E2E | 8 |
| Property | 17 |
| RPC | 10 |
| **Total** | **260** |

---

## Key Principles

1. **Ephemeral by default** — Designed to be forgotten
2. **Content-addressed** — Blake3 hashes for integrity
3. **Multi-parent DAG** — Not just a chain
4. **Selective permanence** — Only commits survive
5. **Pure Rust** — No protobuf, no unsafe
6. **Capability-based** — Runtime discovery, not hardcoding
7. **Primal-agnostic** — Knows only itself

---

## Documentation

| Document | Description |
|----------|-------------|
| [showcase/](./showcase/) | Interactive demos |
| [START_HERE.md](./START_HERE.md) | Developer guide |
| [STATUS.md](./STATUS.md) | Implementation status |
| [WHATS_NEXT.md](./WHATS_NEXT.md) | Roadmap |
| [specs/](./specs/) | Full specifications |

---

## License

AGPL-3.0

---

*rhizoCrypt: The memory that knows when to forget.*
