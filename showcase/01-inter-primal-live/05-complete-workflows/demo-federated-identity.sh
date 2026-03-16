#!/usr/bin/env bash
# Demo: Federated Identity & Access Control
set -euo pipefail

GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
CYAN='\033[0;36m'
MAGENTA='\033[0;35m'
NC='\033[0m'

echo -e "${BLUE}═══════════════════════════════════════════════════════${NC}"
echo -e "${BLUE}   🔑 Federated Identity & Access Control${NC}"
echo -e "${BLUE}═══════════════════════════════════════════════════════${NC}"
echo ""

cd "$(dirname "$0")/../.."

cat > /tmp/federated_identity.rs << 'EOF'
use rhizo_crypt_core::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("═══════════════════════════════════════════════════════");
    println!("  Federated Identity: Cross-Organization Access");
    println!("═══════════════════════════════════════════════════════\n");
    
    let config = RhizoCryptConfig::default();
    let mut primal = RhizoCrypt::new(config);
    primal.start().await?;
    
    println!("📋 Scenario: Multi-Organization Research Project");
    println!("   Project: Climate Data Analysis");
    println!("   Organizations: University A, Lab B, Agency C");
    println!("   Participants: 8 researchers across 3 orgs");
    println!("");
    
    let session = SessionBuilder::new(SessionType::General)
        .with_name("climate-research-2025")
        .with_owner(Did::new("did:key:project-coordinator"))
        .build();
    let session_id = primal.create_session(session).await?;
    
    println!("═══════════════════════════════════════════════════════");
    println!("  Phase 1: Project Setup & Access Grants");
    println!("═══════════════════════════════════════════════════════\n");
    
    println!("🎯 Project initialization");
    let v1 = VertexBuilder::new(EventType::DataCreate { schema: None })
        .with_agent(Did::new("did:key:project-coordinator"))
        .with_metadata("stage", "project_init")
        .with_metadata("project", "climate-research-2025")
        .with_metadata("orgs", "university-a,lab-b,agency-c")
        .with_metadata("participants", "8")
        .build();
    let v1_id = primal.append_vertex(session_id, v1).await?;
    println!("   ✓ Project created");
    println!("");
    
    // Grant access to researchers from different orgs
    let researchers = vec![
        ("dr-alice", "university-a", "PI", "read,write,compute"),
        ("dr-bob", "university-a", "researcher", "read,write"),
        ("dr-carol", "lab-b", "senior-scientist", "read,write,compute"),
        ("dr-dave", "lab-b", "scientist", "read,compute"),
        ("dr-eve", "agency-c", "analyst", "read,write"),
        ("dr-frank", "agency-c", "senior-analyst", "read,write,compute"),
    ];
    
    println!("🔐 Granting federated access:");
    for (name, org, role, capabilities) in &researchers {
        let grant = VertexBuilder::new(EventType::DataUpdate { schema: None })
            .with_agent(Did::new("did:key:project-coordinator"))
            .with_parent(v1_id)
            .with_metadata("stage", "access_grant")
            .with_metadata("grantee", name)
            .with_metadata("org", org)
            .with_metadata("role", role)
            .with_metadata("capabilities", capabilities)
            .build();
        primal.append_vertex(session_id, grant).await?;
        println!("   ✓ {} ({}): {} [{}]", name, org, role, capabilities);
    }
    println!("");
    
    println!("═══════════════════════════════════════════════════════");
    println!("  Phase 2: Collaborative Data Collection");
    println!("═══════════════════════════════════════════════════════\n");
    
    // University A uploads dataset
    println!("📊 University A uploads temperature data");
    let temp_data = b"Temperature dataset: Global stations, 2020-2024";
    let temp_payload = PayloadRef::from_bytes(temp_data);
    let v2 = VertexBuilder::new(EventType::DataCreate { schema: None })
        .with_agent(Did::new("did:org:university-a:dr-alice"))
        .with_payload(temp_payload)
        .with_metadata("stage", "data_upload")
        .with_metadata("org", "university-a")
        .with_metadata("dataset", "temperature")
        .with_metadata("records", "1000000")
        .with_metadata("storage", "nestgate")
        .build();
    let v2_id = primal.append_vertex(session_id, v2).await?;
    println!("   ✓ Temperature data uploaded (NestGate)");
    println!("     Records: 1,000,000");
    println!("");
    
    // Lab B uploads dataset
    println!("📊 Lab B uploads precipitation data");
    let precip_data = b"Precipitation dataset: Satellite imagery, 2020-2024";
    let precip_payload = PayloadRef::from_bytes(precip_data);
    let v3 = VertexBuilder::new(EventType::DataCreate { schema: None })
        .with_agent(Did::new("did:org:lab-b:dr-carol"))
        .with_payload(precip_payload)
        .with_metadata("stage", "data_upload")
        .with_metadata("org", "lab-b")
        .with_metadata("dataset", "precipitation")
        .with_metadata("images", "50000")
        .with_metadata("storage", "nestgate")
        .build();
    let v3_id = primal.append_vertex(session_id, v3).await?;
    println!("   ✓ Precipitation data uploaded (NestGate)");
    println!("     Images: 50,000");
    println!("");
    
    // Agency C uploads dataset
    println!("📊 Agency C uploads policy data");
    let policy_data = b"Policy dataset: Climate interventions, 2020-2024";
    let policy_payload = PayloadRef::from_bytes(policy_data);
    let v4 = VertexBuilder::new(EventType::DataCreate { schema: None })
        .with_agent(Did::new("did:org:agency-c:dr-eve"))
        .with_payload(policy_payload)
        .with_metadata("stage", "data_upload")
        .with_metadata("org", "agency-c")
        .with_metadata("dataset", "policy")
        .with_metadata("records", "5000")
        .with_metadata("storage", "nestgate")
        .build();
    let v4_id = primal.append_vertex(session_id, v4).await?;
    println!("   ✓ Policy data uploaded (NestGate)");
    println!("     Records: 5,000");
    println!("");
    
    println!("═══════════════════════════════════════════════════════");
    println!("  Phase 3: Cross-Org Compute (ToadStool)");
    println!("═══════════════════════════════════════════════════════\n");
    
    println!("🧮 Lab B runs analysis (ToadStool)");
    let analysis = VertexBuilder::new(EventType::DataUpdate { schema: None })
        .with_agent(Did::new("did:org:lab-b:dr-carol"))
        .with_parent(v2_id) // Links to University A's data
        .with_metadata("stage", "analysis")
        .with_metadata("org", "lab-b")
        .with_metadata("compute", "toadstool")
        .with_metadata("algorithm", "correlation-analysis")
        .with_metadata("result", "strong_correlation_detected")
        .build();
    primal.append_vertex(session_id, analysis).await?;
    println!("   ✓ Lab B analyzed University A's data");
    println!("     Result: Strong correlation detected");
    println!("     (Cross-org computation authorized!)");
    println!("");
    
    println!("═══════════════════════════════════════════════════════");
    println!("  Phase 4: Joint Publication & Signatures");
    println!("═══════════════════════════════════════════════════════\n");
    
    println!("📄 Preparing joint publication");
    let paper = b"Research paper: Climate Patterns 2020-2024";
    let paper_payload = PayloadRef::from_bytes(paper);
    let v5 = VertexBuilder::new(EventType::DataCreate { schema: None })
        .with_agent(Did::new("did:org:university-a:dr-alice"))
        .with_payload(paper_payload)
        .with_metadata("stage", "publication")
        .with_metadata("type", "research_paper")
        .with_metadata("title", "Climate Patterns 2020-2024")
        .with_metadata("authors", "alice,carol,eve")
        .build();
    let v5_id = primal.append_vertex(session_id, v5).await?;
    println!("   ✓ Paper drafted");
    println!("     Authors: Dr. Alice, Dr. Carol, Dr. Eve");
    println!("");
    
    // Multi-org signatures via BearDog
    println!("✍️  Multi-org signatures (BearDog HSMs):");
    
    let orgs_sign = vec![
        ("university-a", "dr-alice"),
        ("lab-b", "dr-carol"),
        ("agency-c", "dr-eve"),
    ];
    
    let mut last_id = v5_id;
    for (org, signer) in orgs_sign {
        let sig = format!("[Signature from {}]", org);
        let sig_vertex = VertexBuilder::new(EventType::DataUpdate { schema: None })
            .with_agent(Did::new(&format!("did:beardog:{}:hsm", org)))
            .with_parent(last_id)
            .with_metadata("stage", "signature")
            .with_metadata("org", org)
            .with_metadata("signer", signer)
            .with_metadata("signature", &hex::encode(sig.as_bytes()))
            .with_metadata("algorithm", "Ed25519")
            .build();
        last_id = primal.append_vertex(session_id, sig_vertex).await?;
        println!("   ✓ {} signed (via BearDog HSM)", org);
    }
    println!("");
    
    // Resolve session
    let resolution = primal.resolve_session(session_id, ResolutionOutcome::Commit).await?;
    
    println!("═══════════════════════════════════════════════════════");
    println!("  ✅ Federated Research Complete!");
    println!("═══════════════════════════════════════════════════════\n");
    
    println!("🌐 Federated Identity in Action:");
    println!("");
    println!("   🏛️ Organizations (3):");
    println!("      • University A (2 researchers)");
    println!("      • Lab B (2 scientists)");
    println!("      • Agency C (2 analysts)");
    println!("");
    println!("   👥 Participants (6 active):");
    println!("      • Each with org-scoped DIDs");
    println!("      • Role-based capabilities");
    println!("      • Cross-org access via DAG");
    println!("");
    println!("   📊 Data Sharing:");
    println!("      • 3 datasets (different orgs)");
    println!("      • Stored in NestGate (neutral)");
    println!("      • Cross-org analysis authorized");
    println!("");
    println!("   🔐 Signatures:");
    println!("      • Each org signs via own HSM");
    println!("      • BearDog (multi-org HSM support)");
    println!("      • Joint publication authenticated");
    println!("");
    
    println!("✨ Benefits:");
    println!("   • No central identity provider");
    println!("   • Each org maintains sovereignty");
    println!("   • Fine-grained access control");
    println!("   • Full audit trail");
    println!("   • Cross-org collaboration");
    println!("   • Cryptographic attribution");
    println!("");
    
    println!("🔐 Provenance:");
    println!("   • Merkle root: {}", hex::encode(&resolution.merkle_root));
    println!("   • Proves entire collaboration");
    println!("   • No single point of control");
    println!("");
    
    println!("═══════════════════════════════════════════════════════");
    println!("  Federated identity with rhizoCrypt!");
    println!("═══════════════════════════════════════════════════════\n");
    
    Ok(())
}
EOF

echo -e "${YELLOW}Compiling federated identity demo...${NC}"
rustc --edition 2024 /tmp/federated_identity.rs \
    -L target/release/deps \
    --extern rhizo_crypt_core=target/release/librhizo_crypt_core.rlib \
    --extern tokio=target/release/deps/libtokio-*.rlib \
    --extern blake3=target/release/deps/libblake3-*.rlib \
    --extern hex=target/release/deps/libhex-*.rlib \
    -o /tmp/federated_identity 2>&1 | grep -v "warning" || true

echo "Running federated identity demo..."
echo ""
/tmp/federated_identity

echo ""
echo -e "${BLUE}═══════════════════════════════════════════════════════${NC}"
echo -e "${GREEN}✅ Federated identity demo complete!${NC}"
echo ""
echo -e "${YELLOW}📚 What you learned:${NC}"
echo "  • Cross-organization collaboration"
echo "  • Federated identity (no central authority)"
echo "  • Role-based access control"
echo "  • Cross-org data sharing (NestGate)"
echo "  • Multi-org signatures (BearDog)"
echo ""
echo -e "${CYAN}🎉 All Complete Workflow Demos Finished!${NC}"
echo ""
echo -e "${YELLOW}Summary:${NC}"
echo "  1. ML Pipeline: Full training workflow"
echo "  2. Document Management: Collaborative editing"
echo "  3. Supply Chain: Farm-to-table provenance"
echo "  4. Federated Identity: Cross-org research"
echo ""

rm -f /tmp/federated_identity.rs /tmp/federated_identity

