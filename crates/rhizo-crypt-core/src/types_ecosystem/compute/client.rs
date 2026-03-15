// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2024–2026 ecoPrimals Project

//! Compute Provider Client - Task Event Integration
//!
//! Client for subscribing to compute task events from capability providers.

use std::net::SocketAddr;
use std::sync::Arc;

use tokio::sync::RwLock;
use tracing::{debug, info, warn};

use crate::discovery::{Capability, DiscoveryRegistry};
use crate::error::{Result, RhizoCryptError};
use crate::types::Timestamp;

use super::types::{ClientState, ComputeEvent, ComputeProviderConfig, TaskId};

/// Compute provider client for task event integration.
///
/// Provides subscription to compute task events for integration
/// into rhizoCrypt sessions.
///
/// ## Usage
///
/// ```no_run
/// # use rhizo_crypt_core::types_ecosystem::compute::{ComputeProviderClient, TaskId};
/// # use std::sync::Arc;
/// # tokio::runtime::Runtime::new().unwrap().block_on(async {
/// # let registry = Arc::new(rhizo_crypt_core::discovery::DiscoveryRegistry::new("doc-test"));
/// // Create with discovery (preferred)
/// let client = ComputeProviderClient::with_discovery(registry);
/// client.connect().await?;
///
/// // Subscribe to a task
/// # let task_id = TaskId::now();
/// let mut events = client.subscribe_task(task_id).await?;
/// while let Some(event) = events.recv().await {
///     let _ = event;
/// }
/// # Ok::<(), rhizo_crypt_core::error::RhizoCryptError>(())
/// # });
/// ```
pub struct ComputeProviderClient {
    /// Client configuration.
    pub config: ComputeProviderConfig,

    /// Discovery registry for finding compute provider.
    registry: Option<Arc<DiscoveryRegistry>>,

    /// Current connection state.
    state: Arc<RwLock<ClientState>>,

    /// Connected endpoint (if any).
    endpoint: Arc<RwLock<Option<SocketAddr>>>,
}

impl ComputeProviderClient {
    /// Create a new client with the given configuration.
    #[must_use]
    pub fn new(config: ComputeProviderConfig) -> Self {
        Self {
            config,
            registry: None,
            state: Arc::new(RwLock::new(ClientState::Disconnected)),
            endpoint: Arc::new(RwLock::new(None)),
        }
    }

    /// Create a client with discovery support.
    #[must_use]
    pub fn with_discovery(registry: Arc<DiscoveryRegistry>) -> Self {
        Self {
            config: ComputeProviderConfig::from_env(),
            registry: Some(registry),
            state: Arc::new(RwLock::new(ClientState::Disconnected)),
            endpoint: Arc::new(RwLock::new(None)),
        }
    }

    /// Get the current connection state.
    pub async fn state(&self) -> ClientState {
        *self.state.read().await
    }

    /// Check if connected.
    pub async fn is_connected(&self) -> bool {
        *self.state.read().await == ClientState::Connected
    }

    /// Connect to compute provider.
    ///
    /// Uses discovery if available, otherwise falls back to configured address.
    ///
    /// # Errors
    ///
    /// Returns error if connection fails or no address is available.
    pub async fn connect(&self) -> Result<()> {
        // Check current state
        let current_state = *self.state.read().await;
        if current_state == ClientState::Connected {
            return Ok(());
        }

        *self.state.write().await = ClientState::Connecting;

        // Try discovery first
        if let Some(registry) = &self.registry
            && let Some(endpoint) = registry.get_endpoint(&Capability::ComputeOrchestration).await
        {
            info!(address = %endpoint.addr, "Discovered compute provider via registry");
            return self.connect_to(endpoint.addr).await;
        }

        // Fall back to configured address
        if let Some(ref addr) = self.config.fallback_address {
            let socket_addr: SocketAddr = addr.parse().map_err(|e| {
                RhizoCryptError::integration(format!(
                    "Invalid compute provider address '{addr}': {e}"
                ))
            })?;
            return self.connect_to(socket_addr).await;
        }

        *self.state.write().await = ClientState::Error;
        Err(RhizoCryptError::integration(
            "No compute provider address available. Set COMPUTE_ENDPOINT or use discovery.",
        ))
    }

    /// Connect to a specific address.
    async fn connect_to(&self, addr: SocketAddr) -> Result<()> {
        debug!(address = %addr, "Connecting to compute provider");

        // Scaffolded mode: verify we can reach the address
        match tokio::time::timeout(
            std::time::Duration::from_millis(self.config.timeout_ms),
            tokio::net::TcpStream::connect(addr),
        )
        .await
        {
            Ok(Ok(_stream)) => {
                info!(address = %addr, "Connected to compute provider (scaffolded mode)");
                *self.endpoint.write().await = Some(addr);
                *self.state.write().await = ClientState::Connected;
                Ok(())
            }
            Ok(Err(e)) => {
                warn!(error = %e, "Failed to connect to compute provider");
                *self.state.write().await = ClientState::Error;
                Err(RhizoCryptError::integration(format!(
                    "Failed to connect to compute provider: {e}"
                )))
            }
            Err(_) => {
                warn!("Compute provider connection timeout");
                *self.state.write().await = ClientState::Error;
                Err(RhizoCryptError::integration("Compute provider connection timeout"))
            }
        }
    }

    /// Subscribe to events for a specific task.
    ///
    /// Returns a receiver that will receive events as they occur.
    ///
    /// # Errors
    ///
    /// Returns error if not connected.
    pub async fn subscribe_task(
        &self,
        task_id: TaskId,
    ) -> Result<tokio::sync::mpsc::Receiver<ComputeEvent>> {
        if *self.state.read().await != ClientState::Connected {
            return Err(RhizoCryptError::integration(
                "Not connected to compute provider. Call connect() first.",
            ));
        }

        debug!(%task_id, "Subscribing to task events");

        // Create channel for events
        let (tx, rx) = tokio::sync::mpsc::channel(self.config.event_buffer_size);

        // Scaffolded mode: return empty channel
        // With live-clients feature, this would connect to the actual event stream
        drop(tx); // Close sender immediately in scaffolded mode

        Ok(rx)
    }

    /// Subscribe to events for all tasks by an agent.
    ///
    /// # Errors
    ///
    /// Returns error if not connected.
    pub async fn subscribe_agent(
        &self,
        agent: &crate::types::Did,
    ) -> Result<tokio::sync::mpsc::Receiver<ComputeEvent>> {
        if *self.state.read().await != ClientState::Connected {
            return Err(RhizoCryptError::integration(
                "Not connected to compute provider. Call connect() first.",
            ));
        }

        debug!(agent = %agent.as_str(), "Subscribing to agent events");

        let (tx, rx) = tokio::sync::mpsc::channel(self.config.event_buffer_size);
        drop(tx);

        Ok(rx)
    }

    /// Get events for a task in a time range.
    ///
    /// # Errors
    ///
    /// Returns error if not connected or query fails.
    pub async fn get_events(
        &self,
        task_id: TaskId,
        start: Timestamp,
        end: Timestamp,
    ) -> Result<Vec<ComputeEvent>> {
        if *self.state.read().await != ClientState::Connected {
            return Err(RhizoCryptError::integration(
                "Not connected to compute provider. Call connect() first.",
            ));
        }

        debug!(%task_id, ?start, ?end, "Querying task events");

        // Scaffolded mode: return empty
        Ok(Vec::new())
    }

    /// Get the current endpoint.
    pub async fn endpoint(&self) -> Option<SocketAddr> {
        *self.endpoint.read().await
    }

    /// Get the discovery registry (if configured).
    #[must_use]
    #[expect(clippy::missing_const_for_fn)] // as_ref() is not const-stable
    pub fn registry(&self) -> Option<&Arc<DiscoveryRegistry>> {
        self.registry.as_ref()
    }

    /// Get the fallback address from config.
    #[must_use]
    pub fn fallback_address(&self) -> Option<&str> {
        self.config.fallback_address.as_deref()
    }

    /// Set the connected state and endpoint.
    pub async fn set_connected(&self, endpoint: SocketAddr) {
        *self.endpoint.write().await = Some(endpoint);
        *self.state.write().await = ClientState::Connected;
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use std::sync::Arc;

    use super::*;

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn test_client_creation() {
        let config = ComputeProviderConfig::default();
        let client = ComputeProviderClient::new(config);
        assert_eq!(client.state().await, ClientState::Disconnected);
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn test_subscribe_without_connection() {
        let client = ComputeProviderClient::new(ComputeProviderConfig::default());
        let result = client.subscribe_task(TaskId::now()).await;
        assert!(result.is_err());
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn test_client_with_discovery() {
        let registry = Arc::new(DiscoveryRegistry::new("test"));
        let client = ComputeProviderClient::with_discovery(registry.clone());

        assert!(client.registry().is_some());
        assert_eq!(client.state().await, ClientState::Disconnected);
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn test_client_is_connected() {
        let client = ComputeProviderClient::new(ComputeProviderConfig::default());
        assert!(!client.is_connected().await);

        // Manually set connected
        *client.state.write().await = ClientState::Connected;
        assert!(client.is_connected().await);
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn test_connect_already_connected() {
        let config = ComputeProviderConfig::with_fallback("127.0.0.1:9800");
        let client = ComputeProviderClient::new(config);

        // Manually set connected
        *client.state.write().await = ClientState::Connected;

        // Should succeed (idempotent)
        let result = client.connect().await;
        assert!(result.is_ok());
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn test_connect_no_address_available() {
        let client = ComputeProviderClient::new(ComputeProviderConfig::default());

        let result = client.connect().await;
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("No compute provider address available"));
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn test_connect_invalid_address() {
        let config = ComputeProviderConfig::with_fallback("invalid-address");
        let client = ComputeProviderClient::new(config);

        let result = client.connect().await;
        assert!(result.is_err());
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn test_subscribe_agent_without_connection() {
        let client = ComputeProviderClient::new(ComputeProviderConfig::default());
        let did = crate::types::Did::new("did:key:test");
        let result = client.subscribe_agent(&did).await;

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("Not connected"));
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn test_subscribe_agent_connected() {
        let client = ComputeProviderClient::new(ComputeProviderConfig::default());
        *client.state.write().await = ClientState::Connected;

        let did = crate::types::Did::new("did:key:test");
        let result = client.subscribe_agent(&did).await;

        assert!(result.is_ok());
        let mut rx = result.unwrap();
        // Channel should be closed (scaffolded mode)
        assert!(rx.recv().await.is_none());
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn test_subscribe_task_connected() {
        let client = ComputeProviderClient::new(ComputeProviderConfig::default());
        *client.state.write().await = ClientState::Connected;

        let result = client.subscribe_task(TaskId::now()).await;

        assert!(result.is_ok());
        let mut rx = result.unwrap();
        // Channel should be closed (scaffolded mode)
        assert!(rx.recv().await.is_none());
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn test_get_events_without_connection() {
        let client = ComputeProviderClient::new(ComputeProviderConfig::default());
        let task_id = TaskId::now();
        let start = Timestamp::now();
        let end = Timestamp::now();

        let result = client.get_events(task_id, start, end).await;
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("Not connected"));
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn test_get_events_connected() {
        let client = ComputeProviderClient::new(ComputeProviderConfig::default());
        *client.state.write().await = ClientState::Connected;

        let task_id = TaskId::now();
        let start = Timestamp::now();
        let end = Timestamp::now();

        let result = client.get_events(task_id, start, end).await;
        assert!(result.is_ok());
        let events = result.unwrap();
        assert!(events.is_empty()); // Scaffolded mode returns empty
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn test_endpoint_management() {
        let client = ComputeProviderClient::new(ComputeProviderConfig::default());

        // Initially no endpoint
        assert!(client.endpoint().await.is_none());

        // Set endpoint
        let addr: SocketAddr = "127.0.0.1:9800".parse().unwrap();
        client.set_connected(addr).await;

        // Verify endpoint and state
        assert_eq!(client.endpoint().await, Some(addr));
        assert_eq!(client.state().await, ClientState::Connected);
        assert!(client.is_connected().await);
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn test_fallback_address() {
        let config = ComputeProviderConfig::with_fallback("10.0.0.1:9800");
        let client = ComputeProviderClient::new(config);

        assert_eq!(client.fallback_address(), Some("10.0.0.1:9800"));
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn test_registry_access() {
        let registry = Arc::new(DiscoveryRegistry::new("test"));
        let client = ComputeProviderClient::with_discovery(registry);

        let retrieved = client.registry();
        assert!(retrieved.is_some());
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn test_state_error_transition() {
        let client = ComputeProviderClient::new(ComputeProviderConfig::default());

        *client.state.write().await = ClientState::Error;
        assert_eq!(client.state().await, ClientState::Error);
        assert!(!client.is_connected().await);
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn test_multiple_state_changes() {
        let client = ComputeProviderClient::new(ComputeProviderConfig::default());

        // Initial state
        assert_eq!(client.state().await, ClientState::Disconnected);

        // Connecting
        *client.state.write().await = ClientState::Connecting;
        assert_eq!(client.state().await, ClientState::Connecting);
        assert!(!client.is_connected().await);

        // Connected
        *client.state.write().await = ClientState::Connected;
        assert_eq!(client.state().await, ClientState::Connected);
        assert!(client.is_connected().await);

        // Error
        *client.state.write().await = ClientState::Error;
        assert_eq!(client.state().await, ClientState::Error);
        assert!(!client.is_connected().await);

        // Back to disconnected
        *client.state.write().await = ClientState::Disconnected;
        assert_eq!(client.state().await, ClientState::Disconnected);
    }
}
