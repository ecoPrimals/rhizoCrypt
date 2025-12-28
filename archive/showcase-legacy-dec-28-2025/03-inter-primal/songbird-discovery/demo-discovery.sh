#!/bin/bash
#
# 🔐 rhizoCrypt Songbird Discovery Demo
#
# Demonstrates capability-based primal discovery.
#

set -e

# Colors
BLUE='\033[0;34m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
PURPLE='\033[0;35m'
CYAN='\033[0;36m'
NC='\033[0m'

log() { echo -e "${BLUE}[$(date '+%H:%M:%S')]${NC} $1"; }
success() { echo -e "${GREEN}✓${NC} $1"; }
warn() { echo -e "${YELLOW}⚠${NC} $1"; }

echo -e "${PURPLE}"
cat << 'EOF'
╔═══════════════════════════════════════════════════════════════╗
║         🔐 rhizoCrypt Capability Discovery Demo                ║
║                                                                ║
║  Demonstrates runtime primal discovery via Songbird            ║
╚═══════════════════════════════════════════════════════════════╝
EOF
echo -e "${NC}"

# Get the path to rhizoCrypt BEFORE changing directories
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
RHIZO_PATH="$(cd "$SCRIPT_DIR/../../.." && pwd)"

DEMO_DIR="/tmp/rhizocrypt-discovery-demo"
rm -rf "$DEMO_DIR"
mkdir -p "$DEMO_DIR"
cd "$DEMO_DIR"

cat > Cargo.toml << EOF
[package]
name = "discovery-demo"
version = "0.1.0"
edition = "2021"

[dependencies]
rhizo-crypt-core = { path = "$RHIZO_PATH/crates/rhizo-crypt-core" }
tokio = { version = "1.0", features = ["full"] }
EOF

mkdir -p src

cat > src/main.rs << 'EOF'
//! Capability Discovery Demo
//!
//! Shows how rhizoCrypt discovers other primals at runtime.

use rhizo_crypt_core::discovery::{
    Capability, DiscoveryRegistry, ServiceEndpoint,
};
use std::net::SocketAddr;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n🔐 rhizoCrypt Capability Discovery Demo\n");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");

    // Create discovery registry with self-knowledge
    let registry = DiscoveryRegistry::new("rhizocrypt-demo");
    println!("📦 Created DiscoveryRegistry");
    println!("   Local primal: {}\n", registry.local_name());

    // Simulate service announcements (in production, Songbird does this)
    println!("📡 Simulating service announcements...\n");

    // BearDog (DID verification)
    let beardog = ServiceEndpoint::new(
        "beardog-alpha",
        "192.0.2.10:8443".parse::<SocketAddr>()?,
        vec![Capability::DidVerification, Capability::Signing],
    );
    registry.register_endpoint(beardog).await;
    println!("   ✓ BearDog registered: DID verification, signing");

    // NestGate (payload storage)
    let nestgate = ServiceEndpoint::new(
        "nestgate-primary",
        "192.0.2.11:7878".parse::<SocketAddr>()?,
        vec![Capability::PayloadStorage, Capability::PayloadRetrieval],
    );
    registry.register_endpoint(nestgate).await;
    println!("   ✓ NestGate registered: payload storage/retrieval");

    // LoamSpine (permanent commits)
    let loamspine = ServiceEndpoint::new(
        "loamspine-node-1",
        "192.0.2.12:9090".parse::<SocketAddr>()?,
        vec![Capability::PermanentCommit, Capability::SliceCheckout],
    );
    registry.register_endpoint(loamspine).await;
    println!("   ✓ LoamSpine registered: permanent commits, slices");

    // Songbird (orchestration)
    let songbird = ServiceEndpoint::new(
        "songbird-tower",
        "192.0.2.1:8080".parse::<SocketAddr>()?,
        vec![Capability::ServiceDiscovery],
    );
    registry.register_endpoint(songbird).await;
    println!("   ✓ Songbird registered: service discovery");

    // ToadStool (compute)
    let toadstool = ServiceEndpoint::new(
        "toadstool-gpu-1",
        "192.0.2.20:7777".parse::<SocketAddr>()?,
        vec![Capability::ComputeOrchestration, Capability::ComputeEvents],
    );
    registry.register_endpoint(toadstool).await;
    println!("   ✓ ToadStool registered: compute orchestration/events");

    // SweetGrass (provenance)
    let sweetgrass = ServiceEndpoint::new(
        "sweetgrass-index",
        "192.0.2.21:6060".parse::<SocketAddr>()?,
        vec![Capability::ProvenanceQuery, Capability::Attribution],
    );
    registry.register_endpoint(sweetgrass).await;
    println!("   ✓ SweetGrass registered: provenance/attribution");

    println!();

    // Query by capability
    println!("🔍 Querying by capability...\n");

    // Find DID verification service
    if let Some(endpoint) = registry.get_endpoint(&Capability::DidVerification).await {
        println!("   DID Verification: {} @ {}", endpoint.primal_name, endpoint.addr);
    }

    // Find payload storage
    if let Some(endpoint) = registry.get_endpoint(&Capability::PayloadStorage).await {
        println!("   Payload Storage: {} @ {}", endpoint.primal_name, endpoint.addr);
    }

    // Find permanent commits
    if let Some(endpoint) = registry.get_endpoint(&Capability::PermanentCommit).await {
        println!("   Permanent Commit: {} @ {}", endpoint.primal_name, endpoint.addr);
    }

    // Find compute
    if let Some(endpoint) = registry.get_endpoint(&Capability::ComputeOrchestration).await {
        println!("   Compute: {} @ {}", endpoint.primal_name, endpoint.addr);
    }

    // Find provenance
    if let Some(endpoint) = registry.get_endpoint(&Capability::ProvenanceQuery).await {
        println!("   Provenance: {} @ {}", endpoint.primal_name, endpoint.addr);
    }

    println!();

    // Show all registered services
    println!("📋 All registered services:\n");
    for endpoint in registry.all_endpoints().await {
        println!("   {} ({:?})", endpoint.primal_name, endpoint.capabilities);
        println!("      └─ {}", endpoint.addr);
    }

    println!();

    // Demonstrate graceful degradation
    println!("🛡️  Demonstrating graceful degradation...\n");

    // Query for a capability that doesn't exist
    let custom_cap = Capability::custom("fancy-ml");
    if registry.get_endpoint(&custom_cap).await.is_none() {
        println!("   ✓ Capability 'fancy-ml' not found - system continues gracefully");
    }

    // Show availability check
    let signing_available = registry.is_available(&Capability::Signing).await;
    let custom_available = registry.is_available(&custom_cap).await;
    
    println!();
    println!("📊 Availability Status:");
    println!("   Signing: {}", if signing_available { "✅ Available" } else { "❌ Unavailable" });
    println!("   fancy-ml: {}", if custom_available { "✅ Available" } else { "❌ Unavailable" });

    println!();
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("✅ Discovery Demo completed!");
    println!();
    println!("Key Takeaways:");
    println!("  • Primals register capabilities, not addresses");
    println!("  • Discovery is runtime, not compile-time");
    println!("  • Missing capabilities handled gracefully");
    println!("  • No hardcoded primal addresses!");
    println!();

    Ok(())
}
EOF

log "Building demo..."
cargo build --release 2>&1 | grep -E "(Compiling|Finished|error)" || true

log "Running discovery demo..."
echo ""
cargo run --release 2>&1

rm -rf "$DEMO_DIR"
success "Demo complete!"
