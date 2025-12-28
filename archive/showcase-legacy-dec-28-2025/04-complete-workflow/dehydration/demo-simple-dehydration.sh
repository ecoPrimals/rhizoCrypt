#!/bin/bash
#
# 🔐 rhizoCrypt Simple Dehydration Demo
#
# Demonstrates the complete capture → commit workflow:
# 1. Create session
# 2. Add vertices
# 3. Compute Merkle root
# 4. Dehydrate to summary
# 5. Show what gets committed vs forgotten
#

set -e

# Colors
BLUE='\033[0;34m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
PURPLE='\033[0;35m'
CYAN='\033[0;36m'
NC='\033[0m'

log() { echo -e "${BLUE}[$(date '+%H:%M:%S')]${NC} $1"; }
success() { echo -e "${GREEN}✓${NC} $1"; }

echo -e "${PURPLE}"
cat << 'EOF'
╔═══════════════════════════════════════════════════════════════╗
║          🔐 rhizoCrypt Dehydration Workflow Demo               ║
║                                                                ║
║  Capture → Merkle → Summary → Commit                           ║
╚═══════════════════════════════════════════════════════════════╝
EOF
echo -e "${NC}"

# Get the path to rhizoCrypt BEFORE changing directories
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
RHIZO_PATH="$(cd "$SCRIPT_DIR/../../.." && pwd)"

DEMO_DIR="/tmp/rhizocrypt-dehydration-demo"
rm -rf "$DEMO_DIR"
mkdir -p "$DEMO_DIR"
cd "$DEMO_DIR"

cat > Cargo.toml << EOF
[package]
name = "dehydration-demo"
version = "0.1.0"
edition = "2021"

[dependencies]
rhizo-crypt-core = { path = "$RHIZO_PATH/crates/rhizo-crypt-core" }
tokio = { version = "1.0", features = ["full"] }
blake3 = "1.8"
hex = "0.4"
EOF

mkdir -p src

cat > src/main.rs << 'EOF'
//! Simple Dehydration Demo
//!
//! Shows the complete session → commit workflow.

use rhizo_crypt_core::{
    EventType, PrimalLifecycle, RhizoCrypt, RhizoCryptConfig,
    SessionBuilder, SessionType, VertexBuilder,
    event::SessionOutcome,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n🔐 rhizoCrypt Dehydration Workflow Demo\n");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");

    // Initialize rhizoCrypt
    println!("📦 Phase 1: Initialize rhizoCrypt\n");
    let config = RhizoCryptConfig::default();
    let mut primal = RhizoCrypt::new(config);
    primal.start().await?;
    println!("   ✓ rhizoCrypt started\n");

    // Create session
    println!("🎬 Phase 2: Create Session\n");
    let session = SessionBuilder::new(SessionType::Experiment {
        protocol_id: "dehydration-demo-v1".to_string(),
    })
    .with_name("Dehydration Demo Session")
    .with_max_vertices(100)
    .build();

    let session_id = primal.create_session(session).await?;
    println!("   ✓ Session created: {}", session_id);
    println!("   ✓ Type: Experiment (dehydration-demo-v1)\n");

    // Add vertices (simulating a research workflow)
    println!("📝 Phase 3: Capture Events (10 vertices)\n");

    let mut last_vertex = None;

    // Session start
    let v = VertexBuilder::new(EventType::SessionStart)
        .with_metadata("researcher", "Alice")
        .build();
    let v1 = primal.append_vertex(session_id, v).await?;
    println!("   v1: Session started by Alice");
    last_vertex = Some(v1);

    // Data collection
    for i in 1..=5 {
        let v = VertexBuilder::new(EventType::DataCreate { 
            schema: Some("experiment-data".to_string()) 
        })
        .with_parent(last_vertex.unwrap())
        .with_metadata("sample_id", format!("sample_{}", i))
        .with_metadata("temperature", format!("{}.5", 20 + i))
        .build();
        let vid = primal.append_vertex(session_id, v).await?;
        println!("   v{}: Data sample {} collected", i + 1, i);
        last_vertex = Some(vid);
    }

    // Analysis
    let analysis = VertexBuilder::new(EventType::DataModify { 
        delta_type: "statistical-analysis".to_string() 
    })
    .with_parent(last_vertex.unwrap())
    .with_metadata("analysis_type", "statistical")
    .with_metadata("result", "significant")
    .build();
    let v7 = primal.append_vertex(session_id, analysis).await?;
    println!("   v7: Statistical analysis performed");

    // Model training
    let training = VertexBuilder::new(EventType::AgentAction {
        action: "ml-training-started".to_string(),
    })
    .with_parent(v7)
    .with_metadata("model", "linear-regression")
    .with_metadata("epochs", "100")
    .build();
    let v8 = primal.append_vertex(session_id, training).await?;
    println!("   v8: ML training started");

    let trained = VertexBuilder::new(EventType::AgentAction {
        action: "ml-training-completed".to_string(),
    })
    .with_parent(v8)
    .with_metadata("accuracy", "0.95")
    .with_metadata("loss", "0.05")
    .build();
    let v9 = primal.append_vertex(session_id, trained).await?;
    println!("   v9: ML training completed");

    // Session end
    let end = VertexBuilder::new(EventType::SessionEnd {
        outcome: SessionOutcome::Success,
    })
    .with_parent(v9)
    .with_metadata("status", "success")
    .with_metadata("conclusion", "hypothesis confirmed")
    .build();
    let _v10 = primal.append_vertex(session_id, end).await?;
    println!("   v10: Session completed");

    println!();

    // Compute Merkle root
    println!("🌳 Phase 4: Compute Merkle Root\n");
    let merkle_root = primal.compute_merkle_root(session_id).await?;
    println!("   ✓ Merkle root: {}", merkle_root);
    println!("   ✓ This proves integrity of all 10 vertices\n");

    // Get session info for summary
    let session_info = primal.get_session(session_id).await?;
    
    // Show dehydration summary
    println!("📊 Phase 5: Dehydration Summary\n");
    println!("   ┌─────────────────────────────────────────────────┐");
    println!("   │           DEHYDRATION SUMMARY                   │");
    println!("   ├─────────────────────────────────────────────────┤");
    println!("   │ Session ID: {}         │", &session_id.to_string()[..16]);
    println!("   │ Vertices: {:>4}                                  │", session_info.vertex_count);
    println!("   │ Merkle Root: {}           │", merkle_root);
    println!("   │ Session Type: Experiment                        │");
    println!("   │ State: {:?}                                │", session_info.state);
    println!("   └─────────────────────────────────────────────────┘");
    println!();

    // Simulate LoamSpine commit
    println!("💾 Phase 6: Commit to LoamSpine (simulated)\n");
    
    // In production, this would call LoamSpine via discovery
    let commit_hash = blake3::hash(format!(
        "commit:{}:{}",
        session_id,
        merkle_root
    ).as_bytes());
    
    println!("   ✓ Commit ID: {}", hex::encode(&commit_hash.as_bytes()[..16]));
    println!("   ✓ LoamSpine would store:");
    println!("     • Session metadata");
    println!("     • Merkle root (integrity proof)");
    println!("     • Attestations (provenance)");
    println!("     • Payload references (NestGate)");
    println!();

    // Show what gets forgotten
    println!("🗑️  Phase 7: Ephemeral Cleanup\n");
    
    println!("   Discarding ephemeral data:");
    println!("     • Individual vertices (10)");
    println!("     • In-memory DAG index");
    println!("     • Session state");
    println!();
    println!("   What's preserved in LoamSpine:");
    println!("     • Merkle root (proves all 10 vertices)");
    println!("     • Session summary");
    println!("     • Attestations");
    println!("     • Payload references");

    primal.discard_session(session_id).await?;
    println!();
    println!("   ✓ Session discarded - only the commit survives!");

    // Cleanup
    primal.stop().await?;

    println!();
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("✅ Dehydration Demo completed!");
    println!();
    println!("The Dehydration Flow:");
    println!();
    println!("  ┌─────────────────┐");
    println!("  │ 10 Vertices     │  ← Ephemeral (forgotten)");
    println!("  │ (raw events)    │");
    println!("  └────────┬────────┘");
    println!("           │");
    println!("           ▼");
    println!("  ┌─────────────────┐");
    println!("  │ Merkle Root     │  ← Proves all vertices");
    println!("  │ (single hash)   │");
    println!("  └────────┬────────┘");
    println!("           │");
    println!("           ▼");
    println!("  ┌─────────────────┐");
    println!("  │ Commit          │  ← Permanent (LoamSpine)");
    println!("  │ (summary)       │");
    println!("  └─────────────────┘");
    println!();
    println!("Key Insight: 10 vertices → 1 Merkle root → 1 commit");
    println!("             Ephemeral  →  Proof       →  Permanent");
    println!();

    Ok(())
}
EOF

log "Building demo..."
cargo build --release 2>&1 | grep -E "(Compiling|Finished|error)" || true

log "Running dehydration demo..."
echo ""
cargo run --release 2>&1

rm -rf "$DEMO_DIR"
success "Demo complete!"
