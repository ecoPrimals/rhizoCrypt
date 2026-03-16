// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2024–2026 ecoPrimals Project

// Demo: Session Lifecycle - Create → Grow → Resolve → Expire
//
// This demo shows the complete lifecycle of a rhizoCrypt session

use rhizo_crypt_core::*;

#[tokio::main]
async fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
    println!("═══════════════════════════════════════════════════════");
    println!("  Session Lifecycle: Create → Grow → Resolve → Expire");
    println!("═══════════════════════════════════════════════════════\n");
    
    // Step 1: Create primal and start it
    println!("📝 Step 1: Initialize rhizoCrypt Primal");
    let config = RhizoCryptConfig::default();
    let mut primal = RhizoCrypt::new(config);
    primal.start().await?;
    println!("✅ Primal started: {:?}\n", primal.state());
    
    // Step 2: Create Session
    println!("📝 Step 2: Create Session");
    let session = SessionBuilder::new(SessionType::General)
        .with_name("lifecycle-demo")
        .build();
    
    let session_id = primal.create_session(session).await?;
    println!("✅ Created session: {}", session_id);
    println!("   Type: General");
    println!("   State: Created → Active\n");
    
    // Step 3: Grow Session (add vertices)
    println!("📝 Step 3: Grow Session (add vertices to DAG)");
    
    // Create first vertex
    let vertex1 = VertexBuilder::new(EventType::SessionStart)
        .with_metadata("step", "1")
        .build();
    let v1_id = primal.append_vertex(session_id, vertex1).await?;
    println!("   ✓ Added vertex 1: SessionStart");
    
    // Create second vertex (child of first)
    let payload_data = b"Hello rhizoCrypt!";
    let payload_ref = PayloadRef::from_bytes(payload_data);
    let vertex2 = VertexBuilder::new(EventType::DataCreate { schema: None })
        .with_parent(v1_id)
        .with_payload(payload_ref)
        .with_metadata("step", "2")
        .build();
    let _v2_id = primal.append_vertex(session_id, vertex2).await?;
    println!("   ✓ Added vertex 2: DataCreate (child of vertex 1)");
    
    // Create third vertex (also child of first - branching)
    let vertex3 = VertexBuilder::new(EventType::DataModify { delta_type: "append".to_string() })
        .with_parent(v1_id)
        .with_metadata("step", "3")
        .build();
    primal.append_vertex(session_id, vertex3).await?;
    println!("   ✓ Added vertex 3: DataModify (another child - DAG branches!)");
    
    let session_check = primal.get_session(session_id).await?;
    println!("\n✅ Session grown: 3 vertices added");
    println!("   State: Active (DAG has {} vertices)\n", session_check.vertex_count);
    
    // Step 4: Query Session State
    println!("📝 Step 4: Query Session State");
    let session_state = primal.get_session(session_id).await?;
    
    println!("   Total vertices: {}", session_state.vertex_count);
    println!("   Genesis (roots): {} vertices", session_state.genesis.len());
    println!("   Frontier (tips): {} vertices", session_state.frontier.len());
    println!("   ✓ DAG structure: 1 root, branching to 2 tips\n");
    
    // Step 5: Generate Merkle Root
    println!("📝 Step 5: Generate Merkle Proof");
    let merkle_root = primal.compute_merkle_root(session_id).await?;
    println!("✅ Merkle root computed:");
    println!("   {}", merkle_root);
    println!("   ✓ Entire session has cryptographic integrity\n");
    
    // Step 6: Resolve Session (Dehydration)
    println!("📝 Step 6: Resolve Session (Dehydration)");
    let merkle_root_final = primal.dehydrate(session_id).await?;
    println!("✅ Session resolved via dehydration:");
    println!("   Merkle root: {}", merkle_root_final);
    let final_session = primal.get_session(session_id).await?;
    println!("   Vertex count: {}", final_session.vertex_count);
    println!("   State: Active → Resolved\n");
    
    // Step 7: Expire Session (Garbage Collection)
    println!("📝 Step 7: Expire Session (Garbage Collection)");
    primal.discard_session(session_id).await?;
    println!("✅ Session expired and garbage collected");
    println!("   State: Resolved → Expired");
    println!("   ✓ Full DAG is now forgotten (ephemeral by design)");
    println!("   ✓ Only dehydration summary would persist\n");
    
    // Summary
    println!("═══════════════════════════════════════════════════════");
    println!("  🎓 Key Takeaways:");
    println!("═══════════════════════════════════════════════════════");
    println!("  • Sessions have a defined lifecycle");
    println!("  • State transitions: Created → Active → Resolved → Expired");
    println!("  • Ephemeral by default (forgotten after resolution)");
    println!("  • Only dehydration summary persists to LoamSpine");
    println!("  • This is the 'Philosophy of Forgetting' in action!");
    println!("═══════════════════════════════════════════════════════\n");
    
    // Stop primal
    primal.stop().await?;
    
    Ok(())
}
