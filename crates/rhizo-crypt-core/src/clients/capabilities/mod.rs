//! Capability-based clients - protocol and vendor agnostic.
//!
//! This module provides generic clients that work with ANY service providing
//! a given capability, regardless of who operates the service or what protocol
//! they use.
//!
//! ## Philosophy
//!
//! Clients should be capability-first, not vendor-specific:
//!
//! - ✅ `SigningClient` - Works with ANY signing provider
//! - ❌ `BearDogClient` - Locks you into one vendor
//!
//! ## Usage
//!
//! ```ignore
//! use rhizo_crypt_core::clients::capabilities::SigningClient;
//! use rhizo_crypt_core::discovery::DiscoveryRegistry;
//!
//! // Discover and connect to ANY signing provider
//! let registry = DiscoveryRegistry::new();
//! let signer = SigningClient::discover(&registry).await?;
//!
//! // Works with BearDog, YubiKey, CloudKMS, HSM, etc.
//! let signature = signer.sign(data, &did).await?;
//! ```

pub mod signing;
pub mod storage;
pub mod permanent;
pub mod compute;
pub mod provenance;

// Re-exports
pub use signing::SigningClient;
pub use storage::StorageClient;
pub use permanent::PermanentStorageClient;
pub use compute::ComputeClient;
pub use provenance::ProvenanceClient;

