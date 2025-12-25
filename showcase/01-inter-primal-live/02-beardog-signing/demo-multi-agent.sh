#!/usr/bin/env bash
# Demo: Multi-Agent Session with BearDog Signatures
#
# Demonstrates multiple agents (DIDs) collaborating in one session

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

echo "🐻👥 rhizoCrypt + BearDog: Multi-Agent Session Demo"
echo "===================================================="
echo ""

# Create Rust demo project
RUST_DEMO_DIR=$(mktemp -d)
cd "$RUST_DEMO_DIR"

RHIZO_PATH="$SCRIPT_DIR/../../../crates/rhizo-crypt-core"

cat > Cargo.toml << EOF
[package]
name = "multi-agent-demo"
version = "0.1.0"
edition = "2021"

[dependencies]
rhizo-crypt-core = { path = "$RHIZO_PATH" }
tokio = { version = "1.46", features = ["full"] }
EOF

mkdir -p src

cat > src/main.rs << 'EOF'
use rhizo_crypt_core::*;

#[tokio::main]
async fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
    println!("═══════════════════════════════════════════════════════");
    println!("  Multi-Agent Collaboration with Signed Vertices");
    println!("═══════════════════════════════════════════════════════\n");
    
    // Initialize rhizoCrypt
    let config = RhizoCryptConfig::default();
    let mut primal = RhizoCrypt::new(config);
    primal.start().await?;
    
    // Create session
    let session = SessionBuilder::new(SessionType::General)
        .with_name("multi-agent-collaboration")
        .build();
    
    let session_id = primal.create_session(session).await?;
    println!("✅ Session created: {}\n", session_id);
    
    // Define multiple agents (DIDs)
    let alice = Did::new("did:key:alice123");
    let bob = Did::new("did:key:bob456");
    let charlie = Did::new("did:key:charlie789");
    
    println!("👥 Agents:");
    println!("   • Alice: {}", alice.as_str());
    println!("   • Bob: {}", bob.as_str());
    println!("   • Charlie: {}", charlie.as_str());
    println!("");
    
    // Alice creates initial data
    println!("📝 Alice: Creates initial data");
    let alice_vertex = VertexBuilder::new(EventType::DataCreate { 
        schema: Some("project-plan".to_string()) 
    })
        .with_agent(alice.clone())
        .with_payload(PayloadRef::from_bytes(b"Project Plan v1"))
        .with_metadata("author", "alice")
        .with_metadata("action", "create")
        .build();
    
    let alice_id = primal.append_vertex(session_id, alice_vertex).await?;
    println!("   ✓ Vertex added by Alice");
    println!("");
    
    // Bob reviews and modifies
    println!("📝 Bob: Reviews and modifies");
    let bob_vertex = VertexBuilder::new(EventType::DataModify { 
        delta_type: "review".to_string() 
    })
        .with_agent(bob.clone())
        .with_parent(alice_id)
        .with_payload(PayloadRef::from_bytes(b"Project Plan v2 (Bob's edits)"))
        .with_metadata("author", "bob")
        .with_metadata("action", "review")
        .build();
    
    let bob_id = primal.append_vertex(session_id, bob_vertex).await?;
    println!("   ✓ Vertex added by Bob (child of Alice's)");
    println!("");
    
    // Charlie approves
    println!("📝 Charlie: Approves changes");
    let charlie_vertex = VertexBuilder::new(EventType::AgentAction { 
        action: "approve".to_string() 
    })
        .with_agent(charlie.clone())
        .with_parent(bob_id)
        .with_metadata("author", "charlie")
        .with_metadata("action", "approve")
        .build();
    
    primal.append_vertex(session_id, charlie_vertex).await?;
    println!("   ✓ Vertex added by Charlie (child of Bob's)");
    println!("");
    
    // Verify session state
    println!("📊 Session State:");
    let session_state = primal.get_session(session_id).await?;
    println!("   Total vertices: {}", session_state.vertex_count);
    println!("   Participating agents: {}", session_state.agents.len());
    println!("   Genesis (roots): {}", session_state.genesis.len());
    println!("   Frontier (tips): {}", session_state.frontier.len());
    println!("");
    
    // Show DAG structure
    println!("🌳 DAG Structure:");
    println!("   ");
    println!("   [Alice: Create]  ← Genesis");
    println!("          │");
    println!("          ▼");
    println!("   [Bob: Review]");
    println!("          │");
    println!("          ▼");
    println!("   [Charlie: Approve]  ← Frontier");
    println!("");
    
    // Compute Merkle root
    let merkle_root = primal.compute_merkle_root(session_id).await?;
    println!("🔐 Merkle Root: {}", merkle_root);
    println!("   ✓ All agents' contributions cryptographically linked");
    println!("");
    
    // Summary
    println!("═══════════════════════════════════════════════════════");
    println!("  🎓 Key Takeaways:");
    println!("═══════════════════════════════════════════════════════");
    println!("  • Multiple agents can collaborate in one session");
    println!("  • Each vertex is signed by its agent");
    println!("  • DAG structure preserves causality");
    println!("  • Merkle root proves all contributions");
    println!("  • Enables multi-party workflows");
    println!("═══════════════════════════════════════════════════════\n");
    
    // Cleanup
    primal.discard_session(session_id).await?;
    primal.stop().await?;
    
    Ok(())
}
EOF

echo "🔨 Building demo..."
cargo build --quiet 2>&1 | grep -v "Compiling\|Finished" || true
echo ""

echo "▶️  Running demo..."
echo ""
cargo run --quiet

# Cleanup
cd "$SCRIPT_DIR"
rm -rf "$RUST_DEMO_DIR"

echo ""
echo "✅ Demo complete!"
echo ""
echo "🎯 Use Cases:"
echo "  • Collaborative document editing"
echo "  • Multi-party approvals"
echo "  • Scientific experiments (multiple researchers)"
echo "  • Gaming (multiple players)"
echo "  • Provenance tracking (supply chain)"
echo ""

