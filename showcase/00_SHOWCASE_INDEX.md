# 🔐 rhizoCrypt Showcase Index

**A progressive tour of rhizoCrypt capabilities**

---

## 📊 Status Overview

| Phase | Name | Demos | Status |
|-------|------|-------|--------|
| 01 | Isolated Instance | 4 | ✅ Complete |
| 02 | RPC Layer | 1 | 🔧 Scaffolded |
| 03 | Inter-Primal | 4 | ✅ Complete |
| 04 | Complete Workflows | 1 | ✅ Complete |
| 05 | Live Integration | 2 | ✅ NEW |

**Total Demos:** 12 (10 mock + 2 live)

---

## 📁 Demo Catalog

### Phase 01: Isolated Instance
*Core rhizoCrypt capabilities in standalone mode*

| Demo | Path | Time | Description |
|------|------|------|-------------|
| Session Lifecycle | `01-isolated/sessions/` | 5m | Create, grow, query, resolve sessions |
| DAG Operations | `01-isolated/dag/` | 5m | Multi-parent DAG, content-addressing |
| Merkle Proofs | `01-isolated/merkle/` | 5m | Tree construction, proof verification |
| Slice Semantics | `01-isolated/slices/` | 5m | Copy/Loan/Consignment modes |

### Phase 02: RPC Layer
*tarpc-based remote access*

| Demo | Path | Time | Description |
|------|------|------|-------------|
| Server Startup | `02-rpc/server/` | 3m | Start rhizoCrypt RPC server |

### Phase 03: Inter-Primal Integration
*Ecosystem collaboration*

| Demo | Path | Time | Description |
|------|------|------|-------------|
| Capability Discovery | `03-inter-primal/songbird-discovery/` | 5m | Runtime primal discovery |
| BearDog Signing | `03-inter-primal/beardog-signing/` | 5m | DID verification, signatures |
| NestGate Payloads | `03-inter-primal/nestgate-payloads/` | 5m | Content-addressed storage |
| LoamSpine Commits | `03-inter-primal/loamspine-commits/` | 5m | Permanent storage, checkout |

### Phase 04: Complete Workflows
*End-to-end scenarios*

| Demo | Path | Time | Description |
|------|------|------|-------------|
| Dehydration | `04-complete-workflow/dehydration/` | 10m | Session → Merkle → Commit |

### Phase 05: Live Integration
*Real primal connections (requires binaries from `../../bins/`)*

| Demo | Path | Time | Description |
|------|------|------|-------------|
| Start Primals | `05-live-integration/start-primals.sh` | 1m | Start Songbird, NestGate |
| Live Discovery | `05-live-integration/demo-live-discovery.sh` | 5m | Real Songbird connection |
| Live Signing | `05-live-integration/demo-live-signing.sh` | 5m | Real BearDog connection |
| Stop Primals | `05-live-integration/stop-primals.sh` | 1m | Cleanup |

---

## 🚀 Quick Start

```bash
# Run all demos interactively
./QUICK_START.sh

# Or run individual demos:
cd 01-isolated/sessions && ./demo-session-lifecycle.sh
cd 01-isolated/dag && ./demo-dag-operations.sh
cd 01-isolated/merkle && ./demo-merkle-proofs.sh
cd 01-isolated/slices && ./demo-slice-semantics.sh
cd 03-inter-primal/songbird-discovery && ./demo-discovery.sh
cd 03-inter-primal/beardog-signing && ./demo-signing.sh
cd 03-inter-primal/nestgate-payloads && ./demo-payload-storage.sh
cd 03-inter-primal/loamspine-commits && ./demo-loamspine-commit.sh
cd 04-complete-workflow/dehydration && ./demo-simple-dehydration.sh
```

---

## 📚 Learning Path

```
Start Here
    │
    ▼
┌─────────────────────────────────────────────────────────────┐
│ Phase 1: Core Concepts                                       │
│   Sessions → DAG → Merkle → Slices                          │
└─────────────────────────────────────────────────────────────┘
    │
    ▼
┌─────────────────────────────────────────────────────────────┐
│ Phase 2: Remote Access (Optional)                            │
│   RPC Server                                                 │
└─────────────────────────────────────────────────────────────┘
    │
    ▼
┌─────────────────────────────────────────────────────────────┐
│ Phase 3: Ecosystem Integration                               │
│   Discovery → Signing → Payloads                            │
└─────────────────────────────────────────────────────────────┘
    │
    ▼
┌─────────────────────────────────────────────────────────────┐
│ Phase 4: Complete Workflows                                  │
│   Dehydration                                               │
└─────────────────────────────────────────────────────────────┘
```

---

## 🔧 Prerequisites

- Rust toolchain (rustup, cargo)
- ~500MB disk space for build artifacts
- Internet connection (for crate downloads on first run)

---

## 📝 Demo Output

Each demo:
1. Creates a temporary directory
2. Builds a standalone Rust program
3. Runs the demo
4. Cleans up after itself

Demo output includes:
- Step-by-step explanations
- Visual diagrams
- Key takeaways

---

*rhizoCrypt: The memory that knows when to forget*
