#!/usr/bin/env bash
# Demo: Document Management Workflow
set -euo pipefail

GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
CYAN='\033[0;36m'
NC='\033[0m'

echo -e "${BLUE}═══════════════════════════════════════════════════════${NC}"
echo -e "${BLUE}   📄 Document Management Workflow${NC}"
echo -e "${BLUE}═══════════════════════════════════════════════════════${NC}"
echo ""

cd "$(dirname "$0")/../.."

cat > /tmp/document_workflow.rs << 'EOF'
use rhizo_crypt_core::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("═══════════════════════════════════════════════════════");
    println!("  Document Workflow: Collaborative Editing");
    println!("═══════════════════════════════════════════════════════\n");
    
    let config = RhizoCryptConfig::default();
    let mut primal = RhizoCrypt::new(config);
    primal.start().await?;
    
    println!("📋 Scenario: Multi-Party Contract Negotiation");
    println!("   Document: Service Agreement");
    println!("   Parties: Client, Vendor, Legal (2 lawyers)");
    println!("");
    
    let session = SessionBuilder::new(SessionType::General)
        .with_name("service-agreement-2025")
        .with_owner(Did::new("did:key:contract-owner"))
        .build();
    let session_id = primal.create_session(session).await?;
    
    println!("═══════════════════════════════════════════════════════");
    println!("  Phase 1: Initial Draft (Vendor)");
    println!("═══════════════════════════════════════════════════════\n");
    
    let doc_v1 = b"SERVICE AGREEMENT\n\nParties: Client Corp & Vendor Inc\n\nTerm: 12 months\nPrice: $TBD\n\n(Draft v1 by Vendor)";
    let payload_v1 = PayloadRef::from_bytes(doc_v1);
    
    let v1 = VertexBuilder::new(EventType::DataCreate { schema: None })
        .with_agent(Did::new("did:key:vendor-rep"))
        .with_payload(payload_v1)
        .with_metadata("stage", "draft")
        .with_metadata("version", "1")
        .with_metadata("author", "vendor-rep")
        .with_metadata("storage", "nestgate")
        .build();
    let v1_id = primal.append_vertex(session_id, v1).await?;
    
    println!("📝 Vendor creates initial draft");
    println!("   ✓ Draft v1 created");
    println!("   ✓ Stored in NestGate");
    println!("   Vertex: {}", v1_id);
    println!("");
    
    println!("═══════════════════════════════════════════════════════");
    println!("  Phase 2: Client Review & Edits");
    println!("═══════════════════════════════════════════════════════\n");
    
    let doc_v2 = b"SERVICE AGREEMENT\n\nParties: Client Corp & Vendor Inc\n\nTerm: 12 months\nPrice: $120,000/year\nPayment: Net 30 days\n\n(Edited by Client)";
    let payload_v2 = PayloadRef::from_bytes(doc_v2);
    
    let v2 = VertexBuilder::new(EventType::DataUpdate { schema: None })
        .with_agent(Did::new("did:key:client-rep"))
        .with_parent(v1_id)
        .with_payload(payload_v2)
        .with_metadata("stage", "review")
        .with_metadata("version", "2")
        .with_metadata("author", "client-rep")
        .with_metadata("changes", "Added price and payment terms")
        .build();
    let v2_id = primal.append_vertex(session_id, v2).await?;
    
    println!("💼 Client reviews and edits");
    println!("   ✓ Added price: $120,000/year");
    println!("   ✓ Added payment terms: Net 30");
    println!("   Vertex: {}", v2_id);
    println!("");
    
    println!("═══════════════════════════════════════════════════════");
    println!("  Phase 3: Legal Review (Lawyer 1)");
    println!("═══════════════════════════════════════════════════════\n");
    
    let doc_v3 = b"SERVICE AGREEMENT\n\nParties: Client Corp & Vendor Inc\n\nTerm: 12 months\nPrice: $120,000/year\nPayment: Net 30 days\nLiability Cap: $500,000\nGoverning Law: Delaware\n\n(Legal review by Lawyer 1)";
    let payload_v3 = PayloadRef::from_bytes(doc_v3);
    
    let v3 = VertexBuilder::new(EventType::DataUpdate { schema: None })
        .with_agent(Did::new("did:key:lawyer-1"))
        .with_parent(v2_id)
        .with_payload(payload_v3)
        .with_metadata("stage", "legal_review")
        .with_metadata("version", "3")
        .with_metadata("author", "lawyer-1")
        .with_metadata("changes", "Added liability and jurisdiction")
        .build();
    let v3_id = primal.append_vertex(session_id, v3).await?;
    
    println!("⚖️  Lawyer 1 adds legal terms");
    println!("   ✓ Liability cap: $500,000");
    println!("   ✓ Governing law: Delaware");
    println!("   Vertex: {}", v3_id);
    println!("");
    
    println!("═══════════════════════════════════════════════════════");
    println!("  Phase 4: Legal Review (Lawyer 2)");
    println!("═══════════════════════════════════════════════════════\n");
    
    let v4 = VertexBuilder::new(EventType::DataUpdate { schema: None })
        .with_agent(Did::new("did:key:lawyer-2"))
        .with_parent(v3_id)
        .with_metadata("stage", "legal_review_2")
        .with_metadata("review_status", "approved")
        .with_metadata("reviewer", "lawyer-2")
        .with_metadata("comment", "All legal terms acceptable")
        .build();
    let v4_id = primal.append_vertex(session_id, v4).await?;
    
    println!("⚖️  Lawyer 2 approves legal terms");
    println!("   ✓ Status: Approved");
    println!("   ✓ Comment: All legal terms acceptable");
    println!("   Vertex: {}", v4_id);
    println!("");
    
    println!("═══════════════════════════════════════════════════════");
    println!("  Phase 5: Final Signatures (BearDog HSM)");
    println!("═══════════════════════════════════════════════════════\n");
    
    // Client signature
    println!("✍️  Client signs document (BearDog HSM)");
    let client_sig = b"[Client signature via BearDog]";
    let v5 = VertexBuilder::new(EventType::DataUpdate { schema: None })
        .with_agent(Did::new("did:beardog:client-hsm"))
        .with_parent(v4_id)
        .with_metadata("stage", "signature")
        .with_metadata("signer", "client-rep")
        .with_metadata("signature", &hex::encode(client_sig))
        .with_metadata("algorithm", "Ed25519")
        .with_metadata("timestamp", "2025-12-26T10:30:00Z")
        .build();
    let v5_id = primal.append_vertex(session_id, v5).await?;
    println!("   ✓ Client signature captured");
    println!("   Vertex: {}", v5_id);
    println!("");
    
    // Vendor signature
    println!("✍️  Vendor signs document (BearDog HSM)");
    let vendor_sig = b"[Vendor signature via BearDog]";
    let v6 = VertexBuilder::new(EventType::DataUpdate { schema: None })
        .with_agent(Did::new("did:beardog:vendor-hsm"))
        .with_parent(v5_id)
        .with_metadata("stage", "signature")
        .with_metadata("signer", "vendor-rep")
        .with_metadata("signature", &hex::encode(vendor_sig))
        .with_metadata("algorithm", "Ed25519")
        .with_metadata("timestamp", "2025-12-26T10:35:00Z")
        .build();
    let _v6_id = primal.append_vertex(session_id, v6).await?;
    println!("   ✓ Vendor signature captured");
    println!("");
    
    // Resolve session
    let resolution = primal.resolve_session(session_id, ResolutionOutcome::Commit).await?;
    
    println!("═══════════════════════════════════════════════════════");
    println!("  ✅ Contract Finalized!");
    println!("═══════════════════════════════════════════════════════\n");
    
    println!("📊 Complete Workflow:");
    println!("");
    println!("   Vendor Rep  → Draft v1");
    println!("   Client Rep  → Edit (add price/terms)");
    println!("   Lawyer 1    → Legal review (add clauses)");
    println!("   Lawyer 2    → Approve legal terms");
    println!("   Client HSM  → Sign (BearDog)");
    println!("   Vendor HSM  → Sign (BearDog)");
    println!("");
    
    println!("🔐 Provenance Proof:");
    println!("   • Full edit history preserved");
    println!("   • All versions stored (NestGate)");
    println!("   • Multi-party collaboration");
    println!("   • Cryptographic signatures (BearDog)");
    println!("   • Immutable audit trail");
    println!("   • Merkle root: {}", hex::encode(&resolution.merkle_root));
    println!("");
    
    println!("✨ Benefits:");
    println!("   • Complete audit trail (who changed what, when)");
    println!("   • Version control built-in");
    println!("   • Content-addressed storage (deduplication)");
    println!("   • Cryptographic signatures");
    println!("   • Dispute resolution (full history)");
    println!("   • Compliance (immutable records)");
    println!("");
    
    println!("═══════════════════════════════════════════════════════");
    println!("  Document workflows with full provenance!");
    println!("═══════════════════════════════════════════════════════\n");
    
    Ok(())
}
EOF

echo -e "${YELLOW}Compiling document workflow demo...${NC}"
rustc --edition 2024 /tmp/document_workflow.rs \
    -L target/release/deps \
    --extern rhizo_crypt_core=target/release/librhizo_crypt_core.rlib \
    --extern tokio=target/release/deps/libtokio-*.rlib \
    --extern blake3=target/release/deps/libblake3-*.rlib \
    --extern hex=target/release/deps/libhex-*.rlib \
    -o /tmp/document_workflow 2>&1 | grep -v "warning" || true

echo "Running document workflow demo..."
echo ""
/tmp/document_workflow

echo ""
echo -e "${BLUE}═══════════════════════════════════════════════════════${NC}"
echo -e "${GREEN}✅ Document workflow demo complete!${NC}"
echo ""
echo -e "${YELLOW}📚 What you learned:${NC}"
echo "  • Multi-party document collaboration"
echo "  • Full version history (NestGate)"
echo "  • Legal review workflow"
echo "  • Cryptographic signatures (BearDog)"
echo "  • Immutable audit trail"
echo ""
echo -e "${YELLOW}▶ Next demo:${NC} ./demo-supply-chain.sh"
echo ""

rm -f /tmp/document_workflow.rs /tmp/document_workflow

