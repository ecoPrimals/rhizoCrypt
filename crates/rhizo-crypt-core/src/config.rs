// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2024–2026 ecoPrimals Project

//! `RhizoCrypt` configuration.
//!
//! This module defines the configuration structure for the `RhizoCrypt` primal.
//!
//! ## Design Philosophy
//!
//! - **Capability-based defaults** — Primals discover services at runtime
//! - **Zero-knowledge initialization** — Works with no external configuration
//! - **Environment-aware** — Can be customized via environment variables

use crate::constants;
use crate::session::SessionConfig;
use serde::{Deserialize, Serialize};
use std::borrow::Cow;
use std::net::{IpAddr, Ipv4Addr};
use std::time::Duration;

/// Configuration for `RhizoCrypt`.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RhizoCryptConfig {
    /// Primal name.
    pub name: String,

    /// Default session configuration.
    pub default_session: SessionConfig,

    /// Maximum concurrent sessions.
    pub max_sessions: usize,

    /// Garbage collection interval.
    #[serde(with = "duration_serde")]
    pub gc_interval: Duration,

    /// Session expiration grace period after commit.
    #[serde(with = "duration_serde")]
    pub expiration_grace: Duration,

    /// Storage backend configuration.
    pub storage: StorageConfig,

    /// Metrics configuration.
    pub metrics: MetricsConfig,

    /// Dehydration configuration.
    pub dehydration: DehydrationClientConfig,

    /// Slice configuration.
    pub slice: SliceConfig,

    /// RPC server configuration.
    pub rpc: RpcConfig,
}

impl Default for RhizoCryptConfig {
    fn default() -> Self {
        Self {
            name: constants::PRIMAL_NAME.to_string(),
            default_session: SessionConfig::default(),
            max_sessions: constants::DEFAULT_MAX_SESSIONS,
            gc_interval: constants::DEFAULT_GC_INTERVAL,
            expiration_grace: constants::DEFAULT_EXPIRATION_GRACE,
            storage: StorageConfig::default(),
            metrics: MetricsConfig::default(),
            dehydration: DehydrationClientConfig::default(),
            slice: SliceConfig::default(),
            rpc: RpcConfig::default(),
        }
    }
}

impl RhizoCryptConfig {
    /// Create a new configuration with the given name.
    #[must_use]
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            ..Self::default()
        }
    }

    /// Set the maximum number of concurrent sessions.
    #[must_use]
    pub const fn with_max_sessions(mut self, max: usize) -> Self {
        self.max_sessions = max;
        self
    }

    /// Set the garbage collection interval.
    #[must_use]
    pub const fn with_gc_interval(mut self, interval: Duration) -> Self {
        self.gc_interval = interval;
        self
    }

    /// Set the storage backend.
    #[must_use]
    pub fn with_storage(mut self, storage: StorageConfig) -> Self {
        self.storage = storage;
        self
    }
}

/// Storage backend configuration.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct StorageConfig {
    /// Storage backend type.
    pub backend: StorageBackend,

    /// Path for persistent storage (redb).
    pub path: Option<String>,

    /// Maximum size in bytes for in-memory storage.
    pub max_memory_bytes: Option<u64>,
}

impl Default for StorageConfig {
    fn default() -> Self {
        Self {
            backend: StorageBackend::Memory,
            path: None,
            max_memory_bytes: Some(1024 * 1024 * 1024), // 1 GB
        }
    }
}

/// Storage backend type.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum StorageBackend {
    /// In-memory storage (fastest, no persistence).
    #[default]
    Memory,

    /// redb storage (Pure Rust, persistent, ACID, MVCC, ecoBin compliant).
    /// Recommended for production use.
    Redb,
}

/// Metrics configuration.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MetricsConfig {
    /// Enable metrics collection.
    pub enabled: bool,

    /// Metrics endpoint path.
    pub endpoint: String,

    /// Include detailed per-session metrics.
    pub per_session_metrics: bool,
}

impl Default for MetricsConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            endpoint: "/metrics".to_string(),
            per_session_metrics: false,
        }
    }
}

/// Dehydration client configuration.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DehydrationClientConfig {
    /// Default timeout for attestation collection.
    #[serde(with = "duration_serde")]
    pub attestation_timeout: Duration,

    /// Maximum attestation retries.
    pub max_retries: u32,

    /// Retry delay.
    #[serde(with = "duration_serde")]
    pub retry_delay: Duration,

    /// Include full vertices in summary by default.
    pub include_vertices: bool,

    /// Include payloads in summary by default.
    pub include_payloads: bool,
}

impl Default for DehydrationClientConfig {
    fn default() -> Self {
        Self {
            attestation_timeout: constants::DEFAULT_ATTESTATION_TIMEOUT,
            max_retries: u32::from(constants::DEFAULT_MAX_RETRIES),
            retry_delay: constants::DEFAULT_DEHYDRATION_RETRY_DELAY,
            include_vertices: false,
            include_payloads: false,
        }
    }
}

/// Slice configuration.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SliceConfig {
    /// Default maximum slice duration.
    #[serde(with = "duration_serde")]
    pub default_max_duration: Duration,

    /// Default loan grace period.
    #[serde(with = "duration_serde")]
    pub default_loan_grace: Duration,

    /// Allow re-slicing by default.
    pub allow_reslice: bool,

    /// Maximum re-slice depth.
    pub max_reslice_depth: u32,

    /// Maximum concurrent slices per session.
    pub max_slices_per_session: usize,
}

impl Default for SliceConfig {
    fn default() -> Self {
        Self {
            default_max_duration: constants::DEFAULT_SESSION_TIMEOUT,
            default_loan_grace: constants::DEFAULT_LOAN_GRACE,
            allow_reslice: false,
            max_reslice_depth: 3,
            max_slices_per_session: constants::DEFAULT_MAX_SLICES_PER_SESSION,
        }
    }
}

/// RPC server configuration.
///
/// ## Capability-based Discovery
///
/// Rather than hardcoding addresses, `RhizoCrypt` can discover its RPC endpoint
/// from the environment or use sensible defaults that work for local development.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RpcConfig {
    /// Host address to bind to.
    ///
    /// Defaults to localhost for security; use `0.0.0.0` for production.
    /// Can be set via `RHIZOCRYPT_RPC_HOST` environment variable.
    pub host: Cow<'static, str>,

    /// Port for tarpc RPC.
    ///
    /// Defaults to 0 (OS-assigned) for automatic port selection.
    /// Can be set via `RHIZOCRYPT_RPC_PORT` environment variable.
    pub port: u16,

    /// Enable RPC server.
    pub enabled: bool,

    /// Maximum concurrent connections.
    pub max_connections: usize,
}

impl Default for RpcConfig {
    fn default() -> Self {
        Self::from_env_or_default()
    }
}

impl RpcConfig {
    /// Default host for local development.
    const DEFAULT_HOST: &'static str = constants::DEFAULT_RPC_HOST;
    /// Default port (0 = OS-assigned for automatic selection).
    const DEFAULT_PORT: u16 = constants::DEFAULT_RPC_PORT;

    /// Create config from environment variables or use defaults.
    ///
    /// Environment variables:
    /// - `RHIZOCRYPT_RPC_HOST` — Host address
    /// - `RHIZOCRYPT_RPC_PORT` — Port number
    /// - `RHIZOCRYPT_RPC_ENABLED` — Enable RPC ("true"/"false")
    #[must_use]
    pub fn from_env_or_default() -> Self {
        Self::from_env_reader(|key| std::env::var(key))
    }

    /// Create config from an arbitrary environment reader (DI pattern).
    ///
    /// Absorbed from sweetGrass v0.7.15 `config_from_reader` pattern.
    /// Enables test isolation without `temp-env` or `unsafe` env mutation.
    #[must_use]
    pub fn from_env_reader<F>(reader: F) -> Self
    where
        F: Fn(&str) -> std::result::Result<String, std::env::VarError>,
    {
        let host = reader("RHIZOCRYPT_RPC_HOST")
            .map(Cow::Owned)
            .unwrap_or(Cow::Borrowed(Self::DEFAULT_HOST));

        let port = reader("RHIZOCRYPT_RPC_PORT")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(Self::DEFAULT_PORT);

        let enabled =
            reader("RHIZOCRYPT_RPC_ENABLED").map_or(true, |s| s.to_lowercase() != "false");

        Self {
            host,
            port,
            enabled,
            max_connections: constants::DEFAULT_MAX_CONNECTIONS,
        }
    }

    /// Create config with explicit host and port.
    #[must_use]
    pub fn with_addr(host: impl Into<Cow<'static, str>>, port: u16) -> Self {
        Self {
            host: host.into(),
            port,
            enabled: true,
            max_connections: constants::DEFAULT_MAX_CONNECTIONS,
        }
    }

    /// Create config for localhost with auto-assigned port.
    ///
    /// Useful for testing where port conflicts must be avoided.
    #[must_use]
    pub const fn localhost_auto() -> Self {
        Self {
            host: Cow::Borrowed(constants::LOCALHOST),
            port: 0,
            enabled: true,
            max_connections: constants::DEFAULT_MAX_CONNECTIONS,
        }
    }

    /// Get the socket address string.
    #[inline]
    #[must_use]
    pub fn socket_addr(&self) -> String {
        format!("{}:{}", self.host, self.port)
    }

    /// Parse the socket address.
    ///
    /// # Errors
    ///
    /// Returns an error if the address cannot be parsed.
    pub fn parse_addr(&self) -> Result<std::net::SocketAddr, std::net::AddrParseError> {
        self.socket_addr().parse()
    }

    /// Get the IP address.
    #[must_use]
    pub fn ip_addr(&self) -> IpAddr {
        self.host.parse().unwrap_or(IpAddr::V4(Ipv4Addr::LOCALHOST))
    }
}

/// Serde helper for Duration.
mod duration_serde {
    use serde::{Deserialize, Deserializer, Serialize, Serializer};
    use std::time::Duration;

    pub fn serialize<S>(duration: &Duration, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        duration.as_secs().serialize(serializer)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Duration, D::Error>
    where
        D: Deserializer<'de>,
    {
        let secs = u64::deserialize(deserializer)?;
        Ok(Duration::from_secs(secs))
    }
}

#[cfg(test)]
#[expect(clippy::unwrap_used, reason = "test code")]
#[path = "config_tests.rs"]
mod tests;
