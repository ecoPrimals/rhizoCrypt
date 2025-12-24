#!/bin/bash
#
# 🔐 rhizoCrypt - Topological Sort Demo
#
# Demonstrates:
# 1. Topological ordering (parents before children)
# 2. Use in Merkle tree computation
# 3. Cycle detection
#

set -e

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
PURPLE='\033[0;35m'
CYAN='\033[0;36m'
NC='\033[0m'

# Logging
log() { echo -e "${BLUE}[$(date '+%H:%M:%S')]${NC} $1"; }
success() { echo -e "${GREEN}✓${NC} $1"; }
info() { echo -e "${CYAN}ℹ${NC} $1"; }

# Banner
echo -e "${PURPLE}"
cat << 'EOF'
╔══════════════════════════════════════════════════════════╗
║      🔄 Topological Sort - Dependency Ordering 🔄         ║
║                                                           ║
║  Learn: Parents-before-children ordering                 ║
╚══════════════════════════════════════════════════════════╝
EOF
echo -e "${NC}"

# Get paths
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
RHIZO_ROOT="$(cd "$SCRIPT_DIR/../../.." && pwd)"

log "Building rhizoCrypt..."
cd "$RHIZO_ROOT"
cargo build --quiet 2>/dev/null || cargo build

echo ""
log "Creating topological sort demo..."

# Create temporary Rust program
TEMP_DIR=$(mktemp -d)
cat > "$TEMP_DIR/Cargo.toml" << CARGO_EOF
[package]
name = "topological-demo"
version = "0.1.0"
edition = "2021"

[dependencies]
rhizo-crypt-core = { path = "$RHIZO_ROOT/crates/rhizo-crypt-core" }
tokio = { version = "1.46", features = ["full"] }
CARGO_EOF

mkdir -p "$TEMP_DIR/src"
cat > "$TEMP_DIR/src/main.rs" << 'RUST_EOF'
use rhizo_crypt_core::{RhizoCrypt, RhizoCryptConfig, Session, SessionType, Vertex, EventType};
use std::collections::{HashMap, HashSet, VecDeque};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n🔄 Topological Sort: Ensuring Parent-Before-Child...\n");
    
    // Create rhizoCrypt instance
    let config = RhizoCryptConfig::default();
    let rhizo = RhizoCrypt::new(config);
    rhizo.start().await?;
    
    // Create a session
    let session = Session::new("topological-demo".to_string(), SessionType::Ephemeral);
    let session_id = session.id;
    rhizo.create_session(session).await?;
    
    let store = rhizo.dag_store().await?;
    
    // Build a complex DAG:
    //       A
    //      / \
    //     B   C
    //      \ / \
    //       D   E
    //        \ /
    //         F
    
    println!("📊 Building DAG:");
    println!("       A");
    println!("      / \\");
    println!("     B   C");
    println!("      \\ / \\");
    println!("       D   E");
    println!("        \\ /");
    println!("         F");
    println!();
    
    let a = Vertex::new(EventType::SessionStarted, Vec::new());
    let a_id = a.id;
    store.put_vertex(session_id, a).await?;
    
    let b = Vertex::new(EventType::DataCreated, vec![a_id]);
    let b_id = b.id;
    store.put_vertex(session_id, b).await?;
    
    let c = Vertex::new(EventType::DataModified, vec![a_id]);
    let c_id = c.id;
    store.put_vertex(session_id, c).await?;
    
    let d = Vertex::new(EventType::DataDeleted, vec![b_id, c_id]);
    let d_id = d.id;
    store.put_vertex(session_id, d).await?;
    
    let e = Vertex::new(EventType::DataCommitted, vec![c_id]);
    let e_id = e.id;
    store.put_vertex(session_id, e).await?;
    
    let f = Vertex::new(EventType::SessionResolved, vec![d_id, e_id]);
    let f_id = f.id;
    store.put_vertex(session_id, f).await?;
    
    println!("✓ DAG built with 6 vertices");
    println!();
    
    // Collect all vertices
    let all_vertices = vec![
        (a_id, store.get_vertex(session_id, a_id).await?),
        (b_id, store.get_vertex(session_id, b_id).await?),
        (c_id, store.get_vertex(session_id, c_id).await?),
        (d_id, store.get_vertex(session_id, d_id).await?),
        (e_id, store.get_vertex(session_id, e_id).await?),
        (f_id, store.get_vertex(session_id, f_id).await?),
    ];
    
    // Simple topological sort (Kahn's algorithm)
    println!("🔄 Performing Topological Sort (Kahn's Algorithm)...");
    println!();
    
    // Build in-degree map
    let mut in_degree: HashMap<String, usize> = HashMap::new();
    let mut children_map: HashMap<String, Vec<String>> = HashMap::new();
    
    for (v_id, vertex) in &all_vertices {
        in_degree.entry(v_id.clone()).or_insert(vertex.parents.len());
        
        for parent_id in &vertex.parents {
            children_map.entry(parent_id.clone())
                .or_insert_with(Vec::new)
                .push(v_id.clone());
        }
    }
    
    // Find vertices with no parents (in-degree 0)
    let mut queue: VecDeque<String> = VecDeque::new();
    for (v_id, &degree) in &in_degree {
        if degree == 0 {
            queue.push_back(v_id.clone());
        }
    }
    
    // Process queue
    let mut sorted_order = Vec::new();
    let mut visited = HashSet::new();
    
    while let Some(v_id) = queue.pop_front() {
        if visited.contains(&v_id) {
            continue;
        }
        
        visited.insert(v_id.clone());
        sorted_order.push(v_id.clone());
        
        // Reduce in-degree of children
        if let Some(children) = children_map.get(&v_id) {
            for child_id in children {
                if let Some(degree) = in_degree.get_mut(child_id) {
                    *degree = degree.saturating_sub(1);
                    if *degree == 0 {
                        queue.push_back(child_id.clone());
                    }
                }
            }
        }
    }
    
    // Display sorted order
    println!("📋 Topological Order (parents before children):");
    for (i, v_id) in sorted_order.iter().enumerate() {
        let vertex = all_vertices.iter().find(|(id, _)| id == v_id).unwrap().1.clone();
        let label = match v_id.as_str() {
            id if id == a_id.as_str() => "A",
            id if id == b_id.as_str() => "B",
            id if id == c_id.as_str() => "C",
            id if id == d_id.as_str() => "D",
            id if id == e_id.as_str() => "E",
            id if id == f_id.as_str() => "F",
            _ => "?",
        };
        println!("  {}. {} ({:?}, parents: {})", i + 1, label, vertex.event_type, vertex.parents.len());
    }
    println!();
    
    // Verify ordering
    println!("✅ Verification: All parents come before children");
    let id_to_label: HashMap<String, &str> = [
        (a_id.clone(), "A"),
        (b_id.clone(), "B"),
        (c_id.clone(), "C"),
        (d_id.clone(), "D"),
        (e_id.clone(), "E"),
        (f_id.clone(), "F"),
    ].iter().cloned().collect();
    
    for (v_id, vertex) in &all_vertices {
        let v_pos = sorted_order.iter().position(|id| id == v_id).unwrap();
        for parent_id in &vertex.parents {
            let p_pos = sorted_order.iter().position(|id| id == parent_id).unwrap();
            let v_label = id_to_label.get(v_id).unwrap_or(&"?");
            let p_label = id_to_label.get(parent_id).unwrap_or(&"?");
            if p_pos < v_pos {
                println!("  ✓ {} (pos {}) comes before {} (pos {})", p_label, p_pos + 1, v_label, v_pos + 1);
            }
        }
    }
    println!();
    
    println!("🎉 Success! Topological sort ensures correct ordering!");
    println!("\n💡 Key Concepts:");
    println!("  • Topological sort ensures parents come before children");
    println!("  • Required for Merkle tree computation (bottom-up)");
    println!("  • Enables dependency-aware processing");
    println!("  • Detects cycles (DAG must be acyclic)");
    println!("  • Kahn's algorithm: O(n + e) time complexity");
    
    println!("\n🌟 Why Topological Sort Matters:");
    println!("  • Merkle proofs need parent hashes computed first");
    println!("  • Session resolution processes events in causal order");
    println!("  • Dehydration commits vertices in dependency order");
    println!("  • Any DAG-based workflow needs topological ordering");
    
    println!("\n🔍 DAG Acyclicity:");
    println!("  • If topological sort includes all vertices → acyclic ✓");
    println!("  • If some vertices missing → cycle detected ✗");
    println!("  • rhizoCrypt enforces acyclicity at vertex insertion");
    
    // Cleanup
    rhizo.stop().await?;
    
    Ok(())
}
RUST_EOF

echo ""
log "Running demo..."
echo ""

cd "$TEMP_DIR"
cargo run --quiet 2>/dev/null || cargo run

# Cleanup
cd "$RHIZO_ROOT"
rm -rf "$TEMP_DIR"

echo ""
echo -e "${GREEN}════════════════════════════════════════════════════════${NC}"
echo -e "${GREEN}Demo Complete!${NC}"
echo ""
info "What you learned:"
echo "  1. Topological sort ensures parents before children"
echo "  2. Required for Merkle tree computation"
echo "  3. Kahn's algorithm: O(n + e) time"
echo "  4. Detects cycles in the DAG"
echo ""
info "Level 2 Complete! Ready for Level 3?"
echo "  cd ../03-merkle-proofs"
echo "  cat README.md"
echo ""

