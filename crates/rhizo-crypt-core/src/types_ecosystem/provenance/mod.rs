// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2024–2026 ecoPrimals Project

//! Provenance Provider Types - Lineage & Attribution
//!
//! Type definitions for provenance query capability providers.
//! These types work with ANY provenance provider (provenance provider, custom audit systems).
//!
//! ## Capability-Based Architecture
//!
//! Provenance providers query rhizoCrypt (we are the provider, not the client).
//! This module defines the queryable interface that provenance systems can call.
//!
//! ```text
//! Provenance Provider      Bootstrap                rhizoCrypt
//!     │                        │                         │
//!     │──discover(dag-engine)─▶│                         │
//!     │◀──ServiceEndpoint──────│                         │
//!     │                        │                         │
//!     │────────query_provenance()───────────────────────▶│
//!     │◀───────ProvenanceChain──────────────────────────│
//! ```

mod client;
mod types;

#[cfg(test)]
mod tests;

// Re-export all public items to maintain the same API surface
pub use client::ProvenanceNotifier;
pub use types::{
    AgentContribution, ClientState, ProvenanceChain, ProvenanceProviderConfig, ProvenanceQueryable,
    SessionAttribution, VertexQuery, VertexRef,
};
