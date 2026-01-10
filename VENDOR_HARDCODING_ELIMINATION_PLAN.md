# 🚀 Vendor Hardcoding Elimination - True Infant Discovery

**Date:** January 9, 2026  
**Status:** Ready for Implementation  
**Goal:** Zero knowledge at birth, discover everything via universal adapter (Songbird)

---

## 🎯 Philosophy: The Infant Model

**Current State:** rhizoCrypt knows **itself only** + discovers via Songbird  
**Target State:** rhizoCrypt knows **itself only** (zero hardcoded primal names anywhere)

```
Birth → Zero Knowledge
   ↓
Bootstrap → Find Universal Adapter (Songbird)
   ↓
Discovery → Request Capabilities (not vendors!)
   ↓
Operation → Use discovered services
```

---

## 📊 Current Hardcoding Audit

### ✅ ALREADY CLEAN (No Action Needed)

1. **Production Code Architecture**
   - ✅ Core engine: 100% capability-based
   - ✅ Client factory: Generic capability clients
   - ✅ Discovery system: Abstract capability enum
   - ✅ Integration traits: Pure capability interfaces

2. **Numeric Hardcoding**
   - ✅ All constants in `constants.rs`
   - ✅ DEFAULT_RPC_PORT = 0 (OS-assigned)
   - ✅ DEFAULT_RPC_HOST = "127.0.0.1" (dev only)
   - ✅ Production uses environment variables
   - ✅ 743 hardcoded IPs/ports ALL in tests/examples (appropriate)

3. **External Service Names**
   - ✅ Kubernetes: Only in docs/deployment manifests (appropriate)
   - ✅ No Consul, Etcd, Zookeeper references
   - ✅ Service mesh agnostic

---

## ⚠️ REMAINING VENDOR HARDCODING

### Category 1: Documentation & Comments (Low Priority)

**Files:** 413 matches across 38 files  
**Context:** Doc strings, comments, examples explaining integration

**Examples:**
```rust
//! Works with BearDog, YubiKey, CloudKMS, HSM, etc.
//! A slice of LoamSpine state checked out into RhizoCrypt.
/// ToadStool HTTP Client - Live Integration
```

**Action:** 
- Keep most (they document what services CAN work, not what MUST work)
- Update where it implies hardcoding
- Add disclaimer: "Example services - any provider implementing the trait works"

---

### Category 2: Type Names (Deprecated Aliases) ✅ ALREADY HANDLED

**Files:** `legacy_aliases.rs`, `integration/mocks.rs`

**Current State:**
```rust
#[deprecated(since = "0.14.0")]
pub type ToadStoolConfig = ComputeProviderConfig;

#[deprecated(since = "0.13.0")]
pub type MockBearDogClient = MockSigningProvider;
```

**Status:** ✅ Already deprecated with migration path
**Timeline:** Remove in v1.0.0 (breaking change)

---

### Category 3: Discovery Helper Methods ⚠️ **NEEDS EVOLUTION**

**File:** `discovery.rs`  
**Lines:** 330-370

**Current Code:**
```rust
impl CapabilityChecker {
    /// Check if BearDog capabilities are available.
    pub async fn has_beardog(&self) -> bool {
        self.registry.is_available(&Capability::DidVerification).await
    }
    
    /// Check if LoamSpine capabilities are available.
    pub async fn has_loamspine(&self) -> bool {
        self.registry.is_available(&Capability::PermanentCommit).await
    }
    
    /// Get the BearDog endpoint if available.
    pub async fn get_beardog_endpoint(&self) -> Result<SocketAddr> {
        // ...
    }
}
```

**Problem:** Method names hardcode primal names (convenience, but not infant discovery)

**Solution:** Deprecate + provide capability-based alternatives

```rust
impl CapabilityChecker {
    // ========== NEW: Capability-Based (Preferred) ==========
    
    /// Check if signing capabilities are available.
    pub async fn has_signing(&self) -> bool {
        self.registry.is_available(&Capability::Signing).await
    }
    
    /// Check if permanent storage capabilities are available.
    pub async fn has_permanent_storage(&self) -> bool {
        self.registry.is_available(&Capability::PermanentCommit).await
    }
    
    /// Get signing provider endpoint.
    pub async fn get_signing_endpoint(&self) -> Result<SocketAddr> {
        self.registry.get_endpoint(&Capability::Signing).await
    }
    
    // ========== DEPRECATED: Vendor-Specific (Backward Compat) ==========
    
    #[deprecated(since = "0.15.0", note = "Use has_signing() instead")]
    pub async fn has_beardog(&self) -> bool {
        self.has_signing().await
    }
    
    #[deprecated(since = "0.15.0", note = "Use has_permanent_storage() instead")]
    pub async fn has_loamspine(&self) -> bool {
        self.has_permanent_storage().await
    }
}
```

---

### Category 4: Test Names & Examples ✅ ACCEPTABLE

**Context:** Test function names, showcase scripts

**Examples:**
```rust
#[tokio::test]
async fn test_mock_beardog_client() { ... }

async fn test_mock_loamspine_client() { ... }
```

**Status:** ✅ Acceptable - tests verify specific integrations  
**Action:** None (tests are allowed to reference specific services)

---

### Category 5: Songbird Special Case ✅ CORRECT

**File:** `safe_env.rs`, multiple locations

```rust
// Songbird is special - it's the universal adapter
std::env::var("SONGBIRD_ADDRESS").ok()
```

**Status:** ✅ This is CORRECT per infant discovery model  
**Reason:** Songbird IS the universal adapter - the ONE thing an infant needs to bootstrap

**The Only Acceptable Hardcoding:**
- Songbird as the universal discovery adapter
- This is by design, not a violation

---

## 🔧 IMPLEMENTATION PLAN

### Phase 1: Deprecate Vendor-Specific Discovery Methods ⚡ (PRIORITY)

**Files to Update:**
1. `crates/rhizo-crypt-core/src/discovery.rs`

**Changes:**
```rust
// Add new capability-based methods
// Deprecate primal-name methods
// Keep backward compatibility
```

**Timeline:** This session (30 minutes)

---

### Phase 2: Update Documentation Strings 📝

**Files:** All 38 files with primal names in comments

**Changes:**
- Add "Example:" prefix where listing specific services
- Add disclaimer about capability-based discovery
- Keep examples (they're helpful!)

**Timeline:** Next session (60 minutes)

---

### Phase 3: Migration Guide 📚

**Create:** `INFANT_DISCOVERY_GUIDE.md`

**Content:**
- Philosophy explanation
- Migration examples
- Capability mapping table
- FAQ

**Timeline:** Next session (30 minutes)

---

### Phase 4: Remove Deprecated Aliases (v1.0.0) 🔥

**Breaking Change** - Schedule for v1.0.0 release

**Files:**
- `legacy_aliases.rs` - Remove entirely
- `integration/mocks.rs` - Remove deprecated type aliases

**Timeline:** v1.0.0 release (future)

---

## 🎯 SUCCESS CRITERIA

### Immediate (v0.15.0)
- [ ] No vendor-specific discovery methods (all deprecated)
- [ ] All new code uses capability-based discovery
- [ ] Migration guide published
- [ ] Backward compatibility maintained

### Future (v1.0.0)
- [ ] All deprecated aliases removed
- [ ] Zero vendor names in production code
- [ ] Only capability references remain

---

## 📋 MIGRATION CHECKLIST

### For rhizoCrypt Developers
- [ ] Update `discovery.rs` with capability-based methods
- [ ] Deprecate vendor-specific helpers
- [ ] Add migration guide
- [ ] Update examples to show capability-based approach

### For rhizoCrypt Users
- [ ] Review deprecation warnings in v0.15.0
- [ ] Update to capability-based methods
- [ ] Test with v0.15.0 before v1.0.0

---

## 🏆 COMPARISON: Before vs After

### Before (Mixed State)
```rust
// Some capability-based
let signer = SigningClient::discover(&registry).await?;

// Some vendor-specific
if checker.has_beardog().await {  // ❌ Hardcoded
    let endpoint = checker.get_beardog_endpoint().await?;
}
```

### After (Pure Infant Discovery)
```rust
// ALL capability-based
let signer = SigningClient::discover(&registry).await?;

if checker.has_signing().await {  // ✅ Agnostic
    let endpoint = checker.get_signing_endpoint().await?;
}
```

---

## 🎯 FINAL STATE: True Infant Discovery

```
rhizoCrypt at Birth:
├── Knows: Self (name: "RhizoCrypt")
├── Knows: Universal Adapter (SONGBIRD_ADDRESS env var)
└── Knows: Nothing else

rhizoCrypt after Bootstrap:
├── Discovered: Signing provider(s)
├── Discovered: Storage provider(s)
├── Discovered: Compute provider(s)
└── Operates: With any implementation of required capabilities
```

**Zero Hardcoding. Zero Vendor Lock-In. Infinite Flexibility.** 🚀

---

**Ready to implement Phase 1?** ✅
