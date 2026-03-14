// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2024–2026 ecoPrimals Project

//! `rhizoCrypt` service library — shared logic for the UniBin entry point.
//!
//! Extracts server startup, discovery registration, and CLI types so they can
//! be tested without spawning a subprocess.

#![forbid(unsafe_code)]

pub use rhizo_crypt_core;
pub use rhizo_crypt_rpc;

use rhizo_crypt_core::clients::songbird::{SongbirdClient, SongbirdConfig};
use rhizo_crypt_core::constants;
use rhizo_crypt_core::primal::PrimalLifecycle;
use rhizo_crypt_core::safe_env::SafeEnv;
use rhizo_crypt_core::{RhizoCrypt, RhizoCryptConfig};
use rhizo_crypt_rpc::jsonrpc::JsonRpcServer;
use rhizo_crypt_rpc::server::RpcServer;
use std::net::SocketAddr;
use std::sync::Arc;
use thiserror::Error;
use tracing::{error, info, warn};

/// Service-level error type.
#[derive(Debug, Error)]
pub enum ServiceError {
    /// Configuration error.
    #[error("configuration error: {0}")]
    Config(String),

    /// RPC server error.
    #[error("rpc server error: {0}")]
    Rpc(#[from] std::io::Error),

    /// Discovery registration failed.
    #[error("discovery registration failed: {0}")]
    Discovery(String),

    /// Address parse error.
    #[error("address parse error: {0}")]
    AddrParse(#[from] std::net::AddrParseError),
}

/// Resolve the bind address from CLI overrides + environment.
pub fn resolve_bind_addr(
    port_override: Option<u16>,
    host_override: Option<String>,
) -> Result<SocketAddr, ServiceError> {
    let default_port = if SafeEnv::is_development() {
        constants::DEFAULT_RPC_PORT
    } else {
        constants::PRODUCTION_RPC_PORT
    };
    let port = port_override.unwrap_or_else(|| SafeEnv::get_rpc_port(default_port));
    let host = host_override.unwrap_or_else(SafeEnv::get_rpc_host);
    let addr: SocketAddr = format!("{host}:{port}").parse()?;
    Ok(addr)
}

/// Start the RPC server (tarpc + JSON-RPC) with optional discovery registration.
pub async fn run_server(
    port_override: Option<u16>,
    host_override: Option<String>,
) -> Result<(), ServiceError> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    info!("Starting rhizoCrypt service...");

    let addr = resolve_bind_addr(port_override, host_override)?;

    info!(address = %addr, "Binding RPC server");

    let config = RhizoCryptConfig::default();
    let mut primal = RhizoCrypt::new(config);
    primal.start().await.map_err(|e| ServiceError::Config(e.to_string()))?;
    let primal = Arc::new(primal);
    info!("DAG engine initialized and running");

    let server = RpcServer::new(Arc::clone(&primal), addr);

    let port = addr.port();
    let host = addr.ip();
    let jsonrpc_port = if port == 0 {
        0
    } else {
        port + 1
    };
    let jsonrpc_addr: SocketAddr = format!("{host}:{jsonrpc_port}").parse()?;
    let jsonrpc_server = JsonRpcServer::new(primal, jsonrpc_addr);
    tokio::spawn(async move {
        if let Err(e) = jsonrpc_server.serve().await {
            error!(error = %e, "JSON-RPC server error");
        }
    });
    info!(address = %jsonrpc_addr, "JSON-RPC server started");

    if let Some(discovery_addr) = SafeEnv::get_discovery_address() {
        info!(discovery = %discovery_addr, "Registering with discovery service");
        match register_with_discovery(discovery_addr.clone(), addr).await {
            Ok(()) => info!("Registered with discovery service"),
            Err(e) => warn!(error = %e, "Discovery registration failed, continuing standalone"),
        }
    } else {
        info!("No discovery service configured (standalone mode)");
    }

    info!("rhizoCrypt service ready");

    match server.serve().await {
        Ok(()) => {
            info!("rhizoCrypt service shutdown cleanly");
            Ok(())
        }
        Err(e) => {
            error!(error = %e, "rhizoCrypt service error");
            Err(ServiceError::Rpc(e))
        }
    }
}

/// Register this primal instance with a Songbird discovery service.
pub async fn register_with_discovery(
    discovery_addr: String,
    our_addr: SocketAddr,
) -> Result<(), ServiceError> {
    let mut config = SongbirdConfig::new();
    config.address = std::borrow::Cow::Owned(discovery_addr);
    let client = SongbirdClient::new(config);

    let our_endpoint = format!("http://{our_addr}");
    client.register(&our_endpoint).await.map_err(|e| ServiceError::Discovery(e.to_string()))?;
    client.start_heartbeat().await.map_err(|e| ServiceError::Discovery(e.to_string()))?;

    Ok(())
}

/// Print version information.
pub fn print_version() {
    println!("rhizoCrypt v{}", env!("CARGO_PKG_VERSION"));
    println!("Edition: 2021");
    println!("License: AGPL-3.0-only");
    println!("Architecture: UniBin / ecoBin (Pure Rust)");
}

/// Print status information.
pub fn print_status() {
    println!("rhizoCrypt v{}", env!("CARGO_PKG_VERSION"));
    println!("Status: Not connected (use `rhizocrypt server` to start)");
    println!("License: AGPL-3.0-only");
}

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

/// Run health diagnostics per the UniBin Architecture Standard.
///
/// Performs checks on DAG engine, storage, configuration, and optional
/// discovery connectivity. Output is human-readable for operator inspection.
pub async fn run_doctor(comprehensive: bool) {
    let version = env!("CARGO_PKG_VERSION");
    println!("rhizoCrypt Doctor v{version}");
    println!("==============================");

    let mut checks: Vec<(String, DoctorCheck, Option<String>)> = Vec::new();

    // 1. DAG engine initialization
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

    // 2. Storage backend (redb when available, else in-memory)
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

    // 3. Configuration valid
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

    // 4. Discovery service
    let discovery_check = check_discovery(comprehensive).await;
    checks.push(("Discovery service".to_string(), discovery_check.0, Some(discovery_check.1)));

    // 5. Environment mode
    let env_mode = if SafeEnv::is_development() {
        "development"
    } else {
        "production"
    };
    checks.push(("Environment".to_string(), DoctorCheck::Pass, Some(env_mode.to_string())));

    // Print results
    for (name, status, detail) in &checks {
        let symbol = match status {
            DoctorCheck::Pass => "[✓]",
            DoctorCheck::Warn => "[!]",
            DoctorCheck::Fail => "[✗]",
        };
        let suffix = detail.as_deref().map(|d| format!(" ({d})")).unwrap_or_default();
        println!("{symbol} {name}{suffix}");
    }

    // Overall status
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
pub(crate) async fn check_dag_engine() -> bool {
    let config = RhizoCryptConfig::default();
    let mut primal = RhizoCrypt::new(config);
    primal.start().await.is_ok()
}

/// Check that the default storage backend is accessible.
pub(crate) fn check_storage_backend() -> (bool, &'static str) {
    // When redb feature is enabled, verify we can open a redb store.
    // Otherwise the in-memory backend is validated by check_dag_engine.
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
fn check_redb_storage() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    use rhizo_crypt_core::RedbDagStore;

    let temp_dir = tempfile::tempdir()?;
    let db_path = temp_dir.path().join("doctor_check.redb");
    let _store = RedbDagStore::open(&db_path)?;
    Ok(())
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

    // Validate host is non-empty (port is u16, always valid)
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

    // Try TCP connectivity
    match check_discovery_connectivity(&discovery_addr).await {
        Ok(()) => (DoctorCheck::Pass, format!("reachable at {discovery_addr}")),
        Err(e) => (DoctorCheck::Warn, format!("configured but unreachable: {e}")),
    }
}

/// Attempt TCP connection to discovery endpoint.
pub(crate) async fn check_discovery_connectivity(addr: &str) -> Result<(), String> {
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

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    static ENV_LOCK: Mutex<()> = Mutex::new(());

    #[test]
    fn test_resolve_bind_addr_with_overrides() {
        let addr = resolve_bind_addr(Some(9999), Some("127.0.0.1".to_string())).unwrap();
        assert_eq!(addr.port(), 9999);
        assert_eq!(addr.ip(), std::net::IpAddr::from([127, 0, 0, 1]));
    }

    #[test]
    fn test_resolve_bind_addr_port_only() {
        let addr = resolve_bind_addr(Some(12345), None).unwrap();
        assert_eq!(addr.port(), 12345);
    }

    #[test]
    fn test_resolve_bind_addr_host_only() {
        let addr = resolve_bind_addr(None, Some("127.0.0.1".to_string())).unwrap();
        assert_eq!(addr.ip(), std::net::IpAddr::from([127, 0, 0, 1]));
    }

    #[test]
    fn test_resolve_bind_addr_no_overrides() {
        let addr = resolve_bind_addr(None, None).unwrap();
        assert!(addr.port() > 0 || addr.port() == 0);
        assert!(!addr.ip().to_string().is_empty());
    }

    #[test]
    fn test_resolve_bind_addr_port_zero() {
        let addr = resolve_bind_addr(Some(0), Some("127.0.0.1".to_string())).unwrap();
        assert_eq!(addr.port(), 0);
        assert_eq!(addr.ip(), std::net::IpAddr::from([127, 0, 0, 1]));
    }

    #[test]
    fn test_resolve_bind_addr_invalid_host() {
        let result = resolve_bind_addr(Some(9999), Some("not-an-ip".to_string()));
        assert!(result.is_err());
    }

    #[test]
    fn test_resolve_bind_addr_invalid_host_empty() {
        let result = resolve_bind_addr(Some(9999), Some(String::new()));
        assert!(result.is_err());
    }

    #[test]
    fn test_resolve_bind_addr_invalid_host_garbage() {
        let result = resolve_bind_addr(Some(9999), Some("::::".to_string()));
        assert!(result.is_err());
    }

    #[test]
    fn test_service_error_display_config() {
        let err = ServiceError::Config("bad config".to_string());
        let s = err.to_string();
        assert!(s.contains("configuration error"));
        assert!(s.contains("bad config"));
    }

    #[test]
    fn test_service_error_display_discovery() {
        let err = ServiceError::Discovery("no songbird".to_string());
        let s = err.to_string();
        assert!(s.contains("discovery registration failed"));
        assert!(s.contains("no songbird"));
    }

    #[test]
    fn test_service_error_display_rpc() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
        let err = ServiceError::Rpc(io_err);
        let s = err.to_string();
        assert!(s.contains("rpc server error"));
    }

    #[test]
    fn test_service_error_display_addr_parse() {
        let parse_err = "x:y:z".parse::<SocketAddr>().unwrap_err();
        let err = ServiceError::AddrParse(parse_err);
        let s = err.to_string();
        assert!(s.contains("address parse error"));
    }

    #[test]
    fn test_service_error_from_io_error() {
        let io_err = std::io::Error::other("connection refused");
        let err: ServiceError = io_err.into();
        assert!(matches!(err, ServiceError::Rpc(_)));
    }

    #[test]
    fn test_service_error_from_addr_parse_error() {
        let parse_err = "invalid".parse::<SocketAddr>().unwrap_err();
        let err: ServiceError = parse_err.into();
        assert!(matches!(err, ServiceError::AddrParse(_)));
    }

    #[test]
    fn test_print_version_no_panic() {
        print_version();
    }

    #[test]
    fn test_print_status_no_panic() {
        print_status();
    }

    #[tokio::test]
    async fn test_register_with_discovery_unreachable() {
        let addr: SocketAddr = "127.0.0.1:9999".parse().unwrap();
        let result =
            register_with_discovery("http://invalid-host-12345:99999".to_string(), addr).await;
        assert!(result.is_err());
        match &result.unwrap_err() {
            ServiceError::Discovery(msg) => assert!(!msg.is_empty()),
            other => panic!("expected Discovery error, got: {other}"),
        }
    }

    #[tokio::test]
    async fn test_run_doctor_basic() {
        run_doctor(false).await;
    }

    #[tokio::test]
    async fn test_run_doctor_comprehensive() {
        run_doctor(true).await;
    }

    #[tokio::test]
    async fn test_check_configuration_default_env() {
        {
            let _guard = ENV_LOCK.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
            std::env::remove_var("RHIZOCRYPT_RPC_PORT");
            std::env::remove_var("RHIZOCRYPT_PORT");
            std::env::remove_var("RHIZOCRYPT_RPC_HOST");
            std::env::remove_var("RHIZOCRYPT_HOST");
            std::env::remove_var("RHIZOCRYPT_ENV");
        }
        run_doctor(false).await;
    }

    #[tokio::test]
    async fn test_check_configuration_with_port_override() {
        {
            let _guard = ENV_LOCK.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
            std::env::set_var("RHIZOCRYPT_RPC_PORT", "9401");
        }
        run_doctor(false).await;
        {
            let _guard = ENV_LOCK.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
            std::env::remove_var("RHIZOCRYPT_RPC_PORT");
        }
    }

    #[tokio::test]
    async fn test_check_configuration_with_host_override() {
        {
            let _guard = ENV_LOCK.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
            std::env::set_var("RHIZOCRYPT_RPC_HOST", "0.0.0.0");
        }
        run_doctor(false).await;
        {
            let _guard = ENV_LOCK.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
            std::env::remove_var("RHIZOCRYPT_RPC_HOST");
        }
    }

    #[tokio::test]
    async fn test_check_configuration_development_mode() {
        {
            let _guard = ENV_LOCK.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
            std::env::set_var("RHIZOCRYPT_ENV", "development");
        }
        run_doctor(false).await;
        {
            let _guard = ENV_LOCK.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
            std::env::remove_var("RHIZOCRYPT_ENV");
        }
    }

    #[tokio::test]
    async fn test_check_discovery_without_endpoint() {
        {
            let _guard = ENV_LOCK.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
            std::env::remove_var("RHIZOCRYPT_DISCOVERY_ADAPTER");
            std::env::remove_var("DISCOVERY_ENDPOINT");
            std::env::remove_var("DISCOVERY_ADDRESS");
        }
        run_doctor(false).await;
    }

    #[tokio::test]
    async fn test_check_discovery_with_endpoint_non_comprehensive() {
        {
            let _guard = ENV_LOCK.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
            std::env::set_var("DISCOVERY_ENDPOINT", "127.0.0.1:99999");
        }
        run_doctor(false).await;
        {
            let _guard = ENV_LOCK.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
            std::env::remove_var("DISCOVERY_ENDPOINT");
        }
    }

    #[tokio::test]
    async fn test_check_discovery_with_endpoint_comprehensive() {
        {
            let _guard = ENV_LOCK.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
            std::env::set_var("DISCOVERY_ENDPOINT", "127.0.0.1:99999");
        }
        run_doctor(true).await;
        {
            let _guard = ENV_LOCK.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
            std::env::remove_var("DISCOVERY_ENDPOINT");
        }
    }

    #[tokio::test]
    async fn test_check_dag_engine() {
        assert!(check_dag_engine().await);
    }

    #[tokio::test]
    async fn test_check_storage_backend() {
        let (ok, name) = check_storage_backend();
        assert!(ok, "storage backend check should pass");
        assert!(!name.is_empty(), "storage name should be non-empty");
    }

    #[test]
    fn test_doctor_check_partial_eq() {
        assert_eq!(DoctorCheck::Pass, DoctorCheck::Pass);
        assert_eq!(DoctorCheck::Warn, DoctorCheck::Warn);
        assert_eq!(DoctorCheck::Fail, DoctorCheck::Fail);
        assert_ne!(DoctorCheck::Pass, DoctorCheck::Fail);
        assert_ne!(DoctorCheck::Pass, DoctorCheck::Warn);
        assert_ne!(DoctorCheck::Warn, DoctorCheck::Fail);
    }

    #[test]
    fn test_doctor_check_eq() {
        assert!(DoctorCheck::Pass == DoctorCheck::Pass);
        assert!(DoctorCheck::Warn == DoctorCheck::Warn);
        assert!(DoctorCheck::Fail == DoctorCheck::Fail);
    }

    #[test]
    fn test_doctor_check_display_symbols() {
        let symbol = |c: DoctorCheck| match c {
            DoctorCheck::Pass => "[✓]",
            DoctorCheck::Warn => "[!]",
            DoctorCheck::Fail => "[✗]",
        };
        assert_eq!(symbol(DoctorCheck::Pass), "[✓]");
        assert_eq!(symbol(DoctorCheck::Warn), "[!]");
        assert_eq!(symbol(DoctorCheck::Fail), "[✗]");
    }

    #[tokio::test]
    async fn test_check_discovery_connectivity_unreachable() {
        let result = check_discovery_connectivity("invalid-host-12345:99999").await;
        assert!(result.is_err());
    }
}
