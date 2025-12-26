#!/usr/bin/env bash
# Demo: Session Lifecycle - Create, Grow, Resolve, Expire
set -euo pipefail

# Colors for output
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo -e "${BLUE}═══════════════════════════════════════════════════════${NC}"
echo -e "${BLUE}   🔄 rhizoCrypt Session Lifecycle Demo${NC}"
echo -e "${BLUE}═══════════════════════════════════════════════════════${NC}"
echo ""

# Get to the right directory
cd "$(dirname "$0")"

echo -e "${YELLOW}📦 Building demo...${NC}"
cargo build --release --bin demo-session-lifecycle 2>&1 | tail -3
echo ""

echo -e "${GREEN}▶ Running session lifecycle demo...${NC}"
echo ""
cargo run --release --bin demo-session-lifecycle

echo ""
echo -e "${BLUE}═══════════════════════════════════════════════════════${NC}"
echo -e "${GREEN}✅ Demo complete!${NC}"
echo ""
echo -e "${YELLOW}📚 What you learned:${NC}"
echo "  • Sessions have distinct lifecycle states"
echo "  • CREATE → GROW → RESOLVE → EXPIRE"
echo "  • Vertices can only be added during GROW phase"
echo "  • Resolution freezes the DAG and computes Merkle root"
echo "  • Expiry discards ephemeral data (privacy by default)"
echo ""
echo -e "${YELLOW}📖 Read more:${NC} ./README.md"
echo -e "${YELLOW}▶ Next demo:${NC} ./demo-ephemeral-persistent.sh"
echo ""
