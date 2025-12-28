# 🔐 rhizoCrypt - Quick Start

**Ephemeral DAG Engine** | **Pure Rust** | **Production Ready**

---

## 📊 Current Status

**Version**: 0.13.0  
**Grade**: ✅ A+ (96/100)  
**Tests**: 434/434 passing  
**Coverage**: 87%+  
**Status**: 🚀 **PRODUCTION READY**

---

## 🚀 Quick Start

### 1. Run the Showcase
```bash
cd showcase/00-local-primal
./RUN_ME_FIRST.sh
```

### 2. Test the System
```bash
cargo test --all-features
```

### 3. Start the Service
```bash
cargo run --bin rhizocrypt-service
```

---

## 📚 Documentation

- **[README.md](./README.md)** - Full documentation and features
- **[STATUS.md](./STATUS.md)** - Detailed system status and metrics
- **[FINAL_REPORT_DEC_28_2025.md](./FINAL_REPORT_DEC_28_2025.md)** - Latest audit report
- **[CHANGELOG.md](./CHANGELOG.md)** - Version history
- **[specs/](./specs/)** - Technical specifications

---

## 🏗️ Key Features

✅ **Lock-Free Concurrency** - DashMap-based, zero read contention  
✅ **Capability-Based** - Zero vendor lock-in, runtime discovery  
✅ **100% Safe Rust** - Zero unsafe code (forbidden at workspace level)  
✅ **87%+ Test Coverage** - Comprehensive unit, E2E, chaos, property tests  
✅ **Phase 1 Integration** - Verified with Songbird, BearDog, NestGate

---

## 🎯 What is rhizoCrypt?

rhizoCrypt is an **ephemeral DAG engine** - the working memory layer for the ecoPrimals ecosystem:

- **Ephemeral by Design** - Default to deletion, selective permanence
- **DAG-Based** - Directed Acyclic Graph for causal ordering
- **Merkle Proofs** - Cryptographic verification at every level
- **Primal Sovereignty** - User data ownership and consent
- **Human Dignity** - No surveillance, user control

---

## 📁 Project Structure

```
rhizoCrypt/
├── crates/
│   ├── rhizo-crypt-core/    # Core DAG engine (412 tests)
│   ├── rhizo-crypt-rpc/      # tarpc RPC layer (22 tests)
│   └── rhizocrypt-service/   # Service binary (11 integration tests)
├── showcase/                  # 41 working demos
├── specs/                     # Technical specifications
└── docs/                      # Additional documentation
```

---

## 🧪 Testing

```bash
# Run all tests
cargo test --all-features

# Run with coverage
cargo llvm-cov --all-features

# Run clippy (pedantic)
cargo clippy --all-features -- -D warnings

# Run benchmarks
cargo bench
```

---

## 🔗 Integration

rhizoCrypt integrates with Phase 1 primals:

- **Songbird** - Discovery and federation
- **BearDog** - Signing and HSM
- **NestGate** - Payload storage
- **LoamSpine** - Permanent storage (dehydration)

See `showcase/01-inter-primal-live/` for live demos.

---

## 📈 Recent Improvements (Dec 28, 2025)

✅ Service binary integration tests (11 new tests)  
✅ Phase 1 primalBins integration verified  
✅ tarpc adapter production-ready  
✅ Zero clippy warnings (was 4)  
✅ All mocks verified isolated to tests  

See [FINAL_REPORT_DEC_28_2025.md](./FINAL_REPORT_DEC_28_2025.md) for details.

---

## 🎓 Learn More

- Start with: `showcase/00-local-primal/RUN_ME_FIRST.sh`
- Read: [README.md](./README.md) for full documentation
- Review: [specs/](./specs/) for technical details
- Check: [STATUS.md](./STATUS.md) for current metrics

---

**🚀 Ready for production deployment, showcase demos, and real-world testing!**
