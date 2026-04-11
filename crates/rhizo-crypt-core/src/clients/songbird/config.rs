// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2024–2026 ecoPrimals Project

//! Songbird client configuration.

use std::borrow::Cow;
use std::collections::HashMap;
use std::time::Duration;

use tracing::warn;

/// Configuration for Songbird client.
///
/// Songbird is special: it's the bootstrap for discovery, so its address
/// is the only one that should be configured directly.
#[derive(Debug, Clone)]
pub struct SongbirdConfig {
    /// Songbird orchestrator address.
    /// This is the bootstrap address - discovered from environment or config.
    pub address: Cow<'static, str>,

    /// Service name for registration.
    pub service_name: Cow<'static, str>,

    /// Capabilities to advertise.
    pub capabilities: Vec<Cow<'static, str>>,

    /// Metadata to include in registration.
    pub metadata: HashMap<String, String>,

    /// Connection timeout in milliseconds.
    pub timeout_ms: u64,

    /// Enable automatic reconnection.
    pub auto_reconnect: bool,

    /// Heartbeat interval for registration refresh.
    /// Songbird registrations expire after 60s; default is 45s.
    pub heartbeat_interval: Duration,
}

impl Default for SongbirdConfig {
    fn default() -> Self {
        Self::from_env()
    }
}

impl SongbirdConfig {
    /// Create a new config with no address configured.
    ///
    /// This is the preferred constructor - requires explicit address configuration.
    /// Songbird is the discovery bootstrap; its address is discovered from
    /// environment, never hardcoded.
    #[must_use]
    pub fn new() -> Self {
        Self {
            address: Cow::Borrowed(""),
            service_name: Cow::Borrowed(crate::constants::PRIMAL_NAME),
            capabilities: crate::constants::ADVERTISED_CAPABILITIES
                .iter()
                .map(|&s| Cow::Borrowed(s))
                .collect(),
            metadata: HashMap::new(),
            timeout_ms: crate::constants::DEFAULT_CAPABILITY_TIMEOUT_MS,
            auto_reconnect: true,
            heartbeat_interval: crate::constants::DEFAULT_HEARTBEAT_INTERVAL,
        }
    }

    /// Check if this config has a valid address configured.
    #[must_use]
    pub fn is_configured(&self) -> bool {
        !self.address.is_empty()
    }
}

impl SongbirdConfig {
    /// Create config from environment variables.
    ///
    /// Environment variables (checked in order):
    /// - `DISCOVERY_ENDPOINT` or `DISCOVERY_SERVICE_ENDPOINT`: Discovery capability endpoint (preferred)
    /// - `SONGBIRD_ADDRESS`: Legacy orchestrator address (acceptable - Songbird is the universal adapter)
    /// - `SONGBIRD_HOST` + `SONGBIRD_PORT`: Alternative host/port specification
    /// - `RHIZOCRYPT_SERVICE_NAME`: Service name for registration
    ///
    /// If no discovery address is configured, the address remains empty and
    /// `connect()` will fail with a clear error. No hardcoded fallbacks.
    #[must_use]
    pub fn from_env() -> Self {
        use crate::safe_env::CapabilityEnv;
        let mut config = Self::new();

        if let Some(addr) = CapabilityEnv::discovery_endpoint() {
            config.address = Cow::Owned(addr);
        } else if let (Ok(host), Ok(port)) =
            (std::env::var("SONGBIRD_HOST"), std::env::var("SONGBIRD_PORT"))
        {
            config.address = Cow::Owned(format!("{host}:{port}"));
        } else {
            warn!(
                "No discovery endpoint configured. \
                 Set DISCOVERY_ENDPOINT, SONGBIRD_ADDRESS, or SONGBIRD_HOST+SONGBIRD_PORT"
            );
        }

        if let Ok(name) = std::env::var("RHIZOCRYPT_SERVICE_NAME") {
            config.service_name = Cow::Owned(name);
        }

        config
    }

    /// Create config with explicit address (for testing or explicit configuration).
    #[must_use]
    pub fn with_address(address: impl Into<Cow<'static, str>>) -> Self {
        let mut config = Self::new();
        config.address = address.into();
        config
    }
}
