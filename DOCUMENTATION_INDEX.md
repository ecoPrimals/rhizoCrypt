# 📚 rhizoCrypt Documentation Index

Complete guide to all rhizoCrypt documentation.

---

## 🚀 Quick Start (Start Here!)

| Document | Purpose | Audience |
|----------|---------|----------|
| **[START_HERE.md](START_HERE.md)** | New user onboarding | Everyone |
| **[README.md](README.md)** | Main project overview | Everyone |
| **[STATUS.md](STATUS.md)** | Current project status | Everyone |

**Recommendation**: Start with `START_HERE.md`, then read `README.md`.

---

## 🎯 December 2025 Audit & Handoff

### Primary Documents

| Document | Size | Purpose |
|----------|------|---------|
| **[HANDOFF_FINAL_DEC_26_2025.md](HANDOFF_FINAL_DEC_26_2025.md)** ⭐ | 11KB | Complete handoff guide with all deliverables |
| **[EXECUTIVE_SUMMARY_FINAL.md](EXECUTIVE_SUMMARY_FINAL.md)** | 5.7KB | Executive-level overview of audit |
| **[VERIFICATION_CHECKLIST.md](VERIFICATION_CHECKLIST.md)** | 9.2KB | 13-point quality verification |

### Archived Audit Reports

Location: `docs/archive/dec-26-2025-audit/`

15 comprehensive reports (~120KB total) documenting the complete audit process:
- Initial findings
- Remediation steps
- Coverage analysis
- Architecture improvements
- Final verification

See [docs/archive/dec-26-2025-audit/README.md](docs/archive/dec-26-2025-audit/README.md) for details.

---

## 📖 Core Documentation

### Project Documentation

| Document | Purpose |
|----------|---------|
| [README.md](README.md) | Main project documentation |
| [START_HERE.md](START_HERE.md) | Getting started guide |
| [STATUS.md](STATUS.md) | Current project status |
| [CHANGELOG.md](CHANGELOG.md) | Version history |
| [RELEASE_NOTES_v0.12.0.md](RELEASE_NOTES_v0.12.0.md) | v0.12.0 release notes |

### Configuration

| Document | Purpose |
|----------|---------|
| [ENV_VARS.md](ENV_VARS.md) | Environment variables reference |
| [INFANT_DISCOVERY.md](INFANT_DISCOVERY.md) | Discovery mechanism details |

---

## 📋 Technical Specifications

Location: `specs/`

| Specification | Purpose |
|--------------|---------|
| [00_SPECIFICATIONS_INDEX.md](specs/00_SPECIFICATIONS_INDEX.md) | Index of all specifications |
| [RHIZOCRYPT_SPECIFICATION.md](specs/RHIZOCRYPT_SPECIFICATION.md) | Core rhizoCrypt specification |
| [ARCHITECTURE.md](specs/ARCHITECTURE.md) | System architecture |
| [DATA_MODEL.md](specs/DATA_MODEL.md) | Data structures and models |
| [DEHYDRATION_PROTOCOL.md](specs/DEHYDRATION_PROTOCOL.md) | Commit to permanent storage |
| [SLICE_SEMANTICS.md](specs/SLICE_SEMANTICS.md) | Checkout from permanent storage |
| [INTEGRATION_SPECIFICATION.md](specs/INTEGRATION_SPECIFICATION.md) | Inter-primal integration |
| [API_SPECIFICATION.md](specs/API_SPECIFICATION.md) | API reference |
| [STORAGE_BACKENDS.md](specs/STORAGE_BACKENDS.md) | Storage backend options |

---

## 🎪 Showcase & Demos

Location: `showcase/`

### Overview Documents

| Document | Purpose | Status |
|----------|---------|--------|
| [showcase/README.md](showcase/README.md) | Showcase overview | Current |
| [showcase/00_SHOWCASE_INDEX.md](showcase/00_SHOWCASE_INDEX.md) | Complete demo index | Current |
| [showcase/QUICK_START.sh](showcase/QUICK_START.sh) | 5-minute quick start | ✅ New |
| [showcase/00_START_HERE.md](showcase/00_START_HERE.md) | Main entry point | ✅ New |
| [SHOWCASE_STATUS_REPORT_DEC_26_2025.md](SHOWCASE_STATUS_REPORT_DEC_26_2025.md) | Completion report | ✅ New |

### Demo Categories

**Level 0: Local Primal** (32 demos) ✅ **100% Complete**
- Location: `showcase/00-local-primal/`
- Purpose: Learn rhizoCrypt standalone (zero external dependencies)
- Sections:
  - Hello rhizoCrypt (3 demos)
  - DAG Engine (4 demos)
  - Merkle Proofs (4 demos)
  - Sessions (4 demos)
  - Slice Semantics ⭐ (6 demos) - All 6 modes with real-world use cases
  - Performance (4 demos)
  - Advanced Patterns (3 demos)
  - Real-World Scenarios ⭐ (4 demos) - ML pipeline, supply chain, etc.

**Level 1: Inter-Primal Integration** (In Progress)
- Location: `showcase/01-inter-primal-live/`
- Purpose: Integration with Phase 1 primals
- Status: Transitioning from mocks to real binaries
- Topics: Songbird, BearDog, NestGate, ToadStool, Complete Workflows

---

## 🏗️ Infrastructure

### CI/CD

| File | Purpose |
|------|---------|
| [.github/workflows/ci.yml](.github/workflows/ci.yml) | GitHub Actions pipeline |
| [.llvm-cov.toml](.llvm-cov.toml) | Coverage configuration |

### Deployment

| File | Purpose |
|------|---------|
| [Dockerfile](Dockerfile) | Production Docker image |
| [k8s/deployment.yaml](k8s/deployment.yaml) | Kubernetes deployment |

---

## 📦 Crate Documentation

### rhizo-crypt-core

| Document | Purpose |
|----------|---------|
| [crates/rhizo-crypt-core/README.md](crates/rhizo-crypt-core/README.md) | Core library documentation |
| Generated docs | Run `cargo doc --open -p rhizo-crypt-core` |

**Key Modules**:
- `lib.rs` - Main RhizoCrypt struct
- `session.rs` - Session management
- `vertex.rs` - Vertex operations
- `merkle.rs` - Merkle tree computation
- `dehydration.rs` - Commit protocol
- `slice.rs` - Checkout semantics
- `discovery.rs` - Capability discovery
- `clients/` - Capability-based clients

### rhizo-crypt-rpc

| Document | Purpose |
|----------|---------|
| Generated docs | Run `cargo doc --open -p rhizo-crypt-rpc` |

**Key Modules**:
- `service.rs` - RPC service implementation
- `client.rs` - RPC client
- `server.rs` - Server setup
- `metrics.rs` - Monitoring

### rhizocrypt-service

| Document | Purpose |
|----------|---------|
| [crates/rhizocrypt-service/README.md](crates/rhizocrypt-service/README.md) | Service binary documentation |

---

## 🔍 Finding What You Need

### I want to...

**...get started quickly**
→ [START_HERE.md](START_HERE.md)

**...understand what rhizoCrypt does**
→ [README.md](README.md)

**...see it in action**
→ [showcase/README.md](showcase/README.md)

**...understand the architecture**
→ [specs/ARCHITECTURE.md](specs/ARCHITECTURE.md)

**...integrate with rhizoCrypt**
→ [specs/INTEGRATION_SPECIFICATION.md](specs/INTEGRATION_SPECIFICATION.md)

**...deploy to production**
→ [HANDOFF_FINAL_DEC_26_2025.md](HANDOFF_FINAL_DEC_26_2025.md)

**...understand the audit results**
→ [EXECUTIVE_SUMMARY_FINAL.md](EXECUTIVE_SUMMARY_FINAL.md)

**...verify quality metrics**
→ [VERIFICATION_CHECKLIST.md](VERIFICATION_CHECKLIST.md)

**...see the API**
→ [specs/API_SPECIFICATION.md](specs/API_SPECIFICATION.md) or `cargo doc --open`

**...understand dehydration**
→ [specs/DEHYDRATION_PROTOCOL.md](specs/DEHYDRATION_PROTOCOL.md)

**...understand slices**
→ [specs/SLICE_SEMANTICS.md](specs/SLICE_SEMANTICS.md)

**...configure environment**
→ [ENV_VARS.md](ENV_VARS.md)

**...understand discovery**
→ [INFANT_DISCOVERY.md](INFANT_DISCOVERY.md)

---

## 📊 Documentation Statistics

| Category | Count | Total Size |
|----------|-------|------------|
| Root Documentation | 10 files | ~100KB |
| Audit Reports (archived) | 15 files | ~120KB |
| Specifications | 9 files | ~80KB |
| Showcase Docs | 25+ files | ~50KB |
| Crate READMEs | 3 files | ~20KB |
| **Total** | **60+ files** | **~370KB** |

---

## 🎯 Documentation by Audience

### For End Users

1. [START_HERE.md](START_HERE.md)
2. [README.md](README.md)
3. [showcase/README.md](showcase/README.md)
4. [ENV_VARS.md](ENV_VARS.md)

### For Developers

1. [README.md](README.md)
2. [specs/ARCHITECTURE.md](specs/ARCHITECTURE.md)
3. [specs/RHIZOCRYPT_SPECIFICATION.md](specs/RHIZOCRYPT_SPECIFICATION.md)
4. [crates/rhizo-crypt-core/README.md](crates/rhizo-crypt-core/README.md)
5. Generated API docs (`cargo doc --open`)

### For Integrators

1. [specs/INTEGRATION_SPECIFICATION.md](specs/INTEGRATION_SPECIFICATION.md)
2. [specs/API_SPECIFICATION.md](specs/API_SPECIFICATION.md)
3. [showcase/01-inter-primal-live/](showcase/01-inter-primal-live/)
4. [INFANT_DISCOVERY.md](INFANT_DISCOVERY.md)

### For Operators

1. [HANDOFF_FINAL_DEC_26_2025.md](HANDOFF_FINAL_DEC_26_2025.md)
2. [Dockerfile](Dockerfile)
3. [k8s/deployment.yaml](k8s/deployment.yaml)
4. [ENV_VARS.md](ENV_VARS.md)

### For Executives

1. [EXECUTIVE_SUMMARY_FINAL.md](EXECUTIVE_SUMMARY_FINAL.md)
2. [STATUS.md](STATUS.md)
3. [README.md](README.md) (metrics section)

---

## 🔄 Documentation Maintenance

### Last Updated

December 26, 2025

### Update Frequency

- **README.md**: Updated with each release
- **STATUS.md**: Updated monthly or after major milestones
- **CHANGELOG.md**: Updated with each release
- **Specifications**: Updated as architecture evolves
- **Showcase**: Updated as demos are added/modified

### Contributing to Documentation

When adding documentation:
1. Update this index
2. Follow existing formatting conventions
3. Add cross-references where appropriate
4. Update the "Last Updated" date
5. Run `cargo doc` to verify API docs

---

## 📞 Questions?

If you can't find what you need:
1. Check this index
2. Search the repository
3. Review the showcase demos
4. Read the specifications
5. Check the audit reports archive

---

**rhizoCrypt Documentation** — *Comprehensive, organized, accessible* 📚

*Last Updated: December 26, 2025*

