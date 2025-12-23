# 🔐 RhizoCrypt — Start Here

Welcome to RhizoCrypt! This guide will get you up to speed quickly.

---

## 📖 What is RhizoCrypt?

RhizoCrypt is the **Core DAG Engine** for Phase 2 of ecoPrimals. Think of it as "git for events" — a content-addressed Directed Acyclic Graph (DAG) that captures everything that happens during a session, then selectively forgets most of it.

**Key insight**: RhizoCrypt is designed to be forgotten. Only what's committed to LoamSpine survives.

---

## 🚀 Quick Start

### 1. Build the Project

```bash
cd /path/to/ecoPrimals/phase2/rhizoCrypt
cargo build
```

### 2. Run Tests

```bash
cargo test
```

### 3. Explore the Code

```bash
# Main entry point
cat crates/rhizo-crypt-core/src/lib.rs

# Configuration
cat crates/rhizo-crypt-core/src/config.rs

# Error types
cat crates/rhizo-crypt-core/src/error.rs
```

---

## 🏗️ Architecture Overview

```
RhizoCrypt
    │
    ├── Sessions (scoped DAGs)
    │   ├── Vertices (events)
    │   │   ├── Content-addressed (Blake3 hash)
    │   │   ├── Parent links (DAG structure)
    │   │   ├── Timestamps (nanosecond precision)
    │   │   └── Optional signatures (BearDog DIDs)
    │   │
    │   ├── Frontier (DAG tips)
    │   └── Genesis (DAG roots)
    │
    ├── Merkle Trees (proofs)
    │   ├── Root computation
    │   └── Inclusion proofs
    │
    └── Dehydration (→ LoamSpine)
        ├── Summary generation
        └── Commit to permanence
```

---

## 📚 Key Concepts

### 1. Vertices
The fundamental unit. Each vertex is:
- **Content-addressed** — ID is Blake3 hash of contents
- **Linked** — References parent vertices by hash
- **Typed** — Has an event type (gaming, scientific, etc.)
- **Optional signature** — BearDog DID attestation

### 2. Sessions
A scoped DAG with a lifecycle:
1. **Creation** — New session spawned
2. **Growth** — Events appended as vertices
3. **Resolution** — Session completes (success/failure)
4. **Dehydration** — Important results → LoamSpine
5. **Expiration** — DAG garbage collected

### 3. Merkle Trees
Enable verification without the full DAG:
- **Root** — Single hash representing entire session
- **Proofs** — Verify a vertex was in the session

### 4. The Philosophy of Forgetting
> Most data should be temporary. Only what matters should be permanent.

RhizoCrypt captures everything, then throws most of it away. This is a feature, not a bug.

---

## 📂 Project Structure

```
rhizoCrypt/
├── Cargo.toml           # Workspace manifest
├── README.md            # Overview
├── STATUS.md            # Current status
├── WHATS_NEXT.md        # Roadmap
├── START_HERE.md        # This file
│
├── crates/
│   └── rhizo-crypt-core/    # Core library
│       ├── Cargo.toml
│       └── src/
│           ├── lib.rs       # Entry + traits
│           ├── config.rs    # Configuration
│           └── error.rs     # Error types
│
├── specs/
│   └── RHIZOCRYPT_SPECIFICATION.md  # Full spec (~800 lines)
│
└── showcase/            # Demo applications (coming soon)
```

---

## 🔗 Integration Points

### Depends On (Gen 1)
| Primal | Purpose |
|--------|---------|
| **BearDog** | DIDs, signatures, lineage proofs |
| **Songbird** | Service discovery, UPA registration |
| **NestGate** | Payload storage (content-addressed) |
| **ToadStool** | Primary event source (compute tasks) |

### Phase 2 Siblings
| Primal | Relationship |
|--------|--------------|
| **LoamSpine** | Receives dehydrated commits |
| **SweetGrass** | Queries DAG for attribution |

---

## 🎯 Current Status

| Aspect | Status |
|--------|--------|
| **Scaffolding** | ✅ Complete |
| **Build** | ✅ Passing |
| **Core Types** | ⬜ Not started |
| **DAG Storage** | ⬜ Not started |
| **Merkle Trees** | ⬜ Not started |

See [STATUS.md](./STATUS.md) for detailed status.

---

## 📝 Next Steps for Contributors

### Immediate (Week 1)
1. Add `blake3` dependency
2. Implement `VertexId` type
3. Implement `Vertex` struct
4. Add vertex hashing

### Short Term (Weeks 2-4)
1. Implement `Session` struct
2. Implement event ingestion
3. Implement in-memory DAG store

See [WHATS_NEXT.md](./WHATS_NEXT.md) for full roadmap.

---

## 📖 Further Reading

| Document | Description |
|----------|-------------|
| [specs/RHIZOCRYPT_SPECIFICATION.md](./specs/RHIZOCRYPT_SPECIFICATION.md) | Complete technical specification |
| [../ARCHITECTURE.md](../ARCHITECTURE.md) | Unified Phase 2 architecture |
| [../INTEGRATION_OVERVIEW.md](../INTEGRATION_OVERVIEW.md) | Cross-primal data flows |
| [../sourDough/CONVENTIONS.md](../sourDough/CONVENTIONS.md) | Coding conventions |

---

## 💡 The Raid Analogy

From the specification:

> **RhizoCrypt** tracks every chaotic event *within* a raid:
> - Player movements and positions
> - Items looted, dropped, traded
> - Shots fired, damage dealt
> 
> **LoamSpine** receives the *validated extraction*:
> - Final inventory delta
> - XP gained
> - Proof of legitimate acquisition
>
> Items without valid DAG history are **rejected**.

Anti-cheat is structural, not bolted on.

---

## ❓ Questions?

- Check [STATUS.md](./STATUS.md) for current state
- Check [WHATS_NEXT.md](./WHATS_NEXT.md) for roadmap
- Read the [specification](./specs/RHIZOCRYPT_SPECIFICATION.md) for deep details

---

*RhizoCrypt: The memory that knows when to forget.*

