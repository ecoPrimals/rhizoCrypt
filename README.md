# rhizoCrypt

**Ephemeral DAG Engine** — Phase 2 Working Memory for the ecoPrimals ecosystem.

| Metric | Value |
|--------|-------|
| Version | 0.13.0-dev |
| License | AGPL-3.0-only |
| Tests | 907 passing (default features) |
| Coverage | 90.83% line coverage (llvm-cov) |
| Clippy | 0 warnings (pedantic + nursery + cargo, all features) |
| Unsafe | `#![forbid(unsafe_code)]` workspace-wide |
| Binary | `rhizocrypt` (UniBin, subcommands via clap) |
| IPC | JSON-RPC 2.0 (HTTP) + tarpc (bincode) — dual-transport first |
| Transport | Platform-agnostic (Unix socket / TCP / abstract socket) |
| Storage | redb (Pure Rust, default) / sled (optional) |
| Deps | ecoBin compliant — zero application C dependencies |
| Audit | `cargo-deny` enforced (advisories, licenses, bans, sources) |
| SPDX | `AGPL-3.0-only` header on all 104 `.rs` files |

---

## What is rhizoCrypt?

rhizoCrypt is the **ephemeral working memory** of the ecoPrimals ecosystem.
It manages content-addressed directed acyclic graphs (DAGs) scoped to sessions.
Data is temporary by default — only explicit **dehydration** commits results to
permanent storage.

**Core primitives:**

- **Vertex** — Content-addressed event node (BLAKE3 hash, multi-parent DAG)
- **Session** — Scoped DAG with lifecycle (create, grow, resolve, expire)
- **Merkle Tree** — Cryptographic integrity proof over session vertices
- **Dehydration** — Commit ephemeral results to permanent storage
- **Slice** — Checkout immutable snapshot from permanent storage (Copy, Loan, Consignment)
- **Capability Discovery** — Runtime service discovery, zero hardcoded vendors

**Philosophy:**

> Ephemeral by default, persistent by consent.

> Orchestrate, don't embed. Each primal stays sovereign.

> Start with zero knowledge, discover capabilities at runtime.

---

## Architecture

```
rhizoCrypt (Ephemeral DAG Engine)
├── Vertex Store (content-addressed, BLAKE3)
├── DAG Index (topological ordering, frontier)
├── Merkle Trees (session integrity proofs)
├── Sessions (scoped lifecycles, lock-free DashMap)
└── Capability Discovery
    ├── Signing         → any signing provider
    ├── Permanent Storage → any commit/checkout provider
    ├── Payload Storage  → any content-addressed store
    ├── Compute          → any orchestration provider
    └── Provenance       → any attribution provider
```

All inter-primal communication uses the Universal IPC Standard:
JSON-RPC 2.0 over HTTP (required) with tarpc/bincode (optional, high-performance).
Method names follow semantic capability naming: `commit.session`,
`signing.verify`, etc. Clients use method negotiation (native → compatibility
fallback) for forward/backward compatibility.

---

## Crates

| Crate | Purpose |
|-------|---------|
| `rhizo-crypt-core` | Core DAG engine: sessions, vertices, merkle, storage, capability clients, discovery |
| `rhizo-crypt-rpc` | tarpc service (24 ops), JSON-RPC 2.0 handler, rate limiting, metrics |
| `rhizocrypt-service` | UniBin binary and library (`server`, `client`, `status`, `version`, `doctor`) |

---

## Quick Start

```bash
# Build
cargo build --workspace

# Run all tests
cargo test --workspace

# Run the service
cargo run -p rhizocrypt-service -- server

# With discovery adapter
RHIZOCRYPT_DISCOVERY_ADAPTER=songbird.local:7500 \
  cargo run -p rhizocrypt-service -- server --port 9400

# Lint (pedantic)
cargo clippy --workspace --all-targets --all-features -- -D warnings

# Coverage
cargo llvm-cov --workspace --html
```

---

## Configuration

rhizoCrypt discovers all services at runtime via environment variables:

| Variable | Purpose |
|----------|---------|
| `RHIZOCRYPT_DISCOVERY_ADAPTER` | Discovery service endpoint (primary) |
| `PERMANENT_STORAGE_ENDPOINT` | Direct permanent storage endpoint |
| `SIGNING_ENDPOINT` | Direct signing provider endpoint |
| `COMPUTE_ENDPOINT` | Direct compute orchestration endpoint |
| `PROVENANCE_ENDPOINT` | Direct provenance query endpoint |
| `RHIZOCRYPT_PORT` | Service listen port (default: OS-assigned dev, 9400 production) |

See [docs/ENV_VARS.md](docs/ENV_VARS.md) for the complete list.

---

## Compliance

| Standard | Status | Notes |
|----------|--------|-------|
| UniBin | Compliant | Single `rhizocrypt` binary with clap subcommands |
| ecoBin | Compliant | Default `redb` backend is 100% Pure Rust; `sled` available as optional feature |
| Universal IPC v3 | Compliant | JSON-RPC 2.0 + tarpc, semantic method names |
| Semantic Naming | Compliant | Native (`commit.*`) + compat (`permanent-storage.*`) with negotiation |
| `#![forbid(unsafe_code)]` | Compliant | Workspace-wide |
| AGPL-3.0-only | Compliant | SPDX headers on all source files |

---

## Documentation

- [docs/ENV_VARS.md](docs/ENV_VARS.md) — Environment variable reference
- [docs/DEPLOYMENT_CHECKLIST.md](docs/DEPLOYMENT_CHECKLIST.md) — Production deployment
- [specs/](specs/) — Formal specifications (architecture, data model, protocols)
- [showcase/](showcase/) — Progressive demo suite (70+ working demos)
- [CHANGELOG.md](CHANGELOG.md) — Version history
- [deny.toml](deny.toml) — Dependency audit policy (`cargo-deny`)

---

## License

AGPL-3.0-only. See [LICENSE](LICENSE).
