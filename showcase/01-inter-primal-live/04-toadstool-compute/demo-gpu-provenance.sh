#!/usr/bin/env bash
# Demo: GPU Provenance with ToadStool
set -euo pipefail

GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
CYAN='\033[0;36m'
NC='\033[0m'

echo -e "${BLUE}═══════════════════════════════════════════════════════${NC}"
echo -e "${BLUE}   🎮 GPU Provenance Tracking${NC}"
echo -e "${BLUE}═══════════════════════════════════════════════════════${NC}"
echo ""

cd "$(dirname "$0")/../.."

cat > /tmp/gpu_provenance.rs << 'EOF'
use rhizo_crypt_core::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("═══════════════════════════════════════════════════════");
    println!("  GPU Provenance: Hardware Attribution");
    println!("═══════════════════════════════════════════════════════\n");
    
    let config = RhizoCryptConfig::default();
    let mut primal = RhizoCrypt::new(config);
    primal.start().await?;
    
    let session = SessionBuilder::new(SessionType::General)
        .with_name("gpu-tracking")
        .build();
    let session_id = primal.create_session(session).await?;
    
    println!("📋 Scenario: Multi-GPU Training with Attribution");
    println!("");
    
    // Request compute job
    println!("📝 Request: Train Large Model");
    let request = VertexBuilder::new(EventType::DataCreate { schema: None })
        .with_agent(Did::new("did:key:researcher"))
        .with_metadata("job", "train-llm")
        .with_metadata("gpus_requested", "8")
        .with_metadata("framework", "pytorch")
        .with_metadata("model_size", "7B")
        .build();
    let req_id = primal.append_vertex(session_id, request).await?;
    println!("   ✓ Job requested");
    println!("     Job: train-llm");
    println!("     GPUs: 8");
    println!("     Model: 7B parameters");
    println!("");
    
    // Simulate multi-GPU execution
    println!("⚙️  ToadStool Executes on 8 GPUs:");
    println!("");
    
    for gpu_id in 0..8 {
        println!("   GPU {}: Training...", gpu_id);
        std::thread::sleep(std::time::Duration::from_millis(50));
        
        let gpu_vertex = VertexBuilder::new(EventType::DataUpdate { schema: None })
            .with_agent(Did::new(&format!("did:toadstool:gpu-{}", gpu_id)))
            .with_parent(req_id)
            .with_metadata("gpu_id", &gpu_id.to_string())
            .with_metadata("gpu_model", "NVIDIA A100")
            .with_metadata("memory_gb", "80")
            .with_metadata("utilization", "98%")
            .with_metadata("power_watts", "350")
            .with_metadata("batch_processed", &(gpu_id * 1000).to_string())
            .build();
        
        let gpu_vid = primal.append_vertex(session_id, gpu_vertex).await?;
        println!("     ✓ Vertex: {} (GPU {})", gpu_vid, gpu_id);
    }
    
    println!("");
    println!("✨ Hardware Attribution:");
    println!("");
    println!("   Each GPU creates a vertex:");
    println!("   • Unique DID (did:toadstool:gpu-N)");
    println!("   • Hardware specs (A100, 80GB)");
    println!("   • Performance metrics (utilization, power)");
    println!("   • Work done (batches processed)");
    println!("");
    
    // Aggregate results
    println!("📊 Aggregate Results:");
    let aggregate = VertexBuilder::new(EventType::DataUpdate { schema: None })
        .with_agent(Did::new("did:toadstool:coordinator"))
        .with_metadata("stage", "aggregate")
        .with_metadata("total_gpus", "8")
        .with_metadata("total_batches", "28000")
        .with_metadata("training_time", "4.2h")
        .with_metadata("total_gpu_hours", "33.6")
        .with_metadata("total_kwh", "11.76")
        .with_metadata("carbon_kg", "5.88")
        .build();
    primal.append_vertex(session_id, aggregate).await?;
    
    println!("   • Total GPUs: 8");
    println!("   • Total batches: 28,000");
    println!("   • Training time: 4.2 hours");
    println!("   • GPU hours: 33.6");
    println!("   • Energy: 11.76 kWh");
    println!("   • Carbon: 5.88 kg CO₂");
    println!("");
    
    println!("🔄 Provenance Graph:");
    println!("");
    println!("                ┌──────────────┐");
    println!("                │   Request    │");
    println!("                │ (Researcher) │");
    println!("                └──────┬───────┘");
    println!("                       │");
    println!("        ┌──────────────┼──────────────┐");
    println!("        │              │              │");
    println!("   ┌────▼───┐     ┌────▼───┐    ┌────▼───┐");
    println!("   │ GPU 0  │ ... │ GPU 4  │... │ GPU 7  │");
    println!("   │ A100   │     │ A100   │    │ A100   │");
    println!("   └────┬───┘     └────┬───┘    └────┬───┘");
    println!("        │              │              │");
    println!("        └──────────────┼──────────────┘");
    println!("                       │");
    println!("                ┌──────▼───────┐");
    println!("                │  Aggregate   │");
    println!("                │(Coordinator) │");
    println!("                └──────────────┘");
    println!("");
    
    let resolution = primal.resolve_session(session_id, ResolutionOutcome::Commit).await?;
    
    println!("🔐 Provenance Benefits:");
    println!("");
    println!("   🎯 Accountability:");
    println!("      • Which GPU did what work");
    println!("      • Performance per GPU");
    println!("      • Power consumption tracking");
    println!("");
    println!("   💰 Cost Attribution:");
    println!("      • GPU hours per user");
    println!("      • Energy costs");
    println!("      • Fair billing");
    println!("");
    println!("   🌍 Environmental Impact:");
    println!("      • Carbon footprint");
    println!("      • Energy efficiency");
    println!("      • Green compute metrics");
    println!("");
    println!("   🔍 Debugging:");
    println!("      • Which GPU underperformed");
    println!("      • Bottleneck identification");
    println!("      • Reproducibility");
    println!("");
    println!("   Merkle root: {}", hex::encode(&resolution.merkle_root));
    println!("");
    
    println!("═══════════════════════════════════════════════════════");
    println!("  ✨ Hardware-Level Provenance:");
    println!("═══════════════════════════════════════════════════════");
    println!("  • Every GPU gets a DID");
    println!("  • Full hardware attribution");
    println!("  • Cost and carbon tracking");
    println!("  • Reproducible compute");
    println!("  • Cryptographic proof of all work");
    println!("═══════════════════════════════════════════════════════\n");
    
    Ok(())
}
EOF

echo -e "${YELLOW}Compiling GPU provenance demo...${NC}"
rustc --edition 2021 /tmp/gpu_provenance.rs \
    -L target/release/deps \
    --extern rhizo_crypt_core=target/release/librhizo_crypt_core.rlib \
    --extern tokio=target/release/deps/libtokio-*.rlib \
    --extern blake3=target/release/deps/libblake3-*.rlib \
    --extern hex=target/release/deps/libhex-*.rlib \
    -o /tmp/gpu_provenance 2>&1 | grep -v "warning" || true

echo "Running GPU provenance demo..."
echo ""
/tmp/gpu_provenance

echo ""
echo -e "${BLUE}═══════════════════════════════════════════════════════${NC}"
echo -e "${GREEN}✅ GPU provenance demo complete!${NC}"
echo ""
echo -e "${YELLOW}📚 What you learned:${NC}"
echo "  • Hardware-level attribution (each GPU = DID)"
echo "  • Performance and power tracking"
echo "  • Cost and carbon accounting"
echo "  • Multi-GPU coordination"
echo "  • Reproducible compute workflows"
echo ""
echo -e "${YELLOW}▶ Next demo:${NC} ./demo-distributed-compute.sh"
echo ""

rm -f /tmp/gpu_provenance.rs /tmp/gpu_provenance

