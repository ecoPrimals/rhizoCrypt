# rhizoCrypt — AI Context Block

**Version**: 0.14.0-dev
**Role**: Ephemeral DAG Engine — working memory for the ecoPrimals ecosystem
**License**: AGPL-3.0-or-later / ORC / CC-BY-SA 4.0 (scyBorg Triple-Copyleft)
**Language**: Rust 2024, edition 2024, MSRV 1.87 (dev toolchain 1.94.1)

## What It Does

rhizoCrypt manages content-addressed directed acyclic graphs (DAGs) scoped to
sessions. Data is temporary by default — only explicit **dehydration** commits
results to permanent storage via capability discovery.

## Core Primitives

- **Vertex** — Content-addressed event node (BLAKE3 hash, multi-parent DAG)
- **Session** — Scoped DAG with lifecycle (create → grow → resolve → expire)
- **Merkle Tree** — Cryptographic integrity proof over session vertices
- **Dehydration** — Commit ephemeral results to permanent storage
- **Slice** — Checkout immutable snapshot (Copy, Loan, Consignment modes)
- **Capability Discovery** — Runtime service discovery, zero hardcoded vendors

## Ecosystem Position

rhizoCrypt is one primal in the ecoPrimals ecosystem. It knows only itself
and discovers sibling capabilities at runtime via a discovery adapter or direct endpoints:

| Capability | What rhizoCrypt Needs | Discovered At Runtime |
|------------|----------------------|----------------------|
| Signing | Vertex signatures, attestations | Any `crypto.sign` provider |
| Permanent Storage | Dehydration commit/checkout | Any `commit.session` provider |
| Payload Storage | Content-addressed blobs | Any `storage.put` provider |
| Compute | Orchestration dispatch | Any `compute.dispatch` provider |
| Provenance | Attribution queries | Any `provenance.query` provider |

## Architecture

Three workspace crates:

| Crate | Purpose |
|-------|---------|
| `rhizo-crypt-core` | DAG engine, sessions, vertices, merkle, storage, capability clients, discovery |
| `rhizo-crypt-rpc` | tarpc service (24 ops), JSON-RPC 2.0 handler (28 methods incl. MCP), NDJSON streaming, rate limiting |
| `rhizocrypt-service` | UniBin binary (`server`, `client`, `status`, `version`, `doctor`) |

## IPC

- **UDS unconditional** on Unix at `$XDG_RUNTIME_DIR/biomeos/rhizocrypt.sock` (Provenance Trio standard)
- **TCP opt-in** via `--port` or `RHIZOCRYPT_PORT` env var (tarpc + JSON-RPC dual-mode)
- **JSON-RPC 2.0** — dual-mode TCP (auto-detects HTTP POST vs newline-delimited) + UDS
- **tarpc 0.37** with bincode — optional, high-performance typed RPC
- **BTSP Phase 2** — X25519 + HMAC-SHA256 handshake enforced on UDS accept when `FAMILY_ID` is set; dev mode (`BIOMEOS_INSECURE=1`) bypasses
- Method names follow `domain.verb` semantic naming (`dag.session.create`, `health.check`)

## Compliance

| Standard | Status |
|----------|--------|
| UniBin | Single binary, clap subcommands |
| ecoBin v3.0 | Zero application C deps, zero reqwest, cross-compile (musl, RISC-V) |
| genomeBin | Multi-stage Dockerfile (musl-static + Alpine), OCI labels, healthcheck |
| Universal IPC v3 | JSON-RPC + tarpc, semantic naming |
| BTSP Phase 2 | Server-side handshake enforcement on UDS accept |
| Capability Wire L3 | Composable: provided/consumed capabilities, cost estimates, dependencies |
| unsafe_code = "deny" | Workspace-wide, zero unsafe blocks |
| AGPL-3.0-or-later | SPDX headers on all 167 `.rs` files |

## Metrics

| Metric | Value |
|--------|-------|
| Tests | 1,540 passing (all features) |
| Coverage | 93.88% lines (CI gate: 90%) |
| Clippy | 0 warnings (pedantic + nursery + cargo + cast lints enforced, `doc_markdown` enforced, `unwrap_used`/`expect_used = "deny"`) |
| Source files | 167 `.rs`, ~49,700 lines |
| Max file size | 724 lines (limit: 1000) |
| Binary size | 5.7 MB (musl-static, stripped, PIE) |
| Fuzz targets | 3 (merkle, session builder, vertex CBOR) |
| Chaos tests | 5 suites (discovery, stress, injection, partition, exhaustion) |

## Storage Backends

- **redb** (default) — Pure Rust, ACID, MVCC, ecoBin compliant
- **In-memory** — Testing and ephemeral workloads

## Key Files

- `Cargo.toml` — Workspace config, lint policy, dependency pins
- `capability_registry.toml` — Capability method registry (28 methods in `METHOD_CATALOG`, 5 domains)
- `deny.toml` — Supply chain audit (ecoBin ban list, advisories, licenses)
- `specs/` — 12 specification documents (incl. `CRYPTO_MODEL.md` — signing provider crypto delegation)
- `showcase/` — 65 progressive demos

## Part of ecoPrimals

Part of the [ecoPrimals](https://github.com/ecoPrimals) sovereign computing
ecosystem. See [wateringHole](https://github.com/ecoPrimals/wateringHole) for
ecosystem standards, primal registry, and inter-primal interaction documentation.
