#!/usr/bin/env bash
# Demo: Event Sourcing Pattern
set -euo pipefail

GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
NC='\033[0m'

echo -e "${BLUE}═══════════════════════════════════════════════════════${NC}"
echo -e "${BLUE}   📜 Event Sourcing Pattern Demo${NC}"
echo -e "${BLUE}═══════════════════════════════════════════════════════${NC}"
echo ""

cd "$(dirname "$0")/../.."

echo -e "${YELLOW}Building event sourcing demo...${NC}"
echo ""

cat > /tmp/event_sourcing.rs << 'EOF'
use rhizo_crypt_core::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("═══════════════════════════════════════════════════════");
    println!("  Event Sourcing: Rebuild State from Event Stream");
    println!("═══════════════════════════════════════════════════════\n");
    
    let config = RhizoCryptConfig::default();
    let mut primal = RhizoCrypt::new(config);
    primal.start().await?;
    
    // Create session for event stream
    let session = SessionBuilder::new(SessionType::General)
        .with_name("event-sourcing-demo")
        .build();
    let session_id = primal.create_session(session).await?;
    
    println!("📝 Recording Event Stream:");
    println!("   (Each event is a vertex in the DAG)\n");
    
    // Event 1: User account created
    let event1 = VertexBuilder::new(EventType::SessionStart)
        .with_metadata("event_type", "AccountCreated")
        .with_metadata("user_id", "alice")
        .with_metadata("email", "alice@example.com")
        .build();
    let v1 = primal.append_vertex(session_id, event1).await?;
    println!("   ✓ Event 1: AccountCreated (user=alice)");
    
    // Event 2: Profile updated
    let event2 = VertexBuilder::new(EventType::DataUpdate { schema: None })
        .with_parent(v1)
        .with_metadata("event_type", "ProfileUpdated")
        .with_metadata("user_id", "alice")
        .with_metadata("field", "display_name")
        .with_metadata("value", "Alice Wonderland")
        .build();
    let v2 = primal.append_vertex(session_id, event2).await?;
    println!("   ✓ Event 2: ProfileUpdated (display_name=Alice Wonderland)");
    
    // Event 3: Settings changed
    let event3 = VertexBuilder::new(EventType::DataUpdate { schema: None })
        .with_parent(v2)
        .with_metadata("event_type", "SettingsChanged")
        .with_metadata("user_id", "alice")
        .with_metadata("setting", "theme")
        .with_metadata("value", "dark")
        .build();
    let v3 = primal.append_vertex(session_id, event3).await?;
    println!("   ✓ Event 3: SettingsChanged (theme=dark)");
    
    // Event 4: Post created
    let event4 = VertexBuilder::new(EventType::DataCreate { schema: None })
        .with_parent(v3)
        .with_metadata("event_type", "PostCreated")
        .with_metadata("user_id", "alice")
        .with_metadata("post_id", "post-001")
        .with_metadata("title", "Hello rhizoCrypt!")
        .build();
    let _v4 = primal.append_vertex(session_id, event4).await?;
    println!("   ✓ Event 4: PostCreated (post-001: Hello rhizoCrypt!)");
    
    // Event 5: Post edited
    let event5 = VertexBuilder::new(EventType::DataUpdate { schema: None })
        .with_parent(_v4)
        .with_metadata("event_type", "PostEdited")
        .with_metadata("user_id", "alice")
        .with_metadata("post_id", "post-001")
        .with_metadata("new_title", "Hello rhizoCrypt & Event Sourcing!")
        .build();
    let _v5 = primal.append_vertex(session_id, event5).await?;
    println!("   ✓ Event 5: PostEdited (post-001: Updated title)\n");
    
    // Rebuild state from events
    println!("🔄 Rebuilding Current State from Event Stream:");
    println!("");
    
    let ancestors = primal.get_ancestors(session_id).await?;
    println!("   📊 Total Events: {}", ancestors.len());
    println!("   📜 Event Stream stored in DAG");
    println!("   ✓ Can rebuild state at any point in time");
    println!("   ✓ Full audit trail preserved");
    println!("   ✓ Time-travel debugging enabled\n");
    
    // Resolve session
    primal.resolve_session(session_id, ResolutionOutcome::Commit).await?;
    
    println!("═══════════════════════════════════════════════════════");
    println!("  🎓 Event Sourcing Benefits:");
    println!("═══════════════════════════════════════════════════════");
    println!("  • Full audit trail - every state change recorded");
    println!("  • Time-travel - rebuild state at any point");
    println!("  • Debugging - trace cause of current state");
    println!("  • Compliance - immutable event log");
    println!("  • Flexibility - derive new projections anytime");
    println!("═══════════════════════════════════════════════════════\n");
    
    Ok(())
}
EOF

echo -e "${GREEN}▶ Running event sourcing demo...${NC}"
echo ""

rustc --edition 2024 /tmp/event_sourcing.rs \
    -L ../../target/release/deps \
    --extern rhizo_crypt_core=../../target/release/librhizo_crypt_core.rlib \
    --extern tokio=../../target/release/deps/libtokio-*.rlib \
    -o /tmp/event_sourcing 2>&1 | grep -v "warning" || true

/tmp/event_sourcing

echo ""
echo -e "${BLUE}═══════════════════════════════════════════════════════${NC}"
echo -e "${GREEN}✅ Event sourcing demo complete!${NC}"
echo ""
echo -e "${YELLOW}📚 What you learned:${NC}"
echo "  • DAG naturally supports event sourcing"
echo "  • Each vertex is an immutable event"
echo "  • Parent links create temporal ordering"
echo "  • Full audit trail by design"
echo "  • Time-travel debugging enabled"
echo ""
echo -e "${YELLOW}▶ Next demo:${NC} ./demo-capability-discovery.sh"
echo ""

rm -f /tmp/event_sourcing.rs /tmp/event_sourcing
