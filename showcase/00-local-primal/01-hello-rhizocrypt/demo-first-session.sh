#!/bin/bash
#
# 🔐 rhizoCrypt - Your First Session Demo
#
# Demonstrates:
# 1. Creating a session
# 2. Session lifecycle states
# 3. Querying session info
#

set -e

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
PURPLE='\033[0;35m'
CYAN='\033[0;36m'
NC='\033[0m'

# Logging
log() { echo -e "${BLUE}[$(date '+%H:%M:%S')]${NC} $1"; }
success() { echo -e "${GREEN}✓${NC} $1"; }
info() { echo -e "${CYAN}ℹ${NC} $1"; }

# Banner
echo -e "${PURPLE}"
cat << 'EOF'
╔══════════════════════════════════════════════════════════╗
║        🔐 Your First rhizoCrypt Session                   ║
║                                                           ║
║  Learn: Session creation, lifecycle, and queries          ║
╚══════════════════════════════════════════════════════════╝
EOF
echo -e "${NC}"

# Get paths
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
RHIZO_ROOT="$(cd "$SCRIPT_DIR/../../.." && pwd)"

log "Building rhizoCrypt..."
cd "$RHIZO_ROOT"
cargo build --quiet 2>/dev/null || cargo build

echo ""
log "Creating Rust demo program..."

# Create temporary Rust program
TEMP_DIR=$(mktemp -d)
cat > "$TEMP_DIR/Cargo.toml" << CARGO_EOF
[package]
name = "first-session-demo"
version = "0.1.0"
edition = "2021"

[dependencies]
rhizo-crypt-core = { path = "$RHIZO_ROOT/crates/rhizo-crypt-core" }
tokio = { version = "1.46", features = ["full"] }
CARGO_EOF

mkdir -p "$TEMP_DIR/src"
cat > "$TEMP_DIR/src/main.rs" << 'RUST_EOF'
use rhizo_crypt_core::{RhizoCrypt, RhizoCryptConfig, SessionType, PrimalLifecycle, session::SessionBuilder, Did};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n🔐 Creating your first rhizoCrypt session...\n");
    
    // Create rhizoCrypt instance with default config
    let config = RhizoCryptConfig::default();
    let mut rhizo = RhizoCrypt::new(config);
    
    // Start the primal
    rhizo.start().await?;
    println!("✓ rhizoCrypt primal started");
    
    // Create a session using builder pattern
    let session = SessionBuilder::new(SessionType::General)
        .with_name("my-first-session")
        .with_owner(Did::new("did:key:alice"))
        .build();
    let session_id = session.id;
    
    rhizo.create_session(session).await?;
    println!("✓ Session created");
    println!("  Session ID: {}", session_id);
    
    // Query session info (not async - lock-free read!)
    let session_info = rhizo.get_session(session_id)?;
    println!("\n📊 Session Information:");
    println!("  Name: {}", session_info.name.as_deref().unwrap_or("(unnamed)"));
    println!("  Type: {:?}", session_info.session_type);
    println!("  State: {:?}", session_info.state);
    println!("  Created: {}", session_info.created_at);
    println!("  Vertex Count: {}", session_info.vertex_count);
    println!("  Genesis Vertices: {}", session_info.genesis.len());
    println!("  Frontier Vertices: {}", session_info.frontier.len());
    
    // List all sessions (also lock-free!)
    let sessions = rhizo.list_sessions();
    println!("\n📋 Total Sessions: {}", sessions.len());
    
    println!("\n🎉 Success! You've created your first rhizoCrypt session!");
    println!("\n💡 Key Concepts:");
    println!("  • Sessions are scoped DAG workspaces");
    println!("  • Each session has a unique UUID v7 ID");
    println!("  • Sessions start in 'Active' state");
    println!("  • Sessions are ephemeral (in-memory)");
    println!("  • Genesis = vertices with no parents");
    println!("  • Frontier = vertices with no children (DAG tips)");
    println!("  • Lock-free reads = no blocking on queries!");
    
    // Cleanup
    rhizo.stop().await?;
    
    Ok(())
}
RUST_EOF

echo ""
log "Running demo..."
echo ""

cd "$TEMP_DIR"
cargo run --quiet 2>/dev/null || cargo run

# Cleanup
cd "$RHIZO_ROOT"
rm -rf "$TEMP_DIR"

echo ""
echo -e "${GREEN}════════════════════════════════════════════════════════${NC}"
echo -e "${GREEN}Demo Complete!${NC}"
echo ""
info "What you learned:"
echo "  1. How to create a rhizoCrypt session"
echo "  2. Session IDs are UUID v7 (time-ordered)"
echo "  3. Sessions have lifecycle states"
echo "  4. Sessions track genesis and frontier"
echo ""
info "Next demo:"
echo "  ./demo-first-vertex.sh  - Learn about content-addressed events"
echo ""
