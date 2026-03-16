// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2024–2026 ecoPrimals Project

//! `rhizoCrypt` service library — shared logic for the `UniBin` entry point.
//!
//! Extracts server startup, discovery registration, and CLI types so they can
//! be tested without spawning a subprocess.

#![cfg_attr(not(test), forbid(unsafe_code))]

pub use rhizo_crypt_core;

mod doctor;
pub use doctor::{
    DoctorCheck, check_dag_engine, check_discovery_connectivity, check_storage_backend, run_doctor,
};

/// `UniBin` exit codes per the Architecture Standard.
pub mod exit_codes {
    /// Success.
    pub const SUCCESS: i32 = 0;
    /// General error.
    pub const GENERAL_ERROR: i32 = 1;
    /// Configuration error.
    pub const CONFIG_ERROR: i32 = 2;
    /// Network error.
    pub const NETWORK_ERROR: i32 = 3;
    /// Interrupted (SIGINT).
    pub const INTERRUPTED: i32 = 130;
}
pub use rhizo_crypt_rpc;

use clap::Subcommand;
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

/// Client operations for interacting with a running rhizoCrypt server.
#[derive(Subcommand)]
pub enum ClientOperation {
    /// Check server health.
    Health,
    /// List active sessions.
    ListSessions,
    /// Get service metrics.
    Metrics,
}

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

    /// Storage backend error (doctor check).
    #[error("storage error: {0}")]
    Storage(String),
}

impl ServiceError {
    /// Map this error to a `UniBin` exit code.
    #[must_use]
    pub const fn exit_code(&self) -> i32 {
        use crate::exit_codes;
        match self {
            Self::Config(_) | Self::AddrParse(_) | Self::Storage(_) => exit_codes::CONFIG_ERROR,
            Self::Rpc(_) | Self::Discovery(_) => exit_codes::NETWORK_ERROR,
        }
    }
}

/// Run a client operation against a running rhizoCrypt server.
///
/// # Errors
///
/// Returns [`ServiceError::AddrParse`] if `address` is not a valid socket address.
/// Returns [`ServiceError::Config`] if the RPC call fails.
pub async fn run_client(address: &str, operation: ClientOperation) -> Result<(), ServiceError> {
    let addr: SocketAddr = address.parse()?;

    let client = rhizo_crypt_rpc::RpcClient::connect(addr)
        .await
        .map_err(|e| ServiceError::Config(format!("Failed to connect: {e}")))?;

    match operation {
        ClientOperation::Health => {
            let health = client
                .health()
                .await
                .map_err(|e| ServiceError::Config(format!("Health check failed: {e}")))?;
            println!(
                "{}",
                serde_json::to_string_pretty(&health).unwrap_or_else(|_| format!("{health:?}"))
            );
        }
        ClientOperation::ListSessions => {
            let sessions = client
                .list_sessions()
                .await
                .map_err(|e| ServiceError::Config(format!("List sessions failed: {e}")))?;
            println!(
                "{}",
                serde_json::to_string_pretty(&sessions).unwrap_or_else(|_| format!("{sessions:?}"))
            );
        }
        ClientOperation::Metrics => {
            let metrics = client
                .metrics()
                .await
                .map_err(|e| ServiceError::Config(format!("Metrics failed: {e}")))?;
            println!(
                "{}",
                serde_json::to_string_pretty(&metrics).unwrap_or_else(|_| format!("{metrics:?}"))
            );
        }
    }

    Ok(())
}

/// Resolve the bind address from CLI overrides + environment.
///
/// # Errors
///
/// Returns [`ServiceError::AddrParse`] if the resulting host:port cannot be parsed.
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

/// Wait for SIGTERM or SIGINT (Unix) or Ctrl+C (other platforms).
async fn shutdown_signal() {
    #[cfg(unix)]
    {
        use tokio::signal::unix::{SignalKind, signal};
        let Ok(mut sigterm) = signal(SignalKind::terminate()) else {
            warn!("Failed to register SIGTERM handler, falling back to ctrl_c");
            let _ = tokio::signal::ctrl_c().await;
            return;
        };
        let Ok(mut sigint) = signal(SignalKind::interrupt()) else {
            warn!("Failed to register SIGINT handler, falling back to ctrl_c");
            let _ = tokio::signal::ctrl_c().await;
            return;
        };
        tokio::select! {
            _ = sigterm.recv() => {}
            _ = sigint.recv() => {}
        }
    }

    #[cfg(not(unix))]
    {
        let _ = tokio::signal::ctrl_c().await;
    }
}

/// Start the RPC server (tarpc + JSON-RPC) with optional discovery registration.
///
/// # Errors
///
/// Returns [`ServiceError::AddrParse`] if the bind address is invalid.
/// Returns [`ServiceError::Config`] if the DAG engine fails to start.
/// Returns [`ServiceError::Rpc`] if the RPC server encounters a fatal I/O error.
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

    let shutdown_tx = server.shutdown_sender();
    let serve_handle = tokio::spawn(async move { server.serve().await });

    tokio::pin!(serve_handle);

    tokio::select! {
        result = &mut serve_handle => {
            match result {
                Ok(Ok(())) => {
                    info!("rhizoCrypt service shutdown cleanly");
                    Ok(())
                }
                Ok(Err(e)) => {
                    error!(error = %e, "rhizoCrypt service error");
                    Err(ServiceError::Rpc(e))
                }
                Err(e) => {
                    error!(error = %e, "server task panicked");
                    Err(ServiceError::Config(format!("server task panicked: {e}")))
                }
            }
        }
        () = shutdown_signal() => {
            info!("Received shutdown signal, stopping gracefully");
            let _ = shutdown_tx.send(true);
            if let Ok(Err(e)) = serve_handle.await {
                error!(error = %e, "rhizoCrypt service error during shutdown");
                return Err(ServiceError::Rpc(e));
            }
            info!("rhizoCrypt service shutdown cleanly");
            Ok(())
        }
    }
}

/// Register this primal instance with a Songbird discovery service.
///
/// # Errors
///
/// Returns [`ServiceError::Discovery`] if registration or heartbeat setup fails.
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
    println!("License: AGPL-3.0-or-later");
    println!("Architecture: UniBin / ecoBin (Pure Rust)");
}

/// Print status information.
pub fn print_status() {
    println!("rhizoCrypt v{}", env!("CARGO_PKG_VERSION"));
    println!("Status: Not connected (use `rhizocrypt server` to start)");
    println!("License: AGPL-3.0-or-later");
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

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
        let err = ServiceError::Discovery("no discovery adapter available".to_string());
        let s = err.to_string();
        assert!(s.contains("discovery registration failed"));
        assert!(s.contains("no discovery adapter"));
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
    fn test_exit_code_constants() {
        use crate::exit_codes;
        assert_eq!(exit_codes::SUCCESS, 0);
        assert_eq!(exit_codes::GENERAL_ERROR, 1);
        assert_eq!(exit_codes::CONFIG_ERROR, 2);
        assert_eq!(exit_codes::NETWORK_ERROR, 3);
        assert_eq!(exit_codes::INTERRUPTED, 130);
    }

    #[test]
    fn test_service_error_exit_code_mapping() {
        use crate::exit_codes;
        let config_err = ServiceError::Config("bad".to_string());
        assert_eq!(config_err.exit_code(), exit_codes::CONFIG_ERROR);

        let rpc_err = ServiceError::Rpc(std::io::Error::other("connection refused"));
        assert_eq!(rpc_err.exit_code(), exit_codes::NETWORK_ERROR);

        let discovery_err = ServiceError::Discovery("unreachable".to_string());
        assert_eq!(discovery_err.exit_code(), exit_codes::NETWORK_ERROR);

        let parse_err = "x:y:z".parse::<SocketAddr>().unwrap_err();
        let addr_err = ServiceError::AddrParse(parse_err);
        assert_eq!(addr_err.exit_code(), exit_codes::CONFIG_ERROR);
    }

    #[tokio::test]
    async fn test_shutdown_signal_does_not_panic() {
        use tokio::time::{Duration, timeout};
        let result = timeout(Duration::from_millis(100), super::shutdown_signal()).await;
        assert!(result.is_err(), "shutdown_signal should block until signal (timeout expected)");
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

    #[test]
    fn test_check_configuration_default_env() {
        temp_env::with_vars(
            [
                ("RHIZOCRYPT_RPC_PORT", None::<&str>),
                ("RHIZOCRYPT_PORT", None),
                ("RHIZOCRYPT_RPC_HOST", None),
                ("RHIZOCRYPT_HOST", None),
                ("RHIZOCRYPT_ENV", None),
            ],
            || {
                let rt = tokio::runtime::Builder::new_multi_thread()
                    .worker_threads(2)
                    .enable_all()
                    .build()
                    .unwrap();
                rt.block_on(run_doctor(false));
            },
        );
    }

    #[test]
    fn test_check_configuration_with_port_override() {
        temp_env::with_vars([("RHIZOCRYPT_RPC_PORT", Some("9401"))], || {
            let rt = tokio::runtime::Builder::new_multi_thread()
                .worker_threads(2)
                .enable_all()
                .build()
                .unwrap();
            rt.block_on(run_doctor(false));
        });
    }

    #[test]
    fn test_check_configuration_with_host_override() {
        temp_env::with_vars([("RHIZOCRYPT_RPC_HOST", Some("0.0.0.0"))], || {
            let rt = tokio::runtime::Builder::new_multi_thread()
                .worker_threads(2)
                .enable_all()
                .build()
                .unwrap();
            rt.block_on(run_doctor(false));
        });
    }

    #[test]
    fn test_check_configuration_development_mode() {
        temp_env::with_vars([("RHIZOCRYPT_ENV", Some("development"))], || {
            let rt = tokio::runtime::Builder::new_multi_thread()
                .worker_threads(2)
                .enable_all()
                .build()
                .unwrap();
            rt.block_on(run_doctor(false));
        });
    }

    #[test]
    fn test_check_discovery_without_endpoint() {
        temp_env::with_vars(
            [
                ("RHIZOCRYPT_DISCOVERY_ADAPTER", None::<&str>),
                ("DISCOVERY_ENDPOINT", None),
                ("DISCOVERY_ADDRESS", None),
            ],
            || {
                let rt = tokio::runtime::Builder::new_multi_thread()
                    .worker_threads(2)
                    .enable_all()
                    .build()
                    .unwrap();
                rt.block_on(run_doctor(false));
            },
        );
    }

    #[test]
    fn test_check_discovery_with_endpoint_non_comprehensive() {
        temp_env::with_vars([("DISCOVERY_ENDPOINT", Some("127.0.0.1:99999"))], || {
            let rt = tokio::runtime::Builder::new_multi_thread()
                .worker_threads(2)
                .enable_all()
                .build()
                .unwrap();
            rt.block_on(run_doctor(false));
        });
    }

    #[test]
    fn test_check_discovery_with_endpoint_comprehensive() {
        temp_env::with_vars([("DISCOVERY_ENDPOINT", Some("127.0.0.1:99999"))], || {
            let rt = tokio::runtime::Builder::new_multi_thread()
                .worker_threads(2)
                .enable_all()
                .build()
                .unwrap();
            rt.block_on(run_doctor(true));
        });
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
