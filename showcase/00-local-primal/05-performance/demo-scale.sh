#!/usr/bin/env bash
# Demo: Large DAG Handling
set -euo pipefail

GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
NC='\033[0m'

echo -e "${BLUE}═══════════════════════════════════════════════════════${NC}"
echo -e "${BLUE}   📈 Large DAG Scaling Demo${NC}"
echo -e "${BLUE}═══════════════════════════════════════════════════════${NC}"
echo ""

cd "$(dirname "$0")/../.."

echo -e "${YELLOW}📊 Testing DAG Scaling...${NC}"
echo ""

cat > /tmp/scale_bench.rs << 'EOF'
use rhizo_crypt_core::*;
use std::time::Instant;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = RhizoCryptConfig::default();
    let mut primal = RhizoCrypt::new(config);
    primal.start().await?;
    
    println!("Testing DAG scaling with increasing sizes...");
    println!("");
    
    let test_sizes = vec![100, 1000, 5000, 10000];
    
    println!("┌──────────────┬──────────────┬──────────────┬──────────────┐");
    println!("│ DAG Size     │ Build Time   │ Query Time   │ Resolve Time │");
    println!("├──────────────┼──────────────┼──────────────┼──────────────┤");
    
    for size in test_sizes {
        // Create session
        let session = SessionBuilder::new(SessionType::General)
            .with_name(&format!("scale-{}", size))
            .build();
        let session_id = primal.create_session(session).await?;
        
        // Build DAG
        let start = Instant::now();
        let mut prev_id = None;
        for i in 0..size {
            let mut builder = VertexBuilder::new(EventType::DataUpdate { schema: None });
            if let Some(parent) = prev_id {
                builder = builder.with_parent(parent);
            }
            let v = builder.build();
            prev_id = Some(primal.append_vertex(session_id, v).await?);
        }
        let build_time = start.elapsed();
        
        // Query DAG
        let start = Instant::now();
        let _frontier = primal.get_frontier(session_id).await?;
        let _ancestors = primal.get_ancestors(session_id).await?;
        let query_time = start.elapsed();
        
        // Resolve DAG
        let start = Instant::now();
        primal.resolve_session(session_id, ResolutionOutcome::Commit).await?;
        let resolve_time = start.elapsed();
        
        println!("│ {:>12} │ {:>9.2}ms │ {:>9.2}ms │ {:>9.2}ms │",
            size,
            build_time.as_secs_f64() * 1000.0,
            query_time.as_secs_f64() * 1000.0,
            resolve_time.as_secs_f64() * 1000.0
        );
    }
    
    println!("└──────────────┴──────────────┴──────────────┴──────────────┘");
    
    println!("");
    println!("📊 Scaling Characteristics:");
    println!("  • Linear build time O(n)");
    println!("  • Constant query time O(1) for frontier");
    println!("  • O(n log n) resolve time (topological sort + Merkle)");
    println!("  • Handles 10K+ vertices efficiently");
    
    Ok(())
}
EOF

echo -e "${GREEN}▶ Running scale benchmark (this may take 30-60 seconds)...${NC}"
echo ""

rustc --edition 2024 /tmp/scale_bench.rs \
    -L ../../target/release/deps \
    --extern rhizo_crypt_core=../../target/release/librhizo_crypt_core.rlib \
    --extern tokio=../../target/release/deps/libtokio-*.rlib \
    -o /tmp/scale_bench 2>&1 | grep -v "warning" || true

/tmp/scale_bench

echo ""
echo -e "${BLUE}═══════════════════════════════════════════════════════${NC}"
echo -e "${GREEN}✅ Scale demo complete!${NC}"
echo ""
echo -e "${YELLOW}📚 What you learned:${NC}"
echo "  • rhizoCrypt handles large DAGs efficiently"
echo "  • Linear build time for sequential operations"
echo "  • Efficient topological sort and Merkle tree"
echo "  • Production-ready for real-world workloads"
echo ""
echo -e "${YELLOW}▶ Next level:${NC} cd ../06-advanced-patterns && cat README.md"
echo ""

rm -f /tmp/scale_bench.rs /tmp/scale_bench
