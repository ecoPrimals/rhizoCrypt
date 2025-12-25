#!/usr/bin/env bash
# Demo: Slice Semantics - Checkout from Permanent Storage
#
# This demo shows how slices work: checking out immutable snapshots
# from permanent storage (LoamSpine) into ephemeral working memory (rhizoCrypt).

set -e

echo "🔐 rhizoCrypt Demo: Slice Semantics"
echo "===================================="
echo ""
echo "Slices are immutable snapshots from permanent storage"
echo "that can be checked out into ephemeral working memory."
echo ""
echo "This enables the Rhizo-Loam pattern:"
echo "  Loam (permanent) → Slice → Rhizo (working) → Dehydrate → Loam"
echo ""

# Create temp directory
DEMO_DIR=$(mktemp -d)
cd "$DEMO_DIR"

# Create demo project
cat > Cargo.toml << 'EOF'
[package]
name = "slices-demo"
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
    println!("  Slice Semantics: Checkout → Work → Commit");
    println!("═══════════════════════════════════════════════════════\n");
    
    let config = RhizoCryptConfig::default();
    let mut primal = RhizoCrypt::new(config);
    primal.start().await?;
    
    // ========================================
    // Part 1: Understanding Slices
    // ========================================
    println!("┌─────────────────────────────────────────────────────┐");
    println!("│  Part 1: What is a Slice?                          │");
    println!("└─────────────────────────────────────────────────────┘\n");
    
    println!("A slice is an immutable snapshot of permanent storage:");
    println!("  • Represents a point-in-time view");
    println!("  • Content-addressed by hash");
    println!("  • Read-only reference");
    println!("  • Can be checked out into working memory\n");
    
    // ========================================
    // Part 2: Simulated Slice Checkout
    // ========================================
    println!("┌─────────────────────────────────────────────────────┐");
    println!("│  Part 2: Slice Checkout (Simulated)                │");
    println!("└─────────────────────────────────────────────────────┘\n");
    
    println!("Creating session with simulated slice checkout...");
    let session = SessionBuilder::new(SessionType::Ephemeral)
        .with_name("slice-demo")
        .with_description("Working over permanent storage snapshot")
        .build();
    
    let session_id = primal.create_session(session).await?;
    println!("✅ Session created: {}\n", session_id);
    
    // Simulate a slice checkout
    // In reality, this would:
    // 1. Query LoamSpine for specific commit
    // 2. Receive immutable snapshot
    // 3. Create genesis vertex referencing slice
    
    println!("Simulating slice checkout from LoamSpine...");
    
    // Create a "slice" vertex (genesis of this session)
    let slice_id = SliceId::new_v7();
    let slice_vertex = VertexBuilder::new(EventType::SliceCheckout {
        slice_id,
        source_commit: "loamspine-commit-abc123".to_string(),
    })
        .with_metadata("slice_mode", "Copy")
        .with_metadata("content_hash", "blake3-hash-xyz789")
        .with_metadata("timestamp", "2025-12-24T20:00:00Z")
        .build();
    
    let slice_vertex_id = primal.append_vertex(session_id, slice_vertex).await?;
    println!("✅ Slice checked out as genesis vertex");
    println!("   Slice ID: {}", slice_id);
    println!("   Vertex ID: {}", slice_vertex_id);
    println!("   Mode: Copy (full immutable snapshot)");
    println!("   Source: loamspine-commit-abc123\n");
    
    // ========================================
    // Part 3: Working Over Slice
    // ========================================
    println!("┌─────────────────────────────────────────────────────┐");
    println!("│  Part 3: Computing Over Slice                      │");
    println!("└─────────────────────────────────────────────────────┘\n");
    
    println!("Now we can compute over the slice data...");
    println!("(In this demo, we simulate operations)\n");
    
    // Simulate reading from slice and computing
    let computation1 = VertexBuilder::new(EventType::DataTransform)
        .with_parent(slice_vertex_id)
        .with_payload_ref(PayloadRef::inline(b"Filtered data"))
        .with_metadata("operation", "filter")
        .build();
    
    let comp1_id = primal.append_vertex(session_id, computation1).await?;
    println!("   ✓ Applied filter operation");
    
    let computation2 = VertexBuilder::new(EventType::DataTransform)
        .with_parent(comp1_id)
        .with_payload_ref(PayloadRef::inline(b"Aggregated result"))
        .with_metadata("operation", "aggregate")
        .build();
    
    primal.append_vertex(session_id, computation2).await?;
    println!("   ✓ Applied aggregation");
    
    let computation3 = VertexBuilder::new(EventType::DataCreate { schema: None })
        .with_parent(comp1_id)
        .with_payload_ref(PayloadRef::inline(b"Derived insights"))
        .with_metadata("operation", "derive")
        .build();
    
    primal.append_vertex(session_id, computation3).await?;
    println!("   ✓ Derived insights\n");
    
    // ========================================
    // Part 4: DAG Visualization
    // ========================================
    println!("┌─────────────────────────────────────────────────────┐");
    println!("│  Part 4: DAG Structure                             │");
    println!("└─────────────────────────────────────────────────────┘\n");
    
    let vertex_count = primal.vertex_count(session_id).await?;
    let genesis = primal.get_genesis(session_id).await?;
    let frontier = primal.get_frontier(session_id).await?;
    
    println!("Session DAG:");
    println!("   Total vertices: {}", vertex_count);
    println!("   Genesis (slice): {} vertex", genesis.len());
    println!("   Frontier (results): {} vertices\n", frontier.len());
    
    println!("Visual structure:");
    println!("   ");
    println!("   [Slice Checkout]  ← Genesis (from LoamSpine)");
    println!("          │");
    println!("          ├─→ [Filter]");
    println!("          │      ├─→ [Aggregate]  ← Frontier");
    println!("          │      └─→ [Derive]     ← Frontier");
    println!("          │");
    println!("   Immutable ↑         ↑ Ephemeral computations");
    println!("   snapshot            (can be committed back)\n");
    
    // ========================================
    // Part 5: Slice Modes
    // ========================================
    println!("┌─────────────────────────────────────────────────────┐");
    println!("│  Part 5: Slice Modes                               │");
    println!("└─────────────────────────────────────────────────────┘\n");
    
    println!("rhizoCrypt supports multiple slice modes:\n");
    
    println!("1. Copy Mode (default):");
    println!("   • Full immutable snapshot");
    println!("   • Read-only access");
    println!("   • No impact on source");
    println!("   • Use: Safe exploration\n");
    
    println!("2. Loan Mode:");
    println!("   • Temporary borrow");
    println!("   • Must return or commit");
    println!("   • Tracked by LoamSpine");
    println!("   • Use: Short-term operations\n");
    
    println!("3. Consignment Mode:");
    println!("   • Transfer ownership");
    println!("   • Original marked as consumed");
    println!("   • One-way operation");
    println!("   • Use: Data migration\n");
    
    // ========================================
    // Summary
    // ========================================
    println!("═══════════════════════════════════════════════════════");
    println!("  🎓 Key Takeaways:");
    println!("═══════════════════════════════════════════════════════");
    println!("  • Slices = immutable snapshots from permanent storage");
    println!("  • Checkout slice → Compute in rhizoCrypt → Dehydrate back");
    println!("  • Working memory over permanent data (Rhizo-Loam pattern)");
    println!("  • Slice becomes genesis vertex in session DAG");
    println!("  • Enables safe, ephemeral computation over permanent data");
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
# Demo: Slice Semantics - Checkout from Permanent Storage
#
# This demo shows how slices work

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR"

echo "🔐 rhizoCrypt Demo: Slice Semantics"
echo "===================================="
echo ""
echo "Slices are immutable snapshots from permanent storage"
echo "that can be checked out into ephemeral working memory."
echo ""

# Build if needed
if [ ! -f "target/debug/demo-slices" ]; then
    echo "Building demo (first run)..."
    cargo build --bin demo-slices --quiet
    echo ""
fi

# Run the demo
cargo run --bin demo-slices --quiet

echo ""
echo "✅ Demo complete!"
echo ""
echo "Next: Try ./demo-dehydration.sh to see the commit protocol"
