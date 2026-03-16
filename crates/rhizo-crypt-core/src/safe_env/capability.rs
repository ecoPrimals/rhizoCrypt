// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2024–2026 ecoPrimals Project

//! Capability-specific environment configuration.
//!
//! Provides standardized environment variable names for each capability.
//!
//! ## Naming Convention
//!
//! Capability-based variables follow this pattern:
//! - **Preferred**: `<CAPABILITY>_ENDPOINT` (e.g., `SIGNING_ENDPOINT`)
//! - **Alternative**: `<CATEGORY>_<CAPABILITY>_ENDPOINT` (e.g., `CRYPTO_SIGNING_ENDPOINT`)
//! - **Legacy**: `<PRIMAL>_ADDRESS` (e.g., `BEARDOG_ADDRESS`) - deprecated
//!
//! ## Infant Discovery
//!
//! In production, primals should discover capabilities at runtime via the
//! universal adapter (`RHIZOCRYPT_DISCOVERY_ADAPTER`). Environment variables
//! are **hints** for development or testing, not requirements.
//!
//! ## Migration Path
//!
//! Old (hardcoded):
//! ```bash
//! BEARDOG_ADDRESS=beardog.local:9500
//! NESTGATE_ADDRESS=nestgate.local:8080
//! ```
//!
//! New (capability-based):
//! ```bash
//! RHIZOCRYPT_DISCOVERY_ADAPTER=songbird.local:7500  # Only this is needed!
//! # OR for development:
//! SIGNING_ENDPOINT=http://localhost:9500
//! PAYLOAD_STORAGE_ENDPOINT=http://localhost:8080
//! ```

use super::SafeEnv;

/// Capability-specific environment configuration.
pub struct CapabilityEnv;

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
                std::env::var("BEARDOG_ADDRESS").ok().inspect(|_| {
                    tracing::warn!(
                        "Using deprecated BEARDOG_ADDRESS environment variable. \
                         Please migrate to SIGNING_ENDPOINT or CRYPTO_SIGNING_ENDPOINT \
                         for capability-based configuration."
                    );
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
                std::env::var("BEARDOG_ADDRESS").ok().inspect(|_| {
                    tracing::warn!(
                        "Using deprecated BEARDOG_ADDRESS for DID verification. \
                         Please migrate to DID_ENDPOINT for capability-based configuration."
                    );
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
                std::env::var("NESTGATE_ADDRESS").ok().inspect(|_| {
                    tracing::warn!(
                        "Using deprecated NESTGATE_ADDRESS environment variable. \
                         Please migrate to PAYLOAD_STORAGE_ENDPOINT \
                         for capability-based configuration."
                    );
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
                std::env::var("LOAMSPINE_ADDRESS").ok().inspect(|_| {
                    tracing::warn!(
                        "Using deprecated LOAMSPINE_ADDRESS environment variable. \
                         Please migrate to PERMANENT_STORAGE_ENDPOINT \
                         for capability-based configuration."
                    );
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
                std::env::var("SONGBIRD_ADDRESS").ok().inspect(|_| {
                    tracing::info!(
                        "Using SONGBIRD_ADDRESS for discovery. \
                         Consider migrating to RHIZOCRYPT_DISCOVERY_ADAPTER for consistency."
                    );
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
    fn test_signing_endpoint_primary() {
        temp_env::with_vars(
            [("CRYPTO_SIGNING_ENDPOINT", Some("signing.example.com:9500"))],
            || {
                let result = CapabilityEnv::signing_endpoint();
                assert_eq!(result, Some("signing.example.com:9500".to_string()));
            },
        );
    }

    #[test]
    fn test_signing_endpoint_short_form() {
        temp_env::with_vars(
            [
                ("CRYPTO_SIGNING_ENDPOINT", None::<&str>),
                ("CRYPTO_SIGNING_ADDRESS", None),
                ("SIGNING_ENDPOINT", Some("signing.example.com:9500")),
            ],
            || {
                let result = CapabilityEnv::signing_endpoint();
                assert_eq!(result, Some("signing.example.com:9500".to_string()));
            },
        );
    }

    #[test]
    fn test_signing_endpoint_legacy() {
        temp_env::with_vars(
            [
                ("CRYPTO_SIGNING_ENDPOINT", None::<&str>),
                ("CRYPTO_SIGNING_ADDRESS", None),
                ("SIGNING_ENDPOINT", None),
                ("SIGNING_ADDRESS", None),
                ("BEARDOG_ADDRESS", Some("beardog.example.com:9500")),
            ],
            || {
                let result = CapabilityEnv::signing_endpoint();
                assert_eq!(result, Some("beardog.example.com:9500".to_string()));
            },
        );
    }

    #[test]
    fn test_did_verification_endpoint_primary() {
        temp_env::with_vars([("DID_VERIFICATION_ENDPOINT", Some("did.example.com:9500"))], || {
            let result = CapabilityEnv::did_verification_endpoint();
            assert_eq!(result, Some("did.example.com:9500".to_string()));
        });
    }

    #[test]
    fn test_did_verification_endpoint_short_form() {
        temp_env::with_vars(
            [
                ("DID_VERIFICATION_ENDPOINT", None::<&str>),
                ("DID_VERIFICATION_ADDRESS", None),
                ("DID_ENDPOINT", Some("did-short.example.com:9500")),
            ],
            || {
                let result = CapabilityEnv::did_verification_endpoint();
                assert_eq!(result, Some("did-short.example.com:9500".to_string()));
            },
        );
    }

    #[test]
    fn test_did_verification_endpoint_legacy() {
        temp_env::with_vars(
            [
                ("DID_VERIFICATION_ENDPOINT", None::<&str>),
                ("DID_VERIFICATION_ADDRESS", None),
                ("DID_ENDPOINT", None),
                ("DID_ADDRESS", None),
                ("BEARDOG_ADDRESS", Some("beardog-did.example.com:9500")),
            ],
            || {
                let result = CapabilityEnv::did_verification_endpoint();
                assert_eq!(result, Some("beardog-did.example.com:9500".to_string()));
            },
        );
    }

    #[test]
    fn test_payload_storage_endpoint_primary() {
        temp_env::with_vars(
            [("PAYLOAD_STORAGE_ENDPOINT", Some("storage.example.com:9600"))],
            || {
                let result = CapabilityEnv::payload_storage_endpoint();
                assert_eq!(result, Some("storage.example.com:9600".to_string()));
            },
        );
    }

    #[test]
    fn test_payload_storage_endpoint_short_form() {
        temp_env::with_vars(
            [
                ("PAYLOAD_STORAGE_ENDPOINT", None::<&str>),
                ("PAYLOAD_STORAGE_ADDRESS", None),
                ("PAYLOAD_ENDPOINT", Some("payload-short.example.com:9600")),
            ],
            || {
                let result = CapabilityEnv::payload_storage_endpoint();
                assert_eq!(result, Some("payload-short.example.com:9600".to_string()));
            },
        );
    }

    #[test]
    fn test_payload_storage_endpoint_legacy() {
        temp_env::with_vars(
            [
                ("PAYLOAD_STORAGE_ENDPOINT", None::<&str>),
                ("PAYLOAD_STORAGE_ADDRESS", None),
                ("PAYLOAD_ENDPOINT", None),
                ("PAYLOAD_ADDRESS", None),
                ("NESTGATE_ADDRESS", Some("nestgate.example.com:9600")),
            ],
            || {
                let result = CapabilityEnv::payload_storage_endpoint();
                assert_eq!(result, Some("nestgate.example.com:9600".to_string()));
            },
        );
    }

    #[test]
    fn test_permanent_commit_endpoint_primary() {
        temp_env::with_vars(
            [("STORAGE_PERMANENT_COMMIT_ENDPOINT", Some("permanent-primary.example.com:9700"))],
            || {
                let result = CapabilityEnv::permanent_commit_endpoint();
                assert_eq!(result, Some("permanent-primary.example.com:9700".to_string()));
            },
        );
    }

    #[test]
    fn test_permanent_commit_endpoint_short_form() {
        temp_env::with_vars(
            [
                ("STORAGE_PERMANENT_COMMIT_ENDPOINT", None::<&str>),
                ("STORAGE_PERMANENT_COMMIT_ADDRESS", None),
                ("PERMANENT_STORAGE_ENDPOINT", Some("permanent.example.com:9700")),
            ],
            || {
                let result = CapabilityEnv::permanent_commit_endpoint();
                assert_eq!(result, Some("permanent.example.com:9700".to_string()));
            },
        );
    }

    #[test]
    fn test_permanent_commit_endpoint_legacy() {
        temp_env::with_vars(
            [
                ("STORAGE_PERMANENT_COMMIT_ENDPOINT", None::<&str>),
                ("STORAGE_PERMANENT_COMMIT_ADDRESS", None),
                ("PERMANENT_STORAGE_ENDPOINT", None),
                ("PERMANENT_STORAGE_ADDRESS", None),
                ("LOAMSPINE_ADDRESS", Some("loamspine.example.com:9700")),
            ],
            || {
                let result = CapabilityEnv::permanent_commit_endpoint();
                assert_eq!(result, Some("loamspine.example.com:9700".to_string()));
            },
        );
    }

    #[test]
    fn test_compute_endpoint_preferred() {
        temp_env::with_vars(
            [("COMPUTE_ORCHESTRATION_ENDPOINT", Some("compute-pref.example.com:9800"))],
            || {
                let result = CapabilityEnv::compute_endpoint();
                assert_eq!(result, Some("compute-pref.example.com:9800".to_string()));
            },
        );
    }

    #[test]
    fn test_compute_endpoint_short_form() {
        temp_env::with_vars(
            [
                ("COMPUTE_ORCHESTRATION_ENDPOINT", None::<&str>),
                ("COMPUTE_ORCHESTRATION_ADDRESS", None),
                ("COMPUTE_ENDPOINT", Some("compute.example.com:9800")),
            ],
            || {
                let result = CapabilityEnv::compute_endpoint();
                assert_eq!(result, Some("compute.example.com:9800".to_string()));
            },
        );
    }

    #[test]
    fn test_provenance_endpoint_preferred() {
        temp_env::with_vars(
            [("PROVENANCE_QUERY_ENDPOINT", Some("provenance-pref.example.com:9900"))],
            || {
                let result = CapabilityEnv::provenance_endpoint();
                assert_eq!(result, Some("provenance-pref.example.com:9900".to_string()));
            },
        );
    }

    #[test]
    fn test_provenance_endpoint_short_form() {
        temp_env::with_vars(
            [
                ("PROVENANCE_QUERY_ENDPOINT", None::<&str>),
                ("PROVENANCE_QUERY_ADDRESS", None),
                ("PROVENANCE_ENDPOINT", Some("provenance.example.com:9900")),
            ],
            || {
                let result = CapabilityEnv::provenance_endpoint();
                assert_eq!(result, Some("provenance.example.com:9900".to_string()));
            },
        );
    }

    #[test]
    fn test_discovery_endpoint_rhizocrypt_adapter() {
        temp_env::with_vars(
            [("RHIZOCRYPT_DISCOVERY_ADAPTER", Some("adapter.example.com:7500"))],
            || {
                let result = CapabilityEnv::discovery_endpoint();
                assert_eq!(result, Some("adapter.example.com:7500".to_string()));
            },
        );
    }

    #[test]
    fn test_discovery_endpoint_discovery_service() {
        temp_env::with_vars(
            [
                ("RHIZOCRYPT_DISCOVERY_ADAPTER", None::<&str>),
                ("DISCOVERY_SERVICE_ENDPOINT", Some("discovery-svc.example.com:8091")),
            ],
            || {
                let result = CapabilityEnv::discovery_endpoint();
                assert_eq!(result, Some("discovery-svc.example.com:8091".to_string()));
            },
        );
    }

    #[test]
    fn test_discovery_endpoint_short_form() {
        temp_env::with_vars(
            [
                ("RHIZOCRYPT_DISCOVERY_ADAPTER", None::<&str>),
                ("DISCOVERY_SERVICE_ENDPOINT", None),
                ("DISCOVERY_SERVICE_ADDRESS", None),
                ("DISCOVERY_ENDPOINT", Some("discovery.example.com:8091")),
            ],
            || {
                let result = CapabilityEnv::discovery_endpoint();
                assert_eq!(result, Some("discovery.example.com:8091".to_string()));
            },
        );
    }

    #[test]
    fn test_discovery_endpoint_songbird_legacy() {
        temp_env::with_vars(
            [
                ("RHIZOCRYPT_DISCOVERY_ADAPTER", None::<&str>),
                ("DISCOVERY_SERVICE_ENDPOINT", None),
                ("DISCOVERY_SERVICE_ADDRESS", None),
                ("DISCOVERY_ENDPOINT", None),
                ("DISCOVERY_ADDRESS", None),
                ("SONGBIRD_ADDRESS", Some("songbird.example.com:7500")),
            ],
            || {
                let result = CapabilityEnv::discovery_endpoint();
                assert_eq!(result, Some("songbird.example.com:7500".to_string()));
            },
        );
    }

    #[test]
    fn test_capability_endpoint_priority() {
        temp_env::with_vars(
            [
                ("CRYPTO_SIGNING_ENDPOINT", Some("primary.example.com:9500")),
                ("SIGNING_ENDPOINT", Some("short.example.com:9500")),
                ("BEARDOG_ADDRESS", Some("legacy.example.com:9500")),
            ],
            || {
                let result = CapabilityEnv::signing_endpoint();
                assert_eq!(result, Some("primary.example.com:9500".to_string()));
            },
        );
    }

    #[test]
    fn test_capability_endpoint_none() {
        temp_env::with_vars(
            [
                ("CRYPTO_SIGNING_ENDPOINT", None::<&str>),
                ("CRYPTO_SIGNING_ADDRESS", None),
                ("SIGNING_ENDPOINT", None),
                ("SIGNING_ADDRESS", None),
                ("BEARDOG_ADDRESS", None),
            ],
            || {
                let result = CapabilityEnv::signing_endpoint();
                assert_eq!(result, None);
            },
        );
    }

    #[test]
    fn test_all_capability_endpoints() {
        temp_env::with_vars(
            [
                ("SIGNING_ENDPOINT", Some("signing.example.com:9500")),
                ("DID_ENDPOINT", Some("did.example.com:9500")),
                ("PAYLOAD_STORAGE_ENDPOINT", Some("payload.example.com:9600")),
            ],
            || {
                let map = CapabilityEnv::all_capability_endpoints();
                assert!(map.contains_key("signing"));
                assert_eq!(map.get("signing").unwrap(), "signing.example.com:9500");
                assert!(map.contains_key("did_verification"));
                assert!(map.contains_key("payload_storage"));
            },
        );
    }

    #[test]
    fn test_is_infant_discovery_mode_empty() {
        let unset_vars: Vec<(&str, Option<&str>)> = [
            "CRYPTO_SIGNING_ENDPOINT",
            "SIGNING_ENDPOINT",
            "BEARDOG_ADDRESS",
            "DID_VERIFICATION_ENDPOINT",
            "DID_ENDPOINT",
            "PAYLOAD_STORAGE_ENDPOINT",
            "PAYLOAD_ENDPOINT",
            "NESTGATE_ADDRESS",
            "STORAGE_PERMANENT_COMMIT_ENDPOINT",
            "PERMANENT_STORAGE_ENDPOINT",
            "LOAMSPINE_ADDRESS",
            "COMPUTE_ORCHESTRATION_ENDPOINT",
            "COMPUTE_ENDPOINT",
            "PROVENANCE_QUERY_ENDPOINT",
            "PROVENANCE_ENDPOINT",
            "RHIZOCRYPT_DISCOVERY_ADAPTER",
            "DISCOVERY_SERVICE_ENDPOINT",
            "DISCOVERY_ENDPOINT",
            "SONGBIRD_ADDRESS",
        ]
        .iter()
        .map(|k| (*k, None))
        .collect();
        temp_env::with_vars(unset_vars, || {
            assert!(CapabilityEnv::is_infant_discovery_mode());
        });
    }

    #[test]
    fn test_is_infant_discovery_mode_with_endpoint() {
        temp_env::with_vars([("SIGNING_ENDPOINT", Some("signing.example.com:9500"))], || {
            assert!(!CapabilityEnv::is_infant_discovery_mode());
        });
    }
}
