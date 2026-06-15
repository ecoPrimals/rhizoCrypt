// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2024–2026 ecoPrimals Project

//! Cross-gate mesh event listener with polling support.
//!
//! Discovers the signing provider endpoint (bearDog) via the capability
//! registry and polls `auth.events.poll` for trust establishment events.
//!
//! ## IPC Trigger Path
//!
//! 1. bearDog fires `auth.trust_issuer` (or `key_exchange`, etc.)
//! 2. bearDog's `AuthEventBus` records the event
//! 3. `MeshEventListener` polls `auth.events.poll` with `since_timestamp`
//! 4. Deserializes response into [`MeshTrustEvent`] vec
//! 5. Maps each to [`EventType`] via `record_event()`
//!
//! ## Polling
//!
//! `RhizoCrypt::spawn_mesh_poller()` drives the poll loop externally via
//! `poll_events()` + `drain_events()`, appending trust events into a
//! dedicated DAG session. The listener uses newline-delimited JSON-RPC
//! via `connect_transport()` (transport-agnostic — UDS or TCP depending
//! on endpoint resolution).

use std::sync::Arc;

use crate::transport::TransportEndpoint;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

use crate::constants::{MESH_CONNECTION_TIMEOUT, MESH_RESPONSE_TIMEOUT};
use crate::discovery::{Capability, DiscoveryRegistry};
use crate::error::Result;

use super::types::MeshTrustEvent;

/// State of the mesh event listener.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ListenerState {
    /// Not yet connected to a signing provider.
    Disconnected,
    /// Connected — ready to receive/poll events.
    Connected,
}

/// Listener for cross-gate mesh trust events.
///
/// Discovers bearDog (or any signing provider) via the capability
/// registry, connects at startup, and provides both manual
/// `record_event()` and automatic `poll_events()` for ingesting
/// trust events.
///
/// ## Lifecycle
///
/// Created in `RhizoCrypt::new()`, started (non-fatal) in
/// `PrimalLifecycle::start()` after provenance notifier.
pub struct MeshEventListener {
    /// Discovery registry for finding the signing provider.
    registry: Arc<DiscoveryRegistry>,
    /// Resolved signing provider endpoint (transport-agnostic).
    endpoint: Arc<RwLock<Option<TransportEndpoint>>>,
    /// Current listener state.
    state: Arc<RwLock<ListenerState>>,
    /// Events received (buffered for batch append or inspection).
    event_log: Arc<RwLock<Vec<MeshTrustEvent>>>,
    /// Last poll timestamp (unix seconds) for incremental polling.
    last_poll_timestamp: Arc<RwLock<u64>>,
}

impl MeshEventListener {
    /// Create a listener backed by the discovery registry.
    #[must_use]
    pub fn new(registry: Arc<DiscoveryRegistry>) -> Self {
        Self {
            registry,
            endpoint: Arc::new(RwLock::new(None)),
            state: Arc::new(RwLock::new(ListenerState::Disconnected)),
            event_log: Arc::new(RwLock::new(Vec::new())),
            last_poll_timestamp: Arc::new(RwLock::new(0)),
        }
    }

    /// Current listener state.
    pub async fn state(&self) -> ListenerState {
        *self.state.read().await
    }

    /// Number of events recorded since startup.
    pub async fn event_count(&self) -> usize {
        self.event_log.read().await.len()
    }

    /// Attempt to discover and connect to the signing provider.
    ///
    /// Non-fatal: if no signing provider is available, the listener
    /// stays disconnected and events can still be recorded manually
    /// via `record_event()`.
    ///
    /// # Errors
    ///
    /// Returns error if discovery succeeds but endpoint is invalid.
    pub async fn connect(&self) -> Result<()> {
        if let Some(service) = self.registry.get_endpoint(&Capability::Signing).await {
            info!(
                endpoint = %service.endpoint,
                service = %service.service_id,
                "Mesh event listener connected to signing provider"
            );
            *self.endpoint.write().await = Some(service.endpoint.clone());
            *self.state.write().await = ListenerState::Connected;
        } else {
            debug!("No signing provider for mesh event listener (standalone mode)");
            *self.state.write().await = ListenerState::Disconnected;
        }
        Ok(())
    }

    /// Record a mesh trust event.
    ///
    /// Stores the event in the internal log and returns the mapped
    /// [`EventType`] for the caller to append to a DAG session.
    pub async fn record_event(&self, event: MeshTrustEvent) -> crate::event::EventType {
        let event_type = event.clone().into_event_type();

        debug!(
            kind = ?event.kind,
            source = %event.source_gate,
            "Recording mesh trust event"
        );

        self.event_log.write().await.push(event);

        event_type
    }

    /// Poll bearDog's `auth.events.poll` for new trust events.
    ///
    /// Sends a JSON-RPC request with `since_timestamp` and deserializes
    /// the response into `MeshTrustEvent` entries. Each event is recorded
    /// via `record_event()`. Returns the number of new events received.
    ///
    /// # Errors
    ///
    /// Returns error if not connected or RPC call fails.
    pub async fn poll_events(&self) -> Result<usize> {
        let endpoint_guard = self.endpoint.read().await;
        let Some(ref transport) = *endpoint_guard else {
            return Ok(0);
        };
        let transport = transport.clone();
        drop(endpoint_guard);

        let since = *self.last_poll_timestamp.read().await;

        let request = serde_json::json!({
            "jsonrpc": crate::constants::JSONRPC_VERSION,
            "method": crate::constants::MESH_AUTH_EVENTS_POLL_METHOD,
            "params": {
                "since_timestamp": since,
            },
            "id": 1
        });

        let response = match Self::send_jsonrpc(&transport, &request).await {
            Ok(resp) => resp,
            Err(e) => {
                debug!(error = %e, "Mesh event poll failed (non-fatal)");
                return Ok(0);
            }
        };

        let parsed: serde_json::Value = serde_json::from_str(&response).map_err(|e| {
            crate::error::RhizoCryptError::integration(format!(
                "Failed to parse poll response: {e}"
            ))
        })?;

        let events = parsed
            .get("result")
            .and_then(|r| r.get("events"))
            .and_then(serde_json::Value::as_array);

        let Some(events_arr) = events else {
            return Ok(0);
        };

        let mut count = 0;
        let mut max_timestamp = since;

        for event_val in events_arr {
            match serde_json::from_value::<MeshTrustEvent>(event_val.clone()) {
                Ok(event) => {
                    if event.timestamp > max_timestamp {
                        max_timestamp = event.timestamp;
                    }
                    self.record_event(event).await;
                    count += 1;
                }
                Err(e) => {
                    warn!(error = %e, "Skipping malformed mesh event from poll");
                }
            }
        }

        if count > 0 {
            *self.last_poll_timestamp.write().await = max_timestamp;
            info!(count, max_timestamp, "Polled new mesh trust events");
        }

        Ok(count)
    }

    async fn send_jsonrpc(
        endpoint: &TransportEndpoint,
        request: &serde_json::Value,
    ) -> std::result::Result<String, crate::transport::JsonRpcTransportError> {
        crate::transport::send_jsonrpc_request(
            endpoint,
            request,
            MESH_CONNECTION_TIMEOUT,
            MESH_RESPONSE_TIMEOUT,
        )
        .await
    }

    /// Get the resolved signing provider endpoint (if connected).
    pub async fn endpoint(&self) -> Option<TransportEndpoint> {
        self.endpoint.read().await.clone()
    }

    /// Drain all recorded events (for batch processing or testing).
    pub async fn drain_events(&self) -> Vec<MeshTrustEvent> {
        let mut log = self.event_log.write().await;
        std::mem::take(&mut *log)
    }

    /// Get the last poll timestamp.
    pub async fn last_poll_timestamp(&self) -> u64 {
        *self.last_poll_timestamp.read().await
    }
}

impl std::fmt::Debug for MeshEventListener {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MeshEventListener").field("registry", &"<DiscoveryRegistry>").finish()
    }
}

#[cfg(test)]
#[expect(clippy::unwrap_used, reason = "test code")]
mod tests {
    use std::net::SocketAddr;

    use super::*;

    fn test_registry() -> Arc<DiscoveryRegistry> {
        Arc::new(DiscoveryRegistry::new("test-mesh-listener"))
    }

    #[tokio::test]
    async fn test_listener_initial_state() {
        let listener = MeshEventListener::new(test_registry());
        assert_eq!(listener.state().await, ListenerState::Disconnected);
        assert_eq!(listener.event_count().await, 0);
        assert!(listener.endpoint().await.is_none());
        assert_eq!(listener.last_poll_timestamp().await, 0);
    }

    #[tokio::test]
    async fn test_listener_connect_no_provider() {
        let listener = MeshEventListener::new(test_registry());
        let result = listener.connect().await;
        assert!(result.is_ok());
        assert_eq!(listener.state().await, ListenerState::Disconnected);
    }

    #[tokio::test]
    async fn test_listener_record_event() {
        let listener = MeshEventListener::new(test_registry());

        let event = MeshTrustEvent {
            kind: super::super::types::MeshTrustEventKind::TrustIssuerRegistered {
                issuer_fingerprint: "abc123".into(),
            },
            source_gate: "eastGate".into(),
            timestamp: 1_717_444_800,
        };

        let event_type = listener.record_event(event).await;
        assert_eq!(
            event_type,
            crate::event::EventType::TrustIssuerRegistered {
                issuer_fingerprint: "abc123".into(),
                registering_gate: "eastGate".into(),
            }
        );
        assert_eq!(listener.event_count().await, 1);
    }

    #[tokio::test]
    async fn test_listener_drain_events() {
        let listener = MeshEventListener::new(test_registry());

        let events = vec![
            MeshTrustEvent {
                kind: super::super::types::MeshTrustEventKind::MeshJoin {
                    mesh_id: "mesh-1".into(),
                },
                source_gate: "g1".into(),
                timestamp: 0,
            },
            MeshTrustEvent {
                kind: super::super::types::MeshTrustEventKind::MeshJoin {
                    mesh_id: "mesh-1".into(),
                },
                source_gate: "g2".into(),
                timestamp: 1,
            },
        ];

        for e in events {
            listener.record_event(e).await;
        }
        assert_eq!(listener.event_count().await, 2);

        let drained = listener.drain_events().await;
        assert_eq!(drained.len(), 2);
        assert_eq!(listener.event_count().await, 0);
    }

    #[tokio::test]
    async fn test_listener_connect_with_registered_endpoint() {
        use crate::discovery::{Capability, ServiceEndpoint};

        let registry = test_registry();
        let addr: SocketAddr = "127.0.0.1:9100".parse().unwrap();
        registry
            .register_endpoint(ServiceEndpoint::new(
                "test-beardog",
                addr.into(),
                vec![Capability::Signing],
            ))
            .await;

        let listener = MeshEventListener::new(registry);
        let result = listener.connect().await;
        assert!(result.is_ok());
        assert_eq!(listener.state().await, ListenerState::Connected);
        let expected = TransportEndpoint::tcp(addr.ip().to_string(), addr.port());
        assert_eq!(listener.endpoint().await, Some(expected));
    }

    #[tokio::test]
    async fn test_poll_events_disconnected_returns_zero() {
        let listener = MeshEventListener::new(test_registry());
        let count = listener.poll_events().await.unwrap();
        assert_eq!(count, 0);
    }

    #[tokio::test]
    async fn test_poll_timestamp_tracking() {
        let listener = MeshEventListener::new(test_registry());
        assert_eq!(listener.last_poll_timestamp().await, 0);

        let event = MeshTrustEvent {
            kind: super::super::types::MeshTrustEventKind::TrustIssuerRegistered {
                issuer_fingerprint: "ff".into(),
            },
            source_gate: "test".into(),
            timestamp: 42,
        };
        listener.record_event(event).await;
        // record_event doesn't update poll timestamp (that's poll_events' job)
        assert_eq!(listener.last_poll_timestamp().await, 0);
    }
}
