# 🔐 rhizoCrypt Isolated Mode Demos

**Phase 1: Single-Instance Capabilities**

---

## 📋 Overview

These demos show rhizoCrypt's core capabilities without any external dependencies.

No Songbird. No BearDog. No network. Just rhizoCrypt.

---

## 📁 Available Demo Directories

### `sessions/`
**Topic:** Session Lifecycle  
**Demos:** 3 planned

- `demo-session-lifecycle.sh` ✅ — Create, grow, resolve sessions
- `demo-session-types.sh` — Different session types
- `demo-session-constraints.sh` — Max vertices, TTL, metadata

### `dag/`
**Topic:** DAG Operations  
**Demos:** 3 planned

- `demo-dag-operations.sh` ✅ — Multi-parent vertices, queries
- `demo-content-addressing.sh` — Blake3, deduplication
- `demo-dag-traversal.sh` — Parent/child navigation

### `merkle/`
**Topic:** Merkle Trees & Proofs  
**Demos:** 3 planned

- `demo-merkle-basics.sh` — Root computation
- `demo-proof-generation.sh` — Inclusion proofs
- `demo-proof-verification.sh` — Verify proofs

### `slices/`
**Topic:** Slice Semantics  
**Demos:** 6 planned (one per mode)

- `demo-slice-copy.sh` — Copy mode
- `demo-slice-loan.sh` — Loan with auto-return
- `demo-slice-escrow.sh` — Multi-party escrow
- `demo-slice-gift.sh` — Permanent transfer
- `demo-slice-transform.sh` — In-place modification
- `demo-slice-view.sh` — Read-only access

---

## 🚀 Quick Start

```bash
# Make scripts executable
chmod +x **/*.sh

# Run individual demos
./sessions/demo-session-lifecycle.sh
./dag/demo-dag-operations.sh

# Or use the quick start from parent
cd ..
./QUICK_START.sh
```

---

## 📊 What You'll Learn

| Topic | Key Concepts |
|-------|-------------|
| **Sessions** | Lifecycle, types, constraints, scoping |
| **DAG** | Multi-parent, content-addressing, queries |
| **Merkle** | Root computation, proof generation/verification |
| **Slices** | 6 modes for data sovereignty |

---

## ⏱️ Estimated Time

- **Sessions:** 10 minutes
- **DAG:** 15 minutes
- **Merkle:** 15 minutes
- **Slices:** 30 minutes
- **Total:** ~1 hour

---

## 🔗 Next Steps

After completing Phase 1:
1. Explore `../02-rpc/` for RPC layer
2. Try `../03-inter-primal/` for ecosystem integration
3. See `../04-complete-workflow/` for full dehydration

---

*rhizoCrypt Phase 1: Core capabilities without dependencies*

