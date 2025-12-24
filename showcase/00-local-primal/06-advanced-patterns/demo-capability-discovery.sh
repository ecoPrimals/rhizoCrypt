#!/bin/bash
#
# 🔐 rhizoCrypt - Capability Discovery Demo
#
# Demonstrates:
# 1. Pure Infant Discovery (no hardcoded primal names)
# 2. Capability-based addressing (what, not who)
# 3. Runtime discovery via Songbird
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
warning() { echo -e "${YELLOW}⚠${NC} $1"; }

# Banner
echo -e "${PURPLE}"
cat << 'EOF'
╔══════════════════════════════════════════════════════════╗
║   🌟 Capability Discovery - Pure Infant Discovery 🌟     ║
║                                                           ║
║  Learn: Discover capabilities, not primals               ║
╚══════════════════════════════════════════════════════════╝
EOF
echo -e "${NC}"

echo ""
echo -e "${CYAN}Note: This demo is conceptual - actual discovery requires Songbird${NC}"
echo ""

# Get paths
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
RHIZO_ROOT="$(cd "$SCRIPT_DIR/../../.." && pwd)"

log "Building rhizoCrypt..."
cd "$RHIZO_ROOT"
cargo build --quiet 2>/dev/null || cargo build

echo ""
log "Creating capability discovery demo..."

# Create temporary Rust program
TEMP_DIR=$(mktemp -d)
cat > "$TEMP_DIR/Cargo.toml" << CARGO_EOF
[package]
name = "capability-discovery-demo"
version = "0.1.0"
edition = "2021"

[dependencies]
rhizo-crypt-core = { path = "$RHIZO_ROOT/crates/rhizo-crypt-core" }
tokio = { version = "1.46", features = ["full"] }
CARGO_EOF

mkdir -p "$TEMP_DIR/src"
cat > "$TEMP_DIR/src/main.rs" << 'RUST_EOF'
use rhizo_crypt_core::{RhizoCrypt, RhizoCryptConfig};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n🌟 Capability Discovery: Pure Infant Discovery...\n");
    
    // Create rhizoCrypt instance
    let config = RhizoCryptConfig::default();
    let rhizo = RhizoCrypt::new(config);
    rhizo.start().await?;
    
    // Demonstrate capability-based thinking
    println!("🚫 Old Way (Hardcoded Primal Names):");
    println!("   ❌ let beardog = connect(\"https://beardog.tower1.example\");");
    println!("   ❌ let signature = beardog.sign(data);");
    println!();
    println!("   Problems:");
    println!("     • Hardcoded primal name (BearDog)");
    println!("     • Hardcoded tower address");
    println!("     • No user sovereignty (can't choose provider)");
    println!("     • No federation (single provider)");
    println!();
    
    println!("✅ New Way (Capability-Based Discovery):");
    println!("   ✓ let signer = discover_capability(\"crypto:signing\").await?;");
    println!("   ✓ let signature = signer.sign(data).await?;");
    println!();
    println!("   Benefits:");
    println!("     • No hardcoded primal names!");
    println!("     • User chooses which tower");
    println!("     • Federation: Multiple providers");
    println!("     • Fallback: Try alternate providers");
    println!();
    
    // Show capability discovery flow
    println!("🔄 Capability Discovery Flow:");
    println!();
    println!("   Step 1: rhizoCrypt needs crypto:signing");
    println!("           ↓");
    println!("   Step 2: Query Songbird: \"Who provides crypto:signing?\"");
    println!("           ↓");
    println!("   Step 3: Songbird responds:");
    println!("           • Tower A: https://beardog.tower-a.example");
    println!("           • Tower B: https://beardog.tower-b.example");
    println!("           • Tower C: https://custom-signer.tower-c.example");
    println!("           ↓");
    println!("   Step 4: rhizoCrypt tries Tower A (user's preference)");
    println!("           ↓");
    println!("   Step 5: Connect and use capability");
    println!();
    
    // Show available capabilities
    println!("📋 Standard Capabilities:");
    println!();
    println!("   crypto:signing");
    println!("     → Sign data, verify signatures (e.g., BearDog)");
    println!();
    println!("   storage:payload");
    println!("     → Store/retrieve large payloads (e.g., NestGate)");
    println!();
    println!("   storage:permanent");
    println!("     → Permanent storage, dehydration (e.g., LoamSpine)");
    println!();
    println!("   compute:orchestration");
    println!("     → Compute workloads, scripts (e.g., ToadStool)");
    println!();
    println!("   discovery:registry");
    println!("     → Capability discovery, federation (e.g., Songbird)");
    println!();
    println!("   provenance:tracking");
    println!("     → Data lineage, attribution (e.g., SweetGrass)");
    println!();
    
    // Show environment variable usage
    println!("🔧 How rhizoCrypt Discovers Capabilities:");
    println!();
    println!("   1. Check environment for capability endpoints:");
    println!("      • RHIZOCRYPT_CRYPTO_SIGNING_ENDPOINT");
    println!("      • RHIZOCRYPT_STORAGE_PAYLOAD_ENDPOINT");
    println!("      • RHIZOCRYPT_STORAGE_PERMANENT_ENDPOINT");
    println!();
    println!("   2. If not set, query Songbird (discovery registry)");
    println!();
    println!("   3. Fallback to legacy env vars (with deprecation warning):");
    println!("      • BEARDOG_ADDRESS → crypto:signing");
    println!("      • NESTGATE_ADDRESS → storage:payload");
    println!("      • LOAMSPINE_ADDRESS → storage:permanent");
    println!();
    
    println!("🎉 Success! You understand capability-based discovery!");
    println!("\n💡 Key Concepts:");
    println!("  • Pure Infant Discovery: No hardcoded primal names");
    println!("  • Capability-Based: Ask for \"what\" not \"who\"");
    println!("  • Runtime Discovery: Find providers at startup");
    println!("  • User Sovereignty: User chooses which tower");
    println!("  • Federation: Multiple providers for same capability");
    
    println!("\n🌟 Sovereignty Benefits:");
    println!("  • User Control: Choose your own towers");
    println!("  • Vendor Independence: Not locked to specific providers");
    println!("  • Privacy: Use trusted towers for sensitive operations");
    println!("  • Resilience: Fallback to alternate providers");
    
    println!("\n🔐 Architecture Principles:");
    println!("  • Primal code has no knowledge of other primals");
    println!("  • rhizoCrypt doesn't know about BearDog, NestGate, etc.");
    println!("  • Discovery happens at runtime via Songbird");
    println!("  • Capabilities are interfaces, not implementations");
    
    println!("\n📖 Real-World Example:");
    println!("   // rhizoCrypt code (no primal names!)");
    println!("   let signer = rhizo.get_capability(\"crypto:signing\").await?;");
    println!("   let signature = signer.sign(session_merkle_root).await?;");
    println!();
    println!("   // User's environment determines provider:");
    println!("   export RHIZOCRYPT_CRYPTO_SIGNING_ENDPOINT=\"https://my-tower.example/beardog\"");
    println!("   // → rhizoCrypt uses user's chosen tower!");
    println!();
    
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
echo "  1. Pure Infant Discovery (no hardcoded primal names)"
echo "  2. Capability-based addressing (what, not who)"
echo "  3. Runtime discovery via Songbird"
echo "  4. User sovereignty (choose your own towers)"
echo ""
info "Congratulations! You've completed all advanced patterns demos."
echo ""
info "Next steps:"
echo "  • Run the full automated tour: cd ../..; ./RUN_ME_FIRST.sh"
echo "  • Explore inter-primal demos: cd ../../01-inter-primal"
echo "  • Build your own rhizoCrypt application!"
echo ""

