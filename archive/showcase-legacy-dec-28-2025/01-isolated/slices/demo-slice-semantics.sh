#!/bin/bash
#
# 🔐 rhizoCrypt Slice Semantics Demo
#
# Demonstrates slice modes and checkout workflows.
#

set -e

# Colors
BLUE='\033[0;34m'
GREEN='\033[0;32m'
PURPLE='\033[0;35m'
NC='\033[0m'

log() { echo -e "${BLUE}[$(date '+%H:%M:%S')]${NC} $1"; }
success() { echo -e "${GREEN}✓${NC} $1"; }

echo -e "${PURPLE}"
cat << 'EOF'
╔═══════════════════════════════════════════════════════════════╗
║            🔐 rhizoCrypt Slice Semantics Demo                  ║
║                                                                ║
║  Demonstrates: Slice Modes • Checkouts • Rehydration           ║
╚═══════════════════════════════════════════════════════════════╝
EOF
echo -e "${NC}"

# Get the path to rhizoCrypt BEFORE changing directories
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
RHIZO_PATH="$(cd "$SCRIPT_DIR/../../.." && pwd)"

DEMO_DIR="/tmp/rhizocrypt-slice-demo"
rm -rf "$DEMO_DIR"
mkdir -p "$DEMO_DIR"
cd "$DEMO_DIR"

cat > Cargo.toml << EOF
[package]
name = "slice-demo"
version = "0.1.0"
edition = "2021"

[dependencies]
rhizo-crypt-core = { path = "$RHIZO_PATH/crates/rhizo-crypt-core" }
tokio = { version = "1.0", features = ["full"] }
EOF

mkdir -p src

cat > src/main.rs << 'EOF'
//! Slice Semantics Demo
//!
//! Shows slice modes and checkout workflows.

use rhizo_crypt_core::{
    EventType, PrimalLifecycle, RhizoCrypt, RhizoCryptConfig,
    SessionBuilder, SessionType, VertexBuilder,
    slice::SliceMode,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n🔐 rhizoCrypt Slice Semantics Demo\n");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");

    // Initialize
    let config = RhizoCryptConfig::default();
    let mut primal = RhizoCrypt::new(config);
    primal.start().await?;

    // Create a base session
    println!("📦 Creating base session...\n");
    let session = SessionBuilder::new(SessionType::General)
        .with_name("Base Session")
        .build();
    let session_id = primal.create_session(session).await?;

    // Add some vertices
    let genesis = VertexBuilder::new(EventType::SessionStart).build();
    let v1 = primal.append_vertex(session_id, genesis).await?;
    
    for i in 1..5 {
        let v = VertexBuilder::new(EventType::DataCreate { schema: None })
            .with_parent(v1)
            .with_metadata("index", format!("{}", i))
            .build();
        primal.append_vertex(session_id, v).await?;
    }

    let info = primal.get_session(session_id).await?;
    println!("   Session: {} ({})", session_id, info.name.as_deref().unwrap_or("unnamed"));
    println!("   Vertices: {}\n", info.vertex_count);

    // Demonstrate slice modes
    println!("🗂️  Slice Modes:\n");

    println!("   1️⃣  SliceMode::Copy {{ allow_recopy }}");
    println!("      • Creates an independent copy of the session");
    println!("      • Changes don't affect the original");
    println!("      • Use case: Give a game to play locally\n");

    let copy_mode = SliceMode::Copy { allow_recopy: false };
    println!("      Example: {:?}", copy_mode);
    println!("      allow_recopy: false → Can't be further copied\n");

    println!("   2️⃣  SliceMode::Loan {{ terms, allow_subloan }}");
    println!("      • Temporary use rights with conditions");
    println!("      • Auto-returns on expiry or condition");
    println!("      • Use case: Lend game to friend for weekend\n");

    // Note: LoanTerms would require more setup, showing concept only
    println!("      (Loan mode includes duration, conditions, etc.)\n");

    println!("   3️⃣  SliceMode::Consignment {{ consignee }}");
    println!("      • Held by third party (not owner)");
    println!("      • Escrow-like arrangements");
    println!("      • Use case: Marketplace holding before sale\n");

    // Demonstrate checkout concept
    println!("📥 Checkout Workflow:\n");
    println!("   ┌──────────────┐");
    println!("   │  LoamSpine   │ ← Permanent storage");
    println!("   │   (Commit)   │");
    println!("   └──────┬───────┘");
    println!("          │ checkout");
    println!("          ▼");
    println!("   ┌──────────────┐");
    println!("   │  rhizoCrypt  │ ← Ephemeral session");
    println!("   │   (Slice)    │");
    println!("   └──────────────┘\n");

    // Simulate checkout flow
    println!("   Checkout steps:");
    println!("   1. Request commit from LoamSpine");
    println!("   2. Specify slice mode");
    println!("   3. Receive session data");
    println!("   4. Rehydrate in rhizoCrypt");
    println!("   5. Work with session\n");

    // Demonstrate rehydration concept
    println!("💧 Rehydration:\n");
    println!("   Dehydration (commit):  Session → Merkle Root → LoamSpine");
    println!("   Rehydration (checkout): LoamSpine → Slice → Session\n");

    // Show slice properties
    println!("📊 Slice Mode Properties:\n");
    println!("   ┌──────────────────────────────────────────────────────┐");
    println!("   │ Property          │ Copy  │ Loan  │ Consignment      │");
    println!("   ├───────────────────┼───────┼───────┼──────────────────┤");
    println!("   │ Local Use         │  ✓    │  ✓    │  ✗              │");
    println!("   │ Network Effects   │  ✗    │  ✓    │  ✓              │");
    println!("   │ Time Limited      │  ✗    │  ✓    │  ±              │");
    println!("   │ Third-Party Hold  │  ✗    │  ✗    │  ✓              │");
    println!("   │ Further Sharing   │  ±    │  ±    │  ✗              │");
    println!("   └──────────────────────────────────────────────────────┘\n");

    // Cleanup
    primal.discard_session(session_id).await?;
    primal.stop().await?;

    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("✅ Slice Demo completed!");
    println!();
    println!("Key Takeaways:");
    println!("  • Slices provide controlled access to session data");
    println!("  • Copy mode for independent local use");
    println!("  • Loan mode for time-limited sharing");
    println!("  • Consignment mode for third-party custody");
    println!("  • Checkout from LoamSpine rehydrates sessions");
    println!();

    Ok(())
}
EOF

log "Building demo..."
cargo build --release 2>&1 | grep -E "(Compiling|Finished|error)" || true

log "Running slice semantics demo..."
echo ""
cargo run --release 2>&1

rm -rf "$DEMO_DIR"
success "Demo complete!"
