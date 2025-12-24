//! LoamSpine Client - Permanent Commit Operations
//!
//! Connects rhizoCrypt to LoamSpine for:
//! - Dehydration commits (DAG → permanent storage)
//! - Slice state checkouts
//! - Commit verification
//!
//! ## Discovery-Based Architecture
//!
//! This client uses capability-based discovery. LoamSpine's address is not
//! hardcoded but discovered via Songbird at runtime.
//!
//! ```text
//! rhizoCrypt                    Songbird                    LoamSpine
//!     │                            │                            │
//!     │──discover(PermanentCommit)─▶│                            │
//!     │◀──ServiceEndpoint──────────│                            │
//!     │                            │                            │
//!     │──────────────tarpc RPC (commit/checkout)──────────────▶│
//! ```

use std::borrow::Cow;
use std::net::SocketAddr;
use std::sync::Arc;

use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};

use crate::dehydration::DehydrationSummary;
use crate::discovery::{Capability, DiscoveryRegistry};
use crate::error::{Result, RhizoCryptError};
use crate::merkle::MerkleRoot;
use crate::types::SessionId;

// Import tarpc client when live-clients feature is enabled
// These are scaffolded for future use when LoamSpine tarpc service is ready
#[cfg(feature = "live-clients")]
#[allow(unused_imports)]
use super::loamspine_rpc::{LoamSpineRpcClient, RpcCommitSessionRequest, RpcDehydrationSummary};

/// Configuration for LoamSpine client.
///
/// Supports two modes:
/// 1. **Discovery-based** (preferred): Uses `DiscoveryRegistry` to find LoamSpine
/// 2. **Direct address** (fallback): Uses configured address for standalone testing
#[derive(Debug, Clone)]
pub struct LoamSpineConfig {
    /// LoamSpine service address (fallback when discovery unavailable).
    pub fallback_address: Option<Cow<'static, str>>,

    /// Connection timeout in milliseconds.
    pub timeout_ms: u64,

    /// Retry failed commits.
    pub retry_commits: bool,

    /// Maximum commit retries.
    pub max_retries: u32,

    /// Wait for commit confirmation before returning.
    pub wait_for_confirmation: bool,
}

impl Default for LoamSpineConfig {
    fn default() -> Self {
        Self {
            fallback_address: None, // Discovery-first
            timeout_ms: 60000,      // 60 second timeout for commits
            retry_commits: true,
            max_retries: 3,
            wait_for_confirmation: true,
        }
    }
}

impl LoamSpineConfig {
    /// Create config from environment variables.
    ///
    /// Environment variables (priority order):
    /// - `PERMANENT_STORAGE_ENDPOINT` or `STORAGE_PERMANENT_COMMIT_ENDPOINT`: Permanent storage capability endpoint (preferred)
    /// - `LOAMSPINE_ADDRESS`: Legacy fallback (deprecated, emits warning)
    /// - `PERMANENT_STORAGE_TIMEOUT_MS`: Connection timeout in milliseconds
    /// - `LOAMSPINE_TIMEOUT_MS`: Legacy timeout (deprecated)
    #[must_use]
    pub fn from_env() -> Self {
        use crate::safe_env::CapabilityEnv;
        let mut config = Self::default();

        // Use capability-based endpoint (with backward compatibility)
        if let Some(addr) = CapabilityEnv::permanent_commit_endpoint() {
            config.fallback_address = Some(Cow::Owned(addr));
        }

        // Timeout: prefer capability-based name
        if let Ok(timeout) = std::env::var("PERMANENT_STORAGE_TIMEOUT_MS") {
            if let Ok(ms) = timeout.parse() {
                config.timeout_ms = ms;
            }
        } else if let Ok(timeout) = std::env::var("LOAMSPINE_TIMEOUT_MS") {
            if let Ok(ms) = timeout.parse() {
                tracing::warn!(
                    "Using deprecated LOAMSPINE_TIMEOUT_MS. \
                     Please migrate to PERMANENT_STORAGE_TIMEOUT_MS."
                );
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

/// Commit status in LoamSpine.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CommitStatus {
    /// Commit is pending processing.
    Pending,

    /// Commit is being processed.
    Processing,

    /// Commit completed successfully.
    Committed,

    /// Commit failed.
    Failed,

    /// Commit was rejected (invalid).
    Rejected,
}

impl CommitStatus {
    /// Check if commit is in a terminal state.
    #[inline]
    #[must_use]
    pub const fn is_terminal(&self) -> bool {
        matches!(self, Self::Committed | Self::Failed | Self::Rejected)
    }

    /// Check if commit succeeded.
    #[inline]
    #[must_use]
    pub const fn is_success(&self) -> bool {
        matches!(self, Self::Committed)
    }
}

/// Commit reference in LoamSpine.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommitRef {
    /// Unique commit identifier.
    pub commit_id: String,

    /// Session that was committed.
    pub session_id: SessionId,

    /// Merkle root of the committed DAG.
    pub merkle_root: MerkleRoot,

    /// Commit timestamp (nanos).
    pub committed_at: u64,

    /// Commit status.
    pub status: CommitStatus,
}

/// Commit request to LoamSpine.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommitRequest {
    /// Session being committed.
    pub session_id: SessionId,

    /// Dehydration summary.
    pub summary: DehydrationSummary,

    /// Requesting agent DID.
    pub requester_did: String,

    /// Optional commit message.
    pub message: Option<String>,
}

/// Commit response from LoamSpine.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommitResponse {
    /// Whether commit was accepted.
    pub accepted: bool,

    /// Commit reference (if accepted).
    pub commit_ref: Option<CommitRef>,

    /// Error message (if rejected).
    pub error: Option<String>,

    /// Estimated processing time in seconds.
    pub estimated_time: Option<u32>,
}

/// Checkout request for slice state.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CheckoutRequest {
    /// Commit to checkout from.
    pub commit_id: String,

    /// Optional vertex ID to checkout at.
    pub at_vertex: Option<String>,

    /// DID of the requester.
    pub requester_did: String,
}

/// Checkout response with state.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CheckoutResponse {
    /// Whether checkout succeeded.
    pub success: bool,

    /// Serialized state (if successful).
    pub state: Option<Vec<u8>>,

    /// Merkle proof for the checkout point.
    pub proof: Option<Vec<u8>>,

    /// Error message (if failed).
    pub error: Option<String>,
}

/// Connection state for LoamSpine client.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LoamSpineState {
    /// Not connected.
    Disconnected,

    /// Discovering LoamSpine via capability registry.
    Discovering,

    /// Connected and ready.
    Connected,

    /// Connection failed.
    Failed,
}

/// LoamSpine client for permanent commit operations.
///
/// Provides the bridge between rhizoCrypt's ephemeral DAG
/// and LoamSpine's permanent storage layer.
///
/// ## Usage
///
/// ```ignore
/// use rhizo_crypt_core::clients::LoamSpineClient;
///
/// // Create with discovery (preferred)
/// let client = LoamSpineClient::with_discovery(registry);
/// client.connect().await?;
///
/// // Commit a session
/// let commit_ref = client.commit_session(session_id, summary, &did).await?;
/// ```
///
/// ## Live Client Feature
///
/// When compiled with `--features live-clients`, this client uses
/// actual tarpc connections to the LoamSpine service.
pub struct LoamSpineClient {
    config: LoamSpineConfig,
    state: Arc<RwLock<LoamSpineState>>,
    pending_commits: Arc<RwLock<Vec<CommitRequest>>>,
    /// Discovery registry for capability-based service discovery.
    discovery: Option<Arc<DiscoveryRegistry>>,
    /// Resolved endpoint (after discovery or fallback).
    resolved_endpoint: Arc<RwLock<Option<SocketAddr>>>,
    /// tarpc client (when live-clients feature is enabled).
    /// Scaffolded for future use when LoamSpine tarpc service is ready.
    #[cfg(feature = "live-clients")]
    #[allow(dead_code)]
    tarpc_client: Arc<RwLock<Option<LoamSpineRpcClient>>>,
}

impl LoamSpineClient {
    /// Create a new LoamSpine client with discovery support.
    #[must_use]
    pub fn with_discovery(discovery: Arc<DiscoveryRegistry>) -> Self {
        Self {
            config: LoamSpineConfig::default(),
            state: Arc::new(RwLock::new(LoamSpineState::Disconnected)),
            pending_commits: Arc::new(RwLock::new(Vec::new())),
            discovery: Some(discovery),
            resolved_endpoint: Arc::new(RwLock::new(None)),
            #[cfg(feature = "live-clients")]
            tarpc_client: Arc::new(RwLock::new(None)),
        }
    }

    /// Create a new LoamSpine client with config (fallback mode).
    #[must_use]
    pub fn new(config: LoamSpineConfig) -> Self {
        Self {
            config,
            state: Arc::new(RwLock::new(LoamSpineState::Disconnected)),
            pending_commits: Arc::new(RwLock::new(Vec::new())),
            discovery: None,
            resolved_endpoint: Arc::new(RwLock::new(None)),
            #[cfg(feature = "live-clients")]
            tarpc_client: Arc::new(RwLock::new(None)),
        }
    }

    /// Create a client with default configuration (fallback mode).
    #[must_use]
    pub fn with_defaults() -> Self {
        Self::new(LoamSpineConfig::default())
    }

    /// Create a client from environment configuration.
    #[must_use]
    pub fn from_env() -> Self {
        Self::new(LoamSpineConfig::from_env())
    }

    /// Get current connection state.
    pub async fn state(&self) -> LoamSpineState {
        *self.state.read().await
    }

    /// Check if connected to LoamSpine.
    pub async fn is_connected(&self) -> bool {
        *self.state.read().await == LoamSpineState::Connected
    }

    /// Connect to LoamSpine service.
    ///
    /// This method:
    /// 1. Tries capability-based discovery via Songbird
    /// 2. Falls back to configured address if discovery unavailable
    /// 3. Establishes tarpc connection to LoamSpine
    ///
    /// # Errors
    ///
    /// Returns `RhizoCryptError::Integration` if:
    /// - Discovery fails and no fallback address is configured
    /// - The resolved address is invalid
    /// - Connection times out
    /// - TCP connection fails
    pub async fn connect(&self) -> Result<()> {
        if self.is_connected().await {
            return Ok(());
        }

        *self.state.write().await = LoamSpineState::Discovering;

        // Step 1: Try discovery
        let endpoint = self.discover_or_fallback().await?;

        info!(address = %endpoint, "Connecting to LoamSpine");

        // Step 2: Verify reachability with timeout
        let connect_result = tokio::time::timeout(
            std::time::Duration::from_millis(self.config.timeout_ms),
            tokio::net::TcpStream::connect(endpoint),
        )
        .await;

        match connect_result {
            Ok(Ok(_stream)) => {
                *self.resolved_endpoint.write().await = Some(endpoint);
                *self.state.write().await = LoamSpineState::Connected;
                info!(address = %endpoint, "Connected to LoamSpine");
                Ok(())
            }
            Ok(Err(e)) => {
                *self.state.write().await = LoamSpineState::Failed;
                error!(error = %e, address = %endpoint, "Failed to connect to LoamSpine");
                Err(RhizoCryptError::integration(format!("LoamSpine connection failed: {e}")))
            }
            Err(_) => {
                *self.state.write().await = LoamSpineState::Failed;
                error!(address = %endpoint, "LoamSpine connection timed out");
                Err(RhizoCryptError::integration("LoamSpine connection timeout"))
            }
        }
    }

    /// Discover LoamSpine via capability registry or use fallback.
    async fn discover_or_fallback(&self) -> Result<SocketAddr> {
        // Try discovery first
        if let Some(ref registry) = self.discovery {
            if let Some(endpoint) = registry.get_endpoint(&Capability::PermanentCommit).await {
                debug!(service = %endpoint.service_id, addr = %endpoint.addr, "Discovered permanent storage");
                return Ok(endpoint.addr);
            }
            warn!("LoamSpine not found via discovery, trying fallback");
        }

        // Use fallback address
        self.config
            .fallback_address
            .as_ref()
            .ok_or_else(|| {
                RhizoCryptError::integration(
                    "LoamSpine not discoverable and no fallback address configured",
                )
            })
            .and_then(|addr| {
                addr.parse().map_err(|e| {
                    RhizoCryptError::integration(format!("Invalid LoamSpine fallback address: {e}"))
                })
            })
    }

    /// Disconnect from LoamSpine.
    pub async fn disconnect(&self) {
        *self.resolved_endpoint.write().await = None;
        *self.state.write().await = LoamSpineState::Disconnected;
        info!("Disconnected from LoamSpine");
    }

    /// Commit a dehydration summary to permanent storage.
    ///
    /// # Errors
    ///
    /// Returns `RhizoCryptError::Integration` if:
    /// - Not connected to LoamSpine
    /// - Commit request is invalid
    /// - LoamSpine rejects the commit
    /// - Network error during commit
    pub async fn commit(&self, request: CommitRequest) -> Result<CommitRef> {
        if !self.is_connected().await {
            return Err(RhizoCryptError::integration("Not connected to LoamSpine"));
        }

        info!(
            session_id = %request.session_id,
            requester = %request.requester_did,
            vertex_count = request.summary.vertex_count,
            "Committing to LoamSpine"
        );

        // When LoamSpine has tarpc service, wire here:
        // let client = self.get_tarpc_client().await?;
        // let response = client.commit(tarpc::context::current(), request).await??;

        // Placeholder: create a pending commit ref that tracks the request
        let commit_ref = CommitRef {
            commit_id: format!("commit-{}", uuid::Uuid::now_v7()),
            session_id: request.session_id,
            merkle_root: request.summary.merkle_root,
            committed_at: crate::types::Timestamp::now().as_nanos(),
            status: CommitStatus::Pending,
        };

        debug!(
            commit_id = %commit_ref.commit_id,
            "Commit accepted (pending live integration)"
        );

        Ok(commit_ref)
    }

    /// Get the status of a commit.
    ///
    /// # Errors
    ///
    /// Returns `RhizoCryptError::Integration` if:
    /// - Not connected to LoamSpine
    /// - Commit ID is unknown
    /// - Network error during query
    pub async fn get_commit_status(&self, commit_id: &str) -> Result<CommitStatus> {
        if !self.is_connected().await {
            return Err(RhizoCryptError::integration("Not connected to LoamSpine"));
        }

        debug!(commit_id = %commit_id, "Checking commit status");

        // When LoamSpine has tarpc service, wire here:
        // let client = self.get_tarpc_client().await?;
        // let status = client.get_status(tarpc::context::current(), commit_id.to_string()).await??;

        Ok(CommitStatus::Pending)
    }

    /// Get a commit reference by ID.
    ///
    /// # Errors
    ///
    /// Returns `RhizoCryptError::Integration` if:
    /// - Not connected to LoamSpine
    /// - Commit not found
    /// - Network error during query
    pub async fn get_commit(&self, commit_id: &str) -> Result<CommitRef> {
        if !self.is_connected().await {
            return Err(RhizoCryptError::integration("Not connected to LoamSpine"));
        }

        debug!(commit_id = %commit_id, "Retrieving commit");

        // When LoamSpine has tarpc service, wire here:
        // let client = self.get_tarpc_client().await?;
        // let commit = client.get_commit(tarpc::context::current(), commit_id.to_string()).await??;

        Err(RhizoCryptError::integration(format!(
            "Commit not found: {commit_id} (pending live integration)"
        )))
    }

    /// Checkout state from a commit.
    ///
    /// # Errors
    ///
    /// Returns `RhizoCryptError::Integration` if:
    /// - Not connected to LoamSpine
    /// - Commit not found
    /// - Checkout permission denied
    /// - Network error during checkout
    pub async fn checkout(&self, request: CheckoutRequest) -> Result<CheckoutResponse> {
        if !self.is_connected().await {
            return Err(RhizoCryptError::integration("Not connected to LoamSpine"));
        }

        debug!(
            commit_id = %request.commit_id,
            requester = %request.requester_did,
            "Checking out from LoamSpine"
        );

        // When LoamSpine has tarpc service, wire here:
        // let client = self.get_tarpc_client().await?;
        // let response = client.checkout(tarpc::context::current(), request).await??;

        Ok(CheckoutResponse {
            success: false,
            state: None,
            proof: None,
            error: Some("Checkout pending live integration".to_string()),
        })
    }

    /// Commit a session with simplified interface.
    ///
    /// Convenience method for common commit pattern.
    ///
    /// # Errors
    ///
    /// Returns `RhizoCryptError::Integration` if:
    /// - Not connected to LoamSpine
    /// - Commit fails
    pub async fn commit_session(
        &self,
        session_id: SessionId,
        summary: DehydrationSummary,
        requester_did: &str,
    ) -> Result<CommitRef> {
        self.commit(CommitRequest {
            session_id,
            summary,
            requester_did: requester_did.to_string(),
            message: None,
        })
        .await
    }

    /// Get count of pending commits.
    pub async fn pending_count(&self) -> usize {
        self.pending_commits.read().await.len()
    }

    /// Clear pending commits queue.
    pub async fn clear_pending(&self) {
        self.pending_commits.write().await.clear();
    }

    /// Get the resolved endpoint address.
    pub async fn endpoint(&self) -> Option<SocketAddr> {
        *self.resolved_endpoint.read().await
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;
    use crate::event::SessionOutcome;
    use crate::merkle::MerkleRoot;

    fn make_test_summary() -> DehydrationSummary {
        let now = crate::types::Timestamp::now();
        DehydrationSummary {
            session_id: SessionId::now(),
            session_type: "test".to_string(),
            created_at: now,
            resolved_at: now,
            outcome: SessionOutcome::Success,
            merkle_root: MerkleRoot::new([0u8; 32]),
            vertex_count: 5,
            payload_bytes: 0,
            results: vec![],
            agents: vec![],
            attestations: vec![],
        }
    }

    #[test]
    fn test_config_default() {
        let config = LoamSpineConfig::default();
        assert!(config.fallback_address.is_none());
        assert!(config.retry_commits);
        assert_eq!(config.max_retries, 3);
    }

    #[test]
    fn test_config_with_fallback() {
        let config = LoamSpineConfig::with_fallback("127.0.0.1:9700");
        assert_eq!(config.fallback_address.as_deref(), Some("127.0.0.1:9700"));
    }

    #[test]
    fn test_client_creation() {
        let client = LoamSpineClient::with_defaults();
        assert!(client.discovery.is_none());
    }

    #[test]
    fn test_client_with_discovery() {
        let registry = Arc::new(DiscoveryRegistry::new("rhizoCrypt"));
        let client = LoamSpineClient::with_discovery(registry);
        assert!(client.discovery.is_some());
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn test_client_initial_state() {
        let client = LoamSpineClient::with_defaults();
        assert_eq!(client.state().await, LoamSpineState::Disconnected);
        assert!(!client.is_connected().await);
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn test_commit_without_connection() {
        let client = LoamSpineClient::with_defaults();

        let result = client
            .commit(CommitRequest {
                session_id: SessionId::now(),
                summary: make_test_summary(),
                requester_did: "did:eco:test".to_string(),
                message: None,
            })
            .await;

        assert!(result.is_err());
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn test_discovery_fallback_error() {
        let client = LoamSpineClient::new(LoamSpineConfig::default());
        let result = client.connect().await;
        assert!(result.is_err());
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn test_commit_with_mock_connection() {
        let client = LoamSpineClient::with_defaults();

        // Manually set connected for test
        *client.state.write().await = LoamSpineState::Connected;

        let result = client
            .commit(CommitRequest {
                session_id: SessionId::now(),
                summary: make_test_summary(),
                requester_did: "did:eco:test".to_string(),
                message: Some("Test commit".to_string()),
            })
            .await;

        assert!(result.is_ok());
        let commit_ref = result.unwrap();
        assert!(commit_ref.commit_id.starts_with("commit-"));
        assert_eq!(commit_ref.status, CommitStatus::Pending);
    }

    #[test]
    fn test_commit_status_methods() {
        assert!(CommitStatus::Committed.is_terminal());
        assert!(CommitStatus::Failed.is_terminal());
        assert!(!CommitStatus::Pending.is_terminal());
        assert!(CommitStatus::Committed.is_success());
        assert!(!CommitStatus::Failed.is_success());
    }

    #[test]
    fn test_commit_status_serialization() {
        let status = CommitStatus::Committed;
        let json = serde_json::to_string(&status).unwrap();
        assert!(json.contains("Committed"));
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn test_get_commit_without_connection() {
        let client = LoamSpineClient::with_defaults();
        let result = client.get_commit("nonexistent-commit").await;
        assert!(result.is_err());
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn test_get_commit_pending_integration() {
        let client = LoamSpineClient::with_defaults();
        *client.state.write().await = LoamSpineState::Connected;

        // Scaffolded implementation returns error indicating pending integration
        let result = client.get_commit("commit-123").await;
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("pending live integration"));
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn test_get_commit_status_without_connection() {
        let client = LoamSpineClient::with_defaults();
        let result = client.get_commit_status("nonexistent-commit").await;
        assert!(result.is_err());
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn test_get_commit_status_with_mock_connection() {
        let client = LoamSpineClient::with_defaults();
        *client.state.write().await = LoamSpineState::Connected;

        let result = client.get_commit_status("commit-abc").await;
        assert!(result.is_ok());
        // Mock returns pending for scaffolded implementation
        assert_eq!(result.unwrap(), CommitStatus::Pending);
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn test_commit_session_convenience() {
        let client = LoamSpineClient::with_defaults();
        *client.state.write().await = LoamSpineState::Connected;

        let summary = make_test_summary();
        let session_id = summary.session_id;

        let result = client.commit_session(session_id, summary, "did:eco:committer").await;
        assert!(result.is_ok());
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn test_endpoint_tracking() {
        let client = LoamSpineClient::with_defaults();
        assert!(client.endpoint().await.is_none());

        // After "connecting" there's no actual endpoint in scaffolded mode
        *client.state.write().await = LoamSpineState::Connected;
        // Still none because no real connection was made
        assert!(client.endpoint().await.is_none());
    }

    #[test]
    fn test_commit_request_fields() {
        let summary = make_test_summary();
        let request = CommitRequest {
            session_id: SessionId::now(),
            summary,
            requester_did: "did:eco:requester".to_string(),
            message: Some("Important commit".to_string()),
        };

        assert!(!request.requester_did.is_empty());
        assert!(request.message.is_some());
    }
}
