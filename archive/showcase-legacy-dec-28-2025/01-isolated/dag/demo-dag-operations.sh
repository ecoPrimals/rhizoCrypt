#!/bin/bash
#
# 🔐 rhizoCrypt DAG Operations Demo
#
# Demonstrates advanced DAG operations:
# 1. Create multi-parent vertices
# 2. Query vertices and sessions
# 3. Compute Merkle root
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

log() { echo -e "${BLUE}[$(date '+%H:%M:%S')]${NC} $1"; }
success() { echo -e "${GREEN}✓${NC} $1"; }

echo -e "${PURPLE}"
cat << 'EOF'
╔═══════════════════════════════════════════════════════════════╗
║             🔐 rhizoCrypt DAG Operations Demo                  ║
║                                                                ║
║  Demonstrates: Multi-parent • Content-Addressing • Merkle      ║
╚═══════════════════════════════════════════════════════════════╝
EOF
echo -e "${NC}"

log "Checking prerequisites..."

if ! command -v cargo &> /dev/null; then
    echo -e "${RED}Error: cargo not found.${NC}"
    exit 1
fi

# Get the path to rhizoCrypt BEFORE changing directories
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
RHIZO_PATH="$(cd "$SCRIPT_DIR/../../.." && pwd)"

DEMO_DIR="/tmp/rhizocrypt-dag-demo"
rm -rf "$DEMO_DIR"
mkdir -p "$DEMO_DIR"
cd "$DEMO_DIR"

log "Demo directory: $DEMO_DIR"

cat > Cargo.toml << EOF
[package]
name = "dag-demo"
version = "0.1.0"
edition = "2021"

[dependencies]
rhizo-crypt-core = { path = "$RHIZO_PATH/crates/rhizo-crypt-core" }
tokio = { version = "1.0", features = ["full"] }
EOF

mkdir -p src

cat > src/main.rs << 'EOF'
//! DAG Operations Demo
//!
//! Shows multi-parent DAG and content addressing.

use rhizo_crypt_core::{
    EventType, PrimalLifecycle, RhizoCrypt, RhizoCryptConfig,
    SessionBuilder, SessionType, VertexBuilder,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n🔐 rhizoCrypt DAG Operations Demo\n");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");

    // Initialize
    let config = RhizoCryptConfig::default();
    let mut primal = RhizoCrypt::new(config);
    primal.start().await?;

    // Create session
    let session = SessionBuilder::new(SessionType::General)
        .with_name("DAG Demo")
        .build();
    let session_id = primal.create_session(session).await?;
    println!("📦 Session created: {}\n", session_id);

    // Build a complex DAG structure
    println!("🌳 Building complex DAG structure...\n");

    /*
       We'll build this DAG:
       
           [Genesis/v1]
              / \
           [v2]  [v3]
              \  /  \
              [v4]  [v5]
                \   /
                [v6]   <- Multi-parent merge!
    */

    // Genesis vertex
    let genesis = VertexBuilder::new(EventType::SessionStart)
        .with_metadata("stage", "genesis")
        .build();
    let v1 = primal.append_vertex(session_id, genesis).await?;
    println!("   v1 (genesis): {}", &v1.to_string()[..16]);

    // Two branches from genesis
    let branch_a = VertexBuilder::new(EventType::DataCreate { schema: None })
        .with_parent(v1)
        .with_metadata("branch", "A")
        .build();
    let v2 = primal.append_vertex(session_id, branch_a).await?;
    println!("   v2 (branch A): {}", &v2.to_string()[..16]);

    let branch_b = VertexBuilder::new(EventType::DataCreate { schema: None })
        .with_parent(v1)
        .with_metadata("branch", "B")
        .build();
    let v3 = primal.append_vertex(session_id, branch_b).await?;
    println!("   v3 (branch B): {}", &v3.to_string()[..16]);

    // Merge v2 and v3 into v4 (multi-parent!)
    let merge = VertexBuilder::new(EventType::DataModify { 
        delta_type: "merge".to_string() 
    })
        .with_parent(v2)
        .with_parent(v3)  // Multi-parent!
        .with_metadata("action", "merge")
        .build();
    let v4 = primal.append_vertex(session_id, merge).await?;
    println!("   v4 (merge): {} [2 parents]", &v4.to_string()[..16]);

    // Continue from v3
    let cont = VertexBuilder::new(EventType::DataCreate { schema: None })
        .with_parent(v3)
        .with_metadata("action", "continue")
        .build();
    let v5 = primal.append_vertex(session_id, cont).await?;
    println!("   v5 (continue): {}", &v5.to_string()[..16]);

    // Final merge v4 and v5 (multi-parent!)
    let final_merge = VertexBuilder::new(EventType::AgentAction { 
        action: "finalize".to_string() 
    })
        .with_parent(v4)
        .with_parent(v5)  // Multi-parent!
        .with_metadata("stage", "final")
        .build();
    let v6 = primal.append_vertex(session_id, final_merge).await?;
    println!("   v6 (final): {} [2 parents]", &v6.to_string()[..16]);

    println!();

    // Query operations
    println!("🔍 Querying the DAG...\n");

    // Get session info
    let session_info = primal.get_session(session_id).await?;
    println!("   Total vertices: {}", session_info.vertex_count);
    println!("   Session state: {:?}", session_info.state);

    println!();

    // Verify multi-parent vertices
    println!("🔗 Verifying multi-parent vertices...\n");

    let v4_data = primal.get_vertex(session_id, v4).await?;
    println!("   v4 has {} parents:", v4_data.parents.len());
    for p in &v4_data.parents {
        println!("     • {}", &p.to_string()[..16]);
    }

    let v6_data = primal.get_vertex(session_id, v6).await?;
    println!("   v6 has {} parents:", v6_data.parents.len());
    for p in &v6_data.parents {
        println!("     • {}", &p.to_string()[..16]);
    }

    println!();

    // Content addressing demonstration
    println!("🔑 Content Addressing Demo...\n");
    
    // Same content = same ID
    let test1 = VertexBuilder::new(EventType::DataCreate { schema: None })
        .with_metadata("key", "value")
        .build();
    let test2 = VertexBuilder::new(EventType::DataCreate { schema: None })
        .with_metadata("key", "value")
        .build();
    
    let id1 = test1.compute_id();
    let id2 = test2.compute_id();
    
    println!("   Vertex 1 ID: {}", &id1.to_string()[..16]);
    println!("   Vertex 2 ID: {}", &id2.to_string()[..16]);
    println!("   Same content → Same ID: {}", id1 == id2);

    println!();

    // DAG visualization
    println!("📊 DAG Structure:\n");
    println!("          [v1 genesis]");
    println!("             /    \\");
    println!("          [v2]    [v3]");
    println!("            \\    /   \\");
    println!("            [v4]     [v5]");
    println!("              \\      /");
    println!("              [v6 final]");
    println!();

    // Merkle root
    let root = primal.compute_merkle_root(session_id).await?;
    println!("🌳 Merkle Root: {}", root);

    // Cleanup
    primal.discard_session(session_id).await?;
    primal.stop().await?;

    println!();
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("✅ DAG Demo completed!");
    println!();
    println!("Key Takeaways:");
    println!("  • DAG supports multiple parents (branching & merging)");
    println!("  • Vertices are content-addressed (Blake3)");
    println!("  • Same content produces same ID");
    println!("  • Merkle root proves all vertices");
    println!();

    Ok(())
}
EOF

log "Building demo..."
cargo build --release 2>&1 | grep -E "(Compiling|Finished|error)" || true

log "Running DAG operations demo..."
echo ""
cargo run --release 2>&1

rm -rf "$DEMO_DIR"
success "Demo complete!"
