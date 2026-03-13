// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2024–2026 ecoPrimals Project

//! Songbird connection management.

use std::net::SocketAddr;

use tracing::{debug, error, info};

use crate::error::{Result, RhizoCryptError};

use super::super::songbird_types::ClientState;
use super::client::SongbirdClient;

#[cfg(feature = "live-clients")]
use super::super::songbird_rpc::SongbirdRpcClient;

impl SongbirdClient {
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
}
