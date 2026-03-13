// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2024–2026 ecoPrimals Project

//! Runtime primal discovery - capability-based service location.
//!
//! This module implements the ecoPrimals principle that primals have only self-knowledge
//! and discover other primals at runtime through capability-based discovery.
//!
//! ## Philosophy
//!
//! - **No hardcoded addresses** — Services are discovered, not configured
//! - **Capability-based** — Request what you need, not who provides it
//! - **Runtime resolution** — Bindings happen at runtime, not compile time
//! - **Graceful degradation** — Missing services don't crash, they report unavailability

mod capability;
mod endpoint;
mod registry;
mod resolution;

// Re-export all public items to maintain the same API surface
pub use capability::Capability;
pub use endpoint::ServiceEndpoint;
pub use registry::{DiscoveryRegistry, DiscoveryStatus};
pub use resolution::ClientProvider;
