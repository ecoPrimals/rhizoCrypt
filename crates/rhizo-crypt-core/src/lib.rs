// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2024–2026 ecoPrimals Project

//! # RhizoCrypt
//!
//! Core DAG Engine - Ephemeral Working Memory
//!
//! ## Overview
//!
//! RhizoCrypt is the ephemeral DAG engine of the ecoPrimals ecosystem. It provides
//! git-like functionality for capturing, linking, and eventually committing events
//! to permanent storage via capability-discovered providers.
//!
//! ## Key Concepts
//!
//! - **Vertex**: A single event in the DAG, content-addressed by Blake3 hash
//! - **Session**: A scoped DAG with lifecycle (create → grow → resolve → expire)
//! - **Dehydration**: The process of committing DAG results to permanent storage
//! - **Slice**: A "checkout" of permanent storage state into the DAG for async operations
//!
//! ## Quick Start
//!
//! ```rust
//! use rhizo_crypt_core::{PrimalLifecycle, RhizoCrypt, RhizoCryptConfig};
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let config = RhizoCryptConfig::default();
//! let mut primal = RhizoCrypt::new(config);
//! primal.start().await?;
//! # Ok(())
//! # }
//! ```
//!
//! ## Architecture
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────────┐
//! │                        RhizoCrypt                                │
//! │                     (Core DAG Engine)                            │
//! │                                                                  │
//! │  ┌─────────┐  ┌─────────┐  ┌─────────┐  ┌─────────────┐        │
//! │  │ Vertex  │  │  DAG    │  │ Merkle  │  │  Sessions   │        │
//! │  │ Store   │  │ Index   │  │ Trees   │  │  (scopes)   │        │
//! │  └─────────┘  └─────────┘  └─────────┘  └─────────────┘        │
//! └─────────────────────────────────────────────────────────────────┘
//! ```

#![cfg_attr(not(test), forbid(unsafe_code))]
#![cfg_attr(test, expect(clippy::unwrap_used, reason = "tests use unwrap for concise assertions"))]
#![cfg_attr(
    test,
    expect(clippy::expect_used, reason = "tests use expect for descriptive failures")
)]

// ============================================================================
// Module Declarations
// ============================================================================

pub mod clients;
pub mod config;
pub mod constants;
pub mod dehydration;
pub mod dehydration_wire;
pub mod discovery;
pub mod error;
pub mod event;
pub mod integration;
pub mod merkle;
pub mod metrics;
pub mod niche;
pub mod primal;
pub mod rhizocrypt;
pub mod safe_env;
pub mod session;
pub mod slice;
pub mod store;
pub mod transport;
pub mod types;
pub mod types_ecosystem;
pub mod validation;
pub mod vertex;

#[cfg(test)]
pub mod testing;

// Optional storage backends
#[cfg(feature = "redb")]
pub mod store_redb;
#[cfg(feature = "sled")]
pub mod store_sled;

// ============================================================================
// Core Re-exports
// ============================================================================

// Configuration
pub use config::{
    DehydrationClientConfig, MetricsConfig, RhizoCryptConfig, RpcConfig, SliceConfig,
    StorageBackend, StorageConfig,
};

// Dehydration
pub use dehydration::{
    Attestation, DehydrationConfig, DehydrationStatus, DehydrationSummary, ResultEntry,
};

// Discovery
pub use discovery::{
    Capability, ClientProvider, DiscoveryRegistry, DiscoveryStatus, ServiceEndpoint,
};

// Error handling
pub use error::{
    DispatchOutcome, IpcErrorPhase, OrExit, Result, RhizoCryptError, extract_rpc_error,
};

// Validation harness (canonical home: `validation` module; also re-exported from `error`)
pub use validation::{StderrSink, StringSink, ValidationHarness, ValidationSink};

// Events
pub use event::EventType;

// Integration
pub use integration::{ClientFactory, IntegrationStatus, ServiceStatus};

// Merkle trees
pub use merkle::{MerkleProof, MerkleRoot};

// Metrics
pub use metrics::PrimalMetrics;

// Primal traits and types
pub use primal::{
    HealthReport, HealthStatus, PrimalError, PrimalHealth, PrimalLifecycle, PrimalState,
};

// Main RhizoCrypt implementation
pub use rhizocrypt::RhizoCrypt;

// Environment handling
pub use safe_env::{CapabilityEnv, SafeEnv};

// Sessions
pub use session::{Session, SessionBuilder, SessionConfig, SessionState, SessionType};

// Slices
pub use slice::{
    LoanTerms, ResolutionOutcome, ResolutionRoute, Slice, SliceBuilder, SliceConstraints,
    SliceMode, SliceOrigin, SliceState,
};

// Storage
pub use store::{
    DagBackend, DagStore, InMemoryDagStore, InMemoryPayloadStore, PayloadStore, StorageHealth,
    StorageStats,
};

#[cfg(feature = "redb")]
pub use store_redb::RedbDagStore;
#[cfg(feature = "sled")]
#[expect(deprecated, reason = "sled backend is deprecated; re-export retained until removal")]
pub use store_sled::SledDagStore;

// Types
pub use types::{ContentHash, Did, PayloadRef, SessionId, Signature, SliceId, Timestamp, VertexId};

// Vertices
pub use vertex::{MetadataValue, Vertex, VertexBuilder};

// ============================================================================
// Capability-Based Integration (RECOMMENDED)
// ============================================================================

/// Capability-based provider traits - vendor-agnostic, federation-ready.
///
/// These traits define capabilities (signing, storage, etc.) without hardcoding
/// specific primal names. Any service can implement these traits.
///
/// ## Philosophy
///
/// Request **capabilities**, not **vendors**:
/// - ✅ "I need crypto:signing capability"
/// - ❌ "I need BearDog"
///
/// ## Example
///
/// ```no_run
/// # use rhizo_crypt_core::clients::capabilities::SigningClient;
/// # use rhizo_crypt_core::types::Did;
/// # use std::sync::Arc;
/// # tokio::runtime::Runtime::new().unwrap().block_on(async {
/// # let registry = Arc::new(rhizo_crypt_core::discovery::DiscoveryRegistry::new("doc-test"));
/// # registry.register_endpoint(rhizo_crypt_core::discovery::ServiceEndpoint::new(
/// #     "test-signer", "127.0.0.1:9500".parse().unwrap(),
/// #     vec![rhizo_crypt_core::discovery::Capability::Signing],
/// # )).await;
/// // Discover ANY signing provider (BearDog, YubiKey, CloudKMS, etc.)
/// let signer = SigningClient::discover(&registry).await?;
/// let did = Did::new("did:key:test");
/// let signature = signer.sign(b"data", &did).await?;
/// # Ok::<(), rhizo_crypt_core::error::RhizoCryptError>(())
/// # });
/// ```
pub use integration::{PayloadStorageProvider, PermanentStorageProvider, SigningProvider};

/// Capability-based client implementations.
///
/// These clients use discovery to find providers at runtime.
pub use clients::capabilities::{
    ComputeClient, PermanentStorageClient, ProvenanceClient, SigningClient, StorageClient,
};

// ============================================================================
// Test Utilities (Capability-Based Mocks)
// ============================================================================

/// Mock providers for testing - capability-based, not vendor-specific.
#[cfg(any(test, feature = "test-utils"))]
pub use integration::{
    MockPayloadStorageProvider, MockPermanentStorageProvider, MockSigningProvider,
};

// ============================================================================
// Ecosystem Type Definitions
// ============================================================================

// Compute capability types
pub use types_ecosystem::compute::{
    ComputeEvent, ComputeProviderClient, ComputeProviderConfig, TaskId,
};

// Provenance capability types
pub use types_ecosystem::provenance::{
    AgentContribution, ProvenanceChain, ProvenanceNotifier, ProvenanceProviderConfig,
    ProvenanceQueryable, SessionAttribution, VertexQuery, VertexRef,
};
