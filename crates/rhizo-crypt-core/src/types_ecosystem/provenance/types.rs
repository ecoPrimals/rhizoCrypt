// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2024–2026 ecoPrimals Project

//! Provenance Provider Types - Lineage & Attribution
//!
//! Type definitions for provenance query capability providers.
//! These types work with ANY provenance provider (provenance provider, custom audit systems).

use std::borrow::Cow;
use std::collections::HashSet;

use serde::{Deserialize, Serialize};

use crate::constants::{DEFAULT_CAPABILITY_TIMEOUT_MS, PROVENANCE_DEFAULT_MAX_RESULTS};
use crate::types::{Did, PayloadRef, SessionId, Timestamp, VertexId};

/// Reference to a vertex for external queries.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct VertexRef {
    /// Session containing the vertex.
    pub session_id: SessionId,
    /// Vertex identifier.
    pub vertex_id: VertexId,
    /// Event type.
    pub event_type: String,
    /// Agent DID (if any).
    pub agent: Option<Did>,
    /// Creation timestamp.
    pub timestamp: Timestamp,
    /// Payload reference (if any).
    pub payload_ref: Option<PayloadRef>,
}

/// Provenance chain for tracking data lineage.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct ProvenanceChain {
    /// Vertices in the chain (ordered by causality).
    pub vertices: Vec<VertexRef>,
    /// Agent DIDs involved (typed for semantic correctness).
    pub agents: HashSet<Did>,
    /// Data hashes referenced.
    pub data_hashes: HashSet<[u8; 32]>,
}

impl ProvenanceChain {
    /// Create a new empty chain.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a vertex to the chain.
    pub fn add_vertex(&mut self, vertex: VertexRef) {
        if let Some(ref agent) = vertex.agent {
            self.agents.insert(agent.clone());
        }
        if let Some(ref payload) = vertex.payload_ref {
            self.data_hashes.insert(payload.hash);
        }
        self.vertices.push(vertex);
    }

    /// Get the number of vertices in the chain.
    #[must_use]
    pub const fn len(&self) -> usize {
        self.vertices.len()
    }

    /// Check if the chain is empty.
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.vertices.is_empty()
    }
}

/// Agent contribution to a session.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AgentContribution {
    /// Agent DID.
    pub agent: Did,
    /// Number of events created.
    pub event_count: u64,
    /// Types of events created.
    pub event_types: Vec<String>,
    /// First event timestamp.
    pub first_event: Timestamp,
    /// Last event timestamp.
    pub last_event: Timestamp,
}

/// Session attribution information for provenance provider.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SessionAttribution {
    /// Session identifier.
    pub session_id: SessionId,
    /// Session type.
    pub session_type: String,
    /// Agent contributions.
    pub agents: Vec<AgentContribution>,
    /// Input data hashes.
    pub data_inputs: Vec<[u8; 32]>,
    /// Output data hashes.
    pub data_outputs: Vec<[u8; 32]>,
    /// Merkle root of the session.
    pub merkle_root: [u8; 32],
}

/// Configuration for provenance provider queryable interface.
#[derive(Debug, Clone)]
pub struct ProvenanceProviderConfig {
    /// provenance provider service address (for push notifications).
    pub push_address: Option<Cow<'static, str>>,

    /// Query timeout in milliseconds.
    pub timeout_ms: u64,

    /// Maximum results per query.
    pub max_results: usize,

    /// Enable query caching.
    pub cache_enabled: bool,

    /// Cache TTL in seconds.
    pub cache_ttl_secs: u64,
}

impl Default for ProvenanceProviderConfig {
    fn default() -> Self {
        Self {
            push_address: None,
            timeout_ms: DEFAULT_CAPABILITY_TIMEOUT_MS,
            max_results: PROVENANCE_DEFAULT_MAX_RESULTS,
            cache_enabled: true,
            cache_ttl_secs: 300,
        }
    }
}

impl ProvenanceProviderConfig {
    /// Create config from environment variables.
    ///
    /// Environment variables:
    /// - `PROVENANCE_ENDPOINT` or `PROVENANCE_QUERY_ENDPOINT`: Provenance capability endpoint
    /// - `PROVENANCE_TIMEOUT_MS`: Query timeout in milliseconds
    #[must_use]
    pub fn from_env() -> Self {
        use crate::safe_env::CapabilityEnv;
        let mut config = Self::default();

        if let Some(addr) = CapabilityEnv::provenance_endpoint() {
            config.push_address = Some(Cow::Owned(addr));
        }

        if let Ok(timeout) = std::env::var("PROVENANCE_TIMEOUT_MS")
            && let Ok(ms) = timeout.parse()
        {
            config.timeout_ms = ms;
        }

        config
    }

    /// Create config with a specific push address.
    #[must_use]
    pub fn with_push_address(address: impl Into<Cow<'static, str>>) -> Self {
        Self {
            push_address: Some(address.into()),
            ..Self::default()
        }
    }
}

/// Query parameters for vertex searches.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct VertexQuery {
    /// Filter by agent.
    pub agent: Option<Did>,
    /// Filter by event types.
    pub event_types: Option<Vec<String>>,
    /// Filter by session.
    pub session_id: Option<SessionId>,
    /// Filter by time range start.
    pub after: Option<Timestamp>,
    /// Filter by time range end.
    pub before: Option<Timestamp>,
    /// Filter by payload hash.
    pub payload_hash: Option<[u8; 32]>,
    /// Maximum results.
    pub limit: Option<usize>,
}

impl VertexQuery {
    /// Create a new empty query.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Filter by agent.
    #[must_use]
    pub fn with_agent(mut self, agent: Did) -> Self {
        self.agent = Some(agent);
        self
    }

    /// Filter by event types.
    #[must_use]
    pub fn with_event_types(mut self, types: Vec<String>) -> Self {
        self.event_types = Some(types);
        self
    }

    /// Filter by session.
    #[must_use]
    pub const fn with_session(mut self, session_id: SessionId) -> Self {
        self.session_id = Some(session_id);
        self
    }

    /// Set maximum results.
    #[must_use]
    pub const fn with_limit(mut self, limit: usize) -> Self {
        self.limit = Some(limit);
        self
    }
}

/// provenance provider queryable interface.
///
/// This is the interface that provenance provider uses to query rhizoCrypt
/// for provenance and attribution information.
///
/// ## Usage
///
/// ```no_run
/// # use rhizo_crypt_core::types_ecosystem::provenance::{ProvenanceQueryable, VertexRef, ProvenanceChain};
/// # use rhizo_crypt_core::types::VertexId;
/// #
/// # async fn example(queryable: &impl ProvenanceQueryable) -> rhizo_crypt_core::error::Result<()> {
/// #     let data_hash = [0u8; 32];
/// #     let vertices = queryable.get_vertices_for_data(data_hash).await?;
/// #     let vertex_id = vertices.first().map(|v| v.vertex_id).unwrap_or(VertexId::from_bytes(&[0u8; 32]));
/// #     let _chain = queryable.get_provenance_chain(vertex_id).await?;
/// #     Ok(())
/// # }
/// // rhizoCrypt implements this trait; pass any ProvenanceQueryable impl
/// ```
pub trait ProvenanceQueryable: Send + Sync {
    /// Get all vertices related to a data hash.
    fn get_vertices_for_data(
        &self,
        data_hash: [u8; 32],
    ) -> impl std::future::Future<Output = crate::error::Result<Vec<VertexRef>>> + Send;

    /// Get provenance chain for a vertex.
    fn get_provenance_chain(
        &self,
        vertex_id: VertexId,
    ) -> impl std::future::Future<Output = crate::error::Result<ProvenanceChain>> + Send;

    /// Query vertices by parameters.
    fn query_vertices(
        &self,
        query: VertexQuery,
    ) -> impl std::future::Future<Output = crate::error::Result<Vec<VertexRef>>> + Send;

    /// Get session attribution for provenance provider.
    fn get_session_attribution(
        &self,
        session_id: SessionId,
    ) -> impl std::future::Future<Output = crate::error::Result<SessionAttribution>> + Send;
}

/// Client state.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ClientState {
    /// Not connected.
    #[default]
    Disconnected,
    /// Connected and ready.
    Connected,
}
