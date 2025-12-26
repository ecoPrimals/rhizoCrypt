#!/usr/bin/env bash
# Demo: Ephemeral vs Persistent Sessions
set -euo pipefail

# Colors
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
NC='\033[0m'

echo -e "${BLUE}═══════════════════════════════════════════════════════${NC}"
echo -e "${BLUE}   💾 Ephemeral vs Persistent Sessions${NC}"
echo -e "${BLUE}═══════════════════════════════════════════════════════${NC}"
echo ""

cd "$(dirname "$0")"

echo -e "${YELLOW}📦 Building demo...${NC}"
cargo build --release --bin demo-ephemeral-persistent 2>&1 | tail -3
echo ""

echo -e "${GREEN}▶ Running ephemeral vs persistent demo...${NC}"
echo ""
cargo run --release --bin demo-ephemeral-persistent

echo ""
echo -e "${BLUE}═══════════════════════════════════════════════════════${NC}"
echo -e "${GREEN}✅ Demo complete!${NC}"
echo ""
echo -e "${YELLOW}📚 What you learned:${NC}"
echo "  • Ephemeral: Fast, in-memory, privacy-first (default)"
echo "  • Persistent: Optionally saved, audit trail, requires consent"
echo "  • Choose ephemeral for temporary computations"
echo "  • Choose persistent when provenance matters"
echo "  • Human dignity: Forget by default, remember by consent"
echo ""
echo -e "${YELLOW}📖 Read more:${NC} ./README.md"
echo -e "${YELLOW}▶ Next demo:${NC} ./demo-slices.sh"
echo ""
