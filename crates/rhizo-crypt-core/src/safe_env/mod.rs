// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2024–2026 ecoPrimals Project

//! Safe environment variable access for infant discovery.
//!
//! This module provides safe, consistent access to environment variables
//! following the ecoPrimals zero-hardcoding principle.
//!
//! ## Philosophy
//!
//! - **Primals start with zero knowledge** — No hardcoded addresses, ports, or names
//! - **Environment variables provide hints** — Not requirements
//! - **Graceful fallback** — Missing variables don't crash the system
//! - **Type-safe parsing** — Clear error handling for malformed values
//!
//! ## Usage
//!
//! ```rust
//! use rhizo_crypt_core::safe_env::SafeEnv;
//!
//! // Get with default
//! let port: u16 = SafeEnv::parse("RHIZOCRYPT_PORT", 9400);
//!
//! // Get string with default
//! let host = SafeEnv::get_or_default("RHIZOCRYPT_HOST", "0.0.0.0");
//!
//! // Check if in development mode
//! if SafeEnv::is_development() {
//!     // Enable development fallbacks
//! }
//! ```

mod capability;

pub use capability::CapabilityEnv;

use std::str::FromStr;

/// Safe environment variable access.
///
/// Provides type-safe access to environment variables with sensible defaults.
/// Never panics on missing or malformed variables.
pub struct SafeEnv;

impl SafeEnv {
    /// Environment variable for environment mode.
    const ENV_MODE: &'static str = "RHIZOCRYPT_ENV";

    /// Development mode value.
    const DEV_MODE: &'static str = "development";

    /// Get an environment variable or return the default.
    #[inline]
    #[must_use]
    pub fn get_or_default(key: &str, default: &str) -> String {
        std::env::var(key).unwrap_or_else(|_| default.to_string())
    }

    /// Get an optional environment variable.
    #[inline]
    #[must_use]
    pub fn get_optional(key: &str) -> Option<String> {
        std::env::var(key).ok()
    }

    /// Parse an environment variable or return the default.
    ///
    /// If the variable is set but cannot be parsed, logs a warning and returns the default.
    #[must_use]
    pub fn parse<T>(key: &str, default: T) -> T
    where
        T: FromStr,
    {
        std::env::var(key)
            .ok()
            .and_then(|s| {
                s.parse().ok().or_else(|| {
                    tracing::warn!(key, value = %s, "Failed to parse environment variable, using default");
                    None
                })
            })
            .unwrap_or(default)
    }

    /// Parse an optional environment variable.
    ///
    /// Returns `None` if the variable is not set or cannot be parsed.
    #[must_use]
    pub fn parse_optional<T>(key: &str) -> Option<T>
    where
        T: FromStr,
    {
        std::env::var(key).ok().and_then(|s| s.parse().ok())
    }

    /// Check if running in development mode.
    ///
    /// Development mode is enabled when `RHIZOCRYPT_ENV=development`.
    /// In development mode, fallback addresses may be used.
    #[inline]
    #[must_use]
    pub fn is_development() -> bool {
        std::env::var(Self::ENV_MODE).is_ok_and(|s| s.to_lowercase() == Self::DEV_MODE)
    }

    /// Check if running in production mode.
    ///
    /// Production mode is the default when `RHIZOCRYPT_ENV` is not set to "development".
    #[inline]
    #[must_use]
    pub fn is_production() -> bool {
        !Self::is_development()
    }

    /// Get an endpoint address from environment.
    ///
    /// Checks `{PREFIX}_ENDPOINT` and `{PREFIX}_ADDRESS` variants.
    /// Returns `None` if neither is set.
    #[must_use]
    pub fn get_endpoint(prefix: &str) -> Option<String> {
        let key_endpoint = format!("{prefix}_ENDPOINT");
        let key_address = format!("{prefix}_ADDRESS");

        std::env::var(&key_endpoint).ok().or_else(|| std::env::var(&key_address).ok())
    }

    /// Get a capability endpoint from environment.
    ///
    /// Uses the capability string (e.g., "crypto:signing") to construct
    /// an environment variable name by replacing `:` with `_` and uppercasing.
    ///
    /// Example: "crypto:signing" → `CRYPTO_SIGNING_ENDPOINT`
    #[must_use]
    pub fn get_capability_endpoint(capability: &str) -> Option<String> {
        let normalized = capability.replace(':', "_").to_uppercase();
        Self::get_endpoint(&normalized)
    }

    /// Get the discovery service address.
    ///
    /// This is the ONLY address that may be hardcoded (as a last resort).
    /// All other services are discovered through this endpoint.
    ///
    /// Priority order:
    /// 1. `RHIZOCRYPT_DISCOVERY_ADAPTER` (recommended, new standard)
    /// 2. `DISCOVERY_ENDPOINT` (capability-based)
    /// 3. `DISCOVERY_ADDRESS` (capability-based)
    /// 4. None (infant discovery: primal starts with zero knowledge)
    #[must_use]
    pub fn get_discovery_address() -> Option<String> {
        std::env::var("RHIZOCRYPT_DISCOVERY_ADAPTER")
            .ok()
            .or_else(|| Self::get_endpoint("DISCOVERY"))
    }

    /// Get the RPC port, with environment override.
    ///
    /// Priority order:
    /// 1. `RHIZOCRYPT_RPC_PORT` (preferred)
    /// 2. `RHIZOCRYPT_PORT` (legacy, backward-compatible)
    /// 3. `default` parameter
    #[must_use]
    pub fn get_rpc_port(default: u16) -> u16 {
        Self::parse_optional::<u16>("RHIZOCRYPT_RPC_PORT")
            .or_else(|| Self::parse_optional::<u16>("RHIZOCRYPT_PORT"))
            .unwrap_or(default)
    }

    /// Get the JSON-RPC port, with environment override.
    ///
    /// Priority order:
    /// 1. `RHIZOCRYPT_JSONRPC_PORT` (explicit override)
    /// 2. Calculated from `tarpc_port + JSONRPC_PORT_OFFSET`
    ///
    /// When the tarpc port is 0 (OS-assigned), returns 0 so the OS also
    /// assigns the JSON-RPC port independently.
    #[must_use]
    pub fn get_jsonrpc_port(tarpc_port: u16) -> u16 {
        Self::parse_optional::<u16>("RHIZOCRYPT_JSONRPC_PORT").unwrap_or_else(|| {
            if tarpc_port == 0 {
                0
            } else {
                tarpc_port.saturating_add(crate::constants::JSONRPC_PORT_OFFSET)
            }
        })
    }

    /// Get the RPC host, with environment override.
    ///
    /// Priority order:
    /// 1. `RHIZOCRYPT_RPC_HOST` (preferred)
    /// 2. `RHIZOCRYPT_HOST` (legacy, backward-compatible)
    /// 3. Production bind address (all interfaces)
    #[must_use]
    pub fn get_rpc_host() -> String {
        Self::get_optional("RHIZOCRYPT_RPC_HOST")
            .or_else(|| Self::get_optional("RHIZOCRYPT_HOST"))
            .unwrap_or_else(|| crate::constants::PRODUCTION_BIND_ADDRESS.to_string())
    }

    /// Get the metrics port, with environment override.
    #[must_use]
    pub fn get_metrics_port(default: u16) -> u16 {
        Self::parse("RHIZOCRYPT_METRICS_PORT", default)
    }

    /// Construct the canonical socket env var name for any primal.
    ///
    /// Absorbed from sweetGrass V0717 generic helper pattern. Avoids
    /// per-primal constant proliferation — any primal can be discovered
    /// via `{UPPER_NAME}_SOCKET` at runtime.
    ///
    /// # Example
    ///
    /// ```
    /// use rhizo_crypt_core::safe_env::SafeEnv;
    /// assert_eq!(SafeEnv::socket_env_var("rhizoCrypt"), "RHIZOCRYPT_SOCKET");
    /// assert_eq!(SafeEnv::socket_env_var("loamSpine"), "LOAMSPINE_SOCKET");
    /// ```
    #[must_use]
    pub fn socket_env_var(primal_name: &str) -> String {
        format!("{}_SOCKET", primal_name.to_uppercase())
    }

    /// Construct the canonical address env var name for any primal.
    ///
    /// Absorbed from sweetGrass V0717 generic helper pattern.
    ///
    /// # Example
    ///
    /// ```
    /// use rhizo_crypt_core::safe_env::SafeEnv;
    /// assert_eq!(SafeEnv::address_env_var("bearDog"), "BEARDOG_ADDRESS");
    /// ```
    #[must_use]
    pub fn address_env_var(primal_name: &str) -> String {
        format!("{}_ADDRESS", primal_name.to_uppercase())
    }

    /// Get the socket path for a primal by name.
    ///
    /// Looks up `{PRIMAL}_SOCKET` env var and falls back to the XDG
    /// runtime directory convention: `$XDG_RUNTIME_DIR/biomeos/{name}.sock`.
    #[must_use]
    pub fn get_socket_path(primal_name: &str) -> Option<std::path::PathBuf> {
        let env_key = Self::socket_env_var(primal_name);
        if let Some(path) = Self::get_optional(&env_key) {
            return Some(std::path::PathBuf::from(path));
        }

        // XDG fallback
        std::env::var("XDG_RUNTIME_DIR").ok().map(|xdg| {
            std::path::PathBuf::from(xdg)
                .join(crate::constants::BIOMEOS_SOCKET_SUBDIR)
                .join(format!("{primal_name}.sock"))
        })
    }
}

#[cfg(test)]
#[path = "tests.rs"]
mod tests;
