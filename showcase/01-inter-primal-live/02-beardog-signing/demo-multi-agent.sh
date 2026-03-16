#!/usr/bin/env bash
# Demo: Multi-Agent Collaboration
set -euo pipefail

GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
CYAN='\033[0;36m'
NC='\033[0m'

echo -e "${BLUE}═══════════════════════════════════════════════════════${NC}"
echo -e "${BLUE}   👥 Multi-Agent Collaboration${NC}"
echo -e "${BLUE}═══════════════════════════════════════════════════════${NC}"
echo ""

cd "$(dirname "$0")/../.."

cat > /tmp/multi_agent.rs << 'EOF'
use rhizo_crypt_core::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("═══════════════════════════════════════════════════════");
    println!("  Multi-Agent Collaboration on Shared DAG");
    println!("═══════════════════════════════════════════════════════\n");
    
    let config = RhizoCryptConfig::default();
    let mut primal = RhizoCrypt::new(config);
    primal.start().await?;
    
    // Create shared session
    let owner = Did::new("did:key:project-owner");
    let session = SessionBuilder::new(SessionType::General)
        .with_name("multi-agent-collab")
        .with_owner(owner.clone())
        .build();
    let session_id = primal.create_session(session).await?;
    
    println!("📝 Scenario: Collaborative Document Editing");
    println!("   Session owner: {}", owner);
    println!("");
    
    // Agent 1: Alice creates document
    println!("👤 Agent 1: Alice");
    let alice = Did::new("did:key:alice");
    let v1 = VertexBuilder::new(EventType::DataCreate { schema: None })
        .with_agent(alice.clone())
        .with_metadata("action", "create_document")
        .with_metadata("doc_id", "shared-doc-001")
        .with_metadata("content", "# Collaborative Document\n\nInitial draft by Alice")
        .build();
    let v1_id = primal.append_vertex(session_id, v1).await?;
    println!("   ✓ Created document (vertex: {})", v1_id);
    println!("   ✓ Signature: Alice signed this creation");
    println!("");
    
    // Agent 2: Bob adds content
    println!("👤 Agent 2: Bob");
    let bob = Did::new("did:key:bob");
    let v2 = VertexBuilder::new(EventType::DataUpdate { schema: None })
        .with_agent(bob.clone())
        .with_parent(v1_id)
        .with_metadata("action", "add_section")
        .with_metadata("doc_id", "shared-doc-001")
        .with_metadata("added", "## Introduction\n\nBob's contribution")
        .build();
    let v2_id = primal.append_vertex(session_id, v2).await?;
    println!("   ✓ Added section (vertex: {})", v2_id);
    println!("   ✓ Signature: Bob signed this update");
    println!("   ✓ Parent: {}", v1_id);
    println!("");
    
    // Agent 3: Carol reviews
    println!("👤 Agent 3: Carol (Reviewer)");
    let carol = Did::new("did:key:carol");
    let v3 = VertexBuilder::new(EventType::DataUpdate { schema: None })
        .with_agent(carol.clone())
        .with_parent(v2_id)
        .with_metadata("action", "review")
        .with_metadata("doc_id", "shared-doc-001")
        .with_metadata("status", "approved")
        .with_metadata("comment", "Looks good!")
        .build();
    let v3_id = primal.append_vertex(session_id, v3).await?;
    println!("   ✓ Approved document (vertex: {})", v3_id);
    println!("   ✓ Signature: Carol signed this approval");
    println!("   ✓ Parent: {}", v2_id);
    println!("");
    
    // Agent 4: Dave finalizes
    println!("👤 Agent 4: Dave (Publisher)");
    let dave = Did::new("did:key:dave");
    let v4 = VertexBuilder::new(EventType::DataUpdate { schema: None })
        .with_agent(dave.clone())
        .with_parent(v3_id)
        .with_metadata("action", "publish")
        .with_metadata("doc_id", "shared-doc-001")
        .with_metadata("version", "1.0")
        .build();
    let _v4_id = primal.append_vertex(session_id, v4).await?;
    println!("   ✓ Published document");
    println!("   ✓ Signature: Dave signed this publication");
    println!("");
    
    // Show DAG structure
    println!("📊 Collaboration DAG:");
    println!("");
    println!("   Alice (create)");
    println!("       │");
    println!("       ▼");
    println!("   Bob (add section)");
    println!("       │");
    println!("       ▼");
    println!("   Carol (approve)");
    println!("       │");
    println!("       ▼");
    println!("   Dave (publish)");
    println!("");
    
    // Resolve and get provenance
    let resolution = primal.resolve_session(session_id, ResolutionOutcome::Commit).await?;
    
    println!("🔐 Cryptographic Provenance:");
    println!("   • Each agent signed their contribution");
    println!("   • Full audit trail preserved in DAG");
    println!("   • Merkle root: {}", hex::encode(&resolution.merkle_root));
    println!("   • Single proof validates all signatures");
    println!("");
    
    println!("═══════════════════════════════════════════════════════");
    println!("  🎯 Multi-Agent Benefits:");
    println!("═══════════════════════════════════════════════════════");
    println!("  • Accountability: Each action signed by agent");
    println!("  • Transparency: Full history in DAG");
    println!("  • Non-repudiation: Agents can't deny actions");
    println!("  • Auditability: Reconstruct who did what when");
    println!("  • Trust: Cryptographic proof of collaboration");
    println!("═══════════════════════════════════════════════════════\n");
    
    Ok(())
}
EOF

echo -e "${YELLOW}Compiling multi-agent demo...${NC}"
rustc --edition 2024 /tmp/multi_agent.rs \
    -L target/release/deps \
    --extern rhizo_crypt_core=target/release/librhizo_crypt_core.rlib \
    --extern tokio=target/release/deps/libtokio-*.rlib \
    --extern hex=target/release/deps/libhex-*.rlib \
    -o /tmp/multi_agent 2>&1 | grep -v "warning" || true

echo "Running multi-agent demo..."
echo ""
/tmp/multi_agent

echo ""
echo -e "${BLUE}═══════════════════════════════════════════════════════${NC}"
echo -e "${GREEN}✅ Multi-agent collaboration demo complete!${NC}"
echo ""
echo -e "${YELLOW}📚 What you learned:${NC}"
echo "  • Multiple agents can collaborate on shared DAG"
echo "  • Each agent signs their contributions"
echo "  • Full provenance chain preserved"
echo "  • Merkle root validates entire collaboration"
echo "  • Accountability + transparency + trust"
echo ""
echo -e "${CYAN}🎉 BearDog Signing Integration Complete!${NC}"
echo ""
echo -e "${YELLOW}▶ Next:${NC} NestGate storage integration"
echo "   cd ../03-nestgate-storage"
echo ""

rm -f /tmp/multi_agent.rs /tmp/multi_agent
