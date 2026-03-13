// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2024–2026 ecoPrimals Project

//! Types for Songbird client.
//!
//! This module contains the core types used by the Songbird client for
//! service discovery, registration, and federation status.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Information about a discovered service.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceInfo {
    /// Service identifier.
    pub id: String,

    /// Service name.
    pub name: String,

    /// Service endpoint address.
    pub endpoint: String,

    /// Service capabilities.
    pub capabilities: Vec<String>,

    /// Service status.
    pub status: String,

    /// Optional metadata.
    #[serde(default)]
    pub metadata: HashMap<String, String>,
}

impl ServiceInfo {
    /// Check if this service provides a specific capability.
    #[must_use]
    pub fn has_capability(&self, cap: &str) -> bool {
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
