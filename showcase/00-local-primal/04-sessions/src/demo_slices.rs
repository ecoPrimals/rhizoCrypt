// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2024–2026 ecoPrimals Project

// Demo: Working with Slices
//
// This demo shows slice checkout patterns

use rhizo_crypt_core::*;

#[tokio::main]
async fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
    println!("═══════════════════════════════════════════════════════");
    println!("  Slice Semantics: Checkout → Work → Commit");
    println!("═══════════════════════════════════════════════════════\n");
    
    let config = RhizoCryptConfig::default();
    let mut primal = RhizoCrypt::new(config);
    primal.start().await?;
    
    // Part 1: Understanding Slices
    println!("┌─────────────────────────────────────────────────────┐");
    println!("│  Part 1: What is a Slice?                          │");
    println!("└─────────────────────────────────────────────────────┘\n");
    
    println!("A slice is an immutable snapshot of permanent storage:");
    println!("  • Represents a point-in-time view");
    println!("  • Content-addressed by hash");
    println!("  • Read-only reference");
    println!("  • Can be checked out into working memory\n");
    
    // Part 2: Simulated Slice Checkout
    println!("┌─────────────────────────────────────────────────────┐");
    println!("│  Part 2: Slice Checkout (Simulated)                │");
    println!("└─────────────────────────────────────────────────────┘\n");
    
    println!("Creating session with simulated slice checkout...");
    let session = SessionBuilder::new(SessionType::General)
        .with_name("slice-demo")
        .build();
    
    let session_id = primal.create_session(session).await?;
    println!("✅ Session created: {}\n", session_id);
    
    // Simulate a slice checkout
    println!("Simulating slice checkout from LoamSpine...");
    
    let slice_id = SliceId::now();
    let slice_vertex = VertexBuilder::new(EventType::SliceCheckout {
        slice_id,
        mode: event::SliceMode::Copy { allow_recopy: false },
    })
        .with_metadata("source", "loamspine-commit-abc123")
        .build();
    
    let slice_vertex_id = primal.append_vertex(session_id, slice_vertex).await?;
    println!("✅ Slice checked out as genesis vertex");
    println!("   Slice ID: {}", slice_id);
    println!("   Vertex ID: {}", slice_vertex_id);
    println!("   Mode: Copy (full immutable snapshot)\n");
    
    // Part 3: Computing Over Slice
    println!("┌─────────────────────────────────────────────────────┐");
    println!("│  Part 3: Computing Over Slice                      │");
    println!("└─────────────────────────────────────────────────────┘\n");
    
    println!("Now we can compute over the slice data...\n");
    
    // Simulate reading from slice and computing
    let computation1 = VertexBuilder::new(EventType::SliceOperation {
        slice_id,
        operation: "filter".to_string(),
    })
        .with_parent(slice_vertex_id)
        .build();
    
    let comp1_id = primal.append_vertex(session_id, computation1).await?;
    println!("   ✓ Applied filter operation");
    
    let computation2 = VertexBuilder::new(EventType::SliceOperation {
        slice_id,
        operation: "aggregate".to_string(),
    })
        .with_parent(comp1_id)
        .build();
    
    primal.append_vertex(session_id, computation2).await?;
    println!("   ✓ Applied aggregation");
    
    let computation3 = VertexBuilder::new(EventType::DataCreate { schema: None })
        .with_parent(comp1_id)
        .build();
    
    primal.append_vertex(session_id, computation3).await?;
    println!("   ✓ Derived insights\n");
    
    // Part 4: DAG Visualization
    println!("┌─────────────────────────────────────────────────────┐");
    println!("│  Part 4: DAG Structure                             │");
    println!("└─────────────────────────────────────────────────────┘\n");
    
    let session_state = primal.get_session(session_id).await?;
    
    println!("Session DAG:");
    println!("   Total vertices: {}", session_state.vertex_count);
    println!("   Genesis (slice): {} vertex", session_state.genesis.len());
    println!("   Frontier (results): {} vertices\n", session_state.frontier.len());
    
    println!("Visual structure:");
    println!("   ");
    println!("   [Slice Checkout]  ← Genesis (from LoamSpine)");
    println!("          │");
    println!("          ├─→ [Filter]");
    println!("          │      ├─→ [Aggregate]  ← Frontier");
    println!("          │      └─→ [Derive]     ← Frontier");
    println!("   ");
    println!("   Immutable snapshot → Ephemeral computations\n");
    
    // Summary
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
