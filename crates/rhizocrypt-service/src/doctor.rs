// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2024–2026 ecoPrimals Project

//! `UniBin` doctor diagnostics for `rhizoCrypt`.
//!
//! Performs health checks on the DAG engine, storage backends, configuration,
//! and optional discovery connectivity. Output is human-readable for operator
//! inspection.

use crate::ServiceError;
use rhizo_crypt_core::constants;
use rhizo_crypt_core::primal::PrimalLifecycle;
use rhizo_crypt_core::safe_env::SafeEnv;
use rhizo_crypt_core::{RhizoCrypt, RhizoCryptConfig};

/// Result of a single doctor check.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DoctorCheck {
    /// Check passed.
    Pass,
    /// Check passed with a warning (non-fatal).
    Warn,
    /// Check failed.
    Fail,
}

/// Run health diagnostics per the `UniBin` Architecture Standard.
///
/// Performs checks on DAG engine, storage, transport, configuration, and
/// optional discovery connectivity. Output is human-readable for operator
/// inspection.
pub async fn run_doctor(comprehensive: bool) {
    let version = env!("CARGO_PKG_VERSION");
    println!("rhizoCrypt Doctor v{version}");
    println!("==============================");

    let mut checks: Vec<(String, DoctorCheck, Option<String>)> = Vec::new();

    let dag_ok = check_dag_engine().await;
    checks.push((
        "DAG engine initialization".to_string(),
        if dag_ok {
            DoctorCheck::Pass
        } else {
            DoctorCheck::Fail
        },
        None,
    ));

    let (storage_ok, storage_name) = check_storage_backend();
    checks.push((
        format!("Storage backend ({storage_name})"),
        if storage_ok {
            DoctorCheck::Pass
        } else {
            DoctorCheck::Fail
        },
        None,
    ));

    let transport_checks = check_transport();
    checks.extend(transport_checks);

    let (config_ok, config_msg) = check_configuration();
    checks.push((
        "Configuration valid".to_string(),
        if config_ok {
            DoctorCheck::Pass
        } else {
            DoctorCheck::Fail
        },
        Some(config_msg),
    ));

    let discovery_check = check_discovery(comprehensive).await;
    checks.push(("Discovery service".to_string(), discovery_check.0, Some(discovery_check.1)));

    let env_mode = if SafeEnv::is_development() {
        "development"
    } else {
        "production"
    };
    checks.push(("Environment".to_string(), DoctorCheck::Pass, Some(env_mode.to_string())));

    for (name, status, detail) in &checks {
        let symbol = match status {
            DoctorCheck::Pass => "[✓]",
            DoctorCheck::Warn => "[!]",
            DoctorCheck::Fail => "[✗]",
        };
        let suffix = detail.as_deref().map(|d| format!(" ({d})")).unwrap_or_default();
        println!("{symbol} {name}{suffix}");
    }

    let has_fail = checks.iter().any(|(_, s, _)| *s == DoctorCheck::Fail);
    let discovery_standalone = checks.iter().any(|(n, s, msg)| {
        n == "Discovery service"
            && *s == DoctorCheck::Warn
            && msg.as_deref().is_some_and(|m| m.contains("standalone"))
    });

    let overall = if has_fail {
        "Unhealthy"
    } else if discovery_standalone {
        "Healthy (standalone mode)"
    } else {
        "Healthy"
    };

    println!();
    println!("Overall: {overall}");
}

/// Check that the DAG engine can initialize and start.
pub async fn check_dag_engine() -> bool {
    let config = RhizoCryptConfig::default();
    let mut primal = RhizoCrypt::new(config);
    primal.start().await.is_ok()
}

/// Check that the default storage backend is accessible.
#[must_use]
pub fn check_storage_backend() -> (bool, &'static str) {
    #[cfg(feature = "redb")]
    {
        match check_redb_storage() {
            Ok(()) => (true, "redb"),
            Err(_) => (false, "redb"),
        }
    }

    #[cfg(not(feature = "redb"))]
    {
        (true, "memory")
    }
}

#[cfg(feature = "redb")]
fn check_redb_storage() -> Result<(), ServiceError> {
    use rhizo_crypt_core::RedbDagStore;

    let temp_dir = tempfile::tempdir().map_err(|e| ServiceError::Storage(e.to_string()))?;
    let db_path = temp_dir.path().join("doctor_check.redb");
    let _store = RedbDagStore::open(&db_path).map_err(|e| ServiceError::Storage(e.to_string()))?;
    Ok(())
}

/// Check transport configuration (UDS + TCP).
///
/// Reports the active transport modes and socket paths so downstream springs
/// can verify that UDS is available without inspecting source code.
fn check_transport() -> Vec<(String, DoctorCheck, Option<String>)> {
    use rhizo_crypt_core::transport;

    let mut results = Vec::new();

    // UDS transport (unconditional on Unix)
    #[cfg(unix)]
    {
        let socket_path = rhizo_crypt_rpc::jsonrpc::uds::default_socket_path();
        let parent_exists = socket_path.parent().is_some_and(std::path::Path::exists);
        let (status, detail) = if parent_exists {
            (DoctorCheck::Pass, format!("unconditional, path={}", socket_path.display()))
        } else {
            (
                DoctorCheck::Warn,
                format!(
                    "socket dir missing (will create on startup), path={}",
                    socket_path.display()
                ),
            )
        };
        results.push(("Transport: UDS".to_string(), status, Some(detail)));
    }

    #[cfg(not(unix))]
    {
        results.push((
            "Transport: UDS".to_string(),
            DoctorCheck::Warn,
            Some("unavailable (non-Unix platform)".to_string()),
        ));
    }

    // TCP transport (opt-in)
    let tcp_active = crate::has_explicit_tcp_config();
    let tcp_detail = if tcp_active {
        "enabled via env (RHIZOCRYPT_PORT or RHIZOCRYPT_RPC_PORT)".to_string()
    } else {
        "disabled (opt-in: pass --port or set RHIZOCRYPT_PORT)".to_string()
    };
    results.push(("Transport: TCP".to_string(), DoctorCheck::Pass, Some(tcp_detail)));

    // BTSP security
    let btsp_required = rhizo_crypt_rpc::btsp::is_btsp_required();
    let insecure = transport::is_biomeos_insecure();
    let family_id = transport::read_family_id("RHIZOCRYPT");
    let btsp_detail = if btsp_required {
        let fid = family_id.as_deref().unwrap_or("(missing FAMILY_SEED — will reject all)");
        format!("enforced (FAMILY_ID={fid})")
    } else if insecure {
        "bypassed (BIOMEOS_INSECURE=1, dev mode)".to_string()
    } else {
        "not required (no FAMILY_ID set, raw JSON-RPC)".to_string()
    };
    results.push((
        "Transport: BTSP".to_string(),
        if btsp_required && family_id.is_none() {
            DoctorCheck::Warn
        } else {
            DoctorCheck::Pass
        },
        Some(btsp_detail),
    ));

    results
}

/// Check environment variable configuration.
fn check_configuration() -> (bool, String) {
    let default_port = if SafeEnv::is_development() {
        constants::DEFAULT_RPC_PORT
    } else {
        constants::PRODUCTION_RPC_PORT
    };
    let port = SafeEnv::get_rpc_port(default_port);
    let host = SafeEnv::get_rpc_host();
    let env_mode = if SafeEnv::is_development() {
        "development"
    } else {
        "production"
    };

    let valid = !host.is_empty();
    let msg = format!("port={port}, host={host}, env={env_mode}");
    (valid, msg)
}

/// Check discovery service configuration and optionally connectivity.
async fn check_discovery(comprehensive: bool) -> (DoctorCheck, String) {
    let Some(discovery_addr) = SafeEnv::get_discovery_address() else {
        return (DoctorCheck::Warn, "not configured (standalone mode)".to_string());
    };

    if !comprehensive {
        return (
            DoctorCheck::Pass,
            format!("configured at {discovery_addr} (use --comprehensive to verify connectivity)"),
        );
    }

    match check_discovery_connectivity(&discovery_addr).await {
        Ok(()) => (DoctorCheck::Pass, format!("reachable at {discovery_addr}")),
        Err(e) => (DoctorCheck::Warn, format!("configured but unreachable: {e}")),
    }
}

/// Attempt TCP connection to discovery endpoint.
///
/// # Errors
///
/// Returns `Err` when the address cannot be parsed or the endpoint is unreachable.
pub async fn check_discovery_connectivity(addr: &str) -> Result<(), String> {
    let host_port = addr
        .strip_prefix("http://")
        .or_else(|| addr.strip_prefix("https://"))
        .unwrap_or(addr)
        .trim_end_matches('/');

    let socket_addr: std::net::SocketAddr =
        host_port.parse().map_err(|e: std::net::AddrParseError| e.to_string())?;

    tokio::net::TcpStream::connect(socket_addr).await.map_err(|e: std::io::Error| e.to_string())?;

    Ok(())
}
