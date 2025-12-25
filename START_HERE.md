# 🚀 Start Here: rhizoCrypt

**Welcome to rhizoCrypt** — the ephemeral DAG engine for the ecoPrimals ecosystem!

---

## 🎯 What is rhizoCrypt?

rhizoCrypt is **working memory** for sovereign data:

- **Ephemeral by default** — Forgets unless you explicitly commit
- **DAG-based** — Directed acyclic graph for causality
- **Cryptographically proven** — Merkle trees for integrity
- **Pure Rust** — Zero C/C++ dependencies, zero unsafe code

Think of it as:
- **Git** for events (not files)
- **Redis** for graphs (not key-value)
- **Ephemeral** by design (not permanent)

---

## 🏃 Quick Start (5 minutes)

### 1. Build

```bash
# Clone and build
cargo build --workspace

# Run tests
cargo test --workspace
```

### 2. Try Your First Demo

```bash
cd showcase/00-local-primal/01-hello-rhizocrypt
./demo-first-session.sh
```

This shows you:
- Creating a session
- Adding vertices to the DAG
- Querying the graph
- Computing Merkle proofs

### 3. Explore More

```bash
# Session lifecycle
cd ../04-sessions
./demo-session-lifecycle.sh

# Multi-agent collaboration
cd ../../01-inter-primal-live/02-beardog-signing
./demo-multi-agent.sh
```

---

## 📚 Learning Path

### Level 1: Basics (30 minutes)

**Goal**: Understand core concepts

1. **Hello rhizoCrypt** (3 demos)
   ```bash
   cd showcase/00-local-primal/01-hello-rhizocrypt
   ./demo-first-session.sh
   ./demo-first-vertex.sh
   ./demo-query-dag.sh
   ```

2. **Read**: [specs/RHIZOCRYPT_SPECIFICATION.md](specs/RHIZOCRYPT_SPECIFICATION.md)

**You'll learn**:
- What is a session?
- What is a vertex?
- How does the DAG work?

---

### Level 2: DAG Engine (45 minutes)

**Goal**: Master DAG operations

1. **DAG Engine** (4 demos)
   ```bash
   cd showcase/00-local-primal/02-dag-engine
   ./demo-genesis.sh        # Roots of the DAG
   ./demo-frontier.sh       # Tips of the DAG
   ./demo-multi-parent.sh   # Branching and merging
   ./demo-topological-sort.sh  # Causality ordering
   ```

2. **Read**: [specs/ARCHITECTURE.md](specs/ARCHITECTURE.md)

**You'll learn**:
- Genesis vertices (roots)
- Frontier vertices (tips)
- Parent-child relationships
- Topological ordering

---

### Level 3: Cryptography (45 minutes)

**Goal**: Understand integrity proofs

1. **Merkle Proofs** (4 demos)
   ```bash
   cd showcase/00-local-primal/03-merkle-proofs
   ./demo-content-addressing.sh  # Blake3 hashing
   ./demo-merkle-tree.sh         # Tree construction
   ./demo-merkle-proof.sh        # Proof generation
   ./demo-tamper-detection.sh    # Integrity verification
   ```

2. **Read**: [specs/DEHYDRATION_PROTOCOL.md](specs/DEHYDRATION_PROTOCOL.md)

**You'll learn**:
- Content addressing (Blake3)
- Merkle tree construction
- Proof generation and verification
- Tamper detection

---

### Level 4: Sessions (60 minutes)

**Goal**: Master session lifecycle

1. **Sessions** (4 demos)
   ```bash
   cd showcase/00-local-primal/04-sessions
   ./demo-session-lifecycle.sh      # Full lifecycle
   ./demo-ephemeral-persistent.sh   # Session types
   ./demo-slices.sh                 # Checkout semantics
   ./demo-dehydration.sh            # Commit protocol
   ```

2. **Read**: [specs/SLICE_SEMANTICS.md](specs/SLICE_SEMANTICS.md)

**You'll learn**:
- Session lifecycle (Create → Grow → Resolve → Expire)
- Ephemeral vs persistent sessions
- Slice checkout from permanent storage
- Dehydration (commit to LoamSpine)

---

### Level 5: Integration (90 minutes)

**Goal**: Connect with Phase 1 primals

1. **Songbird (Discovery)**
   ```bash
   cd showcase/01-inter-primal-live/01-songbird-discovery
   ./start-songbird.sh
   ./demo-register.sh
   ./demo-discover.sh
   ./demo-health.sh
   ```

2. **BearDog (Signing)**
   ```bash
   cd ../02-beardog-signing
   ./demo-hsm-discover.sh
   ./demo-generate-keys.sh
   ./demo-sign-vertex.sh
   ./demo-multi-agent.sh
   ```

**You'll learn**:
- Capability-based discovery
- Service registration
- DID management
- Cryptographic signatures
- Multi-agent collaboration

---

## 🛠️ Development

### Running Tests

```bash
# All tests (228)
cargo test --workspace

# Specific crate
cargo test -p rhizo-crypt-core

# With coverage
cargo llvm-cov --workspace --summary-only

# Watch mode
cargo watch -x test
```

### Code Quality

```bash
# Lint
cargo clippy --workspace -- -D warnings

# Format
cargo fmt --all

# Check formatting
cargo fmt --check
```

### Benchmarks

```bash
cargo bench -p rhizo-crypt-core
```

---

## 📖 Documentation

### Essential Reading

1. **[README.md](README.md)** - Project overview
2. **[STATUS.md](STATUS.md)** - Current status and metrics
3. **[specs/RHIZOCRYPT_SPECIFICATION.md](specs/RHIZOCRYPT_SPECIFICATION.md)** - Core specification

### Specifications

- [ARCHITECTURE.md](specs/ARCHITECTURE.md) - System design
- [DATA_MODEL.md](specs/DATA_MODEL.md) - Data structures
- [DEHYDRATION_PROTOCOL.md](specs/DEHYDRATION_PROTOCOL.md) - Commit protocol
- [SLICE_SEMANTICS.md](specs/SLICE_SEMANTICS.md) - Checkout semantics
- [STORAGE_BACKENDS.md](specs/STORAGE_BACKENDS.md) - Storage options

### Showcase

- [showcase/README.md](showcase/README.md) - Demo overview
- [showcase/00_SHOWCASE_INDEX.md](showcase/00_SHOWCASE_INDEX.md) - Full index

---

## 🎯 Common Tasks

### Create a Session

```rust
use rhizo_crypt_core::*;

let config = RhizoCryptConfig::default();
let mut primal = RhizoCrypt::new(config);
primal.start().await?;

let session = SessionBuilder::new(SessionType::General)
    .with_name("my-session")
    .build();

let session_id = primal.create_session(session).await?;
```

### Add a Vertex

```rust
let vertex = VertexBuilder::new(EventType::DataCreate { 
    schema: Some("my-data".to_string()) 
})
    .with_payload(PayloadRef::from_bytes(b"Hello!"))
    .with_metadata("author", "alice")
    .build();

let vertex_id = primal.append_vertex(session_id, vertex).await?;
```

### Compute Merkle Root

```rust
let merkle_root = primal.compute_merkle_root(session_id).await?;
println!("Merkle root: {}", merkle_root);
```

### Dehydrate Session

```rust
let merkle_root = primal.dehydrate(session_id).await?;
// This would send the summary to LoamSpine in production
```

---

## 🤔 FAQ

### What makes rhizoCrypt "ephemeral"?

Sessions are forgotten by default. Only explicit dehydration creates permanence. This is the "Philosophy of Forgetting" — privacy through ephemerality.

### Why DAG instead of blockchain?

DAGs allow:
- Concurrent operations (no single chain)
- Flexible causality (multiple parents)
- Efficient merging (no conflicts)
- Local-first operation (no consensus)

### What is "dehydration"?

Dehydration is committing ephemeral DAG results to permanent storage (LoamSpine). Think of it as:
- Git commit (but for events)
- Database transaction (but for graphs)
- Checkpoint (but cryptographically proven)

### How does it integrate with other primals?

rhizoCrypt uses **capability-based discovery**:
1. Register with Songbird (discovery service)
2. Query for capabilities (e.g., "signing", "storage")
3. Connect to discovered services
4. No hardcoded addresses!

### Is it production-ready?

**Yes!** 
- ✅ 228 tests passing
- ✅ 64% test coverage
- ✅ Zero clippy warnings
- ✅ Zero unsafe code
- ✅ 100% Pure Rust
- ✅ 25 working demos

---

## 🚀 Next Steps

### For Users

1. Run the showcase demos
2. Read the specifications
3. Try the integration demos
4. Explore the codebase

### For Developers

1. Read [STATUS.md](STATUS.md) for current priorities
2. Check [showcase/01-inter-primal-live/GAPS_DISCOVERED.md](showcase/01-inter-primal-live/GAPS_DISCOVERED.md) for known issues
3. Pick a task from the roadmap
4. Submit a PR!

### For Integrators

1. Review [specs/INTEGRATION_SPECIFICATION.md](specs/INTEGRATION_SPECIFICATION.md)
2. Try the inter-primal demos
3. Check the RPC API documentation
4. Build your integration!

---

## 📞 Getting Help

- **Documentation**: See `specs/` directory
- **Examples**: See `showcase/` directory
- **Issues**: Document in `GAPS_DISCOVERED.md`
- **Code**: Read the well-commented source

---

**Welcome to rhizoCrypt!** 🔐🌱

*Ephemeral working memory for the sovereign data ecosystem*
