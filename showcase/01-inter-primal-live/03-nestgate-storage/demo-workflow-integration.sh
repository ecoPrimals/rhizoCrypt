#!/usr/bin/env bash
# Demo: Complete Workflow - rhizoCrypt + NestGate
set -euo pipefail

GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
CYAN='\033[0;36m'
NC='\033[0m'

echo -e "${BLUE}═══════════════════════════════════════════════════════${NC}"
echo -e "${BLUE}   🔄 Complete Workflow: DAG + Storage${NC}"
echo -e "${BLUE}═══════════════════════════════════════════════════════${NC}"
echo ""

cd "$(dirname "$0")/../.."

cat > /tmp/workflow_integration.rs << 'EOF'
use rhizo_crypt_core::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("═══════════════════════════════════════════════════════");
    println!("  Complete Workflow: Document Management System");
    println!("═══════════════════════════════════════════════════════\n");
    
    let config = RhizoCryptConfig::default();
    let mut primal = RhizoCrypt::new(config);
    primal.start().await?;
    
    // Create session for document workflow
    let owner = Did::new("did:key:project-owner");
    let session = SessionBuilder::new(SessionType::General)
        .with_name("document-workflow")
        .with_owner(owner.clone())
        .build();
    let session_id = primal.create_session(session).await?;
    
    println!("📋 Workflow: Collaborative Document Creation");
    println!("   Session: document-workflow");
    println!("   Owner: {}", owner);
    println!("");
    
    // Step 1: Create initial document
    println!("📝 Step 1: Create Document (Alice)");
    let doc_v1 = b"# Project Proposal\n\n## Overview\nInitial draft by Alice";
    let payload_v1 = PayloadRef::from_bytes(doc_v1);
    let hash_v1 = blake3::hash(doc_v1);
    
    let v1 = VertexBuilder::new(EventType::DataCreate { schema: None })
        .with_agent(Did::new("did:key:alice"))
        .with_payload(payload_v1)
        .with_metadata("action", "create")
        .with_metadata("version", "1")
        .with_metadata("size", &doc_v1.len().to_string())
        .build();
    let v1_id = primal.append_vertex(session_id, v1).await?;
    
    println!("   ✓ Document created");
    println!("     Vertex: {}", v1_id);
    println!("     Payload hash: {}", hash_v1);
    println!("     Size: {} bytes", doc_v1.len());
    println!("     Storage: NestGate");
    println!("");
    
    // Step 2: Edit document
    println!("📝 Step 2: Edit Document (Bob)");
    let doc_v2 = b"# Project Proposal\n\n## Overview\nInitial draft by Alice\n\n## Budget\nAdded by Bob";
    let payload_v2 = PayloadRef::from_bytes(doc_v2);
    let hash_v2 = blake3::hash(doc_v2);
    
    let v2 = VertexBuilder::new(EventType::DataUpdate { schema: None })
        .with_agent(Did::new("did:key:bob"))
        .with_parent(v1_id)
        .with_payload(payload_v2)
        .with_metadata("action", "edit")
        .with_metadata("version", "2")
        .with_metadata("size", &doc_v2.len().to_string())
        .build();
    let v2_id = primal.append_vertex(session_id, v2).await?;
    
    println!("   ✓ Document updated");
    println!("     Vertex: {}", v2_id);
    println!("     Parent: {}", v1_id);
    println!("     Payload hash: {}", hash_v2);
    println!("     Size: {} bytes (grew by {} bytes)", doc_v2.len(), doc_v2.len() - doc_v1.len());
    println!("");
    
    // Step 3: Review
    println!("📝 Step 3: Review Document (Carol)");
    let v3 = VertexBuilder::new(EventType::DataUpdate { schema: None })
        .with_agent(Did::new("did:key:carol"))
        .with_parent(v2_id)
        .with_metadata("action", "review")
        .with_metadata("status", "approved")
        .with_metadata("comment", "Looks good, ready to finalize")
        .build();
    let v3_id = primal.append_vertex(session_id, v3).await?;
    
    println!("   ✓ Review recorded");
    println!("     Vertex: {}", v3_id);
    println!("     Status: Approved");
    println!("");
    
    // Step 4: Finalize
    println!("📝 Step 4: Finalize Document (Dave)");
    let doc_final = b"# Project Proposal\n\n## Overview\nInitial draft by Alice\n\n## Budget\nAdded by Bob\n\n## Approved\nFinalized by Dave";
    let payload_final = PayloadRef::from_bytes(doc_final);
    let hash_final = blake3::hash(doc_final);
    
    let v4 = VertexBuilder::new(EventType::DataUpdate { schema: None })
        .with_agent(Did::new("did:key:dave"))
        .with_parent(v3_id)
        .with_payload(payload_final)
        .with_metadata("action", "finalize")
        .with_metadata("version", "final")
        .with_metadata("size", &doc_final.len().to_string())
        .build();
    let _v4_id = primal.append_vertex(session_id, v4).await?;
    
    println!("   ✓ Document finalized");
    println!("     Final payload hash: {}", hash_final);
    println!("     Final size: {} bytes", doc_final.len());
    println!("");
    
    // Show workflow
    println!("🔄 Complete Workflow:");
    println!("");
    println!("   DAG (rhizoCrypt):           Storage (NestGate):");
    println!("   ┌──────────────┐           ┌──────────────┐");
    println!("   │ Alice:Create │─────ref──>│ Doc v1       │");
    println!("   └──────┬───────┘           └──────────────┘");
    println!("          │");
    println!("   ┌──────▼───────┐           ┌──────────────┐");
    println!("   │ Bob: Edit    │─────ref──>│ Doc v2       │");
    println!("   └──────┬───────┘           └──────────────┘");
    println!("          │");
    println!("   ┌──────▼───────┐           (no payload)");
    println!("   │ Carol:Review │");
    println!("   └──────┬───────┘");
    println!("          │");
    println!("   ┌──────▼───────┐           ┌──────────────┐");
    println!("   │ Dave:Finalize│─────ref──>│ Doc final    │");
    println!("   └──────────────┘           └──────────────┘");
    println!("");
    
    // Resolve session
    let resolution = primal.resolve_session(session_id, ResolutionOutcome::Commit).await?;
    
    println!("🔐 Provenance Proof:");
    println!("   • DAG tracks all changes (who, what, when)");
    println!("   • Storage holds all versions");
    println!("   • Merkle root: {}", hex::encode(&resolution.merkle_root));
    println!("   • Single proof validates entire workflow");
    println!("");
    
    println!("═══════════════════════════════════════════════════════");
    println!("  ✨ Integration Benefits:");
    println!("═══════════════════════════════════════════════════════");
    println!("  rhizoCrypt (DAG):          NestGate (Storage):");
    println!("  • Provenance tracking      • Large file storage");
    println!("  • Workflow history         • Content deduplication");
    println!("  • Agent attribution        • ZFS compression");
    println!("  • Cryptographic proofs     • Snapshot features");
    println!("  • Ephemeral by default     • Persistent storage");
    println!("");
    println!("  Together: Complete document management system!");
    println!("═══════════════════════════════════════════════════════\n");
    
    Ok(())
}
EOF

echo -e "${YELLOW}Compiling workflow integration demo...${NC}"
rustc --edition 2021 /tmp/workflow_integration.rs \
    -L target/release/deps \
    --extern rhizo_crypt_core=target/release/librhizo_crypt_core.rlib \
    --extern tokio=target/release/deps/libtokio-*.rlib \
    --extern blake3=target/release/deps/libblake3-*.rlib \
    --extern hex=target/release/deps/libhex-*.rlib \
    -o /tmp/workflow_integration 2>&1 | grep -v "warning" || true

echo "Running workflow integration demo..."
echo ""
/tmp/workflow_integration

echo ""
echo -e "${BLUE}═══════════════════════════════════════════════════════${NC}"
echo -e "${GREEN}✅ Workflow integration demo complete!${NC}"
echo ""
echo -e "${YELLOW}📚 What you learned:${NC}"
echo "  • rhizoCrypt tracks provenance (DAG)"
echo "  • NestGate stores payloads (content-addressed)"
echo "  • Perfect separation of concerns"
echo "  • Full audit trail + efficient storage"
echo "  • Complete document management workflow"
echo ""
echo -e "${CYAN}🎉 NestGate Storage Integration Complete!${NC}"
echo ""
echo -e "${YELLOW}▶ Next:${NC} ToadStool compute integration"
echo "   cd ../04-toadstool-compute"
echo ""

rm -f /tmp/workflow_integration.rs /tmp/workflow_integration
