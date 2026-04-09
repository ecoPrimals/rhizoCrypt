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
use rhizo_crypt_core::constants;
use rhizo_crypt_core::primal::PrimalLifecycle;
use rhizo_crypt_core::safe_env::SafeEnv;
use rhizo_crypt_core::{RhizoCrypt, RhizoCryptConfig};
use rhizo_crypt_rpc::jsonrpc::JsonRpcServer;
use rhizo_crypt_rpc::server::RpcServer;
use std::net::SocketAddr;
use std::path::PathBuf;
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

/// Start the RPC server (tarpc + JSON-RPC + optional UDS) with optional discovery registration.
///
/// `unix_socket`:
/// - `None` — no UDS listener
/// - `Some("")` — UDS at the default ecosystem path (`$XDG_RUNTIME_DIR/biomeos/rhizocrypt.sock`)
/// - `Some(path)` — UDS at the given custom path
///
/// # Errors
///
/// Returns [`ServiceError::AddrParse`] if the bind address is invalid.
/// Returns [`ServiceError::Config`] if the DAG engine fails to start.
/// Returns [`ServiceError::Rpc`] if the RPC server encounters a fatal I/O error.
pub async fn run_server(
    port_override: Option<u16>,
    host_override: Option<String>,
    unix_socket: Option<String>,
) -> Result<(), ServiceError> {
    run_server_with_ready(port_override, host_override, unix_socket, None).await
}

/// Run the server with an optional readiness notification.
///
/// When `ready` is `Some`, the notifier fires once the RPC server is bound
/// and accepting connections. Used by integration tests to avoid sleep-based
/// readiness polling.
///
/// # Errors
///
/// Returns [`ServiceError`] on bind, config, or runtime failures.
pub async fn run_server_with_ready(
    port_override: Option<u16>,
    host_override: Option<String>,
    unix_socket: Option<String>,
    ready: Option<Arc<tokio::sync::Notify>>,
) -> Result<(), ServiceError> {
    let _ = tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .try_init();

    info!("Starting rhizoCrypt service...");

    rhizo_crypt_core::transport::btsp_env_guard("RHIZOCRYPT")
        .map_err(|e| ServiceError::Config(e.to_string()))?;

    if rhizo_crypt_core::transport::is_biomeos_insecure() {
        warn!("BIOMEOS_INSECURE=1 — running in development mode (no BTSP handshake)");
    }

    if let Some(fid) = rhizo_crypt_core::transport::read_family_id("RHIZOCRYPT") {
        info!(family_id = %fid, "BTSP Phase 1: family-scoped socket naming active");
    }

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
    let jsonrpc_port = SafeEnv::get_jsonrpc_port(port);
    let jsonrpc_addr: SocketAddr = format!("{host}:{jsonrpc_port}").parse()?;
    let jsonrpc_server = JsonRpcServer::new(Arc::clone(&primal), jsonrpc_addr);
    tokio::spawn(async move {
        if let Err(e) = jsonrpc_server.serve().await {
            error!(error = %e, "JSON-RPC server error");
        }
    });
    info!(address = %jsonrpc_addr, "JSON-RPC server started (dual-mode: HTTP + newline)");

    #[cfg(unix)]
    let uds_shutdown_tx = start_uds_listener(unix_socket.as_ref(), &primal);

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
    let is_running = server.running_flag();
    let serve_handle = tokio::spawn(async move { server.serve().await });

    if let Some(notify) = ready {
        tokio::spawn(async move {
            loop {
                if is_running.load(std::sync::atomic::Ordering::SeqCst) {
                    notify.notify_one();
                    break;
                }
                tokio::task::yield_now().await;
            }
        });
    }

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
            #[cfg(unix)]
            {
                let _ = uds_shutdown_tx.send(true);
            }
            if let Ok(Err(e)) = serve_handle.await {
                error!(error = %e, "rhizoCrypt service error during shutdown");
                return Err(ServiceError::Rpc(e));
            }
            info!("rhizoCrypt service shutdown cleanly");
            Ok(())
        }
    }
}

/// Resolve UDS path from the CLI value.
///
/// Empty string → ecosystem default (`$XDG_RUNTIME_DIR/biomeos/rhizocrypt.sock`).
/// Non-empty → use as-is.
#[cfg(unix)]
fn resolve_uds_path(raw: &str) -> PathBuf {
    if raw.is_empty() {
        rhizo_crypt_rpc::jsonrpc::uds::default_socket_path()
    } else {
        PathBuf::from(raw)
    }
}

/// Optionally start the UDS JSON-RPC listener, returning the shutdown sender.
#[cfg(unix)]
fn start_uds_listener(
    unix_socket: Option<&String>,
    primal: &Arc<RhizoCrypt>,
) -> tokio::sync::watch::Sender<bool> {
    let (shutdown_tx, shutdown_rx) = tokio::sync::watch::channel(false);
    if let Some(raw_path) = unix_socket {
        let socket_path = resolve_uds_path(raw_path);
        info!(path = %socket_path.display(), "Starting UDS JSON-RPC listener");
        let uds_server =
            rhizo_crypt_rpc::jsonrpc::uds::UdsJsonRpcServer::new(Arc::clone(primal), socket_path);
        tokio::spawn(async move {
            if let Err(e) = uds_server.serve(shutdown_rx).await {
                error!(error = %e, "UDS JSON-RPC server error");
            }
        });
    }
    shutdown_tx
}

/// Register this primal with the configured discovery adapter.
///
/// The discovery adapter is the one bootstrap address a primal needs.
/// All other primals are discovered at runtime via capability queries.
/// This function is adapter-agnostic — any compatible endpoint
/// accepting `register` + `heartbeat` JSON-RPC methods will work.
///
/// # Errors
///
/// Returns [`ServiceError::Discovery`] if registration or heartbeat setup fails.
pub async fn register_with_discovery(
    discovery_addr: String,
    our_addr: SocketAddr,
) -> Result<(), ServiceError> {
    use rhizo_crypt_core::clients::songbird::{DiscoveryClient, DiscoveryConfig};

    let mut config = DiscoveryConfig::new();
    config.address = std::borrow::Cow::Owned(discovery_addr);
    let client = DiscoveryClient::new(config);

    let our_endpoint = format!("http://{our_addr}");
    client.register(&our_endpoint).await.map_err(|e| ServiceError::Discovery(e.to_string()))?;
    client.start_heartbeat().await.map_err(|e| ServiceError::Discovery(e.to_string()))?;

    Ok(())
}

/// Print version information.
pub fn print_version() {
    println!("rhizoCrypt v{}", env!("CARGO_PKG_VERSION"));
    println!("Edition: 2024");
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
#[expect(clippy::unwrap_used, reason = "test code")]
#[path = "lib_tests.rs"]
mod tests;
