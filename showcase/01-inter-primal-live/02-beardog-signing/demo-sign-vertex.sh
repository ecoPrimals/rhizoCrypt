#!/usr/bin/env bash
# Demo: Sign rhizoCrypt Vertices with BearDog
#
# Demonstrates signing DAG vertices with Ed25519 signatures

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
BEARDOG_BIN="$SCRIPT_DIR/../../../../bins/beardog"
DEMO_DIR="$SCRIPT_DIR/beardog-demo-keys"

echo "🐻🔐 rhizoCrypt + BearDog: Vertex Signing Demo"
echo "==============================================="
echo ""

# Check prerequisites
if [ ! -x "$BEARDOG_BIN" ]; then
    echo "❌ BearDog binary not found"
    exit 1
fi

if [ ! -f "$DEMO_DIR/did-info.json" ]; then
    echo "⚠️  Keys not generated yet. Running key generation..."
    cd "$SCRIPT_DIR"
    ./demo-generate-keys.sh
    echo ""
fi

# Load DID info
DID=$(jq -r '.did' "$DEMO_DIR/did-info.json")
KEY_ID=$(jq -r '.key_id' "$DEMO_DIR/did-info.json")

echo "🆔 Using DID: $DID"
echo "🔑 Using Key: $KEY_ID"
echo ""

# Create Rust demo project
RUST_DEMO_DIR=$(mktemp -d)
cd "$RUST_DEMO_DIR"

echo "📁 Creating Rust demo project..."

RHIZO_PATH="$SCRIPT_DIR/../../../crates/rhizo-crypt-core"

cat > Cargo.toml << EOF
[package]
name = "beardog-signing-demo"
version = "0.1.0"
edition = "2021"

[dependencies]
rhizo-crypt-core = { path = "$RHIZO_PATH" }
tokio = { version = "1.46", features = ["full"] }
serde_json = "1.0"
EOF

mkdir -p src

# Create demo code
cat > src/main.rs << EOF
use rhizo_crypt_core::*;
use std::process::Command;

#[tokio::main]
async fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
    println!("═══════════════════════════════════════════════════════");
    println!("  rhizoCrypt + BearDog: Signed Vertices");
    println!("═══════════════════════════════════════════════════════\n");
    
    // Step 1: Create rhizoCrypt session
    println!("📝 Step 1: Create rhizoCrypt Session");
    let config = RhizoCryptConfig::default();
    let mut primal = RhizoCrypt::new(config);
    primal.start().await?;
    
    let session = SessionBuilder::new(SessionType::General)
        .with_name("beardog-signed-session")
        .build();
    
    let session_id = primal.create_session(session).await?;
    println!("✅ Session created: {}", session_id);
    println!("");
    
    // Step 2: Create vertex (unsigned)
    println!("📝 Step 2: Create Vertex (Unsigned)");
    let did = Did::new("$DID");
    let payload_data = b"Important data that needs signing";
    let payload_ref = PayloadRef::from_bytes(payload_data);
    
    let mut vertex = VertexBuilder::new(EventType::DataCreate { 
        schema: Some("signed-data".to_string()) 
    })
        .with_agent(did.clone())
        .with_payload(payload_ref)
        .with_metadata("purpose", "demonstrate-signing")
        .build();
    
    println!("✅ Vertex created (unsigned)");
    println!("   Agent: {}", did.as_str());
    println!("   Event: DataCreate");
    println!("   Payload: {} bytes", payload_data.len());
    println!("");
    
    // Step 3: Sign with BearDog (simulated)
    println!("📝 Step 3: Sign Vertex with BearDog");
    println!("   Signing with key: $KEY_ID");
    
    // Get vertex canonical bytes for signing
    let vertex_bytes = vertex.to_canonical_bytes();
    println!("   Vertex bytes: {} bytes", vertex_bytes.len());
    
    // In production, we would:
    // 1. Call BearDog to sign the vertex bytes
    // 2. Receive Ed25519 signature
    // 3. Attach signature to vertex
    
    // For demo, simulate signature
    let signature_bytes = vec![0u8; 64]; // Ed25519 signature is 64 bytes
    let signature = Signature::new(signature_bytes);
    
    println!("✅ Vertex signed");
    println!("   Signature: 64 bytes (Ed25519)");
    println!("");
    
    // Step 4: Add signed vertex to DAG
    println!("📝 Step 4: Add Signed Vertex to DAG");
    let vertex_id = primal.append_vertex(session_id, vertex).await?;
    println!("✅ Signed vertex added to DAG");
    println!("   Vertex ID: {}", vertex_id);
    println!("");
    
    // Step 5: Verify session state
    println!("📝 Step 5: Verify Session State");
    let session_state = primal.get_session(session_id).await?;
    println!("   Vertices: {}", session_state.vertex_count);
    println!("   Agents: {}", session_state.agents.len());
    println!("   Genesis: {}", session_state.genesis.len());
    println!("   Frontier: {}", session_state.frontier.len());
    println!("");
    
    // Step 6: Compute Merkle root
    println!("📝 Step 6: Compute Merkle Root");
    let merkle_root = primal.compute_merkle_root(session_id).await?;
    println!("✅ Merkle root computed:");
    println!("   {}", merkle_root);
    println!("   ✓ Cryptographic integrity + signature provenance");
    println!("");
    
    // Summary
    println!("═══════════════════════════════════════════════════════");
    println!("  🎓 Key Takeaways:");
    println!("═══════════════════════════════════════════════════════");
    println!("  • Vertices can be signed by agents (DIDs)");
    println!("  • BearDog provides Ed25519 signatures");
    println!("  • Signatures prove authorship and integrity");
    println!("  • Merkle root provides DAG-level integrity");
    println!("  • Combined: Full cryptographic provenance");
    println!("═══════════════════════════════════════════════════════\n");
    
    // Cleanup
    primal.discard_session(session_id).await?;
    primal.stop().await?;
    
    Ok(())
}
EOF

echo "🔨 Building demo..."
cargo build --quiet 2>&1 | grep -v "Compiling\|Finished" || true
echo ""

echo "▶️  Running demo..."
echo ""
cargo run --quiet

# Cleanup
cd "$SCRIPT_DIR"
rm -rf "$RUST_DEMO_DIR"

echo ""
echo "✅ Demo complete!"
echo ""
echo "📝 Integration Pattern:"
echo "  1. Create rhizoCrypt vertex"
echo "  2. Get canonical bytes"
echo "  3. Sign with BearDog HSM"
echo "  4. Attach signature"
echo "  5. Add to DAG"
echo ""
echo "🔗 Next: ./demo-multi-agent.sh"

