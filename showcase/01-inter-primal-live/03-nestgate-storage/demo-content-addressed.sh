#!/usr/bin/env bash
# Demo: Content-Addressed Storage
set -euo pipefail

GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
CYAN='\033[0;36m'
NC='\033[0m'

echo -e "${BLUE}═══════════════════════════════════════════════════════${NC}"
echo -e "${BLUE}   🔗 Content-Addressed Storage${NC}"
echo -e "${BLUE}═══════════════════════════════════════════════════════${NC}"
echo ""

cd "$(dirname "$0")/../.."

cat > /tmp/content_addressed.rs << 'EOF'
use rhizo_crypt_core::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("═══════════════════════════════════════════════════════");
    println!("  Content-Addressed Storage: Deduplication");
    println!("═══════════════════════════════════════════════════════\n");
    
    let config = RhizoCryptConfig::default();
    let mut primal = RhizoCrypt::new(config);
    primal.start().await?;
    
    let session = SessionBuilder::new(SessionType::General)
        .with_name("content-addressing-demo")
        .build();
    let session_id = primal.create_session(session).await?;
    
    println!("📝 Scenario: Multiple Users Store Same Document");
    println!("");
    
    // Same content stored by different users
    let document_content = b"The quick brown fox jumps over the lazy dog";
    
    // User Alice stores document
    println!("👤 User Alice stores document");
    let alice_payload = PayloadRef::from_bytes(document_content);
    let alice_hash = blake3::hash(document_content);
    println!("   Hash: {}", alice_hash);
    
    let v1 = VertexBuilder::new(EventType::DataCreate { schema: None })
        .with_agent(Did::new("did:key:alice"))
        .with_payload(alice_payload)
        .with_metadata("filename", "my-document.txt")
        .with_metadata("user", "alice")
        .build();
    primal.append_vertex(session_id, v1).await?;
    println!("   ✓ Vertex created");
    println!("");
    
    // User Bob stores the SAME document
    println!("👤 User Bob stores identical document");
    let bob_payload = PayloadRef::from_bytes(document_content);
    let bob_hash = blake3::hash(document_content);
    println!("   Hash: {}", bob_hash);
    
    let v2 = VertexBuilder::new(EventType::DataCreate { schema: None })
        .with_agent(Did::new("did:key:bob"))
        .with_payload(bob_payload)
        .with_metadata("filename", "bobs-copy.txt")
        .with_metadata("user", "bob")
        .build();
    primal.append_vertex(session_id, v2).await?;
    println!("   ✓ Vertex created");
    println!("");
    
    // User Carol stores the SAME document again
    println!("👤 User Carol stores identical document");
    let carol_payload = PayloadRef::from_bytes(document_content);
    let carol_hash = blake3::hash(document_content);
    println!("   Hash: {}", carol_hash);
    
    let v3 = VertexBuilder::new(EventType::DataCreate { schema: None })
        .with_agent(Did::new("did:key:carol"))
        .with_payload(carol_payload)
        .with_metadata("filename", "carols-version.txt")
        .with_metadata("user", "carol")
        .build();
    primal.append_vertex(session_id, v3).await?;
    println!("   ✓ Vertex created");
    println!("");
    
    println!("🔍 Content-Addressing Analysis:");
    println!("");
    println!("   All three hashes are IDENTICAL:");
    println!("   Alice:  {}", alice_hash);
    println!("   Bob:    {}", bob_hash);
    println!("   Carol:  {}", carol_hash);
    println!("");
    println!("   ✨ Deduplication in Action:");
    println!("   • Three vertices created");
    println!("   • Three different users");
    println!("   • Three different filenames");
    println!("   • But only ONE copy of content stored!");
    println!("");
    
    println!("📊 Storage Efficiency:");
    println!("");
    println!("   Without content-addressing:");
    println!("   ┌─────────┬─────────┬─────────┐");
    println!("   │ Alice   │  Bob    │ Carol   │");
    println!("   │ Copy 1  │ Copy 2  │ Copy 3  │ = 3× storage");
    println!("   └─────────┴─────────┴─────────┘");
    println!("");
    println!("   With content-addressing:");
    println!("   ┌─────────┬─────────┬─────────┐");
    println!("   │ Alice   │  Bob    │ Carol   │");
    println!("   │  ref ───┼─ ref ───┼─ ref    │ → [Single Copy]");
    println!("   └─────────┴─────────┴─────────┘   = 1× storage");
    println!("");
    
    primal.resolve_session(session_id, ResolutionOutcome::Commit).await?;
    
    println!("═══════════════════════════════════════════════════════");
    println!("  🎯 Content-Addressing Benefits:");
    println!("═══════════════════════════════════════════════════════");
    println!("  • Automatic deduplication (same content = same hash)");
    println!("  • Storage efficiency (no duplicate data)");
    println!("  • Integrity verification (hash proves content)");
    println!("  • Fast lookups (hash = address)");
    println!("  • Immutable references (content can't change)");
    println!("═══════════════════════════════════════════════════════\n");
    
    Ok(())
}
EOF

echo -e "${YELLOW}Compiling content-addressed demo...${NC}"
rustc --edition 2021 /tmp/content_addressed.rs \
    -L target/release/deps \
    --extern rhizo_crypt_core=target/release/librhizo_crypt_core.rlib \
    --extern tokio=target/release/deps/libtokio-*.rlib \
    --extern blake3=target/release/deps/libblake3-*.rlib \
    -o /tmp/content_addressed 2>&1 | grep -v "warning" || true

echo "Running content-addressed demo..."
echo ""
/tmp/content_addressed

echo ""
echo -e "${BLUE}═══════════════════════════════════════════════════════${NC}"
echo -e "${GREEN}✅ Content-addressed storage demo complete!${NC}"
echo ""
echo -e "${YELLOW}📚 What you learned:${NC}"
echo "  • Same content = same hash (deterministic)"
echo "  • Automatic deduplication"
echo "  • Massive storage savings"
echo "  • Hash verifies content integrity"
echo "  • NestGate + rhizoCrypt = efficient storage"
echo ""
echo -e "${YELLOW}▶ Next demo:${NC} ./demo-workflow-integration.sh"
echo ""

rm -f /tmp/content_addressed.rs /tmp/content_addressed
