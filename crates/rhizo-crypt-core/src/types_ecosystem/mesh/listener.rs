// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2024–2026 ecoPrimals Project

//! Cross-gate mesh event listener.
//!
//! Discovers the signing provider endpoint (bearDog) via the capability
//! registry and prepares to receive trust establishment events. When
//! events arrive (via poll or push), maps them to [`EventType`] mesh
//! variants and appends them to a dedicated mesh-trust DAG session.
//!
//! ## IPC Trigger Path
//!
//! 1. bearDog fires `auth.trust_issuer` (or `key_exchange`, etc.)
//! 2. `MeshEventListener` receives the event via JSON-RPC
//! 3. Deserializes into [`MeshTrustEvent`]
//! 4. Maps to [`EventType::TrustIssuerRegistered`] (etc.)
//! 5. Appends to the mesh-trust session via internal DAG API
//!
//! ## Current Status
//!
//! Scaffold: discovery + connect lifecycle implemented. Event ingestion
//! requires bearDog w137+ to expose `auth.events.subscribe` or
//! `ipc.watch` on its signing endpoint. Until then, events can be
//! injected via `record_event()` from the RPC handler layer.

use std::net::SocketAddr;
use std::sync::Arc;

use tokio::sync::RwLock;
use tracing::{debug, info};

use crate::discovery::{Capability, DiscoveryRegistry};
use crate::error::Result;

use super::types::MeshTrustEvent;

/// State of the mesh event listener.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ListenerState {
    /// Not yet connected to a signing provider.
    Disconnected,
    /// Connected — ready to receive events.
    Connected,
    /// Connection failed — will retry on next event.
    Failed,
}

/// Listener for cross-gate mesh trust events.
///
/// Discovers bearDog (or any signing provider) via the capability
/// registry, connects at startup, and provides `record_event()` for
/// mapping trust events to DAG vertices.
///
/// ## Lifecycle
///
/// Created in `RhizoCrypt::new()`, started (non-fatal) in
/// `PrimalLifecycle::start()` after provenance notifier.
pub struct MeshEventListener {
    /// Discovery registry for finding the signing provider.
    registry: Arc<DiscoveryRegistry>,
    /// Resolved signing provider endpoint.
    endpoint: Arc<RwLock<Option<SocketAddr>>>,
    /// Current listener state.
    state: Arc<RwLock<ListenerState>>,
    /// Events received (buffered for batch append or inspection).
    event_log: Arc<RwLock<Vec<MeshTrustEvent>>>,
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
        if let Some(endpoint) = self.registry.get_endpoint(&Capability::Signing).await {
            info!(
                address = %endpoint.addr,
                service = %endpoint.service_id,
                "Mesh event listener connected to signing provider"
            );
            *self.endpoint.write().await = Some(endpoint.addr);
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
    ///
    /// This is the primary ingestion point. In the future, a background
    /// task will poll/watch the signing provider and call this method
    /// automatically. For now, the RPC handler layer can call this when
    /// it receives cross-gate trust notifications.
    pub async fn record_event(
        &self,
        event: MeshTrustEvent,
    ) -> crate::event::EventType {
        let event_type = event.clone().into_event_type();

        debug!(
            kind = ?event.kind,
            source = %event.source_gate,
            "Recording mesh trust event"
        );

        self.event_log.write().await.push(event);

        event_type
    }

    /// Get the resolved signing provider endpoint (if connected).
    pub async fn endpoint(&self) -> Option<SocketAddr> {
        *self.endpoint.read().await
    }

    /// Drain all recorded events (for batch processing or testing).
    pub async fn drain_events(&self) -> Vec<MeshTrustEvent> {
        let mut log = self.event_log.write().await;
        std::mem::take(&mut *log)
    }
}

impl std::fmt::Debug for MeshEventListener {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MeshEventListener")
            .field("registry", &"<DiscoveryRegistry>")
            .finish()
    }
}

#[cfg(test)]
#[expect(clippy::unwrap_used, reason = "test code")]
mod tests {
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
                addr,
                vec![Capability::Signing],
            ))
            .await;

        let listener = MeshEventListener::new(registry);
        let result = listener.connect().await;
        assert!(result.is_ok());
        assert_eq!(listener.state().await, ListenerState::Connected);
        assert_eq!(listener.endpoint().await, Some(addr));
    }
}
