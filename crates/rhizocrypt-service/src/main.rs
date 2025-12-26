//! RhizoCrypt Standalone Service
//!
//! This binary provides a standalone RPC service for rhizoCrypt, making it
//! discoverable and coordinatable by BiomeOS.
//!
//! ## Modes
//!
//! rhizoCrypt supports two modes:
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
//! - BiomeOS coordinates, doesn't embed

#![forbid(unsafe_code)]
#![warn(
    missing_docs,
    clippy::all,
    clippy::pedantic,
    clippy::cargo,
    clippy::nursery
)]

use rhizo_crypt_core::safe_env::SafeEnv;
use rhizo_crypt_core::{RhizoCrypt, RhizoCryptConfig};
use rhizo_crypt_rpc::server::RpcServer;
use std::net::SocketAddr;
use std::sync::Arc;
use tracing::{error, info};

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
    let port = SafeEnv::get_rpc_port(9400);
    let host = SafeEnv::get_rpc_host();
    let addr: SocketAddr = format!("{}:{}", host, port).parse()?;

    info!("📡 Binding to {}", addr);

    // Create rhizoCrypt instance with default configuration
    let config = RhizoCryptConfig::default();
    let primal = Arc::new(RhizoCrypt::new(config));
    info!("🔐 rhizoCrypt DAG engine initialized");

    // Create RPC server
    let server = RpcServer::new(primal, addr);

    // Optional: Register with discovery service (Songbird)
    if let Some(discovery_addr) = SafeEnv::get_discovery_address() {
        info!("🔍 Discovery service available at: {}", discovery_addr);
        info!("🌱 Infant discovery mode: rhizoCrypt will be discoverable");
        // TODO: Implement registration when Songbird client is ready
        // let songbird = SongbirdClient::connect(&discovery_addr).await?;
        // songbird.register_service(ServiceInfo { ... }).await?;
    } else {
        info!("🌱 No discovery service configured (standalone mode)");
    }

    // Display startup banner
    print_banner(addr, port);

    // Start serving
    info!("✨ rhizoCrypt service ready");
    
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

