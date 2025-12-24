# 🔐 rhizoCrypt - Root Documentation Index

**Last Updated**: December 24, 2025  
**Version**: 0.1.0  
**Status**: Production Ready

---

## 📚 Essential Documents

### Getting Started
1. **[START_HERE.md](START_HERE.md)** - New user entry point
   - Quick start guide
   - Architecture overview
   - Key concepts
   - Project structure

2. **[README.md](README.md)** - Project overview
   - What is rhizoCrypt?
   - Features & capabilities
   - Installation
   - Quick examples

3. **[STATUS.md](STATUS.md)** - Current project status
   - Build status
   - Quality metrics
   - Implementation completeness
   - Test coverage

---

## 🎯 Core Documentation

### Architecture & Specifications
- **[specs/](specs/)** - Complete specifications
  - `00_SPECIFICATIONS_INDEX.md` - Spec index
  - `RHIZOCRYPT_SPECIFICATION.md` - Master spec
  - `ARCHITECTURE.md` - System architecture
  - `DATA_MODEL.md` - Data structures
  - `DEHYDRATION_PROTOCOL.md` - Persistence protocol
  - `SLICE_SEMANTICS.md` - Slice operations
  - `API_SPECIFICATION.md` - API reference
  - `INTEGRATION_SPECIFICATION.md` - Integration guide
  - `STORAGE_BACKENDS.md` - Storage options

### Configuration
- **[ENV_VARS.md](ENV_VARS.md)** - Environment variables
  - Capability-based configuration
  - Legacy variable migration
  - Configuration examples

### Development
- **[CHANGELOG.md](CHANGELOG.md)** - Version history
  - Release notes
  - Breaking changes
  - Migration guides

- **[WHATS_NEXT.md](WHATS_NEXT.md)** - Roadmap
  - Completed features
  - Future enhancements
  - Optional improvements

---

## 🚀 Showcase & Examples

### Interactive Showcase
- **[showcase/](showcase/)** - Comprehensive showcase
  - `README.md` - Showcase overview
  - `00-local-primal/` - **22 local demos** (Sprint 1 ✅)
  - `02-rpc-layer/` - **5 RPC examples** (Sprint 2 ✅)
  - `03-inter-primal/` - Inter-primal coordination (Sprint 3 🚧)
  - `04-complete-workflow/` - Real-world scenarios (Sprint 4 ⏳)

### Quick Start
- **[showcase/00-local-primal/RUN_ME_FIRST.sh](showcase/00-local-primal/RUN_ME_FIRST.sh)**
  - Automated 60-minute tour
  - Progressive learning path
  - All 22 demos in sequence

---

## 📊 Quality & Metrics

### Current Metrics (Dec 24, 2025)
- ✅ **0 TODOs** in production code
- ✅ **0 unsafe blocks** (100% safe Rust)
- ✅ **0 unwraps** in production (proper error handling)
- ✅ **0 hardcoding** (capability-based, primal-agnostic)
- ✅ **85.22% test coverage** (target: 40%, achieved: 85%+)
- ✅ **Max file size: 925 lines** (target: 1000)
- ✅ **All linting passed** (Clippy pedantic)
- ✅ **All formatting passed** (rustfmt)

### Quality Gates
All quality gates **PASSED** ✅

---

## 🗂️ Document Organization

### Root Directory (Current)
```
rhizoCrypt/
├── START_HERE.md              ← Start here!
├── README.md                  ← Project overview
├── STATUS.md                  ← Current status
├── CHANGELOG.md               ← Version history
├── ENV_VARS.md                ← Configuration guide
├── WHATS_NEXT.md              ← Roadmap
├── ROOT_DOCS_INDEX.md         ← This file
│
├── specs/                     ← Complete specifications
├── showcase/                  ← Interactive demos
├── crates/                    ← Source code
└── docs/                      ← Additional documentation
```

### Documentation Archive
```
docs/
├── sessions/                  ← Session-specific reports
│   └── 2025-12-24/           ← Dec 24 session
│       ├── COMPREHENSIVE_AUDIT_DEC_24_2025.md
│       ├── SPRINT_1_COMPLETE_DEC_24_2025.md
│       ├── SPRINT_2_COMPLETE_DEC_24_2025.md
│       └── ... (11 session documents)
│
└── archive/                   ← Historical documents
    └── ... (older documents)
```

---

## 🎓 Learning Paths

### For New Users
1. Read `START_HERE.md`
2. Review `README.md`
3. Run `showcase/00-local-primal/RUN_ME_FIRST.sh`
4. Explore `showcase/00-local-primal/` demos
5. Check `STATUS.md` for current state

### For Developers
1. Read `specs/00_SPECIFICATIONS_INDEX.md`
2. Review `specs/ARCHITECTURE.md`
3. Study `specs/API_SPECIFICATION.md`
4. Explore `crates/rhizo-crypt-core/src/`
5. Run tests: `cargo test --workspace`

### For Integrators
1. Read `specs/INTEGRATION_SPECIFICATION.md`
2. Review `ENV_VARS.md` for configuration
3. Explore `showcase/02-rpc-layer/` examples
4. Study `showcase/03-inter-primal/` (when complete)
5. Check `crates/rhizo-crypt-rpc/` for RPC API

---

## 📖 Quick Reference

### Build & Test
```bash
# Build
cargo build --workspace

# Test
cargo test --workspace

# Coverage
cargo llvm-cov --workspace --html

# Lint
cargo clippy --workspace -- -D warnings

# Format
cargo fmt --all -- --check
```

### Run Showcase
```bash
# Automated tour (60 minutes)
cd showcase/00-local-primal
./RUN_ME_FIRST.sh

# Individual demos
cd showcase/00-local-primal/01-hello-rhizocrypt
./demo-first-session.sh
```

### RPC Examples
```bash
# Start server (terminal 1)
cd crates/rhizo-crypt-rpc
cargo run --example rpc-server

# Run client (terminal 2)
cd showcase/02-rpc-layer/examples
cargo run --bin basic-client
```

---

## 🔗 External Resources

### Crate Documentation
- **rhizo-crypt-core**: Core DAG engine
  - `crates/rhizo-crypt-core/README.md`
  - `cargo doc --open -p rhizo-crypt-core`

- **rhizo-crypt-rpc**: RPC layer
  - `crates/rhizo-crypt-rpc/README.md`
  - `cargo doc --open -p rhizo-crypt-rpc`

### Ecosystem Integration
- **Phase 1 Primals**: `../../phase1/`
  - BearDog (crypto:signing)
  - NestGate (storage:payload)
  - LoamSpine (storage:permanent)
  - Songbird (discovery:registry)
  - ToadStool (compute:orchestration)

---

## 📝 Document Maintenance

### When to Update
- **START_HERE.md**: On major architecture changes
- **README.md**: On feature additions
- **STATUS.md**: Weekly or on milestone completion
- **CHANGELOG.md**: On every release
- **ENV_VARS.md**: On configuration changes
- **WHATS_NEXT.md**: Monthly or on roadmap updates

### Document Lifecycle
1. **Active**: Current, frequently referenced
2. **Archived**: Historical, moved to `docs/sessions/`
3. **Deprecated**: Outdated, moved to `docs/archive/`

---

## 🎉 Recent Achievements (Dec 24, 2025)

### Completed
- ✅ **Sprint 1**: 22 local demos (100%)
- ✅ **Sprint 2**: 5 RPC examples (100%)
- ✅ **Comprehensive Audit**: All quality gates passed
- ✅ **Zero Technical Debt**: No TODOs, unsafe, unwraps, hardcoding

### In Progress
- 🚧 **Sprint 3**: Inter-primal coordination
- 🚧 **Sprint 4**: Real-world scenarios

### Metrics
- **27 executable examples** created
- **~5,700 lines** of demo code
- **~3,500 lines** of documentation
- **85.22% test coverage**

---

## 🌟 Key Principles

### Technical
- ✅ **Pure Rust** - No unsafe code
- ✅ **Type Safety** - Compile-time checks
- ✅ **Zero Debt** - No TODOs, unwraps, hardcoding
- ✅ **Idiomatic** - Rust best practices

### Architectural
- ✅ **Content-Addressed** - Blake3 hashing
- ✅ **Multi-Parent DAG** - Not just a chain
- ✅ **Merkle Integrity** - Cryptographic proofs
- ✅ **Capability-Based** - Primal-agnostic

### Sovereignty
- ✅ **User Control** - Explicit consent
- ✅ **Privacy-First** - Ephemeral by default
- ✅ **No Surveillance** - No tracking
- ✅ **Data Ownership** - User sovereignty

### Human Dignity
- ✅ **Transparent** - Clear operations
- ✅ **Empowering** - User agency
- ✅ **Respectful** - Dignity-preserving
- ✅ **Ethical** - Human-centered design

---

## 📞 Getting Help

### Documentation
- Start with `START_HERE.md`
- Check `STATUS.md` for current state
- Review `specs/` for detailed specifications
- Explore `showcase/` for examples

### Code
- Read inline documentation
- Run `cargo doc --open`
- Study tests in `crates/*/tests/`
- Review examples in `showcase/`

### Community
- Check GitHub issues
- Review CHANGELOG for known issues
- Consult ecosystem documentation

---

*"Clean docs, clear path."* 📚✨

---

**End of Root Documentation Index**

