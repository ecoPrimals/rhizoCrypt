#!/bin/bash
#
# 🔐 rhizoCrypt - Ephemeral vs Persistent Sessions Demo
#
# Demonstrates:
# 1. Ephemeral sessions (in-memory, no trace)
# 2. Persistent sessions (optionally saved)
# 3. When to use each type
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
║     🔄 Ephemeral vs Persistent Sessions 🔄                ║
║                                                           ║
║  Learn: Session types and when to use each               ║
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
log "Creating session types demo..."

# Create temporary Rust program
TEMP_DIR=$(mktemp -d)
cat > "$TEMP_DIR/Cargo.toml" << CARGO_EOF
[package]
name = "session-types-demo"
version = "0.1.0"
edition = "2021"

[dependencies]
rhizo-crypt-core = { path = "$RHIZO_ROOT/crates/rhizo-crypt-core" }
tokio = { version = "1.46", features = ["full"] }
CARGO_EOF

mkdir -p "$TEMP_DIR/src"
cat > "$TEMP_DIR/src/main.rs" << 'RUST_EOF'
use rhizo_crypt_core::{RhizoCrypt, RhizoCryptConfig, Session, SessionType, Vertex, EventType};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n🔄 Ephemeral vs Persistent Sessions...\n");
    
    // Create rhizoCrypt instance
    let config = RhizoCryptConfig::default();
    let rhizo = RhizoCrypt::new(config);
    rhizo.start().await?;
    
    let store = rhizo.dag_store().await?;
    
    // Demo 1: Ephemeral Session
    println!("📊 Demo 1: Ephemeral Session");
    println!("   (Default: Privacy-First, No Trace)");
    println!();
    
    let ephemeral = Session::new("ephemeral-demo".to_string(), SessionType::Ephemeral);
    let ephemeral_id = ephemeral.id;
    rhizo.create_session(ephemeral).await?;
    
    println!("   ✓ Session created: {}", ephemeral_id);
    println!("   Type: Ephemeral");
    println!();
    
    println!("   Characteristics:");
    println!("     • In-memory only (fast)");
    println!("     • No disk I/O");
    println!("     • Discarded on expire");
    println!("     • Leaves no trace (privacy)");
    println!("     • Default for most use cases");
    println!();
    
    // Add some vertices
    let v1 = Vertex::new(EventType::SessionStarted, Vec::new());
    let v1_id = v1.id.clone();
    store.put_vertex(ephemeral_id, v1).await?;
    
    let v2 = Vertex::new(EventType::DataCreated, vec![v1_id]);
    store.put_vertex(ephemeral_id, v2).await?;
    
    let count = store.count_vertices(ephemeral_id).await?;
    println!("   ✓ Added {} vertices (in-memory)", count);
    println!();
    
    println!("   When session expires:");
    println!("     → DAG discarded");
    println!("     → Memory released");
    println!("     → No permanent trace");
    println!();
    
    // Demo 2: Persistent Session
    println!("📊 Demo 2: Persistent Session");
    println!("   (Opt-In: For Audit Trails)");
    println!();
    
    let persistent = Session::new("persistent-demo".to_string(), SessionType::Persistent);
    let persistent_id = persistent.id;
    rhizo.create_session(persistent).await?;
    
    println!("   ✓ Session created: {}", persistent_id);
    println!("   Type: Persistent");
    println!();
    
    println!("   Characteristics:");
    println!("     • Can be saved to storage");
    println!("     • Requires explicit consent");
    println!("     • Audit trail preserved");
    println!("     • Can be resumed later");
    println!("     • Use for compliance");
    println!();
    
    // Add some vertices
    let v3 = Vertex::new(EventType::SessionStarted, Vec::new());
    let v3_id = v3.id.clone();
    store.put_vertex(persistent_id, v3).await?;
    
    let v4 = Vertex::new(EventType::DataCreated, vec![v3_id]);
    store.put_vertex(persistent_id, v4).await?;
    
    let count = store.count_vertices(persistent_id).await?;
    println!("   ✓ Added {} vertices", count);
    println!();
    
    println!("   When session expires:");
    println!("     → DAG can be saved (user choice)");
    println!("     → Dehydrate to LoamSpine (permanent storage)");
    println!("     → Audit trail preserved");
    println!();
    
    // Compare
    println!("⚖️  Comparison:");
    println!();
    println!("   ┌────────────────────┬─────────────┬─────────────┐");
    println!("   │ Feature            │ Ephemeral   │ Persistent  │");
    println!("   ├────────────────────┼─────────────┼─────────────┤");
    println!("   │ Storage            │ Memory only │ Can save    │");
    println!("   │ Speed              │ Fast        │ Fast        │");
    println!("   │ Privacy            │ High        │ Medium      │");
    println!("   │ Audit trail        │ No          │ Yes         │");
    println!("   │ Resumable          │ No          │ Yes         │");
    println!("   │ Default            │ Yes         │ No          │");
    println!("   │ User consent       │ Implicit    │ Explicit    │");
    println!("   └────────────────────┴─────────────┴─────────────┘");
    println!();
    
    // Use cases
    println!("🌟 Use Cases:");
    println!();
    println!("   Ephemeral (Default):");
    println!("     • Interactive computation");
    println!("     • Temporary aggregations");
    println!("     • Exploratory analysis");
    println!("     • Privacy-sensitive operations");
    println!("     • Most day-to-day work");
    println!();
    println!("   Persistent (Opt-In):");
    println!("     • Compliance audit logs");
    println!("     • Long-running workflows");
    println!("     • Multi-step processes");
    println!("     • Provenance tracking");
    println!("     • Financial transactions");
    println!();
    
    println!("🎉 Success! You understand session types!");
    println!("\n💡 Key Principles:");
    println!("  • Ephemeral by Default (privacy-first)");
    println!("  • Persistent by Consent (user control)");
    println!("  • Both types are fast (in-memory)");
    println!("  • Choice depends on use case");
    
    println!("\n🔐 Sovereignty & Human Dignity:");
    println!("  • User controls data persistence");
    println!("  • Explicit consent required for saving");
    println!("  • Ephemeral = no trace (privacy)");
    println!("  • Persistent = audit trail (transparency)");
    println!("  • No surveillance by default");
    
    // Cleanup
    rhizo.stop().await?;
    println!("\n✓ Sessions expired");
    println!("  • Ephemeral: DAG discarded, no trace");
    println!("  • Persistent: Could have been saved (not in this demo)");
    
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
echo "  1. Ephemeral sessions (default, no trace)"
echo "  2. Persistent sessions (opt-in, audit trail)"
echo "  3. When to use each type"
echo "  4. Sovereignty principles (user control)"
echo ""
info "Next demo:"
echo "  ./demo-slices.sh  - Checkout from permanent storage"
echo ""

