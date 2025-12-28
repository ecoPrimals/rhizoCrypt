#!/bin/bash
#
# 🔐 rhizoCrypt Merkle Proofs Demo
#
# Demonstrates Merkle tree construction and proof verification:
# 1. Build a Merkle tree from vertices
# 2. Generate inclusion proofs
# 3. Verify proofs
#

set -e

# Colors
BLUE='\033[0;34m'
GREEN='\033[0;32m'
PURPLE='\033[0;35m'
NC='\033[0m'

log() { echo -e "${BLUE}[$(date '+%H:%M:%S')]${NC} $1"; }
success() { echo -e "${GREEN}✓${NC} $1"; }

echo -e "${PURPLE}"
cat << 'EOF'
╔═══════════════════════════════════════════════════════════════╗
║            🔐 rhizoCrypt Merkle Proofs Demo                    ║
║                                                                ║
║  Demonstrates: Tree Construction • Proof Generation • Verify   ║
╚═══════════════════════════════════════════════════════════════╝
EOF
echo -e "${NC}"

# Get the path to rhizoCrypt BEFORE changing directories
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
RHIZO_PATH="$(cd "$SCRIPT_DIR/../../.." && pwd)"

DEMO_DIR="/tmp/rhizocrypt-merkle-demo"
rm -rf "$DEMO_DIR"
mkdir -p "$DEMO_DIR"
cd "$DEMO_DIR"

cat > Cargo.toml << EOF
[package]
name = "merkle-demo"
version = "0.1.0"
edition = "2021"

[dependencies]
rhizo-crypt-core = { path = "$RHIZO_PATH/crates/rhizo-crypt-core" }
tokio = { version = "1.0", features = ["full"] }
hex = "0.4"
EOF

mkdir -p src

cat > src/main.rs << 'EOF'
//! Merkle Proofs Demo
//!
//! Shows Merkle tree construction and proof verification.

use rhizo_crypt_core::{
    EventType, PrimalLifecycle, RhizoCrypt, RhizoCryptConfig,
    SessionBuilder, SessionType, VertexBuilder,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n🔐 rhizoCrypt Merkle Proofs Demo\n");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");

    // Initialize
    let config = RhizoCryptConfig::default();
    let mut primal = RhizoCrypt::new(config);
    primal.start().await?;

    // Create session
    let session = SessionBuilder::new(SessionType::General)
        .with_name("Merkle Demo")
        .build();
    let session_id = primal.create_session(session).await?;
    println!("📦 Session created\n");

    // Add vertices to build a tree
    println!("🌳 Building Merkle Tree with 8 vertices...\n");

    let genesis = VertexBuilder::new(EventType::SessionStart).build();
    let v1 = primal.append_vertex(session_id, genesis).await?;
    
    let mut vertices = vec![v1];
    for i in 1..8 {
        let v = VertexBuilder::new(EventType::DataCreate { schema: None })
            .with_parent(vertices[i - 1])
            .with_metadata("index", format!("{}", i))
            .build();
        let vid = primal.append_vertex(session_id, v).await?;
        vertices.push(vid);
        println!("   v{}: {}", i + 1, vid);
    }
    println!();

    // Compute Merkle root
    println!("🔑 Computing Merkle Root...\n");
    let merkle_root = primal.compute_merkle_root(session_id).await?;
    println!("   Root: {}", merkle_root);
    println!();

    // Generate inclusion proof for a specific vertex
    println!("📜 Generating Inclusion Proof...\n");
    let target_vertex_id = vertices[4]; // Pick vertex 5
    println!("   Target vertex: v5 ({})", target_vertex_id);
    
    // Get proof via the primal
    let proof = primal.generate_merkle_proof(session_id, target_vertex_id).await?;
    println!("   Proof sibling count: {} hashes", proof.siblings.len());
    println!("   Proof position: {} of {}", proof.position, proof.total_vertices);
    println!("   Proof siblings:");
    for (i, (direction, hash)) in proof.siblings.iter().enumerate() {
        println!("     [{:>2}] {:?} - {}", i, direction, hex::encode(hash));
    }
    println!();

    // Verify the proof
    println!("✅ Verifying Proof...\n");
    
    // Get the actual vertex to verify
    let target_vertex = primal.get_vertex(session_id, target_vertex_id).await?;
    
    let verified = proof.verify(&target_vertex);
    println!("   Verification result: {}", if verified { "✓ VALID" } else { "✗ INVALID" });
    println!();

    // Show the tree structure
    println!("📊 Merkle Tree Structure (8 leaves):\n");
    println!("                    [Root]");
    println!("                   /      \\");
    println!("               [H01]      [H23]");
    println!("              /    \\      /    \\");
    println!("           [H0] [H1]  [H2] [H3]");
    println!("           /\\   /\\    /\\   /\\");
    println!("          v1 v2 v3 v4 v5 v6 v7 v8");
    println!();
    println!("   H(i) = blake3(H(left) || H(right))");
    println!();

    // Demonstrate proof compactness
    println!("💡 Proof Efficiency:\n");
    println!("   Vertices: 8");
    println!("   Proof size: {} hashes", proof.siblings.len());
    println!("   Proof complexity: O(log n) = O(log 8) = O(3)");
    println!();
    println!("   With 1,000,000 vertices:");
    println!("   Proof size would be: ~20 hashes (O(log 1M) ≈ 20)");
    println!();

    // Cleanup
    primal.discard_session(session_id).await?;
    primal.stop().await?;

    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("✅ Merkle Demo completed!");
    println!();
    println!("Key Takeaways:");
    println!("  • Merkle trees prove integrity of all vertices");
    println!("  • Single root hash represents entire session");
    println!("  • Proofs are O(log n) — efficient even for large DAGs");
    println!("  • Verification doesn't require the full dataset");
    println!();

    Ok(())
}
EOF

log "Building demo..."
cargo build --release 2>&1 | grep -E "(Compiling|Finished|error)" || true

log "Running Merkle proofs demo..."
echo ""
cargo run --release 2>&1

rm -rf "$DEMO_DIR"
success "Demo complete!"
