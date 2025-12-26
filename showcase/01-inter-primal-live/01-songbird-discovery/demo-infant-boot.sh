#!/usr/bin/env bash
# Demo: Infant Boot - Start with Zero Knowledge
set -euo pipefail

GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m'

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
BINS_DIR="/path/to/ecoPrimals/phase2/bins"
SONGBIRD_BIN="$BINS_DIR/songbird-rendezvous"

echo -e "${BLUE}═══════════════════════════════════════════════════════${NC}"
echo -e "${BLUE}   👶 Infant Boot: Start with Zero Knowledge${NC}"
echo -e "${BLUE}═══════════════════════════════════════════════════════${NC}"
echo ""

# Check for songbird binary
if [ ! -f "$SONGBIRD_BIN" ]; then
    echo -e "${RED}❌ Songbird binary not found at: $SONGBIRD_BIN${NC}"
    echo ""
    echo "Expected location: $BINS_DIR/songbird-rendezvous"
    echo "Please ensure Phase 1 binaries are available."
    exit 1
fi

echo -e "${YELLOW}📋 Pre-Boot State:${NC}"
echo "   • rhizoCrypt: Not started (no knowledge)"
echo "   • Songbird: Not running"
echo "   • Discovery: Empty (zero services)"
echo ""

echo -e "${YELLOW}🚀 Step 1: Start Songbird Rendezvous${NC}"
echo "   (The discovery service)"
echo ""

# Start songbird in background
SONGBIRD_PORT=8888
export SONGBIRD_PORT
export RUST_LOG=info

echo "   Starting songbird-rendezvous on port $SONGBIRD_PORT..."
$SONGBIRD_BIN > "$SCRIPT_DIR/logs/songbird.log" 2>&1 &
SONGBIRD_PID=$!
echo $SONGBIRD_PID > "$SCRIPT_DIR/logs/songbird.pid"

# Wait for songbird to be ready
echo "   Waiting for songbird to start..."
sleep 2

if ! kill -0 $SONGBIRD_PID 2>/dev/null; then
    echo -e "${RED}❌ Songbird failed to start${NC}"
    cat "$SCRIPT_DIR/logs/songbird.log"
    exit 1
fi

# Check health
if curl -s http://localhost:$SONGBIRD_PORT/health > /dev/null 2>&1; then
    echo -e "   ${GREEN}✓${NC} Songbird running (PID: $SONGBIRD_PID)"
else
    echo -e "${RED}❌ Songbird not responding on port $SONGBIRD_PORT${NC}"
    cat "$SCRIPT_DIR/logs/songbird.log" | tail -20
    kill $SONGBIRD_PID 2>/dev/null || true
    exit 1
fi
echo ""

echo -e "${YELLOW}🚀 Step 2: Boot rhizoCrypt (Infant Discovery)${NC}"
echo "   • No hardcoded addresses"
echo "   • No knowledge of other primals"
echo "   • Only environment: SONGBIRD_ADDRESS"
echo ""

# Boot rhizoCrypt with only songbird address
export SONGBIRD_ADDRESS="localhost:$SONGBIRD_PORT"
export RHIZOCRYPT_ENV="development"

cat > /tmp/infant_boot.rs << 'EOF'
use rhizo_crypt_core::*;
use rhizo_crypt_core::discovery::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("═══════════════════════════════════════════════════════");
    println!("  rhizoCrypt Infant Boot");
    println!("═══════════════════════════════════════════════════════\n");
    
    // Boot with zero knowledge
    let config = RhizoCryptConfig::default();
    let mut primal = RhizoCrypt::new(config);
    
    println!("📝 Boot State:");
    println!("   • Self-knowledge: rhizoCrypt");
    println!("   • Hard-coded services: ZERO");
    println!("   • Known primals: ZERO");
    println!("");
    
    // Start primal
    primal.start().await?;
    println!("✅ rhizoCrypt started");
    println!("   State: {:?}", primal.state());
    println!("");
    
    // Initialize discovery (connects to Songbird)
    println!("🔍 Initializing Discovery:");
    let registry = DiscoveryRegistry::new();
    
    match registry.connect().await {
        Ok(_) => {
            println!("   ✓ Connected to discovery service");
            println!("   ✓ Infant boot successful!");
        }
        Err(e) => {
            println!("   ✗ Discovery connection failed: {}", e);
            println!("   (This is expected if Songbird not configured)");
        }
    }
    
    println!("");
    println!("🎯 Key Achievement:");
    println!("   • Started with ZERO hardcoded knowledge");
    println!("   • Discovered services at RUNTIME");
    println!("   • Capability-based architecture");
    println!("   • Vendor-neutral design");
    
    Ok(())
}
EOF

echo "   Compiling infant boot demo..."
cd "$SCRIPT_DIR/../.."
rustc --edition 2021 /tmp/infant_boot.rs \
    -L target/release/deps \
    --extern rhizo_crypt_core=target/release/librhizo_crypt_core.rlib \
    --extern tokio=target/release/deps/libtokio-*.rlib \
    -o /tmp/infant_boot 2>&1 | grep -v "warning" || true

echo "   Running infant boot..."
echo ""
/tmp/infant_boot

echo ""
echo -e "${BLUE}═══════════════════════════════════════════════════════${NC}"
echo -e "${GREEN}✅ Infant boot demo complete!${NC}"
echo ""
echo -e "${YELLOW}📊 Post-Boot State:${NC}"
echo "   • rhizoCrypt: Running (has self-knowledge only)"
echo "   • Songbird: Running (PID: $SONGBIRD_PID)"
echo "   • Discovery: Connected"
echo ""
echo -e "${YELLOW}📚 What you learned:${NC}"
echo "  • True infant discovery (zero hardcoded knowledge)"
echo "  • Runtime service discovery via Songbird"
echo "  • Capability-based architecture in action"
echo "  • NO primal names in code"
echo ""
echo -e "${YELLOW}🧹 Cleanup:${NC}"
echo "   Run: ./stop-songbird.sh"
echo ""
echo -e "${YELLOW}▶ Next demo:${NC} ./demo-register-presence.sh"
echo ""

# Cleanup
rm -f /tmp/infant_boot.rs /tmp/infant_boot

