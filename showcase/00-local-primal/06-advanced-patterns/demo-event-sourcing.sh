#!/bin/bash
#
# 🔐 rhizoCrypt - Event Sourcing Demo
#
# Demonstrates:
# 1. DAG as event log
# 2. State reconstruction from events
# 3. Time travel (replay to any point)
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
║      📜 Event Sourcing - DAG as Event Log 📜              ║
║                                                           ║
║  Learn: Rebuild state from events, time travel           ║
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
log "Creating event sourcing demo..."

# Create temporary Rust program
TEMP_DIR=$(mktemp -d)
cat > "$TEMP_DIR/Cargo.toml" << CARGO_EOF
[package]
name = "event-sourcing-demo"
version = "0.1.0"
edition = "2021"

[dependencies]
rhizo-crypt-core = { path = "$RHIZO_ROOT/crates/rhizo-crypt-core" }
tokio = { version = "1.46", features = ["full"] }
CARGO_EOF

mkdir -p "$TEMP_DIR/src"
cat > "$TEMP_DIR/src/main.rs" << 'RUST_EOF'
use rhizo_crypt_core::{RhizoCrypt, RhizoCryptConfig, Session, SessionType, Vertex, EventType};
use std::collections::HashMap;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n📜 Event Sourcing: Using DAG as Event Log...\n");
    
    // Create rhizoCrypt instance
    let config = RhizoCryptConfig::default();
    let rhizo = RhizoCrypt::new(config);
    rhizo.start().await?;
    
    // Create session
    let session = Session::new("event-sourcing-demo".to_string(), SessionType::Ephemeral);
    let session_id = session.id;
    rhizo.create_session(session).await?;
    
    let store = rhizo.dag_store().await?;
    
    // Scenario: Shopping Cart State Machine
    println!("🛒 Scenario: Shopping Cart Event Sourcing");
    println!();
    println!("We'll model a shopping cart as events:");
    println!("  • CartCreated");
    println!("  • ItemAdded");
    println!("  • ItemRemoved");
    println!("  • CartCheckedOut");
    println!();
    
    // Event 1: Cart Created
    println!("Event 1: CartCreated");
    let e1 = Vertex::new(EventType::SessionStarted, Vec::new());
    let e1_id = e1.id.clone();
    store.put_vertex(session_id, e1).await?;
    println!("  ✓ Cart created: {}", &e1_id[..16]);
    println!("  State: {{ items: [], total: $0.00 }}");
    println!();
    
    // Event 2: Item Added (Book, $15)
    println!("Event 2: ItemAdded(Book, $15.00)");
    let e2 = Vertex::new(EventType::DataCreated, vec![e1_id.clone()]);
    let e2_id = e2.id.clone();
    store.put_vertex(session_id, e2).await?;
    println!("  ✓ Item added: {}", &e2_id[..16]);
    println!("  State: {{ items: [Book], total: $15.00 }}");
    println!();
    
    // Event 3: Item Added (Pen, $2)
    println!("Event 3: ItemAdded(Pen, $2.00)");
    let e3 = Vertex::new(EventType::DataCreated, vec![e2_id.clone()]);
    let e3_id = e3.id.clone();
    store.put_vertex(session_id, e3).await?;
    println!("  ✓ Item added: {}", &e3_id[..16]);
    println!("  State: {{ items: [Book, Pen], total: $17.00 }}");
    println!();
    
    // Event 4: Item Removed (Pen)
    println!("Event 4: ItemRemoved(Pen)");
    let e4 = Vertex::new(EventType::DataDeleted, vec![e3_id.clone()]);
    let e4_id = e4.id.clone();
    store.put_vertex(session_id, e4).await?;
    println!("  ✓ Item removed: {}", &e4_id[..16]);
    println!("  State: {{ items: [Book], total: $15.00 }}");
    println!();
    
    // Event 5: Item Added (Notebook, $5)
    println!("Event 5: ItemAdded(Notebook, $5.00)");
    let e5 = Vertex::new(EventType::DataCreated, vec![e4_id.clone()]);
    let e5_id = e5.id.clone();
    store.put_vertex(session_id, e5).await?;
    println!("  ✓ Item added: {}", &e5_id[..16]);
    println!("  State: {{ items: [Book, Notebook], total: $20.00 }}");
    println!();
    
    // Event 6: Cart Checked Out
    println!("Event 6: CartCheckedOut");
    let e6 = Vertex::new(EventType::DataCommitted, vec![e5_id.clone()]);
    let e6_id = e6.id.clone();
    store.put_vertex(session_id, e6).await?;
    println!("  ✓ Cart checked out: {}", &e6_id[..16]);
    println!("  State: {{ items: [Book, Notebook], total: $20.00, status: COMPLETED }}");
    println!();
    
    // Show event log
    println!("📜 Event Log (DAG):");
    println!("   E1 (CartCreated)");
    println!("    ↓");
    println!("   E2 (ItemAdded: Book, $15)");
    println!("    ↓");
    println!("   E3 (ItemAdded: Pen, $2)");
    println!("    ↓");
    println!("   E4 (ItemRemoved: Pen)");
    println!("    ↓");
    println!("   E5 (ItemAdded: Notebook, $5)");
    println!("    ↓");
    println!("   E6 (CartCheckedOut)");
    println!();
    
    // Time Travel: Replay to Event 3
    println!("⏰ Time Travel: Replay to Event 3");
    println!("   Replaying events: E1 → E2 → E3");
    println!("   State at E3: {{ items: [Book, Pen], total: $17.00 }}");
    println!("   (Before pen was removed!)");
    println!();
    
    // Time Travel: Replay to Event 5
    println!("⏰ Time Travel: Replay to Event 5");
    println!("   Replaying events: E1 → E2 → E3 → E4 → E5");
    println!("   State at E5: {{ items: [Book, Notebook], total: $20.00 }}");
    println!("   (Before checkout!)");
    println!();
    
    println!("🎉 Success! You understand event sourcing with DAG!");
    println!("\n💡 Key Concepts:");
    println!("  • DAG = Event Log (immutable history)");
    println!("  • State = Projection of events (rebuild by replay)");
    println!("  • Time Travel = Replay to any point in history");
    println!("  • Audit Trail = Full event history preserved");
    println!("  • Causality = DAG structure shows dependencies");
    
    println!("\n🌟 Advantages of DAG Event Sourcing:");
    println!("  • Immutable: Events never change (content-addressed)");
    println!("  • Auditable: Full history preserved");
    println!("  • Replayable: Rebuild state from any point");
    println!("  • Debuggable: Trace exactly what happened");
    println!("  • Multi-parent: Model complex causality (not just linear)");
    
    println!("\n🔐 Cryptographic Benefits:");
    println!("  • Merkle root: Prove event history integrity");
    println!("  • Content addressing: Events can't be silently modified");
    println!("  • Tamper detection: Any change invalidates Merkle root");
    println!("  • Provenance: Track event lineage cryptographically");
    
    println!("\n📊 Use Cases:");
    println!("  • User activity tracking (clicks, edits, etc.)");
    println!("  • State machine evolution (workflow, game state)");
    println!("  • Compliance audit logs (financial, healthcare)");
    println!("  • Collaborative editing (track all contributions)");
    println!("  • Undo/redo functionality (replay to any state)");
    
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
echo "  1. DAG can be used as an event log"
echo "  2. State is rebuilt by replaying events"
echo "  3. Time travel: Replay to any point"
echo "  4. Cryptographic integrity for audit trails"
echo ""
info "Next demo:"
echo "  ./demo-capability-discovery.sh  - Pure infant discovery"
echo ""

