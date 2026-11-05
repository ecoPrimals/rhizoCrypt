// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2024–2026 ecoPrimals Project

//! Types for Songbird client.
//!
//! This module contains the core types used by the Songbird client for
//! service discovery, registration, and federation status.
//!
//! `ServiceInfo` is the canonical type from `cellmembrane-types`.

use serde::{Deserialize, Serialize};

// ── Canonical re-exports ──
pub use cellmembrane_types::ecosystem::ServiceInfo;
pub use cellmembrane_types::ecosystem::EcosystemServiceStatus as ServiceStatus;

/// Extension trait for `ServiceInfo` — rhizoCrypt convenience helpers.
pub trait ServiceInfoExt {
    /// Check if this service provides a specific capability.
    fn has_capability(&self, cap: &str) -> bool;
}

impl ServiceInfoExt for ServiceInfo {
    fn has_capability(&self, cap: &str) -> bool {
        self.capabilities.iter().any(|c| c == cap)
    }
}

/// Registration result from Songbird.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegistrationResult {
    /// Whether registration succeeded.
    pub success: bool,

    /// Registration message.
    pub message: String,

    /// Assigned service ID (if successful).
    pub service_id: Option<String>,
}

/// Federation status from Songbird.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FederationStatus {
    /// Total registered services.
    pub total_services: usize,

    /// Total federation peers.
    pub total_peers: usize,

    /// Orchestrator uptime in seconds.
    pub uptime_seconds: u64,

    /// Orchestrator version.
    pub version: String,
}

/// Client state for connection management.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClientState {
    /// Not connected.
    Disconnected,

    /// Connecting to orchestrator.
    Connecting,

    /// Connected and ready.
    Connected,

    /// Registered with mesh.
    Registered,

    /// Connection failed.
    Failed,
}
