// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2024–2026 ecoPrimals Project

//! Cross-gate mesh trust event listener.
//!
//! Records trust establishment and mesh lifecycle events from bearDog
//! (or any `crypto:signing` provider) into rhizoCrypt's DAG. Mirrors
//! the outbound [`ProvenanceNotifier`](super::provenance::ProvenanceNotifier)
//! but in the inbound direction: bearDog fires auth events, rhizoCrypt
//! records them as `EventType::TrustIssuerRegistered` etc.
//!
//! ## IPC Trigger Path
//!
//! ```text
//! bearDog (auth.trust_issuer fires)
//!   → rhizoCrypt MeshEventListener polls/watches signing endpoint
//!   → maps payload to EventType::TrustIssuerRegistered
//!   → appends to dedicated mesh-trust session via internal DAG API
//! ```
//!
//! ## Lifecycle
//!
//! Started in [`PrimalLifecycle::start()`] after provenance notifier.
//! Non-fatal — mesh event recording is optional.

mod listener;
mod types;

pub use listener::MeshEventListener;
pub use types::{MeshTrustEvent, MeshTrustEventKind};
