// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2024–2026 ecoPrimals Project

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

use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;

use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};

use crate::error::{Result, RhizoCryptError};

use super::super::songbird_types::{
    ClientState, FederationStatus, RegistrationResult, ServiceInfo,
};
use super::config::SongbirdConfig;

#[cfg(feature = "live-clients")]
use super::super::songbird_rpc::{RpcServiceRegistration, SongbirdRpcClient};

/// Songbird client for service mesh integration.
///
/// Provides capability-based discovery and service registration
/// for the ecoPrimals mesh network.
///
/// ## Usage
///
/// ```no_run
/// # use rhizo_crypt_core::clients::SongbirdClient;
/// # tokio::runtime::Runtime::new().unwrap().block_on(async {
/// // Create from environment (production)
/// let client = SongbirdClient::from_env();
/// client.connect().await?;
/// client.register("127.0.0.1:9400").await?;
///
/// // Start heartbeat to maintain registration (60s TTL)
/// let _handle = client.start_heartbeat().await?;
///
/// // Discover capabilities (not specific primals)
/// let _signer_info = client.discover_signing_provider().await?;
/// # Ok::<(), rhizo_crypt_core::error::RhizoCryptError>(())
/// # });
/// ```
///
/// ## Heartbeat Mechanism
///
/// Songbird registrations expire after 60 seconds. The heartbeat task
/// automatically refreshes the registration every 45 seconds to prevent expiry.
///
/// ## Live Client Feature
///
/// When compiled with `--features live-clients`, this client uses
/// actual tarpc connections to the Songbird orchestrator.
pub struct SongbirdClient {
    pub(crate) config: SongbirdConfig,
    pub(crate) state: Arc<RwLock<ClientState>>,
    pub(crate) service_id: Arc<RwLock<Option<String>>>,
    pub(crate) discovered_services: Arc<RwLock<HashMap<String, Vec<ServiceInfo>>>>,
    /// Resolved endpoint for tarpc.
    pub(crate) resolved_endpoint: Arc<RwLock<Option<SocketAddr>>>,
    /// tarpc client (when live-clients feature is enabled).
    #[cfg(feature = "live-clients")]
    pub(crate) tarpc_client: Arc<RwLock<Option<SongbirdRpcClient>>>,
    /// Our registered endpoint (for heartbeat refreshes).
    pub(crate) our_endpoint: Arc<RwLock<Option<String>>>,
    /// Heartbeat task handle (if running).
    pub(crate) heartbeat_handle: Arc<RwLock<Option<tokio::task::JoinHandle<()>>>>,
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
            our_endpoint: Arc::new(RwLock::new(None)),
            heartbeat_handle: Arc::new(RwLock::new(None)),
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
            tracing::debug!(
                service = %self.config.service_name,
                endpoint = %our_endpoint,
                "Scaffolded registration (live-clients feature disabled)"
            );
            RegistrationResult {
                success: true,
                message: "Registration pending live integration".to_string(),
                service_id: Some(format!("rhizocrypt-{}", uuid::Uuid::now_v7())),
            }
        };

        if result.success
            && let Some(ref id) = result.service_id
        {
            *self.service_id.write().await = Some(id.clone());
            *self.state.write().await = ClientState::Registered;
            *self.our_endpoint.write().await = Some(our_endpoint.to_string());
            info!(service_id = %id, "Registered with Songbird mesh");
        }

        Ok(result)
    }

    /// Start heartbeat task to maintain registration.
    ///
    /// Songbird registrations expire after 60 seconds. This task refreshes
    /// the registration every 45 seconds to prevent expiry.
    ///
    /// # Errors
    ///
    /// Returns `RhizoCryptError::Integration` if:
    /// - Not registered with Songbird
    /// - Heartbeat task already running
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use rhizo_crypt_core::clients::SongbirdClient;
    /// # tokio::runtime::Runtime::new().unwrap().block_on(async {
    /// let client = SongbirdClient::from_env();
    /// client.connect().await?;
    /// client.register("127.0.0.1:9400").await?;
    ///
    /// // Start heartbeat (refreshes every 45s)
    /// let _handle = client.start_heartbeat().await?;
    ///
    /// // Later, stop heartbeat
    /// client.stop_heartbeat().await;
    /// # Ok::<(), rhizo_crypt_core::error::RhizoCryptError>(())
    /// # });
    /// ```
    pub async fn start_heartbeat(&self) -> Result<()> {
        // Check if already running
        {
            let handle_guard = self.heartbeat_handle.read().await;
            if handle_guard.is_some() {
                return Err(RhizoCryptError::integration(
                    "Heartbeat already running - call stop_heartbeat() first",
                ));
            }
        }

        // Check if registered
        if *self.state.read().await != ClientState::Registered {
            return Err(RhizoCryptError::integration("Not registered - call register() first"));
        }

        // Clone self for the async task
        let client = Self {
            config: self.config.clone(),
            state: Arc::clone(&self.state),
            service_id: Arc::clone(&self.service_id),
            discovered_services: Arc::clone(&self.discovered_services),
            resolved_endpoint: Arc::clone(&self.resolved_endpoint),
            #[cfg(feature = "live-clients")]
            tarpc_client: Arc::clone(&self.tarpc_client),
            our_endpoint: Arc::clone(&self.our_endpoint),
            heartbeat_handle: Arc::clone(&self.heartbeat_handle),
        };

        // Spawn heartbeat task
        let handle = tokio::spawn(async move {
            info!("Heartbeat task started (45s interval)");
            loop {
                tokio::time::sleep(std::time::Duration::from_secs(45)).await;

                // Check if still registered
                if *client.state.read().await != ClientState::Registered {
                    warn!("No longer registered, stopping heartbeat");
                    break;
                }

                // Refresh registration
                if let Err(e) = client.refresh_registration().await {
                    error!(error = %e, "Failed to refresh registration");
                    // Continue trying - don't stop heartbeat on single failure
                }
            }
            info!("Heartbeat task stopped");
        });

        *self.heartbeat_handle.write().await = Some(handle);
        info!("Heartbeat started successfully");
        Ok(())
    }

    /// Stop the heartbeat task.
    ///
    /// This method is graceful - it's safe to call even if no heartbeat is running.
    pub async fn stop_heartbeat(&self) {
        let mut handle_guard = self.heartbeat_handle.write().await;
        if let Some(handle) = handle_guard.take() {
            handle.abort();
            info!("Heartbeat task stopped");
        }
    }

    /// Refresh the registration (called by heartbeat task).
    ///
    /// Re-registers with Songbird to extend the TTL.
    ///
    /// # Errors
    ///
    /// Returns `RhizoCryptError::Integration` if:
    /// - Not registered
    /// - No endpoint saved
    /// - Re-registration fails
    async fn refresh_registration(&self) -> Result<()> {
        let endpoint_guard = self.our_endpoint.read().await;
        let endpoint = endpoint_guard.as_ref().ok_or_else(|| {
            RhizoCryptError::integration("No endpoint saved - cannot refresh registration")
        })?;

        debug!(endpoint = %endpoint, "Refreshing Songbird registration");

        // Re-register (same as initial registration)
        let result = self.register(endpoint).await?;

        if result.success {
            debug!("Registration refreshed successfully");
            Ok(())
        } else {
            Err(RhizoCryptError::integration(format!(
                "Registration refresh failed: {}",
                result.message
            )))
        }
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

    /// Get the resolved endpoint address.
    pub async fn endpoint(&self) -> Option<SocketAddr> {
        *self.resolved_endpoint.read().await
    }
}

impl Clone for SongbirdClient {
    fn clone(&self) -> Self {
        Self {
            config: self.config.clone(),
            state: Arc::clone(&self.state),
            service_id: Arc::clone(&self.service_id),
            discovered_services: Arc::clone(&self.discovered_services),
            resolved_endpoint: Arc::clone(&self.resolved_endpoint),
            #[cfg(feature = "live-clients")]
            tarpc_client: Arc::clone(&self.tarpc_client),
            our_endpoint: Arc::clone(&self.our_endpoint),
            heartbeat_handle: Arc::clone(&self.heartbeat_handle),
        }
    }
}

// Tests are in tests.rs (as a sibling module with access to private fields)
#[cfg(test)]
#[path = "tests.rs"]
mod tests;
