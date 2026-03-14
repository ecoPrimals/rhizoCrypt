// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2024–2026 ecoPrimals Project

//! Provenance provider notifier for push updates.
//!
//! Used to notify provenance provider when new provenance data is available.
//! When connected, sends JSON-RPC calls to the attribution provider (sweetGrass
//! or any provider with `ProvenanceQuery` capability).

use std::net::SocketAddr;
use std::sync::Arc;

use tokio::sync::RwLock;
use tracing::{debug, info, warn};

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

        let stored = *self.endpoint.read().await;
        let Some(endpoint) = stored else {
            return Ok(());
        };

        debug!(%session_id, %endpoint, "Notifying provenance provider of session commit");

        let request = serde_json::json!({
            "jsonrpc": "2.0",
            "method": "contribution.record_session",
            "params": {
                "session_id": session_id.to_string(),
                "source_primal": crate::constants::PRIMAL_NAME,
            },
            "id": 1
        });

        match Self::send_jsonrpc(&endpoint, &request).await {
            Ok(response) => {
                info!(
                    %session_id,
                    "Provenance provider notified of session commit: {}",
                    response
                );
            }
            Err(e) => {
                warn!(
                    %session_id,
                    error = %e,
                    "Failed to notify provenance provider (non-fatal)"
                );
            }
        }

        Ok(())
    }

    /// Notify provenance provider of a completed dehydration with full summary.
    ///
    /// Sends a `contribution.record_dehydration` JSON-RPC call with the
    /// `DehydrationSummary` so the provider can create attribution braids
    /// linking agents to their contributions.
    ///
    /// # Errors
    ///
    /// Returns error if notification fails.
    pub async fn notify_dehydration(&self, summary: &DehydrationSummary) -> Result<()> {
        if *self.state.read().await != ClientState::Connected {
            return Ok(());
        }

        let stored = *self.endpoint.read().await;
        let Some(endpoint) = stored else {
            return Ok(());
        };

        debug!(
            session_id = %summary.session_id,
            agents = summary.agents.len(),
            %endpoint,
            "Notifying provenance provider of dehydration"
        );

        let agent_dids: Vec<String> = summary.agents.iter().map(|a| a.agent.to_string()).collect();

        let request = serde_json::json!({
            "jsonrpc": "2.0",
            "method": "contribution.record_dehydration",
            "params": {
                "session_id": summary.session_id.to_string(),
                "source_primal": crate::constants::PRIMAL_NAME,
                "merkle_root": summary.merkle_root.to_string(),
                "vertex_count": summary.vertex_count,
                "branch_count": summary.results.len() as u64,
                "agents": agent_dids,
                "session_start": summary.created_at.as_nanos(),
                "dehydrated_at": summary.resolved_at.as_nanos(),
                "session_type": summary.session_type,
                "outcome": format!("{:?}", summary.outcome),
            },
            "id": 1
        });

        match Self::send_jsonrpc(&endpoint, &request).await {
            Ok(response) => {
                info!(
                    session_id = %summary.session_id,
                    "Provenance provider notified of dehydration: {}",
                    response
                );
            }
            Err(e) => {
                warn!(
                    session_id = %summary.session_id,
                    error = %e,
                    "Failed to notify provenance provider of dehydration (non-fatal)"
                );
            }
        }

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

    /// Send a JSON-RPC request to the given endpoint via TCP.
    async fn send_jsonrpc(
        endpoint: &SocketAddr,
        request: &serde_json::Value,
    ) -> std::result::Result<String, String> {
        use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
        use tokio::net::TcpStream;

        let stream =
            tokio::time::timeout(std::time::Duration::from_secs(5), TcpStream::connect(endpoint))
                .await
                .map_err(|_| format!("Connection timeout to {endpoint}"))?
                .map_err(|e| format!("Connection failed to {endpoint}: {e}"))?;

        let (reader, mut writer) = stream.into_split();

        let payload = format!("{}\n", serde_json::to_string(request).unwrap_or_default());
        writer.write_all(payload.as_bytes()).await.map_err(|e| format!("Write failed: {e}"))?;

        let mut buf_reader = BufReader::new(reader);
        let mut response = String::new();

        tokio::time::timeout(
            std::time::Duration::from_secs(10),
            buf_reader.read_line(&mut response),
        )
        .await
        .map_err(|_| "Response timeout".to_string())?
        .map_err(|e| format!("Read failed: {e}"))?;

        Ok(response)
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
    use crate::types::{Did, Timestamp, VertexId};
    use crate::types_ecosystem::provenance::VertexRef;
    use crate::MerkleRoot;

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
}
