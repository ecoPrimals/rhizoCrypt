# 🔐 Level 0: Local Primal - START HERE

**Goal**: Master rhizoCrypt standalone capabilities with zero dependencies

**Time**: ~30 minutes  
**Prerequisites**: Rust installed  
**Dependencies**: None - runs completely local

---

## 🎯 What You'll Learn

By the end of Level 0, you'll understand:

✅ **Session Lifecycle** - Create → Grow → Resolve → Dehydrate  
✅ **Content-Addressed DAG** - Blake3 hashing, multi-parent vertices  
✅ **Merkle Proofs** - Cryptographic integrity and tamper detection  
✅ **Slice Semantics** - 6 checkout modes for flexible data access  
✅ **Performance** - Sub-microsecond operations  
✅ **Real-World Scenarios** - Tangible use cases

---

## 🚀 Quick Start

### Fastest Path (Guided Tour)
```bash
./RUN_ME_FIRST.sh
```
This runs all demos in sequence with explanations.

### Self-Guided Path
Run demos in order:
```bash
cd 01-hello-rhizocrypt && ./demo-first-session.sh
cd ../02-dag-engine && ./demo-genesis.sh
# ... and so on
```

---

## 📁 Demo Structure

```
00-local-primal/
├── 01-hello-rhizocrypt/       (3 demos, 10 min)
│   ├── demo-first-session.sh  ← Start here!
│   ├── demo-first-vertex.sh
│   └── demo-query-dag.sh
│
├── 02-dag-engine/             (4 demos, 15 min)
│   ├── demo-genesis.sh
│   ├── demo-multi-parent.sh
│   ├── demo-frontier.sh
│   └── demo-topological-sort.sh
│
├── 03-merkle-proofs/          (4 demos, 15 min)
│   ├── demo-content-addressing.sh
│   ├── demo-merkle-tree.sh
│   ├── demo-merkle-proof.sh
│   └── demo-tamper-detection.sh
│
├── 04-sessions/               (4 demos, 15 min)
│   ├── demo-session-lifecycle.sh
│   ├── demo-ephemeral-persistent.sh
│   ├── demo-slices.sh
│   └── demo-dehydration.sh
│
├── 04-slice-semantics/        (6 demos, 20 min) ⭐ NEW
│   ├── demo-copy-mode.sh
│   ├── demo-loan-mode.sh
│   ├── demo-consignment-mode.sh
│   ├── demo-escrow-mode.sh
│   ├── demo-mirror-mode.sh
│   └── demo-provenance-mode.sh
│
├── 05-performance/            (4 demos, 10 min)
│   ├── demo-latency.sh
│   ├── demo-memory.sh
│   ├── demo-throughput.sh
│   └── demo-scale.sh
│
├── 06-advanced-patterns/      (3 demos, 15 min)
│   ├── demo-event-sourcing.sh
│   ├── demo-capability-discovery.sh
│   └── demo-multi-session.sh
│
└── 06-real-world-scenarios/   (4 demos, 30 min) ⭐ NEW
    ├── demo-gaming-session.sh
    ├── demo-document-workflow.sh
    ├── demo-ml-pipeline.sh
    └── demo-supply-chain.sh
```

**Total**: 32 demos, ~30 minutes for guided tour, ~2 hours for deep dive

---

## 🎓 Progressive Learning

### Phase 1: Hello rhizoCrypt (10 min)
**Goal**: Get comfortable with basic operations

**Demos**:
1. `demo-first-session.sh` - Create your first session
2. `demo-first-vertex.sh` - Add vertices to the DAG
3. `demo-query-dag.sh` - Query and explore the graph

**Key Concepts**: Session, Vertex, DAG

**Success**: You can create a session and add vertices

---

### Phase 2: DAG Engine (15 min)
**Goal**: Understand directed acyclic graph operations

**Demos**:
1. `demo-genesis.sh` - Genesis vertices (roots)
2. `demo-multi-parent.sh` - Vertices with multiple parents
3. `demo-frontier.sh` - Latest vertices (tips of the DAG)
4. `demo-topological-sort.sh` - Ordering for computation

**Key Concepts**: Genesis, Multi-parent, Frontier, Topological sort

**Success**: You understand DAG structure and queries

---

### Phase 3: Merkle Proofs (15 min)
**Goal**: Master cryptographic integrity

**Demos**:
1. `demo-content-addressing.sh` - Blake3 hashing
2. `demo-merkle-tree.sh` - Build Merkle tree for session
3. `demo-merkle-proof.sh` - Generate and verify proofs
4. `demo-tamper-detection.sh` - Detect modifications

**Key Concepts**: Content-addressing, Merkle root, Proof, Verification

**Success**: You can prove integrity and detect tampering

---

### Phase 4: Sessions & Slices (35 min)
**Goal**: Understand lifecycle and data access patterns

**Demos (Sessions)**:
1. `demo-session-lifecycle.sh` - Full lifecycle
2. `demo-ephemeral-persistent.sh` - Philosophy of forgetting
3. `demo-slices.sh` - Basic slice semantics
4. `demo-dehydration.sh` - Commit to permanent storage

**Demos (Slice Semantics)**:
1. `demo-copy-mode.sh` - Full ownership transfer
2. `demo-loan-mode.sh` - Temporary access with return
3. `demo-consignment-mode.sh` - Conditional transfer
4. `demo-escrow-mode.sh` - Multi-party holding
5. `demo-mirror-mode.sh` - Synchronized copy
6. `demo-provenance-mode.sh` - Read-only with history

**Key Concepts**: Lifecycle, Dehydration, 6 slice modes

**Success**: You understand when to use each slice mode

---

### Phase 5: Performance (10 min)
**Goal**: See sub-microsecond operations

**Demos**:
1. `demo-latency.sh` - Operation latencies (ns/µs)
2. `demo-memory.sh` - Memory efficiency
3. `demo-throughput.sh` - Operations per second
4. `demo-scale.sh` - Large DAGs (1000+ vertices)

**Key Concepts**: Performance, Scalability, Efficiency

**Success**: You know rhizoCrypt can handle production loads

---

### Phase 6: Advanced & Real-World (45 min)
**Goal**: See rhizoCrypt in action

**Demos (Patterns)**:
1. `demo-event-sourcing.sh` - Event-driven architecture
2. `demo-capability-discovery.sh` - Runtime discovery
3. `demo-multi-session.sh` - Coordinating multiple sessions

**Demos (Real-World)**:
1. `demo-gaming-session.sh` - Gaming + ML training
2. `demo-document-workflow.sh` - Contract negotiation
3. `demo-ml-pipeline.sh` - Multi-agent ML workflow
4. `demo-supply-chain.sh` - Farm-to-table tracking

**Key Concepts**: Event sourcing, Discovery, Real scenarios

**Success**: You can map rhizoCrypt to your use case

---

## 🎯 Success Criteria

### You're ready for Level 1 when:
- [ ] You can create a session
- [ ] You can add vertices with parents
- [ ] You understand Merkle proofs
- [ ] You know which slice mode to use
- [ ] You've seen at least one real-world scenario
- [ ] You can explain dehydration to someone

---

## 💡 Tips for Success

### Start with RUN_ME_FIRST.sh
The guided tour is the fastest way to learn.

### Read the Output
Demos explain what's happening. Pay attention to the logs.

### Try Breaking Things
After `demo-merkle-tree.sh`, modify a vertex. See tamper detection work.

### Ask "Why This Way?"
Each design decision has a reason. Try to understand the "why."

### Map to Your Use Case
As you learn, think: "How would I use this in my app?"

---

## 📊 Expected Outcomes

By the end of Level 0, you should:

✅ **Understand**: All core rhizoCrypt concepts  
✅ **Experience**: Sub-microsecond performance  
✅ **See**: Real-world applications  
✅ **Know**: When to use each feature  
✅ **Feel**: Confident to integrate rhizoCrypt  

---

## 🔗 Quick Links

- **Next**: Level 1 (../01-inter-primal-live/00_START_HERE.md)
- **Showcase Overview**: ../00_START_HERE.md
- **Specs**: ../../specs/RHIZOCRYPT_SPECIFICATION.md
- **Architecture**: ../../specs/ARCHITECTURE.md

---

## ❓ FAQ

**Q: Do I need to run demos in order?**  
A: Recommended but not required. Concepts build on each other.

**Q: How long does RUN_ME_FIRST.sh take?**  
A: About 30 minutes with explanations. You can skip through faster.

**Q: Can I run specific demos?**  
A: Yes! Each demo is standalone. Just cd into the directory and run.

**Q: What if a demo fails?**  
A: Check that you've built rhizoCrypt: `cd ../.. && cargo build --workspace --release`

**Q: Do I need external services?**  
A: No! Level 0 is 100% local. No Songbird, no network, no dependencies.

---

## 🚀 Ready to Start?

```bash
# Guided tour (recommended)
./RUN_ME_FIRST.sh

# Or start with first demo
cd 01-hello-rhizocrypt
./demo-first-session.sh
```

---

**🔐 Let's master rhizoCrypt's local capabilities!** 🔐

*Questions? See individual demo READMEs or check ../../specs/*

*Last Updated: December 26, 2025*
