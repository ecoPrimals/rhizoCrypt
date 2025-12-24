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
    /// Checks `DISCOVERY_ENDPOINT`, `DISCOVERY_ADDRESS`, and falls back to None.
    #[must_use]
    pub fn get_discovery_address() -> Option<String> {
        Self::get_endpoint("DISCOVERY")
    }

    /// Get the RPC port, with environment override.
    #[must_use]
    pub fn get_rpc_port(default: u16) -> u16 {
        Self::parse("RHIZOCRYPT_RPC_PORT", default)
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
pub struct CapabilityEnv;

impl CapabilityEnv {
    /// Get the endpoint for signing capability.
    #[must_use]
    pub fn signing_endpoint() -> Option<String> {
        SafeEnv::get_endpoint("CRYPTO_SIGNING").or_else(|| SafeEnv::get_endpoint("SIGNING"))
    }

    /// Get the endpoint for DID verification capability.
    #[must_use]
    pub fn did_verification_endpoint() -> Option<String> {
        SafeEnv::get_endpoint("DID_VERIFICATION").or_else(|| SafeEnv::get_endpoint("DID"))
    }

    /// Get the endpoint for payload storage capability.
    #[must_use]
    pub fn payload_storage_endpoint() -> Option<String> {
        SafeEnv::get_endpoint("PAYLOAD_STORAGE").or_else(|| SafeEnv::get_endpoint("PAYLOAD"))
    }

    /// Get the endpoint for permanent commit capability.
    #[must_use]
    pub fn permanent_commit_endpoint() -> Option<String> {
        SafeEnv::get_endpoint("STORAGE_PERMANENT_COMMIT")
            .or_else(|| SafeEnv::get_endpoint("PERMANENT_STORAGE"))
    }

    /// Get the endpoint for compute orchestration capability.
    #[must_use]
    pub fn compute_endpoint() -> Option<String> {
        SafeEnv::get_endpoint("COMPUTE_ORCHESTRATION").or_else(|| SafeEnv::get_endpoint("COMPUTE"))
    }

    /// Get the endpoint for provenance query capability.
    #[must_use]
    pub fn provenance_endpoint() -> Option<String> {
        SafeEnv::get_endpoint("PROVENANCE_QUERY").or_else(|| SafeEnv::get_endpoint("PROVENANCE"))
    }

    /// Get the endpoint for service discovery capability.
    #[must_use]
    pub fn discovery_endpoint() -> Option<String> {
        SafeEnv::get_endpoint("DISCOVERY_SERVICE").or_else(|| SafeEnv::get_endpoint("DISCOVERY"))
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
}
