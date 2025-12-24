#!/bin/bash
#
# 🔐 rhizoCrypt - Multi-Session Workflow Demo
#
# Demonstrates:
# 1. Creating multiple sessions
# 2. Session isolation
# 3. Cross-session data flow (conceptual - requires LoamSpine)
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
║      🔄 Multi-Session Workflow - Isolation & Flow 🔄      ║
║                                                           ║
║  Learn: Session isolation and cross-session data flow    ║
╚══════════════════════════════════════════════════════════╝
EOF
echo -e "${NC}"

# Get paths
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
RHIZO_ROOT="$(cd "$SCRIPT_DIR/../../.." && pwd)"

log "Building rhizoCrypt..."
cd "$RHIZO_ROOT"
cargo build --quiet 2>/dev/null || cargo build

echo ""
log "Creating multi-session demo..."

# Create temporary Rust program
TEMP_DIR=$(mktemp -d)
cat > "$TEMP_DIR/Cargo.toml" << CARGO_EOF
[package]
name = "multi-session-demo"
version = "0.1.0"
edition = "2021"

[dependencies]
rhizo-crypt-core = { path = "$RHIZO_ROOT/crates/rhizo-crypt-core" }
tokio = { version = "1.46", features = ["full"] }
CARGO_EOF

mkdir -p "$TEMP_DIR/src"
cat > "$TEMP_DIR/src/main.rs" << 'RUST_EOF'
use rhizo_crypt_core::{RhizoCrypt, RhizoCryptConfig, Session, SessionType, Vertex, EventType};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n🔄 Multi-Session Workflow: Isolation & Composition...\n");
    
    // Create rhizoCrypt instance
    let config = RhizoCryptConfig::default();
    let rhizo = RhizoCrypt::new(config);
    rhizo.start().await?;
    
    // Demo 1: Session Isolation
    println!("📊 Demo 1: Session Isolation");
    println!("Creating two independent sessions...");
    println!();
    
    let session1 = Session::new("extraction".to_string(), SessionType::Ephemeral);
    let session1_id = session1.id;
    rhizo.create_session(session1).await?;
    println!("✓ Session 1 (extraction): {}", session1_id);
    
    let session2 = Session::new("transformation".to_string(), SessionType::Ephemeral);
    let session2_id = session2.id;
    rhizo.create_session(session2).await?;
    println!("✓ Session 2 (transformation): {}", session2_id);
    println!();
    
    let store = rhizo.dag_store().await?;
    
    // Add vertices to Session 1
    println!("Adding vertices to Session 1:");
    let v1 = Vertex::new(EventType::DataCreated, Vec::new());
    let v1_id = v1.id.clone();
    store.put_vertex(session1_id, v1).await?;
    println!("  ✓ Vertex A: {}", &v1_id[..16]);
    
    let v2 = Vertex::new(EventType::DataModified, vec![v1_id.clone()]);
    let v2_id = v2.id.clone();
    store.put_vertex(session1_id, v2).await?;
    println!("  ✓ Vertex B: {}", &v2_id[..16]);
    
    // Add vertices to Session 2
    println!("\nAdding vertices to Session 2:");
    let v3 = Vertex::new(EventType::DataCreated, Vec::new());
    let v3_id = v3.id.clone();
    store.put_vertex(session2_id, v3).await?;
    println!("  ✓ Vertex X: {}", &v3_id[..16]);
    
    let v4 = Vertex::new(EventType::DataModified, vec![v3_id.clone()]);
    let v4_id = v4.id.clone();
    store.put_vertex(session2_id, v4).await?;
    println!("  ✓ Vertex Y: {}", &v4_id[..16]);
    
    println!();
    
    // Verify isolation
    println!("🔍 Verifying Isolation:");
    let count1 = store.count_vertices(session1_id).await?;
    let count2 = store.count_vertices(session2_id).await?;
    
    println!("  Session 1 vertices: {}", count1);
    println!("  Session 2 vertices: {}", count2);
    println!();
    
    if count1 == 2 && count2 == 2 {
        println!("✅ SUCCESS: Sessions are isolated!");
        println!("   Each session has its own DAG");
        println!("   No cross-contamination");
    }
    println!();
    
    // Demo 2: Cross-Session Data Flow (Conceptual)
    println!("📊 Demo 2: Cross-Session Data Flow (Conceptual)");
    println!();
    println!("In a real workflow with LoamSpine:");
    println!();
    println!("  Step 1: Session 1 (Extract)");
    println!("    - Create extraction session");
    println!("    - Build DAG with extracted data");
    println!("    - Dehydrate to LoamSpine");
    println!("    - Get commit ID");
    println!();
    println!("  Step 2: Session 2 (Transform)");
    println!("    - Create transformation session");
    println!("    - Checkout slice from LoamSpine (commit ID)");
    println!("    - Apply transformations (new DAG)");
    println!("    - Dehydrate results to LoamSpine");
    println!();
    println!("  Step 3: Session 3 (Load)");
    println!("    - Create load session");
    println!("    - Checkout slice from LoamSpine");
    println!("    - Load to destination");
    println!("    - Dehydrate completion status");
    println!();
    
    // Visualize workflow
    println!("🌳 Workflow Visualization:");
    println!();
    println!("   Session 1 (Extract)");
    println!("        ↓");
    println!("   [Dehydrate]");
    println!("        ↓");
    println!("   LoamSpine (Permanent Storage)");
    println!("        ↓");
    println!("   [Slice Checkout]");
    println!("        ↓");
    println!("   Session 2 (Transform)");
    println!("        ↓");
    println!("   [Dehydrate]");
    println!("        ↓");
    println!("   LoamSpine");
    println!("        ↓");
    println!("   [Slice Checkout]");
    println!("        ↓");
    println!("   Session 3 (Load)");
    println!();
    
    println!("🎉 Success! You understand multi-session workflows!");
    println!("\n💡 Key Concepts:");
    println!("  • Sessions are isolated (independent DAGs)");
    println!("  • Data flows through permanent storage (LoamSpine)");
    println!("  • Dehydration = commit session results");
    println!("  • Slice = checkout permanent data into new session");
    println!("  • Each session has independent lifecycle");
    
    println!("\n🌟 Use Cases:");
    println!("  • ETL Pipelines: Extract → Transform → Load");
    println!("  • Multi-Stage Computation: Stage 1 → Stage 2 → Stage 3");
    println!("  • Parallel Processing: Fan-out → Compute → Fan-in");
    println!("  • Agent Collaboration: Agent 1 → Agent 2 → Agent 3");
    
    println!("\n🔐 Benefits:");
    println!("  • Fault Isolation: Session failure doesn't affect others");
    println!("  • Checkpointing: Dehydrate at each stage");
    println!("  • Retry: Restart from last checkpoint");
    println!("  • Provenance: Full lineage tracked in DAGs");
    println!("  • Parallelism: Independent sessions can run concurrently");
    
    // Cleanup
    rhizo.stop().await?;
    
    Ok(())
}
RUST_EOF

echo ""
log "Running demo..."
echo ""

cd "$TEMP_DIR"
cargo run --quiet 2>/dev/null || cargo run

# Cleanup
cd "$RHIZO_ROOT"
rm -rf "$TEMP_DIR"

echo ""
echo -e "${GREEN}════════════════════════════════════════════════════════${NC}"
echo -e "${GREEN}Demo Complete!${NC}"
echo ""
info "What you learned:"
echo "  1. Sessions are isolated (independent DAGs)"
echo "  2. Data flows through permanent storage"
echo "  3. Dehydration commits session results"
echo "  4. Slices checkout permanent data"
echo ""
info "Next demo:"
echo "  ./demo-event-sourcing.sh  - Learn about event sourcing patterns"
echo ""

