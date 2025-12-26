#!/usr/bin/env bash
# Demo: Distributed Compute Across Multiple ToadStool Nodes
set -euo pipefail

GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
CYAN='\033[0;36m'
MAGENTA='\033[0;35m'
NC='\033[0m'

echo -e "${BLUE}═══════════════════════════════════════════════════════${NC}"
echo -e "${BLUE}   🌐 Distributed Compute Orchestration${NC}"
echo -e "${BLUE}═══════════════════════════════════════════════════════${NC}"
echo ""

cd "$(dirname "$0")/../.."

cat > /tmp/distributed_compute.rs << 'EOF'
use rhizo_crypt_core::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("═══════════════════════════════════════════════════════");
    println!("  Distributed Compute: Multi-Node Orchestration");
    println!("═══════════════════════════════════════════════════════\n");
    
    let config = RhizoCryptConfig::default();
    let mut primal = RhizoCrypt::new(config);
    primal.start().await?;
    
    let session = SessionBuilder::new(SessionType::General)
        .with_name("distributed-inference")
        .build();
    let session_id = primal.create_session(session).await?;
    
    println!("📋 Scenario: Distributed Inference Across Data Centers");
    println!("");
    
    // Request distributed inference
    println!("📝 Request: Global Inference Service");
    let request = VertexBuilder::new(EventType::DataCreate { schema: None })
        .with_agent(Did::new("did:key:service-operator"))
        .with_metadata("service", "global-inference")
        .with_metadata("regions", "us-west,eu-central,asia-east")
        .with_metadata("model", "llama-3-70b")
        .with_metadata("replicas", "12")
        .build();
    let req_id = primal.append_vertex(session_id, request).await?;
    println!("   ✓ Service requested");
    println!("     Model: Llama 3 70B");
    println!("     Regions: 3 (US, EU, Asia)");
    println!("     Replicas: 12");
    println!("");
    
    // Simulate distributed nodes
    let regions = vec![
        ("us-west", "Portland", vec!["node-0", "node-1", "node-2", "node-3"]),
        ("eu-central", "Frankfurt", vec!["node-4", "node-5", "node-6", "node-7"]),
        ("asia-east", "Tokyo", vec!["node-8", "node-9", "node-10", "node-11"]),
    ];
    
    println!("⚙️  ToadStool Nodes Activate:");
    println!("");
    
    for (region_id, location, nodes) in regions {
        println!("   🌍 Region: {} ({})", location, region_id);
        
        for node_id in nodes {
            let node_vertex = VertexBuilder::new(EventType::DataUpdate { schema: None })
                .with_agent(Did::new(&format!("did:toadstool:{}:{}", region_id, node_id)))
                .with_parent(req_id)
                .with_metadata("region", region_id)
                .with_metadata("location", location)
                .with_metadata("node_id", node_id)
                .with_metadata("status", "ready")
                .with_metadata("gpu_count", "8")
                .with_metadata("latency_ms", &format!("{}", rand::random::<u32>() % 50 + 10))
                .build();
            
            primal.append_vertex(session_id, node_vertex).await?;
            println!("      ✓ {} ready", node_id);
        }
        println!("");
    }
    
    // Simulate inference requests
    println!("📊 Processing Inference Requests:");
    println!("");
    
    let requests_data = vec![
        ("Request from Seattle", "us-west:node-1", "42ms"),
        ("Request from Berlin", "eu-central:node-5", "38ms"),
        ("Request from Singapore", "asia-east:node-9", "55ms"),
        ("Request from London", "eu-central:node-6", "28ms"),
        ("Request from San Francisco", "us-west:node-2", "45ms"),
    ];
    
    for (request_desc, serving_node, latency) in requests_data {
        let inference = VertexBuilder::new(EventType::DataUpdate { schema: None })
            .with_agent(Did::new(&format!("did:toadstool:{}", serving_node)))
            .with_metadata("request", request_desc)
            .with_metadata("served_by", serving_node)
            .with_metadata("latency", latency)
            .with_metadata("tokens", &format!("{}", rand::random::<u32>() % 200 + 100))
            .build();
        primal.append_vertex(session_id, inference).await?;
        println!("   ✓ {}: {} ({})", request_desc, serving_node, latency);
    }
    println!("");
    
    // Aggregate metrics
    println!("📈 Global Metrics:");
    let metrics = VertexBuilder::new(EventType::DataUpdate { schema: None })
        .with_agent(Did::new("did:toadstool:global-coordinator"))
        .with_metadata("stage", "metrics")
        .with_metadata("total_nodes", "12")
        .with_metadata("total_requests", "5")
        .with_metadata("avg_latency_ms", "41.6")
        .with_metadata("total_tokens", "750")
        .with_metadata("uptime_pct", "99.99")
        .build();
    primal.append_vertex(session_id, metrics).await?;
    
    println!("   • Total nodes: 12 (across 3 regions)");
    println!("   • Requests served: 5");
    println!("   • Avg latency: 41.6ms");
    println!("   • Total tokens: 750");
    println!("   • Uptime: 99.99%");
    println!("");
    
    println!("🌍 Geographic Distribution:");
    println!("");
    println!("   ┌─────────────────────────────────────────────────┐");
    println!("   │                                                 │");
    println!("   │  🇺🇸 US West (Portland)    🇪🇺 EU (Frankfurt)   │");
    println!("   │     ├─ node-0                ├─ node-4          │");
    println!("   │     ├─ node-1                ├─ node-5          │");
    println!("   │     ├─ node-2                ├─ node-6          │");
    println!("   │     └─ node-3                └─ node-7          │");
    println!("   │                                                 │");
    println!("   │              🇯🇵 Asia East (Tokyo)              │");
    println!("   │                 ├─ node-8                       │");
    println!("   │                 ├─ node-9                       │");
    println!("   │                 ├─ node-10                      │");
    println!("   │                 └─ node-11                      │");
    println!("   │                                                 │");
    println!("   └─────────────────────────────────────────────────┘");
    println!("");
    
    let resolution = primal.resolve_session(session_id, ResolutionOutcome::Commit).await?;
    
    println!("🔐 Distributed Provenance:");
    println!("");
    println!("   ✨ Benefits:");
    println!("      • Global service topology captured");
    println!("      • Per-node performance tracked");
    println!("      • Request routing documented");
    println!("      • Geographic attribution");
    println!("      • Full audit trail");
    println!("");
    println!("   🎯 Use Cases:");
    println!("      • SLA verification");
    println!("      • Cost allocation per region");
    println!("      • Performance debugging");
    println!("      • Capacity planning");
    println!("      • Compliance (data residency)");
    println!("");
    println!("   Merkle root: {}", hex::encode(&resolution.merkle_root));
    println!("");
    
    println!("═══════════════════════════════════════════════════════");
    println!("  ✨ Distributed Compute with rhizoCrypt:");
    println!("═══════════════════════════════════════════════════════");
    println!("  • Multi-region orchestration");
    println!("  • Per-node attribution");
    println!("  • Global provenance in single DAG");
    println!("  • Geo-distributed accountability");
    println!("  • Cryptographic proof of all compute");
    println!("═══════════════════════════════════════════════════════\n");
    
    Ok(())
}

// Fake rand for demo
mod rand {
    pub fn random<T>() -> T where T: Default {
        T::default()
    }
}
EOF

echo -e "${YELLOW}Compiling distributed compute demo...${NC}"
rustc --edition 2021 /tmp/distributed_compute.rs \
    -L target/release/deps \
    --extern rhizo_crypt_core=target/release/librhizo_crypt_core.rlib \
    --extern tokio=target/release/deps/libtokio-*.rlib \
    --extern blake3=target/release/deps/libblake3-*.rlib \
    --extern hex=target/release/deps/libhex-*.rlib \
    -o /tmp/distributed_compute 2>&1 | grep -v "warning" || true

echo "Running distributed compute demo..."
echo ""
/tmp/distributed_compute

echo ""
echo -e "${BLUE}═══════════════════════════════════════════════════════${NC}"
echo -e "${GREEN}✅ Distributed compute demo complete!${NC}"
echo ""
echo -e "${YELLOW}📚 What you learned:${NC}"
echo "  • Multi-region compute orchestration"
echo "  • Geographic node attribution"
echo "  • Global provenance in single DAG"
echo "  • Performance tracking per region"
echo "  • Distributed accountability"
echo ""
echo -e "${CYAN}🎉 ToadStool Compute Integration Complete!${NC}"
echo ""
echo -e "${YELLOW}▶ Next:${NC} End-to-end workflow demos"
echo "   cd ../05-complete-workflows"
echo ""

rm -f /tmp/distributed_compute.rs /tmp/distributed_compute

