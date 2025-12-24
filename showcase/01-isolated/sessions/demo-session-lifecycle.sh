#!/bin/bash
#
# 🔐 rhizoCrypt Session Lifecycle Demo
#
# Demonstrates the complete session lifecycle:
# 1. Create a session
# 2. Append vertices (events)
# 3. Query the DAG
# 4. Resolve/discard the session
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
╔═══════════════════════════════════════════════════════════════╗
║          🔐 rhizoCrypt Session Lifecycle Demo                  ║
║                                                                ║
║  Demonstrates: Create → Grow → Query → Resolve                 ║
╚═══════════════════════════════════════════════════════════════╝
EOF
echo -e "${NC}"

# Check if we can run Rust
log "Checking prerequisites..."

if ! command -v cargo &> /dev/null; then
    echo -e "${RED}Error: cargo not found. Please install Rust.${NC}"
    exit 1
fi

# Get the path to rhizoCrypt BEFORE changing directories
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
RHIZO_PATH="$(cd "$SCRIPT_DIR/../../.." && pwd)"

# Create demo directory
DEMO_DIR="/tmp/rhizocrypt-session-demo"
rm -rf "$DEMO_DIR"
mkdir -p "$DEMO_DIR"
cd "$DEMO_DIR"

log "Demo directory: $DEMO_DIR"
log "rhizoCrypt path: $RHIZO_PATH"

# Create Rust demo program
log "Creating session lifecycle demo..."

cat > Cargo.toml << EOF
[package]
name = "session-demo"
version = "0.1.0"
edition = "2021"

[dependencies]
rhizo-crypt-core = { path = "$RHIZO_PATH/crates/rhizo-crypt-core" }
tokio = { version = "1.0", features = ["full"] }
EOF

mkdir -p src

cat > src/main.rs << 'EOF'
//! Session Lifecycle Demo
//! 
//! Demonstrates rhizoCrypt's core session management capabilities.

use rhizo_crypt_core::{
    EventType, PrimalLifecycle, RhizoCrypt, RhizoCryptConfig,
    SessionBuilder, SessionType, VertexBuilder,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n🔐 rhizoCrypt Session Lifecycle Demo\n");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");

    // Step 1: Create rhizoCrypt instance
    println!("📦 Step 1: Creating rhizoCrypt instance...");
    let config = RhizoCryptConfig::default();
    let mut primal = RhizoCrypt::new(config);
    primal.start().await?;
    println!("   ✓ rhizoCrypt started\n");

    // Step 2: Create a gaming session
    println!("🎮 Step 2: Creating a gaming session...");
    let session = SessionBuilder::new(SessionType::Gaming {
        game_id: "chess-match-001".to_string(),
    })
    .with_name("Grandmaster Championship")
    .with_max_vertices(1000)
    .build();

    let session_id = primal.create_session(session).await?;
    println!("   ✓ Session created: {}", session_id);
    println!("   ✓ Type: Gaming (chess-match-001)");
    println!("   ✓ Max vertices: 1000\n");

    // Step 3: Append game events
    println!("♟️  Step 3: Appending game events...");
    
    // Game start
    let start_vertex = VertexBuilder::new(EventType::SessionStart)
        .with_metadata("white", "Magnus")
        .with_metadata("black", "Hikaru")
        .build();
    let v1 = primal.append_vertex(session_id, start_vertex).await?;
    println!("   ✓ Game started: {}", v1);

    // First move
    let move1 = VertexBuilder::new(EventType::DataCreate { schema: Some("chess-move".to_string()) })
        .with_parent(v1)
        .with_metadata("move", "e4")
        .with_metadata("player", "white")
        .build();
    let v2 = primal.append_vertex(session_id, move1).await?;
    println!("   ✓ Move e4: {}", v2);

    // Response
    let move2 = VertexBuilder::new(EventType::DataCreate { schema: Some("chess-move".to_string()) })
        .with_parent(v2)
        .with_metadata("move", "e5")
        .with_metadata("player", "black")
        .build();
    let v3 = primal.append_vertex(session_id, move2).await?;
    println!("   ✓ Move e5: {}", v3);

    // More moves (showing multi-parent DAG)
    let move3 = VertexBuilder::new(EventType::DataCreate { schema: Some("chess-move".to_string()) })
        .with_parent(v3)
        .with_metadata("move", "Nf3")
        .with_metadata("player", "white")
        .build();
    let v4 = primal.append_vertex(session_id, move3).await?;
    println!("   ✓ Move Nf3: {}", v4);

    println!();

    // Step 4: Query the DAG
    println!("🔍 Step 4: Querying the DAG...");
    
    let vertex_count = primal.total_vertex_count().await;
    println!("   ✓ Total vertices: {}", vertex_count);
    
    let session_info = primal.get_session(session_id).await?;
    println!("   ✓ Session state: {:?}", session_info.state);
    println!("   ✓ Session vertices: {}", session_info.vertex_count);
    println!();

    // Step 5: Get Merkle root
    println!("🌳 Step 5: Computing Merkle root...");
    let merkle_root = primal.compute_merkle_root(session_id).await?;
    println!("   ✓ Merkle root: {}", merkle_root);
    println!();

    // Step 6: Demonstrate vertex retrieval
    println!("📖 Step 6: Retrieving a vertex...");
    let vertex = primal.get_vertex(session_id, v2).await?;
    println!("   ✓ Vertex ID: {}", vertex.compute_id());
    println!("   ✓ Event type: {:?}", vertex.event_type);
    println!("   ✓ Parents: {:?}", vertex.parents);
    if let Some(mv) = vertex.metadata.get("move") {
        println!("   ✓ Move: {:?}", mv);
    }
    println!();

    // Step 7: Session summary
    println!("📊 Step 7: Session summary...");
    println!("   ┌─────────────────────────────────────┐");
    println!("   │ Session: Grandmaster Championship   │");
    println!("   │ Vertices: 4                         │");
    println!("   │ State: Active                       │");
    println!("   │ Merkle Root: {}...  │", &merkle_root.to_string()[..16]);
    println!("   └─────────────────────────────────────┘");
    println!();

    // Step 8: Discard session (ephemeral - designed to be forgotten!)
    println!("🗑️  Step 8: Discarding session (ephemeral by design)...");
    primal.discard_session(session_id).await?;
    println!("   ✓ Session discarded - the memory that knows when to forget!");
    println!();

    // Cleanup
    primal.stop().await?;

    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("✅ Demo completed successfully!");
    println!();
    println!("Key Takeaways:");
    println!("  • Sessions scope DAG operations");
    println!("  • Vertices are content-addressed (Blake3)");
    println!("  • DAG supports multi-parent relationships");
    println!("  • Merkle roots prove integrity");
    println!("  • Ephemeral by default - only commits survive");
    println!();

    Ok(())
}
EOF

log "Building demo..."
cargo build --release 2>&1 | grep -E "(Compiling|Finished|error)" || true

log "Running session lifecycle demo..."
echo ""
cargo run --release 2>&1

# Cleanup
log "Cleaning up..."
rm -rf "$DEMO_DIR"

success "Demo complete!"

