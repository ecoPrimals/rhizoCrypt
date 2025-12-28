# 🌱 Hardcoding Elimination Plan - Infant Discovery Evolution

**Date**: December 28, 2025  
**Philosophy**: "Primals start with zero knowledge and discover, like an infant"  
**Goal**: Eliminate ALL hardcoding (primal names, ports, addresses, service names)

---

## 📊 Current State Analysis

### ✅ **Already Good**

1. **Capability-Based Architecture** (v0.13.0)
   - ✅ `SigningProvider` instead of `BearDogClient`
   - ✅ `PermanentStorageProvider` instead of `LoamSpineClient`
   - ✅ `PayloadStorageProvider` instead of `NestGateClient`
   - ✅ Factory pattern for capability discovery
   - ✅ `SafeEnv` module for environment variable access

2. **Infant Discovery** (Partial)
   - ✅ `RHIZOCRYPT_DISCOVERY_ADAPTER` for bootstrap
   - ✅ Environment-based configuration (`safe_env.rs`)
   - ✅ No hardcoded addresses in main service
   - ✅ Graceful fallback to development mode

3. **Service Independence**
   - ✅ Service starts with `RHIZOCRYPT_PORT=0` (OS-assigned)
   - ✅ Optional Songbird registration
   - ✅ Standalone mode supported

### ❌ **Hardcoding to Eliminate**

| Category | Count | Examples | Status |
|----------|-------|----------|--------|
| **Primal Names** | 1011+ matches | `beardog`, `nestgate`, `songbird`, `toadstool`, `loamspine` | 🔴 High |
| **File Names** | 23 files | `beardog_http.rs`, `nestgate_http.rs`, `songbird_rpc.rs` | 🟡 Medium |
| **Legacy Modules** | 6 files | `clients/legacy/*` (beardog, nestgate, toadstool, loamspine, sweetgrass) | 🔴 High |
| **Port Numbers** | 47 files | `9400`, `8080`, `127.0.0.1`, `0.0.0.0` | 🟡 Medium |
| **Service Names** | Unknown | k8s, consul, etcd mentions? | 🟢 Low |

---

## 🎯 Elimination Strategy

### Phase 1: Legacy Client Modules (P0 - Critical)

**Problem**: `clients/legacy/` contains primal-specific clients with hardcoded assumptions.

**Files to Eliminate**:
```
crates/rhizo-crypt-core/src/clients/legacy/
├── beardog.rs          ❌ Remove (use SigningProvider)
├── nestgate.rs         ❌ Remove (use PayloadStorageProvider)
├── toadstool.rs        ❌ Remove (use ComputeProvider)
├── loamspine.rs        ❌ Remove (use PermanentStorageProvider)
├── sweetgrass.rs       ❌ Remove (deprecated)
└── mod.rs              ❌ Remove (legacy exports)
```

**Migration Path**:
1. ✅ Already have: `capabilities/signing.rs` → `SigningProvider`
2. ✅ Already have: `capabilities/storage.rs` → `PayloadStorageProvider`
3. ✅ Already have: `capabilities/permanent.rs` → `PermanentStorageProvider`
4. ✅ Already have: `capabilities/compute.rs` → `ComputeProvider`
5. ❌ Need: Update all call sites to use capabilities instead

**Action**:
- [ ] Search for all `legacy::` imports
- [ ] Replace with capability-based equivalents
- [ ] Delete entire `legacy/` directory
- [ ] Update tests to use capability mocks

---

### Phase 2: Vendor-Named Files (P0 - Critical)

**Problem**: File names encode vendor lock-in.

**Files to Rename/Eliminate**:
```
crates/rhizo-crypt-core/src/clients/
├── beardog_http.rs      → DELETE (use adapters/http.rs + capabilities)
├── nestgate_http.rs     → DELETE (use adapters/http.rs + capabilities)
├── toadstool_http.rs    → DELETE (use adapters/http.rs + capabilities)
├── loamspine_rpc.rs     → DELETE (use adapters/tarpc.rs + capabilities)
├── songbird_rpc.rs      → KEEP but rename to `bootstrap_adapter.rs`
├── songbird/            → RENAME to `bootstrap/`
├── songbird_types.rs    → RENAME to `bootstrap_types.rs`
```

**Rationale**:
- **Songbird** is special: It's the **bootstrap adapter** (universal adapter for infant discovery)
- But it should be named for its **role**, not vendor
- Any compatible discovery service (Consul, etcd, Nomad, Kubernetes, custom) can fill this role

**Migration Path**:
1. Generic adapters already exist:
   - ✅ `adapters/http.rs` - Works with ANY HTTP service
   - ✅ `adapters/tarpc.rs` - Works with ANY tarpc service
   - ✅ `adapters/mod.rs` - Protocol-agnostic adapter trait

2. Capability clients already use generic adapters:
   - ✅ `SigningClient` works with HTTP or tarpc
   - ✅ `StorageClient` works with HTTP or tarpc
   - ✅ `ComputeClient` works with HTTP or tarpc

3. **Action**:
   - [ ] Delete vendor-specific HTTP/RPC files
   - [ ] Rename `songbird/` → `bootstrap/`
   - [ ] Update all imports
   - [ ] Verify tests still pass

---

### Phase 3: Port Number Hardcoding (P1 - Important)

**Problem**: 47 files contain hardcoded ports, IPs, or addresses.

**Current Constants** (Good Pattern from Songbird):
```rust
// Songbird's approach: Named constants with clear defaults
pub const DEFAULT_HTTP_PORT: u16 = 8080;
pub const DEFAULT_DISCOVERY_PORT: u16 = 8081;
pub const LOCALHOST: &str = "127.0.0.1";
pub const PRODUCTION_BIND_ADDRESS: &str = "0.0.0.0";
```

**Our Current State**:
```rust
// config.rs - Already good!
pub struct RpcConfig {
    pub host: Cow<'static, str>,  // From RHIZOCRYPT_RPC_HOST
    pub port: u16,                 // From RHIZOCRYPT_RPC_PORT or 0
}

const DEFAULT_HOST: &'static str = "127.0.0.1";
const DEFAULT_PORT: u16 = 0;  // OS-assigned!
```

**Issues**:
- Tests use hardcoded `127.0.0.1:9400`
- Showcase scripts hardcode ports
- Some demo scripts use `localhost:8080`

**Action**:
- [ ] Create `crates/rhizo-crypt-core/src/constants.rs` (like Songbird)
- [ ] Move all magic numbers to constants with clear names
- [ ] Update tests to use port 0 or OS-assigned
- [ ] Update showcase scripts to read from env vars
- [ ] Add `RHIZOCRYPT_TEST_PORT` for test isolation

---

### Phase 4: Primal Name References (P1 - Important)

**Problem**: 1011+ case-insensitive matches for primal names in code.

**Categories**:

1. **Module Names**: Already addressed in Phase 2
2. **Comments & Docs**: Need update
3. **Test Data**: Use generic names
4. **Error Messages**: Use capability names
5. **Env Var Names**: Already good (capability-based)

**Examples to Fix**:

```rust
// ❌ BAD: Hardcoded primal name
log::info!("Connecting to BearDog at {}", addr);
log::error!("Failed to reach NestGate");

// ✅ GOOD: Capability-based
log::info!("Connecting to signing provider at {}", addr);
log::error!("Failed to reach payload storage provider");
```

```rust
// ❌ BAD: Test data with primal names
let did = Did::from("did:primal:beardog:test");

// ✅ GOOD: Generic test data
let did = Did::from("did:primal:test:entity");
```

**Action**:
- [ ] Audit all 1011 matches
- [ ] Categorize: code vs docs vs tests
- [ ] Rename in code to capability terms
- [ ] Update docs to use generic examples
- [ ] Keep Phase 1 references ONLY in showcase/integration docs

---

### Phase 5: External Service Names (P2 - Nice to Have)

**Problem**: Potential hardcoding of orchestration systems (k8s, consul, etc.)

**Current State**: Unknown, need audit

**Action**:
- [ ] Search for `kubernetes`, `k8s`, `consul`, `etcd`, `nomad`
- [ ] Ensure all are behind capability interfaces
- [ ] Use env vars for orchestrator endpoints
- [ ] Document which orchestrators are supported

---

## 🏗️ File Structure Evolution

### Before (Current):
```
crates/rhizo-crypt-core/src/clients/
├── capabilities/           ✅ Keep (vendor-agnostic)
├── adapters/               ✅ Keep (protocol-agnostic)
├── factory.rs              ✅ Keep (discovery logic)
├── legacy/                 ❌ DELETE (vendor lock-in)
│   ├── beardog.rs
│   ├── nestgate.rs
│   ├── toadstool.rs
│   ├── loamspine.rs
│   └── sweetgrass.rs
├── beardog_http.rs         ❌ DELETE (use adapters/http + capabilities)
├── nestgate_http.rs        ❌ DELETE
├── toadstool_http.rs       ❌ DELETE
├── loamspine_rpc.rs        ❌ DELETE (use adapters/tarpc + capabilities)
├── songbird_rpc.rs         🟡 RENAME → bootstrap_adapter.rs
├── songbird/               🟡 RENAME → bootstrap/
└── songbird_types.rs       🟡 RENAME → bootstrap_types.rs
```

### After (Target):
```
crates/rhizo-crypt-core/src/clients/
├── capabilities/           ✅ Vendor-agnostic traits
│   ├── signing.rs          (ANY signing service)
│   ├── storage.rs          (ANY payload storage)
│   ├── permanent.rs        (ANY permanent storage)
│   ├── compute.rs          (ANY compute provider)
│   └── provenance.rs       (ANY audit log)
├── adapters/               ✅ Protocol-agnostic adapters
│   ├── http.rs             (ANY HTTP service)
│   ├── tarpc.rs            (ANY tarpc service)
│   └── mod.rs              (Adapter trait)
├── bootstrap/              🌱 Infant discovery bootstrap
│   ├── client.rs           (Discovery service client - Songbird, Consul, etc.)
│   ├── mod.rs
│   └── tests.rs
├── bootstrap_adapter.rs    🌱 RPC adapter for bootstrap service
├── bootstrap_types.rs      🌱 Discovery protocol types
├── factory.rs              ✅ Creates capability clients from discovery
├── mod.rs                  ✅ Public API
└── constants.rs            🆕 All magic numbers centralized
```

**Key Insight**: `bootstrap/` is the ONLY module that knows about a specific service (the discovery/bootstrap service). Everything else is pure capabilities.

---

## 🌱 Infant Discovery Flow (After Completion)

```
┌─────────────────────────────────────────────────────────────┐
│ 1. BIRTH - Zero Knowledge                                   │
│    rhizoCrypt starts, knows only:                           │
│    - Self: "I am rhizoCrypt, a DAG engine"                  │
│    - Capabilities I provide: "ephemeral:dag-engine"         │
│    - Capabilities I need: NONE required (standalone!)       │
└─────────────────────────────────────────────────────────────┘
                             ↓
┌─────────────────────────────────────────────────────────────┐
│ 2. BOOTSTRAP (Optional)                                     │
│    Check env: RHIZOCRYPT_DISCOVERY_ADAPTER                  │
│    If present: Connect to bootstrap service                 │
│    If absent: Standalone mode (no discovery)                │
└─────────────────────────────────────────────────────────────┘
                             ↓
┌─────────────────────────────────────────────────────────────┐
│ 3. SELF-REGISTRATION (Optional)                             │
│    If bootstrap available:                                  │
│      Register capabilities: ["ephemeral:dag-engine"]        │
│      Announce endpoint: "tarpc://self:9400"                 │
└─────────────────────────────────────────────────────────────┘
                             ↓
┌─────────────────────────────────────────────────────────────┐
│ 4. ON-DEMAND DISCOVERY                                      │
│    When needed (e.g., dehydration):                         │
│      Query: "Who provides permanent:storage?"               │
│      Receive: ["tarpc://host:port", "http://host:port"]     │
│      Connect: Use adapter factory                           │
└─────────────────────────────────────────────────────────────┘
                             ↓
┌─────────────────────────────────────────────────────────────┐
│ 5. RUNTIME OPERATION                                        │
│    Use cached capability clients                            │
│    No compile-time knowledge of providers                   │
│    Works with ANY compatible service                        │
└─────────────────────────────────────────────────────────────┘
```

**Key Properties**:
- ✅ Zero compile-time dependencies
- ✅ Zero hardcoded addresses
- ✅ Zero vendor lock-in
- ✅ Works standalone (no discovery required)
- ✅ Works federated (discovery optional)
- ✅ Graceful degradation (missing capabilities = reduced features)

---

## 📋 Execution Checklist

### Phase 1: Legacy Elimination (P0)
- [ ] Find all `legacy::` imports (grep)
- [ ] Replace with capability-based equivalents
- [ ] Update tests to use capability mocks
- [ ] Delete `crates/rhizo-crypt-core/src/clients/legacy/` directory
- [ ] Run `cargo test --workspace`
- [ ] Run `cargo clippy --workspace`

### Phase 2: Vendor-Named Files (P0)
- [ ] Delete `beardog_http.rs`, `nestgate_http.rs`, `toadstool_http.rs`, `loamspine_rpc.rs`
- [ ] Rename `songbird/` → `bootstrap/`
- [ ] Rename `songbird_rpc.rs` → `bootstrap_adapter.rs`
- [ ] Rename `songbird_types.rs` → `bootstrap_types.rs`
- [ ] Update all imports across codebase
- [ ] Run tests

### Phase 3: Numeric Constants (P1)
- [ ] Create `src/constants.rs` with named constants
- [ ] Audit 47 files with port/IP hardcoding
- [ ] Replace magic numbers with named constants
- [ ] Update test harness for port 0
- [ ] Update showcase scripts for env vars
- [ ] Run tests

### Phase 4: Primal Name References (P1)
- [ ] Categorize 1011+ matches (code vs docs vs tests)
- [ ] Update code references to capability terms
- [ ] Update comments to generic examples
- [ ] Update error messages
- [ ] Run tests

### Phase 5: External Services (P2)
- [ ] Search for orchestrator names
- [ ] Ensure capability interfaces
- [ ] Document supported orchestrators
- [ ] Update deployment docs

---

## 🎯 Success Criteria

### Must Have (P0)
- ✅ Zero files named after primals (except showcase/docs)
- ✅ Zero `legacy::` imports
- ✅ All capability clients work with generic adapters
- ✅ Service starts with zero hardcoded dependencies
- ✅ All tests pass

### Should Have (P1)
- ✅ All ports configurable via env vars
- ✅ All IPs configurable via env vars
- ✅ Named constants for all magic numbers
- ✅ Primal names only in docs/showcase
- ✅ Error messages use capability terms

### Nice to Have (P2)
- ✅ Orchestrator-agnostic deployment
- ✅ Zero mentions of external service names in code
- ✅ Complete infant discovery documentation
- ✅ Visual flow diagrams

---

## 🚀 Timeline Estimate

| Phase | Effort | Duration |
|-------|--------|----------|
| Phase 1: Legacy Elimination | High | 2-3 hours |
| Phase 2: Rename Files | Medium | 1-2 hours |
| Phase 3: Numeric Constants | Medium | 1-2 hours |
| Phase 4: Name References | High | 3-4 hours |
| Phase 5: External Services | Low | 0-1 hour |
| **Total** | **High** | **7-12 hours** |

---

## 📚 References

**Phase 1 Examples**:
- Songbird: `songbird-types/src/constants.rs` - Clean constant organization
- BearDog: `beardog-discovery/src/discovery.rs` - Capability-based env vars
- BearDog: Uses `BEARDOG_*` prefix for self-knowledge only

**Key Insight from Phase 1**:
> Even primals that know their own name (e.g., `BEARDOG_API_BIND_ADDR`) 
> do NOT hardcode other primals' names. Discovery is always via capabilities.

---

## 🎓 Philosophy: Infant Discovery

### **The Infant Primal**

When a human infant is born, they:
1. Have zero knowledge of the world
2. Know only themselves (bodily awareness)
3. Discover everything through interaction
4. Learn by asking "what can you do?" not "what is your name?"

Our primals follow the same pattern:

```bash
# Birth: Zero knowledge
./rhizocrypt-service

# Knows only self
RHIZOCRYPT_PORT=9400 ./rhizocrypt-service

# Optional bootstrap (like learning language)
RHIZOCRYPT_DISCOVERY_ADAPTER=bootstrap.local:7500 ./rhizocrypt-service

# Everything else discovered at runtime
# No hardcoded knowledge of beardog, nestgate, etc.
```

### **No 2^n Connection Problem**

Traditional systems: Every service hardcodes every other service.
- N services → N² connections to configure
- Add a service → Update N other configs

Infant discovery: Every service knows only the bootstrap.
- N services → N connections (to bootstrap)
- Add a service → Update 1 config (its own)

This is the power of capability-based architecture.

---

**Status**: 🚀 Ready to Execute  
**Impact**: 🔥 Ecosystem Leadership - First Pure Infant Discovery Primal

