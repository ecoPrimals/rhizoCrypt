#!/bin/bash
#
# 🔐 rhizoCrypt - Slices (Checkout) Demo
#
# Demonstrates:
# 1. Slice concept (snapshot from permanent storage)
# 2. Checkout workflow
# 3. Working over immutable data
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
║        🗂️  Slices - Checkout from Storage 🗂️             ║
║                                                           ║
║  Learn: Working over immutable permanent data            ║
╚══════════════════════════════════════════════════════════╝
EOF
echo -e "${NC}"

echo ""
echo -e "${CYAN}Note: This demo is conceptual - actual slices require LoamSpine${NC}"
echo ""

# Get paths
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
RHIZO_ROOT="$(cd "$SCRIPT_DIR/../../.." && pwd)"

log "Building rhizoCrypt..."
cd "$RHIZO_ROOT"
cargo build --quiet 2>/dev/null || cargo build

echo ""
log "Creating slices demo..."

# Create temporary Rust program
TEMP_DIR=$(mktemp -d)
cat > "$TEMP_DIR/Cargo.toml" << CARGO_EOF
[package]
name = "slices-demo"
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
    println!("\n🗂️  Slices: Checkout from Permanent Storage...\n");
    
    // Create rhizoCrypt instance
    let config = RhizoCryptConfig::default();
    let rhizo = RhizoCrypt::new(config);
    rhizo.start().await?;
    
    // Explain the slice concept
    println!("📚 What is a Slice?");
    println!();
    println!("   A slice is a checkout of permanent data into");
    println!("   rhizoCrypt's working memory (DAG).");
    println!();
    println!("   Think of it like:");
    println!("     • Git checkout: Bring code into working directory");
    println!("     • Database view: Read-only snapshot");
    println!("     • Memory map: Access without copying");
    println!();
    
    // Show workflow
    println!("🔄 Slice Workflow:");
    println!();
    println!("   Step 1: Data exists in LoamSpine (permanent storage)");
    println!("      └─ Committed from previous session");
    println!("      └─ Identified by commit ID");
    println!();
    println!("   Step 2: Create new session");
    println!("      └─ New ephemeral working memory");
    println!();
    println!("   Step 3: Checkout slice");
    println!("      └─ session.checkout_slice(commit_id)");
    println!("      └─ Creates genesis vertex in DAG");
    println!("      └─ References permanent data");
    println!();
    println!("   Step 4: Work over slice");
    println!("      └─ Read immutable data");
    println!("      └─ Create new computed vertices");
    println!("      └─ Build DAG on top of slice");
    println!();
    println!("   Step 5: Dehydrate results");
    println!("      └─ Commit new data to LoamSpine");
    println!("      └─ Links back to source slice");
    println!();
    
    // Simulate slice checkout
    let session = Session::new("slice-demo".to_string(), SessionType::Ephemeral);
    let session_id = session.id;
    rhizo.create_session(session).await?;
    
    let store = rhizo.dag_store().await?;
    
    println!("📊 Simulated Slice Checkout:");
    println!();
    
    // In a real implementation, this would:
    // 1. Query LoamSpine for commit data
    // 2. Create genesis vertex referencing slice
    // 3. Lazily load data on access
    
    let slice_vertex = Vertex::new(EventType::SliceCheckout, Vec::new());
    let slice_id = slice_vertex.id.clone();
    store.put_vertex(session_id, slice_vertex).await?;
    
    println!("   ✓ Slice checked out: {}", &slice_id[..16]);
    println!("   Type: SliceCheckout (genesis vertex)");
    println!("   Source: LoamSpine commit abc123...");
    println!();
    
    // Work over slice
    println!("   Working over slice:");
    
    let compute1 = Vertex::new(EventType::DataCreated, vec![slice_id.clone()]);
    let compute1_id = compute1.id.clone();
    store.put_vertex(session_id, compute1).await?;
    println!("     ✓ Computation 1: {}", &compute1_id[..16]);
    
    let compute2 = Vertex::new(EventType::DataModified, vec![compute1_id.clone()]);
    let compute2_id = compute2.id.clone();
    store.put_vertex(session_id, compute2).await?;
    println!("     ✓ Computation 2: {}", &compute2_id[..16]);
    
    let result = Vertex::new(EventType::DataCommitted, vec![compute2_id]);
    let result_id = result.id.clone();
    store.put_vertex(session_id, result).await?;
    println!("     ✓ Result: {}", &result_id[..16]);
    println!();
    
    // Show DAG structure
    println!("   🌳 DAG Structure:");
    println!("      [Slice] (from LoamSpine)");
    println!("         ↓");
    println!("      Compute 1");
    println!("         ↓");
    println!("      Compute 2");
    println!("         ↓");
    println!("      Result");
    println!();
    
    println!("🎉 Success! You understand slices!");
    println!("\n💡 Key Concepts:");
    println!("  • Slice = checkout from permanent storage");
    println!("  • Immutable snapshot (read-only)");
    println!("  • Genesis vertex in DAG");
    println!("  • Work over permanent data without copying");
    println!("  • Dehydrate results back to storage");
    
    println!("\n🌟 Benefits:");
    println!("  • Efficient: No full data copy");
    println!("  • Immutable: Source data can't be modified");
    println!("  • Composable: Multiple sessions can checkout same slice");
    println!("  • Traceable: DAG links to source commit");
    println!("  • Scalable: Work over large datasets");
    
    println!("\n📖 Real-World Example:");
    println!("   // Checkout data from LoamSpine");
    println!("   let slice_id = session.checkout_slice(commit_id).await?;");
    println!();
    println!("   // Read slice data (lazily loaded)");
    println!("   let data = slice.get_data(key).await?;");
    println!();
    println!("   // Compute over slice");
    println!("   let result = process(data);");
    println!();
    println!("   // Dehydrate result");
    println!("   let new_commit = session.dehydrate().await?;");
    println!();
    
    println!("🔗 Slice Provenance:");
    println!("   Every slice tracks its source:");
    println!("     • Source commit ID");
    println!("     • Checkout timestamp");
    println!("     • Read-only access");
    println!("   → Full lineage preserved");
    
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
echo "  1. Slice = checkout from permanent storage"
echo "  2. Immutable snapshot (read-only)"
echo "  3. Work over permanent data efficiently"
echo "  4. Provenance tracking"
echo ""
info "Next demo:"
echo "  ./demo-dehydration.sh  - Commit results to storage"
echo ""

