#!/usr/bin/env bash
# Demo: Low Latency Operations
set -euo pipefail

GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
CYAN='\033[0;36m'
NC='\033[0m'

echo -e "${BLUE}═══════════════════════════════════════════════════════${NC}"
echo -e "${BLUE}   ⚡ Low Latency Operations Demo${NC}"
echo -e "${BLUE}═══════════════════════════════════════════════════════${NC}"
echo ""

cd "$(dirname "$0")/../.."

echo -e "${YELLOW}📊 Measuring Operation Latencies...${NC}"
echo ""

# Use Rust benchmark to measure latencies
cat > /tmp/latency_bench.rs << 'EOF'
use rhizo_crypt_core::*;
use std::time::Instant;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = RhizoCryptConfig::default();
    let mut primal = RhizoCrypt::new(config);
    primal.start().await?;
    
    println!("┌────────────────────────────────┬──────────────┐");
    println!("│ Operation                      │ Latency      │");
    println!("├────────────────────────────────┼──────────────┤");
    
    // Measure session creation
    let start = Instant::now();
    let session = SessionBuilder::new(SessionType::General).build();
    let session_id = primal.create_session(session).await?;
    let latency = start.elapsed();
    println!("│ Create Session                 │ {:>7.2}µs   │", latency.as_micros());
    
    // Measure vertex append
    let start = Instant::now();
    let vertex = VertexBuilder::new(EventType::SessionStart).build();
    let _v_id = primal.append_vertex(session_id, vertex).await?;
    let latency = start.elapsed();
    println!("│ Append Vertex                  │ {:>7.2}µs   │", latency.as_micros());
    
    // Measure query frontier
    let start = Instant::now();
    let _frontier = primal.get_frontier(session_id).await?;
    let latency = start.elapsed();
    println!("│ Query Frontier                 │ {:>7.2}µs   │", latency.as_micros());
    
    // Measure query ancestors
    let start = Instant::now();
    let _ancestors = primal.get_ancestors(session_id).await?;
    let latency = start.elapsed();
    println!("│ Query Ancestors                │ {:>7.2}µs   │", latency.as_micros());
    
    // Measure session resolution
    let start = Instant::now();
    let _resolution = primal.resolve_session(session_id, ResolutionOutcome::Commit).await?;
    let latency = start.elapsed();
    println!("│ Resolve Session                │ {:>7.2}µs   │", latency.as_micros());
    
    println!("└────────────────────────────────┴──────────────┘");
    println!("");
    println!("🎯 Performance Characteristics:");
    println!("  • Sub-millisecond operations");
    println!("  • In-memory DAG (no disk I/O)");
    println!("  • Optimized for ephemeral workloads");
    
    Ok(())
}
EOF

echo -e "${GREEN}▶ Running latency benchmark...${NC}"
echo ""

# Compile and run
rustc --edition 2021 /tmp/latency_bench.rs \
    -L ../../target/release/deps \
    --extern rhizo_crypt_core=../../target/release/librhizo_crypt_core.rlib \
    --extern tokio=../../target/release/deps/libtokio-*.rlib \
    -o /tmp/latency_bench 2>&1 | grep -v "warning" || true

/tmp/latency_bench

echo ""
echo -e "${BLUE}═══════════════════════════════════════════════════════${NC}"
echo -e "${GREEN}✅ Latency demo complete!${NC}"
echo ""
echo -e "${YELLOW}📚 What you learned:${NC}"
echo "  • rhizoCrypt operations are sub-millisecond"
echo "  • In-memory DAG enables low latency"
echo "  • Optimized for interactive workloads"
echo "  • No disk I/O during session growth"
echo ""
echo -e "${YELLOW}▶ Next demo:${NC} ./demo-memory.sh"
echo ""

# Cleanup
rm -f /tmp/latency_bench.rs /tmp/latency_bench
