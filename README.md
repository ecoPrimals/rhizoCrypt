# rhizoCrypt

**Ephemeral DAG Engine** — Phase 2 Working Memory for the ecoPrimals ecosystem.

| Metric | Value |
|--------|-------|
| Version | 0.14.17 |
| License | AGPL-3.0-or-later / ORC / CC-BY-SA 4.0 ([scyBorg Triple-Copyleft](LICENSE)) |
| Tests | 1,878 passing (`--all-features`, Jul 18, 2026) |
| Coverage | 93.70% lines (llvm-cov, Jul 18, 2026) |
| Clippy | 0 warnings (pedantic + nursery + cargo + cast lints, `unwrap_used`/`expect_used = "deny"`, `missing_errors_doc = "warn"`) |
| Edition | 2024 (rust-version 1.87) |
| Unsafe | `unsafe_code = "deny"` workspace-wide, `#![forbid(unsafe_code)]` in non-test, zero `unsafe` blocks |
| Binary | `rhizocrypt` (UniBin, subcommands via clap) |
| IPC | JSON-RPC 2.0 (HTTP + newline) + tarpc 0.37 (bincode) — UDS unconditional, TCP opt-in |
| Streaming | NDJSON pipeline coordination for `event.append_batch` |
| Resilience | Lock-free CircuitBreaker (atomics) + RetryPolicy for IPC clients |
| Error Model | Structured `IpcErrorPhase` + `DispatchOutcome` (protocol vs application) |
| Discovery | Capability-based + manifest (PG-32) + Neural API `primal.announce` (Wave 43) |
| Chaos | `ChaosEngine` framework with 7 fault classes |
| Transport | UDS unconditional, TCP opt-in (`--port`), `TRANSPORT_ENDPOINT` injection (local impl), BTSP Phase 3 (ChaCha20-Poly1305) |
| Storage | `DagBackend` enum: redb (Pure Rust, ACID, default) / in-memory |
| Deps | ecoBin compliant — zero application C deps, zero cross-primal compile deps, zero reqwest |
| Audit | `cargo-deny` enforced (18-crate ecoBin ban list incl. reqwest + ring, advisories, licenses, sources) |
| SPDX | `AGPL-3.0-or-later` header on all 223 `.rs` files |
| Niche | `niche.rs` `METHOD_CATALOG` — single source of truth (identity, capabilities, costs, deps, domains, MCP tools) |
| Validation | `validation.rs` composable harness + pluggable sinks (ludoSpring V22) |
| Registry | `config/capability_registry.toml` (37 methods, 7 domains, stability tiers, `provenance.*` → `dag.*` wire aliases) |
| Deploy | `graphs/rhizocrypt_deploy.toml` (biomeOS niche, `fallback = "skip"`) |
| Cross-compile | plasmidBin: musl (x86_64, aarch64), RISC-V — ecoBin v3.0 |

---

## What is rhizoCrypt?

rhizoCrypt is the **ephemeral working memory** of the ecoPrimals ecosystem.
It manages content-addressed directed acyclic graphs (DAGs) scoped to sessions.
Data is temporary by default — only explicit **dehydration** commits results to
permanent storage.

**Canonical capability domain: `dag`** — deploy graphs and capability routing
should use `by_capability = "dag"` when targeting rhizoCrypt. The `"provenance"`
domain belongs to sweetGrass; rhizoCrypt *consumes* provenance capabilities but
does not *provide* them.

**Wire-name aliases (GAP-36)**: Downstream springs may call `provenance.*`
methods (e.g. `provenance.session.create`, `provenance.event.append`).
rhizoCrypt normalizes these to `dag.*` at dispatch time — both names are
valid on the wire. See `capability_registry.toml` for the full alias table.

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
| `rhizo-crypt-rpc` | tarpc 0.37 service (28 ops), JSON-RPC 2.0 handler (37 methods across 7 domains), NDJSON streaming, rate limiting, metrics |
| `rhizocrypt-service` | UniBin binary and library (`server`, `client`, `status`, `version`, `doctor`) |

---

## Quick Start

```bash
# Build
cargo build --workspace

# Run all tests
cargo test --workspace

# Run the service (UDS-only, default socket)
cargo run -p rhizocrypt-service -- server

# With TCP opt-in (tarpc + JSON-RPC on port 9400)
cargo run -p rhizocrypt-service -- server --port 9400

# With custom UDS path + TCP
cargo run -p rhizocrypt-service -- server --unix /tmp/rhizocrypt.sock --port 9400

# With discovery adapter
RHIZOCRYPT_DISCOVERY_ADAPTER=songbird.local:7500 \
  cargo run -p rhizocrypt-service -- server --port 9400

# Lint (pedantic)
cargo clippy --workspace --all-targets --all-features -- -D warnings

# Coverage
cargo llvm-cov --workspace --html
```

---

## Transport Model (GAP-06 resolved + Wave 100 transport evolution)

UDS is **unconditional** on Unix — no flags needed. TCP is **opt-in** via
`--port` or `RHIZOCRYPT_PORT` (Tier 5 fallback for standalone/debug only).

**Transport injection** (Wave 100–101): Accepts `TRANSPORT_ENDPOINT` env var as
JSON-encoded `TransportEndpoint` from the launcher. Outbound IPC uses local
`connect_transport()` — transport-agnostic (UDS, TCP, or mesh relay). Wire
format is the contract — no cross-primal compile deps.

```
rhizocrypt server                           # UDS-only (default)
rhizocrypt server --port 9400               # UDS + TCP (opt-in)
rhizocrypt server --unix /tmp/rc.sock       # UDS at custom path
rhizocrypt doctor                           # Verify transport (shows socket path)
```

**Verify from downstream** (socat-style):
```bash
echo '{"jsonrpc":"2.0","method":"health.liveness","id":1}' | \
  socat - UNIX-CONNECT:$XDG_RUNTIME_DIR/biomeos/rhizocrypt.sock
# → {"jsonrpc":"2.0","result":{"status":"alive",...},"id":1}
```

Socket path: `$XDG_RUNTIME_DIR/biomeos/rhizocrypt[-{family_id}].sock`.
Family-scoped when `FAMILY_ID` is set (composition standard).

---

## Composition Readiness (Stadial)

### Downstream Pairing

| Partner | Role | Key Methods |
|---------|------|-------------|
| wetSpring | DAG checkpointing for 264-clone LTEE pipelines | `dag.session.create`, `dag.event.append`, `dag.partial_dehydrate` |
| lithoSpore | Provenance DAG verification substrate | `dag.merkle.root`, `dag.merkle.proof`, `dag.dehydration.trigger` |
| projectFOUNDATION | Thread lineage — DAG sessions anchor evidence | `dag.session.get` (summary), `dag.vertex.query` |
| healthSpring | Nest atomic clinical data pipeline | `provenance.session.create`, `provenance.event.append` (aliases) |

### Degradation Behavior

When rhizoCrypt is **unavailable**, downstream consumers degrade as follows:

- **wetSpring**: Emits partial braids with `dag_merkle_root: ""` and
  `"status": "partial"`. Per-clone BLAKE3 hashes remain verifiable. Science
  is never gated behind provenance.
- **lithoSpore**: Falls back to per-vertex BLAKE3 hashes for verification
  instead of full Merkle proofs. Individual event integrity is preserved.
- **biomeOS graph execution**: DAG-dependent phases skip with
  `"dag capability not available"`. Other composition phases proceed.

### Neural API Registration (Wave 43)

On startup after UDS bind, rhizoCrypt sends `primal.announce` to biomeOS's
Neural API socket. This registers `dag`, `integrity`, `merkle` capabilities
with cost hints and latency estimates so the Neural API can route
`capability.call` dispatches with informed affinity. Discovery uses tiered
lookup: `$NEURAL_API_SOCKET` → `$XDG_RUNTIME_DIR/biomeos/neural-api-{family}.sock`
→ `/tmp/biomeos/neural-api-{family}.sock`. Non-fatal if biomeOS is unavailable.

### Stability Tiers

31 of 37 methods are **stable**. 6 are **evolving**:
`dag.partial_dehydrate`, `dag.branch`, `dag.diff`, `dag.merge`, `dag.federate`
(Wave 60), `mesh.events.record` (Wave 76c).

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
| `RHIZOCRYPT_PORT` | Opt-in TCP: tarpc listen port (triggers TCP transport) |
| `RHIZOCRYPT_JSONRPC_PORT` | Opt-in TCP: JSON-RPC port (default: tarpc port + 1) |
| `XDG_RUNTIME_DIR` | UDS socket directory base (default: `/run/user/$UID`); socket at `$XDG_RUNTIME_DIR/biomeos/rhizocrypt.sock` |

See [docs/ENV_VARS.md](docs/ENV_VARS.md) for the complete list.

---

## Compliance

| Standard | Status | Notes |
|----------|--------|-------|
| UniBin | Compliant | Single `rhizocrypt` binary with clap subcommands |
| ecoBin v3.0 | Compliant | Default `redb` backend is 100% Pure Rust; cross-compile via plasmidBin (musl, RISC-V) |
| Universal IPC v3 | Compliant | JSON-RPC 2.0 + tarpc, semantic method names |
| Semantic Naming | Compliant | Native (`commit.*`) + compat (`permanent-storage.*`) with negotiation |
| `unsafe_code = "deny"` | Compliant | Workspace-wide, `forbid` in non-test builds |
| scyBorg Triple-Copyleft | Compliant | AGPL-3.0+ (software), ORC (mechanics), CC-BY-SA 4.0 (docs) |

---

## Documentation

- [docs/ENV_VARS.md](docs/ENV_VARS.md) — Environment variable reference
- [docs/DEPLOYMENT_CHECKLIST.md](docs/DEPLOYMENT_CHECKLIST.md) — Production deployment
- [specs/](specs/) — Formal specifications (architecture, data model, protocols, experiments)
- [specs/CRYPTO_MODEL.md](specs/CRYPTO_MODEL.md) — Canonical crypto delegation pattern (signing provider IPC)
- [CHANGELOG.md](CHANGELOG.md) — Version history
- [deny.toml](deny.toml) — Dependency audit policy (`cargo-deny`)
- [capability_registry.toml](config/capability_registry.toml) — Capability registry for biomeOS routing
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
