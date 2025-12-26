//! Legacy primal-specific clients (DEPRECATED).
//!
//! ⚠️ **WARNING**: These clients are vendor-specific and create lock-in.
//!
//! ## Why Deprecated?
//!
//! These clients hardcode specific primal names (BearDog, NestGate, etc.)
//! which creates vendor lock-in and prevents federation.
//!
//! ## Migration Path
//!
//! | Old (Deprecated) | New (Capability-Based) |
//! |------------------|------------------------|
//! | `BearDogClient` | `capabilities::SigningClient` |
//! | `NestGateClient` | `capabilities::StorageClient` |
//! | `LoamSpineClient` | `capabilities::PermanentStorageClient` |
//! | `ToadStoolClient` | `capabilities::ComputeClient` |
//! | `SweetGrassClient` | `capabilities::ProvenanceClient` |
//!
//! ## Example Migration
//!
//! ```ignore
//! // ❌ OLD: Vendor-specific
//! use rhizo_crypt_core::clients::BearDogClient;
//! let beardog = BearDogClient::discover(&registry).await?;
//!
//! // ✅ NEW: Capability-based
//! use rhizo_crypt_core::clients::capabilities::SigningClient;
//! let signer = SigningClient::discover(&registry).await?;
//! // Works with BearDog, YubiKey, CloudKMS, HSM, etc.
//! ```
//!
//! ## Timeline
//!
//! - **v0.11.0**: Deprecated (still works, warnings)
//! - **v1.0.0**: Moved to separate `rhizocrypt-legacy` crate
//!
//! ## For Existing Code
//!
//! These clients will continue to work in v0.11.x with deprecation warnings.
//! Update to capability-based clients before v1.0.0.

pub mod beardog;
pub mod loamspine;
pub mod nestgate;
pub mod sweetgrass;
pub mod toadstool;
