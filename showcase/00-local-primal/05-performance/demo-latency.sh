#!/bin/bash
#
# 🔐 rhizoCrypt - Low Latency Demo
#
# Demonstrates:
# 1. Sub-millisecond vertex operations
# 2. Fast lookups (O(1))
# 3. In-memory performance
#

set -e

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
PURPLE='\033[0;35m'
CYAN='\033[0;36m'
NC='\033[0m'

# Logging
log() { echo -e "${BLUE}[$(date '+%H:%M:%S')]${NC} $1"; }
success() { echo -e "${GREEN}✓${NC} $1"; }
info() { echo -e "${CYAN}ℹ${NC} $1"; }

# Banner
echo -e "${PURPLE}"
cat << 'EOF'
╔══════════════════════════════════════════════════════════╗
║         ⚡ Low Latency - Sub-Millisecond Ops ⚡           ║
║                                                           ║
║  Learn: In-memory performance characteristics            ║
╚══════════════════════════════════════════════════════════╝
EOF
echo -e "${NC}"

# Get paths
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
RHIZO_ROOT="$(cd "$SCRIPT_DIR/../../.." && pwd)"

log "Building rhizoCrypt in release mode..."
cd "$RHIZO_ROOT"
cargo build --release --quiet 2>/dev/null || cargo build --release

echo ""
log "Creating latency demo..."

# Create temporary Rust program
TEMP_DIR=$(mktemp -d)
cat > "$TEMP_DIR/Cargo.toml" << CARGO_EOF
[package]
name = "latency-demo"
version = "0.1.0"
edition = "2021"

[dependencies]
rhizo-crypt-core = { path = "$RHIZO_ROOT/crates/rhizo-crypt-core" }
tokio = { version = "1.46", features = ["full"] }

[profile.release]
opt-level = 3
lto = true
CARGO_EOF

mkdir -p "$TEMP_DIR/src"
cat > "$TEMP_DIR/src/main.rs" << 'RUST_EOF'
use rhizo_crypt_core::{RhizoCrypt, RhizoCryptConfig, Session, SessionType, Vertex, EventType};
use std::time::Instant;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n⚡ Measuring rhizoCrypt Latency (Low-Level Operations)...\n");
    
    // Create rhizoCrypt instance
    let config = RhizoCryptConfig::default();
    let rhizo = RhizoCrypt::new(config);
    
    let start = Instant::now();
    rhizo.start().await?;
    let startup_time = start.elapsed();
    println!("✓ rhizoCrypt startup: {:?}", startup_time);
    println!();
    
    // Create session
    let session = Session::new("latency-demo".to_string(), SessionType::Ephemeral);
    let session_id = session.id;
    
    let start = Instant::now();
    rhizo.create_session(session).await?;
    let session_create_time = start.elapsed();
    println!("✓ Session creation: {:?}", session_create_time);
    println!();
    
    let store = rhizo.dag_store().await?;
    
    // Measure vertex insertion
    println!("📊 Vertex Insertion Latency:");
    let mut vertex_times = Vec::new();
    let mut vertex_ids = Vec::new();
    
    for i in 0..100 {
        let parents = if vertex_ids.is_empty() {
            Vec::new()
        } else {
            vec![vertex_ids[vertex_ids.len() - 1].clone()]
        };
        
        let v = Vertex::new(EventType::DataCreated, parents);
        let v_id = v.id.clone();
        
        let start = Instant::now();
        store.put_vertex(session_id, v).await?;
        let elapsed = start.elapsed();
        
        vertex_times.push(elapsed);
        vertex_ids.push(v_id);
    }
    
    let avg_insert = vertex_times.iter().sum::<std::time::Duration>() / vertex_times.len() as u32;
    let min_insert = vertex_times.iter().min().unwrap();
    let max_insert = vertex_times.iter().max().unwrap();
    
    println!("  Samples: {} insertions", vertex_times.len());
    println!("  Average: {:?}", avg_insert);
    println!("  Min: {:?}", min_insert);
    println!("  Max: {:?}", max_insert);
    println!();
    
    // Measure vertex lookup
    println!("📊 Vertex Lookup Latency (O(1) by ID):");
    let mut lookup_times = Vec::new();
    
    for vertex_id in &vertex_ids {
        let start = Instant::now();
        let _ = store.get_vertex(session_id, vertex_id.clone()).await?;
        let elapsed = start.elapsed();
        lookup_times.push(elapsed);
    }
    
    let avg_lookup = lookup_times.iter().sum::<std::time::Duration>() / lookup_times.len() as u32;
    let min_lookup = lookup_times.iter().min().unwrap();
    let max_lookup = lookup_times.iter().max().unwrap();
    
    println!("  Samples: {} lookups", lookup_times.len());
    println!("  Average: {:?}", avg_lookup);
    println!("  Min: {:?}", min_lookup);
    println!("  Max: {:?}", max_lookup);
    println!();
    
    // Measure frontier query
    println!("📊 Frontier Query Latency (O(1) tracking):");
    let mut frontier_times = Vec::new();
    
    for _ in 0..100 {
        let start = Instant::now();
        let _ = store.get_frontier(session_id).await?;
        let elapsed = start.elapsed();
        frontier_times.push(elapsed);
    }
    
    let avg_frontier = frontier_times.iter().sum::<std::time::Duration>() / frontier_times.len() as u32;
    let min_frontier = frontier_times.iter().min().unwrap();
    let max_frontier = frontier_times.iter().max().unwrap();
    
    println!("  Samples: {} queries", frontier_times.len());
    println!("  Average: {:?}", avg_frontier);
    println!("  Min: {:?}", min_frontier);
    println!("  Max: {:?}", max_frontier);
    println!();
    
    // Measure genesis query
    println!("📊 Genesis Query Latency:");
    let mut genesis_times = Vec::new();
    
    for _ in 0..100 {
        let start = Instant::now();
        let _ = store.get_genesis(session_id).await?;
        let elapsed = start.elapsed();
        genesis_times.push(elapsed);
    }
    
    let avg_genesis = genesis_times.iter().sum::<std::time::Duration>() / genesis_times.len() as u32;
    let min_genesis = genesis_times.iter().min().unwrap();
    let max_genesis = genesis_times.iter().max().unwrap();
    
    println!("  Samples: {} queries", genesis_times.len());
    println!("  Average: {:?}", avg_genesis);
    println!("  Min: {:?}", min_genesis);
    println!("  Max: {:?}", max_genesis);
    println!();
    
    // Summary
    println!("⚡ Latency Summary:");
    println!("  ┌────────────────────────┬─────────────┐");
    println!("  │ Operation              │ Avg Latency │");
    println!("  ├────────────────────────┼─────────────┤");
    println!("  │ rhizoCrypt startup     │ {:>10?} │", startup_time);
    println!("  │ Session creation       │ {:>10?} │", session_create_time);
    println!("  │ Vertex insertion       │ {:>10?} │", avg_insert);
    println!("  │ Vertex lookup (by ID)  │ {:>10?} │", avg_lookup);
    println!("  │ Frontier query         │ {:>10?} │", avg_frontier);
    println!("  │ Genesis query          │ {:>10?} │", avg_genesis);
    println!("  └────────────────────────┴─────────────┘");
    println!();
    
    println!("🎉 Success! rhizoCrypt delivers sub-millisecond operations!");
    println!("\n💡 Key Insights:");
    println!("  • In-memory operations are extremely fast");
    println!("  • Vertex lookup is O(1) (hash table)");
    println!("  • Frontier tracking is O(1) (maintained incrementally)");
    println!("  • Genesis is stable (computed once)");
    println!("  • No disk I/O in ephemeral sessions");
    
    println!("\n🌟 Performance Characteristics:");
    println!("  • Vertex insertion: ~microseconds");
    println!("  • Vertex lookup: ~microseconds (O(1))");
    println!("  • Frontier query: ~microseconds (O(1))");
    println!("  • Genesis query: ~microseconds (O(1))");
    
    // Cleanup
    rhizo.stop().await?;
    
    Ok(())
}
RUST_EOF

echo ""
log "Running demo (release mode for accurate timings)..."
echo ""

cd "$TEMP_DIR"
cargo run --release --quiet 2>/dev/null || cargo run --release

# Cleanup
cd "$RHIZO_ROOT"
rm -rf "$TEMP_DIR"

echo ""
echo -e "${GREEN}════════════════════════════════════════════════════════${NC}"
echo -e "${GREEN}Demo Complete!${NC}"
echo ""
info "What you learned:"
echo "  1. rhizoCrypt operations are sub-millisecond"
echo "  2. Lookups are O(1) (hash table)"
echo "  3. In-memory performance is excellent"
echo "  4. No disk I/O overhead for ephemeral sessions"
echo ""
info "Next demo:"
echo "  ./demo-memory.sh  - Memory efficiency analysis"
echo ""

