# 🔐 Level 2: DAG Engine

**Time**: 10 minutes  
**Skill**: Beginner  
**Prerequisites**: Level 1

---

## 🎯 Goal

Understand rhizoCrypt's multi-parent DAG operations - not just a blockchain!

---

## 📚 What You'll Learn

1. **Multi-Parent DAG** - Vertices can have multiple parents (not just a chain)
2. **Frontier Tracking** - DAG tips are tracked automatically
3. **Genesis Detection** - DAG roots are identified
4. **Topological Ordering** - Parents always come before children

---

## 🌳 DAG vs Blockchain

### Blockchain (Single Parent):
```
A → B → C → D
```

### DAG (Multi-Parent):
```
     A
    / \
   B   C
    \ / \
     D   E
```

**Key Difference**: DAG allows multiple parents, enabling complex branching and merging.

---

## 🚀 Demos

### Demo 1: Multi-Parent DAG
```bash
./demo-multi-parent.sh
```

**What it does**:
- Creates vertices with multiple parents
- Shows diamond pattern (merge scenario)
- Demonstrates branching and joining
- Visualizes DAG structure

**Concepts**:
- Vertices can have 0, 1, or many parents
- Multi-parent enables complex workflows
- Diamond pattern (common in merge scenarios)

---

### Demo 2: Frontier Tracking
```bash
./demo-frontier.sh
```

**What it does**:
- Shows how frontier updates as DAG grows
- Demonstrates automatic frontier management
- Visualizes DAG tips over time

**Concepts**:
- Frontier = vertices with no children
- Frontier updates automatically on vertex addition
- Frontier is always current (O(1) tracking)

---

### Demo 3: Genesis Detection
```bash
./demo-genesis.sh
```

**What it does**:
- Identifies DAG roots (vertices with no parents)
- Shows multiple genesis vertices
- Demonstrates genesis stability

**Concepts**:
- Genesis = vertices with no parents
- Can have multiple genesis vertices
- Genesis vertices are DAG entry points

---

### Demo 4: Topological Sort
```bash
./demo-topological-sort.sh
```

**What it does**:
- Sorts DAG in dependency order
- Shows parents always before children
- Demonstrates use in Merkle tree construction

**Concepts**:
- Topological order ensures parent-before-child
- Required for Merkle tree computation
- Detects cycles (DAG must be acyclic)

---

## 💡 Why Multi-Parent DAG?

### 1. **Parallel Operations**
```
   Task A
   /    \
Task B  Task C  ← Both can run in parallel
   \    /
   Task D       ← Depends on both B and C
```

### 2. **Merge Scenarios**
```
Version 1    Version 2
    \           /
     \         /
   Merged Version  ← Tracks both parent versions
```

### 3. **Attribution**
```
Agent 1 Action    Agent 2 Action
       \              /
        \            /
     Combined Result  ← Credits both agents
```

---

## 🔍 DAG Properties

### Directed
- Edges have direction (parent → child)
- Cannot traverse backwards in time

### Acyclic
- No cycles allowed
- Prevents temporal paradoxes
- Enables topological ordering

### Multi-Parent
- Vertices can have multiple parents
- Enables complex relationships
- Models real-world causality

---

## 📊 Performance Characteristics

| Operation | Time Complexity |
|-----------|----------------|
| Add vertex | O(1) |
| Find vertex | O(1) |
| Get frontier | O(1) |
| Get genesis | O(1) |
| Get children | O(c) where c = child count |
| Topological sort | O(n + e) where n = vertices, e = edges |

---

## 🎓 Advanced Concepts

### Frontier Evolution
```
Initial:  [A]
After B:  [B]          (A removed, B added)
After C:  [B, C]       (C added)
After D:  [D, C]       (B removed, D added)
```

### Genesis Stability
```
Once a genesis vertex is added, it remains genesis forever.
(Unless explicitly modified, which breaks immutability)
```

### Diamond Pattern
```
     A
    / \
   B   C
    \ /
     D    ← D has two parents: B and C
```

This is common in:
- Merge operations
- Multi-agent collaboration
- Parallel computation

---

## 🔗 Next Steps

**Ready for cryptographic integrity?** Move to Level 3:
```bash
cd ../03-merkle-proofs
cat README.md
```

---

*"Not just a chain - a graph of possibilities."* 🌳

