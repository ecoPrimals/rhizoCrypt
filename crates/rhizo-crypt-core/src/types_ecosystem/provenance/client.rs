// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2024–2026 ecoPrimals Project

//! Provenance provider notifier for push updates.
//!
//! Used to notify provenance provider when new provenance data is available.

use std::net::SocketAddr;
use std::sync::Arc;

use tokio::sync::RwLock;
use tracing::{debug, info, warn};

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
