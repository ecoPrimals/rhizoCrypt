#!/bin/bash
#
# 🔐 rhizoCrypt - Multi-Parent DAG Demo
#
# Demonstrates:
# 1. Vertices with multiple parents
# 2. Diamond pattern (branch and merge)
# 3. DAG structure visualization
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
║        🌳 Multi-Parent DAG - Not Just a Chain! 🌳         ║
║                                                           ║
║  Learn: Multi-parent vertices, diamond pattern           ║
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
log "Creating multi-parent DAG demo..."

# Create temporary Rust program
TEMP_DIR=$(mktemp -d)
cat > "$TEMP_DIR/Cargo.toml" << CARGO_EOF
[package]
name = "multi-parent-demo"
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
    println!("\n🌳 Building a Multi-Parent DAG (Diamond Pattern)...\n");
    
    // Create rhizoCrypt instance
    let config = RhizoCryptConfig::default();
    let rhizo = RhizoCrypt::new(config);
    rhizo.start().await?;
    
    // Create a session
    let session = Session::new("multi-parent-demo".to_string(), SessionType::Ephemeral);
    let session_id = session.id;
    rhizo.create_session(session).await?;
    
    let store = rhizo.dag_store().await?;
    
    // Build the Diamond Pattern:
    //      A
    //     / \
    //    B   C
    //     \ /
    //      D
    
    println!("📊 Building DAG Structure:");
    println!("       A");
    println!("      / \\");
    println!("     B   C");
    println!("      \\ /");
    println!("       D");
    println!();
    
    // Vertex A (genesis - no parents)
    println!("Step 1: Create genesis vertex A");
    let a = Vertex::new(EventType::SessionStarted, Vec::new());
    let a_id = a.id;
    store.put_vertex(session_id, a).await?;
    println!("  ✓ A (genesis): {}", a_id);
    println!("    Parents: 0");
    
    // Vertex B (child of A)
    println!("\nStep 2: Create vertex B (child of A)");
    let b = Vertex::new(EventType::DataCreated, vec![a_id]);
    let b_id = b.id;
    store.put_vertex(session_id, b).await?;
    println!("  ✓ B: {}", b_id);
    println!("    Parents: 1 (A)");
    
    // Vertex C (also child of A)
    println!("\nStep 3: Create vertex C (also child of A)");
    let c = Vertex::new(EventType::DataModified, vec![a_id]);
    let c_id = c.id;
    store.put_vertex(session_id, c).await?;
    println!("  ✓ C: {}", c_id);
    println!("    Parents: 1 (A)");
    
    // Vertex D (child of BOTH B and C - multi-parent!)
    println!("\nStep 4: Create vertex D (child of BOTH B and C)");
    let d = Vertex::new(EventType::DataCommitted, vec![b_id, c_id]);
    let d_id = d.id;
    store.put_vertex(session_id, d).await?;
    println!("  ✓ D: {}", d_id);
    println!("    Parents: 2 (B, C) ← MULTI-PARENT!");
    
    // Query the DAG
    println!("\n🔍 DAG Analysis:");
    
    let count = store.count_vertices(session_id).await?;
    println!("  Total vertices: {}", count);
    
    let genesis = store.get_genesis(session_id).await?;
    println!("  Genesis vertices: {} (A)", genesis.len());
    
    let frontier = store.get_frontier(session_id).await?;
    println!("  Frontier vertices: {} (D)", frontier.len());
    
    // Verify D has two parents
    let d_vertex = store.get_vertex(session_id, d_id).await?;
    println!("\n✨ Multi-Parent Verification:");
    println!("  Vertex D has {} parents", d_vertex.parents.len());
    println!("    Parent 1: {}", d_vertex.parents[0]);
    println!("    Parent 2: {}", d_vertex.parents[1]);
    
    println!("\n🎉 Success! You've created a multi-parent DAG!");
    println!("\n💡 Key Concepts:");
    println!("  • DAG ≠ Blockchain (vertices can have multiple parents)");
    println!("  • Diamond pattern models branch + merge");
    println!("  • Multi-parent enables parallel workflows");
    println!("  • Common in: merges, multi-agent, parallel compute");
    println!("  • Vertex D tracks causality from BOTH B and C");
    
    println!("\n🌟 Real-World Examples:");
    println!("  • Code merge (two branches → one merged commit)");
    println!("  • Multi-agent workflow (two agents → combined result)");
    println!("  • Parallel computation (two tasks → joined output)");
    
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
echo "  1. Vertices can have multiple parents (not just a chain)"
echo "  2. Diamond pattern: A → (B,C) → D"
echo "  3. Multi-parent enables complex workflows"
echo "  4. DAG structure preserves causality"
echo ""
info "Next demo:"
echo "  ./demo-frontier.sh  - Learn about frontier tracking"
echo ""

