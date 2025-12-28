#!/usr/bin/env bash
# Demo: Discover BearDog HSM
set -euo pipefail

GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m'

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
BINS_DIR="/home/strandgate/Development/ecoPrimals/primalBins"
BEARDOG_BIN="$BINS_DIR/beardog"
SONGBIRD_BIN="$BINS_DIR/songbird-rendezvous"

echo -e "${BLUE}═══════════════════════════════════════════════════════${NC}"
echo -e "${BLUE}   🔐 Discover BearDog HSM${NC}"
echo -e "${BLUE}═══════════════════════════════════════════════════════${NC}"
echo ""

# Check binaries
if [ ! -f "$BEARDOG_BIN" ]; then
    echo -e "${RED}❌ BearDog binary not found at: $BEARDOG_BIN${NC}"
    exit 1
fi

if [ ! -f "$SONGBIRD_BIN" ]; then
    echo -e "${RED}❌ Songbird binary not found at: $SONGBIRD_BIN${NC}"
    exit 1
fi

# Start Songbird if not running
SONGBIRD_PORT=8888
if ! curl -s http://localhost:$SONGBIRD_PORT/health > /dev/null 2>&1; then
    echo -e "${YELLOW}Starting Songbird discovery service...${NC}"
    mkdir -p "$SCRIPT_DIR/logs"
    $SONGBIRD_BIN > "$SCRIPT_DIR/logs/songbird.log" 2>&1 &
    SONGBIRD_PID=$!
    echo $SONGBIRD_PID > "$SCRIPT_DIR/logs/songbird.pid"
    sleep 2
    echo -e "${GREEN}✓${NC} Songbird running (PID: $SONGBIRD_PID)"
fi
echo ""

echo -e "${YELLOW}🚀 Starting BearDog HSM...${NC}"
echo ""

# Configure BearDog
export BEARDOG_PORT=9800
export SONGBIRD_ADDRESS="localhost:$SONGBIRD_PORT"
export RUST_LOG=info

# Check if BearDog has CLI help
echo "Checking BearDog capabilities..."
if $BEARDOG_BIN --help > /dev/null 2>&1; then
    echo -e "${GREEN}✓${NC} BearDog binary functional"
else
    echo -e "${YELLOW}⚠️  BearDog may need configuration${NC}"
fi
echo ""

# For this demo, we'll show capability-based discovery concept
# even if BearDog isn't running as a service
echo -e "${YELLOW}📝 Capability-Based Discovery Pattern:${NC}"
echo ""

cat > /tmp/beardog_discovery.rs << 'EOF'
use rhizo_crypt_core::*;
use rhizo_crypt_core::discovery::*;
use rhizo_crypt_core::clients::capabilities::SigningClient;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("═══════════════════════════════════════════════════════");
    println!("  Discovering Signing Services (Capability-Based)");
    println!("═══════════════════════════════════════════════════════\n");
    
    // Initialize discovery
    let registry = DiscoveryRegistry::new();
    
    println!("🔍 Query: Need 'signing' capability");
    println!("   (NOT looking for 'beardog' specifically!)");
    println!("");
    
    // Discover by capability (not by name!)
    match registry.discover_by_capability(Capability::Signing).await {
        Ok(services) => {
            println!("✅ Found {} signing service(s):", services.len());
            for service in services {
                println!("   • {} at {}", service.service_id, service.address);
                println!("     Capabilities: {:?}", service.capabilities);
                if let Some(metadata) = service.metadata {
                    println!("     Metadata: {:?}", metadata);
                }
            }
            println!("");
            
            // Try to create signing client (vendor-neutral)
            match SigningClient::discover(&registry).await {
                Ok(client) => {
                    println!("✅ SigningClient created (vendor-neutral)");
                    println!("   Works with ANY signing provider!");
                    println!("   • BearDog HSM");
                    println!("   • YubiKey");
                    println!("   • Cloud KMS");
                    println!("   • Any provider with 'signing' capability");
                }
                Err(e) => {
                    println!("ℹ️  Could not create client: {}", e);
                    println!("   (This is expected if no signing service registered)");
                }
            }
        }
        Err(e) => {
            println!("ℹ️  No signing services discovered: {}", e);
            println!("");
            println!("📝 How it works when BearDog IS running:");
            println!("   1. BearDog registers with Songbird");
            println!("   2. Advertises 'signing' capability");
            println!("   3. rhizoCrypt discovers via capability");
            println!("   4. Creates vendor-neutral SigningClient");
            println!("   5. Signs vertices without hardcoding");
        }
    }
    
    println!("");
    println!("🎯 Key Principle:");
    println!("   ❌ BAD:  beardog.sign(data)");
    println!("   ✅ GOOD: SigningClient::discover().sign(data)");
    println!("");
    println!("   Benefits:");
    println!("   • Swap HSM providers without code changes");
    println!("   • Multiple signing services (redundancy)");
    println!("   • No vendor lock-in");
    println!("   • Federation-ready");
    
    Ok(())
}
EOF

echo "Compiling discovery demo..."
cd "$SCRIPT_DIR/../.."
rustc --edition 2021 /tmp/beardog_discovery.rs \
    -L target/release/deps \
    --extern rhizo_crypt_core=target/release/librhizo_crypt_core.rlib \
    --extern tokio=target/release/deps/libtokio-*.rlib \
    -o /tmp/beardog_discovery 2>&1 | grep -v "warning" || true

echo "Running discovery demo..."
echo ""
/tmp/beardog_discovery

echo ""
echo -e "${BLUE}═══════════════════════════════════════════════════════${NC}"
echo -e "${GREEN}✅ HSM discovery demo complete!${NC}"
echo ""
echo -e "${YELLOW}📚 What you learned:${NC}"
echo "  • Discover signing services by CAPABILITY"
echo "  • Vendor-neutral SigningClient"
echo "  • Works with ANY HSM provider"
echo "  • No hardcoded primal names"
echo ""
echo -e "${YELLOW}▶ Next demo:${NC} ./demo-sign-vertex.sh"
echo ""

rm -f /tmp/beardog_discovery.rs /tmp/beardog_discovery

