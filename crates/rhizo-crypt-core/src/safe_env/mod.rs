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
        std::env::var(Self::ENV_MODE).map(|s| s.to_lowercase() == Self::DEV_MODE).unwrap_or(false)
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
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, unsafe_code)]
mod tests {
    use super::*;

    static ENV_LOCK: parking_lot::Mutex<()> = parking_lot::Mutex::new(());

    #[test]
    fn test_get_or_default() {
        let key = "RHIZOCRYPT_TEST_DEFAULT_123";
        let result = SafeEnv::get_or_default(key, "fallback");
        assert_eq!(result, "fallback");
    }

    #[test]
    fn test_parse_default() {
        let key = "RHIZOCRYPT_TEST_PARSE_123";
        let result: u16 = SafeEnv::parse(key, 8080);
        assert_eq!(result, 8080);
    }

    #[test]
    fn test_is_development_default() {
        let _guard = ENV_LOCK.lock();
        unsafe { std::env::remove_var("RHIZOCRYPT_ENV") };
        assert!(!SafeEnv::is_development());
        assert!(SafeEnv::is_production());
    }

    #[test]
    fn test_get_optional_missing() {
        let key = "RHIZOCRYPT_TEST_OPTIONAL_MISSING_123";
        let result = SafeEnv::get_optional(key);
        assert!(result.is_none());
    }

    #[test]
    fn test_get_capability_endpoint_format() {
        let capability = "crypto:signing";
        let normalized = capability.replace(':', "_").to_uppercase();
        assert_eq!(normalized, "CRYPTO_SIGNING");
    }

    #[test]
    fn test_get_or_default_with_value() {
        let _guard = ENV_LOCK.lock();
        let key = "RHIZOCRYPT_TEST_WITH_VALUE";
        unsafe { std::env::set_var(key, "custom_value") };
        let result = SafeEnv::get_or_default(key, "fallback");
        assert_eq!(result, "custom_value");
        unsafe { std::env::remove_var(key) };
    }

    #[test]
    fn test_get_optional_with_value() {
        let _guard = ENV_LOCK.lock();
        let key = "RHIZOCRYPT_TEST_OPTIONAL_WITH_VALUE";
        unsafe { std::env::set_var(key, "some_value") };
        let result = SafeEnv::get_optional(key);
        assert_eq!(result, Some("some_value".to_string()));
        unsafe { std::env::remove_var(key) };
    }

    #[test]
    fn test_parse_with_valid_value() {
        let _guard = ENV_LOCK.lock();
        let key = "RHIZOCRYPT_TEST_PARSE_VALID";
        unsafe { std::env::set_var(key, "9999") };
        let result: u16 = SafeEnv::parse(key, 8080);
        assert_eq!(result, 9999);
        unsafe { std::env::remove_var(key) };
    }

    #[test]
    fn test_parse_with_invalid_value() {
        let _guard = ENV_LOCK.lock();
        let key = "RHIZOCRYPT_TEST_PARSE_INVALID";
        unsafe { std::env::set_var(key, "not_a_number") };
        let result: u16 = SafeEnv::parse(key, 8080);
        assert_eq!(result, 8080, "Should use default on parse failure");
        unsafe { std::env::remove_var(key) };
    }

    #[test]
    fn test_parse_optional_with_valid() {
        let _guard = ENV_LOCK.lock();
        let key = "RHIZOCRYPT_TEST_PARSE_OPT_VALID";
        unsafe { std::env::set_var(key, "7777") };
        let result: Option<u16> = SafeEnv::parse_optional(key);
        assert_eq!(result, Some(7777));
        unsafe { std::env::remove_var(key) };
    }

    #[test]
    fn test_parse_optional_with_invalid() {
        let _guard = ENV_LOCK.lock();
        let key = "RHIZOCRYPT_TEST_PARSE_OPT_INVALID";
        unsafe { std::env::set_var(key, "invalid") };
        let result: Option<u16> = SafeEnv::parse_optional(key);
        assert_eq!(result, None);
        unsafe { std::env::remove_var(key) };
    }

    #[test]
    fn test_parse_optional_missing() {
        let key = "RHIZOCRYPT_TEST_PARSE_OPT_MISSING";
        let result: Option<u16> = SafeEnv::parse_optional(key);
        assert_eq!(result, None);
    }

    #[test]
    fn test_parse_empty_string() {
        let _guard = ENV_LOCK.lock();
        let key = "RHIZOCRYPT_TEST_PARSE_EMPTY";
        unsafe { std::env::set_var(key, "") };
        let result: u16 = SafeEnv::parse(key, 8080);
        assert_eq!(result, 8080, "Empty string should use default");
        unsafe { std::env::remove_var(key) };
    }

    #[test]
    fn test_parse_whitespace() {
        let _guard = ENV_LOCK.lock();
        let key = "RHIZOCRYPT_TEST_PARSE_WHITESPACE";
        unsafe { std::env::set_var(key, "   ") };
        let result: u16 = SafeEnv::parse(key, 8080);
        assert_eq!(result, 8080, "Whitespace-only should use default");
        unsafe { std::env::remove_var(key) };
    }

    #[test]
    fn test_parse_optional_empty_string() {
        let _guard = ENV_LOCK.lock();
        let key = "RHIZOCRYPT_TEST_PARSE_OPT_EMPTY";
        unsafe { std::env::set_var(key, "") };
        let result: Option<u16> = SafeEnv::parse_optional(key);
        assert_eq!(result, None);
        unsafe { std::env::remove_var(key) };
    }

    #[test]
    fn test_parse_bool_type() {
        let _guard = ENV_LOCK.lock();
        let key = "RHIZOCRYPT_TEST_PARSE_BOOL";
        unsafe { std::env::set_var(key, "true") };
        let result: bool = SafeEnv::parse(key, false);
        assert!(result);
        unsafe { std::env::remove_var(key) };
    }

    #[test]
    fn test_get_discovery_address_rhizocrypt_adapter() {
        let _guard = ENV_LOCK.lock();
        unsafe { std::env::set_var("RHIZOCRYPT_DISCOVERY_ADAPTER", "adapter.example.com:7500") };
        let result = SafeEnv::get_discovery_address();
        assert_eq!(result, Some("adapter.example.com:7500".to_string()));
        unsafe { std::env::remove_var("RHIZOCRYPT_DISCOVERY_ADAPTER") };
    }

    #[test]
    fn test_get_discovery_address_priority() {
        let _guard = ENV_LOCK.lock();
        unsafe { std::env::set_var("RHIZOCRYPT_DISCOVERY_ADAPTER", "primary.example.com:7500") };
        unsafe { std::env::set_var("DISCOVERY_ENDPOINT", "fallback.example.com:8091") };
        let result = SafeEnv::get_discovery_address();
        assert_eq!(result, Some("primary.example.com:7500".to_string()));
        unsafe { std::env::remove_var("RHIZOCRYPT_DISCOVERY_ADAPTER") };
        unsafe { std::env::remove_var("DISCOVERY_ENDPOINT") };
    }

    #[test]
    fn test_get_rpc_port_legacy() {
        let _guard = ENV_LOCK.lock();
        unsafe { std::env::remove_var("RHIZOCRYPT_RPC_PORT") };
        unsafe { std::env::set_var("RHIZOCRYPT_PORT", "8888") };
        let result = SafeEnv::get_rpc_port(1000);
        assert_eq!(result, 8888);
        unsafe { std::env::remove_var("RHIZOCRYPT_PORT") };
    }

    #[test]
    fn test_get_rpc_port_preferred_takes_priority() {
        let _guard = ENV_LOCK.lock();
        unsafe { std::env::set_var("RHIZOCRYPT_RPC_PORT", "9500") };
        unsafe { std::env::set_var("RHIZOCRYPT_PORT", "9400") };
        let result = SafeEnv::get_rpc_port(9000);
        assert_eq!(result, 9500);
        unsafe { std::env::remove_var("RHIZOCRYPT_RPC_PORT") };
        unsafe { std::env::remove_var("RHIZOCRYPT_PORT") };
    }

    #[test]
    fn test_get_capability_endpoint_with_address_suffix() {
        let _guard = ENV_LOCK.lock();
        unsafe { std::env::set_var("CRYPTO_SIGNING_ADDRESS", "signing-addr.example.com:9500") };
        let result = SafeEnv::get_capability_endpoint("crypto:signing");
        assert_eq!(result, Some("signing-addr.example.com:9500".to_string()));
        unsafe { std::env::remove_var("CRYPTO_SIGNING_ADDRESS") };
    }

    #[test]
    fn test_is_development_true() {
        let _guard = ENV_LOCK.lock();
        unsafe { std::env::set_var("RHIZOCRYPT_ENV", "development") };
        assert!(SafeEnv::is_development());
        assert!(!SafeEnv::is_production());
        unsafe { std::env::remove_var("RHIZOCRYPT_ENV") };
    }

    #[test]
    fn test_is_development_case_insensitive() {
        let _guard = ENV_LOCK.lock();
        let original = std::env::var("RHIZOCRYPT_ENV").ok();

        unsafe { std::env::set_var("RHIZOCRYPT_ENV", "DEVELOPMENT") };
        assert!(SafeEnv::is_development());

        unsafe { std::env::set_var("RHIZOCRYPT_ENV", "Development") };
        assert!(SafeEnv::is_development());

        match original {
            Some(val) => unsafe { std::env::set_var("RHIZOCRYPT_ENV", val) },
            None => unsafe { std::env::remove_var("RHIZOCRYPT_ENV") },
        }
    }

    #[test]
    fn test_is_production_default() {
        let _guard = ENV_LOCK.lock();
        unsafe { std::env::remove_var("RHIZOCRYPT_ENV") };
        assert!(SafeEnv::is_production());
        assert!(!SafeEnv::is_development());
    }

    #[test]
    fn test_is_production_explicit() {
        let _guard = ENV_LOCK.lock();
        unsafe { std::env::set_var("RHIZOCRYPT_ENV", "production") };
        assert!(SafeEnv::is_production());
        assert!(!SafeEnv::is_development());
        unsafe { std::env::remove_var("RHIZOCRYPT_ENV") };
    }

    #[test]
    fn test_get_endpoint_with_endpoint_suffix() {
        let _guard = ENV_LOCK.lock();
        let prefix = "TEST_SERVICE";
        unsafe { std::env::set_var("TEST_SERVICE_ENDPOINT", "service.example.com:9000") };
        let result = SafeEnv::get_endpoint(prefix);
        assert_eq!(result, Some("service.example.com:9000".to_string()));
        unsafe { std::env::remove_var("TEST_SERVICE_ENDPOINT") };
    }

    #[test]
    fn test_get_endpoint_with_address_suffix() {
        let _guard = ENV_LOCK.lock();
        let prefix = "TEST_SERVICE";
        unsafe { std::env::set_var("TEST_SERVICE_ADDRESS", "service.example.com:9001") };
        let result = SafeEnv::get_endpoint(prefix);
        assert_eq!(result, Some("service.example.com:9001".to_string()));
        unsafe { std::env::remove_var("TEST_SERVICE_ADDRESS") };
    }

    #[test]
    fn test_get_endpoint_priority_endpoint_over_address() {
        let _guard = ENV_LOCK.lock();
        let prefix = "TEST_SERVICE";
        unsafe { std::env::set_var("TEST_SERVICE_ENDPOINT", "endpoint.example.com:9000") };
        unsafe { std::env::set_var("TEST_SERVICE_ADDRESS", "address.example.com:9001") };
        let result = SafeEnv::get_endpoint(prefix);
        assert_eq!(result, Some("endpoint.example.com:9000".to_string()));
        unsafe { std::env::remove_var("TEST_SERVICE_ENDPOINT") };
        unsafe { std::env::remove_var("TEST_SERVICE_ADDRESS") };
    }

    #[test]
    fn test_get_endpoint_missing() {
        let prefix = "NONEXISTENT_SERVICE";
        let result = SafeEnv::get_endpoint(prefix);
        assert_eq!(result, None);
    }

    #[test]
    fn test_get_capability_endpoint() {
        let _guard = ENV_LOCK.lock();
        unsafe { std::env::set_var("CRYPTO_SIGNING_ENDPOINT", "signing.example.com:9500") };
        let result = SafeEnv::get_capability_endpoint("crypto:signing");
        assert_eq!(result, Some("signing.example.com:9500".to_string()));
        unsafe { std::env::remove_var("CRYPTO_SIGNING_ENDPOINT") };
    }

    #[test]
    fn test_get_discovery_address() {
        let _guard = ENV_LOCK.lock();
        unsafe { std::env::set_var("DISCOVERY_ENDPOINT", "discovery.example.com:8091") };
        let result = SafeEnv::get_discovery_address();
        assert_eq!(result, Some("discovery.example.com:8091".to_string()));
        unsafe { std::env::remove_var("DISCOVERY_ENDPOINT") };
    }

    #[test]
    fn test_get_rpc_port_default() {
        let _guard = ENV_LOCK.lock();
        unsafe { std::env::remove_var("RHIZOCRYPT_RPC_PORT") };
        let result = SafeEnv::get_rpc_port(9400);
        assert_eq!(result, 9400);
    }

    #[test]
    fn test_get_rpc_port_custom() {
        let _guard = ENV_LOCK.lock();
        unsafe { std::env::set_var("RHIZOCRYPT_RPC_PORT", "9999") };
        let result = SafeEnv::get_rpc_port(9400);
        assert_eq!(result, 9999);
        unsafe { std::env::remove_var("RHIZOCRYPT_RPC_PORT") };
    }

    #[test]
    fn test_get_rpc_host_default() {
        let _guard = ENV_LOCK.lock();
        unsafe { std::env::remove_var("RHIZOCRYPT_RPC_HOST") };
        unsafe { std::env::remove_var("RHIZOCRYPT_HOST") };
        let result = SafeEnv::get_rpc_host();
        assert_eq!(result, "0.0.0.0");
    }

    #[test]
    fn test_get_rpc_host_custom() {
        let _guard = ENV_LOCK.lock();
        unsafe { std::env::set_var("RHIZOCRYPT_RPC_HOST", "127.0.0.1") };
        let result = SafeEnv::get_rpc_host();
        assert_eq!(result, "127.0.0.1");
        unsafe { std::env::remove_var("RHIZOCRYPT_RPC_HOST") };
    }

    #[test]
    fn test_get_rpc_host_legacy_fallback() {
        let _guard = ENV_LOCK.lock();
        unsafe { std::env::remove_var("RHIZOCRYPT_RPC_HOST") };
        unsafe { std::env::set_var("RHIZOCRYPT_HOST", "10.0.0.1") };
        let result = SafeEnv::get_rpc_host();
        assert_eq!(result, "10.0.0.1");
        unsafe { std::env::remove_var("RHIZOCRYPT_HOST") };
    }

    #[test]
    fn test_get_metrics_port_default() {
        let _guard = ENV_LOCK.lock();
        unsafe { std::env::remove_var("RHIZOCRYPT_METRICS_PORT") };
        let result = SafeEnv::get_metrics_port(9401);
        assert_eq!(result, 9401);
    }

    #[test]
    fn test_get_metrics_port_custom() {
        let _guard = ENV_LOCK.lock();
        unsafe { std::env::set_var("RHIZOCRYPT_METRICS_PORT", "8888") };
        let result = SafeEnv::get_metrics_port(9401);
        assert_eq!(result, 8888);
        unsafe { std::env::remove_var("RHIZOCRYPT_METRICS_PORT") };
    }
}
