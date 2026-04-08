// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2024–2026 ecoPrimals Project

use super::*;

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
    temp_env::with_vars([("RHIZOCRYPT_ENV", None::<&str>)], || {
        assert!(!SafeEnv::is_development());
        assert!(SafeEnv::is_production());
    });
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
    let key = "RHIZOCRYPT_TEST_WITH_VALUE";
    temp_env::with_vars([(key, Some("custom_value"))], || {
        let result = SafeEnv::get_or_default(key, "fallback");
        assert_eq!(result, "custom_value");
    });
}

#[test]
fn test_get_optional_with_value() {
    let key = "RHIZOCRYPT_TEST_OPTIONAL_WITH_VALUE";
    temp_env::with_vars([(key, Some("some_value"))], || {
        let result = SafeEnv::get_optional(key);
        assert_eq!(result, Some("some_value".to_string()));
    });
}

#[test]
fn test_parse_with_valid_value() {
    let key = "RHIZOCRYPT_TEST_PARSE_VALID";
    temp_env::with_vars([(key, Some("9999"))], || {
        let result: u16 = SafeEnv::parse(key, 8080);
        assert_eq!(result, 9999);
    });
}

#[test]
fn test_parse_with_invalid_value() {
    let key = "RHIZOCRYPT_TEST_PARSE_INVALID";
    temp_env::with_vars([(key, Some("not_a_number"))], || {
        let result: u16 = SafeEnv::parse(key, 8080);
        assert_eq!(result, 8080, "Should use default on parse failure");
    });
}

#[test]
fn test_parse_optional_with_valid() {
    let key = "RHIZOCRYPT_TEST_PARSE_OPT_VALID";
    temp_env::with_vars([(key, Some("7777"))], || {
        let result: Option<u16> = SafeEnv::parse_optional(key);
        assert_eq!(result, Some(7777));
    });
}

#[test]
fn test_parse_optional_with_invalid() {
    let key = "RHIZOCRYPT_TEST_PARSE_OPT_INVALID";
    temp_env::with_vars([(key, Some("invalid"))], || {
        let result: Option<u16> = SafeEnv::parse_optional(key);
        assert_eq!(result, None);
    });
}

#[test]
fn test_parse_optional_missing() {
    let key = "RHIZOCRYPT_TEST_PARSE_OPT_MISSING";
    let result: Option<u16> = SafeEnv::parse_optional(key);
    assert_eq!(result, None);
}

#[test]
fn test_parse_empty_string() {
    let key = "RHIZOCRYPT_TEST_PARSE_EMPTY";
    temp_env::with_vars([(key, Some(""))], || {
        let result: u16 = SafeEnv::parse(key, 8080);
        assert_eq!(result, 8080, "Empty string should use default");
    });
}

#[test]
fn test_parse_whitespace() {
    let key = "RHIZOCRYPT_TEST_PARSE_WHITESPACE";
    temp_env::with_vars([(key, Some("   "))], || {
        let result: u16 = SafeEnv::parse(key, 8080);
        assert_eq!(result, 8080, "Whitespace-only should use default");
    });
}

#[test]
fn test_parse_optional_empty_string() {
    let key = "RHIZOCRYPT_TEST_PARSE_OPT_EMPTY";
    temp_env::with_vars([(key, Some(""))], || {
        let result: Option<u16> = SafeEnv::parse_optional(key);
        assert_eq!(result, None);
    });
}

#[test]
fn test_parse_bool_type() {
    let key = "RHIZOCRYPT_TEST_PARSE_BOOL";
    temp_env::with_vars([(key, Some("true"))], || {
        let result: bool = SafeEnv::parse(key, false);
        assert!(result);
    });
}

#[test]
fn test_get_discovery_address_rhizocrypt_adapter() {
    temp_env::with_vars(
        [("RHIZOCRYPT_DISCOVERY_ADAPTER", Some("adapter.example.com:7500"))],
        || {
            let result = SafeEnv::get_discovery_address();
            assert_eq!(result, Some("adapter.example.com:7500".to_string()));
        },
    );
}

#[test]
fn test_get_discovery_address_priority() {
    temp_env::with_vars(
        [
            ("RHIZOCRYPT_DISCOVERY_ADAPTER", Some("primary.example.com:7500")),
            ("DISCOVERY_ENDPOINT", Some("fallback.example.com:8091")),
        ],
        || {
            let result = SafeEnv::get_discovery_address();
            assert_eq!(result, Some("primary.example.com:7500".to_string()));
        },
    );
}

#[test]
fn test_get_rpc_port_legacy() {
    temp_env::with_vars(
        [("RHIZOCRYPT_RPC_PORT", None::<&str>), ("RHIZOCRYPT_PORT", Some("8888"))],
        || {
            let result = SafeEnv::get_rpc_port(1000);
            assert_eq!(result, 8888);
        },
    );
}

#[test]
fn test_get_rpc_port_preferred_takes_priority() {
    temp_env::with_vars(
        [("RHIZOCRYPT_RPC_PORT", Some("9500")), ("RHIZOCRYPT_PORT", Some("9400"))],
        || {
            let result = SafeEnv::get_rpc_port(9000);
            assert_eq!(result, 9500);
        },
    );
}

#[test]
fn test_get_capability_endpoint_with_address_suffix() {
    temp_env::with_vars(
        [("CRYPTO_SIGNING_ADDRESS", Some("signing-addr.example.com:9500"))],
        || {
            let result = SafeEnv::get_capability_endpoint("crypto:signing");
            assert_eq!(result, Some("signing-addr.example.com:9500".to_string()));
        },
    );
}

#[test]
fn test_is_development_true() {
    temp_env::with_vars([("RHIZOCRYPT_ENV", Some("development"))], || {
        assert!(SafeEnv::is_development());
        assert!(!SafeEnv::is_production());
    });
}

#[test]
fn test_is_development_case_insensitive() {
    temp_env::with_vars([("RHIZOCRYPT_ENV", Some("DEVELOPMENT"))], || {
        assert!(SafeEnv::is_development());
    });
    temp_env::with_vars([("RHIZOCRYPT_ENV", Some("Development"))], || {
        assert!(SafeEnv::is_development());
    });
}

#[test]
fn test_is_production_default() {
    temp_env::with_vars([("RHIZOCRYPT_ENV", None::<&str>)], || {
        assert!(SafeEnv::is_production());
        assert!(!SafeEnv::is_development());
    });
}

#[test]
fn test_is_production_explicit() {
    temp_env::with_vars([("RHIZOCRYPT_ENV", Some("production"))], || {
        assert!(SafeEnv::is_production());
        assert!(!SafeEnv::is_development());
    });
}

#[test]
fn test_get_endpoint_with_endpoint_suffix() {
    let prefix = "TEST_SERVICE";
    temp_env::with_vars([("TEST_SERVICE_ENDPOINT", Some("service.example.com:9000"))], || {
        let result = SafeEnv::get_endpoint(prefix);
        assert_eq!(result, Some("service.example.com:9000".to_string()));
    });
}

#[test]
fn test_get_endpoint_with_address_suffix() {
    let prefix = "TEST_SERVICE";
    temp_env::with_vars([("TEST_SERVICE_ADDRESS", Some("service.example.com:9001"))], || {
        let result = SafeEnv::get_endpoint(prefix);
        assert_eq!(result, Some("service.example.com:9001".to_string()));
    });
}

#[test]
fn test_get_endpoint_priority_endpoint_over_address() {
    let prefix = "TEST_SERVICE";
    temp_env::with_vars(
        [
            ("TEST_SERVICE_ENDPOINT", Some("endpoint.example.com:9000")),
            ("TEST_SERVICE_ADDRESS", Some("address.example.com:9001")),
        ],
        || {
            let result = SafeEnv::get_endpoint(prefix);
            assert_eq!(result, Some("endpoint.example.com:9000".to_string()));
        },
    );
}

#[test]
fn test_get_endpoint_missing() {
    let prefix = "NONEXISTENT_SERVICE";
    let result = SafeEnv::get_endpoint(prefix);
    assert_eq!(result, None);
}

#[test]
fn test_get_capability_endpoint() {
    temp_env::with_vars([("CRYPTO_SIGNING_ENDPOINT", Some("signing.example.com:9500"))], || {
        let result = SafeEnv::get_capability_endpoint("crypto:signing");
        assert_eq!(result, Some("signing.example.com:9500".to_string()));
    });
}

#[test]
fn test_get_discovery_address() {
    temp_env::with_vars([("DISCOVERY_ENDPOINT", Some("discovery.example.com:8091"))], || {
        let result = SafeEnv::get_discovery_address();
        assert_eq!(result, Some("discovery.example.com:8091".to_string()));
    });
}

#[test]
fn test_get_rpc_port_default() {
    temp_env::with_vars([("RHIZOCRYPT_RPC_PORT", None::<&str>)], || {
        let result = SafeEnv::get_rpc_port(9400);
        assert_eq!(result, 9400);
    });
}

#[test]
fn test_get_rpc_port_custom() {
    temp_env::with_vars([("RHIZOCRYPT_RPC_PORT", Some("9999"))], || {
        let result = SafeEnv::get_rpc_port(9400);
        assert_eq!(result, 9999);
    });
}

#[test]
fn test_get_jsonrpc_port_default_offset() {
    temp_env::with_vars([("RHIZOCRYPT_JSONRPC_PORT", None::<&str>)], || {
        let result = SafeEnv::get_jsonrpc_port(9400);
        assert_eq!(result, 9400 + crate::constants::JSONRPC_PORT_OFFSET);
    });
}

#[test]
fn test_get_jsonrpc_port_explicit_override() {
    temp_env::with_vars([("RHIZOCRYPT_JSONRPC_PORT", Some("7777"))], || {
        let result = SafeEnv::get_jsonrpc_port(9400);
        assert_eq!(result, 7777);
    });
}

#[test]
fn test_get_jsonrpc_port_zero_passthrough() {
    temp_env::with_vars([("RHIZOCRYPT_JSONRPC_PORT", None::<&str>)], || {
        let result = SafeEnv::get_jsonrpc_port(0);
        assert_eq!(result, 0, "OS-assigned tarpc port should yield OS-assigned JSON-RPC port");
    });
}

#[test]
fn test_get_jsonrpc_port_saturating() {
    temp_env::with_vars([("RHIZOCRYPT_JSONRPC_PORT", None::<&str>)], || {
        let result = SafeEnv::get_jsonrpc_port(u16::MAX);
        assert_eq!(result, u16::MAX, "should saturate rather than overflow");
    });
}

#[test]
fn test_get_rpc_host_default() {
    temp_env::with_vars(
        [("RHIZOCRYPT_RPC_HOST", None::<&str>), ("RHIZOCRYPT_HOST", None::<&str>)],
        || {
            let result = SafeEnv::get_rpc_host();
            assert_eq!(result, "0.0.0.0");
        },
    );
}

#[test]
fn test_get_rpc_host_custom() {
    temp_env::with_vars([("RHIZOCRYPT_RPC_HOST", Some("127.0.0.1"))], || {
        let result = SafeEnv::get_rpc_host();
        assert_eq!(result, "127.0.0.1");
    });
}

#[test]
fn test_get_rpc_host_legacy_fallback() {
    temp_env::with_vars(
        [("RHIZOCRYPT_RPC_HOST", None::<&str>), ("RHIZOCRYPT_HOST", Some("10.0.0.1"))],
        || {
            let result = SafeEnv::get_rpc_host();
            assert_eq!(result, "10.0.0.1");
        },
    );
}

#[test]
fn test_get_metrics_port_default() {
    temp_env::with_vars([("RHIZOCRYPT_METRICS_PORT", None::<&str>)], || {
        let result = SafeEnv::get_metrics_port(9401);
        assert_eq!(result, 9401);
    });
}

#[test]
fn test_get_metrics_port_custom() {
    temp_env::with_vars([("RHIZOCRYPT_METRICS_PORT", Some("8888"))], || {
        let result = SafeEnv::get_metrics_port(9401);
        assert_eq!(result, 8888);
    });
}

#[test]
fn test_socket_env_var() {
    assert_eq!(SafeEnv::socket_env_var("rhizoCrypt"), "RHIZOCRYPT_SOCKET");
    assert_eq!(SafeEnv::socket_env_var("primalA"), "PRIMALA_SOCKET");
    assert_eq!(SafeEnv::socket_env_var("primalB"), "PRIMALB_SOCKET");
}

#[test]
fn test_address_env_var() {
    assert_eq!(SafeEnv::address_env_var("primalA"), "PRIMALA_ADDRESS");
    assert_eq!(SafeEnv::address_env_var("discoveryAdapter"), "DISCOVERYADAPTER_ADDRESS");
}

#[test]
fn test_get_socket_path_from_env() {
    temp_env::with_vars([("RHIZOCRYPT_SOCKET", Some("/run/biomeos/rhizoCrypt.sock"))], || {
        let path = SafeEnv::get_socket_path("rhizoCrypt");
        assert_eq!(path, Some(std::path::PathBuf::from("/run/biomeos/rhizoCrypt.sock")));
    });
}

#[test]
fn test_get_socket_path_xdg_fallback() {
    temp_env::with_vars(
        [("RHIZOCRYPT_SOCKET", None::<&str>), ("XDG_RUNTIME_DIR", Some("/run/user/1000"))],
        || {
            let path = SafeEnv::get_socket_path("rhizoCrypt");
            assert_eq!(
                path,
                Some(std::path::PathBuf::from("/run/user/1000/biomeos/rhizoCrypt.sock"))
            );
        },
    );
}

#[test]
fn test_get_socket_path_none() {
    temp_env::with_vars(
        [("RHIZOCRYPT_SOCKET", None::<&str>), ("XDG_RUNTIME_DIR", None::<&str>)],
        || {
            let path = SafeEnv::get_socket_path("rhizoCrypt");
            assert!(path.is_none());
        },
    );
}
