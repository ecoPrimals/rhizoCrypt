#!/usr/bin/env bash
# Demo: DAG-Driven Compute with ToadStool
set -euo pipefail

GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
CYAN='\033[0;36m'
NC='\033[0m'

echo -e "${BLUE}═══════════════════════════════════════════════════════${NC}"
echo -e "${BLUE}   🧮 DAG-Driven Compute with ToadStool${NC}"
echo -e "${BLUE}═══════════════════════════════════════════════════════${NC}"
echo ""

cd "$(dirname "$0")/../.."

cat > /tmp/dag_compute.rs << 'EOF'
use rhizo_crypt_core::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("═══════════════════════════════════════════════════════");
    println!("  DAG-Driven Compute: Provenance + Execution");
    println!("═══════════════════════════════════════════════════════\n");
    
    let config = RhizoCryptConfig::default();
    let mut primal = RhizoCrypt::new(config);
    primal.start().await?;
    
    // Create session for compute workflow
    let session = SessionBuilder::new(SessionType::General)
        .with_name("compute-workflow")
        .build();
    let session_id = primal.create_session(session).await?;
    
    println!("📋 Scenario: ML Training with Provenance");
    println!("");
    
    // Step 1: Data preparation
    println!("📝 Step 1: Prepare Dataset");
    let dataset = PayloadRef::from_bytes(b"Training data: [1,2,3,4,5]");
    let v1 = VertexBuilder::new(EventType::DataCreate { schema: None })
        .with_agent(Did::new("did:key:data-scientist"))
        .with_payload(dataset)
        .with_metadata("stage", "data_prep")
        .with_metadata("dataset", "mnist-subset")
        .with_metadata("size", "50000")
        .build();
    let v1_id = primal.append_vertex(session_id, v1).await?;
    println!("   ✓ Dataset prepared: {}", v1_id);
    println!("");
    
    // Step 2: Request compute
    println!("🧮 Step 2: Request Compute from ToadStool");
    let compute_request = VertexBuilder::new(EventType::DataUpdate { schema: None })
        .with_agent(Did::new("did:key:ml-pipeline"))
        .with_parent(v1_id)
        .with_metadata("stage", "compute_request")
        .with_metadata("compute_type", "gpu")
        .with_metadata("framework", "pytorch")
        .with_metadata("epochs", "10")
        .build();
    let v2_id = primal.append_vertex(session_id, compute_request).await?;
    println!("   ✓ Compute requested: {}", v2_id);
    println!("   • GPU compute");
    println!("   • PyTorch framework");
    println!("   • 10 training epochs");
    println!("");
    
    // Step 3: Compute execution
    println!("⚙️  Step 3: ToadStool Executes Training");
    println!("   (Simulating GPU training...)");
    std::thread::sleep(std::time::Duration::from_millis(500));
    
    let model_payload = PayloadRef::from_bytes(b"Trained model weights: [0.42, 0.87, ...]");
    let compute_result = VertexBuilder::new(EventType::DataUpdate { schema: None })
        .with_agent(Did::new("did:toadstool:gpu-node-7"))
        .with_parent(v2_id)
        .with_payload(model_payload)
        .with_metadata("stage", "compute_complete")
        .with_metadata("accuracy", "0.95")
        .with_metadata("loss", "0.05")
        .with_metadata("duration_sec", "127")
        .with_metadata("gpu_hours", "0.035")
        .build();
    let v3_id = primal.append_vertex(session_id, compute_result).await?;
    println!("   ✓ Training complete: {}", v3_id);
    println!("   • Accuracy: 0.95");
    println!("   • Loss: 0.05");
    println!("   • Duration: 127 seconds");
    println!("   • GPU hours: 0.035");
    println!("");
    
    // Step 4: Validation
    println!("✅ Step 4: Validate Results");
    let validation = VertexBuilder::new(EventType::DataUpdate { schema: None })
        .with_agent(Did::new("did:key:data-scientist"))
        .with_parent(v3_id)
        .with_metadata("stage", "validation")
        .with_metadata("status", "approved")
        .with_metadata("test_accuracy", "0.94")
        .build();
    let _v4_id = primal.append_vertex(session_id, validation).await?;
    println!("   ✓ Model validated");
    println!("   • Test accuracy: 0.94");
    println!("   • Status: Approved");
    println!("");
    
    // Show DAG
    println!("🔄 Complete Compute Workflow:");
    println!("");
    println!("   ┌────────────────┐");
    println!("   │ Data Prep      │  Scientist prepares dataset");
    println!("   │ (Scientist)    │");
    println!("   └────────┬───────┘");
    println!("            │");
    println!("   ┌────────▼───────┐");
    println!("   │ Request Compute│  ML pipeline requests GPU");
    println!("   │ (ML Pipeline)  │");
    println!("   └────────┬───────┘");
    println!("            │");
    println!("   ┌────────▼───────┐");
    println!("   │ Train Model    │  ToadStool executes on GPU");
    println!("   │ (ToadStool)    │");
    println!("   └────────┬───────┘");
    println!("            │");
    println!("   ┌────────▼───────┐");
    println!("   │ Validate       │  Scientist approves");
    println!("   │ (Scientist)    │");
    println!("   └────────────────┘");
    println!("");
    
    // Resolve session
    let resolution = primal.resolve_session(session_id, ResolutionOutcome::Commit).await?;
    
    println!("🔐 Provenance Proof:");
    println!("   • Full training lineage captured");
    println!("   • Dataset → Request → Execution → Validation");
    println!("   • Multi-agent workflow (3 agents)");
    println!("   • Merkle root: {}", hex::encode(&resolution.merkle_root));
    println!("   • Cryptographic proof of entire workflow");
    println!("");
    
    println!("═══════════════════════════════════════════════════════");
    println!("  ✨ DAG-Driven Compute Benefits:");
    println!("═══════════════════════════════════════════════════════");
    println!("  rhizoCrypt (DAG):          ToadStool (Compute):");
    println!("  • Training provenance      • GPU execution");
    println!("  • Dataset lineage          • ML frameworks");
    println!("  • Agent attribution        • Resource allocation");
    println!("  • Result validation        • Performance metrics");
    println!("  • Audit trail              • Cost tracking");
    println!("");
    println!("  Together: Reproducible ML with full provenance!");
    println!("═══════════════════════════════════════════════════════\n");
    
    Ok(())
}
EOF

echo -e "${YELLOW}Compiling DAG-compute demo...${NC}"
rustc --edition 2021 /tmp/dag_compute.rs \
    -L target/release/deps \
    --extern rhizo_crypt_core=target/release/librhizo_crypt_core.rlib \
    --extern tokio=target/release/deps/libtokio-*.rlib \
    --extern blake3=target/release/deps/libblake3-*.rlib \
    --extern hex=target/release/deps/libhex-*.rlib \
    -o /tmp/dag_compute 2>&1 | grep -v "warning" || true

echo "Running DAG-compute demo..."
echo ""
/tmp/dag_compute

echo ""
echo -e "${BLUE}═══════════════════════════════════════════════════════${NC}"
echo -e "${GREEN}✅ DAG-driven compute demo complete!${NC}"
echo ""
echo -e "${YELLOW}📚 What you learned:${NC}"
echo "  • rhizoCrypt tracks ML training provenance"
echo "  • ToadStool provides GPU compute"
echo "  • Full workflow captured in DAG"
echo "  • Multi-agent collaboration"
echo "  • Reproducible ML with audit trail"
echo ""
echo -e "${YELLOW}▶ Next demo:${NC} ./demo-gpu-provenance.sh"
echo ""

rm -f /tmp/dag_compute.rs /tmp/dag_compute

