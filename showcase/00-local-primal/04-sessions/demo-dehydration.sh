#!/bin/bash
#
# 🔐 rhizoCrypt - Dehydration (Commit) Demo
#
# Demonstrates:
# 1. Dehydration concept (commit to permanent storage)
# 2. Commit workflow
# 3. Ephemeral → Permanent transition
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
║    💾 Dehydration - Commit to Permanent Storage 💾        ║
║                                                           ║
║  Learn: Ephemeral → Permanent transition                 ║
╚══════════════════════════════════════════════════════════╝
EOF
echo -e "${NC}"

echo ""
echo -e "${CYAN}Note: This demo is conceptual - actual dehydration requires LoamSpine${NC}"
echo ""

# Get paths
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
RHIZO_ROOT="$(cd "$SCRIPT_DIR/../../.." && pwd)"

log "Building rhizoCrypt..."
cd "$RHIZO_ROOT"
cargo build --quiet 2>/dev/null || cargo build

echo ""
log "Creating dehydration demo..."

# Create temporary Rust program
TEMP_DIR=$(mktemp -d)
cat > "$TEMP_DIR/Cargo.toml" << CARGO_EOF
[package]
name = "dehydration-demo"
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
    println!("\n💾 Dehydration: Commit Ephemeral DAG to Permanent Storage...\n");
    
    // Create rhizoCrypt instance
    let config = RhizoCryptConfig::default();
    let rhizo = RhizoCrypt::new(config);
    rhizo.start().await?;
    
    // Explain dehydration
    println!("📚 What is Dehydration?");
    println!();
    println!("   Dehydration is the process of committing");
    println!("   ephemeral DAG results to permanent storage.");
    println!();
    println!("   Think of it like:");
    println!("     • Git commit: Working directory → Repository");
    println!("     • Database commit: Transaction → Persistent storage");
    println!("     • Save file: RAM → Disk");
    println!();
    
    // Show workflow
    println!("🔄 Dehydration Workflow:");
    println!();
    println!("   Step 1: Session is active (ephemeral DAG growing)");
    println!("      └─ Vertices added to working memory");
    println!();
    println!("   Step 2: Resolve session (finalize DAG)");
    println!("      └─ Compute topological sort");
    println!("      └─ Build Merkle tree");
    println!("      └─ Calculate Merkle root");
    println!();
    println!("   Step 3: Dehydrate (commit to LoamSpine)");
    println!("      └─ Get frontier vertices");
    println!("      └─ Commit to permanent storage");
    println!("      └─ Include Merkle root (integrity)");
    println!("      └─ Include provenance metadata");
    println!();
    println!("   Step 4: LoamSpine returns commit ID");
    println!("      └─ Unique identifier for committed data");
    println!("      └─ Can be used for future slice checkouts");
    println!();
    
    // Simulate dehydration workflow
    let session = Session::new("dehydration-demo".to_string(), SessionType::Persistent);
    let session_id = session.id;
    rhizo.create_session(session).await?;
    
    let store = rhizo.dag_store().await?;
    
    println!("📊 Simulated Dehydration:");
    println!();
    
    // Build a DAG
    println!("   Phase 1: Build ephemeral DAG");
    
    let v1 = Vertex::new(EventType::SessionStarted, Vec::new());
    let v1_id = v1.id.clone();
    store.put_vertex(session_id, v1).await?;
    println!("     ✓ Vertex 1: {}", &v1_id[..16]);
    
    let v2 = Vertex::new(EventType::DataCreated, vec![v1_id.clone()]);
    let v2_id = v2.id.clone();
    store.put_vertex(session_id, v2).await?;
    println!("     ✓ Vertex 2: {}", &v2_id[..16]);
    
    let v3 = Vertex::new(EventType::DataModified, vec![v2_id.clone()]);
    let v3_id = v3.id.clone();
    store.put_vertex(session_id, v3).await?;
    println!("     ✓ Vertex 3: {}", &v3_id[..16]);
    
    let v4 = Vertex::new(EventType::DataCommitted, vec![v3_id.clone()]);
    let v4_id = v4.id.clone();
    store.put_vertex(session_id, v4).await?;
    println!("     ✓ Vertex 4: {}", &v4_id[..16]);
    println!();
    
    let count = store.count_vertices(session_id).await?;
    println!("     DAG: {} vertices (ephemeral)", count);
    println!();
    
    // Resolve
    println!("   Phase 2: Resolve session");
    println!("     • Topological sort: V1 → V2 → V3 → V4");
    println!("     • Build Merkle tree");
    
    let genesis = store.get_genesis(session_id).await?;
    let frontier = store.get_frontier(session_id).await?;
    
    println!("     • Merkle root: abc123def456... (simulated)");
    println!("     • Frontier: {} vertices", frontier.len());
    println!();
    
    // Dehydrate (simulated)
    println!("   Phase 3: Dehydrate to LoamSpine");
    println!("     → Connecting to LoamSpine...");
    println!("     → Committing frontier vertices...");
    println!("     → Including Merkle root...");
    println!("     → Including provenance metadata...");
    println!();
    println!("     ✓ Dehydration complete!");
    println!("     Commit ID: commit_789xyz (simulated)");
    println!();
    
    // Show what gets committed
    println!("   📦 What Gets Committed:");
    println!("     • Frontier vertices ({} vertices)", frontier.len());
    println!("     • Merkle root (integrity proof)");
    println!("     • Session metadata:");
    println!("       - Session ID");
    println!("       - Timestamp");
    println!("       - Total vertex count");
    println!("     • Provenance:");
    println!("       - Source slices (if any)");
    println!("       - Computation lineage");
    println!();
    
    // Show DAG structure
    println!("   🌳 DAG (Ephemeral → Permanent):");
    println!("      V1 (SessionStarted)");
    println!("       ↓");
    println!("      V2 (DataCreated)");
    println!("       ↓");
    println!("      V3 (DataModified)");
    println!("       ↓");
    println!("      V4 (DataCommitted) ← Frontier");
    println!("       ↓");
    println!("      [Dehydrate to LoamSpine]");
    println!("       ↓");
    println!("      Commit ID: commit_789xyz");
    println!();
    
    println!("🎉 Success! You understand dehydration!");
    println!("\n💡 Key Concepts:");
    println!("  • Dehydration = ephemeral → permanent");
    println!("  • Only frontier vertices committed (not full DAG)");
    println!("  • Merkle root provides integrity proof");
    println!("  • Commit ID enables future slice checkouts");
    println!("  • Provenance metadata preserved");
    
    println!("\n🌟 Benefits:");
    println!("  • Explicit: User controls persistence");
    println!("  • Efficient: Only commit what's needed (frontier)");
    println!("  • Verifiable: Merkle root proves integrity");
    println!("  • Traceable: Full provenance preserved");
    println!("  • Composable: Committed data can be sliced later");
    
    println!("\n🔐 Sovereignty & Consent:");
    println!("  • Dehydration requires explicit user action");
    println!("  • Ephemeral by default (no auto-save)");
    println!("  • User chooses what to persist");
    println!("  • No surveillance (ephemeral discarded)");
    println!("  • Audit trail only when needed");
    
    println!("\n📖 Real-World Example:");
    println!("   // Build ephemeral DAG");
    println!("   let session = Session::new(\"my-session\", Persistent);");
    println!("   // ... add vertices ...");
    println!();
    println!("   // Resolve session (finalize)");
    println!("   session.resolve().await?;");
    println!();
    println!("   // Dehydrate to permanent storage");
    println!("   let commit_id = session.dehydrate().await?;");
    println!();
    println!("   // Later: Checkout as slice in new session");
    println!("   let new_session = Session::new(\"next-session\", Ephemeral);");
    println!("   new_session.checkout_slice(commit_id).await?;");
    println!();
    
    println!("🔗 Full Lifecycle:");
    println!();
    println!("   Session 1:");
    println!("     CREATE → GROW → RESOLVE → DEHYDRATE");
    println!("                                   ↓");
    println!("                         LoamSpine (commit_123)");
    println!("                                   ↓");
    println!("   Session 2:");
    println!("     CREATE → SLICE(commit_123) → GROW → DEHYDRATE");
    println!("                                             ↓");
    println!("                               LoamSpine (commit_456)");
    println!();
    println!("   → Full lineage preserved!");
    
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
echo "  1. Dehydration = ephemeral → permanent"
echo "  2. Only frontier vertices committed"
echo "  3. Merkle root provides integrity"
echo "  4. Explicit user consent required"
echo ""
info "Level 4 Complete! Congratulations!"
echo ""
info "You've completed all 6 levels of the rhizoCrypt showcase!"
echo ""
info "Next steps:"
echo "  • Run the full automated tour: cd ../..; ./RUN_ME_FIRST.sh"
echo "  • Explore inter-primal demos: cd ../../01-inter-primal"
echo "  • Build your own rhizoCrypt application!"
echo ""

