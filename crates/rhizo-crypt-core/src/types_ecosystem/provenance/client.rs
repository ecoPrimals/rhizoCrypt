// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2024–2026 ecoPrimals Project

//! Provenance provider notifier for push updates.
//!
//! Used to notify provenance provider when new provenance data is available.
//! When connected, sends JSON-RPC calls to any attribution provider that
//! implements the `ProvenanceQuery` capability.

use std::sync::Arc;

use crate::transport::TransportEndpoint;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

use crate::constants::{PROVENANCE_CONNECTION_TIMEOUT, PROVENANCE_RESPONSE_TIMEOUT};
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

    /// Connected endpoint (transport-agnostic).
    endpoint: Arc<RwLock<Option<TransportEndpoint>>>,
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
        if let Some(registry) = &self.registry
            && let Some(service) = registry.get_endpoint(&Capability::ProvenanceQuery).await
        {
            info!(endpoint = %service.endpoint, "Discovered provenance provider via registry");
            *self.endpoint.write().await = Some(service.endpoint.clone());
            *self.state.write().await = ClientState::Connected;
            return Ok(());
        }

        if let Some(ref addr) = self.config.push_address {
            let transport = serde_json::from_str::<TransportEndpoint>(addr).or_else(|_| {
                addr.parse::<std::net::SocketAddr>()
                    .map(|sa| TransportEndpoint::tcp(sa.ip().to_string(), sa.port()))
                    .map_err(|e| {
                        RhizoCryptError::integration(format!(
                            "Invalid provenance provider address '{addr}': {e}"
                        ))
                    })
            })?;

            debug!(endpoint = %transport, "Connecting to provenance provider");
            *self.endpoint.write().await = Some(transport);
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

        let Some(endpoint) = self.endpoint.read().await.clone() else {
            return Ok(());
        };

        debug!(%session_id, %endpoint, "Notifying provenance provider of session commit");

        let request = serde_json::json!({
            "jsonrpc": crate::constants::JSONRPC_VERSION,
            "method": crate::constants::PROVENANCE_RECORD_SESSION_METHOD,
            "params": {
                "session_id": session_id.to_string(),
                "source_primal": crate::constants::PRIMAL_NAME,
            },
            "id": 1
        });

        Self::log_notify_result(
            "session commit",
            &format!("{session_id}"),
            Self::send_jsonrpc(&endpoint, &request).await,
        );

        Ok(())
    }

    /// Notify provenance provider of a completed dehydration with full summary.
    ///
    /// Converts the internal [`DehydrationSummary`] to a
    /// [`DehydrationWireSummary`](crate::dehydration_wire::DehydrationWireSummary)
    /// for the `contribution.record_dehydration` JSON-RPC call.
    ///
    /// # Errors
    ///
    /// Returns error if notification fails.
    pub async fn notify_dehydration(&self, summary: &DehydrationSummary) -> Result<()> {
        if *self.state.read().await != ClientState::Connected {
            return Ok(());
        }

        let Some(endpoint) = self.endpoint.read().await.clone() else {
            return Ok(());
        };

        debug!(
            session_id = %summary.session_id,
            agents = summary.agents.len(),
            %endpoint,
            "Notifying provenance provider of dehydration"
        );

        let wire_summary: crate::dehydration_wire::DehydrationWireSummary = summary.into();

        let request = serde_json::json!({
            "jsonrpc": crate::constants::JSONRPC_VERSION,
            "method": crate::constants::PROVENANCE_RECORD_DEHYDRATION_METHOD,
            "params": wire_summary,
            "id": 1
        });

        Self::log_notify_result(
            "dehydration",
            &format!("{}", summary.session_id),
            Self::send_jsonrpc(&endpoint, &request).await,
        );

        Ok(())
    }

    /// Notify provenance provider of a new provenance chain.
    ///
    /// Sends a `contribution.record_provenance` JSON-RPC call with the chain's
    /// vertex references. Non-fatal on failure (graceful degradation per
    /// the Provenance Trio graceful degradation pattern).
    ///
    /// # Errors
    ///
    /// Returns error if notification fails.
    pub async fn notify_provenance(&self, chain: &ProvenanceChain) -> Result<()> {
        if *self.state.read().await != ClientState::Connected {
            return Ok(());
        }

        let Some(endpoint) = self.endpoint.read().await.clone() else {
            return Ok(());
        };

        debug!(
            vertices = chain.len(),
            %endpoint,
            "Notifying provenance provider of provenance update"
        );

        let request = serde_json::json!({
            "jsonrpc": crate::constants::JSONRPC_VERSION,
            "method": crate::constants::PROVENANCE_RECORD_PROVENANCE_METHOD,
            "params": {
                "source_primal": crate::constants::PRIMAL_NAME,
                "vertices": chain.vertices,
                "agent_count": chain.agents.len(),
            },
            "id": 1
        });

        Self::log_notify_result(
            "provenance chain",
            &format!("{} vertices", chain.len()),
            Self::send_jsonrpc(&endpoint, &request).await,
        );

        Ok(())
    }

    fn log_notify_result(
        kind: &str,
        context: &str,
        result: std::result::Result<String, crate::transport::JsonRpcTransportError>,
    ) {
        match result {
            Ok(response) => {
                info!(context, "Provenance provider notified of {kind}: {response}");
            }
            Err(e) => {
                warn!(context, error = %e, "Failed to notify provenance provider of {kind} (non-fatal)");
            }
        }
    }

    async fn send_jsonrpc(
        endpoint: &TransportEndpoint,
        request: &serde_json::Value,
    ) -> std::result::Result<String, crate::transport::JsonRpcTransportError> {
        crate::transport::send_jsonrpc_request(
            endpoint,
            request,
            PROVENANCE_CONNECTION_TIMEOUT,
            PROVENANCE_RESPONSE_TIMEOUT,
        )
        .await
    }

    /// Get the current endpoint.
    pub async fn endpoint(&self) -> Option<TransportEndpoint> {
        self.endpoint.read().await.clone()
    }
}

#[cfg(test)]
#[path = "client_tests.rs"]
mod tests;
