// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2024–2026 ecoPrimals Project

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
//! - ❌ Vendor-specific clients - Lock you into one provider
//!
//! ## Usage
//!
//! ```no_run
//! # use rhizo_crypt_core::clients::capabilities::SigningClient;
//! # use rhizo_crypt_core::discovery::DiscoveryRegistry;
//! # use rhizo_crypt_core::types::Did;
//! # use std::sync::Arc;
//! # tokio::runtime::Runtime::new().unwrap().block_on(async {
//! # let registry = Arc::new(DiscoveryRegistry::new("doc-test"));
//! # registry.register_endpoint(rhizo_crypt_core::discovery::ServiceEndpoint::new(
//! #     "test-signer", "127.0.0.1:9500".parse().unwrap(),
//! #     vec![rhizo_crypt_core::discovery::Capability::Signing],
//! # )).await;
//! // Discover and connect to ANY signing provider
//! let signer = SigningClient::discover(&registry).await?;
//!
//! // Works with BearDog, YubiKey, CloudKMS, HSM, etc.
//! let did = Did::new("did:key:test");
//! let signature = signer.sign(b"data", &did).await?;
//! # Ok::<(), rhizo_crypt_core::error::RhizoCryptError>(())
//! # });
//! ```

pub mod compute;
pub mod permanent;
pub mod provenance;
pub mod signing;
pub mod storage;

// Re-exports
pub use compute::ComputeClient;
pub use permanent::PermanentStorageClient;
pub use provenance::ProvenanceClient;
pub use signing::SigningClient;
pub use storage::StorageClient;
