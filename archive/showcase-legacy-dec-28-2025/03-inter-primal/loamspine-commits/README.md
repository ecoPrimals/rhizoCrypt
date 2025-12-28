# 🔐 rhizoCrypt LoamSpine Commit Demos

**Phase 3: Inter-Primal — Permanent Storage & Commits**

---

## 📋 What You'll Learn

- Committing sessions to permanent storage
- Merkle root preservation
- Slice checkout from commits
- Lineage tracking

---

## 🚀 Quick Start

```bash
# Run the commit demo
./demo-loamspine-commit.sh
```

---

## 🔗 Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                        rhizoCrypt                                │
│  ┌───────────────────────────────────────────────────────────┐  │
│  │              Session (ephemeral)                           │  │
│  │  [v1] ─→ [v2] ─→ [v3] ─→ ... ─→ [vN]                      │  │
│  │                     │                                       │  │
│  │               Merkle Root                                   │  │
│  └───────────────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────────────┘
                           │
                           │ commit (dehydration)
                           ▼
┌─────────────────────────────────────────────────────────────────┐
│                        LoamSpine                                 │
│  ┌───────────────────────────────────────────────────────────┐  │
│  │              Commit (permanent)                            │  │
│  │  • Session ID                                              │  │
│  │  • Merkle Root (proves all vertices)                       │  │
│  │  • Attestations (who contributed)                          │  │
│  │  • Payload References (NestGate)                           │  │
│  │  • Lineage (parent commits)                                │  │
│  └───────────────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────────────┘
```

---

## 💡 Why LoamSpine?

| Aspect | rhizoCrypt | LoamSpine |
|--------|------------|-----------|
| Lifetime | Ephemeral | Permanent |
| Stores | Full DAG | Merkle Root |
| Size | Large | Compact |
| Use | Active work | Archive |

---

## 📜 Commit Contents

```rust
pub struct Commit {
    /// Unique commit identifier
    pub id: CommitId,
    
    /// Original session ID
    pub session_id: SessionId,
    
    /// Merkle root proving all vertices
    pub merkle_root: MerkleRoot,
    
    /// Participant attestations
    pub attestations: Vec<Attestation>,
    
    /// References to payloads in NestGate
    pub payload_refs: Vec<PayloadRef>,
    
    /// Parent commits (for lineage)
    pub parents: Vec<CommitId>,
}
```

---

*rhizoCrypt: Ephemeral sessions, permanent commits*

