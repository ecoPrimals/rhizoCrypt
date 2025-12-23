# 🔐 RhizoCrypt — Project Status

**Last Updated**: December 22, 2025  
**Version**: 0.1.0  
**Status**: 🌱 **Scaffolded** — Ready for Core Implementation  
**Grade**: N/A (Pre-implementation)

---

## 📊 Current State

### Build Status
| Metric | Status |
|--------|--------|
| **Compilation** | ✅ Clean |
| **Tests** | ✅ 0/0 (scaffold only) |
| **Linting** | ✅ Clean (pedantic clippy) |
| **Documentation** | 🟡 Scaffold docs only |

### Implementation Progress

| Component | Status | Notes |
|-----------|--------|-------|
| **Core Traits** | ✅ Done | `PrimalLifecycle`, `PrimalHealth` |
| **Configuration** | ✅ Done | Basic `RhizoCryptConfig` |
| **Error Types** | ✅ Done | Basic `RhizoCryptError` |
| **Vertex Structure** | ⬜ Not Started | Core data type |
| **Session Management** | ⬜ Not Started | DAG scoping |
| **Event Ingestion** | ⬜ Not Started | High-performance append |
| **DAG Store** | ⬜ Not Started | In-memory first |
| **Merkle Trees** | ⬜ Not Started | Proof generation |
| **Dehydration** | ⬜ Not Started | LoamSpine commit |

---

## 🎯 What RhizoCrypt Does

RhizoCrypt is the **Core DAG Engine** — the git-like foundation for Phase 2:

```
┌─────────────────────────────────────────────────────────────────┐
│                        RhizoCrypt                                │
│                     (Core DAG Engine)                            │
│                                                                  │
│  ┌─────────┐  ┌─────────┐  ┌─────────┐  ┌─────────────┐        │
│  │ Vertex  │  │  DAG    │  │ Merkle  │  │  Sessions   │        │
│  │ Store   │  │ Index   │  │ Trees   │  │  (scopes)   │        │
│  └─────────┘  └─────────┘  └─────────┘  └─────────────┘        │
└─────────────────────────────────────────────────────────────────┘
```

**Key Concepts**:
- **Ephemeral by default** — designed to be forgotten
- **Content-addressed** — vertices identified by Blake3 hash
- **Multi-parent DAG** — not just a chain
- **Selective permanence** — only commits to LoamSpine survive

---

## 📁 Project Structure

```
rhizoCrypt/
├── Cargo.toml                    # Workspace manifest
├── README.md                     # Project overview
├── STATUS.md                     # This file
├── WHATS_NEXT.md                # Roadmap
├── START_HERE.md                # Developer guide
├── crates/
│   └── rhizo-crypt-core/        # Core library
│       ├── Cargo.toml
│       └── src/
│           ├── lib.rs           # Main entry
│           ├── config.rs        # Configuration
│           └── error.rs         # Error types
├── specs/
│   └── RHIZOCRYPT_SPECIFICATION.md  # Full spec (~800 lines)
└── showcase/                     # Demo applications
```

---

## 🔗 Dependencies

### Gen 1 Primals (Required)
| Primal | Purpose | Status |
|--------|---------|--------|
| **BearDog** | DIDs, Signatures | ✅ Ready |
| **Songbird** | Service Discovery | ✅ Ready |
| **NestGate** | Payload Storage | ✅ Ready |
| **ToadStool** | Event Source | ✅ Ready |

### Phase 2 Siblings
| Primal | Relationship | Status |
|--------|--------------|--------|
| **LoamSpine** | Receives commits | 🌱 Scaffolded |
| **SweetGrass** | Queries DAG | 🌱 Scaffolded |

---

## 📈 Metrics

```
Lines of Code:       ~100 (scaffold)
Test Coverage:       0% (no tests yet)
Unsafe Blocks:       0
Files:               3 source files
Dependencies:        sourdough-core
```

---

## 🚀 Next Milestone

**Phase 1: Core Data Structures** (Target: Week 1-2)

1. Implement `Vertex` struct
2. Implement `Session` struct
3. Add Blake3 content addressing
4. Basic in-memory DAG store

See [WHATS_NEXT.md](./WHATS_NEXT.md) for full roadmap.

---

## 📚 Key Documents

| Document | Purpose |
|----------|---------|
| [README.md](./README.md) | Project overview |
| [START_HERE.md](./START_HERE.md) | Developer onboarding |
| [WHATS_NEXT.md](./WHATS_NEXT.md) | Implementation roadmap |
| [specs/RHIZOCRYPT_SPECIFICATION.md](./specs/RHIZOCRYPT_SPECIFICATION.md) | Full specification |

---

*RhizoCrypt: The memory that knows when to forget.*

