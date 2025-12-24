# 🔐 Level 1: Hello rhizoCrypt

**Time**: 5 minutes  
**Skill**: Beginner  
**Prerequisites**: None

---

## 🎯 Goal

Your first rhizoCrypt session and vertex. Learn the basics of content-addressed events and session lifecycle.

---

## 📚 What You'll Learn

1. **Sessions** - Scoped DAG workspaces with lifecycle
2. **Vertices** - Content-addressed events (Blake3 hash)
3. **Content-Addressing** - Same content = same ID
4. **Session Lifecycle** - Create → Active → Resolved

---

## 🚀 Demos

### Demo 1: Your First Session (2 min)
```bash
./demo-first-session.sh
```

**What it does**:
- Creates a new session
- Shows session ID (UUID v7)
- Displays session state (Created → Active)
- Queries session info

**Key Concepts**:
- Sessions are scoped DAG workspaces
- Each session has a unique ID
- Sessions have lifecycle states

---

### Demo 2: Your First Vertex (2 min)
```bash
./demo-first-vertex.sh
```

**What it does**:
- Creates a vertex (event)
- Shows Blake3 content hash as ID
- Demonstrates content-addressing
- Adds vertex to session

**Key Concepts**:
- Vertices are content-addressed (Blake3)
- Same content = same vertex ID
- Vertices are immutable

---

### Demo 3: Query the DAG (1 min)
```bash
./demo-query-dag.sh
```

**What it does**:
- Queries vertices in session
- Shows DAG structure
- Displays frontier (DAG tips)
- Counts vertices

**Key Concepts**:
- DAG is queryable
- Frontier = vertices with no children
- Genesis = vertices with no parents

---

## 🎓 Key Takeaways

### Content-Addressing
```
Event Data: "User logged in"
     ↓ Blake3 Hash
Vertex ID: blake3::Hash("...")
```

**Why it matters**:
- Deduplication (same content = same ID)
- Integrity (any change = different ID)
- Verifiable (can prove inclusion)

### Session Lifecycle
```
Created → Active → Resolving → Resolved
                             ↘ RolledBack
```

**Why it matters**:
- Sessions are temporary (ephemeral by default)
- Only resolved sessions can be committed
- Rolled back sessions are discarded

---

## 💡 Try This

### Experiment 1: Same Content = Same ID
```bash
# Run demo twice - notice same vertex ID
./demo-first-vertex.sh
./demo-first-vertex.sh
```

### Experiment 2: Different Content = Different ID
```bash
# Modify the demo to use different content
# See how the vertex ID changes
```

---

## 🔗 Next Steps

**Ready for more?** Move to Level 2:
```bash
cd ../02-dag-engine
./demo-multi-parent.sh
```

---

*"Every journey begins with a single vertex."* 🔐

