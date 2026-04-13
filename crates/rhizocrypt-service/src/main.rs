// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2024–2026 ecoPrimals Project

//! `rhizoCrypt` — Ephemeral DAG Engine (`UniBin`)
//!
//! Single binary, multiple modes via subcommands per the `UniBin` architecture standard.
//!
//! ## Usage
//!
//! ```bash
//! rhizocrypt server                       # UDS-only (default socket)
//! rhizocrypt server --port 9400           # UDS + TCP (opt-in)
//! rhizocrypt server --unix /tmp/rc.sock   # UDS at custom path
//! rhizocrypt status                       # Check service health
//! rhizocrypt doctor                       # Health diagnostics (UniBin standard)
//! rhizocrypt doctor --comprehensive       # Detailed checks including discovery
//! rhizocrypt client health                # Check server health (UniBin standard)
//! rhizocrypt client --address HOST:PORT list-sessions  # List sessions
//! rhizocrypt --version                    # Version info
//! rhizocrypt --help                       # Help
//! ```
//!
//! ## Transport Model
//!
//! UDS is unconditional on Unix (Provenance Trio standard). TCP is opt-in via
//! `--port`, `RHIZOCRYPT_PORT`, or `RHIZOCRYPT_JSONRPC_PORT`.
//!
//! ## Environment Variables
//!
//! - `RHIZOCRYPT_PORT` - Opt-in TCP: tarpc port (triggers TCP transport)
//! - `RHIZOCRYPT_JSONRPC_PORT` - Opt-in TCP: JSON-RPC port
//! - `RHIZOCRYPT_HOST` - TCP bind address (default: 0.0.0.0)
//! - `RHIZOCRYPT_DISCOVERY_ADAPTER` or `DISCOVERY_ENDPOINT` - Discovery adapter for registration
//! - `RHIZOCRYPT_ENV` - Environment mode (development/production)

#![forbid(unsafe_code)]

use clap::{Parser, Subcommand};
use rhizo_crypt_core::constants;
use rhizocrypt_service::ClientOperation;

fn default_client_address() -> String {
    format!("{}:{}", constants::LOCALHOST, constants::PRODUCTION_RPC_PORT)
}

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
    /// Start the RPC service (UDS unconditional, TCP opt-in).
    Server {
        /// Opt-in TCP: bind tarpc + JSON-RPC on this port.
        ///
        /// When omitted and no `RHIZOCRYPT_PORT` env var is set, only the
        /// UDS transport is started (Provenance Trio standard).
        #[arg(short, long)]
        port: Option<u16>,

        /// TCP bind address (only used when TCP is active).
        #[arg(long)]
        host: Option<String>,

        /// Override the UDS socket path.
        ///
        /// UDS is always active on Unix. Use this to override the default
        /// `$XDG_RUNTIME_DIR/biomeos/rhizocrypt.sock` path.
        #[arg(long, value_name = "PATH", num_args = 0..=1, default_missing_value = "")]
        unix: Option<String>,
    },

    /// Show service status and version information.
    Status,

    /// Print version and build information.
    Version,

    /// Run health diagnostics (`UniBin` Architecture Standard).
    Doctor {
        /// Run detailed checks including discovery connectivity.
        #[arg(long)]
        comprehensive: bool,
    },

    /// Connect to a running rhizoCrypt server and execute RPC commands.
    Client {
        /// Server address to connect to.
        #[arg(short, long, default_value_t = default_client_address())]
        address: String,

        /// Client operation to perform.
        #[command(subcommand)]
        operation: ClientOperation,
    },
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    let result = match cli.command {
        Commands::Server {
            port,
            host,
            unix,
        } => {
            #[cfg(unix)]
            let uds = Some(unix.unwrap_or_default());
            #[cfg(not(unix))]
            let uds = unix;
            rhizocrypt_service::run_server(port, host, uds).await
        }
        Commands::Status => {
            rhizocrypt_service::print_status();
            Ok(())
        }
        Commands::Version => {
            rhizocrypt_service::print_version();
            Ok(())
        }
        Commands::Doctor {
            comprehensive,
        } => {
            rhizocrypt_service::run_doctor(comprehensive).await;
            Ok(())
        }
        Commands::Client {
            address,
            operation,
        } => rhizocrypt_service::run_client(&address, operation).await,
    };

    if let Err(e) = result {
        eprintln!("Error: {e}");
        std::process::exit(e.exit_code());
    }
}
