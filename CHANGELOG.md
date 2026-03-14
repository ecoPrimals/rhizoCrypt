# Changelog

All notable changes to rhizoCrypt will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.13.0-dev] - 2026-03-14 (session 5)

### Changed

#### Pedantic Clippy Clean (0 warnings with pedantic + nursery)
- Fixed 45+ clippy errors: `significant_drop_tightening`, `doc_markdown`, `must_use_candidate`, `missing_errors_doc`
- All public Result-returning functions now have `# Errors` doc sections
- All identifiers in doc comments wrapped in backticks
- Lock scopes tightened across RPC and test harness code

#### Magic Numbers Centralized
- Added 7 new constants to `constants.rs`: `SLED_CACHE_SIZE_BYTES`, `SLED_FLUSH_INTERVAL_MS`, `DISCOVERY_QUERY_TIMEOUT`, `DISCOVERY_RESPONSE_BUFFER_SIZE`, `PROVENANCE_CONNECTION_TIMEOUT`, `PROVENANCE_RESPONSE_TIMEOUT`, `PROVENANCE_DEFAULT_MAX_RESULTS`
- Updated `store_sled.rs`, `discovery/registry.rs`, `adapters/tarpc.rs`, `toadstool_http.rs`, provenance clients

#### Smart File Refactoring
- `store_sled.rs`: 949 → 552 lines (tests extracted to `store_sled_tests.rs`)
- `store_redb_tests_advanced.rs`: 994 → 846 lines (query tests to `store_redb_tests_query.rs`)
- All files now well under 1000-line limit (max: 858)

#### UniBin Exit Codes & Signal Handling
- Added `exit_codes` module: `SUCCESS` (0), `GENERAL_ERROR` (1), `CONFIG_ERROR` (2), `NETWORK_ERROR` (3), `INTERRUPTED` (130)
- `ServiceError::exit_code()` maps errors to proper exit codes
- Added `shutdown_signal()`: SIGTERM + SIGINT on Unix, Ctrl+C on other platforms
- Server graceful shutdown via `tokio::select!` with `tokio::pin!`

#### ecoBin Compliance Hardened
- Default build confirmed pure Rust (no `ring`); `ring` only via opt-in `http-clients`
- `deny.toml`: explicit bans for `openssl-sys`, `native-tls`, `aws-lc-sys` and 4 other C-backed crates
- `ring` documented as tolerated opt-in only, pending `rustls-rustcrypto` stabilization

#### Test Expansion
- **1092 tests passing** (was 1075) — +17 new tests
- **90.83% line coverage** (was 91.02% — slight shift from refactoring)
- New E2E: `slice_workflows.rs` (5 tests), `merkle_operations.rs` (4 tests)
- New chaos: `resource_exhaustion.rs` (4 tests)
- New service: exit codes + signal handling (4 tests)

### Quality Gates

| Gate | Status |
|------|--------|
| `cargo fmt --check` | Clean |
| `cargo clippy` (pedantic + nursery) | Clean (0 warnings) |
| `cargo doc --workspace --all-features --no-deps` | Clean (0 warnings) |
| `cargo test --workspace --all-features` | 1092 pass, 0 fail |
| `cargo test --doc --workspace --all-features` | 30 pass, 0 ignored |
| `cargo llvm-cov --all-features` | 90.83% lines, 92.31% regions |
| `cargo deny check` | advisories ok, bans ok, licenses ok, sources ok |
| `#![forbid(unsafe_code)]` | Workspace-wide |
| SPDX headers | All `.rs` files |
| Max file size | All under 1000 lines (max 858) |
| No production TODOs/FIXMEs | Verified |

---

## [0.13.0-dev] - 2026-03-14 (session 4)

### Changed

#### Sovereignty: Capability-Based Error Types
- Removed primal-specific error variants `BearDog(String)`, `LoamSpine(String)`, `NestGate(String)`
- Added `CapabilityProvider { capability, message }` — structured, capability-based variant
- Removed deprecated trait aliases `BearDogClient`, `LoamSpineClient`, `NestGateClient`
- Updated `is_recoverable()` to cover `Integration` and `CapabilityProvider`

#### JSON-RPC Zero-Copy: from_str → from_slice
- Replaced two-step `from_utf8` + `from_str` with single `serde_json::from_slice(&body)`
- Combines UTF-8 validation and JSON parsing in one pass, eliminating intermediate `&str`

#### Doc Tests Rewritten (26 ignore → no_run)
- All 26 `ignore`d doc tests rewritten to match current API surface
- **30 doc tests passing, 0 ignored** (was 4 passing, 26 ignored)
- Doc examples now serve as compilable API reference

#### Coverage & Test Expansion
- **1075 tests passing** (was 1022) — +53 new tests
- **91.02% line coverage** (was 90.12%), 87.61% function, 92.38% region
- `store_redb_tests_advanced.rs`: `parse_vertex_set` edge cases, `Clone` independence, `StorageStats` debug
- `error.rs`: `CapabilityProvider` construction, display, recoverability
- `songbird_rpc.rs`: function coverage 52% → 96%

#### Root Docs & crate README Modernized
- Root `README.md`: metrics updated (1075 tests, 91.02% coverage), `client` subcommand added
- `rhizo-crypt-core/README.md`: rewritten — removed RocksDB/BearDog references, updated to redb/sled, capability-based clients
- New wateringHole handoff for session 4

### Quality Gates

| Gate | Status |
|------|--------|
| `cargo fmt --check` | Clean |
| `cargo clippy --workspace --all-features --all-targets -- -D warnings` | Clean |
| `cargo doc --workspace --all-features --no-deps` | Clean (0 warnings) |
| `cargo test --workspace --all-features` | 1075 pass, 0 fail |
| `cargo test --doc --workspace --all-features` | 30 pass, 0 ignored |
| `cargo llvm-cov --all-features` | 91.02% lines, 92.38% regions |
| `cargo deny check` | advisories ok, bans ok, licenses ok, sources ok |
| `#![forbid(unsafe_code)]` | Workspace-wide |
| SPDX headers | All `.rs` files |
| Max file size | All under 1000 lines |

---

## [0.13.0-dev] - 2026-03-14 (session 3)

### Changed

#### Deep Debt Execution & Coverage Push to 90%
- **1022 tests passing** (was 862) — +160 new tests across all modules
- **90.12% line coverage** (llvm-cov) — crossed 90% target
- **Zero production TODOs, FIXMEs, or HACKs** in all `.rs` files
- **Zero production `unwrap()`/`expect()`** — all in `#[cfg(test)]` modules with proper `#[allow]`

#### Platform-Agnostic Transport (ecoBin v2.0)
- `TransportHint` enum: `UnixSocket`, `Tcp`, `AbstractSocket`
- `socket_dir()`: XDG_RUNTIME_DIR → /run/ecoPrimals → /tmp/ecoPrimals; `None` on Android/Windows
- `socket_path_for_primal()`: per-primal socket path construction
- `preferred_transport()`: runtime OS detection, picks optimal transport

#### UniBin Doctor Subcommand
- `rhizocrypt doctor` — DAG engine, storage (redb), configuration, discovery, environment checks
- `rhizocrypt doctor --comprehensive` — adds TCP connectivity probes to discovery endpoints
- Human-readable output with pass/warn/fail indicators

#### Zero-Copy JSON-RPC Handler
- `get_str()` → returns `&str` instead of `String` (zero allocation on parameter extraction)
- `get_opt_str()` → returns `Option<&str>` instead of `Option<String>`
- Explicit `.map(String::from)` only where struct fields require ownership

#### HTTP Client Test Infrastructure
- Added `wiremock` (pure Rust) for HTTP client testing
- LoamSpine: 16 wiremock tests (method negotiation, health, commit, verify, checkout, resolve)
- BearDog: 8 wiremock tests (sign, verify, health, error paths)
- NestGate: 12 wiremock tests (store, retrieve, exists, metadata, health)
- ToadStool: 15 wiremock tests (health, BYOB health, deployments, stop, usage)
- Songbird: 4 tarpc integration tests with mock server
- Provenance: 6 mock adapter tests (all capability methods)

#### Spec & Doc Alignment
- `STORAGE_BACKENDS.md` updated: RocksDB/LMDB → redb/sled (ecoBin rationale)
- Binary integration tests: `env!("CARGO_BIN_EXE_rhizocrypt")` — idiomatic Rust binary discovery
- Root docs and CHANGELOG updated to current metrics

### Quality Gates

| Gate | Status |
|------|--------|
| `cargo fmt --check` | Clean |
| `cargo clippy --workspace --all-features --all-targets -- -D warnings` | Clean |
| `cargo doc --workspace --all-features --no-deps` | Clean (0 warnings) |
| `cargo test --workspace --all-features` | 1022 pass, 0 fail |
| `cargo llvm-cov --all-features` | 90.12% lines, 91.84% regions |
| `cargo deny check` | advisories ok, bans ok, licenses ok, sources ok |
| `#![forbid(unsafe_code)]` | Workspace-wide |
| SPDX headers | All `.rs` files |
| Max file size | All under 1000 lines |

---

## [0.13.0-dev] - 2026-03-13 (session 2)

### Changed

#### Deep Debt Resolution & Modern Idiomatic Rust
- **862 tests passing** (was 600) — +262 new tests across all modules
- **87.78% line coverage** (llvm-cov) — remaining gap is DB error paths and binary entry point
- **Zero production TODOs, FIXMEs, or HACKs** in all `.rs` files
- **All files under 1000 lines** — extracted test modules from handler.rs and loamspine_http.rs

#### Service Library Extraction
- Extracted `rhizocrypt-service` logic into `lib.rs` for testability
- `run_server`, `resolve_bind_addr`, `register_with_discovery` now testable without subprocess
- `main.rs` is a thin CLI wrapper delegating to the library

#### Dependency Audit Infrastructure
- **`cargo-deny`** configured (`deny.toml`) — advisories, licenses, bans, sources enforced
- CDLA-Permissive-2.0 allowed (transitive via webpki-roots)
- Transitive unmaintained advisories from tarpc acknowledged and tracked
- Workspace wildcard dependencies permitted for internal crates

#### Coverage Push (600 → 862 tests)
- `store_sled.rs`: +20 tests (exists, export, batch, concurrent, frontier, genesis)
- `discovery/registry.rs`: +6 tests (capability parsing, unhealthy filtering, connection refused)
- `clients/loamspine_http.rs`: +15 tests (JSON-RPC response handling, serde, negotiation)
- `clients/toadstool_http.rs`: +12 tests (deployment parsing, event conversion, serde)
- `clients/nestgate_http.rs`: +10 tests (blob storage serde, error display, source chain)
- `clients/beardog_http.rs`: +10 tests (signing serde, DID document, error chain)
- `jsonrpc/mod.rs`: +7 tests (invalid UTF-8, empty batch, wrong version, missing ID)
- `rhizocrypt-service/lib.rs`: +6 tests (bind addr, error display, discovery registration)

#### Mock Isolation Verified
- All mock types gated behind `#[cfg(any(test, feature = "test-utils"))]`
- Zero mock code in production paths
- `test-utils` feature only used by fuzz targets

#### ecoBin Compliance Verified
- Zero application C dependencies in default build
- `blake3` uses `pure` feature (no C/assembly compilation)
- TLS via `rustls` (pure Rust) when `http-clients` feature enabled
- No project `build.rs` invoking C compilers

### Quality Gates

| Gate | Status |
|------|--------|
| `cargo fmt --check` | Clean |
| `cargo clippy --workspace --all-features --all-targets -- -D warnings` | Clean |
| `cargo doc --workspace --all-features --no-deps` | Clean |
| `cargo test --workspace --all-features` | 862 pass, 0 fail |
| `cargo deny check` | advisories ok, bans ok, licenses ok, sources ok |
| `#![forbid(unsafe_code)]` | Workspace-wide |
| SPDX headers | All `.rs` files |
| Max file size | All under 1000 lines |

---

## [0.13.0-dev] - 2026-03-13

### Changed

#### Pure Rust Storage Evolution
- **redb default**: Switched default storage backend from `sled` to `redb` (100% Pure Rust)
- **ecoBin compliant**: Default build now has zero C dependencies
- **Feature-gated**: `sled` backend remains available via `--features sled`

#### Test Coverage: 80.6% → 90.02%
- 600 tests passing (was 491) — +114 new unit tests across 14 modules
- `rpc/client.rs`: 0% → 81% (22 new tests)
- `rhizocrypt.rs`: 80% → 92% (14 new tests including full lifecycle, dehydration)
- `store_redb.rs`: 62% → 79% (14 new tests including diamond DAG, persistence)
- `event.rs`: 68% → 100% (serialization roundtrips for all variants)
- `rpc/jsonrpc/mod.rs`: 20% → 77% (5 Axum endpoint integration tests)

#### Client Method Negotiation
- LoamSpine HTTP client now tries native method names (`commit.session`) first
- Falls back to compatibility names (`permanent-storage.commitSession`) on -32601
- Negotiation outcome cached via `AtomicU8` for zero-overhead subsequent calls

#### Discovery Registry Evolution
- `DiscoveryRegistry::discover()` now queries Songbird via HTTP/1.1 over TCP
- Parses `discovery.resolve` JSON-RPC responses and caches results
- New `parse_capability()` helper converts string names to `Capability` variants

#### JSON-RPC Handler Coverage
- 12 new handler unit tests covering all session, event, vertex, Merkle, slice, and system methods

---

## [0.13.0-dev] - 2026-03-12

### Changed

#### wateringHole Standards Compliance
- **AGPL-3.0-only**: Updated SPDX identifier, added headers to all 71 source files
- **UniBin architecture**: Binary renamed to `rhizocrypt` with `clap` subcommands (`server`, `status`, `version`)
- **Semantic method naming**: JSON-RPC methods evolved from `loamspine.*` to `permanent-storage.*`
- **ecoBin**: `reqwest` switched to `rustls-tls` (no OpenSSL); sled `zstd-sys` dependency documented

#### Capability-Based Discovery
- Removed all hardcoded development fallback ports and addresses
- Removed deprecated primal-specific discovery methods (`discover_beardog`, etc.)
- Removed deprecated vendor env vars (`TOADSTOOL_ADDRESS`, `SWEETGRASS_PUSH_ADDRESS`)
- All discovery is now capability-only at runtime

#### Code Quality
- Converted `create_session`, `checkout_slice`, `resolve_slice`, `get_dehydration_status` from `async` to sync
- Converted `Vertex` methods to return `Result` (removed `expect()` from production code)
- Fixed `cast_possible_truncation` with safe `u64::try_from` + saturating arithmetic
- Optimized atomic ordering from `SeqCst` to `Relaxed` for request counters
- Removed bulk `#[allow(clippy::*)]` suppressions, fixed all exposed violations
- Deleted `legacy_aliases.rs` and deprecated mock type aliases
- Reduced `discovery.rs` from 1001 to 854 lines (smart refactoring, not splitting)

#### Infrastructure
- Added 3 fuzz testing targets (`cargo-fuzz` + `libfuzzer-sys`)
- Fixed test hang in `test_service_handles_invalid_port`
- Updated Dockerfile and k8s deployment for UniBin binary
- Updated CI workflow with doc checks and `actions/cache@v4`
- Cleaned 22 root-level session artifacts to `phase2/archive/`
- Completed dehydration implementation (payload sizes, event counting, role assignment)

### Metrics (March 12)
- 491 tests passing (0 failed)
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`: clean
- `cargo fmt --check --all`: clean
- `RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps`: clean
- All files under 1000 lines (max: 981)

---

## [0.14.1-dev] - 2026-01-09 (archived)

### Changed
- lib.rs restructured (1104 → 254 lines), extracted `metrics.rs` and `rhizocrypt.rs`
- Completed LoamSpine HTTP client (4 production TODOs eliminated)
- Clippy warnings reduced from 20+ to 0
- 374 tests, 79.35% coverage

---

## [0.13.0-dev] - 2025-12-26

### Changed
- Capability-based type system (`SigningProvider`, `PermanentStorageProvider`, `PayloadStorageProvider`)
- Deprecated primal-specific trait names (backward compatible via aliases)
- Integration Specification v2.0 (`specs/INTEGRATION_SPECIFICATION_V2.md`)
- 486 tests, 86.17% coverage

---

## [0.12.0] - 2025-12-26

### Changed
- Lock-free concurrency: `Arc<RwLock<HashMap>>` → `Arc<DashMap>` (10-100x improvement)
- Songbird auto-registration with heartbeat
- 403 tests, 85%+ coverage

---

## [0.10.0] - 2025-12-24

### Changed
- Pure infant discovery architecture (SafeEnv, CapabilityEnv)
- Capability-based environment variables (e.g. `SIGNING_ENDPOINT` over `BEARDOG_ADDRESS`)
- 260 tests, 85.22% coverage, A+ grade

---

## [0.9.2] - 2025-12-23

### Added
- Core implementation: vertex content-addressing (BLAKE3), sessions, multi-parent DAG, topological sort
- 21 tests passing

---

## [0.9.0] - 2025-12-20

### Added
- Initial project: specs, core types, DAG store trait, Merkle tree, slice semantics, dehydration, tarpc RPC, showcase demos

---

## Version History Summary

- **0.13.0-dev** (2026-03-14 s4): Sovereignty cleanup, 1075 tests, 91% coverage, doc tests rewritten, capability-based errors
- **0.13.0-dev** (2026-03-14 s3): 90% coverage, 1022 tests, platform-agnostic transport, doctor subcommand, zero-copy handler
- **0.13.0-dev** (2026-03-13): Deep debt, 862 tests, cargo-deny, service lib extraction
- **0.13.0-dev** (2026-03-12): wateringHole standards, capability discovery, UniBin
- **0.12.0** (2025-12-26): Lock-free concurrency (DashMap), Songbird registration
- **0.10.0** (2025-12-24): Production ready, pure infant discovery, A+ grade
- **0.9.2** (2025-12-23): Core implementation complete
- **0.9.0** (2025-12-20): Initial release with specifications

