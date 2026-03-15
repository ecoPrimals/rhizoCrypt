// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2024–2026 ecoPrimals Project

//! `rhizoCrypt` — Ephemeral DAG Engine (`UniBin`)
//!
//! Single binary, multiple modes via subcommands per the `UniBin` architecture standard.
//!
//! ## Usage
//!
//! ```bash
//! rhizocrypt server                    # Start the RPC service
//! rhizocrypt server --port 9400        # Custom port
//! rhizocrypt status                    # Check service health
//! rhizocrypt doctor                    # Health diagnostics (UniBin standard)
//! rhizocrypt doctor --comprehensive    # Detailed checks including discovery
//! rhizocrypt client health             # Check server health (UniBin standard)
//! rhizocrypt client --address HOST:PORT list-sessions  # List sessions
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
    /// Start the RPC service.
    Server {
        /// Port to bind to (overrides `RHIZOCRYPT_PORT` env var).
        #[arg(short, long)]
        port: Option<u16>,

        /// Host address to bind to (overrides `RHIZOCRYPT_HOST` env var).
        #[arg(long)]
        host: Option<String>,
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
        } => rhizocrypt_service::run_server(port, host).await,
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
