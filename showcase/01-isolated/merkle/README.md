# 🔐 rhizoCrypt Merkle Tree Demos

**Phase 1: Isolated Instance — Merkle Trees & Proofs**

---

## 📋 What You'll Learn

- Merkle tree construction from vertices
- Computing the Merkle root
- Generating inclusion proofs
- Verifying proofs efficiently

---

## 🚀 Quick Start

```bash
# Run the Merkle proofs demo
./demo-merkle-proofs.sh
```

---

## 📁 Available Demos

### 1. `demo-merkle-proofs.sh`
**Time:** 5 minutes  
**Complexity:** Intermediate

Demonstrates:
- Building a Merkle tree with 8 vertices
- Computing the root hash
- Generating an inclusion proof
- Verifying the proof
- Proof efficiency analysis

---

## 🌳 Merkle Tree Structure

```
                    [Root]
                   /      \
               [H01]      [H23]
              /    \      /    \
           [H0] [H1]  [H2] [H3]
           /\   /\    /\   /\
          v1 v2 v3 v4 v5 v6 v7 v8
```

Each internal node is computed as:
```
H(parent) = blake3(H(left) || H(right))
```

---

## 📜 Inclusion Proofs

An inclusion proof shows that a specific vertex is part of the tree:

| Vertices | Proof Size | Complexity |
|----------|------------|------------|
| 8 | 3 hashes | O(log 8) |
| 1,000 | 10 hashes | O(log 1000) |
| 1,000,000 | 20 hashes | O(log 1M) |

**Key Insight**: Proof size grows logarithmically, not linearly!

---

## 🔐 Why Merkle Trees?

1. **Integrity**: Single root proves all vertices unchanged
2. **Efficiency**: O(log n) proof size and verification
3. **Dehydration**: Root is what gets committed to LoamSpine
4. **Selective Disclosure**: Prove specific vertices without revealing all

---

## 💡 Key Concepts

### Root Hash
A single 32-byte hash that represents the entire session:
```rust
let root = primal.compute_merkle_root(session_id).await?;
```

### Inclusion Proof
A path from leaf to root:
```rust
let proof = primal.generate_merkle_proof(session_id, vertex_id).await?;
```

### Verification
Check if a vertex is included without the full tree:
```rust
let valid = proof.verify(&vertex_id, &root);
```

---

## 🔗 Next Steps

After understanding Merkle trees:
1. Explore `../slices/` for slice semantics
2. Try `../../04-complete-workflow/` for dehydration
3. See how Merkle roots are committed to LoamSpine

---

*rhizoCrypt: O(log n) proofs for any session size*

