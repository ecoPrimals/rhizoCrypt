#!/bin/bash
#
# 🔐 rhizoCrypt BearDog Signing Demo
#
# Demonstrates how vertices would be signed with BearDog DIDs.
# This demo simulates the signing workflow without requiring live BearDog.
#

set -e

# Colors
BLUE='\033[0;34m'
GREEN='\033[0;32m'
PURPLE='\033[0;35m'
YELLOW='\033[1;33m'
NC='\033[0m'

log() { echo -e "${BLUE}[$(date '+%H:%M:%S')]${NC} $1"; }
success() { echo -e "${GREEN}✓${NC} $1"; }
warn() { echo -e "${YELLOW}⚠${NC} $1"; }

echo -e "${PURPLE}"
cat << 'EOF'
╔═══════════════════════════════════════════════════════════════╗
║          🔐 rhizoCrypt BearDog Signing Demo                    ║
║                                                                ║
║  Demonstrates DID verification and vertex signing workflow     ║
╚═══════════════════════════════════════════════════════════════╝
EOF
echo -e "${NC}"

# Get the path to rhizoCrypt BEFORE changing directories
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
RHIZO_PATH="$(cd "$SCRIPT_DIR/../../.." && pwd)"

DEMO_DIR="/tmp/rhizocrypt-signing-demo"
rm -rf "$DEMO_DIR"
mkdir -p "$DEMO_DIR"
cd "$DEMO_DIR"

cat > Cargo.toml << EOF
[package]
name = "signing-demo"
version = "0.1.0"
edition = "2021"

[dependencies]
rhizo-crypt-core = { path = "$RHIZO_PATH/crates/rhizo-crypt-core" }
tokio = { version = "1.0", features = ["full"] }
hex = "0.4"
blake3 = "1.8"
EOF

mkdir -p src

cat > src/main.rs << 'EOF'
//! BearDog Signing Demo
//!
//! Shows how vertices would be signed with BearDog DIDs.
//! This demo simulates the workflow without requiring live BearDog.

use rhizo_crypt_core::{
    EventType, PrimalLifecycle, RhizoCrypt, RhizoCryptConfig,
    SessionBuilder, SessionType, VertexBuilder,
};

/// Simulated signature (in production, BearDog provides this)
struct SimulatedSignature {
    signer_did: String,
    signature_bytes: [u8; 32],
}

impl SimulatedSignature {
    fn new(signer_did: &str, data: &[u8]) -> Self {
        // In production, BearDog would create a real cryptographic signature
        let combined = format!("{}:{}", signer_did, hex::encode(data));
        let hash = blake3::hash(combined.as_bytes());
        
        Self {
            signer_did: signer_did.to_string(),
            signature_bytes: *hash.as_bytes(),
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n🔐 rhizoCrypt BearDog Signing Demo\n");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");

    // Initialize
    let config = RhizoCryptConfig::default();
    let mut primal = RhizoCrypt::new(config);
    primal.start().await?;

    // Agent DIDs (in production, these come from BearDog)
    let alice_did = "did:beardog:alice123";
    let bob_did = "did:beardog:bob456";

    println!("🐻 BearDog integration (simulated)\n");
    println!("👤 Agents:");
    println!("   • Alice: {}", alice_did);
    println!("   • Bob: {}", bob_did);
    println!();

    // Create session
    let session = SessionBuilder::new(SessionType::Collaboration {
        workspace_id: "joint-research".to_string(),
    })
    .with_name("Multi-Agent Collaboration")
    .build();
    let session_id = primal.create_session(session).await?;

    println!("📦 Created collaboration session: {}\n", &session_id.to_string()[..16]);

    // Alice creates vertex
    println!("✍️  Step 1: Alice creates a vertex...\n");
    
    let alice_vertex = VertexBuilder::new(EventType::DataCreate { 
        schema: Some("research-note".to_string()) 
    })
    .with_metadata("author", alice_did)
    .with_metadata("content", "Initial hypothesis")
    .build();
    
    let v1 = primal.append_vertex(session_id, alice_vertex).await?;
    println!("   Vertex ID: {}", v1);
    
    // Simulate BearDog signing
    let alice_sig = SimulatedSignature::new(alice_did, v1.to_string().as_bytes());
    println!("   Alice's signature: {}...", hex::encode(&alice_sig.signature_bytes[..8]));
    println!();

    // Bob creates vertex
    println!("✍️  Step 2: Bob creates a vertex...\n");
    
    let bob_vertex = VertexBuilder::new(EventType::DataCreate { 
        schema: Some("research-note".to_string()) 
    })
    .with_parent(v1)
    .with_metadata("author", bob_did)
    .with_metadata("content", "Supporting data")
    .build();
    
    let v2 = primal.append_vertex(session_id, bob_vertex).await?;
    println!("   Vertex ID: {}", v2);
    
    let bob_sig = SimulatedSignature::new(bob_did, v2.to_string().as_bytes());
    println!("   Bob's signature: {}...", hex::encode(&bob_sig.signature_bytes[..8]));
    println!();

    // Verify signatures (simulated)
    println!("🔍 Step 3: Verify signatures (simulated)...\n");
    
    // In production, BearDog would verify these cryptographically
    println!("   Alice's signature valid: true");
    println!("   Bob's signature valid: true");
    println!();

    // Request attestations for dehydration
    println!("📜 Step 4: Request attestations for dehydration...\n");

    let merkle_root = primal.compute_merkle_root(session_id).await?;
    
    // Simulate attestations
    let alice_attest = SimulatedSignature::new(alice_did, merkle_root.as_bytes());
    println!("   Alice attests: \"I contributed to session {}\"", &session_id.to_string()[..8]);
    println!("   Attestation: {}...", hex::encode(&alice_attest.signature_bytes[..8]));
    
    let bob_attest = SimulatedSignature::new(bob_did, merkle_root.as_bytes());
    println!("   Bob attests: \"I contributed to session {}\"", &session_id.to_string()[..8]);
    println!("   Attestation: {}...", hex::encode(&bob_attest.signature_bytes[..8]));
    println!();

    // Summary
    println!("📊 Dehydration Summary:\n");
    println!("   ┌─────────────────────────────────────────────┐");
    println!("   │ Session: Multi-Agent Collaboration          │");
    println!("   │ Merkle Root: {}       │", merkle_root);
    println!("   │ Attestations:                               │");
    println!("   │   • {} (Alice)     │", alice_did);
    println!("   │   • {} (Bob)       │", bob_did);
    println!("   └─────────────────────────────────────────────┘");
    println!();

    // Cleanup
    primal.discard_session(session_id).await?;
    primal.stop().await?;

    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("✅ Signing Demo completed!");
    println!();
    println!("Key Takeaways:");
    println!("  • Each agent signs their own vertices with their DID");
    println!("  • BearDog provides cryptographic signatures");
    println!("  • Attestations prove contributions to a session");
    println!("  • Dehydration summary includes all attestations");
    println!();
    println!("Note: This demo simulates BearDog. In production,");
    println!("      use the live BearDog client for real signatures.");
    println!();

    Ok(())
}
EOF

log "Building demo..."
cargo build --release 2>&1 | grep -E "(Compiling|Finished|error)" || true

log "Running signing demo..."
echo ""
cargo run --release 2>&1

rm -rf "$DEMO_DIR"
success "Demo complete!"
