# 🚀 Start Here - rhizoCrypt

Welcome to **rhizoCrypt** — the ephemeral DAG engine for the ecoPrimals ecosystem!

---

## ✅ Quick Status Check

**Current Status:** 🟢 **PRODUCTION READY**

- ✅ **403/403 tests passing** (100%)
- ✅ **85%+ code coverage**
- ✅ **Zero unsafe code**
- ✅ **Zero clippy warnings** (strict mode)
- ✅ **25 showcase demos** complete
- ✅ **Production ready** verified

**Last Verified:** December 26, 2025

---

## 🎯 What You Need to Know

### 1. **What is rhizoCrypt?**

rhizoCrypt is the **ephemeral working memory** for the ecoPrimals ecosystem:
- **DAG Engine** - Content-addressed directed acyclic graph
- **Sessions** - Scoped workflows with lifecycle
- **Merkle Proofs** - Cryptographic integrity
- **Dehydration** - Commit to permanent storage
- **Capability-Based** - Zero hardcoding, runtime discovery

### 2. **Philosophy**

> **"Ephemeral by default, persistent by consent."**

rhizoCrypt forgets by design. Only explicit dehydration creates permanence.

> **"Orchestrate, don't embed."**

rhizoCrypt coordinates other primals without coupling to them.

> **"Like an infant, discover at runtime."**

rhizoCrypt starts with zero hardcoded services. Everything discovered by capability.

---

## 🏃 Quick Start

### Option 1: Try the Showcase (Recommended)

```bash
cd showcase/00-local-primal
./RUN_ME_FIRST.sh
```

This runs a guided tour through rhizoCrypt capabilities.

### Option 2: Run Tests

```bash
cargo test --workspace
```

All 403 tests should pass in ~5 seconds.

### Option 3: Build and Run Service

```bash
# Build
cargo build --release --bin rhizocrypt-service

# Run
./target/release/rhizocrypt-service
```

Service starts on port 9400 by default.

---

## 📚 Documentation Structure

### Essential Reading
1. **[README.md](README.md)** - Main overview
2. **[STATUS.md](STATUS.md)** - Current project status  
3. **This file** - Getting started guide

### Session Reports (December 2025)
- **[SESSIONS_INDEX.md](SESSIONS_INDEX.md)** - Index of all session reports
- **[README_SESSION_DEC_26_2025.md](README_SESSION_DEC_26_2025.md)** - Latest session summary
- **[FINAL_STATUS_DEC_26_2025.md](FINAL_STATUS_DEC_26_2025.md)** - Complete final status

### Technical Specifications
- **[specs/RHIZOCRYPT_SPECIFICATION.md](specs/RHIZOCRYPT_SPECIFICATION.md)** - Core specification
- **[specs/ARCHITECTURE.md](specs/ARCHITECTURE.md)** - System architecture
- **[specs/DEHYDRATION_PROTOCOL.md](specs/DEHYDRATION_PROTOCOL.md)** - Commit protocol
- **[specs/SLICE_SEMANTICS.md](specs/SLICE_SEMANTICS.md)** - Checkout semantics

### Showcase & Demos
- **[showcase/README.md](showcase/README.md)** - Showcase overview
- **[showcase/00_SHOWCASE_INDEX.md](showcase/00_SHOWCASE_INDEX.md)** - Full demo index

---

## 🎪 Showcase Path

We recommend following the showcase in order:

### **Level 1: Hello rhizoCrypt** (15 min)
Learn the basics: sessions, vertices, DAG queries
```bash
cd showcase/00-local-primal/01-hello-rhizocrypt
./demo-first-session.sh
```

### **Level 2: DAG Engine** (20 min)
Understand the DAG: genesis, frontier, multi-parent
```bash
cd showcase/00-local-primal/02-dag-engine
./demo-genesis.sh
```

### **Level 3: Merkle Proofs** (20 min)
Cryptographic integrity and tamper detection
```bash
cd showcase/00-local-primal/03-merkle-proofs
./demo-content-addressing.sh
```

### **Level 4: Sessions** (30 min)
Session lifecycle, ephemeral vs persistent, slices, dehydration
```bash
cd showcase/00-local-primal/04-sessions
./demo-session-lifecycle.sh
```

### **Level 5: Performance** (20 min)
Latency, memory efficiency, scalability
```bash
cd showcase/00-local-primal/05-performance
./demo-latency.sh
```

### **Level 6: Advanced Patterns** (30 min)
Event sourcing, capability discovery
```bash
cd showcase/00-local-primal/06-advanced-patterns
./demo-event-sourcing.sh
```

### **Inter-Primal Integration** (60 min)
Real integration with Phase 1 primals (requires binaries)
```bash
cd showcase/01-inter-primal-live/01-songbird-discovery
./demo-infant-boot.sh
```

**Total Time:** ~3 hours for complete walkthrough

---

## 🔧 Development Workflow

### Build
```bash
# Debug
cargo build --workspace

# Release
cargo build --workspace --release
```

### Test
```bash
# All tests
cargo test --workspace

# Specific crate
cargo test -p rhizo-crypt-core

# With coverage
cargo llvm-cov --workspace --html
```

### Lint & Format
```bash
# Clippy (strict)
cargo clippy --workspace --all-targets -- -D warnings

# Format
cargo fmt --all

# Check format
cargo fmt --all -- --check
```

---

## 🎓 Learning Path

### For Users
1. Read [README.md](README.md)
2. Run showcase demos (Level 1-6)
3. Read [specs/RHIZOCRYPT_SPECIFICATION.md](specs/RHIZOCRYPT_SPECIFICATION.md)

### For Developers
1. Read [README.md](README.md) and [ARCHITECTURE.md](specs/ARCHITECTURE.md)
2. Run all tests: `cargo test --workspace`
3. Read the code starting with [crates/rhizo-crypt-core/src/lib.rs](crates/rhizo-crypt-core/src/lib.rs)
4. Try showcase demos to understand integration
5. Read session reports for context

### For Integrators
1. Read [README.md](README.md)
2. Review showcase inter-primal demos
3. Read [INTEGRATION_SPECIFICATION.md](specs/INTEGRATION_SPECIFICATION.md)
4. Study capability clients in [crates/rhizo-crypt-core/src/clients/capabilities/](crates/rhizo-crypt-core/src/clients/capabilities/)

---

## 🏗️ Project Structure

```
rhizoCrypt/
├── crates/
│   ├── rhizo-crypt-core/      # Core DAG engine (381 tests)
│   ├── rhizo-crypt-rpc/        # RPC layer (22 tests)
│   └── rhizocrypt-service/     # Standalone service
├── showcase/
│   ├── 00-local-primal/        # Local demos (9 demos)
│   └── 01-inter-primal-live/   # Integration demos (16 demos)
├── specs/                      # Technical specifications
├── target/                     # Build artifacts
├── README.md                   # Main documentation
├── START_HERE.md              # This file
├── STATUS.md                   # Project status
└── SESSIONS_INDEX.md           # Session reports index
```

---

## 🤝 Contributing

rhizoCrypt follows ecoPrimals principles:

1. **Primal Sovereignty** - Self-knowledge only
2. **Capability-Based** - Discover by capability, not vendor
3. **Ephemeral by Default** - Privacy through forgetting
4. **Cryptographic Provenance** - Merkle proofs everywhere
5. **Zero Unsafe** - 100% safe Rust
6. **Test Everything** - 85%+ coverage

---

## 📊 Current Metrics

| Metric | Value | Status |
|--------|-------|--------|
| Tests | 403/403 | ✅ 100% |
| Coverage | 85%+ | ✅ Excellent |
| Unsafe Code | 0 blocks | ✅ Perfect |
| Clippy | 0 warnings | ✅ Clean |
| File Size | 100% <1000 | ✅ Well-structured |
| Showcase | 25/25 demos | ✅ Complete |

---

## 🚀 Next Steps

### Immediate
1. Run the showcase: `cd showcase/00-local-primal && ./RUN_ME_FIRST.sh`
2. Read [README.md](README.md)
3. Explore the code

### Near-Term
1. Review session reports in [SESSIONS_INDEX.md](SESSIONS_INDEX.md)
2. Study specifications in [specs/](specs/)
3. Try inter-primal integration demos

### Long-Term
1. Deploy to development environment
2. Integrate with your services
3. Contribute improvements

---

## 🔗 Quick Links

- **Main Docs**: [README.md](README.md)
- **Status**: [STATUS.md](STATUS.md)
- **Sessions**: [SESSIONS_INDEX.md](SESSIONS_INDEX.md)
- **Showcase**: [showcase/README.md](showcase/README.md)
- **Specs**: [specs/](specs/)
- **Latest Report**: [FINAL_STATUS_DEC_26_2025.md](FINAL_STATUS_DEC_26_2025.md)

---

## ❓ FAQ

**Q: Is rhizoCrypt production ready?**  
A: Yes! All quality gates passing, 403 tests, zero unsafe code.

**Q: How do I integrate with my primal?**  
A: Implement a capability (signing, storage, etc.) and register with Songbird.

**Q: Where are the Phase 1 binaries?**  
A: Located at `../bins/` relative to this project (see showcase demos).

**Q: Can I use rhizoCrypt standalone?**  
A: Yes! Build `rhizocrypt-service` and run as standalone RPC service.

**Q: How do I run tests?**  
A: `cargo test --workspace` - all 403 tests should pass.

---

**Welcome to rhizoCrypt! Let's build something amazing together.** 🔐🌱

*Last Updated: December 26, 2025*
