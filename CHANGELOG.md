# Changelog

All notable changes to rhizoCrypt will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.13.0-dev] - 2026-03-17 (session 18)

### Changed

#### Deep Debt Execution — Sovereignty, Refactoring, Cross-Compile CI, JSON-RPC Port, Docs Refresh

**1. Primal Sovereignty — `provenance-trio-types` Eliminated**
- Removed compile-time dependency on shared `provenance-trio-types` crate (cross-primal boundary violation)
- Created `dehydration_wire.rs` — rhizoCrypt-owned outbound wire types (`DehydrationWireSummary`, `WireAgentRef`, `WireAttestationRef`, `WireOperationRef`)
- The JSON schema on the wire is the contract, not a shared Rust crate
- Each primal now owns its own serialization types — zero cross-primal compile-time coupling
- Resolves primalSpring coordination issue for all trio workspaces

**2. Smart File Refactoring — `validation.rs` Module Extraction**
- Extracted `ValidationHarness`, `ValidationSink`, `StderrSink`, `StringSink` from `error.rs` (863 → 660 lines) to canonical `validation.rs` (236 lines)
- New module declared in `lib.rs`, re-exported from both `error` and root for backward compatibility

**3. Smart File Refactoring — `registry_tests.rs` Extraction**
- Extracted all `#[cfg(test)]` tests from `discovery/registry.rs` (886 → 399 lines) to `registry_tests.rs` (452 lines)
- Uses `#[path]` attribute for clean separation while preserving module-private access

**4. `RHIZOCRYPT_JSONRPC_PORT` Configuration**
- Added `JSONRPC_PORT_OFFSET` constant to `constants.rs` (default: tarpc port + 1)
- Added `SafeEnv::get_jsonrpc_port(tarpc_port)` to `safe_env/mod.rs` with env override
- `rhizocrypt-service` `run_server` now uses `SafeEnv::get_jsonrpc_port()` instead of hardcoded offset

**5. Cross-Compile CI Job (ecoBin v3.0)**
- Added `cross-compile` matrix job to `.github/workflows/ci.yml`
- Targets: `x86_64-unknown-linux-musl`, `aarch64-unknown-linux-musl`, `riscv64gc-unknown-linux-gnu`
- Uses `cross` for cross-compilation, validates default-features-only builds

**6. Dependency Evolution Documentation**
- Documented `ring` → `rustls-rustcrypto`, `sled` → `redb`, tarpc transitive debt in `deny.toml`
- Documented `sled` deprecation status in `rhizo-crypt-core/Cargo.toml`

**7. Lint Cleanup**
- Removed unfulfilled `#[expect(clippy::unwrap_used)]` from `songbird/client.rs` test module

**8. `deny.toml` Full ecoBin v3.0 Ban List**
- Expanded from 7 to 16 banned C-sys crates: added `openssl-src`, `cmake`, `cc`, `bindgen`, `bzip2-sys`, `curl-sys`, `libz-sys`, `pkg-config`, `vcpkg`
- `cc` allowed as wrapper for `ring` and `blake3` (build-time only, not runtime)
- Resolved RUSTSEC-2026-0049: updated `rustls-webpki` 0.103.8 → 0.103.10
- `cargo deny check` — advisories ok, bans ok, licenses ok, sources ok

**9. Smart File Refactoring — 3 More Extractions**
- `nestgate_http.rs`: 729 → 325 lines (tests → `nestgate_http_tests_wiremock.rs`, 407 lines)
- `signing.rs`: 758 → 408 lines (tests → `signing_tests.rs`, 353 lines)
- `niche.rs`: 732 → 514 lines (tests → `niche_tests.rs`, 221 lines)
- All files now well under 1000-line limit (max: 867)

**10. Ecosystem Documentation Updates**
- Updated `PRIMAL_REGISTRY.md`: 1330 tests, 92.32% coverage, 14-crate ban list, cross-compile CI, sovereign wire types
- Updated `genomeBin/manifest.toml`: `pie_verified = true`, UniBin binary description
- Updated README: 128 SPDX files, 14-crate ecoBin ban list
- Cleaned stale `showcase/00_START_HERE.md` stats and dead references
- Wrote wateringHole handoffs for primalSpring resolution and sessions 17–18

### Quality Gates

- `cargo fmt` — clean
- `cargo clippy --workspace --all-targets --all-features -- -D warnings` — 0 warnings
- `cargo doc --workspace --all-features --no-deps` — 0 warnings (`-D warnings`)
- `cargo test --workspace --all-features` — **1,330 tests passing**, 0 failures
- `cargo deny check` — advisories ok, bans ok, licenses ok, sources ok
- All `.rs` files under 1000 lines (max: 867)

---

## [0.13.0-dev] - 2026-03-17 (session 17)

### Changed

#### Deep Debt Execution — Health Probes, 4-Format Capabilities, ValidationSink, JSON-RPC Fuzz

**1. `health.liveness` + `health.readiness` JSON-RPC Methods** (ecosystem convergence pattern)
- Added zero-cost liveness probe (`{ "alive": true }`) — Kubernetes/biomeOS compatible
- Added readiness probe (checks primal health state, returns version + primal ID)
- Registered in CAPABILITIES, COST_ESTIMATES, SEMANTIC_MAPPINGS, CAPABILITY_DOMAINS
- Wired into JSON-RPC handler dispatch table

**2. `deny.toml` Yanked Crate Hardening** (absorbed from wateringHole standard)
- Added `yanked = "deny"` to `[advisories]` — yanked crates now fail CI

**3. 4-Format Capability Parsing** (absorbed from airSpring v0.8.7)
- Format A: flat string array `["dag.session.create"]`
- Format B: nested objects `[{"name": "dag.session.create"}]`
- Format C: wrapper object `{"capabilities": [...]}` (biomeOS/neuralSpring)
- Format D: double-nested `{"capabilities": [{"name": "..."}]}` (toadStool S155+)
- Also handles `{"methods": [...]}` wrapper (coralReef variant)
- Exported as `discovery::extract_capabilities()`

**4. `ValidationSink` Trait** (absorbed from ludoSpring V22)
- Pluggable output trait for `ValidationHarness` — redirect to JSON, files, buffers
- `StderrSink` (default), `StringSink` (testing)
- `finish_to(sink)` method on `ValidationHarness`
- `checks()` accessor for programmatic inspection

**5. JSON-RPC Proptest Fuzz** (7 new property tests)
- `prop_jsonrpc_request_roundtrip` — valid method/id parsing
- `prop_jsonrpc_error_any_code` — error extraction across full code range
- `prop_jsonrpc_success_no_error` — success responses never extract errors
- `prop_ipc_phase_mutual_exclusion` — retriable and application error are disjoint
- `prop_validation_harness_counts` — pass + fail always equals total
- `prop_validation_sink_captures` — sink output matches harness state

**6. 4-Format Capability Parsing Proptest** (4 new property tests)
- `prop_capabilities_format_a` — flat strings roundtrip
- `prop_capabilities_format_b` — nested object extraction
- `prop_capabilities_format_c` — wrapper object extraction
- `prop_capabilities_format_d` — double-nested extraction

### Quality Gates

- `cargo fmt` — clean
- `cargo clippy --workspace --all-targets -- -D warnings` — 0 warnings
- `cargo doc --workspace --no-deps` — 0 warnings
- `cargo test --workspace` — **1,102 tests passing**, 0 failures
- `cargo deny check` — advisories ok, bans ok, licenses ok, sources ok

---

## [0.13.0-dev] - 2026-03-16 (session 16)

### Changed

#### Deep Debt Execution — Ecosystem Absorption, Lint Migration, Manifest Discovery, Chaos Framework

**1. `#[expect(reason)]` Lint Migration** (ecosystem-wide standard)
- Migrated all crate-level lint attributes to workspace `Cargo.toml` (`[workspace.lints.clippy]`)
- Removed redundant `#![warn(clippy::all/pedantic/nursery)]` from `lib.rs` (workspace handles these)
- Added `missing_errors_doc`, `missing_panics_doc`, `field_reassign_with_default`, `unnecessary_literal_bound`, `similar_names` to workspace `"allow"` config
- Test/bench files use `#[allow]` (blanket policy), production code uses `#[expect(reason)]` (stale suppressions auto-surface)

**2. Generic Socket/Address Env Var Helpers** (absorbed from sweetGrass V0717)
- `SafeEnv::socket_env_var(primal_name)` → `{PRIMAL}_SOCKET`
- `SafeEnv::address_env_var(primal_name)` → `{PRIMAL}_ADDRESS`
- `SafeEnv::get_socket_path(primal_name)` → checks env, falls back to `$XDG_RUNTIME_DIR/ecoPrimals/{name}.sock`
- Eliminates per-primal constant proliferation

**3. Manifest-Based Discovery** (absorbed from toadStool S156 / barraCuda v0.3.5)
- New `discovery::manifest` module for `$XDG_RUNTIME_DIR/ecoPrimals/*.json` scanning
- `PrimalManifest` struct: primal, version, socket, address, capabilities
- `scan_manifests()`, `discover_by_capability()`, `publish_manifest()`, `unpublish_manifest()`
- Filesystem-based discovery fallback when Songbird is unavailable

**4. `ValidationHarness`** (absorbed from wetSpring V123 `Validator::finish_with_code()`)
- Composable validation harness that collects all failures before deciding exit code
- `check(name, passed)`, `all_passed()`, `pass_count()`, `fail_count()`, `finish()` with summary
- Suitable for `rhizocrypt doctor` and `rhizocrypt validate` subcommands

**5. Chaos Testing Framework** (absorbed from squirrel ChaosEngine pattern)
- New `testing::chaos` module with `ChaosEngine`, `ChaosConfig`, `ChaosScenario` trait
- 7 `FaultClass` variants: `NetworkPartition`, `Latency`, `ProcessCrash`, `ResourceExhaustion`, `ClockSkew`, `ConcurrencyStorm`, `CorruptInput`
- `ChaosMetrics` with survival rate, error/recovery counts, duration tracking
- Engine filters scenarios by enabled fault classes, respects max duration budget

### Quality Gates

- `cargo fmt` — clean
- `cargo clippy --workspace --all-targets -- -D warnings` — 0 warnings
- `cargo doc --workspace --no-deps` — 0 warnings
- `cargo test --workspace` — **1,080 tests passing**, 0 failures
- `cargo deny check` — advisories ok, bans ok, licenses ok, sources ok

---

## [0.13.0-dev] - 2026-03-16 (session 15)

### Changed

#### Deep Debt Execution — Resilience, DispatchOutcome, OrExit, Dual-Format Discovery, Proptest

**1. IpcErrorPhase Convenience Helpers**
- Added `is_method_not_found()`, `is_timeout_likely()`, `is_retriable()`, `is_application_error()` to `IpcErrorPhase`
- Enables targeted retry/escalation decisions without manual pattern-matching

**2. DispatchOutcome Enum** (absorbed from airSpring / biomeOS dispatch patterns)
- New `DispatchOutcome<T>` — separates protocol errors from application results
- `Ok(T)`, `ApplicationError { code, message }`, `ProtocolError(RhizoCryptError)`
- `into_result()` folds both error variants for callers that don't need the distinction

**3. extract_rpc_error() Centralized Parser**
- Extracts `(code, message)` from JSON-RPC error objects — used by every IPC adapter
- Replaces inline error extraction in `unix_socket.rs` `parse_json_rpc_response()`

**4. OrExit<T> Trait** (absorbed from wetSpring V123)
- Zero-panic validation binaries — `or_exit(context)` prints structured error + exits
- Implemented for `Result<T, E: Display>` and `Option<T>`

**5. Dual-Format Capability Parsing** (absorbed from groundSpring / neuralSpring / airSpring / wetSpring)
- Discovery response now handles flat strings (`["Signing"]`) and nested objects (`[{"name": "Signing", "version": "1.0"}]`)
- Custom serde `Visitor` implementation for `DiscoveredEndpoint.capabilities`
- `parse_capability()` now accepts colon-delimited names (`crypto:signing`, `did:verification`, etc.)

**6. CircuitBreaker + RetryPolicy** (absorbed from healthSpring V28 / airSpring V15)
- New `clients/resilience.rs` module with transport-agnostic resilience primitives
- `CircuitBreaker`: consecutive-failure threshold → open → cooldown → half-open probe
- `RetryPolicy`: exponential backoff with configurable max, `should_retry()` gates by `IpcErrorPhase`
- Only transport-level failures (Connect, Write, Read) are retriable; application errors pass through

**7. Proptest Roundtrip Coverage**
- New property tests: `capability_list()` JSON roundtrip, `IpcErrorPhase` invariants,
  `extract_rpc_error()` presence/absence, `DispatchOutcome` value preservation

**8. Clippy Fix: Unused Feature-Gated Imports**
- `tests_tarpc.rs` imports now `#[cfg(feature = "live-clients")]` to avoid unfulfilled lint expectations

### Quality Gates

| Gate | Status |
|------|--------|
| `cargo fmt --check` | Clean |
| `cargo clippy` (pedantic + nursery + cargo, `-D warnings`) | Clean (0 warnings) |
| `cargo doc --workspace --no-deps` | Clean (0 warnings) |
| `cargo test --workspace` | 1056+ pass (default features), 0 fail |
| `cargo deny check` | advisories ok, bans ok, licenses ok, sources ok |
| `unsafe_code = "deny"` | Workspace-wide |
| `unwrap_used`/`expect_used` | `"deny"` workspace-wide |
| Coverage gate | 92.32% lines (`--fail-under-lines 90` CI enforced) |
| SPDX headers | All `.rs` files |
| Max file size | All under 1000 lines |
| Production unwrap/expect | Zero |

---

## [0.13.0-dev] - 2026-03-16 (session 14)

### Changed

#### Deep Debt Execution — Structured IPC, tarpc 0.37, Capability Domains, NDJSON Streaming

**1. Structured IPC Error Types** (absorbed from healthSpring V28 `SendError` pattern)
- Added `IpcErrorPhase` enum (7 variants: Connect, Write, Read, InvalidJson, HttpStatus, NoResult, JsonRpcError)
- Evolved `unix_socket.rs` from opaque `Integration(String)` to structured `Ipc { phase, message }`
- Each IPC lifecycle phase now emits typed errors for targeted retry and observability

**2. tarpc 0.34 → 0.37**
- Bumped workspace tarpc dependency; resolved `RUSTSEC-2024-0387` (opentelemetry_api)
- Updated `deny.toml` to remove resolved advisory ignore
- opentelemetry, tokio-serde, tarpc-plugins all upgraded

**3. Capability Domain Introspection** (absorbed from ludoSpring V20 `capability_domains.rs`)
- Added `CapabilityDomain`, `CapabilityMethod` structs with `external: bool` flag to `niche.rs`
- `capability_list()` now includes `domains`, `locality` (local/external counts), per-method `external` flag
- All 23 rhizoCrypt methods classified as local (CPU-only infrastructure)

**4. DI Config Reader Pattern** (absorbed from sweetGrass v0.7.15)
- Added `RpcConfig::from_env_reader(F)` — dependency-injected environment reader
- Tests can supply mock readers without `temp-env` or `unsafe` env mutation

**5. NDJSON Streaming Support** (absorbed from biomeOS v2.43 Pipeline coordination)
- New `streaming` module: `StreamItem` enum (Data, Progress, End, Error)
- `StreamingAppendResult` for streaming `event.append_batch` responses
- `parse_ndjson_line()` for pipeline consumption
- biomeOS Pipeline coordination graphs can now wire bounded channels

**6. Constant Provenance Documentation**
- All key constants in `constants.rs` now include `Derivation:` / `Source:` / `Chosen:` provenance
- Explains origin, validation context, and rationale for each constant

**7. Debris Cleanup**
- Fixed `Edition: 2021` → `Edition: 2024` in `rhizocrypt version` output
- Fixed K8s ConfigMap env vars: `RHIZOCRYPT_HOST` → `RHIZOCRYPT_RPC_HOST`, `RHIZOCRYPT_PORT` → `RHIZOCRYPT_RPC_PORT`
- Updated README: 1222→1244 tests, 110→118 SPDX files, tarpc 0.37, NDJSON streaming

### Quality Gates

| Gate | Status |
|------|--------|
| `cargo fmt --check` | Clean |
| `cargo clippy` (pedantic + nursery + cargo, all features) | Clean (0 warnings) |
| `cargo doc --workspace --all-features --no-deps` | Clean |
| `cargo test --workspace --all-features` | 1244 pass, 0 fail |
| `cargo deny check` | Clean |
| `unsafe_code = "deny"` | Workspace-wide |
| `unwrap_used`/`expect_used` | `"deny"` workspace-wide |
| Coverage gate | 92.32% lines (`--fail-under-lines 90` CI enforced) |
| SPDX headers | All 118 `.rs` files |
| Max file size | All under 1000 lines |
| Production unwrap/expect | Zero |

---

## [0.13.0-dev] - 2026-03-16 (session 13)

### Added

#### Content Similarity Index Experiment Spec

- New spec: `specs/CONTENT_INDEX_EXPERIMENT.md` — locality-sensitive cross-session discovery
- Proposes a secondary hash index using LSH for structural vertex similarity
- Feature-gated behind `content-index` (no impact on default builds)
- Bridges DAG branching (rhizoCrypt) with linear layering (LoamSpine) concepts
- Spring participation guide published to `wateringHole/CONTENT_SIMILARITY_EXPERIMENT_GUIDE.md`
- ISSUE-012 opened in `SPRING_EVOLUTION_ISSUES.md` for cross-primal coordination

### Changed

#### Documentation Refresh

- Updated `specs/00_SPECIFICATIONS_INDEX.md` with experimental spec section
- Updated `README.md`: test count (1222), coverage (92.32%), SPDX file count (110)
- Updated `CHANGELOG.md` with session 13
- Handoff published to `wateringHole/handoffs/`

### Quality Gates

| Gate | Status |
|------|--------|
| `cargo fmt --check` | Clean |
| `cargo clippy` (pedantic + nursery + cargo, all features) | Clean (0 warnings) |
| `cargo doc --workspace --all-features --no-deps` | Clean |
| `cargo test --workspace --all-features` | 1222 pass, 0 fail |
| `cargo deny check` | Clean |
| `unsafe_code = "deny"` | Workspace-wide |
| `unwrap_used`/`expect_used` | `"deny"` workspace-wide |
| Coverage gate | 92.32% lines (`--fail-under-lines 90` CI enforced) |
| SPDX headers | All 110 `.rs` files |
| Max file size | All under 1000 lines |
| Production unwrap/expect | Zero |

---

## [0.13.0-dev] - 2026-03-16 (session 12)

### Changed

#### Comprehensive Audit — Deep Debt, Idiomatic Rust, Zero-Copy

**1. `#[allow]` → `#[expect]` Migration (42 test modules)**
- Migrated all remaining `#[allow(clippy::unwrap_used, clippy::expect_used)]` to precise `#[expect(...)]` attributes across 42 test files
- Each module now declares only the lint suppressions it actually triggers — unfulfilled `#[expect]` fails the build
- Removed suppressions entirely where tests used neither `unwrap()` nor `expect()`

**2. Safe Type Conversions (10 files)**
- Replaced all `as` casts with `TryFrom`/`TryInto` + saturating fallback (`unwrap_or(MAX)`)
- `binary_integration.rs`: `child.id() as i32` → `i32::try_from(...).expect("pid fits in i32")`; removed `#![allow(clippy::cast_possible_wrap)]`
- `service.rs`: `l as usize` → `usize::try_from(l).unwrap_or(usize::MAX)`, `session_count as u64` → `u64::try_from(...)`
- `store_redb.rs`, `store_sled.rs`, `types.rs`, `dehydration.rs`: all `len() as u64` → `u64::try_from(len).unwrap_or(u64::MAX)`
- `loamspine_http.rs`: `MethodSupport as u8` → `MethodSupport::to_u8()` typed conversion method

**3. Zero-Copy: `SignResponse.signature`**
- Evolved `signing.rs` `SignResponse.signature` from `Vec<u8>` to `bytes::Bytes`
- Eliminates intermediate allocation on signing response deserialization

**4. Smart File Refactoring — `store_redb_tests_advanced.rs`**
- Extracted stats/metrics test domain into `store_redb_tests_stats.rs` (324 lines)
- `store_redb_tests_advanced.rs` reduced from 1001 → 681 lines (was 1 line over limit)
- Both files under 1000-line limit with coherent domain grouping

**5. `rustfmt.toml` Edition Sync**
- Updated `edition = "2021"` → `edition = "2024"` to match workspace `Cargo.toml`

**6. Build Environment Documentation**
- Documented `CARGO_TARGET_DIR` workaround in `docs/ENV_VARS.md` for noexec mount conflicts

### Quality Gates

| Gate | Status |
|------|--------|
| `cargo fmt --check` | Clean |
| `cargo clippy` (pedantic + nursery + cargo, all features) | Clean (0 warnings) |
| `cargo doc --workspace --all-features --no-deps` | Clean |
| `cargo test --workspace --all-features` | 1188 pass, 0 fail |
| `cargo deny check` | Clean |
| `unsafe_code = "deny"` | Workspace-wide (zero unsafe in tests via temp-env) |
| `unwrap_used`/`expect_used` | `"deny"` workspace-wide |
| Coverage gate | 91.63% lines (`--fail-under-lines 90` CI enforced) |
| SPDX headers | All 107 `.rs` files |
| Max file size | All under 1000 lines |
| Production unwrap/expect | Zero |

---

## [0.13.0-dev] - 2026-03-16 (session 11)

### Changed

#### Cross-Ecosystem Absorption (8 patterns from springs + primals)

**1. Niche Self-Knowledge Module** (absorbed from squirrel, neuralSpring, groundSpring, airSpring)
- Created `crates/rhizo-crypt-core/src/niche.rs` — single source of truth for primal identity, capabilities, consumed capabilities, cost estimates, operation dependencies, and semantic mappings
- `capability.list` now sources all data from `niche.rs` instead of hardcoded inline vectors
- 11 new niche module tests (consistency, cross-reference, domain validation)

**2. Enhanced `capability.list` Response** (absorbed from loamSpine, sweetGrass)
- `CapabilityDescriptor` now includes per-method `MethodDescriptor` with `cost` tier (low/medium/high) and `deps` (prerequisite operations)
- biomeOS Pathway Learner can now optimize graph execution order for rhizoCrypt
- `build_capability_descriptors()` builds response from `niche.rs` constants

**3. `temp-env` for Test Isolation** (absorbed from squirrel, groundSpring)
- Replaced all 183 `unsafe { std::env::set_var/remove_var }` blocks across 7 files with `temp_env::with_vars`
- Eliminated all `#[allow(unsafe_code)]` from test modules — zero `unsafe` in the entire codebase
- Removed all `ENV_LOCK` / `ENV_TEST_LOCK` static mutexes and manual cleanup helpers
- Added `temp-env = "0.3"` as workspace dev-dependency

**4. Deploy Graph `fallback = "skip"`** (absorbed from wetSpring)
- Added `fallback = "skip"` to all 4 optional nodes in `graphs/rhizocrypt_deploy.toml` (beardog, songbird, loamspine, sweetgrass)
- biomeOS ConditionalDag now gracefully skips unavailable optional dependencies

**5. CI Coverage Threshold** (absorbed from beardog, biomeOS, barraCuda)
- Added `--fail-under-lines 90` enforcement to CI coverage job in `.github/workflows/ci.yml`
- Prevents coverage regressions below 90%

**6. Workspace Lint Strictness** (absorbed from ludoSpring, squirrel, provenance-trio-types)
- Upgraded `unwrap_used` and `expect_used` from `"warn"` to `"deny"` in workspace lints
- Production code already had zero instances; this prevents regressions

**7. `#[expect(reason = "...")]` Strings** (absorbed from toadstool, loamSpine)
- Added `reason = "..."` strings to all 4 `#[expect()]` attrs in production code
- Documents *why* each lint suppression exists for audit trail

**8. wateringHole Documentation Updates**
- Updated `RHIZOCRYPT_LEVERAGE_GUIDE.md` with niche self-knowledge section, enhanced capability.list format
- Updated `PRIMAL_REGISTRY.md` with post-absorption status

#### Additional Improvements
- Added `serde` `rc` feature to workspace deps (fixes pre-existing `Arc<str>` serialization issue)

### Quality Gates

| Gate | Status |
|------|--------|
| `cargo fmt --check` | Clean |
| `cargo clippy` (pedantic + nursery + cargo, all features) | Clean (0 warnings) |
| `cargo doc --workspace --all-features --no-deps` | Clean |
| `cargo test --workspace --all-features` | 1188+ pass, 0 fail |
| `cargo deny check` | Clean |
| `unsafe_code = "deny"` | Workspace-wide (zero unsafe in tests via temp-env) |
| `unwrap_used`/`expect_used` | `"deny"` workspace-wide |
| Coverage gate | `--fail-under-lines 90` CI enforced |
| SPDX headers | All 110 `.rs` files |
| Max file size | All under 1000 lines |
| Production unwrap/expect | Zero |

---

## [0.13.0-dev] - 2026-03-15 (session 10)

### Changed

#### Edition 2024 Migration (absorbed from wetSpring, airSpring, healthSpring)
- Migrated workspace from Edition 2021 to **Edition 2024** with `rust-version = "1.87"`
- Updated all three Cargo.toml files (workspace, fuzz, showcase)
- Wrapped 183 `std::env::set_var`/`remove_var` calls in `unsafe {}` (Edition 2024 requirement)
- Changed workspace lint from `forbid` to `deny` for `unsafe_code`; `forbid` preserved in non-test builds via `#[cfg_attr(not(test), forbid(unsafe_code))]`
- Collapsed 10 nested `if`/`if let` chains into Edition 2024 `if let` chains
- Applied Edition 2024 `rustfmt` import reordering (types before modules)

#### biomeOS Niche Standard Compliance
- Created `graphs/rhizocrypt_deploy.toml` — 5-node deploy graph (BearDog → Songbird → rhizoCrypt → LoamSpine → sweetGrass) for biomeOS orchestration
- Created `capability_registry.toml` — 23 JSON-RPC methods across 7 domains (`dag.session`, `dag.event`, `dag.vertex`, `dag.merkle`, `dag.slice`, `dag.dehydration`, `health`, `capability`)

#### `#[expect()]` Lint Migration (absorbed from wetSpring V117)
- Migrated 5 production `#[allow(clippy::...)]` to `#[expect(clippy::...)]`
- Caught and removed 1 stale suppression (`missing_const_for_fn` on `RateLimiter::disabled()`)

#### wateringHole Documentation Sync
- Fixed stale method names in `SPRING_PROVENANCE_TRIO_INTEGRATION_PATTERN.md` (`dag.session.append` → `dag.event.append`, `dag.dehydrate` → `dag.dehydration.trigger`)
- Updated `RHIZOCRYPT_LEVERAGE_GUIDE.md` with all 23 current semantic method names + `capability.list`
- Updated `PRIMAL_REGISTRY.md` rhizoCrypt entry (1177 tests, 91.47% coverage, Edition 2024)

### Quality Gates

| Gate | Status |
|------|--------|
| `cargo fmt --check` | Clean |
| `cargo clippy` (pedantic + nursery + cargo, all features) | Clean (0 warnings) |
| `cargo doc --workspace --all-features --no-deps` | Clean |
| `cargo test --workspace --all-features` | 1177 pass, 0 fail |
| `cargo deny check` | Clean |
| `unsafe_code = "deny"` | Workspace-wide (`forbid` in non-test builds) |
| SPDX headers | All 106 `.rs` files |
| Max file size | All under 1000 lines |
| Production unwrap/expect | Zero |

---

## [0.13.0-dev] - 2026-03-15 (session 9)

### Changed

#### Semantic JSON-RPC Method Naming
- `dag.dehydrate` → `dag.dehydration.trigger` (Spring-as-Niche compliant)
- `system.health` → `health.check` (Spring-as-Niche compliant)
- `system.metrics` → `health.metrics` (Spring-as-Niche compliant)
- Added `capability.list` JSON-RPC endpoint for runtime discovery
- Updated all handler tests, integration tests, and showcase scripts

#### Coverage Expansion (91.47% line coverage)
- Added 270 tests across the workspace (907 → 1177)
- `store_redb.rs` coverage: 68% → 90%+ (25 new tests)
- `store_sled.rs` coverage: 79% → 90%+ (25 new tests)
- `songbird/client.rs` coverage: 75% → 90%+ (16 new tests)
- `doctor.rs` coverage: 81% → 90%+ (16 new tests)
- `rhizocrypt-service/lib.rs` coverage: 81% → 90%+ (18 new tests)

#### Zero-Copy Evolution
- `vertex.rs::to_canonical_bytes()` returns `bytes::Bytes` instead of `Vec<u8>`
- Updated signing and store backends to consume `Bytes` directly

#### CI Pipeline Hardening
- Added `cargo-deny` job for license, advisory, and ban enforcement
- Added `--all-features` to coverage and doc CI jobs

#### Dependency Audit
- Resolved AGPL-3.0-only license for `provenance-trio-types` in `deny.toml`
- Cleaned `ring` skip from deny config
- All `cargo deny check` gates green

#### Test Isolation Fix
- Fixed env var race condition in `resolve_bind_addr_*` tests — `clear_bind_addr_env()` helper
- Ensures all bind-address tests sanitize global env state before and after

#### Documentation Updates
- Updated `DEPLOYMENT_CHECKLIST.md` method names: `system.health` → `health.check`, `system.metrics` → `health.metrics`
- Updated README test count to 1177, coverage to 91.47%, SPDX count to 106
- Updated CHANGELOG through session 9

### Quality Gates

| Gate | Status |
|------|--------|
| `cargo fmt --check` | Clean |
| `cargo clippy` (pedantic + nursery + cargo, all features) | Clean (0 warnings) |
| `cargo doc --workspace --all-features --no-deps -D warnings` | Clean |
| `cargo test --workspace --all-features` | 1177 pass, 0 fail |
| `cargo deny check` | Clean |
| `#![forbid(unsafe_code)]` | Workspace-wide |
| SPDX headers | All 106 `.rs` files |
| Max file size | All under 1000 lines |
| Production unwrap/expect | Zero |

---

## [0.13.0-dev] - 2026-03-15 (session 8)

### Changed

#### O(1) Vertex-to-Session Index
- Added `vertex_session_index: Arc<DashMap<VertexId, SessionId>>` to `RhizoCrypt`
- Populated on `append_vertex`, cleaned on `discard_session` and `stop`
- Exposed `session_for_vertex()` for O(1) lookup
- `verify_proof` RPC now uses the index directly — eliminated O(N) session scan

#### CheckoutSlice Evolution
- Replaced placeholder `spine_index: u64` with real LoamSpine parameters: `spine_id`, `entry_hash` (hex), `entry_index`, `session_id`, `checkout_vertex`, `owner`, `holder`, `certificate_id`
- Eliminated all three placeholder values (`[0u8; 32]`, `SessionId::now()`, `VertexId::ZERO`)
- Added hex decode with proper error handling via `RpcError::InvalidRequest`
- Updated JSON-RPC handler, tarpc client, handler tests, and integration tests

#### Zero-Copy DID (`Did` → `Arc<str>`)
- Evolved `Did(pub String)` to `Did(Arc<str>)` with `#[serde(transparent)]`
- `Did::default()` uses `LazyLock` static — allocated once, cloned O(1)
- All DID cloning across sessions, slices, dehydration, and RPC is now a trivial pointer increment

#### Additional Improvements
- Fixed broken intra-doc link in `signing.rs` (`[sign]` → `[Self::sign]`)
- Removed unnecessary intermediate `result` variable in JSON-RPC handler
- Replaced hardcoded `"no songbird"` test assertion with capability-based language
- Extracted `DEFAULT_KEY_TYPE` and `DEFAULT_CONTENT_TYPE` constants in HTTP clients
- Evolved `beardog_http::sign()` and `nestgate_http::retrieve()` to return `bytes::Bytes`
- Used central `constants::LOCALHOST` instead of hardcoded `"127.0.0.1"` in config
- Eliminated redundant `clone()` and double `id()` call in `append_vertex`

#### Documentation & Cleanup
- Updated README test count to 907+
- Updated CHANGELOG with session 8 entry
- Moved cross-project audit doc to wateringHole fossil record
- Created new wateringHole handoff

### Quality Gates

| Gate | Status |
|------|--------|
| `cargo fmt --check` | Clean |
| `cargo clippy` (pedantic + nursery + cargo, all features) | Clean (0 warnings) |
| `cargo doc --workspace --all-features --no-deps -D warnings` | Clean |
| `cargo test --workspace` | 907 pass, 0 fail |
| `#![forbid(unsafe_code)]` | Workspace-wide |
| SPDX headers | All 105 `.rs` files |
| Max file size | All under 1000 lines |
| Production unwrap/expect | Zero |

---

## [0.13.0-dev] - 2026-03-15 (session 7)

### Changed

#### scyBorg License Alignment
- Updated SPDX identifier from `AGPL-3.0-only` to `AGPL-3.0-or-later` across all 105 `.rs` files, `Cargo.toml`, `deny.toml`, `Dockerfile`, CI workflow, and all documentation
- Aligned with wateringHole/scyBorg licensing standard (AGPL-3.0 + ORC + CC-BY-SA)

#### Smart Refactoring — `store_redb.rs`
- Extracted `read_vertex_set` and `write_vertex_set` helpers — eliminated `#[allow(clippy::too_many_lines)]` on `put_vertex`
- `Debug` impl uses `finish_non_exhaustive()` — removed `#[allow(clippy::missing_fields_in_debug)]`

#### Zero-Copy Signing
- Added `sign_owned(Bytes)` / `verify_owned(Bytes)` paths to signing capability client
- `sign_vertex` / `verify_vertex` use `Bytes::from(Vec<u8>)` (ownership transfer) instead of `Bytes::copy_from_slice`

#### Metrics Hardening
- Fixed duplicate padding entries in `ALL_METHODS` array — defined `RPC_METHOD_COUNT` / `ERROR_TYPE_COUNT` constants
- Safe `f64` → `u64` cast: explicit `is_finite()` + positivity check before truncation

#### Modern Async Traits (RPITIT)
- Converted `PermanentStorageProvider` impl in `loamspine_http.rs` from `fn -> impl Future { async move }` to `async fn`
- Removed `#[allow(clippy::manual_async_fn)]` and eliminated pre-async-block cloning

#### Idiomatic Patterns
- `safe_env/capability.rs`: `Option::inspect()` replacing `.map(|x| { side_effect; x })` — removed `#[allow(clippy::manual_inspect)]`
- `store_sled.rs`: `SledExportEntry` type alias replacing `#[allow(clippy::type_complexity)]`; `finish_non_exhaustive()` for Debug
- `doctor.rs`: Added `#[must_use]` and `# Errors` doc section per pedantic clippy

#### Documentation Cleanup
- Rewrote `docs/DEPLOYMENT_CHECKLIST.md` (port 9400, 882+ tests, redb/sled storage, JSON-RPC health checks)
- Fixed `docs/ENV_VARS.md` (`RHIZOCRYPT_DISCOVERY_ADAPTER` as primary, `RHIZOCRYPT_PORT` matching code)
- Updated `README.md` metrics, fixed broken spec links, cleaned showcase port references
- Archived legacy `INTEGRATION_SPECIFICATION.md` to `specs/archive/`
- New wateringHole handoff documenting this session

### Quality Gates

| Gate | Status |
|------|--------|
| `cargo fmt --check` | Clean |
| `cargo clippy` (pedantic + nursery + cargo, all features) | Clean (0 warnings) |
| `cargo doc --workspace --all-features --no-deps` | Clean |
| `cargo test --workspace` | 907 pass, 0 fail |
| `cargo llvm-cov` | 90.88% line coverage |
| `#![forbid(unsafe_code)]` | Workspace-wide (all entry points) |
| SPDX headers | All 105 `.rs` files |
| Max file size | All under 1000 lines |

---

## [0.13.0-dev] - 2026-03-15 (session 6)

### Changed

#### Deep Debt Reduction & Modern Idiomatic Rust
- Evolved `ProtocolAdapter` trait: `call_json(&str, String)` → `call_json(&str, &str)` — borrows where possible, allocates only at transport boundary
- Replaced all `Box<dyn Error>` with typed `ServiceError::Storage` variant in doctor checks
- Extracted `serialize_response()` helper in JSON-RPC mod — logs serialization errors via `tracing::warn!` instead of silently falling back with `unwrap_or_default()`
- Eliminated redundant `.clone()` on last-use values flagged by clippy `redundant_clone`
- Fixed `default_value` in clap `Client` subcommand: now references `constants::LOCALHOST` and `constants::PRODUCTION_RPC_PORT` instead of hardcoded `"127.0.0.1:9400"`

#### Constants Centralization (continued)
- Added `DEFAULT_GC_INTERVAL`, `RATE_LIMIT_CLEANUP_INTERVAL`, `RATE_LIMIT_CLEANUP_INTERVAL_DEV` to `constants.rs`
- Dehydration config now uses `constants::DEFAULT_ATTESTATION_TIMEOUT_SECS` and named `FULL_ATTESTATION_TIMEOUT_SECS`
- Rate limiter cleanup intervals reference constants instead of inline `Duration::from_secs(60/300)`

#### Sovereignty Hardening
- Removed cloud provider references ("AWS KMS, GCP KMS, Azure Key Vault", "S3, Azure") from capability trait doc comments
- All capability docs now use agnostic language ("discovered at runtime via capabilities")

#### Smart File Refactoring
- Extracted `doctor.rs` (197 lines) from `rhizocrypt-service/src/lib.rs` — contains `DoctorCheck`, `run_doctor`, all health check functions
- `lib.rs` reduced from 809 → 624 lines

#### Provenance Trio Wire Types
- Added provenance wire types for notification format (later inlined in session 18)
- `ProvenanceNotifier` converts internal `DehydrationSummary` to wire format

### Quality Gates

| Gate | Status |
|------|--------|
| `cargo fmt --check` | Clean |
| `cargo clippy` (pedantic + nursery + cargo, all features) | Clean (0 warnings) |
| `cargo doc --workspace --all-features --no-deps` | Clean |
| `cargo test --workspace` | 907 pass, 0 fail (default features) |
| `#![forbid(unsafe_code)]` | Workspace-wide (all 4 entry points) |
| SPDX headers | All 104 `.rs` files |
| Max file size | All under 1000 lines (max 858) |
| No production `todo!()`, `unimplemented!()`, `TODO`, `FIXME` | Verified |

---

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
- **AGPL-3.0-or-later**: Updated SPDX identifier, added headers to all 71 source files
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

- **0.13.0-dev** (2026-03-17 s18): Sovereignty — provenance-trio-types eliminated, wire types inlined, 14-crate ecoBin deny, 6 file extractions, cross-compile CI, RUSTSEC-2026-0049 fix
- **0.13.0-dev** (2026-03-17 s17): Deep debt — health probes, 4-format capabilities, ValidationSink, JSON-RPC fuzz
- **0.13.0-dev** (2026-03-16 s14): Deep debt — structured IPC errors, tarpc 0.37, capability domain introspection, NDJSON streaming, DI config, constant provenance, debris cleanup
- **0.13.0-dev** (2026-03-16 s12): Deep audit — `#[expect]` migration (42 files), safe `TryFrom` casts, zero-copy signing, file refactoring, rustfmt edition sync
- **0.13.0-dev** (2026-03-16 s11): Cross-ecosystem absorption — niche.rs, enhanced capability.list, temp-env, deploy fallback, CI coverage gate, deny unwrap/expect
- **0.13.0-dev** (2026-03-15 s10): Edition 2024, deploy graph, capability registry, `#[expect]` lint migration
- **0.13.0-dev** (2026-03-15 s8): O(1) vertex-to-session index, checkout_slice evolution, Did→Arc\<str\>, 907 tests
- **0.13.0-dev** (2026-03-15 s7): scyBorg license, zero-copy signing, store_redb refactor, modern async traits, docs cleanup
- **0.13.0-dev** (2026-03-14 s4): Sovereignty cleanup, 1075 tests, 91% coverage, doc tests rewritten, capability-based errors
- **0.13.0-dev** (2026-03-14 s3): 90% coverage, 1022 tests, platform-agnostic transport, doctor subcommand, zero-copy handler
- **0.13.0-dev** (2026-03-13): Deep debt, 862 tests, cargo-deny, service lib extraction
- **0.13.0-dev** (2026-03-12): wateringHole standards, capability discovery, UniBin
- **0.12.0** (2025-12-26): Lock-free concurrency (DashMap), Songbird registration
- **0.10.0** (2025-12-24): Production ready, pure infant discovery, A+ grade
- **0.9.2** (2025-12-23): Core implementation complete
- **0.9.0** (2025-12-20): Initial release with specifications

