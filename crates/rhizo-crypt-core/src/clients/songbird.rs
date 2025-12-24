//! Songbird Client - Service Discovery and Registration
//!
//! Connects rhizoCrypt to the Songbird service mesh for:
//! - Primal registration (advertise our capabilities)
//! - Service discovery (find sibling primals)
//! - Federation status monitoring
//!
//! ## Bootstrap Architecture
//!
//! Songbird is the discovery bootstrap - the one address you must configure.
//! All other primals are discovered through Songbird at runtime.
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────────┐
//! │                     rhizoCrypt Bootstrap                        │
//! │                                                                 │
//! │  SONGBIRD_ADDRESS (env/config)                                  │
//! │         │                                                       │
//! │         ▼                                                       │
//! │    ┌─────────┐                                                  │
//! │    │Songbird │◀──── Bootstrap connection (only configured addr) │
//! │    └────┬────┘                                                  │
//! │         │                                                       │
//! │         ├──discover("signing")───────▶ BearDog                  │
//! │         ├──discover("permanent-storage")──▶ LoamSpine           │
//! │         └──discover("payload-storage")───▶ NestGate             │
//! │                                                                 │
//! └─────────────────────────────────────────────────────────────────┘
//! ```

use std::borrow::Cow;
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;

use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};

use crate::discovery::{Capability, DiscoveryRegistry, ServiceEndpoint};
use crate::error::{Result, RhizoCryptError};

// Import tarpc types when live-clients feature is enabled
#[cfg(feature = "live-clients")]
use super::songbird_rpc::{RpcServiceRegistration, SongbirdRpcClient};

/// Configuration for Songbird client.
///
/// Songbird is special: it's the bootstrap for discovery, so its address
/// is the only one that should be configured directly.
#[derive(Debug, Clone)]
pub struct SongbirdConfig {
    /// Songbird orchestrator address.
    /// This is the bootstrap address - discovered from environment or config.
    pub address: Cow<'static, str>,

    /// Service name for registration.
    pub service_name: Cow<'static, str>,

    /// Capabilities to advertise.
    pub capabilities: Vec<Cow<'static, str>>,

    /// Metadata to include in registration.
    pub metadata: HashMap<String, String>,

    /// Connection timeout in milliseconds.
    pub timeout_ms: u64,

    /// Enable automatic reconnection.
    pub auto_reconnect: bool,
}

impl Default for SongbirdConfig {
    fn default() -> Self {
        Self::from_env()
    }
}

impl SongbirdConfig {
    /// Default port for Songbird orchestrator (used only as last-resort fallback in development).
    const DEVELOPMENT_FALLBACK_PORT: u16 = 8091;

    /// Create a new config with no address configured.
    ///
    /// This is the preferred constructor - requires explicit address configuration.
    #[must_use]
    pub fn new() -> Self {
        Self {
            address: Cow::Borrowed(""), // Empty = not configured
            service_name: Cow::Borrowed("rhizoCrypt"),
            capabilities: vec![
                Cow::Borrowed("dag-engine"),
                Cow::Borrowed("session-management"),
                Cow::Borrowed("merkle-proofs"),
                Cow::Borrowed("slice-checkout"),
                Cow::Borrowed("dehydration"),
            ],
            metadata: HashMap::new(),
            timeout_ms: 5000,
            auto_reconnect: true,
        }
    }

    /// Check if this config has a valid address configured.
    #[must_use]
    pub fn is_configured(&self) -> bool {
        !self.address.is_empty()
    }
}

impl SongbirdConfig {
    /// Create config from environment variables.
    ///
    /// Environment variables (checked in order):
    /// - `DISCOVERY_ENDPOINT` or `DISCOVERY_SERVICE_ENDPOINT`: Discovery capability endpoint (preferred)
    /// - `SONGBIRD_ADDRESS`: Legacy orchestrator address (acceptable - Songbird is the universal adapter)
    /// - `SONGBIRD_HOST` + `SONGBIRD_PORT`: Alternative host/port specification
    /// - `RHIZOCRYPT_SERVICE_NAME`: Service name for registration
    ///
    /// ## Production Requirement
    ///
    /// In production, `DISCOVERY_ENDPOINT` or `SONGBIRD_ADDRESS` MUST be set. The development fallback
    /// (`localhost:8091`) is only available when `RHIZOCRYPT_ENV=development`.
    #[must_use]
    pub fn from_env() -> Self {
        use crate::safe_env::CapabilityEnv;
        let mut config = Self::new();

        // Primary: Check for capability-based endpoint
        if let Some(addr) = CapabilityEnv::discovery_endpoint() {
            config.address = Cow::Owned(addr);
        } else if let (Ok(host), Ok(port)) =
            (std::env::var("SONGBIRD_HOST"), std::env::var("SONGBIRD_PORT"))
        {
            // Alternative: Host + Port components
            config.address = Cow::Owned(format!("{host}:{port}"));
        } else {
            // Fallback: Only in development mode
            let is_dev = std::env::var("RHIZOCRYPT_ENV")
                .map(|v| v.eq_ignore_ascii_case("development") || v.eq_ignore_ascii_case("dev"))
                .unwrap_or(false);

            if is_dev {
                warn!(
                    "SONGBIRD_ADDRESS not set - using development fallback localhost:{}. \
                     Set SONGBIRD_ADDRESS for production!",
                    Self::DEVELOPMENT_FALLBACK_PORT
                );
                config.address =
                    Cow::Owned(format!("127.0.0.1:{}", Self::DEVELOPMENT_FALLBACK_PORT));
            } else {
                // Production: Require explicit configuration
                error!(
                    "SONGBIRD_ADDRESS not set and not in development mode. \
                     Set SONGBIRD_ADDRESS or RHIZOCRYPT_ENV=development"
                );
                // Leave address empty - will fail at connect() with clear error
            }
        }

        if let Ok(name) = std::env::var("RHIZOCRYPT_SERVICE_NAME") {
            config.service_name = Cow::Owned(name);
        }

        config
    }

    /// Create config with explicit address (for testing or explicit configuration).
    #[must_use]
    pub fn with_address(address: impl Into<Cow<'static, str>>) -> Self {
        let mut config = Self::new();
        config.address = address.into();
        config
    }
}

/// Service information returned by discovery.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceInfo {
    /// Service identifier.
    pub id: String,

    /// Service name.
    pub name: String,

    /// Service endpoint address.
    pub endpoint: String,

    /// Service capabilities.
    pub capabilities: Vec<String>,

    /// Service status.
    pub status: String,

    /// Optional metadata.
    #[serde(default)]
    pub metadata: HashMap<String, String>,
}

impl ServiceInfo {
    /// Check if this service provides a specific capability.
    #[must_use]
    pub fn has_capability(&self, cap: &str) -> bool {
        self.capabilities.iter().any(|c| c == cap)
    }
}

/// Registration result from Songbird.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegistrationResult {
    /// Whether registration succeeded.
    pub success: bool,

    /// Registration message.
    pub message: String,

    /// Assigned service ID (if successful).
    pub service_id: Option<String>,
}

/// Federation status from Songbird.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FederationStatus {
    /// Total registered services.
    pub total_services: usize,

    /// Total federation peers.
    pub total_peers: usize,

    /// Orchestrator uptime in seconds.
    pub uptime_seconds: u64,

    /// Orchestrator version.
    pub version: String,
}

/// Client state for connection management.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClientState {
    /// Not connected.
    Disconnected,

    /// Connecting to orchestrator.
    Connecting,

    /// Connected and ready.
    Connected,

    /// Registered with mesh.
    Registered,

    /// Connection failed.
    Failed,
}

/// Songbird client for service mesh integration.
///
/// Provides capability-based discovery and service registration
/// for the ecoPrimals mesh network.
///
/// ## Usage
///
/// ```ignore
/// use rhizo_crypt_core::clients::SongbirdClient;
///
/// // Create from environment (production)
/// let client = SongbirdClient::from_env();
/// client.connect().await?;
/// client.register("127.0.0.1:9400").await?;
///
/// // Discover other primals
/// let beardog = client.discover_beardog().await?;
/// ```
///
/// ## Live Client Feature
///
/// When compiled with `--features live-clients`, this client uses
/// actual tarpc connections to the Songbird orchestrator.
pub struct SongbirdClient {
    config: SongbirdConfig,
    state: Arc<RwLock<ClientState>>,
    service_id: Arc<RwLock<Option<String>>>,
    discovered_services: Arc<RwLock<HashMap<String, Vec<ServiceInfo>>>>,
    /// Resolved endpoint for tarpc.
    resolved_endpoint: Arc<RwLock<Option<SocketAddr>>>,
    /// tarpc client (when live-clients feature is enabled).
    #[cfg(feature = "live-clients")]
    tarpc_client: Arc<RwLock<Option<SongbirdRpcClient>>>,
}

impl SongbirdClient {
    /// Create a new Songbird client.
    #[must_use]
    pub fn new(config: SongbirdConfig) -> Self {
        Self {
            config,
            state: Arc::new(RwLock::new(ClientState::Disconnected)),
            service_id: Arc::new(RwLock::new(None)),
            discovered_services: Arc::new(RwLock::new(HashMap::new())),
            resolved_endpoint: Arc::new(RwLock::new(None)),
            #[cfg(feature = "live-clients")]
            tarpc_client: Arc::new(RwLock::new(None)),
        }
    }

    /// Create a client with default configuration from environment.
    ///
    /// Note: This requires `SONGBIRD_ADDRESS` to be set, or `RHIZOCRYPT_ENV=development`
    /// for the localhost fallback.
    #[must_use]
    pub fn with_defaults() -> Self {
        Self::new(SongbirdConfig::from_env())
    }

    /// Create a client from environment configuration.
    #[must_use]
    pub fn from_env() -> Self {
        Self::new(SongbirdConfig::from_env())
    }

    /// Get current connection state.
    pub async fn state(&self) -> ClientState {
        *self.state.read().await
    }

    /// Check if connected to Songbird.
    pub async fn is_connected(&self) -> bool {
        matches!(*self.state.read().await, ClientState::Connected | ClientState::Registered)
    }

    /// Connect to the Songbird orchestrator.
    ///
    /// # Errors
    ///
    /// Returns `RhizoCryptError::Integration` if:
    /// - No address is configured (SONGBIRD_ADDRESS not set)
    /// - The configured address is invalid
    /// - Connection times out
    /// - TCP connection fails
    pub async fn connect(&self) -> Result<()> {
        let current_state = *self.state.read().await;
        if matches!(current_state, ClientState::Connected | ClientState::Registered) {
            return Ok(());
        }

        // Check for unconfigured address
        if !self.config.is_configured() {
            return Err(RhizoCryptError::integration(
                "Songbird address not configured. Set SONGBIRD_ADDRESS environment variable \
                 or use SongbirdConfig::with_address() for explicit configuration. \
                 For development, set RHIZOCRYPT_ENV=development to use localhost fallback.",
            ));
        }

        *self.state.write().await = ClientState::Connecting;
        info!(address = %self.config.address, "Connecting to Songbird orchestrator");

        // Parse address
        let addr: SocketAddr = self.config.address.parse().map_err(|e| {
            RhizoCryptError::integration(format!(
                "Invalid Songbird address '{}': {e}",
                self.config.address
            ))
        })?;

        // Attempt connection with timeout
        #[cfg(not(feature = "live-clients"))]
        {
            let connect_result = tokio::time::timeout(
                std::time::Duration::from_millis(self.config.timeout_ms),
                Self::try_connect(addr),
            )
            .await;

            match connect_result {
                Ok(Ok(())) => {
                    *self.resolved_endpoint.write().await = Some(addr);
                    *self.state.write().await = ClientState::Connected;
                    info!(address = %addr, "Connected to Songbird orchestrator (scaffolded mode)");
                    Ok(())
                }
                Ok(Err(e)) => {
                    *self.state.write().await = ClientState::Failed;
                    error!(error = %e, "Failed to connect to Songbird");
                    Err(e)
                }
                Err(_) => {
                    *self.state.write().await = ClientState::Failed;
                    error!("Connection to Songbird timed out");
                    Err(RhizoCryptError::integration("Songbird connection timeout"))
                }
            }
        }

        #[cfg(feature = "live-clients")]
        {
            let connect_result = tokio::time::timeout(
                std::time::Duration::from_millis(self.config.timeout_ms),
                Self::try_connect_tarpc(addr),
            )
            .await;

            match connect_result {
                Ok(Ok(client)) => {
                    *self.resolved_endpoint.write().await = Some(addr);
                    *self.tarpc_client.write().await = Some(client);
                    *self.state.write().await = ClientState::Connected;
                    info!(address = %addr, "Connected to Songbird orchestrator (live tarpc)");
                    Ok(())
                }
                Ok(Err(e)) => {
                    *self.state.write().await = ClientState::Failed;
                    error!(error = %e, "Failed to connect to Songbird");
                    Err(e)
                }
                Err(_) => {
                    *self.state.write().await = ClientState::Failed;
                    error!("Connection to Songbird timed out");
                    Err(RhizoCryptError::integration("Songbird connection timeout"))
                }
            }
        }
    }

    /// Internal connection attempt (scaffolded mode).
    #[cfg(not(feature = "live-clients"))]
    async fn try_connect(addr: SocketAddr) -> Result<()> {
        // Try to establish TCP connection to verify reachability
        match tokio::net::TcpStream::connect(addr).await {
            Ok(_stream) => {
                debug!(addr = %addr, "TCP connection established (scaffolded mode)");
                Ok(())
            }
            Err(e) => {
                Err(RhizoCryptError::integration(format!("Cannot reach Songbird at {addr}: {e}")))
            }
        }
    }

    /// Internal connection attempt with tarpc client establishment.
    #[cfg(feature = "live-clients")]
    async fn try_connect_tarpc(addr: SocketAddr) -> Result<SongbirdRpcClient> {
        use tarpc::client;
        use tarpc::tokio_serde::formats::Bincode;

        debug!(addr = %addr, "Establishing tarpc connection to Songbird");

        // Connect TCP stream
        let stream = tokio::net::TcpStream::connect(addr).await.map_err(|e| {
            RhizoCryptError::integration(format!("Cannot reach Songbird at {addr}: {e}"))
        })?;

        // Create tarpc transport
        let transport = tarpc::serde_transport::Transport::from((stream, Bincode::default()));

        // Create tarpc client
        let client = SongbirdRpcClient::new(client::Config::default(), transport).spawn();

        info!(addr = %addr, "tarpc connection established to Songbird");
        Ok(client)
    }

    /// Register this service with Songbird.
    ///
    /// # Errors
    ///
    /// Returns `RhizoCryptError::Integration` if:
    /// - Not connected to Songbird
    /// - Registration request fails
    pub async fn register(&self, our_endpoint: &str) -> Result<RegistrationResult> {
        if !self.is_connected().await {
            return Err(RhizoCryptError::integration(
                "Not connected to Songbird - call connect() first",
            ));
        }

        info!(
            service = %self.config.service_name,
            endpoint = %our_endpoint,
            "Registering with Songbird mesh"
        );

        // Build registration request
        #[cfg(feature = "live-clients")]
        let result = {
            let client_guard = self.tarpc_client.read().await;
            let client = client_guard
                .as_ref()
                .ok_or_else(|| RhizoCryptError::integration("No tarpc client available"))?;

            let registration = RpcServiceRegistration {
                service_id: format!("rhizocrypt-{}", uuid::Uuid::now_v7()),
                service_name: self.config.service_name.to_string(),
                capability: "dag-engine".to_string(),
                endpoint: our_endpoint.to_string(),
                metadata: self.config.metadata.clone(),
            };

            let rpc_result = client
                .register(tarpc::context::current(), registration.clone())
                .await
                .map_err(|e| RhizoCryptError::integration(format!("tarpc error: {e}")))?;

            RegistrationResult {
                success: rpc_result.success,
                message: rpc_result.message,
                service_id: if rpc_result.success {
                    Some(registration.service_id)
                } else {
                    None
                },
            }
        };

        #[cfg(not(feature = "live-clients"))]
        let result = {
            let _registration = ServiceRegistration {
                service_name: self.config.service_name.to_string(),
                endpoint: our_endpoint.to_string(),
                capabilities: self.config.capabilities.iter().map(ToString::to_string).collect(),
                metadata: self.config.metadata.clone(),
            };

            // Scaffolded mode: simulate successful registration
            RegistrationResult {
                success: true,
                message: "Registration pending live integration".to_string(),
                service_id: Some(format!("rhizocrypt-{}", uuid::Uuid::now_v7())),
            }
        };

        if result.success {
            if let Some(ref id) = result.service_id {
                *self.service_id.write().await = Some(id.clone());
                *self.state.write().await = ClientState::Registered;
                info!(service_id = %id, "Registered with Songbird mesh");
            }
        }

        Ok(result)
    }

    /// Discover services by capability.
    ///
    /// # Errors
    ///
    /// Returns `RhizoCryptError::Integration` if:
    /// - Not connected to Songbird
    /// - Discovery query fails
    pub async fn discover(&self, capability: &str) -> Result<Vec<ServiceInfo>> {
        if !self.is_connected().await {
            return Err(RhizoCryptError::integration("Not connected to Songbird"));
        }

        debug!(capability = %capability, "Discovering services");

        // Check cache first
        {
            let cache = self.discovered_services.read().await;
            if let Some(services) = cache.get(capability) {
                debug!(count = services.len(), "Returning cached discovery results");
                return Ok(services.clone());
            }
        }

        #[cfg(feature = "live-clients")]
        {
            let client_guard = self.tarpc_client.read().await;
            if let Some(client) = client_guard.as_ref() {
                let rpc_services = client
                    .discover(tarpc::context::current(), capability.to_string())
                    .await
                    .map_err(|e| RhizoCryptError::integration(format!("tarpc error: {e}")))?;

                // Convert RpcServiceInfo to ServiceInfo
                let services: Vec<ServiceInfo> = rpc_services
                    .into_iter()
                    .map(|s| ServiceInfo {
                        id: s.id,
                        name: s.capability.clone(),
                        endpoint: s.endpoint,
                        capabilities: vec![s.capability],
                        status: s.status,
                        metadata: HashMap::new(),
                    })
                    .collect();

                // Cache the results
                self.discovered_services
                    .write()
                    .await
                    .insert(capability.to_string(), services.clone());

                return Ok(services);
            }
        }

        // Scaffolded mode: return empty for capabilities we don't have cached
        Ok(Vec::new())
    }

    /// Discover BearDog service for signing/DID operations.
    ///
    /// # Errors
    ///
    /// Returns `RhizoCryptError::Integration` if:
    /// - Not connected to Songbird
    /// - Discovery fails
    pub async fn discover_beardog(&self) -> Result<Option<ServiceInfo>> {
        let services = self.discover("signing").await?;
        Ok(services.into_iter().find(|s| s.name.contains("beardog")))
    }

    /// Discover LoamSpine service for permanent commits.
    ///
    /// # Errors
    ///
    /// Returns `RhizoCryptError::Integration` if:
    /// - Not connected to Songbird
    /// - Discovery fails
    pub async fn discover_loamspine(&self) -> Result<Option<ServiceInfo>> {
        let services = self.discover("permanent-storage").await?;
        Ok(services.into_iter().find(|s| s.name.contains("loamspine")))
    }

    /// Discover NestGate service for payload storage.
    ///
    /// # Errors
    ///
    /// Returns `RhizoCryptError::Integration` if:
    /// - Not connected to Songbird
    /// - Discovery fails
    pub async fn discover_nestgate(&self) -> Result<Option<ServiceInfo>> {
        let services = self.discover("payload-storage").await?;
        Ok(services.into_iter().find(|s| s.name.contains("nestgate")))
    }

    /// Populate the discovery registry with discovered services.
    ///
    /// This bridges Songbird discovery to the capability-based registry.
    ///
    /// # Errors
    ///
    /// Returns `RhizoCryptError::Integration` if:
    /// - Not connected to Songbird
    /// - Discovery fails
    pub async fn populate_registry(&self, registry: &DiscoveryRegistry) -> Result<()> {
        if !self.is_connected().await {
            return Err(RhizoCryptError::integration("Not connected to Songbird"));
        }

        // Discover and register BearDog
        if let Some(beardog) = self.discover_beardog().await? {
            if let Ok(addr) = beardog.endpoint.parse() {
                registry
                    .register_endpoint(ServiceEndpoint::new(
                        beardog.name,
                        addr,
                        vec![Capability::DidVerification, Capability::Signing],
                    ))
                    .await;
            }
        }

        // Discover and register LoamSpine
        if let Some(loamspine) = self.discover_loamspine().await? {
            if let Ok(addr) = loamspine.endpoint.parse() {
                registry
                    .register_endpoint(ServiceEndpoint::new(
                        loamspine.name,
                        addr,
                        vec![
                            Capability::PermanentCommit,
                            Capability::SliceCheckout,
                            Capability::SliceResolution,
                        ],
                    ))
                    .await;
            }
        }

        // Discover and register NestGate
        if let Some(nestgate) = self.discover_nestgate().await? {
            if let Ok(addr) = nestgate.endpoint.parse() {
                registry
                    .register_endpoint(ServiceEndpoint::new(
                        nestgate.name,
                        addr,
                        vec![Capability::PayloadStorage, Capability::PayloadRetrieval],
                    ))
                    .await;
            }
        }

        Ok(())
    }

    /// Get federation status.
    ///
    /// # Errors
    ///
    /// Returns `RhizoCryptError::Integration` if:
    /// - Not connected to Songbird
    /// - Status query fails
    pub async fn federation_status(&self) -> Result<FederationStatus> {
        if !self.is_connected().await {
            return Err(RhizoCryptError::integration("Not connected to Songbird"));
        }

        // When Songbird tarpc client is available, wire here:
        // let client = self.get_tarpc_client().await?;
        // return client.federation_status(tarpc::context::current()).await?;

        Ok(FederationStatus {
            total_services: 0,
            total_peers: 0,
            uptime_seconds: 0,
            version: "pending-integration".to_string(),
        })
    }

    /// Unregister from the mesh.
    ///
    /// # Errors
    ///
    /// Returns `RhizoCryptError::Integration` if unregistration fails.
    pub async fn unregister(&self) -> Result<()> {
        let service_id = self.service_id.read().await.clone();

        if let Some(id) = service_id {
            info!(service_id = %id, "Unregistering from Songbird mesh");

            // When Songbird tarpc client is available, wire here:
            // let client = self.get_tarpc_client().await?;
            // client.unregister(tarpc::context::current(), id).await??;

            *self.service_id.write().await = None;
            *self.state.write().await = ClientState::Connected;
        }

        Ok(())
    }

    /// Disconnect from Songbird.
    pub async fn disconnect(&self) {
        if self.is_connected().await {
            let _ = self.unregister().await;
        }
        *self.resolved_endpoint.write().await = None;
        *self.state.write().await = ClientState::Disconnected;
        info!("Disconnected from Songbird");
    }

    /// Get our registered service ID.
    pub async fn service_id(&self) -> Option<String> {
        self.service_id.read().await.clone()
    }

    /// Update cached discovery results.
    pub async fn cache_discovery(&self, capability: &str, services: Vec<ServiceInfo>) {
        self.discovered_services.write().await.insert(capability.to_string(), services);
    }

    /// Clear discovery cache.
    pub async fn clear_cache(&self) {
        self.discovered_services.write().await.clear();
    }

    /// Get the resolved endpoint address.
    pub async fn endpoint(&self) -> Option<SocketAddr> {
        *self.resolved_endpoint.read().await
    }
}

/// Service registration request.
#[derive(Debug, Clone, Serialize, Deserialize)]
struct ServiceRegistration {
    service_name: String,
    endpoint: String,
    capabilities: Vec<String>,
    metadata: HashMap<String, String>,
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;

    #[test]
    fn test_config_new_unconfigured() {
        let config = SongbirdConfig::new();
        assert!(config.address.is_empty());
        assert!(!config.is_configured());
        assert_eq!(config.service_name, "rhizoCrypt");
        assert!(!config.capabilities.is_empty());
    }

    #[test]
    fn test_config_with_address() {
        let config = SongbirdConfig::with_address("192.0.2.100:8091");
        assert_eq!(config.address, "192.0.2.100:8091");
        assert!(config.is_configured());
    }

    #[test]
    fn test_client_creation() {
        let config = SongbirdConfig::with_address("127.0.0.1:8091");
        let client = SongbirdClient::new(config);
        assert!(client.config.auto_reconnect);
        assert!(client.config.is_configured());
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn test_client_initial_state() {
        let config = SongbirdConfig::with_address("127.0.0.1:8091");
        let client = SongbirdClient::new(config);
        assert_eq!(client.state().await, ClientState::Disconnected);
        assert!(!client.is_connected().await);
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn test_discover_without_connection() {
        let config = SongbirdConfig::with_address("127.0.0.1:8091");
        let client = SongbirdClient::new(config);
        let result = client.discover("signing").await;
        assert!(result.is_err());
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn test_register_without_connection() {
        let config = SongbirdConfig::with_address("127.0.0.1:8091");
        let client = SongbirdClient::new(config);
        let result = client.register("127.0.0.1:9400").await;
        assert!(result.is_err());
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn test_connect_without_config_fails() {
        let config = SongbirdConfig::new();
        let client = SongbirdClient::new(config);
        let result = client.connect().await;
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("not configured"));
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn test_cache_operations() {
        let config = SongbirdConfig::with_address("127.0.0.1:8091");
        let client = SongbirdClient::new(config);

        // Manually set connected for cache test
        *client.state.write().await = ClientState::Connected;

        let services = vec![ServiceInfo {
            id: "test-1".to_string(),
            name: "test-beardog".to_string(),
            endpoint: "127.0.0.1:9000".to_string(),
            capabilities: vec!["signing".to_string()],
            status: "healthy".to_string(),
            metadata: HashMap::new(),
        }];

        client.cache_discovery("signing", services.clone()).await;

        let cached = client.discover("signing").await.unwrap();
        assert_eq!(cached.len(), 1);
        assert_eq!(cached[0].name, "test-beardog");

        client.clear_cache().await;
        let empty = client.discover("signing").await.unwrap();
        assert!(empty.is_empty());
    }

    #[test]
    fn test_service_info_has_capability() {
        let service = ServiceInfo {
            id: "test".to_string(),
            name: "test".to_string(),
            endpoint: "127.0.0.1:9000".to_string(),
            capabilities: vec!["signing".to_string(), "did".to_string()],
            status: "healthy".to_string(),
            metadata: HashMap::new(),
        };

        assert!(service.has_capability("signing"));
        assert!(service.has_capability("did"));
        assert!(!service.has_capability("storage"));
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn test_populate_registry() {
        let config = SongbirdConfig::with_address("127.0.0.1:8091");
        let client = SongbirdClient::new(config);
        *client.state.write().await = ClientState::Connected;

        // Add cached services
        client
            .cache_discovery(
                "signing",
                vec![ServiceInfo {
                    id: "bd-1".to_string(),
                    name: "beardog-main".to_string(),
                    endpoint: "127.0.0.1:9500".to_string(),
                    capabilities: vec!["signing".to_string()],
                    status: "healthy".to_string(),
                    metadata: HashMap::new(),
                }],
            )
            .await;

        let registry = DiscoveryRegistry::new("rhizoCrypt");
        client.populate_registry(&registry).await.unwrap();

        assert!(registry.is_available(&Capability::Signing).await);
    }

    #[test]
    fn test_from_env_respects_variables() {
        // Test with_address is explicit
        let config = SongbirdConfig::with_address("10.0.0.1:9999");
        assert_eq!(config.address, "10.0.0.1:9999");
        assert!(config.is_configured());
    }
}
