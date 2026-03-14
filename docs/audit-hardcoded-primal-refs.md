# Audit: Hardcoded Primal References vs Capability-Based Discovery

**Scope:** phase1/ and phase2/ Rust source files  
**Date:** 2026-03-14

---

## Executive Summary

This audit identifies hardcoded primal references (socket paths, DIDs, ports, primal names) that should migrate to capability-based discovery. Findings are categorized as **production violations** (require evolution) vs **acceptable** (tests, docs, config/orchestration).

---

## Part 1: Production Violations

### 1.1 Hardcoded Socket Paths (Production)

| File | Line | Type | Current | Proposed Evolution |
|------|------|------|---------|-------------------|
| `phase1/toadstool/crates/integration/protocols/src/bear_dog.rs` | 48 | Socket path | `{runtime_dir}/beardog.sock` in `Default` | `discover_capability("crypto.sign")` → get socket from capability registry; fallback to `paths.primal_socket(discover_provider("signing"))` |
| `phase2/biomeOS/crates/biomeos-spore/src/beacon_genetics/capability.rs` | 157–168 | Socket path | `default_socket()` returns `{runtime}/biomeos/beardog.sock` | Use `CapabilityTranslationRegistry` or `discover_capability("beacon.encrypt")` to resolve provider socket |
| `phase2/biomeOS/crates/biomeos-graph/src/executor/node_handlers.rs` | 197–198 | Socket path | `{temp}/beardog.sock`, `/run/biomeos/beardog.sock` | `discover_provider("signing")` → `paths.primal_socket(provider_id)` |
| `phase2/biomeOS/crates/biomeos-federation/src/discovery.rs` | 437, 452, 666, 792, 835 | Socket path | `songbird.sock`, `beardog.sock`, `nestgate.sock` in discovery logic | Query capability registry by capability (e.g. `coordination`, `crypto.sign`, `storage`) instead of primal name |
| `phase2/biomeOS/crates/biomeos-core/src/universal_biomeos_manager/discovery.rs` | 468 | Socket path | `unix:///run/beardog.sock` | `discover_capability("security")` |
| `phase2/biomeOS/crates/biomeos-core/src/p2p_coordination/socket_providers.rs` | 492–504 | Socket path | `/run/beardog.sock`, `/run/songbird.sock` in test setup | Tests: OK. If used in production paths, switch to capability discovery |
| `phase2/biomeOS/crates/biomeos-atomic-deploy/src/capability_translation.rs` | 652–860 | Socket path | `/tmp/beardog.sock`, `/tmp/songbird.sock` in translation logic | Resolve via `capability.call` / registry; no hardcoded primal sockets |
| `phase2/biomeOS/crates/biomeos-atomic-deploy/src/handlers/capability.rs` | 503–812 | Socket path | `/tmp/beardog.sock`, `/tmp/songbird.sock` in capability responses | Return sockets from capability registry, not literals |
| `phase2/biomeOS/crates/biomeos-atomic-deploy/src/neural_api_server/agents/mod.rs` | 110–566 | Socket path | `beardog.sock`, `toadstool.sock` in route config | Route by capability (e.g. `crypto.*` → discover) not primal name |
| `phase2/rhizoCrypt/crates/rhizo-crypt-core/src/clients/adapters/mod.rs` | 298–311 | Socket path | `/run/ecoPrimals/beardog.sock` in tests | Tests: OK. Doc examples: show `discover_capability("crypto.sign")` pattern |
| `phase2/biomeOS/crates/biomeos-ui/src/orchestrator/discovery.rs` | 403–407 | Socket path | `PrimalClient::with_socket("beardog", "/tmp/beardog.sock")` | `discover_capability("crypto.sign")` → use discovered socket |
| `phase1/toadstool/showcase/03-ecosystem-integration/01-songbird-registration/src/main.rs` | 72–77 | Socket path | `{runtime}/biomeos/coordination.sock` (songbird) | `discover_capability("coordination")` |
| `phase1/toadstool/showcase/03-ecosystem-integration/02-beardog-secured-compute/src/main.rs` | 104–109 | Socket path | `{runtime}/biomeos/security.sock` (beardog) | `discover_capability("crypto.sign")` |
| `phase1/toadstool/showcase/03-ecosystem-integration/03-nestgate-artifact-storage/src/main.rs` | 112–117 | Socket path | `{runtime}/biomeos/storage.sock` (nestgate) | `discover_capability("storage")` |
| `phase1/toadstool/showcase/02-compute-patterns/01-capability-discovery/src/main.rs` | 8, 72 | Socket path | `toadstool.jsonrpc.sock` | Self-reference OK; discovery output should use capability IDs |
| `phase1/toadstool/showcase/01-shader-pipeline/01-naga-fallback/src/main.rs` | 73–74 | Socket path | `toadstool.jsonrpc.sock` | Self-reference for compute; OK if toadstool is self |

### 1.2 Hardcoded Primal Names in Production Logic

| File | Line | Type | Current | Proposed Evolution |
|------|------|------|---------|-------------------|
| `phase2/biomeOS/crates/biomeos/src/modes/rootpulse.rs` | 150 | Primal list | `["rhizocrypt", "loamspine", "sweetgrass", "beardog", "nestgate"]` | Query capability registry for active providers; avoid fixed primal list |
| `phase2/biomeOS/crates/biomeos-ui/src/primal_client.rs` | 194–216 | Accessors | `beardog()`, `songbird()`, `nestgate()`, `toadstool()`, `squirrel()` | Add `get_by_capability("crypto.sign")` etc.; keep legacy accessors for compatibility |
| `phase2/biomeOS/crates/biomeos-graph/src/pathway_learner.rs` | 437–473 | Primal names | `"rhizocrypt"`, `"loamspine"`, `"beardog"` in optimization logic | Use capability-based suggestions (e.g. `discover_provider("signing")`) |
| `phase2/biomeOS/crates/biomeos/src/modes/nucleus.rs` | 332–335 | Method name | `TOADSTOOL`, `"toadstool.health"` | Keep for backward compat; add capability-based health routing |
| `phase2/biomeOS/crates/biomeos/src/main.rs` | 684–690 | Primal name | `"beardog"` in log/assert | Logging: low risk; prefer capability-based identification |

### 1.3 Hardcoded Port Numbers (Production)

| File | Line | Type | Current | Proposed Evolution |
|------|------|------|---------|-------------------|
| `phase1/toadstool/examples/config_management_demo.rs` | 94–96 | Ports | `songbird_port = 8080`, `beardog_port = 8081`, `nestgate_port = 8082` | Use `ConfigUtils::get_*_port()` or env; demo shows both bad/good patterns |
| `phase1/toadstool/examples/modern_patterns_showcase.rs` | 142–143 | Ports | `"songbird:8080"`, `"nestgate:8082"` in discovery | `discover_capability("coordination")` → use endpoint from registry |
| `phase1/toadstool/examples/simplified_distributed_demo.rs` | 125 | Port | `http://localhost:8080` Songbird endpoint | Use `SONGBIRD_ENDPOINT` or capability discovery |
| `phase1/toadstool/examples/production_universal_demo.rs` | 245, 440 | Port | `http://localhost:8080` | Use env or capability discovery |
| `phase1/toadstool/examples/network_performance_benchmark.rs` | 154–158 | Hostnames | `songbird.primal.local`, `beardog.primal.local`, etc. | Use capability discovery for benchmark targets |
| `phase2/biomeOS/crates/biomeos/src/modes/plasmodium.rs` | 330 | Port | `format!("{}:8080", gate_id)` | Use config/env for port; avoid hardcoded 8080 |
| `phase2/loamSpine/crates/loam-spine-core/src/constants/network.rs` | 346–351 | Default port | `"http://localhost:8080"` in `build_endpoint` | Use `jsonrpc_port()` / env; tests may keep literals |

### 1.4 Hardcoded DID References (Production)

| File | Line | Type | Current | Proposed Evolution |
|------|------|------|---------|-------------------|
| `phase2/rhizoCrypt/showcase/**/*.sh` | Various | DID literals | `did:beardog:...`, `did:toadstool:...`, `did:nestgate:...` | Showcase scripts: document capability-based DID resolution; production code should use `discover_provider("signing")` → resolve DID from registry |

*Note: Showcase scripts are orchestration/demo; DIDs there are illustrative. Production Rust code should resolve DIDs via capability discovery.*

### 1.5 Direct Primal Method Calls (Should Use capability.call)

| File | Line | Type | Current | Proposed Evolution |
|------|------|------|---------|-------------------|
| `phase1/toadstool/crates/server/src/pure_jsonrpc/handler/mod.rs` | 139–157 | Method routing | `"toadstool.submit_workload"`, `"toadstool.health"` etc. | Keep for backward compat; add `capability.call` dispatch for cross-primal calls |
| `phase2/biomeOS/crates/neural-api-client-sync/src/lib.rs` | 118 | Method | `"method": "capability.call"` | Already capability-based |
| `phase2/biomeOS/crates/biomeos-atomic-deploy/src/neural_executor.rs` | 660–681 | Routing | Routes via `capability.call` | Already capability-based |

---

## Part 2: Production Mocks NOT Behind `#[cfg(test)]`

| File | Line | Type | Current | Proposed Evolution |
|------|------|------|---------|-------------------|
| `phase2/rhizoCrypt/crates/rhizo-crypt-core/src/integration/mocks.rs` | 45–534 | Mock structs | `MockSigningProvider`, `MockPermanentStorageProvider`, `MockPayloadStorageProvider`, `MockProtocolAdapter`, `MockCapabilityFactory` | Add `#[cfg(any(test, feature = "test-mocks"))]` so they are excluded from default production builds |
| `phase1/squirrel/crates/universal-patterns/src/federation/network_connection.rs` | 32–60 | `MockNetworkConnection` | No `#[cfg(test)]` | Add `#[cfg(any(test, feature = "test-mocks"))]` |
| `phase1/squirrel/crates/universal-patterns/src/federation/network/peers.rs` | 86–92 | `MockNetworkConnection` | No `#[cfg(test)]` | Add `#[cfg(any(test, feature = "test-mocks"))]` |
| `phase1/squirrel/crates/universal-patterns/src/federation/federation_network.rs` | 485–491 | `MockNetworkConnection` | No `#[cfg(test)]` | Add `#[cfg(any(test, feature = "test-mocks"))]` |
| `phase1/squirrel/crates/universal-patterns/src/orchestration/mod.rs` | 628–635 | `MockOrchestrationProvider` | Has `#[cfg(test)]` | OK |
| `phase1/squirrel/crates/ecosystem-api/src/client.rs` | 289–302 | `MockServiceMeshClient` | No `#[cfg(test)]` | Add `#[cfg(any(test, feature = "test-mocks"))]` |
| `phase1/squirrel/crates/integration/web/src/database.rs` | 517–522 | `MockDatabaseClient` | No `#[cfg(test)]` | Add `#[cfg(any(test, feature = "test-mocks"))]` |

---

## Part 3: Acceptable References (No Change Required)

### 3.1 Test Code

- All `*_tests.rs`, `tests/`, `#[cfg(test)]` modules
- Test fixtures using `beardog.sock`, `songbird:8080`, etc.
- `phase1/toadstool/crates/testing/` (test-only crate)
- `phase1/toadstool/crates/server/src/mocks.rs` (behind `#[cfg(test)]`)
- `phase1/toadstool/crates/auto_config` mocks (behind `#[cfg(any(test, feature = "test-mocks"))]`)
- `phase1/toadstool/crates/distributed` MockSecurityProvider (behind `#[cfg(test)]`)

### 3.2 Config / Graph / Orchestration

- `phase1/toadstool/examples/biomeos_substrate_demo.rs` YAML config (orchestration)
- `phase1/toadstool/showcase/02-compute-patterns/03-deploy-graph` graph definitions
- `phase1/toadstool/showcase/02-compute-patterns/01-capability-discovery` (demonstrates discovery)
- `phase2/biomeOS/crates/biomeos` nucleus mode primal registration (orchestrator knows primals)

### 3.3 Documentation

- `phase2/biomeOS/crates/biomeos-types/src/paths.rs` (doc examples showing bad vs good)
- `phase2/rhizoCrypt/crates/rhizo-crypt-core/src/safe_env/capability.rs` (doc examples)
- `phase2/rhizoCrypt/showcase/**/README.md` (documentation)

### 3.4 Crate Names / Imports

- `use toadstool_*`, `toadstool_common::*` (crate names, not runtime primal refs)
- `toadstool.jsonrpc.sock` when part of toadstool’s own socket naming (self-reference)

---

## Part 4: Recommended Evolution Path

1. **Capability registry**  
   Ensure a central registry maps capabilities (e.g. `crypto.sign`, `storage`, `coordination`) to provider sockets/DIDs.

2. **Discovery helpers**  
   - `discover_capability("crypto.sign")` → socket/DID  
   - `discover_provider("signing")` → primal ID for `paths.primal_socket()`

3. **Migration order**  
   - `bear_dog.rs` default socket  
   - `biomeos-spore` beacon capability  
   - `biomeos-atomic-deploy` handlers  
   - `biomeos-federation` discovery  
   - `biomeos-ui` discovery

4. **Mocks**  
   Gate all production mocks with `#[cfg(any(test, feature = "test-mocks"))]` and enable `test-mocks` only in dev/test profiles.

---

## Appendix: Quick Reference

| Pattern | Bad | Good |
|---------|-----|------|
| Socket | `connect_to("beardog.sock")` | `discover_capability("crypto.sign")` → socket |
| DID | `Did::new("did:beardog:main")` | `discover_provider("signing")` → DID from registry |
| Port | `"localhost:8080"` | `ConfigUtils::get_songbird_port()` or discovery |
| Method | `beardog.sign(data)` | `capability.call("crypto.sign", params)` |
