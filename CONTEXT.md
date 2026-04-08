# rhizoCrypt — AI Context Block

**Version**: 0.14.0-dev
**Role**: Ephemeral DAG Engine — working memory for the ecoPrimals ecosystem
**License**: AGPL-3.0-or-later / ORC / CC-BY-SA 4.0 (scyBorg Triple-Copyleft)
**Language**: Rust 2024, edition 2024, rust-version 1.87

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

- **JSON-RPC 2.0** — dual-mode TCP (auto-detects HTTP POST vs newline-delimited) + UDS
- **tarpc 0.37** with bincode — optional, high-performance typed RPC
- **Unix domain socket** at `$XDG_RUNTIME_DIR/biomeos/rhizocrypt.sock` (`--unix` flag)
- Method names follow `domain.verb` semantic naming (`dag.session.create`, `health.check`)

## Compliance

| Standard | Status |
|----------|--------|
| UniBin | Single binary, clap subcommands |
| ecoBin v3.0 | Zero application C deps, zero reqwest, cross-compile (musl, RISC-V) |
| genomeBin | Multi-stage Dockerfile (musl-static + Alpine), OCI labels, healthcheck |
| Universal IPC v3 | JSON-RPC + tarpc, semantic naming |
| unsafe_code = "deny" | Workspace-wide, zero unsafe blocks |
| AGPL-3.0-or-later | SPDX headers on all 136 `.rs` files |

## Metrics

| Metric | Value |
|--------|-------|
| Tests | 1,441 passing (all features) |
| Coverage | 94.34% lines, 93.41% functions, 94.81% branches (CI gate: 90%) |
| Clippy | 0 warnings (pedantic + nursery + cargo + cast lints enforced, `doc_markdown` enforced, `unwrap_used`/`expect_used = "deny"`) |
| Source files | 136 `.rs`, ~46,200 lines |
| Max file size | 928 lines (limit: 1000) |
| Binary size | 5.4 MB (musl-static, stripped, PIE) |
| Fuzz targets | 3 (merkle, session builder, vertex CBOR) |
| Chaos tests | 5 suites (discovery, stress, injection, partition, exhaustion) |

## Storage Backends

- **redb** (default) — Pure Rust, ACID, MVCC, ecoBin compliant
- **In-memory** — Testing and ephemeral workloads

## Key Files

- `Cargo.toml` — Workspace config, lint policy, dependency pins
- `capability_registry.toml` — Capability method registry (24 declared, 28 total via handler; 5 domains)
- `deny.toml` — Supply chain audit (ecoBin ban list, advisories, licenses)
- `specs/` — 10 specification documents
- `showcase/` — 70+ progressive demos

## Part of ecoPrimals

Part of the [ecoPrimals](https://github.com/ecoPrimals) sovereign computing
ecosystem. See [wateringHole](https://github.com/ecoPrimals/wateringHole) for
ecosystem standards, primal registry, and inter-primal interaction documentation.
