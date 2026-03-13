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
use rhizo_crypt_core::safe_env::SafeEnv;
use rhizo_crypt_core::primal::PrimalLifecycle;
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
    fn test_resolve_bind_addr_invalid_host() {
        let result = resolve_bind_addr(Some(9999), Some("not-an-ip".to_string()));
        assert!(result.is_err());
    }

    #[test]
    fn test_service_error_display() {
        let err = ServiceError::Config("bad config".to_string());
        assert!(err.to_string().contains("bad config"));

        let err = ServiceError::Discovery("no songbird".to_string());
        assert!(err.to_string().contains("no songbird"));
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
}
