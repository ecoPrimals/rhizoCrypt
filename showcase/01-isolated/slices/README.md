# 🔐 rhizoCrypt Slice Semantics Demos

**Phase 1: Isolated Instance — Slices & Checkouts**

---

## 📋 What You'll Learn

- Slice modes (Copy, Loan, Consignment)
- Checkout patterns
- Rehydration from commits

---

## 🚀 Quick Start

```bash
# Run the slice semantics demo
./demo-slice-semantics.sh
```

---

## 📁 Available Demos

### 1. `demo-slice-semantics.sh`
**Time:** 5 minutes  
**Complexity:** Intermediate

Demonstrates:
- Creating slices from sessions
- Slice mode semantics
- Checkout workflows
- Interaction with LoamSpine (simulated)

---

## 🗂️ Slice Modes

| Mode | Description | Use Case |
|------|-------------|----------|
| `Copy` | Independent copy, local use | Experimentation |
| `Loan` | Temporary use with terms | Lending, sharing |
| `Consignment` | Third-party custody | Escrow, marketplace |

---

## 🔐 Why Slices?

1. **Selective Access**: Only checkout what you need
2. **Controlled Sharing**: Different modes for different needs
3. **Rehydration**: Reconstruct from commits
4. **Sovereignty**: User controls sharing terms

---

## 📥 Checkout Flow

```
┌──────────────┐
│  LoamSpine   │ ← Permanent storage
│   (Commit)   │
└──────┬───────┘
       │ checkout
       ▼
┌──────────────┐
│  rhizoCrypt  │ ← Ephemeral session
│   (Slice)    │
└──────────────┘
```

---

## 🔗 Next Steps

After understanding slices:
1. Explore `../merkle/` for Merkle proofs
2. Try `../../04-complete-workflow/` for dehydration
3. See how slices interact with LoamSpine

---

*rhizoCrypt: Memory that knows when to remember*
