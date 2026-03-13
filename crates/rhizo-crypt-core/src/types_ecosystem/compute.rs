// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2024–2026 ecoPrimals Project

//! Compute Provider Types - Task Events & Configuration
//!
//! Type definitions for compute orchestration capability providers.
//! These types work with ANY compute provider (compute provider, Kubernetes, Nomad, custom).
//!
//! ## Capability-Based Architecture
//!
//! Compute providers are discovered via the `compute:orchestration` capability.
//! The primal doesn't know or care which specific service provides the capability.
//!
//! ```text
//! rhizoCrypt              Bootstrap              Compute Provider
//!     │                      │                         │
//!     │──discover(compute)──▶│                         │
//!     │◀──ServiceEndpoint────│                         │
//!     │                      │                         │
//!     │──────────subscribe(task_id)────────────────────▶│
//!     │◀────────Stream<ComputeEvent>───────────────────│
//! ```

use std::borrow::Cow;
use std::net::SocketAddr;
use std::sync::Arc;

use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

use crate::discovery::{Capability, DiscoveryRegistry};
use crate::error::{Result, RhizoCryptError};
use crate::types::{Did, PayloadRef, Timestamp};

/// Task identifier for compute provider compute tasks.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TaskId(pub [u8; 32]);

impl TaskId {
    /// Create a new task ID from bytes.
    #[must_use]
    pub const fn new(bytes: [u8; 32]) -> Self {
        Self(bytes)
    }

    /// Create a task ID from a UUID v7.
    #[must_use]
    pub fn now() -> Self {
        let uuid = uuid::Uuid::now_v7();
        let mut bytes = [0u8; 32];
        bytes[..16].copy_from_slice(uuid.as_bytes());
        Self(bytes)
    }

    /// Get the underlying bytes.
    #[must_use]
    pub const fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }
}

impl Default for TaskId {
    fn default() -> Self {
        Self::now()
    }
}

impl std::fmt::Display for TaskId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // Format first 8 bytes as hex without external dependency
        for byte in &self.0[..8] {
            write!(f, "{byte:02x}")?;
        }
        Ok(())
    }
}

/// Compute events from compute provider.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum ComputeEvent {
    /// Task created.
    TaskCreated {
        /// Task identifier.
        task_id: TaskId,
        /// Task type (e.g., "ml-training", "inference").
        task_type: String,
        /// Requester DID.
        requester: Did,
        /// Creation timestamp.
        created_at: Timestamp,
    },
    /// Task started execution.
    TaskStarted {
        /// Task identifier.
        task_id: TaskId,
        /// Worker DID.
        worker: Did,
        /// Start timestamp.
        started_at: Timestamp,
    },
    /// Task progress update.
    TaskProgress {
        /// Task identifier.
        task_id: TaskId,
        /// Progress (0.0 to 1.0).
        progress: f32,
        /// Optional status message.
        message: Option<String>,
        /// Update timestamp.
        updated_at: Timestamp,
    },
    /// Task completed successfully.
    TaskCompleted {
        /// Task identifier.
        task_id: TaskId,
        /// Result payload reference.
        result_ref: PayloadRef,
        /// Completion timestamp.
        completed_at: Timestamp,
    },
    /// Task failed.
    TaskFailed {
        /// Task identifier.
        task_id: TaskId,
        /// Error message.
        error: String,
        /// Failure timestamp.
        failed_at: Timestamp,
    },
    /// Task cancelled.
    TaskCancelled {
        /// Task identifier.
        task_id: TaskId,
        /// Cancellation reason.
        reason: String,
        /// Cancellation timestamp.
        cancelled_at: Timestamp,
    },
}

impl ComputeEvent {
    /// Get the task ID for this event.
    #[must_use]
    pub const fn task_id(&self) -> TaskId {
        match self {
            Self::TaskCreated {
                task_id,
                ..
            }
            | Self::TaskStarted {
                task_id,
                ..
            }
            | Self::TaskProgress {
                task_id,
                ..
            }
            | Self::TaskCompleted {
                task_id,
                ..
            }
            | Self::TaskFailed {
                task_id,
                ..
            }
            | Self::TaskCancelled {
                task_id,
                ..
            } => *task_id,
        }
    }

    /// Get the event type name.
    #[must_use]
    pub const fn event_type(&self) -> &'static str {
        match self {
            Self::TaskCreated {
                ..
            } => "task.created",
            Self::TaskStarted {
                ..
            } => "task.started",
            Self::TaskProgress {
                ..
            } => "task.progress",
            Self::TaskCompleted {
                ..
            } => "task.completed",
            Self::TaskFailed {
                ..
            } => "task.failed",
            Self::TaskCancelled {
                ..
            } => "task.cancelled",
        }
    }

    /// Check if this is a terminal event.
    #[must_use]
    pub const fn is_terminal(&self) -> bool {
        matches!(
            self,
            Self::TaskCompleted { .. } | Self::TaskFailed { .. } | Self::TaskCancelled { .. }
        )
    }
}

/// Configuration for compute provider client.
#[derive(Debug, Clone)]
pub struct ComputeProviderConfig {
    /// compute provider service address (fallback when discovery unavailable).
    pub fallback_address: Option<Cow<'static, str>>,

    /// Connection timeout in milliseconds.
    pub timeout_ms: u64,

    /// Event buffer size for subscriptions.
    pub event_buffer_size: usize,

    /// Retry attempts for failed connections.
    pub max_retries: u8,
}

impl Default for ComputeProviderConfig {
    fn default() -> Self {
        Self {
            fallback_address: None, // No fallback - use discovery
            timeout_ms: 5000,
            event_buffer_size: 1000,
            max_retries: 3,
        }
    }
}

impl ComputeProviderConfig {
    /// Create config from environment variables.
    ///
    /// Environment variables:
    /// - `COMPUTE_ENDPOINT` or `COMPUTE_ORCHESTRATION_ENDPOINT`: Compute capability endpoint
    /// - `COMPUTE_TIMEOUT_MS`: Connection timeout in milliseconds
    #[must_use]
    pub fn from_env() -> Self {
        use crate::safe_env::CapabilityEnv;
        let mut config = Self::default();

        if let Some(addr) = CapabilityEnv::compute_endpoint() {
            config.fallback_address = Some(Cow::Owned(addr));
        }

        if let Ok(timeout) = std::env::var("COMPUTE_TIMEOUT_MS") {
            if let Ok(ms) = timeout.parse() {
                config.timeout_ms = ms;
            }
        }

        config
    }

    /// Create config with a specific fallback address (for testing).
    #[must_use]
    pub fn with_fallback(address: impl Into<Cow<'static, str>>) -> Self {
        Self {
            fallback_address: Some(address.into()),
            ..Self::default()
        }
    }
}

/// Client state.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ClientState {
    /// Not connected.
    #[default]
    Disconnected,
    /// Connection in progress.
    Connecting,
    /// Connected and ready.
    Connected,
    /// Disconnected due to error.
    Error,
}

/// Compute provider client for task event integration.
///
/// Provides subscription to compute task events for integration
/// into rhizoCrypt sessions.
///
/// ## Usage
///
/// ```rust,ignore
/// use rhizo_crypt_core::types_ecosystem::compute::{ComputeProviderClient, ComputeProviderConfig};
///
/// // Create with discovery (preferred)
/// let client = ComputeProviderClient::with_discovery(registry);
/// client.connect().await?;
///
/// // Subscribe to a task
/// let mut events = client.subscribe_task(task_id).await?;
/// while let Some(event) = events.recv().await {
///     println!("Event: {:?}", event);
/// }
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
        if let Some(registry) = &self.registry {
            if let Some(endpoint) = registry.get_endpoint(&Capability::ComputeOrchestration).await {
                info!(address = %endpoint.addr, "Discovered compute provider via registry");
                return self.connect_to(endpoint.addr).await;
            }
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
        agent: &Did,
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
    #[allow(clippy::missing_const_for_fn)] // as_ref() is not const-stable
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
    use super::*;

    #[test]
    fn test_config_default() {
        let config = ComputeProviderConfig::default();
        assert!(config.fallback_address.is_none());
        assert_eq!(config.timeout_ms, 5000);
    }

    #[test]
    fn test_config_with_fallback() {
        let config = ComputeProviderConfig::with_fallback("127.0.0.1:9800");
        assert_eq!(config.fallback_address.as_deref(), Some("127.0.0.1:9800"));
    }

    #[test]
    fn test_task_id_display() {
        let id = TaskId::new([
            0xDE, 0xAD, 0xBE, 0xEF, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0,
        ]);
        assert_eq!(format!("{id}"), "deadbeef00000000");
    }

    #[test]
    fn test_compute_event_type() {
        let event = ComputeEvent::TaskCreated {
            task_id: TaskId::now(),
            task_type: "test".to_string(),
            requester: Did::new("did:key:test"),
            created_at: Timestamp::now(),
        };
        assert_eq!(event.event_type(), "task.created");
        assert!(!event.is_terminal());

        let completed = ComputeEvent::TaskCompleted {
            task_id: TaskId::now(),
            result_ref: PayloadRef::from_bytes(b"test-result"),
            completed_at: Timestamp::now(),
        };
        assert!(completed.is_terminal());
    }

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

    // ============================================================================
    // Additional Tests for Coverage Boost (40% → 80%+)
    // ============================================================================

    #[test]
    fn test_task_id_new() {
        let bytes = [1u8; 32];
        let id = TaskId::new(bytes);
        assert_eq!(id.as_bytes(), &bytes);
    }

    #[test]
    fn test_task_id_now() {
        let id1 = TaskId::now();
        let id2 = TaskId::now();
        // UUIDs should be different
        assert_ne!(id1, id2);
    }

    #[test]
    fn test_task_id_default() {
        let id = TaskId::default();
        // Should not be all zeros
        assert_ne!(id.as_bytes(), &[0u8; 32]);
    }

    #[test]
    fn test_task_id_equality() {
        let bytes = [42u8; 32];
        let id1 = TaskId::new(bytes);
        let id2 = TaskId::new(bytes);
        assert_eq!(id1, id2);

        let id3 = TaskId::now();
        assert_ne!(id1, id3);
    }

    #[test]
    fn test_compute_event_task_started() {
        let event = ComputeEvent::TaskStarted {
            task_id: TaskId::now(),
            worker: Did::new("did:key:worker"),
            started_at: Timestamp::now(),
        };
        assert_eq!(event.event_type(), "task.started");
        assert!(!event.is_terminal());
    }

    #[test]
    fn test_compute_event_task_progress() {
        let event = ComputeEvent::TaskProgress {
            task_id: TaskId::now(),
            progress: 0.5,
            message: Some("Processing...".to_string()),
            updated_at: Timestamp::now(),
        };
        assert_eq!(event.event_type(), "task.progress");
        assert!(!event.is_terminal());
    }

    #[test]
    fn test_compute_event_task_failed() {
        let event = ComputeEvent::TaskFailed {
            task_id: TaskId::now(),
            error: "Out of memory".to_string(),
            failed_at: Timestamp::now(),
        };
        assert_eq!(event.event_type(), "task.failed");
        assert!(event.is_terminal());
    }

    #[test]
    fn test_compute_event_task_cancelled() {
        let event = ComputeEvent::TaskCancelled {
            task_id: TaskId::now(),
            reason: "Cancelled by admin".to_string(),
            cancelled_at: Timestamp::now(),
        };
        assert_eq!(event.event_type(), "task.cancelled");
        assert!(event.is_terminal());
    }

    #[test]
    fn test_config_from_env() {
        let config = ComputeProviderConfig::from_env();
        // Default values (may be overridden by env vars)
        assert!(config.timeout_ms > 0);
        assert!(config.event_buffer_size > 0);
    }

    #[test]
    fn test_config_with_custom_values() {
        let mut config = ComputeProviderConfig::default();
        config.timeout_ms = 10000;
        config.event_buffer_size = 200;

        assert_eq!(config.timeout_ms, 10000);
        assert_eq!(config.event_buffer_size, 200);
    }

    #[test]
    fn test_client_state_default() {
        let state = ClientState::default();
        assert_eq!(state, ClientState::Disconnected);
    }

    #[test]
    fn test_client_state_transitions() {
        let state1 = ClientState::Disconnected;
        let state2 = ClientState::Connecting;
        let state3 = ClientState::Connected;
        let state4 = ClientState::Error;

        assert_ne!(state1, state2);
        assert_ne!(state2, state3);
        assert_ne!(state3, state4);
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
        let did = Did::new("did:key:test");
        let result = client.subscribe_agent(&did).await;

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("Not connected"));
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn test_subscribe_agent_connected() {
        let client = ComputeProviderClient::new(ComputeProviderConfig::default());
        *client.state.write().await = ClientState::Connected;

        let did = Did::new("did:key:test");
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

    #[test]
    fn test_compute_event_all_types() {
        let task_id = TaskId::now();
        let timestamp = Timestamp::now();
        let did = Did::new("did:key:test");

        // TaskCreated
        let event = ComputeEvent::TaskCreated {
            task_id,
            task_type: "ml-training".to_string(),
            requester: did.clone(),
            created_at: timestamp,
        };
        assert_eq!(event.event_type(), "task.created");
        assert!(!event.is_terminal());

        // TaskStarted
        let event = ComputeEvent::TaskStarted {
            task_id,
            worker: did,
            started_at: timestamp,
        };
        assert_eq!(event.event_type(), "task.started");
        assert!(!event.is_terminal());

        // TaskProgress with message
        let event = ComputeEvent::TaskProgress {
            task_id,
            progress: 0.75,
            message: Some("Almost done".to_string()),
            updated_at: timestamp,
        };
        assert_eq!(event.event_type(), "task.progress");
        assert!(!event.is_terminal());

        // TaskProgress without message
        let event = ComputeEvent::TaskProgress {
            task_id,
            progress: 0.5,
            message: None,
            updated_at: timestamp,
        };
        assert!(!event.is_terminal());

        // TaskCompleted
        let event = ComputeEvent::TaskCompleted {
            task_id,
            result_ref: PayloadRef::from_bytes(b"result"),
            completed_at: timestamp,
        };
        assert_eq!(event.event_type(), "task.completed");
        assert!(event.is_terminal());

        // TaskFailed
        let event = ComputeEvent::TaskFailed {
            task_id,
            error: "Test error".to_string(),
            failed_at: timestamp,
        };
        assert_eq!(event.event_type(), "task.failed");
        assert!(event.is_terminal());

        // TaskCancelled
        let event = ComputeEvent::TaskCancelled {
            task_id,
            reason: "User requested".to_string(),
            cancelled_at: timestamp,
        };
        assert_eq!(event.event_type(), "task.cancelled");
        assert!(event.is_terminal());
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

    #[test]
    fn test_task_id_serialization() {
        let id = TaskId::now();
        let serialized = serde_json::to_string(&id).unwrap();
        let deserialized: TaskId = serde_json::from_str(&serialized).unwrap();
        assert_eq!(id, deserialized);
    }

    #[test]
    fn test_compute_event_serialization() {
        let event = ComputeEvent::TaskCreated {
            task_id: TaskId::now(),
            task_type: "test".to_string(),
            requester: Did::new("did:key:test"),
            created_at: Timestamp::now(),
        };

        let serialized = serde_json::to_string(&event).unwrap();
        let deserialized: ComputeEvent = serde_json::from_str(&serialized).unwrap();
        assert_eq!(deserialized.event_type(), "task.created");
    }
}
