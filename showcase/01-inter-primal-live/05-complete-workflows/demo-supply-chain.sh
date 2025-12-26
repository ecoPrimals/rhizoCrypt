#!/usr/bin/env bash
# Demo: Supply Chain Provenance
set -euo pipefail

GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
CYAN='\033[0;36m'
NC='\033[0m'

echo -e "${BLUE}═══════════════════════════════════════════════════════${NC}"
echo -e "${BLUE}   📦 Supply Chain Provenance${NC}"
echo -e "${BLUE}═══════════════════════════════════════════════════════${NC}"
echo ""

cd "$(dirname "$0")/../.."

cat > /tmp/supply_chain.rs << 'EOF'
use rhizo_crypt_core::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("═══════════════════════════════════════════════════════");
    println!("  Supply Chain: Farm to Table Provenance");
    println!("═══════════════════════════════════════════════════════\n");
    
    let config = RhizoCryptConfig::default();
    let mut primal = RhizoCrypt::new(config);
    primal.start().await?;
    
    println!("📋 Scenario: Organic Coffee Bean Supply Chain");
    println!("   Product: Single-origin Ethiopian coffee");
    println!("   Journey: Farm → Processing → Export → Roasting → Retail");
    println!("");
    
    let session = SessionBuilder::new(SessionType::General)
        .with_name("coffee-batch-2025-12")
        .with_owner(Did::new("did:key:supply-chain-coordinator"))
        .build();
    let session_id = primal.create_session(session).await?;
    
    println!("═══════════════════════════════════════════════════════");
    println!("  Stage 1: Harvesting (Farm)");
    println!("═══════════════════════════════════════════════════════\n");
    
    println!("🌱 Coffee beans harvested");
    let harvest_data = b"Harvest data: Location, weather, soil conditions...";
    let harvest_payload = PayloadRef::from_bytes(harvest_data);
    
    let v1 = VertexBuilder::new(EventType::DataCreate { schema: None })
        .with_agent(Did::new("did:key:farmer-abebe"))
        .with_payload(harvest_payload)
        .with_metadata("stage", "harvest")
        .with_metadata("location", "Yirgacheffe, Ethiopia")
        .with_metadata("date", "2025-11-15")
        .with_metadata("batch_kg", "500")
        .with_metadata("variety", "Heirloom")
        .with_metadata("organic", "certified")
        .with_metadata("storage", "nestgate")
        .build();
    let v1_id = primal.append_vertex(session_id, v1).await?;
    
    println!("   ✓ Harvest recorded");
    println!("     Location: Yirgacheffe, Ethiopia");
    println!("     Batch: 500 kg");
    println!("     Variety: Heirloom");
    println!("     Organic: Certified");
    println!("     Farmer: Abebe");
    println!("");
    
    println!("═══════════════════════════════════════════════════════");
    println!("  Stage 2: Processing");
    println!("═══════════════════════════════════════════════════════\n");
    
    println!("🏭 Processing (washing, drying)");
    let v2 = VertexBuilder::new(EventType::DataUpdate { schema: None })
        .with_agent(Did::new("did:key:processor-cooperative"))
        .with_parent(v1_id)
        .with_metadata("stage", "processing")
        .with_metadata("method", "washed")
        .with_metadata("drying", "sun-dried")
        .with_metadata("duration_days", "14")
        .with_metadata("final_weight_kg", "420")
        .with_metadata("moisture_pct", "11")
        .build();
    let v2_id = primal.append_vertex(session_id, v2).await?;
    
    println!("   ✓ Processing complete");
    println!("     Method: Washed");
    println!("     Drying: Sun-dried (14 days)");
    println!("     Final weight: 420 kg");
    println!("     Moisture: 11%");
    println!("");
    
    println!("═══════════════════════════════════════════════════════");
    println!("  Stage 3: Quality Control & Export");
    println!("═══════════════════════════════════════════════════════\n");
    
    println!("✅ Quality inspection");
    let v3 = VertexBuilder::new(EventType::DataUpdate { schema: None })
        .with_agent(Did::new("did:key:inspector-ministry"))
        .with_parent(v2_id)
        .with_metadata("stage", "quality_control")
        .with_metadata("grade", "Grade 1")
        .with_metadata("screen_size", "15+")
        .with_metadata("defects", "0")
        .with_metadata("cupping_score", "87")
        .with_metadata("status", "approved_export")
        .build();
    let v3_id = primal.append_vertex(session_id, v3).await?;
    
    println!("   ✓ Quality approved");
    println!("     Grade: Grade 1");
    println!("     Cupping score: 87/100");
    println!("     Defects: 0");
    println!("     Export: Approved");
    println!("");
    
    println!("🚢 Export shipment");
    let v4 = VertexBuilder::new(EventType::DataUpdate { schema: None })
        .with_agent(Did::new("did:key:exporter-international"))
        .with_parent(v3_id)
        .with_metadata("stage", "export")
        .with_metadata("container", "MSCU1234567")
        .with_metadata("departure_port", "Djibouti")
        .with_metadata("destination_port", "Rotterdam")
        .with_metadata("departure_date", "2025-12-01")
        .with_metadata("arrival_date", "2025-12-18")
        .build();
    let v4_id = primal.append_vertex(session_id, v4).await?;
    
    println!("   ✓ Shipment dispatched");
    println!("     Container: MSCU1234567");
    println!("     Route: Djibouti → Rotterdam");
    println!("     Transit: 17 days");
    println!("");
    
    println!("═══════════════════════════════════════════════════════");
    println!("  Stage 4: Roasting");
    println!("═══════════════════════════════════════════════════════\n");
    
    println!("🔥 Roasting");
    let roast_data = b"Roast profile: Temperature curve, duration, color...";
    let roast_payload = PayloadRef::from_bytes(roast_data);
    
    let v5 = VertexBuilder::new(EventType::DataUpdate { schema: None })
        .with_agent(Did::new("did:key:roaster-amsterdam"))
        .with_parent(v4_id)
        .with_payload(roast_payload)
        .with_metadata("stage", "roasting")
        .with_metadata("roast_level", "medium")
        .with_metadata("date", "2025-12-20")
        .with_metadata("batch_kg", "400")
        .with_metadata("roaster", "Probat L12")
        .with_metadata("profile_storage", "nestgate")
        .build();
    let v5_id = primal.append_vertex(session_id, v5).await?;
    
    println!("   ✓ Roasting complete");
    println!("     Level: Medium");
    println!("     Batch: 400 kg");
    println!("     Roaster: Amsterdam");
    println!("     Profile stored: NestGate");
    println!("");
    
    println!("═══════════════════════════════════════════════════════");
    println!("  Stage 5: Packaging & Retail");
    println!("═══════════════════════════════════════════════════════\n");
    
    println!("📦 Packaging");
    let v6 = VertexBuilder::new(EventType::DataUpdate { schema: None })
        .with_agent(Did::new("did:key:packager-distribution"))
        .with_parent(v5_id)
        .with_metadata("stage", "packaging")
        .with_metadata("package_size_g", "250")
        .with_metadata("packages_count", "1600")
        .with_metadata("packaging_type", "compostable")
        .with_metadata("label", "Single-Origin Ethiopian")
        .build();
    let v6_id = primal.append_vertex(session_id, v6).await?;
    
    println!("   ✓ Packaging complete");
    println!("     Packages: 1,600 × 250g");
    println!("     Type: Compostable");
    println!("     Label: Single-Origin Ethiopian");
    println!("");
    
    // QR code signature
    println!("🔐 Batch signature (BearDog)");
    let batch_sig = b"[Batch signature for QR codes]";
    let v7 = VertexBuilder::new(EventType::DataUpdate { schema: None })
        .with_agent(Did::new("did:beardog:signing-service"))
        .with_parent(v6_id)
        .with_metadata("stage", "signing")
        .with_metadata("signature", &hex::encode(batch_sig))
        .with_metadata("algorithm", "Ed25519")
        .with_metadata("purpose", "qr_code_authentication")
        .build();
    let _v7_id = primal.append_vertex(session_id, v7).await?;
    
    println!("   ✓ Batch signed");
    println!("     Each package gets unique QR code");
    println!("     QR links to this provenance DAG");
    println!("");
    
    // Resolve session
    let resolution = primal.resolve_session(session_id, ResolutionOutcome::Commit).await?;
    
    println!("═══════════════════════════════════════════════════════");
    println!("  ✅ Supply Chain Complete!");
    println!("═══════════════════════════════════════════════════════\n");
    
    println!("📊 Complete Journey:");
    println!("");
    println!("   🌱 Farmer (Ethiopia)      → Harvest 500 kg");
    println!("   🏭 Processor              → Wash & dry → 420 kg");
    println!("   ✅ Inspector              → Grade 1, score 87");
    println!("   🚢 Exporter               → Ship to Rotterdam");
    println!("   🔥 Roaster (Amsterdam)    → Medium roast");
    println!("   📦 Packager               → 1,600 packages");
    println!("   🔐 BearDog                → Sign each QR code");
    println!("");
    
    println!("🔐 Provenance Proof:");
    println!("   • Complete farm-to-table lineage");
    println!("   • 7 stages, 7 agents");
    println!("   • All data stored (NestGate)");
    println!("   • Cryptographic signatures (BearDog)");
    println!("   • Merkle root: {}", hex::encode(&resolution.merkle_root));
    println!("");
    
    println!("📱 Consumer Experience:");
    println!("   1. Scan QR code on package");
    println!("   2. View complete provenance:");
    println!("      • Farmer name & location");
    println!("      • Harvest date & conditions");
    println!("      • Processing method");
    println!("      • Quality scores");
    println!("      • Shipping route");
    println!("      • Roast profile");
    println!("   3. Verify authenticity (signature)");
    println!("");
    
    println!("✨ Benefits:");
    println!("   • Consumer trust (full transparency)");
    println!("   • Fair pricing (farmer attribution)");
    println!("   • Quality assurance (cupping scores)");
    println!("   • Anti-counterfeiting (signatures)");
    println!("   • Sustainability (organic certification)");
    println!("   • Recall capability (batch tracking)");
    println!("");
    
    println!("═══════════════════════════════════════════════════════");
    println!("  Farm-to-table provenance with rhizoCrypt!");
    println!("═══════════════════════════════════════════════════════\n");
    
    Ok(())
}
EOF

echo -e "${YELLOW}Compiling supply chain demo...${NC}"
rustc --edition 2021 /tmp/supply_chain.rs \
    -L target/release/deps \
    --extern rhizo_crypt_core=target/release/librhizo_crypt_core.rlib \
    --extern tokio=target/release/deps/libtokio-*.rlib \
    --extern blake3=target/release/deps/libblake3-*.rlib \
    --extern hex=target/release/deps/libhex-*.rlib \
    -o /tmp/supply_chain 2>&1 | grep -v "warning" || true

echo "Running supply chain demo..."
echo ""
/tmp/supply_chain

echo ""
echo -e "${BLUE}═══════════════════════════════════════════════════════${NC}"
echo -e "${GREEN}✅ Supply chain demo complete!${NC}"
echo ""
echo -e "${YELLOW}📚 What you learned:${NC}"
echo "  • Complete farm-to-table provenance"
echo "  • Multi-agent supply chain"
echo "  • Quality and certification tracking"
echo "  • Consumer transparency (QR codes)"
echo "  • Anti-counterfeiting (BearDog signatures)"
echo ""
echo -e "${YELLOW}▶ Next demo:${NC} ./demo-federated-identity.sh"
echo ""

rm -f /tmp/supply_chain.rs /tmp/supply_chain

