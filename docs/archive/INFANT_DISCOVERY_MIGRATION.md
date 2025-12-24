# 🔐 rhizoCrypt — Infant Discovery Migration

**Date**: December 24, 2025  
**Status**: In Progress  
**Philosophy**: **Each primal only knows itself**

---

## 🎯 Goal

Evolve rhizoCrypt to **zero vendor/primal hardcoding**:
- No primal names in environment variables
- No primal names in configuration
- No external service names (k8s, consul, etc.)
- Pure capability-based discovery

### Infant Discovery Principle

```
🐣 Birth: Primal starts with ZERO knowledge
📡 Discovery: Universal adapter (Songbird) provides capabilities
🔗 Connection: Connect by capability, not by name
```

**Example**: Instead of "connect to BearDog", think "I need signing, let me discover who provides it"

---

## 📊 Current State Audit

### Environment Variables (Primal Names)
```bash
# Current (HARDCODED ❌):
BEARDOG_ADDRESS=localhost:9500
NESTGATE_ADDRESS=localhost:9600
SONGBIRD_ADDRESS=localhost:8091
LOAMSPINE_ADDRESS=localhost:9700
TOADSTOOL_ADDRESS=localhost:9800
SWEETGRASS_PUSH_ADDRESS=localhost:9900

# Desired (CAPABILITY-BASED ✅):
SIGNING_CAPABILITY_FALLBACK=localhost:9500
PAYLOAD_STORAGE_FALLBACK=localhost:9600
DISCOVERY_SERVICE_ADDRESS=localhost:8091
PERMANENT_STORAGE_FALLBACK=localhost:9700
COMPUTE_ORCHESTRATION_FALLBACK=localhost:9800
PROVENANCE_QUERY_FALLBACK=localhost:9900
```

### Client Module Names
```
Current:
├── clients/beardog.rs          ❌ Primal name
├── clients/nestgate.rs         ❌ Primal name
├── clients/songbird.rs         ❌ Primal name (acceptable - is the universal adapter)
├── clients/loamspine.rs        ❌ Primal name
├── clients/toadstool.rs        ❌ Primal name
├── clients/sweetgrass.rs       ❌ Primal name

Proposed:
├── clients/signing.rs          ✅ Capability-based
├── clients/payload_storage.rs  ✅ Capability-based
├── clients/discovery.rs        ✅ Capability-based (Songbird is special)
├── clients/permanent_storage.rs ✅ Capability-based
├── clients/compute.rs          ✅ Capability-based
├── clients/provenance.rs       ✅ Capability-based
```

### Trait Names
```rust
// Current (❌):
pub trait BearDogClient { ... }
pub trait NestGateClient { ... }
pub trait SongbirdClient { ... }  // Acceptable - universal adapter
pub trait LoamSpineClient { ... }
pub trait ToadStoolClient { ... }
pub trait SweetGrassQueryable { ... }

// Proposed (✅):
pub trait SigningClient { ... }
pub trait PayloadStorageClient { ... }
pub trait DiscoveryClient { ... }
pub trait PermanentStorageClient { ... }
pub trait ComputeClient { ... }
pub trait ProvenanceClient { ... }
```

### External Vendor References
```bash
grep -ri "kubernetes\|consul\|etcd\|k8s\|zookeeper" crates/
# Result: 0 matches ✅ Already clean!
```

---

## 🗺️ Migration Strategy

### Phase 1: Backward-Compatible Evolution (✅ START HERE)

#### Step 1.1: Add Capability-Based Env Vars (Preferred)
```rust
// In SafeEnv or new CapabilityEnv module:

impl CapabilityEnv {
    /// Get signing capability endpoint (preferred method).
    pub fn signing_endpoint() -> Option<String> {
        // Priority order:
        // 1. SIGNING_CAPABILITY_ENDPOINT (new, preferred)
        // 2. BEARDOG_ADDRESS (legacy, deprecated)
        std::env::var("SIGNING_CAPABILITY_ENDPOINT")
            .or_else(|_| {
                tracing::warn!("Using legacy BEARDOG_ADDRESS (deprecated). Use SIGNING_CAPABILITY_ENDPOINT instead.");
                std::env::var("BEARDOG_ADDRESS")
            })
            .ok()
    }
    
    pub fn payload_storage_endpoint() -> Option<String> {
        std::env::var("PAYLOAD_STORAGE_ENDPOINT")
            .or_else(|_| {
                tracing::warn!("Using legacy NESTGATE_ADDRESS (deprecated). Use PAYLOAD_STORAGE_ENDPOINT instead.");
                std::env::var("NESTGATE_ADDRESS")
            })
            .ok()
    }
    
    // ... similar for all capabilities
}
```

#### Step 1.2: Update Configs to Use Capability Env Vars
```rust
// In each client config:
impl SigningConfig {  // (formerly BearDogConfig)
    pub fn from_env() -> Self {
        let mut config = Self::default();
        
        // Use capability-based env var
        if let Some(addr) = CapabilityEnv::signing_endpoint() {
            config.fallback_address = Some(Cow::Owned(addr));
        }
        
        // Timeout can still be signing-specific
        if let Ok(timeout) = std::env::var("SIGNING_TIMEOUT_MS") {
            if let Ok(ms) = timeout.parse() {
                config.timeout_ms = ms;
            }
        }
        
        config
    }
}
```

#### Step 1.3: Update Documentation
- Deprecation notices for old env vars
- Migration guide for users
- Examples using new capability-based vars

### Phase 2: Rename Internal Types (✅ BACKWARD COMPATIBLE)

#### Step 2.1: Create Type Aliases
```rust
// In clients/mod.rs

// NEW: Capability-based names (preferred)
pub use signing::{SigningClient, SigningConfig};
pub use payload_storage::{PayloadStorageClient, PayloadStorageConfig};
pub use permanent_storage::{PermanentStorageClient, PermanentStorageConfig};
pub use compute::{ComputeClient, ComputeConfig};
pub use provenance::{ProvenanceClient, ProvenanceConfig};

// DEPRECATED: Legacy names for backward compatibility
#[deprecated(since = "0.10.0", note = "Use SigningClient instead")]
pub type BearDogClient = SigningClient;
#[deprecated(since = "0.10.0", note = "Use SigningConfig instead")]
pub type BearDogConfig = SigningConfig;

// ... similar for all clients
```

#### Step 2.2: Rename Module Files
```bash
# Keep old names as symlinks or re-exports initially
mv clients/beardog.rs clients/signing.rs
mv clients/nestgate.rs clients/payload_storage.rs
mv clients/loamspine.rs clients/permanent_storage.rs
mv clients/toadstool.rs clients/compute.rs
mv clients/sweetgrass.rs clients/provenance.rs
```

### Phase 3: Update Discovery System (✅ ALREADY MOSTLY DONE)

The discovery system is already capability-based! Just verify:

```rust
// Already good:
pub enum Capability {
    Signing,                    // ✅ No "BearDog" mention
    PermanentCommit,            // ✅ No "LoamSpine" mention
    PayloadStorage,             // ✅ No "NestGate" mention
    ServiceDiscovery,           // ✅ No "Songbird" mention
    ComputeOrchestration,       // ✅ No "ToadStool" mention
    ProvenanceQuery,            // ✅ No "SweetGrass" mention
}
```

### Phase 4: Remove Legacy Names (⚠️ BREAKING, FUTURE)

After deprecation period (e.g., 6 months):
- Remove type aliases
- Remove legacy env var support
- Remove primal-named modules completely

---

## 📝 Implementation Checklist

### Immediate (This Session)

- [x] Audit current hardcoding
- [ ] Create `CapabilityEnv` extension to `SafeEnv`
- [ ] Add capability-based env var support (backward compatible)
- [ ] Update all client configs to prefer new env vars
- [ ] Add deprecation warnings for legacy env vars
- [ ] Update root README with new env vars
- [ ] Update showcase scripts to use new env vars

### Next Sprint

- [ ] Rename client modules (keeping backward compatibility)
- [ ] Rename trait definitions
- [ ] Add type aliases for deprecated names
- [ ] Update all internal references
- [ ] Update integration tests
- [ ] Update documentation

### Future (v1.0.0)

- [ ] Remove legacy env var support
- [ ] Remove type aliases
- [ ] Remove deprecated modules
- [ ] Update ecosystem-wide (coordinate with other primals)

---

## 🎓 Migration Guide for Users

### For Developers

**Before**:
```bash
export BEARDOG_ADDRESS=localhost:9500
export NESTGATE_ADDRESS=localhost:9600
export LOAMSPINE_ADDRESS=localhost:9700
```

**After** (v0.10.0+):
```bash
export SIGNING_CAPABILITY_ENDPOINT=localhost:9500
export PAYLOAD_STORAGE_ENDPOINT=localhost:9600
export PERMANENT_STORAGE_ENDPOINT=localhost:9700
```

### For Operators

**Before**:
```rust
use rhizo_crypt_core::clients::{BearDogClient, NestGateClient};
```

**After** (v0.10.0+):
```rust
use rhizo_crypt_core::clients::{SigningClient, PayloadStorageClient};
```

**Transition** (both work during deprecation):
```rust
// Old code keeps working (with deprecation warning)
use rhizo_crypt_core::clients::BearDogClient;

// New code (preferred)
use rhizo_crypt_core::clients::SigningClient;
```

---

## 🔍 Verification Steps

### 1. No Primal Names in Prod Code
```bash
# Should find ZERO matches in production code:
grep -r "BEARDOG\|LOAMSPINE\|NESTGATE\|TOADSTOOL\|SWEETGRASS" crates/*/src \
  --exclude-dir=tests \
  --include="*.rs" \
  | grep -v "deprecated\|legacy\|backward"
```

### 2. All Env Vars Capability-Based
```bash
# Should find ZERO legacy env vars in production:
grep -r "BEARDOG_\|NESTGATE_\|LOAMSPINE_\|TOADSTOOL_\|SWEETGRASS_" crates/*/src \
  --include="*.rs" \
  | grep -v "deprecated\|legacy"
```

### 3. Tests Still Pass
```bash
cargo test --workspace
cargo clippy --workspace -- -D warnings
cargo doc --workspace --no-deps
```

---

## 🌟 Benefits

### For rhizoCrypt
- ✅ Pure infant discovery (no hardcoded knowledge)
- ✅ Easier testing (mock any capability)
- ✅ More maintainable (capabilities don't change, implementations do)
- ✅ Ecosystem-agnostic (works with any capability provider)

### For Ecosystem
- ✅ Consistent patterns across all primals
- ✅ Easier onboarding (understand capabilities, not primal names)
- ✅ Flexible deployment (swap implementations without code changes)
- ✅ Future-proof (new primals integrate seamlessly)

---

## 📚 Related Documents

- [ARCHITECTURE.md](./specs/ARCHITECTURE.md) - Primal-agnostic design
- [START_HERE.md](./START_HERE.md) - Developer guide
- [SafeEnv module](./crates/rhizo-crypt-core/src/safe_env.rs) - Type-safe config

---

*"An infant discovers the world not by memorizing names, but by exploring capabilities."* 🐣

