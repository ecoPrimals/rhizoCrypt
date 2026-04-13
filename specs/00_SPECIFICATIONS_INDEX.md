# 🔐 RhizoCrypt — Specifications Index

**Last Updated**: April 13, 2026  
**Version**: 0.14.0-dev  
**Status**: Active Development

---

## Overview

This directory contains the complete specification suite for RhizoCrypt, the ephemeral DAG engine of the ecoPrimals Phase 2 infrastructure. RhizoCrypt provides the "working memory" layer where complex, branching operations occur before resolving to permanent state in LoamSpine.

---

## 🌿 Core Principles

### Pure Rust
RhizoCrypt follows the ecoPrimals commitment to **pure Rust**:
- No protobuf, no external code generation
- **tarpc** for RPC — compile-time type safety via Rust traits
- Lean into the Rust compiler, not external tooling
- Zero `unsafe` blocks

### Primal Sovereignty
Every primal in the ecosystem maintains:
- **Data sovereignty** — Users own their data
- **Consent-based operations** — Agents act only with explicit consent
- **Cryptographic provenance** — All operations are auditable
- **Minimal trust** — Verify, don't trust

### Human Dignity
RhizoCrypt respects human dignity by:
- **Ephemeral by default** — Data is forgotten unless explicitly committed
- **Selective permanence** — Only what matters is preserved
- **No surveillance** — Working memory is not a record
- **User control** — Sessions are owned by their creators

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
| [API_SPECIFICATION.md](./API_SPECIFICATION.md) | tarpc & REST API definitions | ✅ Complete |
| [EVENT_TYPE_REFERENCE.md](./EVENT_TYPE_REFERENCE.md) | `dag.event.append` — 27-variant wire format for domain springs | ✅ Complete |

### Integration Specifications

| Document | Purpose | Status |
|----------|---------|--------|
| [INTEGRATION_SPECIFICATION.md](./archive/INTEGRATION_SPECIFICATION.md) | Legacy v1 integration (superseded by V2) | 📦 Archived |
| [INTEGRATION_SPECIFICATION_V2.md](./INTEGRATION_SPECIFICATION_V2.md) | Capability-based integration (current) | ✅ Complete |
| [STORAGE_BACKENDS.md](./STORAGE_BACKENDS.md) | redb (default), in-memory | ✅ Complete |

### Experimental Specifications

| Document | Purpose | Status |
|----------|---------|--------|
| [CONTENT_INDEX_EXPERIMENT.md](./CONTENT_INDEX_EXPERIMENT.md) | Locality-sensitive cross-session similarity index | 🧪 Proposed |

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
- [Phase 2 Architecture](./ARCHITECTURE.md)

---

## 📖 Reading Order

For new developers, we recommend this reading order:

1. **[ARCHITECTURE.md](./ARCHITECTURE.md)** — Understand the big picture
2. **[DATA_MODEL.md](./DATA_MODEL.md)** — Learn the core data structures
3. **[SLICE_SEMANTICS.md](./SLICE_SEMANTICS.md)** — Understand the Rhizo-Loam layering
4. **[DEHYDRATION_PROTOCOL.md](./DEHYDRATION_PROTOCOL.md)** — Learn how DAGs commit to permanence
5. **[API_SPECIFICATION.md](./API_SPECIFICATION.md)** — See the external interfaces (pure Rust tarpc)
6. **[INTEGRATION_SPECIFICATION_V2.md](./INTEGRATION_SPECIFICATION_V2.md)** — Understand capability-based primal interactions
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
| **tarpc** | Pure Rust RPC via traits — no protobuf |
| **Content Index** | Locality-sensitive secondary index for cross-session similarity (experimental) |

---

## 🛡️ Sovereignty Guarantees

RhizoCrypt provides these sovereignty guarantees:

| Guarantee | Implementation |
|-----------|----------------|
| **Data ownership** | Session creator owns all vertices |
| **Consent tracking** | Agent DIDs recorded on every event |
| **Audit trail** | Full DAG preserved until resolution |
| **Selective forget** | Only dehydrated summaries persist |
| **Cryptographic proof** | Merkle proofs verify inclusion |
| **No vendor lock-in** | Pure Rust, no external dependencies |

---

## 🏗️ Implementation Status

See [../README.md](../README.md) for current implementation status and [../CHANGELOG.md](../CHANGELOG.md) for version history.

---

*RhizoCrypt: The memory that knows when to forget.*
