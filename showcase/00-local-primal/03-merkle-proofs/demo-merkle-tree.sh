#!/bin/bash
#
# 🔐 rhizoCrypt - Merkle Tree Construction Demo
#
# Demonstrates:
# 1. Building Merkle tree from DAG
# 2. Bottom-up hash computation
# 3. Merkle root calculation
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
║      🌳 Merkle Tree Construction - DAG Integrity 🌳       ║
║                                                           ║
║  Learn: Merkle root = cryptographic summary              ║
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
log "Creating Merkle tree demo..."

# Create temporary Rust program
TEMP_DIR=$(mktemp -d)
cat > "$TEMP_DIR/Cargo.toml" << CARGO_EOF
[package]
name = "merkle-tree-demo"
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
    println!("\n🌳 Building a Merkle Tree from DAG...\n");
    
    // Create rhizoCrypt instance
    let config = RhizoCryptConfig::default();
    let rhizo = RhizoCrypt::new(config);
    rhizo.start().await?;
    
    // Create a session
    let session = Session::new("merkle-tree-demo".to_string(), SessionType::Ephemeral);
    let session_id = session.id;
    rhizo.create_session(session).await?;
    
    let store = rhizo.dag_store().await?;
    
    // Build a simple DAG:
    //      A
    //     / \
    //    B   C
    //     \ /
    //      D
    
    println!("📊 Building DAG:");
    println!("       A");
    println!("      / \\");
    println!("     B   C");
    println!("      \\ /");
    println!("       D");
    println!();
    
    let a = Vertex::new(EventType::SessionStarted, Vec::new());
    let a_id = a.id.clone();
    store.put_vertex(session_id, a.clone()).await?;
    println!("✓ A: {}", a_id);
    
    let b = Vertex::new(EventType::DataCreated, vec![a_id.clone()]);
    let b_id = b.id.clone();
    store.put_vertex(session_id, b.clone()).await?;
    println!("✓ B: {}", b_id);
    
    let c = Vertex::new(EventType::DataModified, vec![a_id.clone()]);
    let c_id = c.id.clone();
    store.put_vertex(session_id, c.clone()).await?;
    println!("✓ C: {}", c_id);
    
    let d = Vertex::new(EventType::DataCommitted, vec![b_id.clone(), c_id.clone()]);
    let d_id = d.id.clone();
    store.put_vertex(session_id, d.clone()).await?;
    println!("✓ D: {}", d_id);
    println!();
    
    // Build Merkle tree (bottom-up)
    println!("🔐 Building Merkle Tree (bottom-up):");
    println!();
    
    // Level 0: Leaf hashes (vertex IDs are already Blake3 hashes)
    println!("Level 0 (Leaves - Vertex IDs):");
    println!("  H(A) = {}", &a_id[..16]);
    println!("  H(B) = {}", &b_id[..16]);
    println!("  H(C) = {}", &c_id[..16]);
    println!("  H(D) = {}", &d_id[..16]);
    println!();
    
    // Level 1: Parent hashes
    println!("Level 1 (Parent Hashes):");
    
    // H(AB) = hash(H(A) + H(B))
    let h_ab = blake3::hash(format!("{}{}", a_id, b_id).as_bytes()).to_hex();
    println!("  H(AB) = hash(H(A) + H(B))");
    println!("        = {}", &h_ab[..16]);
    
    // H(CD) = hash(H(C) + H(D))
    let h_cd = blake3::hash(format!("{}{}", c_id, d_id).as_bytes()).to_hex();
    println!("  H(CD) = hash(H(C) + H(D))");
    println!("        = {}", &h_cd[..16]);
    println!();
    
    // Level 2: Merkle root
    println!("Level 2 (Merkle Root):");
    let merkle_root = blake3::hash(format!("{}{}", h_ab, h_cd).as_bytes()).to_hex();
    println!("  Root = hash(H(AB) + H(CD))");
    println!("       = {}", &merkle_root.to_string()[..16]);
    println!();
    
    // Visualize Merkle tree
    println!("🌳 Merkle Tree Visualization:");
    println!("              Root");
    println!("             /    \\");
    println!("          H(AB)  H(CD)");
    println!("          /  \\    /  \\");
    println!("        H(A) H(B) H(C) H(D)");
    println!("         |    |    |    |");
    println!("         A    B    C    D");
    println!("      (vertices in DAG)");
    println!();
    
    println!("🎉 Success! You've built a Merkle tree!");
    println!("\n💡 Key Concepts:");
    println!("  • Merkle tree = binary hash tree over DAG");
    println!("  • Built bottom-up (leaves → root)");
    println!("  • Each parent hash = hash(left_child + right_child)");
    println!("  • Merkle root = cryptographic summary of entire DAG");
    println!("  • Any change propagates up to root");
    
    println!("\n🔐 Cryptographic Properties:");
    println!("  • Root commits to all vertices");
    println!("  • Changing any vertex changes root");
    println!("  • Collision-resistant (can't forge same root)");
    println!("  • Preimage-resistant (can't find vertices from root)");
    
    println!("\n🌟 Use Cases:");
    println!("  • Session integrity verification");
    println!("  • Compact proofs (show vertex in DAG without full DAG)");
    println!("  • Efficient sync (compare roots, sync diff)");
    println!("  • Tamper detection (root mismatch = tampering)");
    
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
echo "  1. Merkle tree built bottom-up from DAG"
echo "  2. Parent hash = hash(left_child + right_child)"
echo "  3. Merkle root = cryptographic summary"
echo "  4. Any change propagates up to root"
echo ""
info "Next demo:"
echo "  ./demo-merkle-proof.sh  - Learn about Merkle proofs"
echo ""

