# rhizoCrypt — AI Context Block

**Version**: 0.14.17
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
| `rhizo-crypt-rpc` | tarpc service (28 ops), JSON-RPC 2.0 handler (39 methods, 7 domains), NDJSON streaming, rate limiting, batch dehydrate/ingest |
| `rhizocrypt-service` | UniBin binary (`server`, `client`, `status`, `version`, `doctor`) |

## IPC

- **G66 transport abstraction** — `TransportEndpoint` (UDS/TCP/MeshRelay), `TransportStream` (AsyncRead+AsyncWrite), `TransportListener` (server-side); platform-neutral IPC, `#[cfg(unix)]` confined to transport layer; `platform_default()` + `from_env_or_default()` for transport injection
- **G65 protocol negotiation** — single-socket (`rhizocrypt.sock`), client sends `PROTOCOLS: tarpc,jsonrpc\n`, server selects best match; generic over any `AsyncRead + AsyncWrite` stream (G66)
- **UDS dual-socket (G64 C2)** — JSON-RPC on `rhizocrypt.sock`, tarpc binary on `rhizocrypt.tarpc.sock` (retained for backward compat)
- **TCP opt-in** via `--port` or `RHIZOCRYPT_PORT` env var (tarpc + JSON-RPC dual-mode)
- **JSON-RPC 2.0** — dual-mode TCP (auto-detects HTTP POST vs newline-delimited) + UDS
- **tarpc 0.37** with bincode — UDS (sub-ms) and TCP, high-performance typed RPC
- **BTSP Phase 2+3** — X25519 + HMAC-SHA256 handshake + ChaCha20-Poly1305 encrypted channel on UDS; server-side auto-detect + client-side `BtspUnixAdapter` for outbound bearDog connections; `btsp.negotiate` upgrades to AEAD framing; dev mode (`BIOMEOS_INSECURE=1`) bypasses
- **G63 Local-Trust** — `SO_PEERCRED` peer credential extraction on UDS connections; `CallerContext` carries kernel-verified UID/GID/PID; `TransportStream::supports_peer_cred()` + `TransportEndpoint::is_local()` for trust decisions
- Method names follow `domain.verb` semantic naming (`dag.session.create`, `health.check`)

## Compliance

| Standard | Status |
|----------|--------|
| UniBin | Single binary, clap subcommands |
| ecoBin v3.0 | Zero application C deps, zero reqwest, cross-compile (musl, RISC-V) |
| genomeBin | Multi-stage Dockerfile (musl-static + scratch), OCI labels, healthcheck |
| Universal IPC v3 | JSON-RPC + tarpc, semantic naming |
| BTSP Phase 2+3 | Server-side auto-detect + client-side `ClientHello` handshake + ChaCha20-Poly1305 encrypted channel via `btsp.negotiate` |
| Capability Wire L3 | Composable: provided/consumed capabilities, cost estimates, dependencies |
| unsafe_code = "deny" | Workspace-wide, zero unsafe blocks |
| G68 Platform Substrate | **COMPLIANT** — `platform_link()`, `PlatformAccess`, `is_symlink_to()` in `transport/platform.rs`; zero raw platform APIs outside transport layer |
| AGPL-3.0-or-later | SPDX headers on all 223 `.rs` files |

## Metrics

| Metric | Value |
|--------|-------|
| Tests | 1,825 passing (all features, Aug 7 2026) |
| Coverage | 93.83% lines (llvm-cov, Jul 18 2026) |
| Clippy | 0 warnings (pedantic + nursery + cargo + cast lints enforced, `doc_markdown` enforced, `unwrap_used`/`expect_used = "deny"`, zero unfulfilled `--tests`) |
| Source files | 223 `.rs`, ~61,400 lines |
| Max file size | ~624 lines production (`store.rs`, limit: 800) |
| Binary size | 5.7 MB (musl-static, stripped, PIE) |
| Fuzz targets | 3 (merkle, session builder, vertex CBOR) |
| Chaos tests | 5 suites (discovery, stress, injection, partition, exhaustion) |

## Storage Backends

- **redb** (default) — Pure Rust, ACID, MVCC, ecoBin compliant
- **In-memory** — Testing and ephemeral workloads

## Key Files

- `Cargo.toml` — Workspace config, lint policy, dependency pins
- `config/capability_registry.toml` — Capability method registry (39 methods in `METHOD_CATALOG`, 7 domains)
- `deny.toml` — Supply chain audit (ecoBin ban list, advisories, licenses)
- `specs/` — 10 specification documents + 2 archived (incl. `CRYPTO_MODEL.md` — signing provider crypto delegation)
## Part of ecoPrimals

Part of the [ecoPrimals](https://github.com/ecoPrimals) sovereign computing
ecosystem. See [wateringHole](https://github.com/ecoPrimals/wateringHole) for
ecosystem standards, primal registry, and inter-primal interaction documentation.
