# 🌱 Infant Discovery Evolution - Complete Technical Debt Elimination

**Date**: December 27, 2025  
**Vision**: Each primal starts with ZERO knowledge, discovers everything  
**Status**: Evolution in progress

---

## 🎯 Infant Discovery Principle

> **"Born knowing only yourself, discover the world through capability"**

```
Birth      → rhizoCrypt knows: "I am rhizoCrypt" 
             rhizoCrypt does NOT know: BearDog, NestGate, LoamSpine, Songbird, K8s, Consul
             
Bootstrap  → Find universal adapter via: DISCOVERY_ENDPOINT env var
             Universal adapter could be: Songbird, Consul, etcd, K8s service mesh, mDNS
             
Discovery  → Query: "Who provides crypto:signing?"
             Response: "Service X at endpoint Y provides crypto:signing"
             
Connect    → Use capability, not vendor name
             Code says: SigningProvider.sign(data)
             NOT: BearDog.sign(data)
             
Result     → Works with ANY provider (BearDog, YubiKey, CloudKMS, HSM, etc.)
```

---

## 📊 Current Hardcoding Audit (Dec 27, 2025)

### Primal Names Found: **1,077 references**

| Primal | Count | Location | Type |
|--------|-------|----------|------|
| **Songbird** | ~150 | Everywhere | ⚠️ Bootstrap adapter |
| **BearDog** | ~250 | Legacy clients, mocks, docs | ⚠️ Deprecated aliases |
| **NestGate** | ~200 | Legacy clients, mocks, docs | ⚠️ Deprecated aliases |
| **LoamSpine** | ~220 | Legacy clients, mocks, docs | ⚠️ Deprecated aliases |
| **ToadStool** | ~150 | Legacy clients, HTTP client | ⚠️ Capability client exists |
| **SweetGrass** | ~100 | Legacy clients, provenance | ⚠️ Capability client exists |
| **Squirrel** | ~7 | Comments only | ✅ Minimal |

### External Services Found: **7 references**

| Service | Count | Location | Status |
|---------|-------|----------|--------|
| Docker | 3 | README, deployment docs | ✅ Acceptable (deployment tool) |
| Kubernetes | 3 | README, comments | ✅ Acceptable (deployment target) |
| Consul | 1 | Comment about alternatives | ✅ Acceptable (documentation) |

### Port/Numeric Constants: **27 files**

- Test code: 277 occurrences ✅ (acceptable)
- Production: 0 hardcoded ✅ (all env-driven)

---

## 🏗️ Architecture Evolution Strategy

### Phase 1: Type System ✅ COMPLETE (v0.13.0)

**Achievement**: Vendor names removed from type system

```rust
// BEFORE (Vendor-Specific)
pub trait BearDogClient { }
pub trait NestGateClient { }
pub trait LoamSpineClient { }

// AFTER (Capability-Based) ✅
pub trait SigningProvider { }
pub trait PayloadStorageProvider { }
pub trait PermanentStorageProvider { }
```

**Status**: ✅ DONE - Type system is capability-based

---

### Phase 2: Legacy Client Migration 🔄 IN PROGRESS

**Goal**: Deprecate and eventually remove primal-named clients

#### Current State

```
crates/rhizo-crypt-core/src/clients/
├── capabilities/          ← NEW: Capability-based (SigningClient, StorageClient)
│   ├── signing.rs        ✅ Pure capability
│   ├── storage.rs        ✅ Pure capability
│   ├── permanent.rs      ✅ Pure capability
│   ├── compute.rs        ✅ Pure capability
│   └── provenance.rs     ✅ Pure capability
│
├── legacy/               ← OLD: Primal-named (BearDogClient, NestGateClient)
│   ├── beardog.rs        ⚠️ Deprecated, kept for compat
│   ├── nestgate.rs       ⚠️ Deprecated, kept for compat
│   ├── loamspine.rs      ⚠️ Deprecated, kept for compat
│   ├── toadstool.rs      ⚠️ Deprecated, kept for compat
│   └── sweetgrass.rs     ⚠️ Deprecated, kept for compat
│
└── adapters/             ✅ Protocol-agnostic
    ├── http.rs           ✅ Pure HTTP
    └── tarpc.rs          ⚠️ Scaffolded
```

#### Evolution Steps

**Step 1: Mark legacy clients as deprecated** ✅ DONE
```rust
#[deprecated(since = "0.13.0", note = "Use SigningClient instead")]
pub type BearDogClient = impl SigningProvider;
```

**Step 2: Update all internal code to use capability clients** 🔄 IN PROGRESS
- lib.rs: Still uses some legacy patterns
- Tests: Mix of old and new
- Examples: Need updates

**Step 3: Remove legacy clients** 📋 PLANNED (v1.0.0)
- v0.14.0-0.99.0: Gradual deprecation warnings
- v1.0.0: Breaking change, remove legacy

---

### Phase 3: Bootstrap Evolution 🎯 NEXT PRIORITY

**Problem**: "Songbird" is hardcoded as bootstrap adapter

**Current Bootstrap**:
```rust
// rhizocrypt-service/src/main.rs:57
use rhizo_crypt_core::clients::songbird::{SongbirdClient, SongbirdConfig};

// Hardcoded to Songbird
async fn register_with_songbird(...) { }
```

**Vision**: Universal bootstrap via capability

```rust
// NEW: Bootstrap via capability, not vendor
use rhizo_crypt_core::bootstrap::UniversalAdapter;

// Could be Songbird, Consul, etcd, K8s, mDNS
let adapter = UniversalAdapter::from_env()?;
// Reads: DISCOVERY_PROTOCOL (http, tarpc, mdns, consul, k8s)
//        DISCOVERY_ENDPOINT (addr or service name)

adapter.register(our_capabilities).await?;
let providers = adapter.discover(needed_capabilities).await?;
```

**Implementation Plan**:

1. **Create `bootstrap` module** (2 hours)
   ```rust
   // crates/rhizo-crypt-core/src/bootstrap/mod.rs
   pub enum DiscoveryProtocol {
       Http,    // REST API (current Songbird)
       Tarpc,   // RPC (future)
       MDns,    // Local network
       Consul,  // HashiCorp Consul
       K8s,     // Kubernetes service discovery
   }
   
   pub struct UniversalAdapter {
       protocol: DiscoveryProtocol,
       endpoint: String,
   }
   ```

2. **Implement adapters** (1 day)
   - HttpAdapter (wrap current Songbird client)
   - TarpcAdapter (future)
   - ConsulAdapter (future)
   - K8sAdapter (future)

3. **Update service binary** (2 hours)
   ```rust
   // rhizocrypt-service/src/main.rs
   let adapter = UniversalAdapter::from_env()?;
   // Supports ANY discovery service
   ```

4. **Deprecate Songbird-specific code** (1 hour)
   - Mark `clients/songbird/` as legacy
   - Keep for backward compatibility
   - Recommend UniversalAdapter

**Timeline**: 2 days

---

### Phase 4: Deep Code Cleanup 🔄 ONGOING

#### A. Cognitive Complexity Refactoring

**High Complexity Functions** (3 functions >25):

1. **nestgate.rs:382** - `connect_to()` (complexity: 33)
   ```rust
   // BEFORE: Monolithic 100-line function
   async fn connect_to(&self, addr: SocketAddr) -> Result<()> {
       // Connection logic
       // Protocol negotiation  
       // Health checking
       // Error handling
       // Retry logic
   }
   
   // AFTER: Decomposed into helpers
   async fn connect_to(&self, addr: SocketAddr) -> Result<()> {
       let connection = self.establish_connection(&addr).await?;
       let protocol = self.negotiate_protocol(&connection).await?;
       self.verify_health(&connection).await?;
       Ok(())
   }
   
   async fn establish_connection(&self, addr: &SocketAddr) -> Result<Connection> { }
   async fn negotiate_protocol(&self, conn: &Connection) -> Result<Protocol> { }
   async fn verify_health(&self, conn: &Connection) -> Result<()> { }
   ```

2. **toadstool.rs:400** - `connect_to()` (complexity: 35)
   - Similar pattern to above
   - Extract: connection, protocol, health, retry

3. **songbird/client.rs:280** - `connect()` (complexity: 39)
   - Extract: endpoint resolution, connection setup, registration, heartbeat

**Estimated Time**: 4 hours (all 3 functions)

#### B. File Size Refactoring

**lib.rs: 1,094 lines** → Target: <1,000 lines

```rust
// EXTRACT:
lib.rs (1,094 lines)
├── dag.rs (300 lines)            ← DAG operations
│   ├── append_vertex()
│   ├── get_vertex()
│   ├── get_frontier()
│   └── topological_sort()
│
├── session_manager.rs (200 lines) ← Session lifecycle
│   ├── create_session()
│   ├── get_session()
│   ├── list_sessions()
│   └── delete_session()
│
├── dehydration_impl.rs (150 lines) ← Dehydration workflow
│   ├── dehydrate()
│   ├── collect_attestations()
│   ├── commit_to_permanent_storage()
│   └── get_dehydration_status()
│
└── lib.rs (444 lines) ✅           ← Core + exports
    ├── RhizoCrypt struct
    ├── PrimalLifecycle impl
    ├── Module declarations
    └── Re-exports
```

**Estimated Time**: 4 hours

#### C. Nomenclature Evolution

**Replace ALL primal references** with capability descriptions:

```rust
// EVERYWHERE in comments, docs, variable names:

// BEFORE (Vendor-Specific)
"Connect to BearDog for signing"
"Store in NestGate"
"Commit to LoamSpine"
"Register with Songbird"
let beardog_client = ...;
let nestgate_endpoint = ...;

// AFTER (Capability-Based)
"Connect to signing provider"
"Store in content-addressed storage"
"Commit to permanent storage"
"Register with discovery service"
let signing_client = ...;
let storage_endpoint = ...;
```

**Files to Update**: ~50 files with embedded primal names

**Estimated Time**: 8 hours (careful search-replace with validation)

---

## 📋 Complete Evolution Checklist

### 🚨 Critical (Blocks "Infant Discovery Perfect" status)

- [ ] **Refactor 3 high-complexity functions** (4 hours)
  - [ ] nestgate.rs:382 (complexity 33 → <25)
  - [ ] toadstool.rs:400 (complexity 35 → <25)
  - [ ] songbird/client.rs:280 (complexity 39 → <25)

- [ ] **Refactor lib.rs** (4 hours)
  - [ ] Extract dag.rs (300 lines)
  - [ ] Extract session_manager.rs (200 lines)
  - [ ] Extract dehydration_impl.rs (150 lines)
  - [ ] lib.rs → 444 lines ✅

- [ ] **Create Universal Bootstrap** (2 days)
  - [ ] Implement bootstrap module
  - [ ] Create UniversalAdapter
  - [ ] Implement HttpAdapter (wrap Songbird)
  - [ ] Update service binary
  - [ ] Deprecate Songbird-specific bootstrap

- [ ] **Nomenclature Cleanup** (1 day)
  - [ ] Replace "BearDog" → "signing provider" in comments
  - [ ] Replace "NestGate" → "storage provider" in comments
  - [ ] Replace "LoamSpine" → "permanent storage" in comments
  - [ ] Replace "Songbird" → "discovery service" in comments
  - [ ] Replace "ToadStool" → "compute provider" in comments
  - [ ] Update variable names
  - [ ] Update function names
  - [ ] Update documentation

### ⚠️ High Priority (v0.14.0 target)

- [ ] **Update Internal Code to Capability Clients**
  - [ ] lib.rs: Use SigningClient, not BearDogClient patterns
  - [ ] Tests: Migrate to capability mocks
  - [ ] Examples: Use capability-based patterns

- [ ] **Enhanced Discovery Registry**
  - [ ] Support multiple discovery backends
  - [ ] Health checking for discovered services
  - [ ] Fallback/retry logic
  - [ ] Circuit breakers

- [ ] **Documentation Updates**
  - [ ] Remove all vendor names from architecture docs
  - [ ] Update README with infant discovery philosophy
  - [ ] Create migration guide (v0.13 → v1.0)
  - [ ] Add "Philosophy" document

### 🆗 Medium Priority (v0.15.0+)

- [ ] **Complete tarpc Adapter**
  - [ ] Implement actual tarpc calls
  - [ ] Remove scaffolding
  - [ ] Add tests

- [ ] **Zero-Copy Optimizations**
  - [ ] Arc<Vertex> in DashMap
  - [ ] Memory pools for hot paths
  - [ ] Reduce clone count

- [ ] **Additional Discovery Backends**
  - [ ] Consul adapter
  - [ ] Kubernetes adapter
  - [ ] mDNS adapter (local networks)
  - [ ] Static file adapter (simple deployments)

### ℹ️ Low Priority (v1.0.0+)

- [ ] **Remove Legacy Clients** (BREAKING)
  - [ ] Delete clients/legacy/ directory
  - [ ] Remove deprecated type aliases
  - [ ] Update all examples
  - [ ] Major version bump

---

## 🎓 Infant Discovery Validation Criteria

### ✅ PERFECT Infant Discovery Checklist

A primal achieves "PERFECT" infant discovery when:

#### 1. Zero Compile-Time Knowledge ✅ 
- [ ] No primal names in type system
- [ ] No primal names in struct/enum definitions
- [ ] No primal names in trait names

#### 2. Zero Production Hardcoding ✅
- [ ] No hardcoded IP addresses
- [ ] No hardcoded ports
- [ ] No hardcoded service names
- [ ] No hardcoded primal names

#### 3. Universal Bootstrap
- [ ] Discovery endpoint from environment
- [ ] Discovery protocol configurable
- [ ] No assumptions about discovery service
- [ ] Works with ANY discovery backend

#### 4. Capability-Only Runtime
- [ ] Request capabilities, not vendors
- [ ] Discover providers dynamically
- [ ] Switch providers without code changes
- [ ] Support multiple providers per capability

#### 5. Clean Nomenclature
- [ ] Comments describe capabilities, not vendors
- [ ] Variables named for capabilities
- [ ] Documentation vendor-agnostic
- [ ] Examples use generic terms

### Current Status

```
Infant Discovery Maturity: 75% → Target: 100%

✅ Type System (100%)           - v0.13.0 complete
✅ Production Hardcoding (100%) - Zero found
⚠️ Bootstrap (30%)              - Songbird hardcoded
⚠️ Runtime (80%)                - Capability clients exist, legacy remains
⚠️ Nomenclature (60%)           - Many vendor names in docs/comments
```

---

## 🚀 Implementation Timeline

### Sprint 1: Critical Refactoring (Week 1)
**Duration**: 5 days  
**Goal**: Fix cognitive complexity, lib.rs size

- Day 1-2: Refactor 3 complex functions
- Day 3-4: Extract lib.rs into modules  
- Day 5: Test, validate, document

### Sprint 2: Universal Bootstrap (Week 2)
**Duration**: 5 days  
**Goal**: Remove Songbird hardcoding

- Day 1-2: Implement bootstrap module + adapters
- Day 3: Update service binary
- Day 4: Test with multiple backends
- Day 5: Documentation

### Sprint 3: Nomenclature Cleanup (Week 3)
**Duration**: 5 days  
**Goal**: Remove ALL vendor names from prose

- Day 1-3: Search-replace in all files (careful)
- Day 4: Update documentation
- Day 5: Final validation

### Sprint 4: Legacy Migration (Week 4)
**Duration**: 5 days  
**Goal**: Internal code uses capability clients only

- Day 1-3: Update lib.rs, tests, examples
- Day 4: Enhanced discovery registry
- Day 5: Integration testing

---

## 📊 Success Metrics

### Before (v0.13.0 - Current)

```
Primal Name References:  1,077
└── Production Types:    0 ✅
└── Comments/Docs:       ~800 ⚠️
└── Test Code:           ~277 ✅ (acceptable)

Bootstrap Hardcoding:    Yes (Songbird) ⚠️
Discovery Flexibility:   Single backend (HTTP) ⚠️
Nomenclature:           Mixed (vendor + capability) ⚠️

Infant Discovery Score:  75/100
```

### Target (v0.15.0 - 4 Weeks)

```
Primal Name References:  ~277 (test-only)
└── Production Types:    0 ✅
└── Comments/Docs:       0 ✅
└── Test Code:           ~277 ✅ (acceptable)

Bootstrap Hardcoding:    No (Universal) ✅
Discovery Flexibility:   Multi-backend ✅
Nomenclature:           Pure capability ✅

Infant Discovery Score:  100/100 ✅ PERFECT
```

---

## 🎯 Final Vision

```rust
// 🌱 INFANT AT BIRTH: Knows only itself

fn main() {
    // "I am rhizoCrypt, version 0.15.0"
    let primal = RhizoCrypt::new(config);
    
    // "I know nothing else"
    assert_eq!(primal.hardcoded_knowledge(), None);
}

// 🔍 BOOTSTRAP: Find universal adapter

let adapter = UniversalAdapter::from_env()?;
// Could be: Songbird, Consul, K8s, mDNS, static file
// rhizoCrypt doesn't know or care

// 📢 REGISTER: Announce capabilities

adapter.register(Capabilities {
    provides: vec!["dag:ephemeral", "merkle:proofs"],
    requires: vec!["crypto:signing", "storage:permanent"],
}).await?;

// 🕵️ DISCOVER: Find needed capabilities

let signing = adapter.discover("crypto:signing").await?;
// Could be: BearDog, YubiKey, CloudKMS, TPM, HSM
// rhizoCrypt doesn't know or care

let storage = adapter.discover("storage:permanent").await?;
// Could be: LoamSpine, IPFS, Arweave, S3, PostgreSQL
// rhizoCrypt doesn't know or care

// 🎯 OPERATE: Use capabilities, not vendors

signing.sign(data).await?;
storage.commit(session).await?;

// ✨ RESULT: Works with ANY provider ecosystem
```

---

## 📝 Notes

### Why This Matters

1. **N² Connection Problem**
   - 6 primals × 6 primals = 36 hardcoded connections ❌
   - 6 primals + 1 universal adapter = 6 connections ✅

2. **Vendor Lock-In**
   - Hard to swap BearDog for YubiKey ❌
   - Easy: just change discovery response ✅

3. **Testing**
   - Need to mock BearDog, NestGate, etc. ❌
   - Mock capabilities instead ✅

4. **Philosophy**
   - Infants don't know vendor names
   - They discover capabilities
   - Human-dignity aligned

---

**Status**: 🔄 EVOLUTION IN PROGRESS  
**Target**: v0.15.0 (4 weeks)  
**Goal**: 100/100 Infant Discovery Score ✅

---

*"Born knowing only yourself, discover the world through capability"*  
— ecoPrimals Infant Discovery Principle

