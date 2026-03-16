#!/usr/bin/env bash
# Demo: Infant Discovery - Zero Hardcoding
set -euo pipefail

GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
CYAN='\033[0;36m'
NC='\033[0m'

echo -e "${BLUE}═══════════════════════════════════════════════════════${NC}"
echo -e "${BLUE}   🔍 Infant Discovery: Zero Hardcoding${NC}"
echo -e "${BLUE}═══════════════════════════════════════════════════════${NC}"
echo ""

cd "$(dirname "$0")/../.."

echo -e "${YELLOW}Demonstrating capability-based discovery...${NC}"
echo ""

cat > /tmp/capability_discovery.rs << 'EOF'
use rhizo_crypt_core::*;
use rhizo_crypt_core::discovery::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("═══════════════════════════════════════════════════════");
    println!("  Infant Discovery: Start with Zero Knowledge");
    println!("═══════════════════════════════════════════════════════\n");
    
    println!("📝 Core Principle:");
    println!("   Primals only have SELF-KNOWLEDGE");
    println!("   All other primals discovered at RUNTIME");
    println!("   No hardcoded primal names or addresses\n");
    
    // Initialize rhizoCrypt (knows only itself)
    let config = RhizoCryptConfig::default();
    let mut primal = RhizoCrypt::new(config);
    primal.start().await?;
    
    println!("✅ rhizoCrypt started");
    println!("   • Knows: Self (rhizoCrypt)");
    println!("   • Knows: Own capabilities");
    println!("   • Discovers: Everything else\n");
    
    // Create discovery registry (infant discovery)
    let registry = DiscoveryRegistry::new();
    
    println!("🔍 Capability-Based Discovery:");
    println!("");
    
    // Discover by CAPABILITY, not by name
    println!("   1. Need: Signing service");
    match registry.discover_by_capability(Capability::Signing).await {
        Ok(services) => {
            println!("      ✓ Found {} signing service(s)", services.len());
            for service in services {
                println!("        - {} ({})", service.service_id, service.address);
            }
        }
        Err(_) => {
            println!("      ℹ  No signing services discovered (yet)");
            println!("        (Would discover from Songbird at runtime)");
        }
    }
    println!("");
    
    println!("   2. Need: Storage service");
    match registry.discover_by_capability(Capability::Storage).await {
        Ok(services) => {
            println!("      ✓ Found {} storage service(s)", services.len());
            for service in services {
                println!("        - {} ({})", service.service_id, service.address);
            }
        }
        Err(_) => {
            println!("      ℹ  No storage services discovered (yet)");
            println!("        (Would discover from Songbird at runtime)");
        }
    }
    println!("");
    
    println!("   3. Need: Compute service");
    match registry.discover_by_capability(Capability::Compute).await {
        Ok(services) => {
            println!("      ✓ Found {} compute service(s)", services.len());
            for service in services {
                println!("        - {} ({})", service.service_id, service.address);
            }
        }
        Err(_) => {
            println!("      ℹ  No compute services discovered (yet)");
            println!("        (Would discover from Songbird at runtime)");
        }
    }
    
    println!("");
    println!("═══════════════════════════════════════════════════════");
    println!("  🎯 Zero-Hardcoding Architecture:");
    println!("═══════════════════════════════════════════════════════");
    println!("  ✅ No primal names in code");
    println!("  ✅ No hardcoded addresses or ports");
    println!("  ✅ Capability-based discovery");
    println!("  ✅ Vendor-neutral (works with ANY provider)");
    println!("  ✅ Runtime flexibility");
    println!("");
    println!("  Example: Need signing?");
    println!("    ❌ BAD:  use beardog::sign()");
    println!("    ✅ GOOD: discover_by_capability(Signing)");
    println!("");
    println!("  Benefits:");
    println!("    • Swap providers without code changes");
    println!("    • Multiple providers (redundancy)");
    println!("    • Federation (discover across towers)");
    println!("    • Sovereignty (no vendor lock-in)");
    println!("═══════════════════════════════════════════════════════\n");
    
    Ok(())
}
EOF

echo -e "${GREEN}▶ Running capability discovery demo...${NC}"
echo ""

rustc --edition 2024 /tmp/capability_discovery.rs \
    -L ../../target/release/deps \
    --extern rhizo_crypt_core=../../target/release/librhizo_crypt_core.rlib \
    --extern tokio=../../target/release/deps/libtokio-*.rlib \
    -o /tmp/capability_discovery 2>&1 | grep -v "warning" || true

/tmp/capability_discovery

echo ""
echo -e "${BLUE}═══════════════════════════════════════════════════════${NC}"
echo -e "${GREEN}✅ Capability discovery demo complete!${NC}"
echo ""
echo -e "${YELLOW}📚 What you learned:${NC}"
echo "  • Primals have only self-knowledge"
echo "  • All other primals discovered at runtime"
echo "  • Capability-based (not name-based) discovery"
echo "  • Zero hardcoding = maximum flexibility"
echo "  • Vendor-neutral architecture"
echo ""
echo -e "${CYAN}🎉 Local Showcase Complete!${NC}"
echo -e "${CYAN}   All 22 demos finished!${NC}"
echo ""
echo -e "${YELLOW}▶ Next:${NC} Inter-primal integration with real Phase 1 binaries"
echo -e "${YELLOW}   See:${NC} ../../01-inter-primal-live/README.md"
echo ""

rm -f /tmp/capability_discovery.rs /tmp/capability_discovery
