#!/bin/bash
#
# 🔐 rhizoCrypt LoamSpine Commit Demo
#
# Demonstrates permanent storage via LoamSpine.
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
║          🔐 rhizoCrypt LoamSpine Commit Demo                   ║
║                                                                ║
║  Demonstrates: Permanent Storage • Merkle Commits              ║
╚═══════════════════════════════════════════════════════════════╝
EOF
echo -e "${NC}"

# Get the path to rhizoCrypt BEFORE changing directories
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
RHIZO_PATH="$(cd "$SCRIPT_DIR/../../.." && pwd)"

DEMO_DIR="/tmp/rhizocrypt-loamspine-demo"
rm -rf "$DEMO_DIR"
mkdir -p "$DEMO_DIR"
cd "$DEMO_DIR"

cat > Cargo.toml << EOF
[package]
name = "loamspine-demo"
version = "0.1.0"
edition = "2021"

[dependencies]
rhizo-crypt-core = { path = "$RHIZO_PATH/crates/rhizo-crypt-core", features = ["test-utils"] }
tokio = { version = "1.0", features = ["full"] }
hex = "0.4"
EOF

mkdir -p src

cat > src/main.rs << 'EOF'
//! LoamSpine Commit Demo
//!
//! Shows permanent storage workflow via LoamSpine.

use rhizo_crypt_core::{
    EventType, PrimalLifecycle, RhizoCrypt, RhizoCryptConfig,
    SessionBuilder, SessionType, VertexBuilder,
    dehydration::DehydrationSummaryBuilder,
    integration::{LoamSpineClient, MockLoamSpineClient},
    types::Did,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n🔐 rhizoCrypt LoamSpine Commit Demo\n");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");

    // Initialize rhizoCrypt
    let config = RhizoCryptConfig::default();
    let mut primal = RhizoCrypt::new(config);
    primal.start().await?;

    // Create a MockLoamSpineClient (simulates LoamSpine)
    let loamspine = MockLoamSpineClient::new();

    // Phase 1: Create a session and add vertices
    println!("📦 Phase 1: Building Session\n");
    let session = SessionBuilder::new(SessionType::Collaboration {
        workspace_id: "demo-workspace".to_string(),
    })
    .with_name("Multi-Agent Research")
    .build();
    let session_id = primal.create_session(session).await?;
    println!("   Session: {}", session_id);

    // Add vertices
    let genesis = VertexBuilder::new(EventType::SessionStart).build();
    let v1 = primal.append_vertex(session_id, genesis).await?;

    for i in 1..=5 {
        let v = VertexBuilder::new(EventType::DataCreate { schema: Some("research".to_string()) })
            .with_parent(v1)
            .with_metadata("step", format!("{}", i))
            .build();
        primal.append_vertex(session_id, v).await?;
    }

    let info = primal.get_session(session_id).await?;
    println!("   Vertices: {}\n", info.vertex_count);

    // Phase 2: Compute Merkle root
    println!("🌳 Phase 2: Computing Merkle Root\n");
    let merkle_root = primal.compute_merkle_root(session_id).await?;
    println!("   Merkle Root: {}", merkle_root);
    println!("   This single hash proves all {} vertices\n", info.vertex_count);

    // Phase 3: Create dehydration summary using builder
    println!("📋 Phase 3: Creating Dehydration Summary\n");
    let session_info = primal.get_session(session_id).await?;
    let summary = DehydrationSummaryBuilder::new(
        session_id,
        "collaboration",
        session_info.created_at,
        merkle_root,
    )
    .with_vertex_count(session_info.vertex_count)
    .build();
    println!("   Summary created with {} vertices\n", summary.vertex_count);

    // Phase 4: Commit to LoamSpine
    println!("💾 Phase 4: Committing to LoamSpine\n");
    let commit_ref = loamspine.commit(&summary).await?;
    println!("   ✓ Commit: spine={}, index={}", commit_ref.spine_id, commit_ref.index);
    println!("   ✓ Session is now permanently stored\n");

    // Phase 5: Verify commit exists
    println!("🔍 Phase 5: Verifying Commit\n");
    let valid = loamspine.verify_commit(&commit_ref).await?;
    println!("   Commit valid: {}\n", valid);

    // Phase 6: Checkout (simulated rehydration)
    println!("📥 Phase 6: Checkout (Rehydration)\n");
    let holder = Did::new("did:example:demo-user");
    let slice_origin = loamspine.checkout_slice(
        &commit_ref.spine_id,
        &commit_ref.entry_hash,
        &holder,
    ).await?;
    println!("   ✓ Checkout successful");
    println!("   ✓ Slice origin: spine={}", slice_origin.spine_id);
    println!("   ✓ Entry index: {}\n", slice_origin.entry_index);

    // Show the flow
    println!("📊 Commit Flow:\n");
    println!("   ┌─────────────────────────────────────────────────┐");
    println!("   │             rhizoCrypt (Ephemeral)              │");
    println!("   │                                                 │");
    println!("   │  Session: {} vertices                           │", info.vertex_count);
    println!("   │  ↓                                              │");
    println!("   │  Merkle Root (computed)                         │");
    println!("   │  ↓                                              │");
    println!("   │  Dehydration Summary                            │");
    println!("   └─────────────────────────────────────────────────┘");
    println!("                         │");
    println!("                         │ commit()");
    println!("                         ▼");
    println!("   ┌─────────────────────────────────────────────────┐");
    println!("   │             LoamSpine (Permanent)               │");
    println!("   │                                                 │");
    println!("   │  Spine: {}                              │", commit_ref.spine_id);
    println!("   │  • Merkle Root (proof of all vertices)          │");
    println!("   │  • Attestations (who contributed)               │");
    println!("   │  • Payload refs (NestGate content)              │");
    println!("   │  • Lineage (parent commits)                     │");
    println!("   └─────────────────────────────────────────────────┘\n");

    // Cleanup
    primal.discard_session(session_id).await?;
    primal.stop().await?;

    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("✅ LoamSpine Demo completed!");
    println!();
    println!("Key Takeaways:");
    println!("  • Sessions are ephemeral, commits are permanent");
    println!("  • Merkle root proves all vertices without storing them");
    println!("  • LoamSpine stores compact summaries, not full DAGs");
    println!("  • Checkout rehydrates sessions from commits");
    println!("  • Lineage enables commit history tracking");
    println!();

    Ok(())
}
EOF

log "Building demo..."
cargo build --release 2>&1 | grep -E "(Compiling|Finished|error)" || true

log "Running LoamSpine commit demo..."
echo ""
cargo run --release 2>&1

rm -rf "$DEMO_DIR"
success "Demo complete!"
