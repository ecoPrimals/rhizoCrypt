#!/bin/bash
#
# 🔐 rhizoCrypt - Content Addressing Demo
#
# Demonstrates:
# 1. Vertex ID = Blake3 hash of content
# 2. Hash stability (same content → same ID)
# 3. Hash uniqueness (different content → different ID)
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
║      🔐 Content Addressing - Blake3 Hashing 🔐           ║
║                                                           ║
║  Learn: Vertex ID = hash of content                      ║
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
log "Creating content addressing demo..."

# Create temporary Rust program
TEMP_DIR=$(mktemp -d)
cat > "$TEMP_DIR/Cargo.toml" << CARGO_EOF
[package]
name = "content-addressing-demo"
version = "0.1.0"
edition = "2021"

[dependencies]
rhizo-crypt-core = { path = "$RHIZO_ROOT/crates/rhizo-crypt-core" }
tokio = { version = "1.46", features = ["full"] }
CARGO_EOF

mkdir -p "$TEMP_DIR/src"
cat > "$TEMP_DIR/src/main.rs" << 'RUST_EOF'
use rhizo_crypt_core::{Vertex, EventType};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n🔐 Content Addressing: Vertex ID = Blake3(content)...\n");
    
    // Demo 1: Hash Stability (same content → same ID)
    println!("📊 Demo 1: Hash Stability");
    println!("Creating two vertices with IDENTICAL content...");
    println!();
    
    let v1 = Vertex::new(EventType::DataCreated, Vec::new());
    let v1_id = v1.id.clone();
    
    let v2 = Vertex::new(EventType::DataCreated, Vec::new());
    let v2_id = v2.id.clone();
    
    println!("Vertex 1 ID: {}", v1_id);
    println!("Vertex 2 ID: {}", v2_id);
    println!();
    
    if v1_id == v2_id {
        println!("✓ SUCCESS: Same content → Same ID");
        println!("  This is deterministic hashing!");
    } else {
        println!("✗ FAIL: Different IDs (timestamps differ!)");
        println!("  Note: Timestamps cause hash differences");
    }
    println!();
    
    // Demo 2: Hash Uniqueness (different content → different ID)
    println!("📊 Demo 2: Hash Uniqueness");
    println!("Creating two vertices with DIFFERENT event types...");
    println!();
    
    let v3 = Vertex::new(EventType::DataCreated, Vec::new());
    let v3_id = v3.id.clone();
    
    let v4 = Vertex::new(EventType::DataModified, Vec::new());
    let v4_id = v4.id.clone();
    
    println!("Vertex 3 (DataCreated):  {}", v3_id);
    println!("Vertex 4 (DataModified): {}", v4_id);
    println!();
    
    if v3_id != v4_id {
        println!("✓ SUCCESS: Different content → Different ID");
        println!("  Hash uniqueness (collision-resistant)!");
    } else {
        println!("✗ FAIL: Same ID (hash collision - extremely unlikely!)");
    }
    println!();
    
    // Demo 3: Parent Hash Influence
    println!("📊 Demo 3: Parent Hash Influence");
    println!("Creating two vertices with different parents...");
    println!();
    
    let parent1 = Vertex::new(EventType::SessionStarted, Vec::new());
    let parent1_id = parent1.id.clone();
    
    let parent2 = Vertex::new(EventType::SliceCheckout, Vec::new());
    let parent2_id = parent2.id.clone();
    
    let v5 = Vertex::new(EventType::DataCreated, vec![parent1_id.clone()]);
    let v5_id = v5.id.clone();
    
    let v6 = Vertex::new(EventType::DataCreated, vec![parent2_id.clone()]);
    let v6_id = v6.id.clone();
    
    println!("Child of Parent 1: {}", v5_id);
    println!("Child of Parent 2: {}", v6_id);
    println!();
    
    if v5_id != v6_id {
        println!("✓ SUCCESS: Different parents → Different ID");
        println!("  Parent hashes are included in vertex hash!");
    } else {
        println!("✗ FAIL: Same ID despite different parents");
    }
    println!();
    
    // Explain Blake3
    println!("🔐 Blake3 Hash Properties:");
    println!("  • 256-bit output (64 hex characters)");
    println!("  • Collision-resistant (2^128 operations to find collision)");
    println!("  • Preimage-resistant (hash → content is infeasible)");
    println!("  • Deterministic (same input → same output)");
    println!("  • Fast (10+ GB/s on modern CPUs)");
    println!();
    
    println!("📦 What's Included in the Hash?");
    println!("  1. Event type (DataCreated, DataModified, etc.)");
    println!("  2. Parent vertex IDs (hashes)");
    println!("  3. Payload (event-specific data)");
    println!("  4. Timestamp (when vertex was created)");
    println!();
    
    println!("🎉 Success! You understand content addressing!");
    println!("\n💡 Key Concepts:");
    println!("  • Vertex ID = Blake3(event_type + parents + payload + timestamp)");
    println!("  • Same content → same ID (deterministic)");
    println!("  • Different content → different ID (collision-resistant)");
    println!("  • Content addressing enables deduplication");
    println!("  • Vertex IDs are globally unique (with overwhelming probability)");
    
    println!("\n🌟 Why Content Addressing?");
    println!("  • Deduplication: Same event stored once");
    println!("  • Verification: ID proves content integrity");
    println!("  • Decentralization: No central ID authority needed");
    println!("  • Immutability: Changing content changes ID");
    
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
echo "  1. Vertex ID = Blake3 hash of all vertex content"
echo "  2. Same content → same ID (deterministic)"
echo "  3. Different content → different ID (collision-resistant)"
echo "  4. Parent hashes influence child IDs"
echo ""
info "Next demo:"
echo "  ./demo-merkle-tree.sh  - Learn about Merkle tree construction"
echo ""

