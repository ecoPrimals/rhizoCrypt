# 🌱 Zero-Hardcoding Initiative - Quick Reference

**Date**: December 25, 2025  
**Status**: 📋 Planning Complete, Ready to Execute  
**Timeline**: 4-5 weeks  

---

## 🎯 Goal

**Achieve 100% infant discovery**: rhizoCrypt starts with ZERO knowledge and discovers everything at runtime through the universal adapter (currently Songbird, but could be any discovery service).

---

## 📊 Current State

| Hardcoding Type | Count | Status |
|----------------|-------|--------|
| Primal names | 895 instances | 🔴 HIGH |
| Uppercase constants | 62 instances | 🔴 HIGH |
| Port numbers | 116 instances | 🟡 MEDIUM |
| External services (k8s, consul) | 0 instances | ✅ CLEAN |

---

## 🏗️ Solution Architecture

### Before (Primal-Specific)
```rust
// ❌ Hardcoded primal names
pub struct BearDogClient { }
pub struct NestGateClient { }

// ❌ Primal-specific methods
client.discover_beardog().await?
```

### After (Capability-Based)
```rust
// ✅ Generic capability clients
pub struct SigningClient { }
pub struct StorageClient { }

// ✅ Capability-based discovery
let signer = SigningClient::discover(registry).await?;
// Works with BearDog, YubiKey, CloudKMS, HSM, etc.
```

---

## 🚀 New Directory Structure

```
crates/rhizo-crypt-core/src/clients/
  ├── universal_adapter.rs      # Bootstrap (Songbird or any discovery)
  ├── capabilities/             # Generic capability clients
  │   ├── signing.rs            # ANY signing provider
  │   ├── storage.rs            # ANY storage provider
  │   ├── permanent.rs          # ANY permanent storage
  │   ├── compute.rs            # ANY compute provider
  │   └── provenance.rs         # ANY provenance provider
  ├── adapters/                 # Protocol adapters
  │   ├── tarpc.rs
  │   ├── http.rs
  │   └── grpc.rs
  └── legacy/                   # Deprecated primal-specific
      ├── beardog.rs            # ⚠️ Deprecated
      └── nestgate.rs           # ⚠️ Deprecated
```

---

## 🌱 Infant Bootstrap Flow

```
1. Birth: Zero knowledge, only self-awareness
   ↓
2. Find Universal Adapter: RHIZOCRYPT_DISCOVERY_ADAPTER env var
   ↓
3. Query Adapter: "What capabilities are available?"
   ↓
4. Discover Services: Build capability registry
   ↓
5. Connect On-Demand: Create clients as needed
```

---

## 📋 Migration Steps (4-5 weeks)

| Week | Tasks | Hours |
|------|-------|-------|
| **1** | Create capability structure, generic clients | 40-50 |
| **2** | Update env vars, deprecate primal clients | 32-40 |
| **3** | Update tests, showcase demos | 50-60 |
| **4-5** | Remove hardcoded ports, documentation | 28-35 |

---

## 🎯 Success Criteria

**After migration, this test should pass:**

```rust
#[test]
async fn test_infant_discovery() {
    // Remove ALL primal-specific env vars
    cleanup_all_primal_env_vars();
    
    // Set ONLY the universal adapter address
    std::env::set_var(
        "RHIZOCRYPT_DISCOVERY_ADAPTER", 
        "http://localhost:8091"
    );
    
    // rhizoCrypt bootstraps with zero knowledge
    let mut primal = RhizoCrypt::bootstrap().await.unwrap();
    
    // Discovers and connects to ANY signing provider
    let signing = primal.signing().await.unwrap();
    
    // Works regardless of WHO provides signing
    let sig = signing.sign(b"test", &did).await.unwrap();
    assert!(!sig.as_bytes().is_empty());
}
```

---

## 🎉 Benefits

1. **Vendor Neutrality**: Swap providers without code changes
2. **Simplified Testing**: Generic mocks for all capabilities
3. **Federation-Ready**: Multiple providers for same capability
4. **Future-Proof**: Add new capabilities without primal-specific code
5. **Infant Deployment**: Single env var to bootstrap everything

---

## 📚 Key Documents

- **Full Plan**: `ZERO_HARDCODING_MIGRATION_PLAN.md`
- **Current Audit**: `COMPREHENSIVE_AUDIT_REPORT_DEC_25_2025.md`
- **Architecture**: `specs/ARCHITECTURE.md` (will be updated)

---

## ⚠️ Breaking Changes (v0.11.0)

### Deprecated (Still Works)
- All primal-specific clients (`BearDogClient`, `NestGateClient`, etc.)
- All primal-specific env vars (`BEARDOG_*`, `NESTGATE_*`, etc.)

### Recommended
- Use `capabilities::SigningClient` instead
- Use `RHIZOCRYPT_CAPABILITY_SIGNING` instead

### Removed in v1.0.0
- All deprecated clients moved to `rhizocrypt-legacy` crate

---

## 🚦 Status

- ✅ **Planning**: Complete
- ✅ **Architecture**: Designed
- ⏳ **Implementation**: Ready to start
- ⏳ **Testing**: Pending implementation
- ⏳ **Documentation**: In progress
- ⏳ **Release**: Planned for v0.11.0

---

*"Like an infant, we start with zero knowledge and discover the world around us."* 🌱

