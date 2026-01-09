//! `RhizoCrypt` Standalone Service
//!
//! This binary provides a standalone RPC service for `rhizoCrypt`, making it
//! discoverable and coordinatable by `BiomeOS`.
//!
//! ## Modes
//!
//! `rhizoCrypt` supports two modes:
//! 1. **Library Mode**: Embedded directly into other primals
//! 2. **Service Mode**: Standalone service discovered via capabilities
//!
//! ## Usage
//!
//! ```bash
//! # Default port (9400)
//! ./rhizocrypt-service
//!
//! # Custom port
//! RHIZOCRYPT_PORT=9400 ./rhizocrypt-service
//!
//! # With discovery registration
//! RHIZOCRYPT_PORT=9400 \
//! SONGBIRD_ADDRESS=songbird.local:7500 \
//! ./rhizocrypt-service
//! ```
//!
//! ## Environment Variables
//!
//! - `RHIZOCRYPT_PORT` - RPC server port (default: 9400)
//! - `RHIZOCRYPT_HOST` - Bind address (default: 0.0.0.0)
//! - `SONGBIRD_ADDRESS` - Discovery service for registration
//! - `RHIZOCRYPT_ENV` - Environment mode (development/production)
//!
//! ## Primal Sovereignty
//!
//! This service embodies primal sovereignty:
//! - Fully standalone and independently deployable
//! - Discoverable via capability-based discovery
//! - No hardcoded dependencies on other primals
//! - `BiomeOS` coordinates, doesn't embed

#![forbid(unsafe_code)]
#![warn(missing_docs, clippy::all, clippy::pedantic, clippy::cargo, clippy::nursery)]
#![allow(clippy::module_name_repetitions)]
#![allow(clippy::must_use_candidate)]
#![allow(clippy::missing_errors_doc)]
#![allow(clippy::missing_panics_doc)]
#![allow(clippy::doc_markdown)]
#![allow(clippy::uninlined_format_args)]
#![allow(clippy::redundant_clone)]
#![allow(clippy::field_reassign_with_default)]
#![allow(clippy::unnecessary_literal_bound)]
#![allow(clippy::match_same_arms)]
#![allow(clippy::multiple_crate_versions)]
#![allow(clippy::significant_drop_tightening)]

use rhizo_crypt_core::clients::songbird::{SongbirdClient, SongbirdConfig};
use rhizo_crypt_core::safe_env::SafeEnv;
use rhizo_crypt_core::{RhizoCrypt, RhizoCryptConfig};
use rhizo_crypt_rpc::server::RpcServer;
use std::net::SocketAddr;
use std::sync::Arc;
use tracing::{error, info, warn};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    info!("🔐 Starting rhizoCrypt service...");

    // Get configuration from environment
    // Use port 0 by default for OS-assigned port (avoids conflicts in tests)
    let default_port = if SafeEnv::is_development() {
        0
    } else {
        9400
    };
    let port = SafeEnv::get_rpc_port(default_port);
    let host = SafeEnv::get_rpc_host();
    let addr: SocketAddr = format!("{}:{}", host, port).parse()?;

    info!("📡 Will bind to {}:{}", host, port);

    // Create rhizoCrypt instance with default configuration
    let config = RhizoCryptConfig::default();
    let primal = Arc::new(RhizoCrypt::new(config));
    info!("🔐 rhizoCrypt DAG engine initialized");

    // Create RPC server (will bind when serve() is called)
    let server = RpcServer::new(primal, addr);

    // Optional: Register with discovery service (Songbird)
    if let Some(discovery_addr) = SafeEnv::get_discovery_address() {
        info!("🔍 Discovery service available at: {}", discovery_addr);
        info!("🌱 Infant discovery mode: registering with Songbird...");

        match register_with_songbird(discovery_addr.clone(), addr).await {
            Ok(()) => {
                info!("✅ Successfully registered with Songbird at {}", discovery_addr);
                info!("🌱 rhizoCrypt is now discoverable by other primals");
            }
            Err(e) => {
                warn!("⚠️  Failed to register with Songbird: {}", e);
                warn!("⚠️  Continuing in standalone mode (not discoverable)");
            }
        }
    } else {
        info!("🌱 No discovery service configured (standalone mode)");
    }

    // Display startup banner
    print_banner(addr, port);

    // Start serving (this binds the port)
    info!("✨ rhizoCrypt service ready - starting RPC server...");

    match server.serve().await {
        Ok(()) => {
            info!("👋 rhizoCrypt service shutdown cleanly");
            Ok(())
        }
        Err(e) => {
            error!("❌ rhizoCrypt service error: {}", e);
            Err(e.into())
        }
    }
}

/// Register rhizoCrypt service with Songbird discovery.
async fn register_with_songbird(
    songbird_addr: String,
    our_addr: SocketAddr,
) -> Result<(), Box<dyn std::error::Error>> {
    // Create Songbird client with owned string
    let mut config = SongbirdConfig::new();
    config.address = std::borrow::Cow::Owned(songbird_addr.clone());
    let client = SongbirdClient::new(config);

    // Register our service
    let our_endpoint = format!("http://{}", our_addr);
    client.register(&our_endpoint).await?;

    // Start heartbeat to maintain registration
    let _ = client.start_heartbeat().await;

    info!("🔄 Heartbeat started - maintaining registration with Songbird");

    Ok(())
}

fn print_banner(addr: SocketAddr, port: u16) {
    println!("\n{}", "═".repeat(78));
    println!("║");
    println!("║  🔐 rhizoCrypt - Ephemeral DAG Engine");
    println!("║");
    println!("║  Status:    🟢 Service Running");
    println!("║  Version:   v0.11.0");
    println!("║  Address:   {}", addr);
    println!("║  Port:      {}", port);
    println!("║  Mode:      Standalone Service (tarpc RPC)");
    println!("║");
    println!("║  Capabilities:");
    println!("║    • Session Management (create, list, get, drop)");
    println!("║    • Vertex Operations (add, get, list)");
    println!("║    • DAG Queries (parents, children, toposort)");
    println!("║    • Merkle Proofs (compute, verify)");
    println!("║    • Dehydration (commit ephemeral → permanent)");
    println!("║    • Slice Operations (checkout, resolve)");
    println!("║");
    println!("║  Primal Sovereignty:");
    println!("║    ✅ Fully standalone and deployable");
    println!("║    ✅ Discoverable via capability-based discovery");
    println!("║    ✅ Zero hardcoded dependencies");
    println!("║    ✅ BiomeOS coordinates, doesn't embed");
    println!("║");
    println!("║  Documentation:");
    println!("║    • README.md - Project overview");
    println!("║    • INFANT_DISCOVERY.md - Capability-based discovery");
    println!("║    • API: 24 RPC methods available");
    println!("║");
    println!("║  \"Like an infant, we start with zero knowledge and discover.\" 🌱");
    println!("║");
    println!("{}", "═".repeat(78));
    println!();
}
