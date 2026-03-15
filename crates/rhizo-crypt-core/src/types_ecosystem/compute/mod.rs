// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2024–2026 ecoPrimals Project

//! Compute Provider Types - Task Events & Configuration
//!
//! Type definitions and client for compute orchestration capability providers.
//! These types work with ANY compute provider (compute provider, Kubernetes, Nomad, custom).
//!
//! ## Capability-Based Architecture
//!
//! Compute providers are discovered via the `compute:orchestration` capability.
//! The primal doesn't know or care which specific service provides the capability.
//!
//! ```text
//! rhizoCrypt              Bootstrap              Compute Provider
//!     │                      │                         │
//!     │──discover(compute)──▶│                         │
//!     │◀──ServiceEndpoint────│                         │
//!     │                      │                         │
//!     │──────────subscribe(task_id)────────────────────▶│
//!     │◀────────Stream<ComputeEvent>───────────────────│
//! ```

mod client;
mod types;

// Re-export all public items to preserve the same API surface
pub use client::ComputeProviderClient;
pub use types::{ClientState, ComputeEvent, ComputeProviderConfig, TaskId};
