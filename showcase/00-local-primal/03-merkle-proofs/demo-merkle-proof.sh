#!/bin/bash
#
# 🔐 rhizoCrypt - Merkle Proof Verification Demo
#
# Demonstrates:
# 1. Creating a Merkle proof
# 2. Verifying proof without full DAG
# 3. Compact proof size (log n)
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
║      ✅ Merkle Proof Verification - Compact Proofs ✅      ║
║                                                           ║
║  Learn: Prove inclusion without full DAG                 ║
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
log "Creating Merkle proof demo..."

# Create temporary Rust program
TEMP_DIR=$(mktemp -d)
cat > "$TEMP_DIR/Cargo.toml" << CARGO_EOF
[package]
name = "merkle-proof-demo"
version = "0.1.0"
edition = "2021"

[dependencies]
rhizo-crypt-core = { path = "$RHIZO_ROOT/crates/rhizo-crypt-core" }
tokio = { version = "1.46", features = ["full"] }
blake3 = "1.5"
CARGO_EOF

mkdir -p "$TEMP_DIR/src"
cat > "$TEMP_DIR/src/main.rs" << 'RUST_EOF'
use rhizo_crypt_core::{RhizoCrypt, RhizoCryptConfig, Session, SessionType, Vertex, EventType};
use blake3;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n✅ Merkle Proof: Prove Vertex Inclusion Without Full DAG...\n");
    
    // Create rhizoCrypt instance
    let config = RhizoCryptConfig::default();
    let rhizo = RhizoCrypt::new(config);
    rhizo.start().await?;
    
    // Create a session
    let session = Session::new("merkle-proof-demo".to_string(), SessionType::Ephemeral);
    let session_id = session.id;
    rhizo.create_session(session).await?;
    
    let store = rhizo.dag_store().await?;
    
    // Build a DAG with 8 vertices (perfect binary tree)
    println!("📊 Building DAG with 8 vertices:");
    println!("            Root");
    println!("           /    \\");
    println!("        H14      H58");
    println!("        / \\      / \\");
    println!("      H12 H34  H56 H78");
    println!("      /\\  /\\   /\\  /\\");
    println!("     A B C D  E F G H");
    println!();
    
    // Create 8 vertices
    let mut vertices = Vec::new();
    for i in 0..8 {
        let event_type = if i == 0 { EventType::SessionStarted } else { EventType::DataCreated };
        let v = Vertex::new(event_type, Vec::new());
        let v_id = v.id.clone();
        store.put_vertex(session_id, v).await?;
        vertices.push(v_id);
    }
    
    println!("✓ Created 8 vertices");
    println!();
    
    // Build Merkle tree
    let mut current_level = vertices.clone();
    let mut all_hashes = vec![current_level.clone()];
    
    while current_level.len() > 1 {
        let mut next_level = Vec::new();
        for chunk in current_level.chunks(2) {
            let left = &chunk[0];
            let right = if chunk.len() > 1 { &chunk[1] } else { &chunk[0] };
            let parent_hash = blake3::hash(format!("{}{}", left, right).as_bytes()).to_hex().to_string();
            next_level.push(parent_hash);
        }
        all_hashes.push(next_level.clone());
        current_level = next_level;
    }
    
    let merkle_root = &current_level[0];
    println!("🔐 Merkle Root: {}", &merkle_root[..16]);
    println!();
    
    // Create Merkle proof for vertex C (index 2)
    println!("📋 Creating Merkle Proof for Vertex C:");
    let target_vertex = &vertices[2];
    println!("  Target: {} (vertex C)", &target_vertex[..16]);
    println!();
    
    // Proof consists of sibling hashes along the path to root
    println!("  Proof Path (siblings needed to verify):");
    
    let mut proof = Vec::new();
    let mut index = 2; // Index of C
    
    // Level 0: Sibling is D (index 3)
    let sibling = &vertices[3];
    proof.push(sibling.clone());
    println!("    Level 0: Sibling D = {}", &sibling[..16]);
    index /= 2;
    
    // Level 1: Sibling is H12 (index 0)
    let sibling = &all_hashes[1][0];
    proof.push(sibling.clone());
    println!("    Level 1: Sibling H12 = {}", &sibling[..16]);
    index /= 2;
    
    // Level 2: Sibling is H58 (index 1)
    let sibling = &all_hashes[2][1];
    proof.push(sibling.clone());
    println!("    Level 2: Sibling H58 = {}", &sibling[..16]);
    
    println!();
    println!("  Proof Size: {} hashes (log₂ 8 = 3)", proof.len());
    println!("  Full DAG: 8 vertices");
    println!("  Space Savings: {}%", 100 - (proof.len() * 100 / vertices.len()));
    println!();
    
    // Verify proof
    println!("✅ Verifying Merkle Proof:");
    println!();
    
    // Start with target vertex hash
    let mut current_hash = target_vertex.clone();
    println!("  Step 1: Start with target hash");
    println!("          H(C) = {}", &current_hash[..16]);
    
    // Combine with sibling D
    current_hash = blake3::hash(format!("{}{}", current_hash, proof[0]).as_bytes()).to_hex().to_string();
    println!("  Step 2: Hash with sibling D");
    println!("          H(CD) = {}", &current_hash[..16]);
    
    // Combine with sibling H12
    current_hash = blake3::hash(format!("{}{}", proof[1], current_hash).as_bytes()).to_hex().to_string();
    println!("  Step 3: Hash with sibling H12");
    println!("          H(1234) = {}", &current_hash[..16]);
    
    // Combine with sibling H58
    current_hash = blake3::hash(format!("{}{}", current_hash, proof[2]).as_bytes()).to_hex().to_string();
    println!("  Step 4: Hash with sibling H58");
    println!("          Root = {}", &current_hash[..16]);
    
    println!();
    
    if &current_hash == merkle_root {
        println!("✅ SUCCESS: Proof verified!");
        println!("   Computed root matches expected root");
        println!("   → Vertex C is in the DAG");
    } else {
        println!("❌ FAIL: Proof verification failed");
    }
    
    println!();
    println!("🎉 Success! You've verified a Merkle proof!");
    println!("\n💡 Key Concepts:");
    println!("  • Merkle proof = path from vertex to root");
    println!("  • Proof consists of sibling hashes along path");
    println!("  • Proof size: O(log n) for n vertices");
    println!("  • Verification without full DAG");
    println!("  • Recompute root, compare with known root");
    
    println!("\n📊 Proof Compactness:");
    println!("  • 8 vertices → 3 hashes (62.5% smaller)");
    println!("  • 1,024 vertices → 10 hashes (99% smaller)");
    println!("  • 1,048,576 vertices → 20 hashes (99.998% smaller)");
    
    println!("\n🌟 Use Cases:");
    println!("  • Prove event occurred without full history");
    println!("  • Selective disclosure (show one event, hide others)");
    println!("  • Efficient audit (auditor verifies without full DAG)");
    println!("  • Federation sync (prove vertex before syncing)");
    
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
echo "  1. Merkle proof = path from vertex to root"
echo "  2. Proof size: O(log n) for n vertices"
echo "  3. Verification without full DAG"
echo "  4. Massive space savings (99%+ for large DAGs)"
echo ""
info "Next demo:"
echo "  ./demo-tamper-detection.sh  - Learn about tamper detection"
echo ""

