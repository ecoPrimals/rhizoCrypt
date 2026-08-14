// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2024–2026 ecoPrimals Project

//! Cross-gate mesh trust event listener.
//!
//! Records trust establishment and mesh lifecycle events from the signing
//! provider (discoverable at runtime via `crypto:signing`) into rhizoCrypt's DAG.
//! Mirrors the outbound [`ProvenanceNotifier`](super::provenance::ProvenanceNotifier)
//! but in the inbound direction: the auth provider fires auth events, rhizoCrypt
//! records them as `EventType::TrustIssuerRegistered` etc.
//!
//! ## IPC Trigger Path
//!
//! ```text
//! signing provider (auth.trust_issuer fires → AuthEventBus records)
//!   → rhizoCrypt MeshEventListener polls auth.events.poll
//!   → deserializes MeshTrustEvent from JSON-RPC response
//!   → maps to EventType::TrustIssuerRegistered via into_event_type()
//!   → records in event log (ready for DAG session append)
//! ```
//!
//! ## Polling
//!
//! `RhizoCrypt::spawn_mesh_poller()` drives the poll loop via
//! `poll_events()` + `drain_events()` every
//! [`MESH_POLL_INTERVAL`](crate::constants::MESH_POLL_INTERVAL).
//! Uses incremental `since_timestamp` to fetch only new events.
//!
//! ## Lifecycle
//!
//! Started in [`crate::PrimalLifecycle::start()`] after provenance notifier.
//! Non-fatal — mesh event recording is optional.

mod listener;
mod types;

pub use listener::MeshEventListener;
pub use types::{MeshTrustEvent, MeshTrustEventKind};
