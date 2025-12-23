# 🔐 RhizoCrypt — Specifications Index

**Last Updated**: December 22, 2025  
**Version**: 0.2.0  
**Status**: Active Development

---

## Overview

This directory contains the complete specification suite for RhizoCrypt, the ephemeral DAG engine of the ecoPrimals Phase 2 infrastructure. RhizoCrypt provides the "working memory" layer where complex, branching operations occur before resolving to permanent state in LoamSpine.

---

## 📚 Specification Documents

### Core Specifications

| Document | Purpose | Status |
|----------|---------|--------|
| [RHIZOCRYPT_SPECIFICATION.md](./RHIZOCRYPT_SPECIFICATION.md) | Master specification document | ✅ Complete |
| [ARCHITECTURE.md](./ARCHITECTURE.md) | High-level architecture & component overview | ✅ Complete |
| [DATA_MODEL.md](./DATA_MODEL.md) | Vertex, Session, DAG data structures | ✅ Complete |

### Protocol Specifications

| Document | Purpose | Status |
|----------|---------|--------|
| [SLICE_SEMANTICS.md](./SLICE_SEMANTICS.md) | Slice modes, resolution routing, waypoints | ✅ Complete |
| [DEHYDRATION_PROTOCOL.md](./DEHYDRATION_PROTOCOL.md) | DAG → LoamSpine commit protocol | ✅ Complete |
| [API_SPECIFICATION.md](./API_SPECIFICATION.md) | gRPC & REST API definitions | ✅ Complete |

### Integration Specifications

| Document | Purpose | Status |
|----------|---------|--------|
| [INTEGRATION_SPECIFICATION.md](./INTEGRATION_SPECIFICATION.md) | BearDog, LoamSpine, SweetGrass, ToadStool | ✅ Complete |
| [STORAGE_BACKENDS.md](./STORAGE_BACKENDS.md) | In-memory, RocksDB, LMDB store implementations | ✅ Complete |

---

## 🧬 Biological Model

RhizoCrypt is named after the **rhizome**—the branching, underground fungal network:

```
         ┌──○──┐                    The Rhizome
         │     │
    ○────┼──○──┼────○              ○ = Event vertex
         │     │                    │ = DAG edge
    ○────┼──○──┼────○              Branches, explores,
         │     │                    and eventually resolves
         └──○──┘                    to the Loam layer below
             │
             ▼
    ═══════════════════            LoamSpine (permanent)
```

---

## 🔗 Related Specifications

### Phase 2 Siblings
- [LoamSpine Specification](../../loamSpine/specs/)
- [SweetGrass Specification](../../sweetGrass/specs/)

### Gen 1 Dependencies
- [BearDog Specification](../../../beardog/specs/)
- [Songbird Specification](../../../songbird/specs/)
- [NestGate Specification](../../../nestgate/specs/)
- [ToadStool Specification](../../../toadstool/specs/)

### Foundational
- [sourDough Core](../../sourDough/)
- [Phase 2 Architecture](../../ARCHITECTURE.md)

---

## 📖 Reading Order

For new developers, we recommend this reading order:

1. **[ARCHITECTURE.md](./ARCHITECTURE.md)** — Understand the big picture
2. **[DATA_MODEL.md](./DATA_MODEL.md)** — Learn the core data structures
3. **[SLICE_SEMANTICS.md](./SLICE_SEMANTICS.md)** — Understand the Rhizo-Loam layering
4. **[DEHYDRATION_PROTOCOL.md](./DEHYDRATION_PROTOCOL.md)** — Learn how DAGs commit to permanence
5. **[API_SPECIFICATION.md](./API_SPECIFICATION.md)** — See the external interfaces
6. **[INTEGRATION_SPECIFICATION.md](./INTEGRATION_SPECIFICATION.md)** — Understand primal interactions
7. **[RHIZOCRYPT_SPECIFICATION.md](./RHIZOCRYPT_SPECIFICATION.md)** — Full reference (read as needed)

---

## 🎯 Key Concepts Quick Reference

| Concept | Definition |
|---------|------------|
| **Vertex** | A single event in the DAG, content-addressed by Blake3 hash |
| **Session** | A scoped DAG with lifecycle (create → grow → resolve → expire) |
| **Slice** | A "checkout" of LoamSpine state into the DAG for async operations |
| **Dehydration** | The process of committing DAG results to LoamSpine |
| **Resolution** | When a DAG concludes: COMMIT, ROLLBACK, or WAYPOINT |
| **Frontier** | The tips of the DAG (vertices with no children) |
| **Merkle Root** | Cryptographic summary of entire session for proofs |

---

## 🏗️ Implementation Status

See [../STATUS.md](../STATUS.md) for current implementation progress.

See [../WHATS_NEXT.md](../WHATS_NEXT.md) for the development roadmap.

---

*RhizoCrypt: The memory that knows when to forget.*

