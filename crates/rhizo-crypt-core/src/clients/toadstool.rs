//! ToadStool Client - Compute Event Sourcing
//!
//! Connects rhizoCrypt to ToadStool for:
//! - Compute task event subscription
//! - ML pipeline status tracking
//! - Task result integration
//!
//! ## Discovery-Based Architecture
//!
//! ToadStool's address is discovered via Songbird at runtime.
//!
//! ```text
//! rhizoCrypt                    Songbird                     ToadStool
//!     │                            │                            │
//!     │──discover(compute-events)─▶│                            │
//!     │◀──ServiceEndpoint──────────│                            │
//!     │                            │                            │
//!     │──────────────subscribe(task_id)─────────────────────────▶│
//!     │◀─────────────Stream<ComputeEvent>───────────────────────│
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

/// Task identifier for ToadStool compute tasks.
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

/// Compute events from ToadStool.
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

/// Configuration for ToadStool client.
#[derive(Debug, Clone)]
pub struct ToadStoolConfig {
    /// ToadStool service address (fallback when discovery unavailable).
    pub fallback_address: Option<Cow<'static, str>>,

    /// Connection timeout in milliseconds.
    pub timeout_ms: u64,

    /// Event buffer size for subscriptions.
    pub event_buffer_size: usize,

    /// Retry attempts for failed connections.
    pub max_retries: u8,
}

impl Default for ToadStoolConfig {
    fn default() -> Self {
        Self {
            fallback_address: None, // No fallback - use discovery
            timeout_ms: 5000,
            event_buffer_size: 1000,
            max_retries: 3,
        }
    }
}

impl ToadStoolConfig {
    /// Create config from environment variables.
    ///
    /// Environment variables:
    /// - `TOADSTOOL_ADDRESS`: ToadStool service address (fallback only)
    /// - `TOADSTOOL_TIMEOUT_MS`: Connection timeout in milliseconds
    #[must_use]
    pub fn from_env() -> Self {
        let mut config = Self::default();

        if let Ok(addr) = std::env::var("TOADSTOOL_ADDRESS") {
            config.fallback_address = Some(Cow::Owned(addr));
        }

        if let Ok(timeout) = std::env::var("TOADSTOOL_TIMEOUT_MS") {
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

/// ToadStool client for compute event integration.
///
/// Provides subscription to compute task events for integration
/// into rhizoCrypt sessions.
///
/// ## Usage
///
/// ```rust,ignore
/// use rhizo_crypt_core::clients::{ToadStoolClient, ToadStoolConfig};
///
/// // Create with discovery (preferred)
/// let client = ToadStoolClient::with_discovery(registry);
/// client.connect().await?;
///
/// // Subscribe to a task
/// let mut events = client.subscribe_task(task_id).await?;
/// while let Some(event) = events.recv().await {
///     println!("Event: {:?}", event);
/// }
/// ```
pub struct ToadStoolClient {
    /// Client configuration.
    pub config: ToadStoolConfig,

    /// Discovery registry for finding ToadStool.
    registry: Option<Arc<DiscoveryRegistry>>,

    /// Current connection state.
    state: Arc<RwLock<ClientState>>,

    /// Connected endpoint (if any).
    endpoint: Arc<RwLock<Option<SocketAddr>>>,
}

impl ToadStoolClient {
    /// Create a new client with the given configuration.
    #[must_use]
    pub fn new(config: ToadStoolConfig) -> Self {
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
            config: ToadStoolConfig::from_env(),
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

    /// Connect to ToadStool.
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
                info!(address = %endpoint.addr, "Discovered ToadStool via registry");
                return self.connect_to(endpoint.addr).await;
            }
        }

        // Fall back to configured address
        if let Some(ref addr) = self.config.fallback_address {
            let socket_addr: SocketAddr = addr.parse().map_err(|e| {
                RhizoCryptError::integration(format!("Invalid ToadStool address '{addr}': {e}"))
            })?;
            return self.connect_to(socket_addr).await;
        }

        *self.state.write().await = ClientState::Error;
        Err(RhizoCryptError::integration(
            "No ToadStool address available. Set TOADSTOOL_ADDRESS or use discovery.",
        ))
    }

    /// Connect to a specific address.
    async fn connect_to(&self, addr: SocketAddr) -> Result<()> {
        debug!(address = %addr, "Connecting to ToadStool");

        // Scaffolded mode: verify we can reach the address
        match tokio::time::timeout(
            std::time::Duration::from_millis(self.config.timeout_ms),
            tokio::net::TcpStream::connect(addr),
        )
        .await
        {
            Ok(Ok(_stream)) => {
                info!(address = %addr, "Connected to ToadStool (scaffolded mode)");
                *self.endpoint.write().await = Some(addr);
                *self.state.write().await = ClientState::Connected;
                Ok(())
            }
            Ok(Err(e)) => {
                warn!(error = %e, "Failed to connect to ToadStool");
                *self.state.write().await = ClientState::Error;
                Err(RhizoCryptError::integration(format!("Failed to connect to ToadStool: {e}")))
            }
            Err(_) => {
                warn!("ToadStool connection timeout");
                *self.state.write().await = ClientState::Error;
                Err(RhizoCryptError::integration("ToadStool connection timeout"))
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
                "Not connected to ToadStool. Call connect() first.",
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
                "Not connected to ToadStool. Call connect() first.",
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
                "Not connected to ToadStool. Call connect() first.",
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
        let config = ToadStoolConfig::default();
        assert!(config.fallback_address.is_none());
        assert_eq!(config.timeout_ms, 5000);
    }

    #[test]
    fn test_config_with_fallback() {
        let config = ToadStoolConfig::with_fallback("127.0.0.1:9800");
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

    #[tokio::test]
    async fn test_client_creation() {
        let config = ToadStoolConfig::default();
        let client = ToadStoolClient::new(config);
        assert_eq!(client.state().await, ClientState::Disconnected);
    }

    #[tokio::test]
    async fn test_subscribe_without_connection() {
        let client = ToadStoolClient::new(ToadStoolConfig::default());
        let result = client.subscribe_task(TaskId::now()).await;
        assert!(result.is_err());
    }
}
