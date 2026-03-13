// SPDX-License-Identifier: AGPL-3.0-only
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

use std::borrow::Cow;
use std::collections::HashSet;
use std::net::SocketAddr;
use std::sync::Arc;

use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

use crate::discovery::{Capability, DiscoveryRegistry};
use crate::error::{Result, RhizoCryptError};
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
    /// Agents involved.
    pub agents: HashSet<String>,
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
            self.agents.insert(agent.as_str().to_string());
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
            timeout_ms: 5000,
            max_results: 1000,
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

        if let Ok(timeout) = std::env::var("PROVENANCE_TIMEOUT_MS") {
            if let Ok(ms) = timeout.parse() {
                config.timeout_ms = ms;
            }
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
/// ```rust,ignore
/// use rhizo_crypt_core::clients::ProvenanceQueryable;
///
/// // rhizoCrypt implements this trait
/// let vertices = queryable.get_vertices_for_data(data_hash).await?;
/// let chain = queryable.get_provenance_chain(vertex_id).await?;
/// ```
pub trait ProvenanceQueryable: Send + Sync {
    /// Get all vertices related to a data hash.
    fn get_vertices_for_data(
        &self,
        data_hash: [u8; 32],
    ) -> impl std::future::Future<Output = Result<Vec<VertexRef>>> + Send;

    /// Get provenance chain for a vertex.
    fn get_provenance_chain(
        &self,
        vertex_id: VertexId,
    ) -> impl std::future::Future<Output = Result<ProvenanceChain>> + Send;

    /// Query vertices by parameters.
    fn query_vertices(
        &self,
        query: VertexQuery,
    ) -> impl std::future::Future<Output = Result<Vec<VertexRef>>> + Send;

    /// Get session attribution for provenance provider.
    fn get_session_attribution(
        &self,
        session_id: SessionId,
    ) -> impl std::future::Future<Output = Result<SessionAttribution>> + Send;
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

/// provenance provider notifier for push updates.
///
/// Used to notify provenance provider when new provenance data is available.
pub struct ProvenanceNotifier {
    /// Client configuration.
    pub config: ProvenanceProviderConfig,

    /// Discovery registry.
    registry: Option<Arc<DiscoveryRegistry>>,

    /// Current state.
    state: Arc<RwLock<ClientState>>,

    /// Connected endpoint.
    endpoint: Arc<RwLock<Option<SocketAddr>>>,
}

impl ProvenanceNotifier {
    /// Create a new notifier with the given configuration.
    #[must_use]
    pub fn new(config: ProvenanceProviderConfig) -> Self {
        Self {
            config,
            registry: None,
            state: Arc::new(RwLock::new(ClientState::Disconnected)),
            endpoint: Arc::new(RwLock::new(None)),
        }
    }

    /// Create a notifier with discovery support.
    #[must_use]
    pub fn with_discovery(registry: Arc<DiscoveryRegistry>) -> Self {
        Self {
            config: ProvenanceProviderConfig::from_env(),
            registry: Some(registry),
            state: Arc::new(RwLock::new(ClientState::Disconnected)),
            endpoint: Arc::new(RwLock::new(None)),
        }
    }

    /// Get the current state.
    pub async fn state(&self) -> ClientState {
        *self.state.read().await
    }

    /// Connect to provenance provider for push notifications.
    ///
    /// # Errors
    ///
    /// Returns error if connection fails.
    pub async fn connect(&self) -> Result<()> {
        // Try discovery first
        if let Some(registry) = &self.registry {
            if let Some(endpoint) = registry.get_endpoint(&Capability::ProvenanceQuery).await {
                info!(address = %endpoint.addr, "Discovered provenance provider via registry");
                *self.endpoint.write().await = Some(endpoint.addr);
                *self.state.write().await = ClientState::Connected;
                return Ok(());
            }
        }

        // Fall back to configured address
        if let Some(ref addr) = self.config.push_address {
            let socket_addr: SocketAddr = addr.parse().map_err(|e| {
                RhizoCryptError::integration(format!(
                    "Invalid provenance provider address '{addr}': {e}"
                ))
            })?;

            debug!(address = %socket_addr, "Connecting to provenance provider");
            *self.endpoint.write().await = Some(socket_addr);
            *self.state.write().await = ClientState::Connected;
            return Ok(());
        }

        // provenance provider is optional - we can operate without it
        warn!("No provenance provider address available. Push notifications disabled.");
        Ok(())
    }

    /// Notify provenance provider of a new session commit.
    ///
    /// # Errors
    ///
    /// Returns error if notification fails.
    pub async fn notify_session_commit(&self, session_id: SessionId) -> Result<()> {
        if *self.state.read().await != ClientState::Connected {
            // Silently ignore if not connected - provenance provider is optional
            return Ok(());
        }

        debug!(%session_id, "Notifying provenance provider of session commit");

        // Scaffolded mode: log but don't send
        // With live-clients feature, this would send the notification

        Ok(())
    }

    /// Notify provenance provider of a new provenance chain.
    ///
    /// # Errors
    ///
    /// Returns error if notification fails.
    pub async fn notify_provenance(&self, chain: &ProvenanceChain) -> Result<()> {
        if *self.state.read().await != ClientState::Connected {
            return Ok(());
        }

        debug!(vertices = chain.len(), "Notifying provenance provider of provenance update");

        Ok(())
    }

    /// Get the current endpoint.
    pub async fn endpoint(&self) -> Option<SocketAddr> {
        *self.endpoint.read().await
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;

    #[test]
    fn test_config_default() {
        let config = ProvenanceProviderConfig::default();
        assert!(config.push_address.is_none());
        assert_eq!(config.timeout_ms, 5000);
        assert!(config.cache_enabled);
    }

    #[test]
    fn test_config_with_push_address() {
        let config = ProvenanceProviderConfig::with_push_address("127.0.0.1:9900");
        assert_eq!(config.push_address.as_deref(), Some("127.0.0.1:9900"));
    }

    #[test]
    fn test_provenance_chain() {
        let mut chain = ProvenanceChain::new();
        assert!(chain.is_empty());

        chain.add_vertex(VertexRef {
            session_id: SessionId::now(),
            vertex_id: VertexId::from_bytes(b"test-vertex"),
            event_type: "test".to_string(),
            agent: Some(Did::new("did:key:test")),
            timestamp: Timestamp::now(),
            payload_ref: None,
        });

        assert_eq!(chain.len(), 1);
        assert!(chain.agents.contains("did:key:test"));
    }

    #[test]
    fn test_vertex_query_builder() {
        let query = VertexQuery::new().with_agent(Did::new("did:key:test")).with_limit(100);

        assert!(query.agent.is_some());
        assert_eq!(query.limit, Some(100));
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn test_notifier_creation() {
        let config = ProvenanceProviderConfig::default();
        let notifier = ProvenanceNotifier::new(config);
        assert_eq!(notifier.state().await, ClientState::Disconnected);
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn test_notify_without_connection() {
        let notifier = ProvenanceNotifier::new(ProvenanceProviderConfig::default());
        // Should succeed silently when not connected
        let result = notifier.notify_session_commit(SessionId::now()).await;
        assert!(result.is_ok());
    }

    // ============================================================================
    // Additional Tests for Coverage Boost (39% → 80%+)
    // ============================================================================

    #[test]
    fn test_vertex_ref_creation() {
        let session_id = SessionId::now();
        let vertex_id = VertexId::from_bytes(b"test-vertex-123");
        let did = Did::new("did:key:agent1");
        let timestamp = Timestamp::now();
        let payload = PayloadRef::from_bytes(b"payload-data");

        let vertex = VertexRef {
            session_id,
            vertex_id,
            event_type: "test.event".to_string(),
            agent: Some(did.clone()),
            timestamp,
            payload_ref: Some(payload),
        };

        assert_eq!(vertex.session_id, session_id);
        assert_eq!(vertex.vertex_id, vertex_id);
        assert_eq!(vertex.event_type, "test.event");
        assert_eq!(vertex.agent, Some(did));
        assert!(vertex.payload_ref.is_some());
    }

    #[test]
    fn test_vertex_ref_without_optional_fields() {
        let vertex = VertexRef {
            session_id: SessionId::now(),
            vertex_id: VertexId::from_bytes(b"vertex"),
            event_type: "event".to_string(),
            agent: None,
            timestamp: Timestamp::now(),
            payload_ref: None,
        };

        assert!(vertex.agent.is_none());
        assert!(vertex.payload_ref.is_none());
    }

    #[test]
    fn test_provenance_chain_new() {
        let chain = ProvenanceChain::new();
        assert!(chain.is_empty());
        assert_eq!(chain.len(), 0);
        assert!(chain.vertices.is_empty());
        assert!(chain.agents.is_empty());
        assert!(chain.data_hashes.is_empty());
    }

    #[test]
    fn test_provenance_chain_add_vertex_with_agent() {
        let mut chain = ProvenanceChain::new();
        let did = Did::new("did:key:agent1");

        chain.add_vertex(VertexRef {
            session_id: SessionId::now(),
            vertex_id: VertexId::from_bytes(b"v1"),
            event_type: "test".to_string(),
            agent: Some(did),
            timestamp: Timestamp::now(),
            payload_ref: None,
        });

        assert_eq!(chain.len(), 1);
        assert_eq!(chain.agents.len(), 1);
        assert!(chain.agents.contains("did:key:agent1"));
    }

    #[test]
    fn test_provenance_chain_add_vertex_with_payload() {
        let mut chain = ProvenanceChain::new();
        let payload = PayloadRef::from_bytes(b"test-data");

        chain.add_vertex(VertexRef {
            session_id: SessionId::now(),
            vertex_id: VertexId::from_bytes(b"v1"),
            event_type: "test".to_string(),
            agent: None,
            timestamp: Timestamp::now(),
            payload_ref: Some(payload),
        });

        assert_eq!(chain.len(), 1);
        assert_eq!(chain.data_hashes.len(), 1);
        assert!(chain.data_hashes.contains(&payload.hash));
    }

    #[test]
    fn test_provenance_chain_multiple_vertices() {
        let mut chain = ProvenanceChain::new();
        let did1 = Did::new("did:key:agent1");
        let did2 = Did::new("did:key:agent2");

        chain.add_vertex(VertexRef {
            session_id: SessionId::now(),
            vertex_id: VertexId::from_bytes(b"v1"),
            event_type: "event1".to_string(),
            agent: Some(did1),
            timestamp: Timestamp::now(),
            payload_ref: None,
        });

        chain.add_vertex(VertexRef {
            session_id: SessionId::now(),
            vertex_id: VertexId::from_bytes(b"v2"),
            event_type: "event2".to_string(),
            agent: Some(did2),
            timestamp: Timestamp::now(),
            payload_ref: None,
        });

        assert_eq!(chain.len(), 2);
        assert_eq!(chain.agents.len(), 2);
        assert!(chain.agents.contains("did:key:agent1"));
        assert!(chain.agents.contains("did:key:agent2"));
    }

    #[test]
    fn test_provenance_chain_default() {
        let chain = ProvenanceChain::default();
        assert!(chain.is_empty());
    }

    #[test]
    fn test_agent_contribution_creation() {
        let did = Did::new("did:key:worker");
        let contribution = AgentContribution {
            agent: did.clone(),
            event_count: 42,
            event_types: vec!["task.created".to_string(), "task.completed".to_string()],
            first_event: Timestamp::now(),
            last_event: Timestamp::now(),
        };

        assert_eq!(contribution.agent, did);
        assert_eq!(contribution.event_count, 42);
        assert_eq!(contribution.event_types.len(), 2);
    }

    #[test]
    fn test_session_attribution_creation() {
        let session_id = SessionId::now();
        let did = Did::new("did:key:agent");

        let attribution = SessionAttribution {
            session_id,
            session_type: "ml-training".to_string(),
            agents: vec![AgentContribution {
                agent: did,
                event_count: 10,
                event_types: vec!["compute".to_string()],
                first_event: Timestamp::now(),
                last_event: Timestamp::now(),
            }],
            data_inputs: vec![[1u8; 32], [2u8; 32]],
            data_outputs: vec![[3u8; 32]],
            merkle_root: [0u8; 32],
        };

        assert_eq!(attribution.session_id, session_id);
        assert_eq!(attribution.agents.len(), 1);
        assert_eq!(attribution.data_inputs.len(), 2);
        assert_eq!(attribution.data_outputs.len(), 1);
    }

    #[test]
    fn test_vertex_query_new() {
        let query = VertexQuery::new();
        assert!(query.agent.is_none());
        assert!(query.session_id.is_none());
        assert!(query.event_types.is_none());
        assert!(query.after.is_none());
        assert!(query.before.is_none());
        assert!(query.payload_hash.is_none());
        assert!(query.limit.is_none());
    }

    #[test]
    fn test_vertex_query_with_agent() {
        let did = Did::new("did:key:test");
        let query = VertexQuery::new().with_agent(did.clone());
        assert_eq!(query.agent, Some(did));
    }

    #[test]
    fn test_vertex_query_with_session() {
        let session_id = SessionId::now();
        let query = VertexQuery::new().with_session(session_id);
        assert_eq!(query.session_id, Some(session_id));
    }

    #[test]
    fn test_vertex_query_with_event_types() {
        let query = VertexQuery::new()
            .with_event_types(vec!["task.created".to_string(), "task.completed".to_string()]);
        assert_eq!(
            query.event_types,
            Some(vec!["task.created".to_string(), "task.completed".to_string()])
        );
    }

    #[test]
    fn test_vertex_query_with_limit() {
        let query = VertexQuery::new().with_limit(50);
        assert_eq!(query.limit, Some(50));
    }

    #[test]
    fn test_vertex_query_with_time_range() {
        let mut query = VertexQuery::new();
        let after_ts = Timestamp::now();
        let before_ts = Timestamp::now();
        query.after = Some(after_ts);
        query.before = Some(before_ts);
        assert_eq!(query.after, Some(after_ts));
        assert_eq!(query.before, Some(before_ts));
    }

    #[test]
    fn test_vertex_query_chaining() {
        let did = Did::new("did:key:test");
        let session_id = SessionId::now();

        let query = VertexQuery::new()
            .with_agent(did.clone())
            .with_session(session_id)
            .with_event_types(vec!["test.event".to_string()])
            .with_limit(100);

        assert_eq!(query.agent, Some(did));
        assert_eq!(query.session_id, Some(session_id));
        assert_eq!(query.event_types, Some(vec!["test.event".to_string()]));
        assert_eq!(query.limit, Some(100));
    }

    #[test]
    fn test_config_from_env() {
        let config = ProvenanceProviderConfig::from_env();
        assert!(config.timeout_ms > 0);
    }

    #[test]
    fn test_config_with_custom_values() {
        let mut config = ProvenanceProviderConfig::default();
        config.timeout_ms = 10000;
        config.cache_ttl_secs = 120;
        config.cache_enabled = false;

        assert_eq!(config.timeout_ms, 10000);
        assert_eq!(config.cache_ttl_secs, 120);
        assert!(!config.cache_enabled);
    }

    #[test]
    fn test_client_state_default() {
        let state = ClientState::default();
        assert_eq!(state, ClientState::Disconnected);
    }

    #[test]
    fn test_client_state_connected() {
        let state1 = ClientState::Disconnected;
        let state2 = ClientState::Connected;
        assert_ne!(state1, state2);
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn test_notifier_with_discovery() {
        let registry = Arc::new(DiscoveryRegistry::new("test"));
        let notifier = ProvenanceNotifier::with_discovery(registry);
        assert_eq!(notifier.state().await, ClientState::Disconnected);
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn test_notifier_connect_with_push_address() {
        let config = ProvenanceProviderConfig::with_push_address("127.0.0.1:9900");
        let notifier = ProvenanceNotifier::new(config);

        let result = notifier.connect().await;
        assert!(result.is_ok());
        assert_eq!(notifier.state().await, ClientState::Connected);
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn test_notifier_connect_invalid_address() {
        let config = ProvenanceProviderConfig::with_push_address("invalid-address");
        let notifier = ProvenanceNotifier::new(config);

        let result = notifier.connect().await;
        assert!(result.is_err());
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn test_notifier_connect_no_address() {
        let notifier = ProvenanceNotifier::new(ProvenanceProviderConfig::default());

        // Should succeed with warning (provenance provider is optional)
        let result = notifier.connect().await;
        assert!(result.is_ok());
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn test_notify_session_commit_connected() {
        let config = ProvenanceProviderConfig::with_push_address("127.0.0.1:9900");
        let notifier = ProvenanceNotifier::new(config);
        notifier.connect().await.unwrap();

        let result = notifier.notify_session_commit(SessionId::now()).await;
        assert!(result.is_ok());
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn test_notify_provenance_without_connection() {
        let notifier = ProvenanceNotifier::new(ProvenanceProviderConfig::default());
        let chain = ProvenanceChain::new();

        // Should succeed silently when not connected
        let result = notifier.notify_provenance(&chain).await;
        assert!(result.is_ok());
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn test_notify_provenance_connected() {
        let config = ProvenanceProviderConfig::with_push_address("127.0.0.1:9900");
        let notifier = ProvenanceNotifier::new(config);
        notifier.connect().await.unwrap();

        let mut chain = ProvenanceChain::new();
        chain.add_vertex(VertexRef {
            session_id: SessionId::now(),
            vertex_id: VertexId::from_bytes(b"v1"),
            event_type: "test".to_string(),
            agent: Some(Did::new("did:key:test")),
            timestamp: Timestamp::now(),
            payload_ref: None,
        });

        let result = notifier.notify_provenance(&chain).await;
        assert!(result.is_ok());
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn test_endpoint_management() {
        let config = ProvenanceProviderConfig::with_push_address("127.0.0.1:9900");
        let notifier = ProvenanceNotifier::new(config);

        // Initially no endpoint
        assert!(notifier.endpoint().await.is_none());

        // Connect
        notifier.connect().await.unwrap();

        // Should have endpoint
        let endpoint = notifier.endpoint().await;
        assert!(endpoint.is_some());
    }

    #[test]
    fn test_vertex_ref_serialization() {
        let vertex = VertexRef {
            session_id: SessionId::now(),
            vertex_id: VertexId::from_bytes(b"test"),
            event_type: "test".to_string(),
            agent: Some(Did::new("did:key:test")),
            timestamp: Timestamp::now(),
            payload_ref: None,
        };

        let serialized = serde_json::to_string(&vertex).unwrap();
        let deserialized: VertexRef = serde_json::from_str(&serialized).unwrap();
        assert_eq!(deserialized.event_type, "test");
    }

    #[test]
    fn test_provenance_chain_serialization() {
        let mut chain = ProvenanceChain::new();
        chain.add_vertex(VertexRef {
            session_id: SessionId::now(),
            vertex_id: VertexId::from_bytes(b"v1"),
            event_type: "test".to_string(),
            agent: Some(Did::new("did:key:test")),
            timestamp: Timestamp::now(),
            payload_ref: None,
        });

        let serialized = serde_json::to_string(&chain).unwrap();
        let deserialized: ProvenanceChain = serde_json::from_str(&serialized).unwrap();
        assert_eq!(deserialized.len(), 1);
    }

    #[test]
    fn test_agent_contribution_serialization() {
        let contribution = AgentContribution {
            agent: Did::new("did:key:test"),
            event_count: 5,
            event_types: vec!["test".to_string()],
            first_event: Timestamp::now(),
            last_event: Timestamp::now(),
        };

        let serialized = serde_json::to_string(&contribution).unwrap();
        let deserialized: AgentContribution = serde_json::from_str(&serialized).unwrap();
        assert_eq!(deserialized.event_count, 5);
    }

    #[test]
    fn test_vertex_query_serialization() {
        let query = VertexQuery::new().with_agent(Did::new("did:key:test")).with_limit(100);

        let serialized = serde_json::to_string(&query).unwrap();
        let deserialized: VertexQuery = serde_json::from_str(&serialized).unwrap();
        assert_eq!(deserialized.limit, Some(100));
    }
}
