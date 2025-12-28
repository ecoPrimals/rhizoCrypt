//! Client Modules - Capability-Based Service Discovery
//!
//! This module provides clients for interacting with services in the ecosystem.
//!
//! ## Architecture Evolution
//!
//! ### ✅ Capability-Based Clients (Current)
//!
//! Generic clients that work with ANY service providing a capability:
//!
//! ```ignore
//! use rhizo_crypt_core::clients::capabilities::{SigningClient, StorageClient};
//!
//! // Discover ANY signing provider at runtime
//! let signer = SigningClient::discover(&registry).await?;
//!
//! // Works with BearDog, YubiKey, CloudKMS, HSM, etc.
//! let signature = signer.sign(data, &did).await?;
//! ```
//!
//! ## Infant Discovery
//!
//! Primals start with zero knowledge and discover services at runtime:
//!
//! 1. **Birth**: Zero knowledge, only self-awareness
//! 2. **Find Universal Adapter**: DISCOVERY_ADAPTER_ADDRESS env var
//! 3. **Query Capabilities**: "Who provides crypto:signing?"
//! 4. **Connect On-Demand**: Create clients as needed
//!
//! No compile-time knowledge of other primals' addresses.

// NEW: Capability-based clients (vendor-agnostic)
pub mod adapters;
pub mod capabilities;
pub mod factory; // Factory for creating and caching capability clients

// HTTP/RPC client implementations
#[cfg(feature = "live-clients")]
pub mod beardog_http;
#[cfg(feature = "live-clients")]
pub mod loamspine_rpc;
#[cfg(feature = "live-clients")]
pub mod nestgate_http;
#[cfg(feature = "live-clients")]
pub mod songbird_rpc;

// HTTP client for ToadStool (compute)
pub mod toadstool_http;

// Universal adapter (bootstrap only)
pub mod songbird;
pub mod songbird_types;

// Re-exports (capability-based API)
pub use capabilities::{
    ComputeClient, PermanentStorageClient, ProvenanceClient, SigningClient, StorageClient,
};

pub use adapters::{AdapterFactory, ProtocolAdapter};

// Factory for creating and caching capability clients
pub use factory::CapabilityClientFactory;

// Bootstrap/discovery (not deprecated - needed for universal adapter)
pub use songbird::{SongbirdClient, SongbirdConfig};
pub use songbird_types::{ClientState, FederationStatus, RegistrationResult, ServiceInfo};
