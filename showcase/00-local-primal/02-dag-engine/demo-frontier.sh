#!/bin/bash
#
# 🔐 rhizoCrypt - Frontier Tracking Demo
#
# Demonstrates:
# 1. Frontier = DAG tips (vertices with no children)
# 2. Automatic frontier updates
# 3. Frontier evolution as DAG grows
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
║          📍 Frontier Tracking - DAG Tips 📍               ║
║                                                           ║
║  Learn: Frontier = vertices with no children             ║
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
log "Creating frontier tracking demo..."

# Create temporary Rust program
TEMP_DIR=$(mktemp -d)
cat > "$TEMP_DIR/Cargo.toml" << CARGO_EOF
[package]
name = "frontier-demo"
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
    println!("\n📍 Tracking the Frontier as DAG Grows...\n");
    
    // Create rhizoCrypt instance
    let config = RhizoCryptConfig::default();
    let rhizo = RhizoCrypt::new(config);
    rhizo.start().await?;
    
    // Create a session
    let session = Session::new("frontier-demo".to_string(), SessionType::Ephemeral);
    let session_id = session.id;
    rhizo.create_session(session).await?;
    
    let store = rhizo.dag_store().await?;
    
    // Helper to show frontier
    let show_frontier = |step: &str, expected_count: usize| async move {
        let frontier = store.get_frontier(session_id).await?;
        println!("  Frontier: {} vertices", frontier.len());
        assert_eq!(frontier.len(), expected_count, "Unexpected frontier size at {}", step);
        for v_id in &frontier {
            println!("    - {}", v_id);
        }
        Ok::<(), Box<dyn std::error::Error>>(())
    };
    
    // Step 1: Add genesis vertex A
    println!("Step 1: Add genesis vertex A");
    let a = Vertex::new(EventType::SessionStarted, Vec::new());
    let a_id = a.id;
    store.put_vertex(session_id, a).await?;
    println!("  ✓ Added A");
    show_frontier("step1", 1).await?;
    println!("  📊 Frontier: [A]");
    println!();
    
    // Step 2: Add child B
    println!("Step 2: Add child B of A");
    let b = Vertex::new(EventType::DataCreated, vec![a_id]);
    let b_id = b.id;
    store.put_vertex(session_id, b).await?;
    println!("  ✓ Added B (child of A)");
    show_frontier("step2", 1).await?;
    println!("  📊 Frontier: [B] ← A removed, B added");
    println!();
    
    // Step 3: Add sibling C (also child of A)
    println!("Step 3: Add sibling C (also child of A)");
    let c = Vertex::new(EventType::DataModified, vec![a_id]);
    let c_id = c.id;
    store.put_vertex(session_id, c).await?;
    println!("  ✓ Added C (child of A)");
    show_frontier("step3", 2).await?;
    println!("  📊 Frontier: [B, C] ← C added");
    println!();
    
    // Step 4: Add D (child of B only)
    println!("Step 4: Add D (child of B)");
    let d = Vertex::new(EventType::DataDeleted, vec![b_id]);
    let d_id = d.id;
    store.put_vertex(session_id, d).await?;
    println!("  ✓ Added D (child of B)");
    show_frontier("step4", 2).await?;
    println!("  📊 Frontier: [D, C] ← B removed, D added");
    println!();
    
    // Step 5: Add E (child of both C and D - merge!)
    println!("Step 5: Add E (child of both C and D)");
    let e = Vertex::new(EventType::DataCommitted, vec![c_id, d_id]);
    let e_id = e.id;
    store.put_vertex(session_id, e).await?;
    println!("  ✓ Added E (child of C and D)");
    show_frontier("step5", 1).await?;
    println!("  📊 Frontier: [E] ← Both C and D removed, E added");
    println!();
    
    // Final DAG structure
    println!("🌳 Final DAG Structure:");
    println!("       A (genesis)");
    println!("      / \\");
    println!("     B   C");
    println!("     |   |");
    println!("     D   |");
    println!("      \\ /");
    println!("       E (frontier)");
    println!();
    
    println!("📊 Frontier Evolution:");
    println!("  Initial:  []");
    println!("  After A:  [A]");
    println!("  After B:  [B]      ← A removed");
    println!("  After C:  [B, C]   ← C added");
    println!("  After D:  [D, C]   ← B removed, D added");
    println!("  After E:  [E]      ← Both C and D removed");
    println!();
    
    println!("🎉 Success! You've tracked frontier evolution!");
    println!("\n💡 Key Concepts:");
    println!("  • Frontier = vertices with no children (DAG tips)");
    println!("  • Frontier updates automatically on vertex addition");
    println!("  • Adding a child removes parent from frontier");
    println!("  • Frontier can grow (branching) or shrink (merging)");
    println!("  • Frontier tracking is O(1) per vertex addition");
    
    println!("\n🌟 Why Frontier Matters:");
    println!("  • Tells you where to attach new vertices");
    println!("  • Identifies current state (latest events)");
    println!("  • Enables efficient graph traversal");
    println!("  • Used in session resolution and dehydration");
    
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
echo "  1. Frontier = DAG tips (vertices with no children)"
echo "  2. Frontier updates automatically as DAG grows"
echo "  3. Frontier can grow (branch) or shrink (merge)"
echo "  4. Frontier tells you where to attach new vertices"
echo ""
info "Next demo:"
echo "  ./demo-genesis.sh  - Learn about genesis vertices"
echo ""

