// Demo: Session Management
//
// This demo shows creating and managing multiple sessions

use rhizo_crypt_core::*;
use std::time::Instant;

#[tokio::main]
async fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
    println!("═══════════════════════════════════════════════════════");
    println!("  Session Management");
    println!("═══════════════════════════════════════════════════════\n");
    
    let config = RhizoCryptConfig::default();
    let mut primal = RhizoCrypt::new(config);
    primal.start().await?;
    
    // Part 1: Create first session
    println!("┌─────────────────────────────────────────────────────┐");
    println!("│  Part 1: High-Speed Session                        │");
    println!("└─────────────────────────────────────────────────────┘\n");
    
    println!("Creating session...");
    let session1 = SessionBuilder::new(SessionType::General)
        .with_name("high-speed-demo")
        .build();
    
    let session1_id = primal.create_session(session1).await?;
    println!("✅ Session created: {}", session1_id);
    
    // Add vertices quickly
    println!("Adding vertices (measuring performance)...");
    let start = Instant::now();
    
    for i in 0..100 {
        let vertex = VertexBuilder::new(EventType::AgentAction {
            action: format!("op-{}", i),
        })
        .build();
        primal.append_vertex(session1_id, vertex).await?;
    }
    
    let duration = start.elapsed();
    println!("✅ Added 100 vertices in {:?}", duration);
    println!("   → Fast! In-memory operations\n");
    
    // Query session
    let session1_state = primal.get_session(session1_id).await?;
    println!("Session stats:");
    println!("   Vertices: {}", session1_state.vertex_count);
    println!("   Frontier: {} tips", session1_state.frontier.len());
    println!("   ✓ All in-memory, blazing fast!\n");
    
    // Part 2: Create second session
    println!("┌─────────────────────────────────────────────────────┐");
    println!("│  Part 2: Audit Trail Session                       │");
    println!("└─────────────────────────────────────────────────────┘\n");
    
    println!("Creating second session...");
    let session2 = SessionBuilder::new(SessionType::General)
        .with_name("audit-demo")
        .build();
    
    let session2_id = primal.create_session(session2).await?;
    println!("✅ Session created: {}", session2_id);
    
    // Add audit events
    println!("Adding critical audit events...");
    
    let audit1 = VertexBuilder::new(EventType::SessionStart)
        .with_metadata("user", "alice")
        .with_metadata("action", "login")
        .build();
    primal.append_vertex(session2_id, audit1).await?;
    
    let audit2 = VertexBuilder::new(EventType::AgentAction {
        action: "data-access".to_string(),
    })
        .with_metadata("resource", "/secure/documents")
        .build();
    primal.append_vertex(session2_id, audit2).await?;
    
    println!("✅ Added 2 audit events");
    println!("   → These events can be preserved for compliance\n");
    
    // Dehydrate audit session
    println!("Dehydrating audit session...");
    let merkle_root = primal.dehydrate(session2_id).await?;
    let session2_state = primal.get_session(session2_id).await?;
    println!("✅ Session dehydrated:");
    println!("   Merkle root: {}", merkle_root);
    println!("   Vertices: {}", session2_state.vertex_count);
    println!("   → This summary would be saved to LoamSpine\n");
    
    // List all sessions
    println!("═══════════════════════════════════════════════════════");
    println!("  📊 Active Sessions");
    println!("═══════════════════════════════════════════════════════\n");
    
    let sessions = primal.list_sessions().await;
    println!("Total sessions: {}", sessions.len());
    for session in sessions {
        println!("  • {} - {} vertices", 
                 session.name.unwrap_or_else(|| "unnamed".to_string()),
                 session.vertex_count);
    }
    
    // Cleanup
    println!("\n🧹 Cleaning up...");
    primal.discard_session(session1_id).await?;
    primal.discard_session(session2_id).await?;
    println!("✅ Sessions discarded\n");
    
    primal.stop().await?;
    
    Ok(())
}
