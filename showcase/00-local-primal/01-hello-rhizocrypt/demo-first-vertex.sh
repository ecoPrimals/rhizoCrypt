#!/bin/bash
#
# 🔐 rhizoCrypt - Your First Vertex Demo
#
# Demonstrates:
# 1. Content-addressed vertices (Blake3)
# 2. Vertex creation with VertexBuilder
# 3. Same content = same ID (deduplication)
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
║        🔐 Your First Content-Addressed Vertex             ║
║                                                           ║
║  Learn: Blake3 hashing, content addressing, vertices      ║
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
name = "first-vertex-demo"
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
    println!("\n🔐 Creating your first content-addressed vertex...\n");
    
    // Create rhizoCrypt instance
    let config = RhizoCryptConfig::default();
    let rhizo = RhizoCrypt::new(config);
    rhizo.start().await?;
    
    // Create a session
    let session = Session::new("vertex-demo".to_string(), SessionType::Ephemeral);
    let session_id = session.id;
    rhizo.create_session(session).await?;
    println!("✓ Session created: {}", session_id);
    
    // Create a vertex (content-addressed event)
    let vertex1 = Vertex::new(EventType::DataCreated, Vec::new());
    let vertex_id1 = vertex1.id;
    
    println!("\n📦 First Vertex:");
    println!("  Event Type: {:?}", vertex1.event_type);
    println!("  Vertex ID: {}", vertex_id1);
    println!("  Timestamp: {}", vertex1.timestamp);
    println!("  Parents: {}", vertex1.parents.len());
    
    // Create another vertex with SAME event type
    let vertex2 = Vertex::new(EventType::DataCreated, Vec::new());
    let vertex_id2 = vertex2.id;
    
    println!("\n📦 Second Vertex (same event type, different time):");
    println!("  Event Type: {:?}", vertex2.event_type);
    println!("  Vertex ID: {}", vertex_id2);
    println!("  Timestamp: {}", vertex2.timestamp);
    
    // Show that IDs are different (different timestamps)
    println!("\n🔍 Content-Addressing in Action:");
    if vertex_id1 == vertex_id2 {
        println!("  ✓ IDs are SAME (identical content)");
    } else {
        println!("  ✓ IDs are DIFFERENT (different timestamps)");
        println!("    Even same event type → different IDs due to timestamp");
    }
    
    // Add vertex to session (via DAG store)
    let store = rhizo.dag_store().await?;
    store.put_vertex(session_id, vertex1).await?;
    println!("\n✓ Vertex added to session");
    
    // Verify it exists
    let exists = store.exists(session_id, vertex_id1).await?;
    println!("✓ Vertex exists in DAG: {}", exists);
    
    // Retrieve it back
    let retrieved = store.get_vertex(session_id, vertex_id1).await?;
    println!("✓ Vertex retrieved: {:?}", retrieved.event_type);
    
    println!("\n🎉 Success! You've created content-addressed vertices!");
    println!("\n💡 Key Concepts:");
    println!("  • Vertices are identified by Blake3 hash of their content");
    println!("  • Vertex ID = Hash(event_type + timestamp + parents + metadata)");
    println!("  • Different timestamps → different IDs (temporal uniqueness)");
    println!("  • Content-addressing enables deduplication and integrity");
    println!("  • Any change to vertex content → completely different ID");
    
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
echo "  1. Vertices are content-addressed (Blake3 hash)"
echo "  2. Vertex ID = hash of all vertex data"
echo "  3. Different timestamps → different IDs"
echo "  4. Content-addressing enables deduplication"
echo ""
info "Next demo:"
echo "  ./demo-query-dag.sh  - Learn about DAG queries"
echo ""

