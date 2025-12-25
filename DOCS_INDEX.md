# 📚 rhizoCrypt Documentation Index

**Last Updated**: December 25, 2025

---

## 🚀 Getting Started

### Essential Documents

| Document | Purpose | Audience |
|----------|---------|----------|
| **[README.md](README.md)** | Project overview, quick start | Everyone |
| **[START_HERE.md](START_HERE.md)** | New user onboarding, learning path | New users |
| **[STATUS.md](STATUS.md)** | Current status, metrics, priorities | Developers |
| **[CHANGELOG.md](CHANGELOG.md)** | Version history, changes | All |

---

## 📖 Specifications

### Core Specs

Located in `specs/` directory:

| Specification | Description |
|---------------|-------------|
| **[RHIZOCRYPT_SPECIFICATION.md](specs/RHIZOCRYPT_SPECIFICATION.md)** | Complete system specification |
| **[ARCHITECTURE.md](specs/ARCHITECTURE.md)** | System design and components |
| **[DATA_MODEL.md](specs/DATA_MODEL.md)** | Data structures and types |
| **[API_SPECIFICATION.md](specs/API_SPECIFICATION.md)** | Public API reference |

### Protocol Specs

| Specification | Description |
|---------------|-------------|
| **[DEHYDRATION_PROTOCOL.md](specs/DEHYDRATION_PROTOCOL.md)** | Commit protocol to permanent storage |
| **[SLICE_SEMANTICS.md](specs/SLICE_SEMANTICS.md)** | Checkout semantics from permanent storage |
| **[STORAGE_BACKENDS.md](specs/STORAGE_BACKENDS.md)** | Storage backend options |
| **[INTEGRATION_SPECIFICATION.md](specs/INTEGRATION_SPECIFICATION.md)** | Inter-primal integration |

---

## 🎪 Showcase & Demos

### Showcase Index

| Document | Description |
|----------|-------------|
| **[showcase/README.md](showcase/README.md)** | Showcase overview |
| **[showcase/00_SHOWCASE_INDEX.md](showcase/00_SHOWCASE_INDEX.md)** | Complete demo index |
| **[showcase/00-local-primal/STATUS.md](showcase/00-local-primal/STATUS.md)** | Local demo status |

### Demo Categories

1. **Local Primal Demos** (`showcase/00-local-primal/`)
   - Level 1: Hello rhizoCrypt (3 demos)
   - Level 2: DAG Engine (4 demos)
   - Level 3: Merkle Proofs (4 demos)
   - Level 4: Sessions (4 demos) ✨
   - Level 5: Performance (4 demos)
   - Level 6: Advanced Patterns (3 demos)

2. **Inter-Primal Integration** (`showcase/01-inter-primal-live/`)
   - Phase 1: Songbird (Discovery) - 4 demos ✅
   - Phase 2: BearDog (Signing) - 4 demos ✅
   - Phase 3: NestGate (Storage) - 4 demos 📋
   - Phase 4: ToadStool (Compute) - 4 demos 📋
   - Phase 5: Squirrel (AI) - 3 demos 📋
   - Phase 6: Complete Workflow - 3 demos 📋

---

## 🛠️ Development

### Configuration

| Document | Description |
|----------|-------------|
| **[ENV_VARS.md](ENV_VARS.md)** | Environment variables reference |
| **[Cargo.toml](Cargo.toml)** | Workspace configuration |
| **[rustfmt.toml](rustfmt.toml)** | Code formatting rules |

### Crate Documentation

| Crate | Location | Description |
|-------|----------|-------------|
| **rhizo-crypt-core** | `crates/rhizo-crypt-core/` | Core DAG engine |
| **rhizo-crypt-rpc** | `crates/rhizo-crypt-rpc/` | RPC layer (tarpc) |

Generate API docs:
```bash
cargo doc --workspace --no-deps --open
```

---

## 📋 Planning & Status

### Current Status

| Document | Description |
|----------|-------------|
| **[STATUS.md](STATUS.md)** | Current metrics, progress, priorities |
| **[WHATS_NEXT.md](WHATS_NEXT.md)** | Roadmap and next steps |

### Integration Status

| Document | Description |
|----------|-------------|
| **[showcase/01-inter-primal-live/README.md](showcase/01-inter-primal-live/README.md)** | Integration overview |
| **[showcase/01-inter-primal-live/GAPS_DISCOVERED.md](showcase/01-inter-primal-live/GAPS_DISCOVERED.md)** | Integration gaps and fixes |

---

## 📦 Archives

### Session Documentation

Historical development session documents (fossil record):

| Archive | Contents |
|---------|----------|
| **[../archive/rhizoCrypt/2025-12-25-session-docs/](../archive/rhizoCrypt/2025-12-25-session-docs/)** | Dec 24-25 session docs |
| **[../archive/rhizoCrypt/2025-12-24-evolution-session/](../archive/rhizoCrypt/2025-12-24-evolution-session/)** | Dec 24 evolution session |
| **[../archive/rhizoCrypt/pure-rust-migration/](../archive/rhizoCrypt/pure-rust-migration/)** | Pure Rust migration artifacts |

### Key Archived Documents

- Pure Rust Evolution reports
- Comprehensive audit reports
- Refactoring completion reports
- Showcase planning documents
- Session summaries

---

## 🔍 Finding Documentation

### By Topic

**Getting Started**:
- [START_HERE.md](START_HERE.md) - New user guide
- [showcase/00-local-primal/00_START_HERE.md](showcase/00-local-primal/00_START_HERE.md) - Demo quick start

**Core Concepts**:
- [specs/RHIZOCRYPT_SPECIFICATION.md](specs/RHIZOCRYPT_SPECIFICATION.md) - Full specification
- [specs/ARCHITECTURE.md](specs/ARCHITECTURE.md) - System design

**API Reference**:
- [specs/API_SPECIFICATION.md](specs/API_SPECIFICATION.md) - Public API
- `cargo doc --open` - Generated API docs

**Integration**:
- [specs/INTEGRATION_SPECIFICATION.md](specs/INTEGRATION_SPECIFICATION.md) - Integration guide
- [showcase/01-inter-primal-live/README.md](showcase/01-inter-primal-live/README.md) - Live integration

**Development**:
- [STATUS.md](STATUS.md) - Current status
- [CHANGELOG.md](CHANGELOG.md) - Version history
- [ENV_VARS.md](ENV_VARS.md) - Configuration

### By Audience

**New Users**:
1. [START_HERE.md](START_HERE.md)
2. [README.md](README.md)
3. [showcase/README.md](showcase/README.md)

**Developers**:
1. [STATUS.md](STATUS.md)
2. [specs/ARCHITECTURE.md](specs/ARCHITECTURE.md)
3. [specs/API_SPECIFICATION.md](specs/API_SPECIFICATION.md)

**Integrators**:
1. [specs/INTEGRATION_SPECIFICATION.md](specs/INTEGRATION_SPECIFICATION.md)
2. [showcase/01-inter-primal-live/README.md](showcase/01-inter-primal-live/README.md)
3. [showcase/01-inter-primal-live/GAPS_DISCOVERED.md](showcase/01-inter-primal-live/GAPS_DISCOVERED.md)

**Contributors**:
1. [STATUS.md](STATUS.md)
2. [WHATS_NEXT.md](WHATS_NEXT.md)
3. [CHANGELOG.md](CHANGELOG.md)

---

## 📊 Documentation Status

| Category | Status | Notes |
|----------|--------|-------|
| **Getting Started** | ✅ | Complete and up-to-date |
| **Specifications** | ✅ | Core specs complete |
| **API Docs** | ⚠️ | Rustdoc needs expansion |
| **Showcase** | ✅ | 25/48 demos (52%) |
| **Integration** | 🔄 | 2/6 phases complete |
| **Archives** | ✅ | Well organized |

---

## 🔗 External Resources

### ecoPrimals Ecosystem

- **Phase 1 Primals**: `../../phase1/`
  - Songbird (Discovery)
  - BearDog (Identity)
  - NestGate (Storage)
  - ToadStool (Compute)
  - Squirrel (AI)

### Binaries

- **Phase 1 Binaries**: `../bins/`
  - Used for live integration demos

---

## 📝 Contributing to Documentation

### Adding New Docs

1. **Specifications**: Add to `specs/` directory
2. **Demos**: Add to appropriate `showcase/` subdirectory
3. **Archives**: Place in `docs/archive/` with date prefix

### Updating Docs

1. Update the document
2. Update `CHANGELOG.md` if significant
3. Update this index if new document added
4. Update `STATUS.md` if status changed

### Documentation Standards

- Use markdown format
- Include date in headers
- Link to related documents
- Keep README.md current
- Archive old session docs

---

**Last Updated**: December 25, 2025  
**Documentation Version**: 1.0.0  
**rhizoCrypt Version**: 0.10.0

---

*For questions about documentation, see [START_HERE.md](START_HERE.md)*
