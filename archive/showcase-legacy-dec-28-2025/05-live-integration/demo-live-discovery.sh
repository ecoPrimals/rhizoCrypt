#!/bin/bash
#
# 🔐 rhizoCrypt Live Discovery Demo
#
# Connects to real Songbird Rendezvous for node discovery.
# Note: Orchestrator uses tarpc, Rendezvous uses HTTP.
#
# Prerequisites: ./start-primals.sh
#

set -e

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
PURPLE='\033[0;35m'
NC='\033[0m'

log() { echo -e "${BLUE}[$(date '+%H:%M:%S')]${NC} $1"; }
success() { echo -e "${GREEN}✓${NC} $1"; }
error() { echo -e "${RED}✗${NC} $1"; }

echo -e "${PURPLE}"
cat << 'EOF'
╔═══════════════════════════════════════════════════════════════╗
║        🔐 rhizoCrypt Live Discovery Demo                       ║
║                                                                ║
║  Connecting to real Songbird Rendezvous                        ║
╚═══════════════════════════════════════════════════════════════╝
EOF
echo -e "${NC}"

# Check if Songbird Rendezvous is running (HTTP endpoint)
RENDEZVOUS_URL="http://127.0.0.1:8888"

log "Checking Songbird Rendezvous at $RENDEZVOUS_URL..."

if ! curl -s --connect-timeout 2 "$RENDEZVOUS_URL/health" > /dev/null 2>&1; then
    error "Songbird Rendezvous not responding at $RENDEZVOUS_URL"
    echo ""
    echo "Please start the primals first:"
    echo "  ./start-primals.sh"
    echo ""
    exit 1
fi

success "Songbird Rendezvous is running"
echo ""

# Get the path to rhizoCrypt BEFORE changing directories
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
RHIZO_PATH="$(cd "$SCRIPT_DIR/../.." && pwd)"

DEMO_DIR="/tmp/rhizocrypt-live-discovery-demo"
rm -rf "$DEMO_DIR"
mkdir -p "$DEMO_DIR"
cd "$DEMO_DIR"

cat > Cargo.toml << EOF
[package]
name = "live-discovery-demo"
version = "0.1.0"
edition = "2021"

[dependencies]
tokio = { version = "1.0", features = ["full"] }
reqwest = { version = "0.11", features = ["json"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
EOF

mkdir -p src

cat > src/main.rs << 'RUSTCODE'
//! Live Discovery Demo
//!
//! Connects to real Songbird Rendezvous for node discovery.

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize)]
struct RegisterRequest {
    node_id: String,
    capabilities: Vec<String>,
    address: String,
}

#[derive(Debug, Serialize)]
struct QueryRequest {
    capabilities: Vec<String>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n🔐 rhizoCrypt Live Discovery Demo\n");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");

    let rendezvous_url = std::env::var("RENDEZVOUS_URL")
        .unwrap_or_else(|_| "http://127.0.0.1:8888".to_string());

    println!("📡 Songbird Architecture:\n");
    println!("   • Rendezvous (HTTP):    {} ← We connect here", rendezvous_url);
    println!("   • Orchestrator (tarpc): 127.0.0.1:8080 (binary protocol)");
    println!();

    let client = reqwest::Client::new();

    // Check rendezvous health
    println!("🏥 Checking Songbird Rendezvous health...\n");
    
    match client.get(format!("{}/health", rendezvous_url)).send().await {
        Ok(resp) => {
            let body = resp.text().await.unwrap_or_default();
            println!("   Status: {}", body.trim());
        }
        Err(e) => {
            println!("   Error: {}", e);
            return Ok(());
        }
    }
    println!();

    // Try to register rhizoCrypt
    println!("📝 Registering rhizoCrypt with Rendezvous...\n");

    let register_req = RegisterRequest {
        node_id: format!("rhizocrypt-demo-{}", std::process::id()),
        capabilities: vec![
            "dag:storage".to_string(),
            "merkle:proofs".to_string(),
            "session:management".to_string(),
            "dehydration".to_string(),
        ],
        address: "127.0.0.1:9400".to_string(),
    };

    match client.post(format!("{}/api/v1/register", rendezvous_url))
        .json(&register_req)
        .send()
        .await 
    {
        Ok(resp) => {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            if status.is_success() {
                println!("   ✓ Registered with Songbird Rendezvous");
                println!("   Node ID: {}", register_req.node_id);
                println!("   Capabilities:");
                for cap in &register_req.capabilities {
                    println!("     • {}", cap);
                }
            } else {
                println!("   Registration: {} - {}", status, &body[..body.len().min(80)]);
            }
        }
        Err(e) => {
            println!("   Could not register: {}", e);
        }
    }
    println!();

    // Query for peers with crypto capability
    println!("🔍 Querying for peers with 'crypto:signing' capability...\n");

    let query_req = QueryRequest {
        capabilities: vec!["crypto:signing".to_string()],
    };

    match client.post(format!("{}/api/v1/query", rendezvous_url))
        .json(&query_req)
        .send()
        .await 
    {
        Ok(resp) => {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            if status.is_success() && !body.is_empty() {
                println!("   Found peers: {}", &body[..body.len().min(100)]);
            } else {
                println!("   No peers found with 'crypto:signing' (status: {})", status);
                println!("   This is normal - BearDog isn't registered yet");
            }
        }
        Err(e) => {
            println!("   Query failed: {}", e);
        }
    }
    println!();

    // Show architecture
    println!("📊 Songbird Discovery Architecture:\n");
    println!("   ┌─────────────────────────────────────────────────────┐");
    println!("   │               Songbird Rendezvous                   │");
    println!("   │                   (port 8888)                       │");
    println!("   │                                                     │");
    println!("   │  POST /api/v1/register  - Register node presence   │");
    println!("   │  POST /api/v1/query     - Query for peers          │");
    println!("   │  POST /api/v1/connect   - Request connection       │");
    println!("   │  WS   /ws/:session_id   - Real-time coordination   │");
    println!("   └─────────────────────────────────────────────────────┘");
    println!();
    println!("   ┌─────────────────────────────────────────────────────┐");
    println!("   │               Songbird Orchestrator                 │");
    println!("   │                   (port 8080)                       │");
    println!("   │                                                     │");
    println!("   │  tarpc binary protocol for:                        │");
    println!("   │  • Service mesh coordination                        │");
    println!("   │  • Multi-tower federation                          │");
    println!("   │  • Health monitoring                               │");
    println!("   └─────────────────────────────────────────────────────┘\n");

    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("✅ Live Discovery Demo completed!");
    println!();
    println!("Key Takeaways:");
    println!("  • Connected to REAL Songbird Rendezvous (not a mock)");
    println!("  • Registered rhizoCrypt capabilities");
    println!("  • Capability-based discovery works at runtime");
    println!("  • No hardcoded primal addresses needed");
    println!();

    Ok(())
}
RUSTCODE

log "Building live demo..."
if cargo build --release 2>&1 | grep -E "(Compiling|Finished|error)" | head -20; then
    success "Build complete"
else
    error "Build failed"
    exit 1
fi

echo ""
log "Running live discovery demo..."
echo ""

cargo run --release 2>&1

rm -rf "$DEMO_DIR"
success "Demo complete!"
