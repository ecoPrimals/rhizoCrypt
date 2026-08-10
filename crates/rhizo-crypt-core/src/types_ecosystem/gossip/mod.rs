// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2024–2026 ecoPrimals Project

//! Gossip mesh integration — swarmVine event emission.
//!
//! rhizoCrypt injects lifecycle events into the gossip mesh via
//! `gossip.spread` JSON-RPC calls to any `gossip:relay` provider
//! (swarmVine in the deploy graph).
//!
//! ## Injection Points
//!
//! ```text
//! dehydrate()       → SessionDehydrated  (session committed to permanent storage)
//! dehydrate_batch() → BatchDehydrated    (N sessions committed)
//! impl_federate()   → Federated          (vertices imported from remote gate)
//! ```
//!
//! ## Ant Colony Pattern
//!
//! These events are the "scout reports" for the ant colony:
//! - `SessionDehydrated` → "permanent data available at this Merkle root"
//! - `BatchDehydrated`   → "bulk ingest complete, N sessions ready"
//! - `Federated`         → "data arrived from remote gate"
//!
//! ## Lifecycle
//!
//! Started in [`PrimalLifecycle::start()`](crate::primal::PrimalLifecycle)
//! after provenance notifier and mesh listener. Non-fatal — gossip
//! emission is optional and fire-and-forget.

mod emitter;
mod types;

pub use emitter::GossipEmitter;
pub use types::GossipEvent;
