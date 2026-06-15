// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2024–2026 ecoPrimals Project

//! Provenance provider notifier for push updates.
//!
//! Used to notify provenance provider when new provenance data is available.
//! When connected, sends JSON-RPC calls to any attribution provider that
//! implements the `ProvenanceQuery` capability.

use std::sync::Arc;

use crate::transport::TransportEndpoint;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

use crate::constants::{PROVENANCE_CONNECTION_TIMEOUT, PROVENANCE_RESPONSE_TIMEOUT};
use crate::dehydration::DehydrationSummary;
use crate::discovery::{Capability, DiscoveryRegistry};
use crate::error::{Result, RhizoCryptError};
use crate::types::SessionId;

use super::types::{ClientState, ProvenanceChain, ProvenanceProviderConfig};

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

    /// Connected endpoint (transport-agnostic).
    endpoint: Arc<RwLock<Option<TransportEndpoint>>>,
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
        if let Some(registry) = &self.registry
            && let Some(service) = registry.get_endpoint(&Capability::ProvenanceQuery).await
        {
            info!(endpoint = %service.endpoint, "Discovered provenance provider via registry");
            *self.endpoint.write().await = Some(service.endpoint.clone());
            *self.state.write().await = ClientState::Connected;
            return Ok(());
        }

        if let Some(ref addr) = self.config.push_address {
            let transport = serde_json::from_str::<TransportEndpoint>(addr)
                .or_else(|_| {
                    addr.parse::<std::net::SocketAddr>()
                        .map(|sa| TransportEndpoint::tcp(sa.ip().to_string(), sa.port()))
                        .map_err(|e| {
                            RhizoCryptError::integration(format!(
                                "Invalid provenance provider address '{addr}': {e}"
                            ))
                        })
                })?;

            debug!(endpoint = %transport, "Connecting to provenance provider");
            *self.endpoint.write().await = Some(transport);
            *self.state.write().await = ClientState::Connected;
            return Ok(());
        }

        // provenance provider is optional - we can operate without it
        warn!("No provenance provider address available. Push notifications disabled.");
        Ok(())
    }

    /// Notify provenance provider of a new session commit.
    ///
    /// Sends a `contribution.record_session` JSON-RPC call to the connected
    /// provenance provider with the session ID so it can begin attribution.
    ///
    /// # Errors
    ///
    /// Returns error if notification fails.
    pub async fn notify_session_commit(&self, session_id: SessionId) -> Result<()> {
        if *self.state.read().await != ClientState::Connected {
            return Ok(());
        }

        let Some(endpoint) = self.endpoint.read().await.clone() else {
            return Ok(());
        };

        debug!(%session_id, %endpoint, "Notifying provenance provider of session commit");

        let request = serde_json::json!({
            "jsonrpc": crate::constants::JSONRPC_VERSION,
            "method": crate::constants::PROVENANCE_RECORD_SESSION_METHOD,
            "params": {
                "session_id": session_id.to_string(),
                "source_primal": crate::constants::PRIMAL_NAME,
            },
            "id": 1
        });

        Self::log_notify_result(
            "session commit",
            &format!("{session_id}"),
            Self::send_jsonrpc(&endpoint, &request).await,
        );

        Ok(())
    }

    /// Notify provenance provider of a completed dehydration with full summary.
    ///
    /// Converts the internal [`DehydrationSummary`] to a
    /// [`DehydrationWireSummary`](crate::dehydration_wire::DehydrationWireSummary)
    /// for the `contribution.record_dehydration` JSON-RPC call.
    ///
    /// # Errors
    ///
    /// Returns error if notification fails.
    pub async fn notify_dehydration(&self, summary: &DehydrationSummary) -> Result<()> {
        if *self.state.read().await != ClientState::Connected {
            return Ok(());
        }

        let Some(endpoint) = self.endpoint.read().await.clone() else {
            return Ok(());
        };

        debug!(
            session_id = %summary.session_id,
            agents = summary.agents.len(),
            %endpoint,
            "Notifying provenance provider of dehydration"
        );

        let wire_summary: crate::dehydration_wire::DehydrationWireSummary = summary.into();

        let request = serde_json::json!({
            "jsonrpc": crate::constants::JSONRPC_VERSION,
            "method": crate::constants::PROVENANCE_RECORD_DEHYDRATION_METHOD,
            "params": wire_summary,
            "id": 1
        });

        Self::log_notify_result(
            "dehydration",
            &format!("{}", summary.session_id),
            Self::send_jsonrpc(&endpoint, &request).await,
        );

        Ok(())
    }

    /// Notify provenance provider of a new provenance chain.
    ///
    /// Sends a `contribution.record_provenance` JSON-RPC call with the chain's
    /// vertex references. Non-fatal on failure (graceful degradation per
    /// the Provenance Trio graceful degradation pattern).
    ///
    /// # Errors
    ///
    /// Returns error if notification fails.
    pub async fn notify_provenance(&self, chain: &ProvenanceChain) -> Result<()> {
        if *self.state.read().await != ClientState::Connected {
            return Ok(());
        }

        let Some(endpoint) = self.endpoint.read().await.clone() else {
            return Ok(());
        };

        debug!(
            vertices = chain.len(),
            %endpoint,
            "Notifying provenance provider of provenance update"
        );

        let request = serde_json::json!({
            "jsonrpc": crate::constants::JSONRPC_VERSION,
            "method": crate::constants::PROVENANCE_RECORD_PROVENANCE_METHOD,
            "params": {
                "source_primal": crate::constants::PRIMAL_NAME,
                "vertices": chain.vertices,
                "agent_count": chain.agents.len(),
            },
            "id": 1
        });

        Self::log_notify_result(
            "provenance chain",
            &format!("{} vertices", chain.len()),
            Self::send_jsonrpc(&endpoint, &request).await,
        );

        Ok(())
    }

    fn log_notify_result(
        kind: &str,
        context: &str,
        result: std::result::Result<String, crate::transport::JsonRpcTransportError>,
    ) {
        match result {
            Ok(response) => {
                info!(context, "Provenance provider notified of {kind}: {response}");
            }
            Err(e) => {
                warn!(context, error = %e, "Failed to notify provenance provider of {kind} (non-fatal)");
            }
        }
    }

    async fn send_jsonrpc(
        endpoint: &TransportEndpoint,
        request: &serde_json::Value,
    ) -> std::result::Result<String, crate::transport::JsonRpcTransportError> {
        crate::transport::send_jsonrpc_request(
            endpoint,
            request,
            PROVENANCE_CONNECTION_TIMEOUT,
            PROVENANCE_RESPONSE_TIMEOUT,
        )
        .await
    }

    /// Get the current endpoint.
    pub async fn endpoint(&self) -> Option<TransportEndpoint> {
        self.endpoint.read().await.clone()
    }
}

#[cfg(test)]
#[expect(clippy::unwrap_used, reason = "test code")]
mod tests {
    use std::net::SocketAddr;

    use super::*;
    use crate::MerkleRoot;
    use crate::types::{Did, Timestamp, VertexId};
    use crate::types_ecosystem::provenance::VertexRef;

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
    async fn test_notify_dehydration_without_connection() {
        use crate::dehydration::{AgentSummary, DehydrationSummaryBuilder};
        use crate::event::SessionOutcome;

        let notifier = ProvenanceNotifier::new(ProvenanceProviderConfig::default());

        let summary = DehydrationSummaryBuilder::new(
            SessionId::now(),
            "test",
            Timestamp::now(),
            MerkleRoot::new([0u8; 32]),
        )
        .with_outcome(SessionOutcome::Success)
        .with_vertex_count(5)
        .with_agent(AgentSummary {
            agent: Did::new("did:key:test"),
            joined_at: Timestamp::now(),
            left_at: None,
            event_count: 3,
            role: "author".to_string(),
        })
        .build();

        let result = notifier.notify_dehydration(&summary).await;
        assert!(result.is_ok());
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn test_notify_dehydration_connected_no_server() {
        use crate::dehydration::{AgentSummary, DehydrationSummaryBuilder};
        use crate::event::SessionOutcome;

        let config = ProvenanceProviderConfig::with_push_address("127.0.0.1:19901");
        let notifier = ProvenanceNotifier::new(config);
        notifier.connect().await.unwrap();

        let summary = DehydrationSummaryBuilder::new(
            SessionId::now(),
            "test",
            Timestamp::now(),
            MerkleRoot::new([0u8; 32]),
        )
        .with_outcome(SessionOutcome::Success)
        .with_vertex_count(5)
        .with_agent(AgentSummary {
            agent: Did::new("did:key:test"),
            joined_at: Timestamp::now(),
            left_at: None,
            event_count: 3,
            role: "author".to_string(),
        })
        .build();

        // Non-fatal: should succeed even when provider is unreachable
        let result = notifier.notify_dehydration(&summary).await;
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
    async fn test_notify_provenance_connected_no_server() {
        let config = ProvenanceProviderConfig::with_push_address("127.0.0.1:19904");
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

        // Non-fatal: should succeed even when provider is unreachable
        let result = notifier.notify_provenance(&chain).await;
        assert!(result.is_ok());
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn test_send_jsonrpc_provenance_with_mock_server() {
        use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};

        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();

        let server_handle = tokio::spawn(async move {
            let (stream, _) = listener.accept().await.unwrap();
            let (reader, mut writer) = stream.into_split();
            let mut buf_reader = BufReader::new(reader);
            let mut request = String::new();
            buf_reader.read_line(&mut request).await.unwrap();

            let parsed: serde_json::Value = serde_json::from_str(&request).unwrap();
            assert_eq!(parsed["method"], "contribution.record_provenance");
            assert!(parsed["params"]["vertices"].is_array());

            let response = r#"{"jsonrpc":"2.0","result":"ok","id":1}"#;
            writer.write_all(format!("{response}\n").as_bytes()).await.unwrap();
        });

        let config = ProvenanceProviderConfig::with_push_address(addr.to_string());
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

        server_handle.await.unwrap();
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

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn test_connect_via_discovery_registry() {
        use crate::discovery::ServiceEndpoint;

        let registry = Arc::new(DiscoveryRegistry::new("test"));
        let addr: SocketAddr = "127.0.0.1:19902".parse().unwrap();
        registry
            .register_endpoint(ServiceEndpoint::new(
                "provenance-test",
                addr.into(),
                vec![Capability::ProvenanceQuery],
            ))
            .await;

        let notifier = ProvenanceNotifier::with_discovery(registry);
        let result = notifier.connect().await;
        assert!(result.is_ok());
        assert_eq!(notifier.state().await, ClientState::Connected);
        let expected = crate::transport::TransportEndpoint::tcp(
            addr.ip().to_string(),
            addr.port(),
        );
        assert_eq!(notifier.endpoint().await, Some(expected));
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn test_send_jsonrpc_success_with_mock_server() {
        use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};

        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();

        let server_handle = tokio::spawn(async move {
            let (stream, _) = listener.accept().await.unwrap();
            let (reader, mut writer) = stream.into_split();
            let mut buf_reader = BufReader::new(reader);
            let mut request = String::new();
            buf_reader.read_line(&mut request).await.unwrap();
            let response = r#"{"jsonrpc":"2.0","result":"ok","id":1}"#;
            writer.write_all(format!("{response}\n").as_bytes()).await.unwrap();
        });

        let config = ProvenanceProviderConfig::with_push_address(addr.to_string());
        let notifier = ProvenanceNotifier::new(config);
        notifier.connect().await.unwrap();

        let result = notifier.notify_session_commit(SessionId::now()).await;
        assert!(result.is_ok());

        server_handle.await.unwrap();
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn test_send_jsonrpc_dehydration_with_mock_server() {
        use crate::dehydration::{AgentSummary, DehydrationSummaryBuilder};
        use crate::event::SessionOutcome;
        use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};

        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();

        let server_handle = tokio::spawn(async move {
            let (stream, _) = listener.accept().await.unwrap();
            let (reader, mut writer) = stream.into_split();
            let mut buf_reader = BufReader::new(reader);
            let mut request = String::new();
            buf_reader.read_line(&mut request).await.unwrap();
            let response = r#"{"jsonrpc":"2.0","result":"ok","id":1}"#;
            writer.write_all(format!("{response}\n").as_bytes()).await.unwrap();
        });

        let config = ProvenanceProviderConfig::with_push_address(addr.to_string());
        let notifier = ProvenanceNotifier::new(config);
        notifier.connect().await.unwrap();

        let summary = DehydrationSummaryBuilder::new(
            SessionId::now(),
            "test",
            Timestamp::now(),
            MerkleRoot::new([0u8; 32]),
        )
        .with_outcome(SessionOutcome::Success)
        .with_vertex_count(5)
        .with_agent(AgentSummary {
            agent: Did::new("did:key:test"),
            joined_at: Timestamp::now(),
            left_at: None,
            event_count: 3,
            role: "author".to_string(),
        })
        .build();

        let result = notifier.notify_dehydration(&summary).await;
        assert!(result.is_ok());

        server_handle.await.unwrap();
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn test_send_jsonrpc_connection_refused() {
        let config = ProvenanceProviderConfig::with_push_address("127.0.0.1:19903");
        let notifier = ProvenanceNotifier::new(config);
        notifier.connect().await.unwrap();

        // Non-fatal: should succeed even when connection is refused
        let result = notifier.notify_session_commit(SessionId::now()).await;
        assert!(result.is_ok());
    }
}
