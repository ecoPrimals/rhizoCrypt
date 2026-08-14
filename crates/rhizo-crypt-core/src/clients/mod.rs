// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2024–2026 ecoPrimals Project

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
//! ```no_run
//! # use rhizo_crypt_core::clients::capabilities::SigningClient;
//! # use rhizo_crypt_core::transport::TransportEndpoint;
//! # use rhizo_crypt_core::types::Did;
//! # use std::sync::Arc;
//! # tokio::runtime::Runtime::new().unwrap().block_on(async {
//! # let registry = Arc::new(rhizo_crypt_core::discovery::DiscoveryRegistry::new("doc-test"));
//! # registry.register_endpoint(rhizo_crypt_core::discovery::ServiceEndpoint::new(
//! #     "test-signer", TransportEndpoint::tcp("127.0.0.1", 9500),
//! #     vec![rhizo_crypt_core::discovery::Capability::Signing],
//! # )).await;
//! // Discover ANY signing provider at runtime
//! let signer = SigningClient::discover(&registry).await?;
//!
//! // Works with any signing provider: HSM, cloud KMS, software signer, etc.
//! let did = Did::new("did:key:test");
//! let signature = signer.sign(b"data", &did).await?;
//! # Ok::<(), rhizo_crypt_core::error::RhizoCryptError>(())
//! # });
//! ```
//!
//! ## Infant Discovery
//!
//! Primals start with zero knowledge and discover services at runtime:
//!
//! 1. **Birth**: Zero knowledge, only self-awareness
//! 2. **Find Universal Adapter**: `DISCOVERY_ADAPTER_ADDRESS` env var
//! 3. **Query Capabilities**: "Who provides crypto:signing?"
//! 4. **Connect On-Demand**: Create clients as needed
//!
//! No compile-time knowledge of other primals' addresses.

// NEW: Capability-based clients (vendor-agnostic)
pub mod adapters;
pub mod capabilities;
pub mod factory;

// tarpc discovery adapter (used by songbird client + service startup tests)
#[cfg(feature = "live-clients")]
#[path = "songbird_rpc.rs"]
pub mod discovery_rpc;

// Universal discovery adapter (bootstrap only)
#[path = "songbird_types.rs"]
pub mod discovery_types;
pub mod songbird;

// Re-exports (capability-based API)
pub use capabilities::{
    ComputeClient, PermanentStorageClient, ProvenanceClient, SigningClient, StorageClient,
};

#[cfg(unix)]
pub use adapters::UnixSocketAdapter;
pub use adapters::{AdapterFactory, ProtocolAdapter};

// Factory for creating and caching capability clients
pub use factory::CapabilityClientFactory;

// Bootstrap/discovery — capability-neutral public API
pub use discovery_types::{ClientState, FederationStatus, RegistrationResult, ServiceInfo};
pub use songbird::{DiscoveryClient, DiscoveryConfig};
