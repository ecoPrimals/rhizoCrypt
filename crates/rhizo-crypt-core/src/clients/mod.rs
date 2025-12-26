//! Client Modules - Capability-Based Service Discovery
//!
//! This module provides clients for interacting with services in the ecosystem.
//!
//! ## Architecture Evolution
//!
//! ### ✅ NEW: Capability-Based Clients (Recommended)
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
//! ### ⚠️ DEPRECATED: Primal-Specific Clients (Legacy)
//!
//! Vendor-specific clients create lock-in. Use capability clients instead.
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

// LEGACY: Primal-specific clients (vendor-specific, deprecated)
pub mod legacy;

// HTTP/RPC client implementations (used by legacy clients)
#[cfg(feature = "live-clients")]
pub mod beardog_http;
#[cfg(feature = "live-clients")]
pub mod loamspine_rpc;
#[cfg(feature = "live-clients")]
pub mod nestgate_http;
#[cfg(feature = "live-clients")]
pub mod songbird_rpc;

// HTTP client for ToadStool
pub mod toadstool_http;

// Universal adapter (bootstrap only - could be Songbird, Consul, etcd, etc.)
pub mod songbird;
pub mod songbird_types;

// Re-exports (NEW capability-based API)
pub use capabilities::{
    ComputeClient, PermanentStorageClient, ProvenanceClient, SigningClient, StorageClient,
};

pub use adapters::{AdapterFactory, ProtocolAdapter};

// Factory for creating and caching capability clients
pub use factory::CapabilityClientFactory;

// Re-exports (LEGACY primal-specific API - deprecated)
#[deprecated(since = "0.11.0", note = "Use capabilities::SigningClient instead")]
pub use legacy::beardog::{BearDogClient, BearDogConfig};

#[deprecated(since = "0.11.0", note = "Use capabilities::StorageClient instead")]
pub use legacy::nestgate::{NestGateClient, NestGateConfig};

#[deprecated(since = "0.11.0", note = "Use capabilities::PermanentStorageClient instead")]
pub use legacy::loamspine::{LoamSpineClient, LoamSpineConfig};

#[deprecated(since = "0.11.0", note = "Use capabilities::ComputeClient instead")]
pub use legacy::toadstool::{ToadStoolClient, ToadStoolConfig};

#[deprecated(since = "0.11.0", note = "Use capabilities::ProvenanceClient instead")]
pub use legacy::sweetgrass::{
    AgentContribution, ProvenanceChain, SessionAttribution, SweetGrassConfig, SweetGrassNotifier,
    SweetGrassQueryable, VertexQuery, VertexRef,
};

// Bootstrap/discovery (not deprecated - needed for universal adapter)
pub use songbird::{SongbirdClient, SongbirdConfig};
pub use songbird_types::{ClientState, FederationStatus, RegistrationResult, ServiceInfo};
