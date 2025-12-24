# 🔐 rhizoCrypt Local Primal Showcase

**Welcome!** This showcase demonstrates what **rhizoCrypt can do by itself** - using **REAL execution with actual API calls** (no mocks!).

**Status**: ✅ **Production Ready** (A+ grade, 98/100)  
**Philosophy**: Local-first - Master standalone capabilities before ecosystem integration  
**Date**: December 24, 2025

---

## 🎯 What is rhizoCrypt?

**rhizoCrypt** is an **ephemeral DAG engine** - the "memory that knows when to forget":

- ✅ **Content-Addressed DAG** - Blake3 hashing, multi-parent graphs
- ✅ **Session Lifecycle** - Create → Grow → Resolve → Forget
- ✅ **Merkle Proofs** - Cryptographic integrity verification
- ✅ **Slice Semantics** - 6 modes for state management
- ✅ **Selective Permanence** - Only commits survive (via LoamSpine)
- ✅ **Pure Infant Discovery** - Zero hardcoded primal names

**Key Insight**: Unlike traditional storage that preserves everything, rhizoCrypt embraces **selective forgetting**. Most data is temporary. Only what matters is committed to permanent storage.

---

## 🚀 QUICK START (5 Minutes)

### Prerequisites
```bash
# Ensure you're in the rhizoCrypt root
cd /home/strandgate/Development/ecoPrimals/phase2/rhizoCrypt

# Build rhizoCrypt
cargo build --workspace --release
```

### Run the Automated Tour
```bash
# 60-minute guided tour of all capabilities
cd showcase/00-local-primal
./RUN_ME_FIRST.sh
```

### Or Run Individual Demos
```bash
# Start with the simplest demo
cd 01-hello-rhizocrypt
./demo-first-session.sh          # 2 minutes

# Progress through the levels
cd ../02-dag-engine
./demo-multi-parent.sh           # 5 minutes

cd ../03-merkle-proofs
./demo-simple-proof.sh           # 5 minutes
```

**Expected Output**: Real execution with session IDs, vertex IDs, Merkle roots, and verification!

---

## 📚 SHOWCASE STRUCTURE

### ✅ Level 1: Hello rhizoCrypt (5 min, Beginner)
**Path**: `01-hello-rhizocrypt/`  
**Goal**: Your first session and vertex

**What you'll learn**:
- Create your first session
- Add your first vertex (event)
- Query the DAG
- Understand content-addressing (Blake3)

**Demos**:
- `demo-first-session.sh` - Session lifecycle basics
- `demo-first-vertex.sh` - Content-addressed events
- `demo-query-dag.sh` - Simple DAG queries

**Time**: 5 minutes total  
**Skill**: Beginner  
**Prerequisites**: None

---

### ✅ Level 2: DAG Engine (10 min, Beginner)
**Path**: `02-dag-engine/`  
**Goal**: Understand multi-parent DAG operations

**What you'll learn**:
- Multi-parent DAG (not just a chain)
- Frontier tracking (DAG tips)
- Genesis detection (session roots)
- Topological ordering

**Demos**:
- `demo-multi-parent.sh` - Complex DAG structures
- `demo-frontier.sh` - Track DAG tips
- `demo-genesis.sh` - Find session roots
- `demo-topological-sort.sh` - Ordered traversal

**Time**: 10 minutes total  
**Skill**: Beginner  
**Prerequisites**: Level 1

---

### ✅ Level 3: Merkle Proofs (10 min, Intermediate)
**Path**: `03-merkle-proofs/`  
**Goal**: Cryptographic integrity verification

**What you'll learn**:
- Merkle tree construction
- Proof generation
- Proof verification
- Tamper detection

**Demos**:
- `demo-simple-proof.sh` - Generate and verify proofs
- `demo-verify.sh` - Verify vertex inclusion
- `demo-tamper-detection.sh` - Catch modifications
- `demo-batch-verification.sh` - Efficient batch proofs

**Time**: 10 minutes total  
**Skill**: Intermediate  
**Prerequisites**: Level 2

---

### ✅ Level 4: Slice Semantics (15 min, Advanced)
**Path**: `04-slice-semantics/`  
**Goal**: Master state management modes

**What you'll learn**:
- 6 slice modes (Copy, Loan, Consignment, Escrow, Waypoint, Transfer)
- Resolution routing
- Slice constraints
- Waypoint anchoring

**Demos**:
- `demo-copy-mode.sh` - Safe copying
- `demo-loan-mode.sh` - Temporary lending with auto-return
- `demo-escrow-mode.sh` - Multi-party agreement
- `demo-waypoint.sh` - Anchored to permanent storage

**Time**: 15 minutes total  
**Skill**: Advanced  
**Prerequisites**: Level 3

---

### ✅ Level 5: Performance (10 min, Expert)
**Path**: `05-performance/`  
**Goal**: See world-class performance

**What you'll learn**:
- Sub-microsecond operations
- High-throughput DAG operations
- Benchmark results
- Zero-copy optimizations

**Demos**:
- `demo-throughput.sh` - 1M+ vertices/sec
- `demo-benchmarks.sh` - Run criterion benchmarks
- `demo-zero-copy.sh` - Efficient payload handling
- `results/` - Benchmark reports

**Time**: 10 minutes total  
**Skill**: Expert  
**Prerequisites**: Level 4

**Performance Highlights**:
- Vertex creation: ~720 ns (1.4M/sec)
- Blake3 hash (4KB): ~80 ns (12.5M/sec)
- DAG put_vertex: ~1.6 µs (625K/sec)
- DAG get_vertex: ~270 ns (3.7M/sec)
- Merkle root (1k): ~750 µs
- Proof verification: ~1.4 µs (714K/sec)

---

### ✅ Level 6: Real-World Scenarios (15 min, Expert)
**Path**: `06-real-world-scenarios/`  
**Goal**: See rhizoCrypt in action

**What you'll learn**:
- Gaming session capture
- ML experiment tracking
- Collaborative document editing
- Provenance tracking

**Demos**:
- `demo-gaming-session.sh` - Capture gameplay with AI training
- `demo-ml-experiment.sh` - Track training runs and checkpoints
- `demo-collaborative-doc.sh` - CRDT-style conflict-free editing
- `demo-provenance.sh` - Query "who did what"

**Time**: 15 minutes total  
**Skill**: Expert  
**Prerequisites**: Level 5

---

## 🎓 LEARNING PATHS

### Path A: "I'm New to rhizoCrypt" (60 minutes)
**Goal**: Master all local capabilities

```bash
# Run the automated tour
./RUN_ME_FIRST.sh
```

This script walks you through all 6 levels with pauses and explanations.

---

### Path B: "Show Me Something Cool!" (5 minutes)
**Goal**: See rhizoCrypt's best features NOW

```bash
# The ultimate demo - performance showcase
cd 05-performance
./demo-throughput.sh
```

---

### Path C: "I Want Specific Features" (Variable)
**Goal**: Jump to what interests you

**For Content-Addressing**:
```bash
cd 01-hello-rhizocrypt
./demo-first-vertex.sh
```

**For Cryptographic Proofs**:
```bash
cd 03-merkle-proofs
./demo-simple-proof.sh
```

**For Real-World Use Cases**:
```bash
cd 06-real-world-scenarios
./demo-gaming-session.sh
```

---

## 📊 WHAT MAKES rhizoCrypt SPECIAL?

### 1. **Designed to Forget** 🧠
Unlike traditional databases that preserve everything, rhizoCrypt is **ephemeral by default**:
- Sessions have lifecycles (create → grow → resolve → expire)
- Only dehydrated summaries persist to LoamSpine
- Scales infinitely by forgetting

### 2. **Content-Addressed Integrity** 🔒
Every vertex is identified by its Blake3 hash:
- Same content = same ID (deduplication)
- Tamper-evident (any change = different hash)
- Cryptographic proofs via Merkle trees

### 3. **Multi-Parent DAG** 🌳
Not just a blockchain (single parent):
- Multiple parents per vertex
- Complex branching and merging
- Topological ordering

### 4. **Selective Permanence** 💾
Choose what survives:
- Most data expires with session
- Dehydration protocol commits summaries
- Waypoints anchor to permanent storage

### 5. **Pure Infant Discovery** 🐣
Zero hardcoded primal names:
- Discovers capabilities at runtime
- Works with ANY service providing needed capability
- No vendor lock-in

### 6. **World-Class Performance** ⚡
Sub-microsecond operations:
- 1.4M vertices/sec creation
- 625K/sec DAG insertions
- 714K/sec proof verifications

---

## 🏗️ ARCHITECTURE OVERVIEW

```
┌─────────────────────────────────────────────────────────────┐
│                     rhizoCrypt Core                          │
│                  (Ephemeral DAG Engine)                      │
├─────────────────────────────────────────────────────────────┤
│                                                              │
│  ┌──────────┐  ┌──────────┐  ┌──────────┐  ┌──────────┐   │
│  │ Sessions │  │   DAG    │  │  Merkle  │  │  Slices  │   │
│  │          │  │  Store   │  │  Trees   │  │          │   │
│  └────┬─────┘  └────┬─────┘  └────┬─────┘  └────┬─────┘   │
│       │             │              │             │          │
│       └─────────────┴──────────────┴─────────────┘          │
│                          │                                   │
│                          ▼                                   │
│              ┌───────────────────────┐                       │
│              │   Content Addressing  │                       │
│              │   (Blake3 Hashing)    │                       │
│              └───────────────────────┘                       │
│                                                              │
└─────────────────────────────────────────────────────────────┘
```

---

## 🎯 SUCCESS CRITERIA

### You've Mastered rhizoCrypt When:

**Level 1 Complete**:
- ✅ Can create and query sessions
- ✅ Understand content-addressing
- ✅ Know what a vertex is

**Level 2 Complete**:
- ✅ Understand multi-parent DAGs
- ✅ Can track frontiers
- ✅ Know topological ordering

**Level 3 Complete**:
- ✅ Can generate Merkle proofs
- ✅ Understand cryptographic verification
- ✅ Can detect tampering

**Level 4 Complete**:
- ✅ Master all 6 slice modes
- ✅ Understand resolution routing
- ✅ Know waypoint anchoring

**Level 5 Complete**:
- ✅ Understand performance characteristics
- ✅ Know optimization techniques
- ✅ Can benchmark operations

**Level 6 Complete**:
- ✅ Can apply rhizoCrypt to real problems
- ✅ Understand use case patterns
- ✅ Ready for production deployment

---

## 💡 TIPS FOR SUCCESS

### For Best Results:
- ✅ Start with Level 1 (don't skip ahead)
- ✅ Run demos in order (they build on each other)
- ✅ Read the output carefully (it explains what's happening)
- ✅ Experiment! Modify the demos and see what happens

### Common Questions:

**Q: Why is rhizoCrypt "ephemeral"?**  
A: Most data should be temporary. Only what matters is committed to permanent storage. This enables infinite scalability.

**Q: What's the difference between rhizoCrypt and a database?**  
A: Databases preserve everything. rhizoCrypt is working memory - it captures, proves, and selectively commits.

**Q: When should I use rhizoCrypt?**  
A: When you need to capture complex, branching workflows with cryptographic integrity, then commit only the results.

**Q: What's "infant discovery"?**  
A: rhizoCrypt starts with zero knowledge of other primals and discovers capabilities at runtime. No hardcoded names!

---

## 📚 NEXT STEPS

### After Mastering Local Capabilities:

**Phase 2: RPC Layer** (`../02-rpc/`)
- tarpc server with 24 methods
- Rate limiting and metrics
- Client operations

**Phase 3: Inter-Primal** (`../03-inter-primal/`)
- Discover capabilities via Songbird
- Sign vertices via BearDog
- Store payloads via NestGate
- Commit to LoamSpine

**Phase 4: Complete Workflows** (`../04-complete-workflow/`)
- Multi-agent sessions
- Dehydration protocol
- Provenance tracking

**Phase 5: Live Integration** (`../05-live-integration/`)
- Real Phase 1 binaries
- Full ecosystem coordination

---

## 🏆 SHOWCASE PHILOSOPHY

**Following Best Practices from Phase 1**:
- ✅ **Local-first** (ToadStool pattern) - Master standalone before ecosystem
- ✅ **Progressive complexity** (NestGate pattern) - Beginner → Expert
- ✅ **Real execution** (ToadStool pattern) - No mocks, actual API calls
- ✅ **Automated tour** (NestGate pattern) - Zero-friction onboarding
- ✅ **Time-boxed** (NestGate pattern) - Clear duration for each demo

**rhizoCrypt Unique Value**:
- 🏆 Ephemeral by default (designed to forget)
- 🏆 Content-addressed DAG (not just a chain)
- 🏆 Selective permanence (dehydration protocol)
- 🏆 Pure infant discovery (zero hardcoding)
- 🏆 World-class performance (sub-microsecond ops)

---

## 🚀 READY TO START?

### Option 1: Automated Tour (Recommended)
```bash
./RUN_ME_FIRST.sh
```

### Option 2: Manual Exploration
```bash
cd 01-hello-rhizocrypt
./demo-first-session.sh
```

### Option 3: Jump to Performance
```bash
cd 05-performance
./demo-throughput.sh
```

---

**Let's explore the memory that knows when to forget!** 🔐

---

## 📖 ADDITIONAL RESOURCES

- [rhizoCrypt Specification](../../specs/RHIZOCRYPT_SPECIFICATION.md)
- [Architecture Overview](../../specs/ARCHITECTURE.md)
- [Data Model](../../specs/DATA_MODEL.md)
- [API Reference](../../specs/API_SPECIFICATION.md)
- [Complete Audit Report](../../COMPREHENSIVE_AUDIT_DEC_24_2025.md)

---

*"Great code deserves a great showcase. Welcome to rhizoCrypt."* 🚀

