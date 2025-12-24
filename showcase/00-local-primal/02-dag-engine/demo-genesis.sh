#!/bin/bash
#
# 🔐 rhizoCrypt - Genesis Detection Demo
#
# Demonstrates:
# 1. Genesis = vertices with no parents (DAG roots)
# 2. Multiple genesis vertices
# 3. Genesis stability
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
║          🌱 Genesis Detection - DAG Roots 🌱              ║
║                                                           ║
║  Learn: Genesis = vertices with no parents               ║
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
log "Creating genesis detection demo..."

# Create temporary Rust program
TEMP_DIR=$(mktemp -d)
cat > "$TEMP_DIR/Cargo.toml" << CARGO_EOF
[package]
name = "genesis-demo"
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
    println!("\n🌱 Detecting Genesis Vertices (DAG Roots)...\n");
    
    // Create rhizoCrypt instance
    let config = RhizoCryptConfig::default();
    let rhizo = RhizoCrypt::new(config);
    rhizo.start().await?;
    
    // Create a session
    let session = Session::new("genesis-demo".to_string(), SessionType::Ephemeral);
    let session_id = session.id;
    rhizo.create_session(session).await?;
    
    let store = rhizo.dag_store().await?;
    
    // Helper to show genesis
    let show_genesis = |step: &str| async move {
        let genesis = store.get_genesis(session_id).await?;
        println!("  Genesis: {} vertices", genesis.len());
        for v_id in &genesis {
            let v = store.get_vertex(session_id, *v_id).await?;
            println!("    - {} ({:?})", v_id, v.event_type);
        }
        Ok::<(), Box<dyn std::error::Error>>(())
    };
    
    // Scenario 1: Single Genesis
    println!("📊 Scenario 1: Single Genesis DAG");
    println!("   A → B → C");
    println!();
    
    let a = Vertex::new(EventType::SessionStarted, Vec::new());
    let a_id = a.id;
    store.put_vertex(session_id, a).await?;
    println!("Step 1: Add A (no parents)");
    show_genesis("step1").await?;
    println!();
    
    let b = Vertex::new(EventType::DataCreated, vec![a_id]);
    let b_id = b.id;
    store.put_vertex(session_id, b).await?;
    println!("Step 2: Add B (child of A)");
    show_genesis("step2").await?;
    println!("  ✓ Genesis unchanged (A is still root)");
    println!();
    
    let _c = Vertex::new(EventType::DataModified, vec![b_id]);
    store.put_vertex(session_id, _c).await?;
    println!("Step 3: Add C (child of B)");
    show_genesis("step3").await?;
    println!("  ✓ Genesis unchanged (A is still root)");
    println!();
    
    // Scenario 2: Multiple Genesis (Disconnected Components)
    println!("📊 Scenario 2: Multiple Genesis (Disconnected)");
    println!("   A → B → C    X → Y");
    println!();
    
    let x = Vertex::new(EventType::SliceCheckout, Vec::new());
    let x_id = x.id;
    store.put_vertex(session_id, x).await?;
    println!("Step 4: Add X (no parents - independent root!)");
    show_genesis("step4").await?;
    println!("  ✓ Genesis now has TWO roots: A and X");
    println!();
    
    let _y = Vertex::new(EventType::DataDeleted, vec![x_id]);
    store.put_vertex(session_id, _y).await?;
    println!("Step 5: Add Y (child of X)");
    show_genesis("step5").await?;
    println!("  ✓ Genesis unchanged (A and X are still roots)");
    println!();
    
    // Final DAG structure
    println!("🌳 Final DAG Structure:");
    println!("   A → B → C    X → Y");
    println!("   └─genesis─┘  └genesis┘");
    println!();
    
    println!("🎉 Success! You've detected genesis vertices!");
    println!("\n💡 Key Concepts:");
    println!("  • Genesis = vertices with NO parents (DAG roots)");
    println!("  • Can have multiple genesis vertices (disconnected components)");
    println!("  • Genesis is stable (doesn't change once set)");
    println!("  • Genesis represents entry points to the DAG");
    
    println!("\n🌟 When Do Multiple Genesis Occur?");
    println!("  • Parallel workflows started independently");
    println!("  • Importing external data (each import is a genesis)");
    println!("  • Session merges (each merged session contributes genesis)");
    println!("  • Slices (each slice checkout can be a genesis)");
    
    println!("\n🔍 Genesis vs Frontier:");
    println!("  • Genesis = roots (no parents, start of DAG)");
    println!("  • Frontier = tips (no children, end of DAG)");
    println!("  • Genesis is stable (doesn't change)");
    println!("  • Frontier is dynamic (updates on vertex addition)");
    
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
echo "  1. Genesis = DAG roots (vertices with no parents)"
echo "  2. Can have multiple genesis vertices"
echo "  3. Genesis is stable (doesn't change)"
echo "  4. Genesis represents DAG entry points"
echo ""
info "Next demo:"
echo "  ./demo-topological-sort.sh  - Learn about topological ordering"
echo ""

