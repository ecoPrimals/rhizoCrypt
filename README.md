# rhizoCrypt

**Ephemeral DAG Engine** — Phase 2 Working Memory for the ecoPrimals ecosystem.

| Metric | Value |
|--------|-------|
| Version | 0.14.0-dev |
| License | AGPL-3.0-or-later / ORC / CC-BY-SA 4.0 ([scyBorg Triple-Copyleft](LICENSE)) |
| Tests | 1,424 passing (`--all-features`) |
| Coverage | 94.34% lines, 93.41% functions (CI gate: 90%) |
| Clippy | 0 warnings (pedantic + nursery + cargo + cast lints, `unwrap_used`/`expect_used = "deny"`, `missing_errors_doc = "warn"`) |
| Edition | 2024 (rust-version 1.87) |
| Unsafe | `unsafe_code = "deny"` workspace-wide, `#![forbid(unsafe_code)]` in non-test, zero `unsafe` blocks |
| Binary | `rhizocrypt` (UniBin, subcommands via clap) |
| IPC | JSON-RPC 2.0 (HTTP + newline) + tarpc 0.37 (bincode) — dual-transport first |
| Streaming | NDJSON pipeline coordination for `event.append_batch` |
| Resilience | Lock-free CircuitBreaker (atomics) + RetryPolicy for IPC clients |
| Error Model | Structured `IpcErrorPhase` + `DispatchOutcome` (protocol vs application) |
| Discovery | Capability-based + manifest-based (`$XDG_RUNTIME_DIR/biomeos/{primal}.json`) |
| Chaos | `ChaosEngine` framework with 7 fault classes |
| Transport | Platform-agnostic (Unix socket / TCP / abstract socket) |
| Storage | `DagBackend` enum: redb (Pure Rust, ACID, default) / in-memory |
| Deps | ecoBin compliant — zero application C deps, zero cross-primal compile deps |
| Audit | `cargo-deny` enforced (16-crate ecoBin ban list, advisories, licenses, sources) |
| SPDX | `AGPL-3.0-or-later` header on all 129 `.rs` files |
| Niche | `niche.rs` self-knowledge (identity, capabilities, costs, deps, domains, MCP tools) |
| Validation | `validation.rs` composable harness + pluggable sinks (ludoSpring V22) |
| Registry | `capability_registry.toml` (27 methods, 8 domains incl. `tools.*` MCP) |
| Deploy | `graphs/rhizocrypt_deploy.toml` (biomeOS niche, `fallback = "skip"`) |
| Cross-compile | CI: musl (x86_64, aarch64), RISC-V — ecoBin v3.0 |

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
JSON-RPC 2.0 over HTTP or newline-delimited TCP/UDS (required) with
tarpc/bincode (optional, high-performance). The TCP JSON-RPC port auto-detects
HTTP POST vs raw newline framing per connection. Unix domain sockets are
served at `$XDG_RUNTIME_DIR/biomeos/rhizocrypt.sock` (ecosystem standard).
Method names follow semantic capability naming: `commit.session`,
`signing.verify`, etc. Clients use method negotiation (native → compatibility
fallback) for forward/backward compatibility.

---

## Crates

| Crate | Purpose |
|-------|---------|
| `rhizo-crypt-core` | Core DAG engine: sessions, vertices, merkle, storage, capability clients, discovery |
| `rhizo-crypt-rpc` | tarpc 0.37 service (24 ops), JSON-RPC 2.0 handler (27 methods incl. `tools.list`, `tools.call`), NDJSON streaming, rate limiting, metrics |
| `rhizocrypt-service` | UniBin binary and library (`server`, `client`, `status`, `version`, `doctor`) |

---

## Quick Start

```bash
# Build
cargo build --workspace

# Run all tests
cargo test --workspace

# Run the service (TCP + dual-mode JSON-RPC)
cargo run -p rhizocrypt-service -- server

# With UDS listener (ecosystem standard socket path)
cargo run -p rhizocrypt-service -- server --unix

# With custom UDS path
cargo run -p rhizocrypt-service -- server --unix /tmp/rhizocrypt.sock

# With discovery adapter
RHIZOCRYPT_DISCOVERY_ADAPTER=songbird.local:7500 \
  cargo run -p rhizocrypt-service -- server --port 9400 --unix

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
| `RHIZOCRYPT_PORT` | tarpc listen port (default: OS-assigned dev, 9400 production) |
| `RHIZOCRYPT_JSONRPC_PORT` | JSON-RPC TCP port (default: tarpc port + 1, dual-mode HTTP + newline) |
| `XDG_RUNTIME_DIR` | UDS socket directory base (default: `/run/user/$UID`); socket at `$XDG_RUNTIME_DIR/biomeos/rhizocrypt.sock` |

See [docs/ENV_VARS.md](docs/ENV_VARS.md) for the complete list.

---

## Compliance

| Standard | Status | Notes |
|----------|--------|-------|
| UniBin | Compliant | Single `rhizocrypt` binary with clap subcommands |
| ecoBin v3.0 | Compliant | Default `redb` backend is 100% Pure Rust; cross-compile CI (musl, RISC-V) |
| Universal IPC v3 | Compliant | JSON-RPC 2.0 + tarpc, semantic method names |
| Semantic Naming | Compliant | Native (`commit.*`) + compat (`permanent-storage.*`) with negotiation |
| `unsafe_code = "deny"` | Compliant | Workspace-wide, `forbid` in non-test builds |
| scyBorg Triple-Copyleft | Compliant | AGPL-3.0+ (software), ORC (mechanics), CC-BY-SA 4.0 (docs) |

---

## Documentation

- [docs/ENV_VARS.md](docs/ENV_VARS.md) — Environment variable reference
- [docs/DEPLOYMENT_CHECKLIST.md](docs/DEPLOYMENT_CHECKLIST.md) — Production deployment
- [specs/](specs/) — Formal specifications (architecture, data model, protocols, experiments)
- [showcase/](showcase/) — Progressive demo suite (70+ working demos)
- [CHANGELOG.md](CHANGELOG.md) — Version history
- [deny.toml](deny.toml) — Dependency audit policy (`cargo-deny`)
- [capability_registry.toml](capability_registry.toml) — Capability registry for biomeOS routing
- [graphs/rhizocrypt_deploy.toml](graphs/rhizocrypt_deploy.toml) — biomeOS deploy graph

---

## License

scyBorg Triple-Copyleft: AGPL-3.0-or-later (software), ORC (game mechanics),
CC-BY-SA 4.0 (creative content/documentation). See [LICENSE](LICENSE).

---

## Part of ecoPrimals

This repo is part of the [ecoPrimals](https://github.com/ecoPrimals) sovereign
computing ecosystem — a collection of pure Rust binaries that coordinate via
JSON-RPC, capability-based routing, and zero compile-time coupling.

See [wateringHole](https://github.com/ecoPrimals/wateringHole) for ecosystem
documentation, standards, and the primal registry.
