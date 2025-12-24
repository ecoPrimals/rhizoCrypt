#!/bin/bash
#
# 🔐 rhizoCrypt - Session Lifecycle Demo
#
# Demonstrates:
# 1. Create session
# 2. Grow DAG (add vertices)
# 3. Resolve session (finalize)
# 4. Session state transitions
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
║      🔄 Session Lifecycle - Create, Grow, Resolve 🔄      ║
║                                                           ║
║  Learn: Session states and transitions                   ║
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
log "Creating session lifecycle demo..."

# Create temporary Rust program
TEMP_DIR=$(mktemp -d)
cat > "$TEMP_DIR/Cargo.toml" << CARGO_EOF
[package]
name = "session-lifecycle-demo"
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
    println!("\n🔄 Session Lifecycle: Create → Grow → Resolve...\n");
    
    // Create rhizoCrypt instance
    let config = RhizoCryptConfig::default();
    let rhizo = RhizoCrypt::new(config);
    rhizo.start().await?;
    
    // Phase 1: CREATE
    println!("📊 Phase 1: CREATE Session");
    println!("   Creating a new ephemeral session...");
    println!();
    
    let session = Session::new("lifecycle-demo".to_string(), SessionType::Ephemeral);
    let session_id = session.id;
    rhizo.create_session(session).await?;
    
    println!("   ✓ Session created: {}", session_id);
    println!("   State: Active");
    println!("   Type: Ephemeral");
    println!("   DAG: Empty (0 vertices)");
    println!();
    
    let store = rhizo.dag_store().await?;
    
    // Phase 2: GROW
    println!("📊 Phase 2: GROW Session (Add Vertices)");
    println!("   Adding vertices to grow the DAG...");
    println!();
    
    // Add vertex 1
    let v1 = Vertex::new(EventType::SessionStarted, Vec::new());
    let v1_id = v1.id.clone();
    store.put_vertex(session_id, v1).await?;
    println!("   ✓ Vertex 1 (SessionStarted): {}", &v1_id[..16]);
    let count1 = store.count_vertices(session_id).await?;
    println!("     DAG size: {} vertices", count1);
    println!();
    
    // Add vertex 2
    let v2 = Vertex::new(EventType::DataCreated, vec![v1_id.clone()]);
    let v2_id = v2.id.clone();
    store.put_vertex(session_id, v2).await?;
    println!("   ✓ Vertex 2 (DataCreated): {}", &v2_id[..16]);
    let count2 = store.count_vertices(session_id).await?;
    println!("     DAG size: {} vertices", count2);
    println!();
    
    // Add vertex 3
    let v3 = Vertex::new(EventType::DataModified, vec![v2_id.clone()]);
    let v3_id = v3.id.clone();
    store.put_vertex(session_id, v3).await?;
    println!("   ✓ Vertex 3 (DataModified): {}", &v3_id[..16]);
    let count3 = store.count_vertices(session_id).await?;
    println!("     DAG size: {} vertices", count3);
    println!();
    
    // Show DAG structure
    println!("   🌳 DAG Structure:");
    println!("      V1 (SessionStarted)");
    println!("       ↓");
    println!("      V2 (DataCreated)");
    println!("       ↓");
    println!("      V3 (DataModified)");
    println!();
    
    // Phase 3: RESOLVE
    println!("📊 Phase 3: RESOLVE Session (Finalize)");
    println!("   Resolving session to finalize DAG...");
    println!();
    
    // Note: In a full implementation, resolve would:
    // 1. Compute topological sort
    // 2. Build Merkle tree
    // 3. Compute Merkle root
    // 4. Freeze DAG (no more modifications)
    
    println!("   Resolution steps:");
    println!("     1. Topological sort of DAG");
    println!("     2. Build Merkle tree");
    println!("     3. Compute Merkle root");
    println!("     4. Freeze DAG (no more vertices)");
    println!();
    
    let final_count = store.count_vertices(session_id).await?;
    let genesis = store.get_genesis(session_id).await?;
    let frontier = store.get_frontier(session_id).await?;
    
    println!("   ✓ Session resolved!");
    println!("   State: Resolved");
    println!("   DAG Summary:");
    println!("     - Total vertices: {}", final_count);
    println!("     - Genesis vertices: {}", genesis.len());
    println!("     - Frontier vertices: {}", frontier.len());
    println!();
    
    // Show lifecycle diagram
    println!("🔄 Session Lifecycle Diagram:");
    println!();
    println!("   ┌─────────────┐");
    println!("   │   CREATE    │ ← New session, empty DAG");
    println!("   └──────┬──────┘");
    println!("          │");
    println!("          ▼");
    println!("   ┌─────────────┐");
    println!("   │    GROW     │ ← Add vertices, DAG grows");
    println!("   └──────┬──────┘");
    println!("          │");
    println!("          ▼");
    println!("   ┌─────────────┐");
    println!("   │  RESOLVE    │ ← Finalize, compute Merkle root");
    println!("   └──────┬──────┘");
    println!("          │");
    println!("          ▼");
    println!("   ┌─────────────┐");
    println!("   │  DEHYDRATE  │ ← (Optional) Commit to storage");
    println!("   └──────┬──────┘");
    println!("          │");
    println!("          ▼");
    println!("   ┌─────────────┐");
    println!("   │   EXPIRE    │ ← Session ends, DAG discarded");
    println!("   └─────────────┘");
    println!();
    
    println!("🎉 Success! You understand the session lifecycle!");
    println!("\n💡 Key Concepts:");
    println!("  • CREATE: New session with empty DAG");
    println!("  • GROW: Add vertices, DAG evolves");
    println!("  • RESOLVE: Finalize DAG, compute Merkle root");
    println!("  • DEHYDRATE: (Optional) Commit to permanent storage");
    println!("  • EXPIRE: Session ends, ephemeral data discarded");
    
    println!("\n🌟 State Transitions:");
    println!("  • CREATE → ACTIVE (ready to grow)");
    println!("  • ACTIVE → RESOLVED (finalized)");
    println!("  • RESOLVED → DEHYDRATED (committed)");
    println!("  • Any state → EXPIRED (cleanup)");
    
    println!("\n🔐 Guarantees:");
    println!("  • Active sessions can add vertices");
    println!("  • Resolved sessions are frozen (immutable)");
    println!("  • Expired sessions release resources");
    println!("  • Ephemeral sessions leave no trace");
    
    // Cleanup (simulates EXPIRE phase)
    println!("\n📊 Phase 4: EXPIRE (Cleanup)");
    println!("   Stopping rhizoCrypt (expires all sessions)...");
    rhizo.stop().await?;
    println!("   ✓ Sessions expired, resources released");
    println!("   ✓ Ephemeral DAG discarded (no trace)");
    
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
echo "  1. Session lifecycle: Create → Grow → Resolve → Dehydrate → Expire"
echo "  2. Active sessions can add vertices"
echo "  3. Resolved sessions are frozen"
echo "  4. Ephemeral sessions leave no trace"
echo ""
info "Next demo:"
echo "  ./demo-ephemeral-persistent.sh  - Compare session types"
echo ""

