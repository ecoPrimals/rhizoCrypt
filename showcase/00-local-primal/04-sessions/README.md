# 🔐 Level 4: Sessions & Lifecycle

**Time**: 10 minutes  
**Skill**: Intermediate  
**Prerequisites**: Level 3

---

## 🎯 Goal

Understand rhizoCrypt's session lifecycle and how sessions manage ephemeral DAGs.

---

## 📚 What You'll Learn

1. **Session Lifecycle** - Create, grow, resolve, expire
2. **Session Types** - Ephemeral vs Persistent
3. **Slices** - Checkout permanent storage into DAG
4. **Dehydration** - Commit DAG results to permanent storage

---

## 🔄 Session Lifecycle

```
┌─────────────┐
│   CREATE    │  ← New session with empty DAG
└──────┬──────┘
       │
       ▼
┌─────────────┐
│    GROW     │  ← Add vertices, build DAG
└──────┬──────┘
       │
       ▼
┌─────────────┐
│  RESOLVE    │  ← Finalize DAG, compute Merkle root
└──────┬──────┘
       │
       ▼
┌─────────────┐
│  DEHYDRATE  │  ← Commit to permanent storage (optional)
└──────┬──────┘
       │
       ▼
┌─────────────┐
│   EXPIRE    │  ← Session ends, DAG discarded
└─────────────┘
```

---

## 🚀 Demos

### Demo 1: Session Lifecycle
```bash
./demo-session-lifecycle.sh
```

**What it does**:
- Creates a session
- Grows the DAG with vertices
- Resolves the session
- Shows session expiry

**Concepts**:
- Session ID = unique identifier
- Session state = Active, Resolved, Expired
- DAG grows during Active state
- Resolution freezes DAG

---

### Demo 2: Ephemeral vs Persistent
```bash
./demo-ephemeral-persistent.sh
```

**What it does**:
- Compares ephemeral and persistent sessions
- Shows when each type is appropriate
- Demonstrates session cleanup

**Concepts**:
- **Ephemeral**: In-memory only, fast, discarded on expire
- **Persistent**: Optionally saved to storage
- Use ephemeral for temporary computations
- Use persistent for audit trails

---

### Demo 3: Slices (Checkout)
```bash
./demo-slices.sh
```

**What it does**:
- Checks out data from permanent storage (LoamSpine)
- Shows slice as DAG vertex
- Demonstrates immutable snapshots

**Concepts**:
- Slice = snapshot of permanent storage
- Slice checkout creates genesis vertex
- Read-only access to permanent data
- Enables working memory over permanent storage

---

### Demo 4: Dehydration (Commit)
```bash
./demo-dehydration.sh
```

**What it does**:
- Commits session results to permanent storage
- Shows dehydration protocol
- Demonstrates provenance tracking

**Concepts**:
- Dehydration = write DAG results to LoamSpine
- Only frontier vertices are committed
- Merkle root provides integrity proof
- Enables ephemeral-to-permanent workflow

---

## 💡 Why Sessions?

### 1. **Scoped Working Memory**
```
Session 1: User A's computation
Session 2: User B's computation
→ Isolated, no cross-contamination
```

### 2. **Ephemeral by Default**
```
Working memory is temporary
→ Privacy: no permanent trace
→ Performance: in-memory only
→ Sovereignty: user controls persistence
```

### 3. **Explicit Persistence**
```
Session → Dehydrate → Permanent Storage
→ User consent required
→ Audit trail when needed
→ Provenance tracking
```

---

## 🔍 Session Types

### Ephemeral Session
```rust
SessionType::Ephemeral
```

**Characteristics**:
- In-memory only
- Fast (no disk I/O)
- Discarded on expire
- Default for privacy

**Use Cases**:
- Interactive computation
- Temporary aggregations
- Exploratory analysis
- Privacy-sensitive operations

---

### Persistent Session
```rust
SessionType::Persistent
```

**Characteristics**:
- Optionally saved to storage
- Can be resumed
- Audit trail preserved
- Requires explicit consent

**Use Cases**:
- Compliance requirements
- Long-running workflows
- Multi-step processes
- Provenance tracking

---

## 📊 Performance Characteristics

| Operation | Time Complexity | Notes |
|-----------|----------------|-------|
| Create session | O(1) | Allocate session ID |
| Add vertex | O(1) | In-memory append |
| Resolve session | O(n log n) | Topological sort + Merkle tree |
| Dehydrate | O(f) | f = frontier size |
| Expire session | O(1) | Drop in-memory structures |

---

## 🎓 Advanced Concepts

### Session Isolation
```
┌─────────────┐  ┌─────────────┐
│  Session 1  │  │  Session 2  │
│    DAG A    │  │    DAG B    │
└─────────────┘  └─────────────┘
      ↓                  ↓
 No shared state!
```

**Benefits**:
- Multi-tenancy: Isolate users
- Parallelism: Independent sessions
- Security: No data leakage

---

### Slice Composition
```
Session:
  ├─ Slice 1 (from LoamSpine)
  ├─ Slice 2 (from LoamSpine)
  └─ Computed Vertices
       └─ Result (frontier)
```

**Workflow**:
1. Checkout slices (immutable snapshots)
2. Compute over slices (ephemeral DAG)
3. Dehydrate results (new permanent data)

---

### Dehydration Protocol
```
1. Resolve session (finalize DAG)
2. Compute Merkle root
3. Get frontier vertices
4. Commit to LoamSpine:
   - Frontier vertices
   - Merkle root
   - Provenance metadata
5. LoamSpine returns commit ID
6. Session records commit ID
```

**Guarantees**:
- Atomicity: All or nothing
- Integrity: Merkle root verifies
- Provenance: Full DAG traceable

---

## 🌟 Real-World Workflows

### Interactive Analysis
```
1. Create ephemeral session
2. Checkout data slices
3. Compute aggregations
4. Show results to user
5. Expire session (discard working memory)
```

### Compliance Audit
```
1. Create persistent session
2. Load audit data
3. Apply filters and transformations
4. Dehydrate results to permanent storage
5. Generate compliance report
6. Merkle root provides integrity proof
```

### Multi-Agent Collaboration
```
Agent 1:
  1. Create session
  2. Checkout shared data
  3. Compute partial result
  4. Dehydrate to LoamSpine

Agent 2:
  1. Create session
  2. Checkout Agent 1's result (slice)
  3. Compute final result
  4. Dehydrate to LoamSpine
```

---

## 🔗 Next Steps

**Ready to see advanced patterns?** Move to Level 5:
```bash
cd ../05-performance
cat README.md
```

Or skip to Level 6 for advanced patterns:
```bash
cd ../06-advanced-patterns
cat README.md
```

---

*"Ephemeral by default, persistent by consent."* 🔄

