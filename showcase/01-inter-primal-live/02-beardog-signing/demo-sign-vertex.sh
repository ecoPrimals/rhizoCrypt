#!/usr/bin/env bash
# Demo: Sign DAG Vertex
set -euo pipefail

GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
CYAN='\033[0;36m'
NC='\033[0m'

echo -e "${BLUE}═══════════════════════════════════════════════════════${NC}"
echo -e "${BLUE}   ✍️  Sign DAG Vertex (Cryptographic Provenance)${NC}"
echo -e "${BLUE}═══════════════════════════════════════════════════════${NC}"
echo ""

cd "$(dirname "$0")/../.."

echo -e "${YELLOW}📝 Creating and Signing a Vertex...${NC}"
echo ""

cat > /tmp/sign_vertex.rs << 'EOF'
use rhizo_crypt_core::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("═══════════════════════════════════════════════════════");
    println!("  Vertex Signing: Cryptographic Provenance");
    println!("═══════════════════════════════════════════════════════\n");
    
    // Initialize rhizoCrypt
    let config = RhizoCryptConfig::default();
    let mut primal = RhizoCrypt::new(config);
    primal.start().await?;
    
    // Create session
    let session = SessionBuilder::new(SessionType::General)
        .with_name("signing-demo")
        .build();
    let session_id = primal.create_session(session).await?;
    
    println!("📝 Step 1: Create Vertex");
    let agent = Did::new("did:key:alice");
    let vertex = VertexBuilder::new(EventType::DataCreate { schema: None })
        .with_agent(agent.clone())
        .with_metadata("action", "create_document")
        .with_metadata("doc_id", "doc-001")
        .build();
    
    println!("   ✓ Vertex created");
    println!("     Agent: {}", agent);
    println!("     Event: DataCreate");
    println!("");
    
    // Append to DAG
    let vertex_id = primal.append_vertex(session_id, vertex).await?;
    println!("📝 Step 2: Append to DAG");
    println!("   ✓ Vertex ID: {}", vertex_id);
    println!("   ✓ Content-addressed (Blake3)");
    println!("");
    
    // Simulate signing (in production, would use SigningClient)
    println!("📝 Step 3: Sign Vertex (Cryptographic Signature)");
    println!("");
    println!("   In production:");
    println!("   1. Discover signing service (capability-based)");
    println!("   2. Send vertex hash to HSM");
    println!("   3. HSM signs with private key");
    println!("   4. Attach signature to vertex");
    println!("");
    
    // Show signature structure
    println!("   Signature Structure:");
    println!("   {");
    println!("     vertex_id: '{}'", vertex_id);
    println!("     agent_did: '{}'", agent);
    println!("     signature: <bytes from HSM>");
    println!("     algorithm: 'Ed25519'");
    println!("     timestamp: <ISO-8601>");
    println!("   }");
    println!("");
    
    // Resolve session to get Merkle root
    let resolution = primal.resolve_session(session_id, ResolutionOutcome::Commit).await?;
    
    println!("📝 Step 4: Merkle Root (Whole DAG Signature)");
    println!("   ✓ Merkle root: {}", hex::encode(&resolution.merkle_root));
    println!("   ✓ Cryptographic proof of entire DAG");
    println!("   ✓ Single signature validates all vertices");
    println!("");
    
    println!("═══════════════════════════════════════════════════════");
    println!("  🎯 Cryptographic Provenance Benefits:");
    println!("═══════════════════════════════════════════════════════");
    println!("  • Authenticity: Proves who created the vertex");
    println!("  • Integrity: Detects any tampering");
    println!("  • Non-repudiation: Agent can't deny authorship");
    println!("  • Audit trail: Full provenance chain");
    println!("  • Merkle root: Single signature for entire DAG");
    println!("═══════════════════════════════════════════════════════\n");
    
    Ok(())
}
EOF

echo "Compiling signing demo..."
rustc --edition 2021 /tmp/sign_vertex.rs \
    -L target/release/deps \
    --extern rhizo_crypt_core=target/release/librhizo_crypt_core.rlib \
    --extern tokio=target/release/deps/libtokio-*.rlib \
    --extern hex=target/release/deps/libhex-*.rlib \
    -o /tmp/sign_vertex 2>&1 | grep -v "warning" || true

echo "Running signing demo..."
echo ""
/tmp/sign_vertex

echo ""
echo -e "${BLUE}═══════════════════════════════════════════════════════${NC}"
echo -e "${GREEN}✅ Vertex signing demo complete!${NC}"
echo ""
echo -e "${YELLOW}📚 What you learned:${NC}"
echo "  • Vertices are content-addressed (Blake3)"
echo "  • Signatures provide cryptographic provenance"
echo "  • Agent DIDs track authorship"
echo "  • Merkle root signs entire DAG"
echo "  • HSM integration via capability-based discovery"
echo ""
echo -e "${YELLOW}▶ Next demo:${NC} ./demo-multi-agent.sh"
echo ""

rm -f /tmp/sign_vertex.rs /tmp/sign_vertex
