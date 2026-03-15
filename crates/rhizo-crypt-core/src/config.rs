// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2024–2026 ecoPrimals Project

//! RhizoCrypt configuration.
//!
//! This module defines the configuration structure for the RhizoCrypt primal.
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

/// Configuration for RhizoCrypt.
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
            max_sessions: 1000,
            gc_interval: Duration::from_secs(60),
            expiration_grace: Duration::from_secs(3600),
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

    /// Path for persistent storage (Sled).
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

    /// Sled storage (100% Pure Rust, persistent, ACID, lock-free).
    Sled,

    /// LMDB storage (persistent, memory-mapped) - Future consideration.
    Lmdb,
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
            attestation_timeout: Duration::from_secs(60),
            max_retries: 3,
            retry_delay: Duration::from_secs(5),
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
            default_max_duration: Duration::from_secs(7 * 24 * 3600), // 1 week
            default_loan_grace: Duration::from_secs(24 * 3600),       // 1 day
            allow_reslice: false,
            max_reslice_depth: 3,
            max_slices_per_session: 100,
        }
    }
}

/// RPC server configuration.
///
/// ## Capability-based Discovery
///
/// Rather than hardcoding addresses, RhizoCrypt can discover its RPC endpoint
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
        let host = std::env::var("RHIZOCRYPT_RPC_HOST")
            .map(Cow::Owned)
            .unwrap_or(Cow::Borrowed(Self::DEFAULT_HOST));

        let port = std::env::var("RHIZOCRYPT_RPC_PORT")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(Self::DEFAULT_PORT);

        let enabled = std::env::var("RHIZOCRYPT_RPC_ENABLED")
            .map(|s| s.to_lowercase() != "false")
            .unwrap_or(true);

        Self {
            host,
            port,
            enabled,
            max_connections: 1000,
        }
    }

    /// Create config with explicit host and port.
    #[must_use]
    pub fn with_addr(host: impl Into<Cow<'static, str>>, port: u16) -> Self {
        Self {
            host: host.into(),
            port,
            enabled: true,
            max_connections: 1000,
        }
    }

    /// Create config for localhost with auto-assigned port.
    ///
    /// Useful for testing where port conflicts must be avoided.
    #[must_use]
    pub const fn localhost_auto() -> Self {
        Self {
            host: Cow::Borrowed("127.0.0.1"),
            port: 0,
            enabled: true,
            max_connections: 1000,
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
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;

    #[test]
    fn test_config_default() {
        let config = RhizoCryptConfig::default();
        assert_eq!(config.name, constants::PRIMAL_NAME);
        assert_eq!(config.max_sessions, 1000);
        assert_eq!(config.storage.backend, StorageBackend::Memory);
    }

    #[test]
    fn test_config_builder() {
        let config = RhizoCryptConfig::new("TestRhizo")
            .with_max_sessions(500)
            .with_gc_interval(Duration::from_secs(120));

        assert_eq!(config.name, "TestRhizo");
        assert_eq!(config.max_sessions, 500);
        assert_eq!(config.gc_interval, Duration::from_secs(120));
    }

    #[test]
    fn test_config_serialization() {
        let config = RhizoCryptConfig::default();
        let json = serde_json::to_string(&config).unwrap();
        let parsed: RhizoCryptConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(config.name, parsed.name);
    }

    #[test]
    fn test_storage_backend_default() {
        assert_eq!(StorageBackend::default(), StorageBackend::Memory);
    }

    #[test]
    fn test_config_with_all_options() {
        let config = RhizoCryptConfig::new("FullConfig")
            .with_max_sessions(2000)
            .with_gc_interval(Duration::from_secs(90))
            .with_storage(StorageConfig {
                backend: StorageBackend::Sled,
                path: Some("/tmp/rhizo".to_string()),
                max_memory_bytes: Some(2 * 1024 * 1024 * 1024),
            });

        assert_eq!(config.name, "FullConfig");
        assert_eq!(config.max_sessions, 2000);
        assert_eq!(config.gc_interval, Duration::from_secs(90));
        assert_eq!(config.storage.backend, StorageBackend::Sled);
        assert_eq!(config.storage.path.as_deref(), Some("/tmp/rhizo"));
        assert_eq!(config.storage.max_memory_bytes, Some(2 * 1024 * 1024 * 1024));
    }

    #[test]
    fn test_config_from_env() {
        std::env::set_var("RHIZOCRYPT_RPC_HOST", "0.0.0.0");
        std::env::set_var("RHIZOCRYPT_RPC_PORT", "9090");
        std::env::set_var("RHIZOCRYPT_RPC_ENABLED", "false");

        let rpc = RpcConfig::from_env_or_default();
        assert_eq!(rpc.host.as_ref(), "0.0.0.0");
        assert_eq!(rpc.port, 9090);
        assert!(!rpc.enabled);

        std::env::remove_var("RHIZOCRYPT_RPC_HOST");
        std::env::remove_var("RHIZOCRYPT_RPC_PORT");
        std::env::remove_var("RHIZOCRYPT_RPC_ENABLED");
    }

    #[test]
    fn test_storage_backend_variants() {
        assert_eq!(StorageBackend::Memory, StorageBackend::Memory);
        assert_eq!(StorageBackend::Sled, StorageBackend::Sled);
        assert_eq!(StorageBackend::Lmdb, StorageBackend::Lmdb);
        assert_ne!(StorageBackend::Memory, StorageBackend::Sled);
    }

    #[test]
    fn test_config_validation() {
        let config = RhizoCryptConfig::default();
        assert!(!config.name.is_empty());
        assert!(config.max_sessions > 0);
        assert!(config.gc_interval.as_secs() > 0);
    }

    #[test]
    fn test_config_clone() {
        let config = RhizoCryptConfig::new("CloneTest").with_max_sessions(100);
        let cloned = config.clone();
        assert_eq!(config.name, cloned.name);
        assert_eq!(config.max_sessions, cloned.max_sessions);
        assert_eq!(config.storage.backend, cloned.storage.backend);
    }
}
