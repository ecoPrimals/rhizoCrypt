// Demo: Dehydration Protocol
//
// This demo shows how dehydration works

use rhizo_crypt_core::*;

#[tokio::main]
async fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
    println!("═══════════════════════════════════════════════════════");
    println!("  Dehydration Protocol: Ephemeral → Permanent");
    println!("═══════════════════════════════════════════════════════\n");
    
    let config = RhizoCryptConfig::default();
    let mut primal = RhizoCrypt::new(config);
    primal.start().await?;
    
    // Part 1: Create Working Session
    println!("┌─────────────────────────────────────────────────────┐");
    println!("│  Part 1: Create Ephemeral Working Session          │");
    println!("└─────────────────────────────────────────────────────┘\n");
    
    let session = SessionBuilder::new(SessionType::General)
        .with_name("dehydration-demo")
        .build();
    
    let session_id = primal.create_session(session).await?;
    println!("✅ Created session: {}", session_id);
    println!("   Goal: Compute results to commit to permanent storage\n");
    
    // Part 2: Build Computation DAG
    println!("┌─────────────────────────────────────────────────────┐");
    println!("│  Part 2: Build Computation DAG                     │");
    println!("└─────────────────────────────────────────────────────┘\n");
    
    println!("Building computation workflow...\n");
    
    // Step 1: Load data
    let load_vertex = VertexBuilder::new(EventType::DataCreate { 
        schema: Some("user_events".to_string()) 
    })
        .with_metadata("source", "sensor_network")
        .with_metadata("records", "1000")
        .build();
    
    let load_id = primal.append_vertex(session_id, load_vertex).await?;
    println!("   ✓ Step 1: Load data (1000 records)");
    
    // Step 2: Clean data
    let clean_vertex = VertexBuilder::new(EventType::DataModify {
        delta_type: "clean".to_string(),
    })
        .with_parent(load_id)
        .with_metadata("operation", "remove_duplicates")
        .with_metadata("records_out", "950")
        .build();
    
    let clean_id = primal.append_vertex(session_id, clean_vertex).await?;
    println!("   ✓ Step 2: Clean data (removed 50 duplicates)");
    
    // Step 3: Aggregate
    let agg_vertex = VertexBuilder::new(EventType::DataModify {
        delta_type: "aggregate".to_string(),
    })
        .with_parent(clean_id)
        .with_metadata("operation", "group_by_user")
        .with_metadata("groups", "120")
        .build();
    
    let agg_id = primal.append_vertex(session_id, agg_vertex).await?;
    println!("   ✓ Step 3: Aggregate (120 user groups)");
    
    // Step 4: Generate insights
    let insight_vertex = VertexBuilder::new(EventType::AgentAction {
        action: "generate_insights".to_string(),
    })
        .with_parent(agg_id)
        .with_metadata("insight_count", "45")
        .with_metadata("confidence", "0.95")
        .build();
    
    primal.append_vertex(session_id, insight_vertex).await?;
    println!("   ✓ Step 4: Generate insights (45 findings)\n");
    
    let session_check = primal.get_session(session_id).await?;
    println!("✅ Built computation DAG with {} vertices\n", session_check.vertex_count);
    
    // Part 3: Pre-Dehydration Analysis
    println!("┌─────────────────────────────────────────────────────┐");
    println!("│  Part 3: Pre-Dehydration Analysis                  │");
    println!("└─────────────────────────────────────────────────────┘\n");
    
    let pre_dehydrate_state = primal.get_session(session_id).await?;
    
    println!("Session state before dehydration:");
    println!("   Genesis (roots): {} vertices", pre_dehydrate_state.genesis.len());
    println!("   Frontier (tips): {} vertices", pre_dehydrate_state.frontier.len());
    println!("   Total vertices: {}", pre_dehydrate_state.vertex_count);
    println!("");
    println!("What will be committed:");
    println!("   → Frontier vertices (the final results)");
    println!("   → Merkle root (integrity proof)");
    println!("   → Full DAG provenance (traceable back to genesis)\n");
    
    // Part 4: Execute Dehydration
    println!("┌─────────────────────────────────────────────────────┐");
    println!("│  Part 4: Execute Dehydration Protocol              │");
    println!("└─────────────────────────────────────────────────────┘\n");
    
    println!("Initiating dehydration...");
    println!("  1. Resolving session (finalizing DAG)...");
    println!("  2. Computing Merkle root...");
    println!("  3. Extracting frontier vertices...");
    println!("  4. Preparing commit payload...\n");
    
    let merkle_root = primal.dehydrate(session_id).await?;
    let post_dehydrate_state = primal.get_session(session_id).await?;
    
    println!("✅ Dehydration complete!\n");
    println!("═══════════════════════════════════════════════════════");
    println!("  📦 Dehydration Summary");
    println!("═══════════════════════════════════════════════════════");
    println!("  Session ID:     {}", session_id);
    println!("  Merkle Root:    {}", merkle_root);
    println!("  Vertex Count:   {}", post_dehydrate_state.vertex_count);
    println!("  State:          {:?}", post_dehydrate_state.state);
    println!("  Created:        {}", post_dehydrate_state.created_at);
    println!("═══════════════════════════════════════════════════════\n");
    
    // Summary
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
