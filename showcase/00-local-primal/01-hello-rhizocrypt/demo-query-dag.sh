#!/bin/bash
#
# 🔐 rhizoCrypt - Query the DAG Demo
#
# Demonstrates:
# 1. Querying vertices in a session
# 2. Frontier tracking (DAG tips)
# 3. Genesis detection (roots)
# 4. Vertex counting
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
║           🔐 Query the DAG - Interactive Demo             ║
║                                                           ║
║  Learn: DAG queries, frontier, genesis, counting          ║
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
name = "query-dag-demo"
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
    println!("\n🔐 Querying the DAG...\n");
    
    // Create rhizoCrypt instance
    let config = RhizoCryptConfig::default();
    let rhizo = RhizoCrypt::new(config);
    rhizo.start().await?;
    
    // Create a session
    let session = Session::new("dag-query-demo".to_string(), SessionType::Ephemeral);
    let session_id = session.id;
    rhizo.create_session(session).await?;
    println!("✓ Session created: {}", session_id);
    
    let store = rhizo.dag_store().await?;
    
    // Add some vertices
    println!("\n📊 Building a small DAG...");
    
    // Genesis vertex (no parents)
    let v1 = Vertex::new(EventType::DataCreated, Vec::new());
    let v1_id = v1.id;
    store.put_vertex(session_id, v1).await?;
    println!("  ✓ Added genesis vertex: {}", v1_id);
    
    // Child of v1
    let v2 = Vertex::new(EventType::DataModified, vec![v1_id]);
    let v2_id = v2.id;
    store.put_vertex(session_id, v2).await?;
    println!("  ✓ Added child vertex: {}", v2_id);
    
    // Another child of v1
    let v3 = Vertex::new(EventType::DataDeleted, vec![v1_id]);
    let v3_id = v3.id;
    store.put_vertex(session_id, v3).await?;
    println!("  ✓ Added another child: {}", v3_id);
    
    // Query: Count vertices
    println!("\n🔍 Query 1: Count Vertices");
    let count = store.count_vertices(session_id).await?;
    println!("  Total vertices: {}", count);
    
    // Query: Get genesis
    println!("\n🔍 Query 2: Find Genesis (roots with no parents)");
    let genesis = store.get_genesis(session_id).await?;
    println!("  Genesis vertices: {}", genesis.len());
    for v_id in &genesis {
        let v = store.get_vertex(session_id, *v_id).await?;
        println!("    - {} ({:?})", v_id, v.event_type);
    }
    
    // Query: Get frontier
    println!("\n🔍 Query 3: Find Frontier (tips with no children)");
    let frontier = store.get_frontier(session_id).await?;
    println!("  Frontier vertices: {}", frontier.len());
    for v_id in &frontier {
        let v = store.get_vertex(session_id, *v_id).await?;
        println!("    - {} ({:?})", v_id, v.event_type);
    }
    
    // Query: Get children
    println!("\n🔍 Query 4: Find Children of Genesis");
    let children = store.get_children(session_id, v1_id).await?;
    println!("  Children of {}: {}", v1_id, children.len());
    for child_id in &children {
        let v = store.get_vertex(session_id, *child_id).await?;
        println!("    - {} ({:?})", child_id, v.event_type);
    }
    
    // Show DAG structure
    println!("\n🌳 DAG Structure:");
    println!("     [Genesis]");
    println!("         |");
    println!("        v1 (DataCreated)");
    println!("       /  \\");
    println!("      /    \\");
    println!("    v2      v3");
    println!(" (Modified) (Deleted)");
    println!("   [Frontier]");
    
    println!("\n🎉 Success! You've queried the DAG!");
    println!("\n💡 Key Concepts:");
    println!("  • Genesis = vertices with no parents (DAG roots)");
    println!("  • Frontier = vertices with no children (DAG tips)");
    println!("  • Children = vertices that reference a vertex as parent");
    println!("  • DAG queries are O(1) for hash lookups");
    println!("  • The DAG structure preserves event causality");
    
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
echo "  1. How to query the DAG (count, genesis, frontier)"
echo "  2. Genesis vertices are DAG roots (no parents)"
echo "  3. Frontier vertices are DAG tips (no children)"
echo "  4. Children queries find dependent vertices"
echo ""
info "Level 1 Complete! Ready for Level 2?"
echo "  cd ../../../02-dag-engine"
echo "  cat README.md"
echo ""

