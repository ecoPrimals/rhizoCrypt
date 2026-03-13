// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2024–2026 ecoPrimals Project

//! `rhizoCrypt` — Ephemeral DAG Engine (UniBin)
//!
//! Single binary, multiple modes via subcommands per the UniBin architecture standard.
//!
//! ## Usage
//!
//! ```bash
//! rhizocrypt server                    # Start the RPC service
//! rhizocrypt server --port 9400        # Custom port
//! rhizocrypt status                    # Check service health
//! rhizocrypt --version                 # Version info
//! rhizocrypt --help                    # Help
//! ```
//!
//! ## Environment Variables
//!
//! - `RHIZOCRYPT_PORT` - RPC server port (default: OS-assigned in dev, 9400 production)
//! - `RHIZOCRYPT_HOST` - Bind address (default: 0.0.0.0)
//! - `DISCOVERY_ENDPOINT` or `SONGBIRD_ADDRESS` - Discovery service for registration
//! - `RHIZOCRYPT_ENV` - Environment mode (development/production)

#![forbid(unsafe_code)]

use clap::{Parser, Subcommand};
use rhizo_crypt_core::clients::songbird::{SongbirdClient, SongbirdConfig};
use rhizo_crypt_core::safe_env::SafeEnv;
use rhizo_crypt_core::{RhizoCrypt, RhizoCryptConfig};
use rhizo_crypt_rpc::server::RpcServer;
use std::net::SocketAddr;
use std::sync::Arc;
use tracing::{error, info, warn};

/// rhizoCrypt — Ephemeral DAG Engine for the ecoPrimals ecosystem.
///
/// Provides git-like DAG operations for capturing, linking, and committing
/// events to permanent storage. Pure Rust, capability-based, sovereign.
#[derive(Parser)]
#[command(
    name = "rhizocrypt",
    version,
    about = "rhizoCrypt — Ephemeral DAG Engine",
    long_about = "Ephemeral DAG engine for the ecoPrimals ecosystem.\n\
                  Captures, links, and commits events to permanent storage.\n\
                  Pure Rust. Capability-based. Sovereign."
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

/// Available subcommands.
#[derive(Subcommand)]
enum Commands {
    /// Start the RPC service.
    Server {
        /// Port to bind to (overrides RHIZOCRYPT_PORT env var).
        #[arg(short, long)]
        port: Option<u16>,

        /// Host address to bind to (overrides RHIZOCRYPT_HOST env var).
        #[arg(long)]
        host: Option<String>,
    },

    /// Show service status and version information.
    Status,

    /// Print version and build information.
    Version,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Server {
            port,
            host,
        } => run_server(port, host).await,
        Commands::Status => {
            println!("rhizoCrypt v{}", env!("CARGO_PKG_VERSION"));
            println!("Status: Not connected (use `rhizocrypt server` to start)");
            println!("License: AGPL-3.0-only");
            Ok(())
        }
        Commands::Version => {
            println!("rhizoCrypt v{}", env!("CARGO_PKG_VERSION"));
            println!("Edition: 2021");
            println!("License: AGPL-3.0-only");
            println!("Architecture: UniBin / ecoBin (Pure Rust)");
            Ok(())
        }
    }
}

async fn run_server(
    port_override: Option<u16>,
    host_override: Option<String>,
) -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    info!("Starting rhizoCrypt service...");

    let default_port = if SafeEnv::is_development() {
        0
    } else {
        9400
    };
    let port = port_override.unwrap_or_else(|| SafeEnv::get_rpc_port(default_port));
    let host = host_override.unwrap_or_else(SafeEnv::get_rpc_host);
    let addr: SocketAddr = format!("{host}:{port}").parse()?;

    info!(address = %addr, "Binding RPC server");

    let config = RhizoCryptConfig::default();
    let primal = Arc::new(RhizoCrypt::new(config));
    info!("DAG engine initialized");

    let server = RpcServer::new(primal, addr);

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
            Err(e.into())
        }
    }
}

async fn register_with_discovery(
    discovery_addr: String,
    our_addr: SocketAddr,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut config = SongbirdConfig::new();
    config.address = std::borrow::Cow::Owned(discovery_addr);
    let client = SongbirdClient::new(config);

    let our_endpoint = format!("http://{our_addr}");
    client.register(&our_endpoint).await?;
    let _ = client.start_heartbeat().await;

    Ok(())
}
