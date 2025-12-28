# ⚡ rhizoCrypt Showcase - QUICK START

**Goal**: Get up and running with rhizoCrypt in 5 minutes!  
**Date**: December 28, 2025

---

## 🎯 What You'll Learn

In the next 5 minutes you'll:
1. ✨ Start a rhizoCrypt session
2. 🌳 Add your first DAG vertex
3. 🔍 Query the DAG
4. 🎉 Understand what rhizoCrypt does!

---

## 📦 Prerequisites

### Required:
- Rust installed (`cargo --version`)
- rhizoCrypt built (`cargo build --release`)

### Optional (for ecosystem demos):
- Phase 1 primal binaries in `/path/to/ecoPrimals/primalBins/`
  - `songbird-cli` (discovery/rendezvous)
  - `beardog` (HSM/signing)
  - `nestgate` (storage/payloads)

---

## 🚀 5-Minute Quick Start

### Step 1: Run Your First Demo (2 minutes)

```bash
cd showcase/00-local-primal/01-hello-rhizocrypt/
./demo-first-session.sh
```

**What you'll see**:
- ✅ Session created with DID
- ✅ Genesis vertex added
- ✅ DAG initialized
- ✅ Session information displayed

### Step 2: Add Data to the DAG (1 minute)

```bash
./demo-first-vertex.sh
```

**What you'll see**:
- ✅ Data vertices added
- ✅ Parents linked correctly
- ✅ Content hashes computed
- ✅ Merkle tree built

### Step 3: Query the DAG (1 minute)

```bash
./demo-query-dag.sh
```

**What you'll see**:
- ✅ Frontier (latest vertices) displayed
- ✅ Topological ordering shown
- ✅ Ancestors traced back to genesis

### Step 4: Understand What Just Happened (1 minute)

You just used **rhizoCrypt** - an **ephemeral DAG engine** that:

🔹 **Ephemeral by Default** - Your data lives in working memory, not permanent storage  
🔹 **DAG-Based** - Events form a directed acyclic graph, not a linear list  
🔹 **Content-Addressed** - Every vertex has a cryptographic hash  
🔹 **Lock-Free** - Concurrent operations scale linearly  

---

## 🗺️ What's Next?

Now that you've seen the basics, explore by **progressive complexity**:

### 🌱 Level 1: Local Capabilities (30 min)
**Path**: `showcase/00-local-primal/`
- ✅ Hello rhizoCrypt (you just did this!)
- 🌳 DAG Engine (genesis, frontier, multi-parent)
- 🔐 Merkle Proofs (content addressing, tamper detection)
- 📦 Sessions (lifecycle, ephemeral vs persistent)
- 🔀 Slice Semantics (checkout modes)

### 🌍 Level 2: Ecosystem Integration (1 hour)
**Path**: `showcase/01-inter-primal-live/`
- 🎵 Songbird Discovery (find other primals)
- 🐻 BearDog Signing (HSM integration)
- 🏰 NestGate Storage (payload management)
- 🍄 ToadStool Compute (task execution)

### 🚀 Level 3: Complete Workflows (2 hours)
**Path**: `showcase/05-complete-workflows/`
- 📄 Document Collaboration
- 🤖 ML Pipeline
- 📦 Supply Chain Tracking

### 🏭 Level 4: Production Features (1 hour)
**Path**: `showcase/08-production-features/`
- 📊 Metrics & Monitoring
- 🏥 Health Checks
- 🔁 Error Recovery

---

## 🎓 Learning Paths

### For Developers:
1. Start with `00-local-primal/` (understand core concepts)
2. Explore `02-dag-engine/` (see DAG operations)
3. Try `06-advanced-patterns/` (real-world patterns)
4. Build with `01-inter-primal-live/` (ecosystem)

### For Architects:
1. Read `00_SHOWCASE_INDEX.md` (overview)
2. Review `06-real-world-scenarios/` (use cases)
3. Study `05-complete-workflows/` (end-to-end)
4. Explore `08-production-features/` (deployment)

### For Security Engineers:
1. Check `03-merkle-proofs/` (cryptographic foundations)
2. Review `02-beardog-signing/` (HSM integration)
3. Study `04-slice-semantics/` (provenance modes)
4. Examine `07-dehydration/` (permanent commit)

---

## 🔍 Key Concepts (1-Minute Primer)

### What is rhizoCrypt?
**rhizoCrypt** is an **ephemeral DAG engine** - a working memory layer for distributed systems.

Think of it as:
- **Git** - but for runtime state (not just code history)
- **Redis** - but with DAG structure (not key-value)
- **Kafka** - but with content addressing (not sequential topics)

### Why Ephemeral?
**Human Dignity**: Data should be ephemeral by default, permanent only by explicit choice.
- No surveillance by default
- User owns their data
- Selective permanence via dehydration

### Why DAG?
**Distributed Coordination**: Events don't have a global order, they have causal relationships.
- Concurrent operations without conflicts
- Cryptographic provenance built-in
- Efficient frontier-based queries

### Why Lock-Free?
**Performance**: DashMap-based concurrency gives 10-100x throughput vs coarse locks.
- Linear scaling with CPU cores
- No read contention
- Safe Rust guarantees

---

## 🛠️ Troubleshooting

### Demo fails with "Connection refused"
**Problem**: Primal service not running  
**Fix**: Check if service started: `ps aux | grep rhizocrypt-service`

### Demo fails with "Binary not found"
**Problem**: Primal bins not available  
**Fix**: Build Phase 1 primals or download from releases

### Demo produces unexpected output
**Problem**: State from previous run  
**Fix**: Clean logs/data: `rm -rf logs/ data/ *.db`

---

## 📚 References

- **Architecture**: `../../specs/ARCHITECTURE.md`
- **API Reference**: `../../specs/RHIZOCRYPT_SPECIFICATION.md`
- **Status**: `../../STATUS.md`
- **Full Index**: `00_SHOWCASE_INDEX.md`

---

## 🎉 You're Ready!

You now understand:
- ✅ What rhizoCrypt is (ephemeral DAG engine)
- ✅ Why it exists (human dignity + distributed coordination)
- ✅ How to run demos (progressive learning path)
- ✅ Where to go next (Level 1 → Level 4)

**Next Step**: Explore `showcase/00-local-primal/` to dive deeper! 🚀

---

*"Ephemeral by default, permanent by choice"* 💎

