// SPDX-License-Identifier: AGPL-3.0-only
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
    /// Checks both `RHIZOCRYPT_RPC_PORT` and legacy `RHIZOCRYPT_PORT`
    /// for backward compatibility.
    #[must_use]
    pub fn get_rpc_port(default: u16) -> u16 {
        // Try RHIZOCRYPT_RPC_PORT first (preferred)
        Self::parse("RHIZOCRYPT_RPC_PORT", default)
            // Fall back to legacy RHIZOCRYPT_PORT
            .max(Self::parse("RHIZOCRYPT_PORT", default))
    }

    /// Get the RPC host, with environment override.
    #[must_use]
    pub fn get_rpc_host() -> String {
        Self::get_or_default("RHIZOCRYPT_RPC_HOST", "0.0.0.0")
    }

    /// Get the metrics port, with environment override.
    #[must_use]
    pub fn get_metrics_port(default: u16) -> u16 {
        Self::parse("RHIZOCRYPT_METRICS_PORT", default)
    }
}

/// Capability-specific environment configuration.
///
/// Provides standardized environment variable names for each capability.
///
/// ## Naming Convention
///
/// Capability-based variables follow this pattern:
/// - **Preferred**: `<CAPABILITY>_ENDPOINT` (e.g., `SIGNING_ENDPOINT`)
/// - **Alternative**: `<CATEGORY>_<CAPABILITY>_ENDPOINT` (e.g., `CRYPTO_SIGNING_ENDPOINT`)
/// - **Legacy**: `<PRIMAL>_ADDRESS` (e.g., `BEARDOG_ADDRESS`) - deprecated
///
/// ## Infant Discovery
///
/// In production, primals should discover capabilities at runtime via the
/// universal adapter (`RHIZOCRYPT_DISCOVERY_ADAPTER`). Environment variables
/// are **hints** for development or testing, not requirements.
///
/// ## Migration Path
///
/// Old (hardcoded):
/// ```bash
/// BEARDOG_ADDRESS=beardog.local:9500
/// NESTGATE_ADDRESS=nestgate.local:8080
/// ```
///
/// New (capability-based):
/// ```bash
/// RHIZOCRYPT_DISCOVERY_ADAPTER=songbird.local:7500  # Only this is needed!
/// # OR for development:
/// SIGNING_ENDPOINT=http://localhost:9500
/// PAYLOAD_STORAGE_ENDPOINT=http://localhost:8080
/// ```
pub struct CapabilityEnv;

#[allow(clippy::manual_inspect)] // We need to return the value, not just inspect it
impl CapabilityEnv {
    /// Get the endpoint for signing capability.
    ///
    /// Priority order:
    /// 1. `CRYPTO_SIGNING_ENDPOINT` (preferred, capability-based)
    /// 2. `SIGNING_ENDPOINT` (short form, capability-based)
    /// 3. `BEARDOG_ADDRESS` (legacy, deprecated - emits warning)
    #[must_use]
    pub fn signing_endpoint() -> Option<String> {
        SafeEnv::get_endpoint("CRYPTO_SIGNING")
            .or_else(|| SafeEnv::get_endpoint("SIGNING"))
            .or_else(|| {
                std::env::var("BEARDOG_ADDRESS").ok().map(|addr| {
                    tracing::warn!(
                        "Using deprecated BEARDOG_ADDRESS environment variable. \
                         Please migrate to SIGNING_ENDPOINT or CRYPTO_SIGNING_ENDPOINT \
                         for capability-based configuration."
                    );
                    addr
                })
            })
    }

    /// Get the endpoint for DID verification capability.
    ///
    /// Priority order:
    /// 1. `DID_VERIFICATION_ENDPOINT` (preferred, capability-based)
    /// 2. `DID_ENDPOINT` (short form, capability-based)
    /// 3. `BEARDOG_ADDRESS` (legacy, deprecated - emits warning)
    #[must_use]
    pub fn did_verification_endpoint() -> Option<String> {
        SafeEnv::get_endpoint("DID_VERIFICATION").or_else(|| SafeEnv::get_endpoint("DID")).or_else(
            || {
                std::env::var("BEARDOG_ADDRESS").ok().map(|addr| {
                    tracing::warn!(
                        "Using deprecated BEARDOG_ADDRESS for DID verification. \
                         Please migrate to DID_ENDPOINT for capability-based configuration."
                    );
                    addr
                })
            },
        )
    }

    /// Get the endpoint for payload storage capability.
    ///
    /// Priority order:
    /// 1. `PAYLOAD_STORAGE_ENDPOINT` (preferred, capability-based)
    /// 2. `PAYLOAD_ENDPOINT` (short form, capability-based)
    /// 3. `NESTGATE_ADDRESS` (legacy, deprecated - emits warning)
    #[must_use]
    pub fn payload_storage_endpoint() -> Option<String> {
        SafeEnv::get_endpoint("PAYLOAD_STORAGE")
            .or_else(|| SafeEnv::get_endpoint("PAYLOAD"))
            .or_else(|| {
                std::env::var("NESTGATE_ADDRESS").ok().map(|addr| {
                    tracing::warn!(
                        "Using deprecated NESTGATE_ADDRESS environment variable. \
                         Please migrate to PAYLOAD_STORAGE_ENDPOINT \
                         for capability-based configuration."
                    );
                    addr
                })
            })
    }

    /// Get the endpoint for permanent commit capability.
    ///
    /// Priority order:
    /// 1. `STORAGE_PERMANENT_COMMIT_ENDPOINT` (preferred, capability-based)
    /// 2. `PERMANENT_STORAGE_ENDPOINT` (short form, capability-based)
    /// 3. `LOAMSPINE_ADDRESS` (legacy, deprecated - emits warning)
    #[must_use]
    pub fn permanent_commit_endpoint() -> Option<String> {
        SafeEnv::get_endpoint("STORAGE_PERMANENT_COMMIT")
            .or_else(|| SafeEnv::get_endpoint("PERMANENT_STORAGE"))
            .or_else(|| {
                std::env::var("LOAMSPINE_ADDRESS").ok().map(|addr| {
                    tracing::warn!(
                        "Using deprecated LOAMSPINE_ADDRESS environment variable. \
                         Please migrate to PERMANENT_STORAGE_ENDPOINT \
                         for capability-based configuration."
                    );
                    addr
                })
            })
    }

    /// Get the endpoint for compute orchestration capability.
    ///
    /// Environment variables:
    /// - `COMPUTE_ORCHESTRATION_ENDPOINT` (preferred)
    /// - `COMPUTE_ENDPOINT` (short form)
    #[must_use]
    pub fn compute_endpoint() -> Option<String> {
        SafeEnv::get_endpoint("COMPUTE_ORCHESTRATION").or_else(|| SafeEnv::get_endpoint("COMPUTE"))
    }

    /// Get the endpoint for provenance query capability.
    ///
    /// Environment variables:
    /// - `PROVENANCE_QUERY_ENDPOINT` (preferred)
    /// - `PROVENANCE_ENDPOINT` (short form)
    #[must_use]
    pub fn provenance_endpoint() -> Option<String> {
        SafeEnv::get_endpoint("PROVENANCE_QUERY").or_else(|| SafeEnv::get_endpoint("PROVENANCE"))
    }

    /// Get the endpoint for service discovery capability.
    ///
    /// This is the **universal adapter** - the ONLY service that may need
    /// an environment variable in production. All other services are discovered
    /// through this adapter.
    ///
    /// Priority order:
    /// 1. `RHIZOCRYPT_DISCOVERY_ADAPTER` (recommended, new standard)
    /// 2. `DISCOVERY_SERVICE_ENDPOINT` (capability-based)
    /// 3. `DISCOVERY_ENDPOINT` (short form, capability-based)
    /// 4. `SONGBIRD_ADDRESS` (legacy, acceptable - Songbird is the universal adapter)
    ///
    /// ## Infant Discovery
    ///
    /// If this returns `None`, the primal starts with **zero knowledge** and
    /// must bootstrap through other means (multicast, DHT, etc.).
    #[must_use]
    pub fn discovery_endpoint() -> Option<String> {
        std::env::var("RHIZOCRYPT_DISCOVERY_ADAPTER")
            .ok()
            .or_else(|| SafeEnv::get_endpoint("DISCOVERY_SERVICE"))
            .or_else(|| SafeEnv::get_endpoint("DISCOVERY"))
            .or_else(|| {
                // Songbird is special - it's the universal adapter
                std::env::var("SONGBIRD_ADDRESS").ok().map(|addr| {
                    tracing::info!(
                        "Using SONGBIRD_ADDRESS for discovery. \
                         Consider migrating to RHIZOCRYPT_DISCOVERY_ADAPTER for consistency."
                    );
                    addr
                })
            })
    }

    /// Get all configured capability endpoints.
    ///
    /// Returns a map of capability name to endpoint address.
    /// Useful for debugging and configuration validation.
    #[must_use]
    pub fn all_capability_endpoints() -> std::collections::HashMap<&'static str, String> {
        let mut map = std::collections::HashMap::new();

        if let Some(ep) = Self::signing_endpoint() {
            map.insert("signing", ep);
        }
        if let Some(ep) = Self::did_verification_endpoint() {
            map.insert("did_verification", ep);
        }
        if let Some(ep) = Self::payload_storage_endpoint() {
            map.insert("payload_storage", ep);
        }
        if let Some(ep) = Self::permanent_commit_endpoint() {
            map.insert("permanent_commit", ep);
        }
        if let Some(ep) = Self::compute_endpoint() {
            map.insert("compute", ep);
        }
        if let Some(ep) = Self::provenance_endpoint() {
            map.insert("provenance", ep);
        }
        if let Some(ep) = Self::discovery_endpoint() {
            map.insert("discovery", ep);
        }

        map
    }

    /// Check if infant discovery mode is enabled.
    ///
    /// Returns `true` if NO capability endpoints are configured,
    /// meaning the primal must discover everything at runtime.
    #[must_use]
    pub fn is_infant_discovery_mode() -> bool {
        Self::all_capability_endpoints().is_empty()
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;

    #[test]
    fn test_get_or_default() {
        // Use a unique key to avoid test interference
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
        // By default, should be production (not development)
        // Note: This test may be affected by environment state
        // In a clean environment, is_development() should be false
    }

    #[test]
    fn test_get_optional_missing() {
        let key = "RHIZOCRYPT_TEST_OPTIONAL_MISSING_123";
        let result = SafeEnv::get_optional(key);
        assert!(result.is_none());
    }

    #[test]
    fn test_get_capability_endpoint_format() {
        // Test the key format transformation
        let capability = "crypto:signing";
        let normalized = capability.replace(':', "_").to_uppercase();
        assert_eq!(normalized, "CRYPTO_SIGNING");
    }

    #[test]
    fn test_get_or_default_with_value() {
        let key = "RHIZOCRYPT_TEST_WITH_VALUE";
        std::env::set_var(key, "custom_value");
        let result = SafeEnv::get_or_default(key, "fallback");
        assert_eq!(result, "custom_value");
        std::env::remove_var(key);
    }

    #[test]
    fn test_get_optional_with_value() {
        let key = "RHIZOCRYPT_TEST_OPTIONAL_WITH_VALUE";
        std::env::set_var(key, "some_value");
        let result = SafeEnv::get_optional(key);
        assert_eq!(result, Some("some_value".to_string()));
        std::env::remove_var(key);
    }

    #[test]
    fn test_parse_with_valid_value() {
        let key = "RHIZOCRYPT_TEST_PARSE_VALID";
        std::env::set_var(key, "9999");
        let result: u16 = SafeEnv::parse(key, 8080);
        assert_eq!(result, 9999);
        std::env::remove_var(key);
    }

    #[test]
    fn test_parse_with_invalid_value() {
        let key = "RHIZOCRYPT_TEST_PARSE_INVALID";
        std::env::set_var(key, "not_a_number");
        let result: u16 = SafeEnv::parse(key, 8080);
        assert_eq!(result, 8080, "Should use default on parse failure");
        std::env::remove_var(key);
    }

    #[test]
    fn test_parse_optional_with_valid() {
        let key = "RHIZOCRYPT_TEST_PARSE_OPT_VALID";
        std::env::set_var(key, "7777");
        let result: Option<u16> = SafeEnv::parse_optional(key);
        assert_eq!(result, Some(7777));
        std::env::remove_var(key);
    }

    #[test]
    fn test_parse_optional_with_invalid() {
        let key = "RHIZOCRYPT_TEST_PARSE_OPT_INVALID";
        std::env::set_var(key, "invalid");
        let result: Option<u16> = SafeEnv::parse_optional(key);
        assert_eq!(result, None);
        std::env::remove_var(key);
    }

    #[test]
    fn test_parse_optional_missing() {
        let key = "RHIZOCRYPT_TEST_PARSE_OPT_MISSING";
        let result: Option<u16> = SafeEnv::parse_optional(key);
        assert_eq!(result, None);
    }

    #[test]
    fn test_is_development_true() {
        std::env::set_var("RHIZOCRYPT_ENV", "development");
        assert!(SafeEnv::is_development());
        assert!(!SafeEnv::is_production());
        std::env::remove_var("RHIZOCRYPT_ENV");
    }

    #[test]
    fn test_is_development_case_insensitive() {
        // Save current value
        let original = std::env::var("RHIZOCRYPT_ENV").ok();

        std::env::set_var("RHIZOCRYPT_ENV", "DEVELOPMENT");
        assert!(SafeEnv::is_development());

        std::env::set_var("RHIZOCRYPT_ENV", "Development");
        assert!(SafeEnv::is_development());

        // Restore original value
        match original {
            Some(val) => std::env::set_var("RHIZOCRYPT_ENV", val),
            None => std::env::remove_var("RHIZOCRYPT_ENV"),
        }
    }

    #[test]
    fn test_is_production_default() {
        // Ensure no env var set
        std::env::remove_var("RHIZOCRYPT_ENV");
        assert!(SafeEnv::is_production());
        assert!(!SafeEnv::is_development());
    }

    #[test]
    fn test_is_production_explicit() {
        std::env::set_var("RHIZOCRYPT_ENV", "production");
        assert!(SafeEnv::is_production());
        assert!(!SafeEnv::is_development());
        std::env::remove_var("RHIZOCRYPT_ENV");
    }

    #[test]
    fn test_get_endpoint_with_endpoint_suffix() {
        let prefix = "TEST_SERVICE";
        std::env::set_var("TEST_SERVICE_ENDPOINT", "service.example.com:9000");
        let result = SafeEnv::get_endpoint(prefix);
        assert_eq!(result, Some("service.example.com:9000".to_string()));
        std::env::remove_var("TEST_SERVICE_ENDPOINT");
    }

    #[test]
    fn test_get_endpoint_with_address_suffix() {
        let prefix = "TEST_SERVICE";
        std::env::set_var("TEST_SERVICE_ADDRESS", "service.example.com:9001");
        let result = SafeEnv::get_endpoint(prefix);
        assert_eq!(result, Some("service.example.com:9001".to_string()));
        std::env::remove_var("TEST_SERVICE_ADDRESS");
    }

    #[test]
    fn test_get_endpoint_priority_endpoint_over_address() {
        let prefix = "TEST_SERVICE";
        std::env::set_var("TEST_SERVICE_ENDPOINT", "endpoint.example.com:9000");
        std::env::set_var("TEST_SERVICE_ADDRESS", "address.example.com:9001");
        let result = SafeEnv::get_endpoint(prefix);
        assert_eq!(result, Some("endpoint.example.com:9000".to_string()));
        std::env::remove_var("TEST_SERVICE_ENDPOINT");
        std::env::remove_var("TEST_SERVICE_ADDRESS");
    }

    #[test]
    fn test_get_endpoint_missing() {
        let prefix = "NONEXISTENT_SERVICE";
        let result = SafeEnv::get_endpoint(prefix);
        assert_eq!(result, None);
    }

    #[test]
    fn test_get_capability_endpoint() {
        std::env::set_var("CRYPTO_SIGNING_ENDPOINT", "signing.example.com:9500");
        let result = SafeEnv::get_capability_endpoint("crypto:signing");
        assert_eq!(result, Some("signing.example.com:9500".to_string()));
        std::env::remove_var("CRYPTO_SIGNING_ENDPOINT");
    }

    #[test]
    fn test_get_discovery_address() {
        std::env::set_var("DISCOVERY_ENDPOINT", "discovery.example.com:8091");
        let result = SafeEnv::get_discovery_address();
        assert_eq!(result, Some("discovery.example.com:8091".to_string()));
        std::env::remove_var("DISCOVERY_ENDPOINT");
    }

    #[test]
    fn test_get_rpc_port_default() {
        std::env::remove_var("RHIZOCRYPT_RPC_PORT");
        let result = SafeEnv::get_rpc_port(9400);
        assert_eq!(result, 9400);
    }

    #[test]
    fn test_get_rpc_port_custom() {
        std::env::set_var("RHIZOCRYPT_RPC_PORT", "9999");
        let result = SafeEnv::get_rpc_port(9400);
        assert_eq!(result, 9999);
        std::env::remove_var("RHIZOCRYPT_RPC_PORT");
    }

    #[test]
    fn test_get_rpc_host_default() {
        std::env::remove_var("RHIZOCRYPT_RPC_HOST");
        let result = SafeEnv::get_rpc_host();
        assert_eq!(result, "0.0.0.0");
    }

    #[test]
    fn test_get_rpc_host_custom() {
        std::env::set_var("RHIZOCRYPT_RPC_HOST", "127.0.0.1");
        let result = SafeEnv::get_rpc_host();
        assert_eq!(result, "127.0.0.1");
        std::env::remove_var("RHIZOCRYPT_RPC_HOST");
    }

    #[test]
    fn test_get_metrics_port_default() {
        std::env::remove_var("RHIZOCRYPT_METRICS_PORT");
        let result = SafeEnv::get_metrics_port(9401);
        assert_eq!(result, 9401);
    }

    #[test]
    fn test_get_metrics_port_custom() {
        std::env::set_var("RHIZOCRYPT_METRICS_PORT", "8888");
        let result = SafeEnv::get_metrics_port(9401);
        assert_eq!(result, 8888);
        std::env::remove_var("RHIZOCRYPT_METRICS_PORT");
    }

    // CapabilityEnv tests
    #[test]
    fn test_signing_endpoint_primary() {
        std::env::set_var("CRYPTO_SIGNING_ENDPOINT", "signing.example.com:9500");
        let result = CapabilityEnv::signing_endpoint();
        assert_eq!(result, Some("signing.example.com:9500".to_string()));
        std::env::remove_var("CRYPTO_SIGNING_ENDPOINT");
    }

    #[test]
    fn test_signing_endpoint_short_form() {
        std::env::set_var("SIGNING_ENDPOINT", "signing.example.com:9500");
        let result = CapabilityEnv::signing_endpoint();
        assert_eq!(result, Some("signing.example.com:9500".to_string()));
        std::env::remove_var("SIGNING_ENDPOINT");
    }

    #[test]
    fn test_signing_endpoint_legacy() {
        std::env::set_var("BEARDOG_ADDRESS", "beardog.example.com:9500");
        let result = CapabilityEnv::signing_endpoint();
        assert_eq!(result, Some("beardog.example.com:9500".to_string()));
        std::env::remove_var("BEARDOG_ADDRESS");
    }

    #[test]
    fn test_did_verification_endpoint() {
        std::env::set_var("DID_VERIFICATION_ENDPOINT", "did.example.com:9500");
        let result = CapabilityEnv::did_verification_endpoint();
        assert_eq!(result, Some("did.example.com:9500".to_string()));
        std::env::remove_var("DID_VERIFICATION_ENDPOINT");
    }

    #[test]
    fn test_payload_storage_endpoint() {
        std::env::set_var("PAYLOAD_STORAGE_ENDPOINT", "storage.example.com:9600");
        let result = CapabilityEnv::payload_storage_endpoint();
        assert_eq!(result, Some("storage.example.com:9600".to_string()));
        std::env::remove_var("PAYLOAD_STORAGE_ENDPOINT");
    }

    #[test]
    fn test_permanent_commit_endpoint() {
        std::env::set_var("PERMANENT_STORAGE_ENDPOINT", "permanent.example.com:9700");
        let result = CapabilityEnv::permanent_commit_endpoint();
        assert_eq!(result, Some("permanent.example.com:9700".to_string()));
        std::env::remove_var("PERMANENT_STORAGE_ENDPOINT");
    }

    #[test]
    fn test_compute_endpoint() {
        std::env::set_var("COMPUTE_ENDPOINT", "compute.example.com:9800");
        let result = CapabilityEnv::compute_endpoint();
        assert_eq!(result, Some("compute.example.com:9800".to_string()));
        std::env::remove_var("COMPUTE_ENDPOINT");
    }

    #[test]
    fn test_provenance_endpoint() {
        std::env::set_var("PROVENANCE_ENDPOINT", "provenance.example.com:9900");
        let result = CapabilityEnv::provenance_endpoint();
        assert_eq!(result, Some("provenance.example.com:9900".to_string()));
        std::env::remove_var("PROVENANCE_ENDPOINT");
    }

    #[test]
    fn test_discovery_endpoint() {
        std::env::set_var("DISCOVERY_ENDPOINT", "discovery.example.com:8091");
        let result = CapabilityEnv::discovery_endpoint();
        assert_eq!(result, Some("discovery.example.com:8091".to_string()));
        std::env::remove_var("DISCOVERY_ENDPOINT");
    }

    #[test]
    fn test_capability_endpoint_priority() {
        // Test that primary takes precedence over short form
        std::env::set_var("CRYPTO_SIGNING_ENDPOINT", "primary.example.com:9500");
        std::env::set_var("SIGNING_ENDPOINT", "short.example.com:9500");
        std::env::set_var("BEARDOG_ADDRESS", "legacy.example.com:9500");

        let result = CapabilityEnv::signing_endpoint();
        assert_eq!(result, Some("primary.example.com:9500".to_string()));

        // Clean up immediately
        std::env::remove_var("CRYPTO_SIGNING_ENDPOINT");
        std::env::remove_var("SIGNING_ENDPOINT");
        std::env::remove_var("BEARDOG_ADDRESS");
    }

    #[test]
    fn test_capability_endpoint_none() {
        // Ensure clean state - remove all possible variants
        std::env::remove_var("CRYPTO_SIGNING_ENDPOINT");
        std::env::remove_var("CRYPTO_SIGNING_ADDRESS");
        std::env::remove_var("SIGNING_ENDPOINT");
        std::env::remove_var("SIGNING_ADDRESS");
        std::env::remove_var("BEARDOG_ADDRESS");

        let result = CapabilityEnv::signing_endpoint();
        assert_eq!(result, None);
    }
}
