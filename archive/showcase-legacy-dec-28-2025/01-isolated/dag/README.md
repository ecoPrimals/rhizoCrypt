# 🔐 rhizoCrypt DAG Demos

**Phase 1: Isolated Instance — DAG Operations**

---

## 📋 What You'll Learn

- Content-addressed vertices (Blake3)
- Multi-parent DAG structures
- Frontier (leaf) queries
- Genesis (root) queries
- Child traversal

---

## 🚀 Quick Start

```bash
# Run the DAG operations demo
./demo-dag-operations.sh
```

---

## 📁 Available Demos

### 1. `demo-dag-operations.sh`
**Time:** 5 minutes  
**Complexity:** Beginner

Demonstrates:
- Building a complex DAG with branches and merges
- Multi-parent vertices (2+ parents)
- Frontier queries (find leaves)
- Genesis queries (find root)
- Content addressing (same content = same ID)

---

## 🌳 DAG Structure

Unlike git's single-parent commits, rhizoCrypt supports true DAGs with multiple parents:

```
      [v1 genesis]
         /    \
      [v2]    [v3]      ← Branching
        \    /   \
        [v4]     [v5]   ← v4 has 2 parents!
          \      /
          [v6 final]    ← v6 has 2 parents!
```

---

## 🔑 Content Addressing

Every vertex is identified by its Blake3 hash:

```rust
let id = vertex.compute_id(); // Blake3 of serialized vertex
```

**Same content = Same ID**

This enables:
- Deduplication
- Integrity verification
- Deterministic references

---

## 📊 DAG Operations

| Operation | Description | Time |
|-----------|-------------|------|
| `put_vertex` | Store vertex | ~1.6 µs |
| `get_vertex` | Retrieve by ID | ~270 ns |
| `get_frontier` | Find leaves | O(1) |
| `get_genesis` | Find root | O(1) |
| `get_children` | Get children | O(n) |

---

## 💡 Key Concepts

### Multi-Parent Vertices
```rust
let merge = VertexBuilder::new(EventType::DataUpdate)
    .with_parent(branch_a_id)
    .with_parent(branch_b_id)  // Multiple parents!
    .build();
```

### Frontier (Leaves)
Vertices with no children. Important for:
- Continuing work from latest state
- Computing Merkle roots
- Finding session "heads"

### Genesis (Root)
The first vertex in a session. Important for:
- Session verification
- DAG traversal
- Provenance queries

---

## 🔗 Next Steps

After understanding DAGs:
1. Explore `../merkle/` for proof generation
2. Try `../slices/` for slice semantics
3. See `../../02-rpc/` for RPC operations

---

*rhizoCrypt: Content-addressed, multi-parent DAG engine*

