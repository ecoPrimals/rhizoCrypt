#!/usr/bin/env bash
# Demo: Dehydration Protocol - Commit to Permanent Storage
#
# This demo shows how dehydration works: committing ephemeral DAG results
# back to permanent storage (LoamSpine) with cryptographic provenance.

set -e

echo "🔐 rhizoCrypt Demo: Dehydration Protocol"
echo "========================================="
echo ""
echo "Dehydration is the process of committing ephemeral DAG results"
echo "to permanent storage with full cryptographic provenance."
echo ""
echo "The protocol ensures:"
echo "  • Atomicity (all or nothing)"
echo "  • Integrity (Merkle root verification)"
echo "  • Provenance (full DAG traceability)"
echo ""

# Create temp directory
DEMO_DIR=$(mktemp -d)
cd "$DEMO_DIR"

# Create demo project
cat > Cargo.toml << 'EOF'
[package]
name = "dehydration-demo"
version = "0.1.0"
edition = "2021"

[dependencies]
rhizo-crypt-core = { path = "../../../../../crates/rhizo-crypt-core" }
tokio = { version = "1.46", features = ["full"] }
EOF

mkdir -p src

# Create demo code
cat > src/main.rs << 'EOF'
use rhizo_crypt_core::*;

#[tokio::main]
async fn main() -> Result<()> {
    println!("═══════════════════════════════════════════════════════");
    println!("  Dehydration Protocol: Ephemeral → Permanent");
    println!("═══════════════════════════════════════════════════════\n");
    
    let config = RhizoCryptConfig::default();
    let mut primal = RhizoCrypt::new(config);
    primal.start().await?;
    
    // ========================================
    // Part 1: Create Working Session
    // ========================================
    println!("┌─────────────────────────────────────────────────────┐");
    println!("│  Part 1: Create Ephemeral Working Session          │");
    println!("└─────────────────────────────────────────────────────┘\n");
    
    let session = SessionBuilder::new(SessionType::Ephemeral)
        .with_name("dehydration-demo")
        .with_description("Compute results for permanent storage")
        .build();
    
    let session_id = primal.create_session(session).await?;
    println!("✅ Created ephemeral session: {}", session_id);
    println!("   Goal: Compute results to commit to permanent storage\n");
    
    // ========================================
    // Part 2: Build Computation DAG
    // ========================================
    println!("┌─────────────────────────────────────────────────────┐");
    println!("│  Part 2: Build Computation DAG                     │");
    println!("└─────────────────────────────────────────────────────┘\n");
    
    println!("Building computation workflow...\n");
    
    // Step 1: Load data
    let load_vertex = VertexBuilder::new(EventType::DataCreate { 
        schema: Some("user_events".to_string()) 
    })
        .with_payload_ref(PayloadRef::inline(b"Raw event data"))
        .with_metadata("source", "sensor_network")
        .with_metadata("records", "1000")
        .build();
    
    let load_id = primal.append_vertex(session_id, load_vertex).await?;
    println!("   ✓ Step 1: Load data (1000 records)");
    
    // Step 2: Clean data
    let clean_vertex = VertexBuilder::new(EventType::DataTransform)
        .with_parent(load_id)
        .with_payload_ref(PayloadRef::inline(b"Cleaned data"))
        .with_metadata("operation", "remove_duplicates")
        .with_metadata("records_out", "950")
        .build();
    
    let clean_id = primal.append_vertex(session_id, clean_vertex).await?;
    println!("   ✓ Step 2: Clean data (removed 50 duplicates)");
    
    // Step 3: Aggregate
    let agg_vertex = VertexBuilder::new(EventType::DataTransform)
        .with_parent(clean_id)
        .with_payload_ref(PayloadRef::inline(b"Aggregated metrics"))
        .with_metadata("operation", "group_by_user")
        .with_metadata("groups", "120")
        .build();
    
    let agg_id = primal.append_vertex(session_id, agg_vertex).await?;
    println!("   ✓ Step 3: Aggregate (120 user groups)");
    
    // Step 4: Generate insights
    let insight_vertex = VertexBuilder::new(EventType::Custom {
        domain: "analytics".into(),
        event_name: "insights_generated".into(),
    })
        .with_parent(agg_id)
        .with_payload_ref(PayloadRef::inline(b"User behavior insights"))
        .with_metadata("insight_count", "45")
        .with_metadata("confidence", "0.95")
        .build();
    
    primal.append_vertex(session_id, insight_vertex).await?;
    println!("   ✓ Step 4: Generate insights (45 findings)\n");
    
    let vertex_count = primal.vertex_count(session_id).await?;
    println!("✅ Built computation DAG with {} vertices\n", vertex_count);
    
    // ========================================
    // Part 3: Pre-Dehydration Analysis
    // ========================================
    println!("┌─────────────────────────────────────────────────────┐");
    println!("│  Part 3: Pre-Dehydration Analysis                  │");
    println!("└─────────────────────────────────────────────────────┘\n");
    
    let genesis = primal.get_genesis(session_id).await?;
    let frontier = primal.get_frontier(session_id).await?;
    
    println!("Session state before dehydration:");
    println!("   Genesis (roots): {} vertices", genesis.len());
    println!("   Frontier (tips): {} vertices", frontier.len());
    println!("   Total vertices: {}", vertex_count);
    println!("");
    println!("What will be committed:");
    println!("   → Frontier vertices (the final results)");
    println!("   → Merkle root (integrity proof)");
    println!("   → Full DAG provenance (traceable back to genesis)\n");
    
    // ========================================
    // Part 4: Execute Dehydration
    // ========================================
    println!("┌─────────────────────────────────────────────────────┐");
    println!("│  Part 4: Execute Dehydration Protocol              │");
    println!("└─────────────────────────────────────────────────────┘\n");
    
    println!("Initiating dehydration...");
    println!("  1. Resolving session (finalizing DAG)...");
    println!("  2. Computing Merkle root...");
    println!("  3. Extracting frontier vertices...");
    println!("  4. Preparing commit payload...\n");
    
    let summary = primal.dehydrate_session(session_id).await?;
    
    println!("✅ Dehydration complete!\n");
    println!("═══════════════════════════════════════════════════════");
    println!("  📦 Dehydration Summary");
    println!("═══════════════════════════════════════════════════════");
    println!("  Session ID:     {}", summary.session_id);
    println!("  Merkle Root:    {}", summary.merkle_root);
    println!("  Vertex Count:   {}", summary.vertex_count);
    println!("  Resolution:     {:?}", summary.resolution);
    println!("  Timestamp:      {}", summary.timestamp);
    println!("═══════════════════════════════════════════════════════\n");
    
    // ========================================
    // Part 5: What Happens Next (LoamSpine)
    // ========================================
    println!("┌─────────────────────────────────────────────────────┐");
    println!("│  Part 5: LoamSpine Commit (Simulated)              │");
    println!("└─────────────────────────────────────────────────────┘\n");
    
    println!("In a full integration, this summary would be sent to LoamSpine:\n");
    
    println!("  LoamSpine would:");
    println!("    1. Verify Merkle root integrity");
    println!("    2. Store frontier vertices");
    println!("    3. Index by content hash (Blake3)");
    println!("    4. Create permanent commit record");
    println!("    5. Return commit ID for future reference\n");
    
    println!("  Example LoamSpine commit record:");
    println!("    Commit ID:      loamspine-commit-xyz123");
    println!("    Merkle Root:    {}", summary.merkle_root);
    println!("    Source Session: {}", summary.session_id);
    println!("    Timestamp:      {}", summary.timestamp);
    println!("    Status:         Permanent\n");
    
    // ========================================
    // Part 6: The Full Cycle
    // ========================================
    println!("┌─────────────────────────────────────────────────────┐");
    println!("│  Part 6: The Full Rhizo-Loam Cycle                 │");
    println!("└─────────────────────────────────────────────────────┘\n");
    
    println!("The complete workflow:");
    println!("");
    println!("  ┌──────────────┐");
    println!("  │  LoamSpine   │  ← Permanent storage");
    println!("  │  (Permanent) │");
    println!("  └──────┬───────┘");
    println!("         │ Slice checkout (read)");
    println!("         ▼");
    println!("  ┌──────────────┐");
    println!("  │  rhizoCrypt  │  ← Working memory");
    println!("  │  (Ephemeral) │     - Build DAG");
    println!("  │              │     - Compute");
    println!("  └──────┬───────┘     - Transform");
    println!("         │ Dehydration (write)");
    println!("         ▼");
    println!("  ┌──────────────┐");
    println!("  │  LoamSpine   │  ← Results stored");
    println!("  │  (Permanent) │");
    println!("  └──────────────┘\n");
    
    // ========================================
    // Summary
    // ========================================
    println!("═══════════════════════════════════════════════════════");
    println!("  🎓 Key Takeaways:");
    println!("═══════════════════════════════════════════════════════");
    println!("  • Dehydration commits ephemeral results to permanent storage");
    println!("  • Merkle root provides cryptographic integrity");
    println!("  • Only frontier vertices are committed (results, not all steps)");
    println!("  • Full DAG provenance is traceable via Merkle tree");
    println!("  • Enables 'compute in working memory, save only results'");
    println!("  • Philosophy: Ephemeral by default, persistent by consent");
    println!("═══════════════════════════════════════════════════════\n");
    
    // Cleanup
    primal.discard_session(session_id).await?;
    primal.stop().await?;
    
    Ok(())
}
EOF

# Build and run
echo "Building demo..."
cargo build --quiet 2>&1 | grep -v "Compiling\|Finished" || true
echo ""
echo "Running demo..."
echo ""
cargo run --quiet

# Cleanup
cd ..
rm -rf "$DEMO_DIR"

#!/usr/bin/env bash
# Demo: Dehydration Protocol - Commit to Permanent Storage
#
# This demo shows how dehydration works

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR"

echo "🔐 rhizoCrypt Demo: Dehydration Protocol"
echo "========================================="
echo ""
echo "Dehydration is the process of committing ephemeral DAG results"
echo "to permanent storage with full cryptographic provenance."
echo ""

# Build if needed
if [ ! -f "target/debug/demo-dehydration" ]; then
    echo "Building demo (first run)..."
    cargo build --bin demo-dehydration --quiet
    echo ""
fi

# Run the demo
cargo run --bin demo-dehydration --quiet

echo ""
echo "✅ Demo complete!"
echo ""
echo "═══════════════════════════════════════════════════════"
echo "  🎉 Level 4 Complete: Sessions & Lifecycle"
echo "═══════════════════════════════════════════════════════"
echo ""
echo "You've learned:"
echo "  ✓ Session lifecycle (create → grow → resolve → expire)"
echo "  ✓ Ephemeral vs Persistent sessions"
echo "  ✓ Slice semantics (checkout from permanent storage)"
echo "  ✓ Dehydration protocol (commit back to permanent storage)"
echo ""
echo "Next level: cd ../05-performance"
