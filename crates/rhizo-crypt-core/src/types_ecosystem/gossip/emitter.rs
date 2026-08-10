// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2024–2026 ecoPrimals Project

//! Gossip mesh emitter — announces DAG lifecycle events to swarmVine.
//!
//! Follows the [`ProvenanceNotifier`](crate::types_ecosystem::provenance::ProvenanceNotifier)
//! pattern: discovers a `gossip:relay` provider via the discovery registry,
//! then sends `gossip.spread` JSON-RPC calls. Non-fatal if unavailable.

use std::sync::Arc;

use tokio::sync::RwLock;
use tracing::{debug, info, warn};

use crate::constants::{GOSSIP_CONNECTION_TIMEOUT, GOSSIP_RESPONSE_TIMEOUT};
use crate::discovery::{Capability, DiscoveryRegistry};
use crate::error::{Result, RhizoCryptError};
use crate::transport::TransportEndpoint;

use super::types::GossipEvent;

/// Emits gossip events to the mesh via any `gossip:relay` provider.
///
/// Created at `RhizoCrypt` startup, attempts connection during
/// `PrimalLifecycle::start()`. If no gossip relay is discoverable,
/// all `emit_*` calls silently succeed (gossip is always optional).
pub struct GossipEmitter {
    registry: Option<Arc<DiscoveryRegistry>>,
    endpoint: Arc<RwLock<Option<TransportEndpoint>>>,
    connected: Arc<RwLock<bool>>,
}

impl GossipEmitter {
    /// Create a gossip emitter with discovery support.
    #[must_use]
    pub fn with_discovery(registry: Arc<DiscoveryRegistry>) -> Self {
        Self {
            registry: Some(registry),
            endpoint: Arc::new(RwLock::new(None)),
            connected: Arc::new(RwLock::new(false)),
        }
    }

    /// Create a disconnected emitter (for standalone/test use).
    #[must_use]
    pub fn disconnected() -> Self {
        Self {
            registry: None,
            endpoint: Arc::new(RwLock::new(None)),
            connected: Arc::new(RwLock::new(false)),
        }
    }

    /// Attempt to discover and connect to a gossip relay.
    ///
    /// # Errors
    ///
    /// Returns error only for unexpected failures; missing provider is OK.
    pub async fn connect(&self) -> Result<()> {
        if let Some(registry) = &self.registry
            && let Some(service) = registry.get_endpoint(&Capability::GossipRelay).await
        {
            info!(endpoint = %service.endpoint, "Discovered gossip relay via registry");
            *self.endpoint.write().await = Some(service.endpoint.clone());
            *self.connected.write().await = true;
            return Ok(());
        }

        if let Ok(val) = std::env::var("GOSSIP_RELAY_ENDPOINT") {
            let transport = serde_json::from_str::<TransportEndpoint>(&val).or_else(|_| {
                val.parse::<std::net::SocketAddr>()
                    .map(|sa| TransportEndpoint::tcp(sa.ip().to_string(), sa.port()))
                    .map_err(|e| {
                        RhizoCryptError::integration(format!(
                            "Invalid gossip relay address '{val}': {e}"
                        ))
                    })
            })?;

            debug!(endpoint = %transport, "Connecting to gossip relay from env");
            *self.endpoint.write().await = Some(transport);
            *self.connected.write().await = true;
            return Ok(());
        }

        debug!("No gossip relay available — gossip emission disabled (non-fatal)");
        Ok(())
    }

    /// Emit a gossip event to the mesh.
    ///
    /// Non-fatal: silently succeeds if no relay is connected or if the
    /// relay is unreachable. Gossip is fire-and-forget.
    ///
    /// # Errors
    ///
    /// Always returns `Ok` — errors from the transport are logged and
    /// swallowed (gossip is best-effort).
    pub async fn emit(&self, event: &GossipEvent) -> Result<()> {
        if !*self.connected.read().await {
            return Ok(());
        }

        let Some(endpoint) = self.endpoint.read().await.clone() else {
            return Ok(());
        };

        let kind = event.kind_str();
        debug!(kind, %endpoint, "Emitting gossip event");

        let request = serde_json::json!({
            "jsonrpc": crate::constants::JSONRPC_VERSION,
            "method": crate::constants::GOSSIP_SPREAD_METHOD,
            "params": {
                "source_primal": crate::constants::PRIMAL_NAME,
                "domain": "dag",
                "event": event,
            },
            "id": 1
        });

        match crate::transport::send_jsonrpc_request(
            &endpoint,
            &request,
            GOSSIP_CONNECTION_TIMEOUT,
            GOSSIP_RESPONSE_TIMEOUT,
        )
        .await
        {
            Ok(response) => {
                info!(kind, "Gossip emitted: {response}");
            }
            Err(e) => {
                warn!(kind, error = %e, "Gossip emission failed (non-fatal)");
            }
        }

        Ok(())
    }

    /// Whether a gossip relay is connected.
    pub async fn is_connected(&self) -> bool {
        *self.connected.read().await
    }
}

#[cfg(test)]
#[expect(clippy::unwrap_used, reason = "test code")]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_disconnected_emitter_noop() {
        let emitter = GossipEmitter::disconnected();
        assert!(!emitter.is_connected().await);

        let event = GossipEvent::SessionDehydrated {
            session_id: "s1".into(),
            merkle_root: "abc".into(),
            vertex_count: 5,
        };
        let result = emitter.emit(&event).await;
        assert!(result.is_ok());
    }

    #[test]
    fn test_connect_without_registry_or_env() {
        let rt = tokio::runtime::Builder::new_current_thread().enable_all().build().unwrap();

        temp_env::with_vars([("GOSSIP_RELAY_ENDPOINT", None::<&str>)], || {
            rt.block_on(async {
                let emitter = GossipEmitter::disconnected();
                let result = emitter.connect().await;
                assert!(result.is_ok());
                assert!(!emitter.is_connected().await);
            });
        });
    }

    #[test]
    fn test_connect_from_env() {
        let rt = tokio::runtime::Builder::new_current_thread().enable_all().build().unwrap();

        temp_env::with_vars([("GOSSIP_RELAY_ENDPOINT", Some("127.0.0.1:7800"))], || {
            rt.block_on(async {
                let e = GossipEmitter {
                    registry: None,
                    endpoint: Arc::new(RwLock::new(None)),
                    connected: Arc::new(RwLock::new(false)),
                };
                e.connect().await.unwrap();
                assert!(e.is_connected().await);
            });
        });
    }

    #[tokio::test]
    async fn test_emit_when_connected_no_server() {
        let emitter = GossipEmitter {
            registry: None,
            endpoint: Arc::new(RwLock::new(Some(TransportEndpoint::tcp(
                "127.0.0.1".to_string(),
                19999,
            )))),
            connected: Arc::new(RwLock::new(true)),
        };

        let event = GossipEvent::Federated {
            session_id: "s1".into(),
            imported_count: 3,
            source_gate: Some("test".into()),
        };

        let result = emitter.emit(&event).await;
        assert!(result.is_ok());
    }
}
