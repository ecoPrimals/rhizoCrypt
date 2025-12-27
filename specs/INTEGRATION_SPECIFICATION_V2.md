# RhizoCrypt — Integration Specification v2.0

**Version**: 2.0.0 🥇  
**Status**: Current (Capability-Based Architecture)  
**Last Updated**: December 26, 2025

---

## 🎯 Overview

RhizoCrypt v0.13.0+ uses a **capability-based integration architecture** that eliminates vendor hardcoding and enables true infant discovery.

### Philosophy Evolution

**v1.0 (Legacy)**: Primal-specific trait names  
**v2.0 (Current)**: Capability-based provider traits 🥇

```
┌─────────────────────────────────────────────────────────────────┐
│                        RhizoCrypt                                │
│                   (Capability-Based Core)                        │
├─────────────────────────────────────────────────────────────────┤
│                                                                  │
│                    ┌─────────────────┐                          │
│                    │  RhizoCrypt     │                          │
│                    │    Core         │                          │
│                    └────────┬────────┘                          │
│                             │                                    │
│         ┌───────────────────┼───────────────────┐               │
│         │                   │                   │               │
│    ┌────▼────┐        ┌────▼────┐        ┌────▼────┐           │
│    │Signing  │        │Permanent│        │Payload  │           │
│    │Provider │        │Storage  │        │Storage  │           │
│    │Adapter  │        │Provider │        │Provider │           │
│    └────┬────┘        └────┬────┘        └────┬────┘           │
│         │                   │                   │               │
└─────────┼───────────────────┼───────────────────┼───────────────┘
          │                   │                   │
          │ Discovery via Songbird                │
          │                   │                   │
          ▼                   ▼                   ▼
     Any Signing         Any Storage        Any Storage
     Service             Service            Service
     (BearDog,          (LoamSpine,        (NestGate,
      YubiKey,           Arweave,           S3,
      CloudKMS...)       IPFS...)           Azure...)
```

---

## 1. Signing Provider

Any service that implements cryptographic signing capabilities.

### 1.1 Provider Interface

```rust
/// Signing provider trait - capability-based (v2.0) 🥇
///
/// Any service can implement this trait to provide signing capabilities.
/// rhizoCrypt discovers providers at runtime via Songbird.
#[async_trait]
pub trait SigningProvider: Send + Sync {
    // ==================== Identity ====================
    
    /// Verify that a DID is valid and active
    async fn verify_did(&self, did: &Did) -> Result<bool>;
    
    // ==================== Signing ====================
    
    /// Sign data with a DID's key
    async fn sign(&self, data: &[u8], did: &Did) -> Result<Signature>;
    
    /// Sign a vertex
    async fn sign_vertex(&self, vertex: &Vertex, did: &Did) -> Result<Signature>;
    
    /// Verify a vertex signature
    async fn verify_vertex_signature(&self, vertex: &Vertex) -> Result<bool>;
    
    // ==================== Attestations ====================
    
    /// Request an attestation from another party
    async fn request_attestation(
        &self,
        attester: &Did,
        summary: &DehydrationSummary,
    ) -> Result<Attestation>;
}
```

### 1.2 Example Implementations

**BearDog** (HSM-based signing):
```rust
struct BearDogSigningAdapter { /* ... */ }

impl SigningProvider for BearDogSigningAdapter {
    async fn sign(&self, data: &[u8], did: &Did) -> Result<Signature> {
        // HSM signing
    }
}
```

**YubiKey** (Hardware token):
```rust
struct YubiKeySigningAdapter { /* ... */ }

impl SigningProvider for YubiKeySigningAdapter {
    async fn sign(&self, data: &[u8], did: &Did) -> Result<Signature> {
        // Hardware token signing
    }
}
```

**CloudKMS** (Cloud provider):
```rust
struct CloudKMSSigningAdapter { /* ... */ }

impl SigningProvider for CloudKMSSigningAdapter {
    async fn sign(&self, data: &[u8], did: &Did) -> Result<Signature> {
        // Cloud KMS signing
    }
}
```

### 1.3 Discovery

```rust
use rhizo_crypt_core::{SigningProvider, discovery::SigningClient};

// Discover ANY signing provider at runtime
let signer: Box<dyn SigningProvider> = SigningClient::discover(&registry).await?;

// Use it - code doesn't know or care which provider
let signature = signer.sign(data, &did).await?;
```

### 1.4 Backward Compatibility

```rust
// OLD (still works with deprecation warning)
#[deprecated(since = "0.13.0", note = "Use SigningProvider instead")]
pub type SigningProvider = dyn SigningProvider;

// Existing code continues to work:
#[allow(deprecated)]
let client: Box<dyn SigningProvider> = create_client();
```

---

## 2. Permanent Storage Provider

Any service that implements immutable, permanent storage capabilities.

### 2.1 Provider Interface

```rust
/// Permanent storage provider trait - capability-based (v2.0) 🥇
///
/// Any service can implement this trait to provide permanent storage.
/// rhizoCrypt discovers providers at runtime via Songbird.
#[async_trait]
pub trait PermanentStorageProvider: Send + Sync {
    // ==================== Commit ====================
    
    /// Commit a dehydration summary to permanent storage
    async fn commit(&self, summary: &DehydrationSummary) -> Result<CommitRef>;
    
    /// Verify a commit exists and is valid
    async fn verify_commit(&self, commit_ref: &CommitRef) -> Result<bool>;
    
    /// Get commit data
    async fn get_commit(&self, commit_ref: &CommitRef) -> Result<Option<DehydrationSummary>>;
    
    // ==================== Slice Operations ====================
    
    /// Checkout a slice from permanent storage
    async fn checkout_slice(
        &self,
        spine_id: &str,
        entry_hash: &[u8; 32],
        holder: &Did,
    ) -> Result<SliceOrigin>;
    
    /// Resolve a slice (return or commit)
    async fn resolve_slice(
        &self,
        slice: &Slice,
        outcome: &ResolutionOutcome,
    ) -> Result<()>;
}
```

### 2.2 Example Implementations

**LoamSpine** (Blockchain-based):
```rust
struct LoamSpineAdapter { /* ... */ }

impl PermanentStorageProvider for LoamSpineAdapter {
    async fn commit(&self, summary: &DehydrationSummary) -> Result<CommitRef> {
        // Blockchain commit
    }
}
```

**Arweave** (Permaweb):
```rust
struct ArweaveAdapter { /* ... */ }

impl PermanentStorageProvider for ArweaveAdapter {
    async fn commit(&self, summary: &DehydrationSummary) -> Result<CommitRef> {
        // Arweave permanent storage
    }
}
```

**IPFS** (Content-addressed):
```rust
struct IPFSAdapter { /* ... */ }

impl PermanentStorageProvider for IPFSAdapter {
    async fn commit(&self, summary: &DehydrationSummary) -> Result<CommitRef> {
        // IPFS pinning
    }
}
```

### 2.3 Discovery

```rust
use rhizo_crypt_core::{PermanentStorageProvider, discovery::PermanentStorageClient};

// Discover ANY permanent storage provider at runtime
let storage: Box<dyn PermanentStorageProvider> = 
    PermanentStorageClient::discover(&registry).await?;

// Commit - code doesn't know which provider
let commit_ref = storage.commit(&summary).await?;
```

### 2.4 Backward Compatibility

```rust
// OLD (still works with deprecation warning)
#[deprecated(since = "0.13.0", note = "Use PermanentStorageProvider instead")]
pub type PermanentStorageProvider = dyn PermanentStorageProvider;
```

---

## 3. Payload Storage Provider

Any service that implements ephemeral blob storage capabilities.

### 3.1 Provider Interface

```rust
/// Payload storage provider trait - capability-based (v2.0) 🥇
///
/// Any service can implement this trait to provide payload storage.
/// rhizoCrypt discovers providers at runtime via Songbird.
#[async_trait]
pub trait PayloadStorageProvider: Send + Sync {
    /// Store a payload and return its reference
    async fn put_payload(&self, data: Bytes) -> Result<PayloadRef>;
    
    /// Retrieve a payload by reference
    async fn get_payload(&self, payload_ref: &PayloadRef) -> Result<Option<Bytes>>;
    
    /// Check if a payload exists
    async fn payload_exists(&self, payload_ref: &PayloadRef) -> Result<bool>;
    
    /// Delete a payload (if provider supports it)
    async fn delete_payload(&self, payload_ref: &PayloadRef) -> Result<()>;
}
```

### 3.2 Example Implementations

**NestGate** (Native storage):
```rust
struct NestGateAdapter { /* ... */ }

impl PayloadStorageProvider for NestGateAdapter {
    async fn put_payload(&self, data: Bytes) -> Result<PayloadRef> {
        // NestGate storage
    }
}
```

**S3** (Cloud storage):
```rust
struct S3Adapter { /* ... */ }

impl PayloadStorageProvider for S3Adapter {
    async fn put_payload(&self, data: Bytes) -> Result<PayloadRef> {
        // S3 upload
    }
}
```

**Azure Blob Storage**:
```rust
struct AzureAdapter { /* ... */ }

impl PayloadStorageProvider for AzureAdapter {
    async fn put_payload(&self, data: Bytes) -> Result<PayloadRef> {
        // Azure blob storage
    }
}
```

### 3.3 Discovery

```rust
use rhizo_crypt_core::{PayloadStorageProvider, discovery::StorageClient};

// Discover ANY payload storage provider at runtime
let storage: Box<dyn PayloadStorageProvider> = 
    StorageClient::discover(&registry).await?;

// Store - code doesn't know which provider
let payload_ref = storage.put_payload(data).await?;
```

### 3.4 Backward Compatibility

```rust
// OLD (still works with deprecation warning)
#[deprecated(since = "0.13.0", note = "Use PayloadStorageProvider instead")]
pub type PayloadStorageProvider = dyn PayloadStorageProvider;
```

---

## 4. Integration Patterns

### 4.1 Infant Discovery Pattern

```rust
// Birth: Zero knowledge
let mut primal = RhizoCrypt::new(config);

// Bootstrap: Find universal adapter (Songbird)
let registry = DiscoveryRegistry::bootstrap().await?;

// Discovery: Query for capabilities (not vendors!)
let signer = registry.discover(&Capability::Signing).await?;
let storage = registry.discover(&Capability::PermanentStorage).await?;

// Operate: Use capabilities
let signature = signer.sign(data, &did).await?;
let commit = storage.commit(&summary).await?;
```

### 4.2 Federation Pattern

```rust
// Discover ALL providers for a capability
let signers = registry.discover_all(&Capability::Signing).await?;

// Choose based on criteria
let fastest = signers.iter()
    .min_by_key(|s| s.latency())
    .unwrap();

let most_secure = signers.iter()
    .max_by_key(|s| s.security_level())
    .unwrap();
```

### 4.3 Fallback Pattern

```rust
// Try primary, fallback to secondary
let storage = match registry.discover(&Capability::PayloadStorage).await {
    Ok(provider) => provider,
    Err(_) => {
        // Fallback to local storage
        Box::new(LocalStorageProvider::new())
    }
};
```

---

## 5. Migration from v1.0

### 5.1 Quick Migration

**Find & Replace**:
```bash
sed -i 's/SigningProvider/SigningProvider/g' *.rs
sed -i 's/PermanentStorageProvider/PermanentStorageProvider/g' *.rs
sed -i 's/PayloadStorageProvider/PayloadStorageProvider/g' *.rs
```

### 5.2 Gradual Migration

**Phase 1**: Allow deprecated names
```rust
#[allow(deprecated)]
use rhizo_crypt_core::SigningProvider;  // ⚠️ Works with warning
```

**Phase 2**: Update to new names
```rust
use rhizo_crypt_core::SigningProvider;  // ✅ Future-proof
```

---

## 6. Benefits of v2.0

### 6.1 Zero Vendor Lock-In 🥇

```rust
// v1.0: Tied to BearDog
let client: Box<dyn SigningProvider> = /* ... */;  // ❌ Vendor lock-in

// v2.0: Works with ANY provider
let provider: Box<dyn SigningProvider> = /* ... */;  // ✅ Vendor agnostic
```

### 6.2 Federation Support 🥇

```rust
// Multiple providers simultaneously
let signers = vec![
    Box::new(BearDogAdapter::new()),    // HSM
    Box::new(YubiKeyAdapter::new()),    // Hardware token
    Box::new(CloudKMSAdapter::new()),   // Cloud KMS
];

// Choose dynamically
let best = choose_best_signer(&signers, &criteria);
```

### 6.3 True Infant Discovery 🥇

```rust
// v1.0: Compile-time knowledge of BearDog
// v2.0: Zero compile-time knowledge, discover at runtime
let signer = discover_signing().await?;  // Could be anything!
```

### 6.4 Easy Testing 🥇

```rust
#[cfg(test)]
use rhizo_crypt_core::MockSigningProvider;  // Clean, capability-based

let mock = MockSigningProvider::permissive();
assert!(mock.sign(data, &did).await.is_ok());
```

---

## 7. Specification Status

| Component | v1.0 (Legacy) | v2.0 (Current) 🥇 | Status |
|-----------|--------------|-------------------|--------|
| Signing | `SigningProvider` | `SigningProvider` | ✅ Complete |
| Permanent Storage | `PermanentStorageProvider` | `PermanentStorageProvider` | ✅ Complete |
| Payload Storage | `PayloadStorageProvider` | `PayloadStorageProvider` | ✅ Complete |
| Compute | `ToadStoolClient` | `ComputeProvider` | 📋 Planned |
| Provenance | `SweetGrassQueryable` | `ProvenanceProvider` | 📋 Planned |

**Current Version**: v2.0 (Capability-Based) 🥇  
**Backward Compatibility**: 100% (v1.0 names still work)  
**Migration Timeline**: 
- v0.13.0: Both versions supported
- v0.14.0-0.99.0: Gradual deprecation
- v1.0.0: v1.0 names removed (breaking change)

---

## 8. References

- **Implementation**: `crates/rhizo-crypt-core/src/integration/mod.rs`
- **Mocks**: `crates/rhizo-crypt-core/src/integration/mocks.rs`
- **Discovery**: `crates/rhizo-crypt-core/src/discovery/`
- **Capability Clients**: `crates/rhizo-crypt-core/src/clients/capabilities/`
- **Migration Guide**: `MIGRATION_QUICK_REFERENCE.md`
- **Full Report**: `HARDCODING_ELIMINATION_COMPLETE.md`

---

**Version**: 2.0.0 🥇  
**Status**: Production Ready - Ecosystem Leader  
**Philosophy**: Request capabilities, not vendors  
**Last Updated**: December 26, 2025

