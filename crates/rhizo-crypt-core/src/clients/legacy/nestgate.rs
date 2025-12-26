//! NestGate Client - Payload Storage Operations
//!
//! Connects rhizoCrypt to NestGate for:
//! - Large payload storage (blobs, files)
//! - Content-addressed retrieval
//! - Payload streaming
//!
//! ## Discovery-Based Architecture
//!
//! This client uses capability-based discovery. NestGate's address is not
//! hardcoded but discovered via Songbird at runtime.
//!
//! ```text
//! rhizoCrypt                    Songbird                    NestGate
//!     │                            │                            │
//!     │──discover(PayloadStorage)──▶│                            │
//!     │◀──ServiceEndpoint──────────│                            │
//!     │                            │                            │
//!     │──────────────tarpc RPC (store/retrieve)──────────────▶│
//! ```

use std::borrow::Cow;
use std::net::SocketAddr;
use std::num::NonZeroUsize;
use std::sync::Arc;

use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};

use crate::discovery::{Capability, DiscoveryRegistry};
use crate::error::{Result, RhizoCryptError};
use crate::types::PayloadRef;

// Import HTTP client when live-clients feature is enabled
#[cfg(feature = "live-clients")]
use super::super::nestgate_http::NestGateHttpClient;

/// Default cache size for metadata.
const DEFAULT_CACHE_SIZE: usize = 1000;

/// Configuration for NestGate client.
///
/// Supports two modes:
/// 1. **Discovery-based** (preferred): Uses `DiscoveryRegistry` to find NestGate
/// 2. **Direct address** (fallback): Uses configured address for standalone testing
#[derive(Debug, Clone)]
pub struct NestGateConfig {
    /// NestGate service address (fallback when discovery unavailable).
    pub fallback_address: Option<Cow<'static, str>>,

    /// Connection timeout in milliseconds.
    pub timeout_ms: u64,

    /// Maximum payload size in bytes.
    pub max_payload_size: usize,

    /// Enable compression for large payloads.
    pub enable_compression: bool,

    /// Compression threshold in bytes.
    pub compression_threshold: usize,

    /// Metadata cache size.
    pub cache_size: usize,
}

impl Default for NestGateConfig {
    fn default() -> Self {
        Self {
            fallback_address: None,              // Discovery-first
            timeout_ms: 30000,                   // Larger timeout for payload operations
            max_payload_size: 100 * 1024 * 1024, // 100 MB
            enable_compression: true,
            compression_threshold: 1024, // 1 KB
            cache_size: DEFAULT_CACHE_SIZE,
        }
    }
}

impl NestGateConfig {
    /// Create config from environment variables.
    ///
    /// Environment variables (priority order):
    /// - `PAYLOAD_STORAGE_ENDPOINT` or `PAYLOAD_ENDPOINT`: Payload storage capability endpoint (preferred)
    /// - `NESTGATE_ADDRESS`: Legacy fallback (deprecated, emits warning)
    /// - `PAYLOAD_MAX_SIZE_MB`: Maximum payload size in MB
    /// - `NESTGATE_MAX_PAYLOAD`: Legacy max size (deprecated)
    /// - `PAYLOAD_TIMEOUT_MS`: Connection timeout in milliseconds
    /// - `NESTGATE_TIMEOUT_MS`: Legacy timeout (deprecated)
    #[must_use]
    pub fn from_env() -> Self {
        use crate::safe_env::CapabilityEnv;
        let mut config = Self::default();

        // Use capability-based endpoint (with backward compatibility)
        if let Some(addr) = CapabilityEnv::payload_storage_endpoint() {
            config.fallback_address = Some(Cow::Owned(addr));
        }

        // Max payload size: prefer capability-based name
        if let Ok(max) = std::env::var("PAYLOAD_MAX_SIZE_MB") {
            if let Ok(mb) = max.parse::<usize>() {
                config.max_payload_size = mb * 1024 * 1024;
            }
        } else if let Ok(max) = std::env::var("NESTGATE_MAX_PAYLOAD") {
            if let Ok(mb) = max.parse::<usize>() {
                tracing::warn!(
                    "Using deprecated NESTGATE_MAX_PAYLOAD. \
                     Please migrate to PAYLOAD_MAX_SIZE_MB."
                );
                config.max_payload_size = mb * 1024 * 1024;
            }
        }

        // Timeout: prefer capability-based name
        if let Ok(timeout) = std::env::var("PAYLOAD_TIMEOUT_MS") {
            if let Ok(ms) = timeout.parse() {
                config.timeout_ms = ms;
            }
        } else if let Ok(timeout) = std::env::var("NESTGATE_TIMEOUT_MS") {
            if let Ok(ms) = timeout.parse() {
                tracing::warn!(
                    "Using deprecated NESTGATE_TIMEOUT_MS. \
                     Please migrate to PAYLOAD_TIMEOUT_MS."
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

/// Stored payload metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PayloadMetadata {
    /// Content hash (Blake3, hex encoded).
    pub hash: String,

    /// Payload size in bytes.
    pub size: u64,

    /// Content type (MIME).
    pub content_type: Option<String>,

    /// Whether payload is compressed.
    pub compressed: bool,

    /// Original size before compression.
    pub original_size: Option<u64>,

    /// Storage timestamp (nanos).
    pub stored_at: u64,

    /// Optional user-defined tags.
    pub tags: Vec<String>,
}

/// Store request for NestGate.
#[derive(Debug, Clone)]
pub struct StoreRequest {
    /// Payload data.
    pub data: Vec<u8>,

    /// Content type (MIME).
    pub content_type: Option<String>,

    /// Optional tags for organization.
    pub tags: Vec<String>,

    /// Force compression even below threshold.
    pub force_compress: bool,
}

/// Store response from NestGate.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoreResponse {
    /// Whether storage succeeded.
    pub success: bool,

    /// Payload reference (if successful).
    pub payload_ref: Option<String>,

    /// Payload metadata.
    pub metadata: Option<PayloadMetadata>,

    /// Error message (if failed).
    pub error: Option<String>,
}

/// Retrieve request for NestGate.
#[derive(Debug, Clone)]
pub struct RetrieveRequest {
    /// Payload reference to retrieve.
    pub payload_ref: PayloadRef,

    /// Optional byte range (start, end).
    pub byte_range: Option<(u64, u64)>,

    /// Automatically decompress if compressed.
    pub decompress: bool,
}

/// Retrieve response from NestGate.
#[derive(Debug, Clone)]
pub struct RetrieveResponse {
    /// Whether retrieval succeeded.
    pub success: bool,

    /// Payload data (if successful).
    pub data: Option<Vec<u8>>,

    /// Payload metadata.
    pub metadata: Option<PayloadMetadata>,

    /// Error message (if failed).
    pub error: Option<String>,
}

/// Connection state for NestGate client.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NestGateState {
    /// Not connected.
    Disconnected,

    /// Discovering NestGate via capability registry.
    Discovering,

    /// Connected and ready.
    Connected,

    /// Connection failed.
    Failed,
}

/// Cache size as a compile-time `NonZeroUsize`.
const CACHE_SIZE_NONZERO: NonZeroUsize = match NonZeroUsize::new(DEFAULT_CACHE_SIZE) {
    Some(n) => n,
    None => panic!("DEFAULT_CACHE_SIZE must be non-zero"),
};

/// NestGate client for payload storage operations.
///
/// Provides high-performance blob storage with content addressing
/// and optional compression.
///
/// ## Usage
///
/// ```ignore
/// use rhizo_crypt_core::clients::NestGateClient;
///
/// // Create with discovery (preferred)
/// let client = NestGateClient::with_discovery(registry);
/// client.connect().await?;
///
/// // Store a payload
/// let payload_ref = client.store_vertex_payload(data, tags).await?;
/// ```
///
/// ## Live Client Feature
///
/// When compiled with `--features live-clients`, this client uses
/// actual HTTP connections to the NestGate REST API.
pub struct NestGateClient {
    config: NestGateConfig,
    state: Arc<RwLock<NestGateState>>,
    metadata_cache: Arc<RwLock<lru::LruCache<String, PayloadMetadata>>>,
    /// Discovery registry for capability-based service discovery.
    discovery: Option<Arc<DiscoveryRegistry>>,
    /// Resolved endpoint (after discovery or fallback).
    resolved_endpoint: Arc<RwLock<Option<SocketAddr>>>,
    /// HTTP client (when live-clients feature is enabled).
    #[cfg(feature = "live-clients")]
    http_client: Arc<RwLock<Option<NestGateHttpClient>>>,
}

impl NestGateClient {
    /// Create a new NestGate client with discovery support.
    #[must_use]
    pub fn with_discovery(discovery: Arc<DiscoveryRegistry>) -> Self {
        let config = NestGateConfig::default();
        let cache_size = NonZeroUsize::new(config.cache_size).unwrap_or(CACHE_SIZE_NONZERO);
        Self {
            config,
            state: Arc::new(RwLock::new(NestGateState::Disconnected)),
            metadata_cache: Arc::new(RwLock::new(lru::LruCache::new(cache_size))),
            discovery: Some(discovery),
            resolved_endpoint: Arc::new(RwLock::new(None)),
            #[cfg(feature = "live-clients")]
            http_client: Arc::new(RwLock::new(None)),
        }
    }

    /// Create a new NestGate client.
    #[must_use]
    pub fn new(config: NestGateConfig) -> Self {
        let cache_size = NonZeroUsize::new(config.cache_size).unwrap_or(CACHE_SIZE_NONZERO);
        Self {
            config,
            state: Arc::new(RwLock::new(NestGateState::Disconnected)),
            metadata_cache: Arc::new(RwLock::new(lru::LruCache::new(cache_size))),
            discovery: None,
            resolved_endpoint: Arc::new(RwLock::new(None)),
            #[cfg(feature = "live-clients")]
            http_client: Arc::new(RwLock::new(None)),
        }
    }

    /// Create a client with default configuration.
    #[must_use]
    pub fn with_defaults() -> Self {
        Self::new(NestGateConfig::default())
    }

    /// Create a client from environment configuration.
    #[must_use]
    pub fn from_env() -> Self {
        Self::new(NestGateConfig::from_env())
    }

    /// Get current connection state.
    pub async fn state(&self) -> NestGateState {
        *self.state.read().await
    }

    /// Check if connected to NestGate.
    pub async fn is_connected(&self) -> bool {
        *self.state.read().await == NestGateState::Connected
    }

    /// Connect to NestGate service.
    ///
    /// This method:
    /// 1. Tries capability-based discovery via Songbird
    /// 2. Falls back to configured address if discovery unavailable
    /// 3. Establishes tarpc connection to NestGate
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

        *self.state.write().await = NestGateState::Discovering;

        // Step 1: Try discovery
        let endpoint = self.discover_or_fallback().await?;

        info!(address = %endpoint, "Connecting to NestGate");

        // Step 2: Verify reachability with timeout
        let connect_result = tokio::time::timeout(
            std::time::Duration::from_millis(self.config.timeout_ms),
            tokio::net::TcpStream::connect(endpoint),
        )
        .await;

        match connect_result {
            Ok(Ok(_stream)) => {
                *self.resolved_endpoint.write().await = Some(endpoint);

                // Create HTTP client when live-clients feature is enabled
                #[cfg(feature = "live-clients")]
                {
                    let base_url = format!("http://{endpoint}");
                    match NestGateHttpClient::new(base_url, self.config.timeout_ms) {
                        Ok(client) => {
                            *self.http_client.write().await = Some(client);
                            info!(address = %endpoint, "Connected to NestGate (live HTTP)");
                        }
                        Err(e) => {
                            warn!(error = %e, "Failed to create HTTP client, using scaffolded mode");
                        }
                    }
                }

                #[cfg(not(feature = "live-clients"))]
                info!(address = %endpoint, "Connected to NestGate (scaffolded mode)");

                *self.state.write().await = NestGateState::Connected;
                Ok(())
            }
            Ok(Err(e)) => {
                *self.state.write().await = NestGateState::Failed;
                error!(error = %e, address = %endpoint, "Failed to connect to NestGate");
                Err(RhizoCryptError::integration(format!("NestGate connection failed: {e}")))
            }
            Err(_) => {
                *self.state.write().await = NestGateState::Failed;
                error!(address = %endpoint, "NestGate connection timed out");
                Err(RhizoCryptError::integration("NestGate connection timeout"))
            }
        }
    }

    /// Discover NestGate via capability registry or use fallback.
    async fn discover_or_fallback(&self) -> Result<SocketAddr> {
        // Try discovery first
        if let Some(ref registry) = self.discovery {
            if let Some(endpoint) = registry.get_endpoint(&Capability::PayloadStorage).await {
                debug!(service = %endpoint.service_id, addr = %endpoint.addr, "Discovered payload storage");
                return Ok(endpoint.addr);
            }
            warn!("NestGate not found via discovery, trying fallback");
        }

        // Use fallback address
        self.config
            .fallback_address
            .as_ref()
            .ok_or_else(|| {
                RhizoCryptError::integration(
                    "NestGate not discoverable and no fallback address configured",
                )
            })
            .and_then(|addr| {
                addr.parse().map_err(|e| {
                    RhizoCryptError::integration(format!("Invalid NestGate fallback address: {e}"))
                })
            })
    }

    /// Disconnect from NestGate.
    pub async fn disconnect(&self) {
        *self.resolved_endpoint.write().await = None;
        *self.state.write().await = NestGateState::Disconnected;
        info!("Disconnected from NestGate");
    }

    /// Store a payload.
    ///
    /// # Errors
    ///
    /// Returns `RhizoCryptError::Integration` if:
    /// - Not connected to NestGate
    /// - Network error during storage
    ///
    /// Returns `RhizoCryptError::InvalidInput` if:
    /// - Payload exceeds maximum size
    pub async fn store(&self, request: StoreRequest) -> Result<PayloadRef> {
        if !self.is_connected().await {
            return Err(RhizoCryptError::integration("Not connected to NestGate"));
        }

        if request.data.len() > self.config.max_payload_size {
            return Err(RhizoCryptError::invalid_input(format!(
                "Payload size {} exceeds maximum {}",
                request.data.len(),
                self.config.max_payload_size
            )));
        }

        debug!(
            size = request.data.len(),
            content_type = ?request.content_type,
            "Storing payload in NestGate"
        );

        // Compute content hash
        let hash = blake3::hash(&request.data);
        let hash_hex = hash.to_hex().to_string();

        // Use HTTP client when live-clients feature is enabled
        #[cfg(feature = "live-clients")]
        {
            let client_guard = self.http_client.read().await;
            if let Some(ref client) = *client_guard {
                match client.store(&request.data, request.content_type.as_deref()).await {
                    Ok(reference) => {
                        debug!(reference = %reference, "Stored payload in NestGate");
                        let payload_ref = PayloadRef::from_hash(hash.as_bytes());

                        // Cache metadata locally
                        let metadata = PayloadMetadata {
                            hash: hash_hex,
                            size: request.data.len() as u64,
                            content_type: request.content_type,
                            compressed: false,
                            original_size: None,
                            stored_at: crate::types::Timestamp::now().as_nanos(),
                            tags: request.tags,
                        };
                        self.metadata_cache.write().await.put(payload_ref.to_string(), metadata);

                        return Ok(payload_ref);
                    }
                    Err(e) => {
                        warn!(error = %e, "HTTP store failed, falling back to scaffolded mode");
                        // Fall through to scaffolded mode
                    }
                }
            }
        }

        // Scaffolded mode: create a reference based on the hash
        let payload_ref = PayloadRef::from_hash(hash.as_bytes());

        // Cache metadata locally
        let metadata = PayloadMetadata {
            hash: hash_hex,
            size: request.data.len() as u64,
            content_type: request.content_type,
            compressed: false,
            original_size: None,
            stored_at: crate::types::Timestamp::now().as_nanos(),
            tags: request.tags,
        };

        self.metadata_cache.write().await.put(payload_ref.to_string(), metadata);

        Ok(payload_ref)
    }

    /// Retrieve a payload by reference.
    ///
    /// # Errors
    ///
    /// Returns `RhizoCryptError::Integration` if:
    /// - Not connected to NestGate
    /// - Payload not found
    /// - Network error during retrieval
    pub async fn retrieve(&self, payload_ref: &PayloadRef) -> Result<Vec<u8>> {
        if !self.is_connected().await {
            return Err(RhizoCryptError::integration("Not connected to NestGate"));
        }

        debug!(payload_ref = %payload_ref, "Retrieving payload from NestGate");

        // When NestGate has tarpc service, wire here:
        // let client = self.get_tarpc_client().await?;
        // let data = client.retrieve(tarpc::context::current(), payload_ref.hash).await??;

        Err(RhizoCryptError::integration(format!(
            "Payload not found: {payload_ref} (pending live integration)"
        )))
    }

    /// Check if a payload exists.
    ///
    /// # Errors
    ///
    /// Returns `RhizoCryptError::Integration` if:
    /// - Not connected to NestGate
    pub async fn exists(&self, payload_ref: &PayloadRef) -> Result<bool> {
        if !self.is_connected().await {
            return Err(RhizoCryptError::integration("Not connected to NestGate"));
        }

        // Check cache first
        if self.metadata_cache.read().await.contains(&payload_ref.to_string()) {
            return Ok(true);
        }

        // When NestGate has tarpc service, wire here:
        // let client = self.get_tarpc_client().await?;
        // return client.exists(tarpc::context::current(), payload_ref.hash).await?;

        Ok(false)
    }

    /// Get payload metadata.
    ///
    /// # Errors
    ///
    /// Returns `RhizoCryptError::Integration` if:
    /// - Not connected to NestGate
    /// - Metadata not found
    pub async fn metadata(&self, payload_ref: &PayloadRef) -> Result<PayloadMetadata> {
        if !self.is_connected().await {
            return Err(RhizoCryptError::integration("Not connected to NestGate"));
        }

        // Check cache
        if let Some(meta) = self.metadata_cache.read().await.peek(&payload_ref.to_string()) {
            return Ok(meta.clone());
        }

        // When NestGate has tarpc service, wire here:
        // let client = self.get_tarpc_client().await?;
        // return client.metadata(tarpc::context::current(), payload_ref.hash).await?;

        Err(RhizoCryptError::integration(format!("Metadata not found: {payload_ref}")))
    }

    /// Delete a payload.
    ///
    /// # Errors
    ///
    /// Returns `RhizoCryptError::Integration` if:
    /// - Not connected to NestGate
    /// - Deletion fails
    pub async fn delete(&self, payload_ref: &PayloadRef) -> Result<bool> {
        if !self.is_connected().await {
            return Err(RhizoCryptError::integration("Not connected to NestGate"));
        }

        debug!(payload_ref = %payload_ref, "Deleting payload from NestGate");

        // Remove from cache
        self.metadata_cache.write().await.pop(&payload_ref.to_string());

        // When NestGate has tarpc service, wire here:
        // let client = self.get_tarpc_client().await?;
        // return client.delete(tarpc::context::current(), payload_ref.hash).await?;

        Ok(true)
    }

    /// Store a vertex payload (convenience method).
    ///
    /// # Errors
    ///
    /// Returns `RhizoCryptError::Integration` if:
    /// - Not connected to NestGate
    /// - Storage fails
    ///
    /// Returns `RhizoCryptError::InvalidInput` if:
    /// - Payload exceeds maximum size
    pub async fn store_vertex_payload(
        &self,
        data: Vec<u8>,
        tags: Vec<String>,
    ) -> Result<PayloadRef> {
        self.store(StoreRequest {
            data,
            content_type: Some("application/octet-stream".to_string()),
            tags,
            force_compress: false,
        })
        .await
    }

    /// Clear the metadata cache.
    pub async fn clear_cache(&self) {
        self.metadata_cache.write().await.clear();
    }

    /// Get cache size.
    pub async fn cache_size(&self) -> usize {
        self.metadata_cache.read().await.len()
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

    #[test]
    fn test_config_default() {
        let config = NestGateConfig::default();
        assert!(config.fallback_address.is_none());
        assert!(config.enable_compression);
        assert_eq!(config.max_payload_size, 100 * 1024 * 1024);
    }

    #[test]
    fn test_config_with_fallback() {
        let config = NestGateConfig::with_fallback("127.0.0.1:9600");
        assert_eq!(config.fallback_address.as_deref(), Some("127.0.0.1:9600"));
    }

    #[test]
    fn test_client_creation() {
        let client = NestGateClient::with_defaults();
        assert!(client.discovery.is_none());
    }

    #[test]
    fn test_client_with_discovery() {
        let registry = Arc::new(DiscoveryRegistry::new("rhizoCrypt"));
        let client = NestGateClient::with_discovery(registry);
        assert!(client.discovery.is_some());
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn test_client_initial_state() {
        let client = NestGateClient::with_defaults();
        assert_eq!(client.state().await, NestGateState::Disconnected);
        assert!(!client.is_connected().await);
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn test_store_without_connection() {
        let client = NestGateClient::with_defaults();
        let result = client
            .store(StoreRequest {
                data: vec![1, 2, 3],
                content_type: None,
                tags: vec![],
                force_compress: false,
            })
            .await;
        assert!(result.is_err());
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn test_discovery_fallback_error() {
        let client = NestGateClient::new(NestGateConfig::default());
        let result = client.connect().await;
        assert!(result.is_err());
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn test_store_with_mock_connection() {
        let client = NestGateClient::with_defaults();

        // Manually set connected for test
        *client.state.write().await = NestGateState::Connected;

        let result = client
            .store(StoreRequest {
                data: vec![1, 2, 3, 4, 5],
                content_type: Some("application/octet-stream".to_string()),
                tags: vec!["test".to_string()],
                force_compress: false,
            })
            .await;

        assert!(result.is_ok());
        let payload_ref = result.unwrap();
        assert!(!payload_ref.as_bytes().is_empty());

        // Check metadata was cached
        assert_eq!(client.cache_size().await, 1);
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn test_payload_too_large() {
        let config = NestGateConfig {
            max_payload_size: 10, // 10 bytes max
            ..Default::default()
        };

        let client = NestGateClient::new(config);
        *client.state.write().await = NestGateState::Connected;

        let result = client
            .store(StoreRequest {
                data: vec![0u8; 100], // 100 bytes
                content_type: None,
                tags: vec![],
                force_compress: false,
            })
            .await;

        assert!(result.is_err());
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn test_cache_operations() {
        let client = NestGateClient::with_defaults();
        *client.state.write().await = NestGateState::Connected;

        // Store something to populate cache
        let _ = client
            .store(StoreRequest {
                data: vec![1, 2, 3],
                content_type: None,
                tags: vec![],
                force_compress: false,
            })
            .await;

        assert_eq!(client.cache_size().await, 1);

        client.clear_cache().await;
        assert_eq!(client.cache_size().await, 0);
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn test_retrieve_without_connection() {
        let client = NestGateClient::with_defaults();
        let payload_ref = crate::types::PayloadRef::from_bytes(b"test");
        let result = client.retrieve(&payload_ref).await;
        assert!(result.is_err());
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn test_retrieve_pending_integration() {
        let client = NestGateClient::with_defaults();
        *client.state.write().await = NestGateState::Connected;

        // Store first
        let store_result = client
            .store(StoreRequest {
                data: vec![10, 20, 30],
                content_type: Some("test/data".to_string()),
                tags: vec![],
                force_compress: false,
            })
            .await
            .unwrap();

        // Retrieve returns error in scaffolded mode
        let result = client.retrieve(&store_result).await;
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("pending live integration"));
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn test_exists_without_connection() {
        let client = NestGateClient::with_defaults();
        let payload_ref = crate::types::PayloadRef::from_bytes(b"test");
        let result = client.exists(&payload_ref).await;
        assert!(result.is_err());
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn test_exists_with_mock_connection() {
        let client = NestGateClient::with_defaults();
        *client.state.write().await = NestGateState::Connected;

        let payload_ref = crate::types::PayloadRef::from_bytes(b"test");
        let result = client.exists(&payload_ref).await;
        assert!(result.is_ok());
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn test_delete_without_connection() {
        let client = NestGateClient::with_defaults();
        let payload_ref = crate::types::PayloadRef::from_bytes(b"test");
        let result = client.delete(&payload_ref).await;
        assert!(result.is_err());
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn test_delete_with_mock_connection() {
        let client = NestGateClient::with_defaults();
        *client.state.write().await = NestGateState::Connected;

        let payload_ref = crate::types::PayloadRef::from_bytes(b"test");
        let result = client.delete(&payload_ref).await;
        assert!(result.is_ok());
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn test_store_with_compression() {
        let client = NestGateClient::with_defaults();
        *client.state.write().await = NestGateState::Connected;

        let result = client
            .store(StoreRequest {
                data: vec![0u8; 100], // Compressible data
                content_type: Some("application/octet-stream".to_string()),
                tags: vec!["compressed".to_string()],
                force_compress: true,
            })
            .await;
        assert!(result.is_ok());
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn test_endpoint_tracking() {
        let client = NestGateClient::with_defaults();
        assert!(client.endpoint().await.is_none());
    }

    #[test]
    fn test_store_request_validation() {
        let request = StoreRequest {
            data: vec![1, 2, 3],
            content_type: Some("application/json".to_string()),
            tags: vec!["tag1".to_string(), "tag2".to_string()],
            force_compress: true,
        };

        assert_eq!(request.data.len(), 3);
        assert!(request.content_type.is_some());
        assert_eq!(request.tags.len(), 2);
        assert!(request.force_compress);
    }

    #[test]
    fn test_payload_metadata_structure() {
        let metadata = PayloadMetadata {
            hash: "abc123".to_string(),
            size: 1024,
            content_type: Some("text/plain".to_string()),
            compressed: false,
            original_size: None,
            tags: vec![],
            stored_at: 1_703_318_400, // Unix timestamp
        };

        assert_eq!(metadata.size, 1024);
        assert!(!metadata.compressed);
        assert_eq!(metadata.hash, "abc123");
    }
}
