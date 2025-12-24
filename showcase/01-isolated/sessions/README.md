# 🔐 rhizoCrypt Session Demos

**Phase 1: Isolated Instance — Session Lifecycle**

---

## 📋 What You'll Learn

- How to create and configure sessions
- Session types (General, Gaming, Experiment, Collaboration)
- Session lifecycle states (Created → Active → Resolving → Resolved)
- Session constraints and limits

---

## 🚀 Quick Start

```bash
# Run the session lifecycle demo
./demo-session-lifecycle.sh
```

---

## 📁 Available Demos

### 1. `demo-session-lifecycle.sh`
**Time:** 2 minutes  
**Complexity:** Beginner

Demonstrates the complete session lifecycle:
- Create a gaming session
- Append chess move events
- Query the DAG
- Compute Merkle root
- Discard session (ephemeral design)

### 2. `demo-session-types.sh` (Coming Soon)
**Time:** 5 minutes  
**Complexity:** Beginner

Shows all session types:
- General (default)
- Gaming (game-scoped)
- Experiment (protocol-scoped)
- Collaboration (workspace-scoped)

### 3. `demo-session-constraints.sh` (Coming Soon)
**Time:** 5 minutes  
**Complexity:** Intermediate

Demonstrates session constraints:
- Max vertices
- TTL (time-to-live)
- Custom metadata

---

## 📊 Session Lifecycle

```
┌─────────────┐
│   Created   │  ← SessionBuilder::new(...).build()
└──────┬──────┘
       │
       ▼
┌─────────────┐
│   Active    │  ← append_vertex(), query operations
└──────┬──────┘
       │
       ├──────────────┐
       ▼              ▼
┌─────────────┐  ┌─────────────┐
│  Resolved   │  │ Rolled Back │
│  (commit)   │  │  (discard)  │
└─────────────┘  └─────────────┘
```

---

## 🎮 Session Types

| Type | Purpose | Example |
|------|---------|---------|
| `General` | Default session | Generic event capture |
| `Gaming` | Game-scoped DAG | Chess match, game session |
| `Experiment` | Protocol-scoped | ML training run |
| `Collaboration` | Workspace-scoped | Document editing |

---

## 💡 Key Concepts

### Sessions are Scopes
Each session is an isolated DAG. Vertices in one session don't affect others.

### Ephemeral by Default
Sessions are designed to be forgotten. Only explicit commits (dehydration) persist.

### Content-Addressed Vertices
Every vertex gets a Blake3 hash as its ID. Same content = same ID.

### Multi-Parent DAG
Unlike git, vertices can have multiple parents, enabling complex relationships.

---

## 🔗 Next Steps

After understanding sessions:
1. Explore `../dag/` for DAG operations
2. Try `../merkle/` for proof generation
3. See `../slices/` for slice semantics

---

*rhizoCrypt: The memory that knows when to forget.*

