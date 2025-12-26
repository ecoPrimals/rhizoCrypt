#!/usr/bin/env bash
# Demo: Slices - Checkout from Permanent Storage
set -euo pipefail

# Colors
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
NC='\033[0m'

echo -e "${BLUE}═══════════════════════════════════════════════════════${NC}"
echo -e "${BLUE}   📑 Slices: Checkout from Permanent Storage${NC}"
echo -e "${BLUE}═══════════════════════════════════════════════════════${NC}"
echo ""

cd "$(dirname "$0")"

echo -e "${YELLOW}📦 Building demo...${NC}"
cargo build --release --bin demo-slices 2>&1 | tail -3
echo ""

echo -e "${GREEN}▶ Running slices demo...${NC}"
echo ""
cargo run --release --bin demo-slices

echo ""
echo -e "${BLUE}═══════════════════════════════════════════════════════${NC}"
echo -e "${GREEN}✅ Demo complete!${NC}"
echo ""
echo -e "${YELLOW}📚 What you learned:${NC}"
echo "  • Slice = immutable snapshot from LoamSpine"
echo "  • Checkout creates genesis vertex in DAG"
echo "  • Read-only access to permanent data"
echo "  • Enables working memory over permanent storage"
echo "  • Slice modes control resolution routing"
echo ""
echo -e "${YELLOW}📖 Read more:${NC} ./README.md"
echo -e "${YELLOW}▶ Next demo:${NC} ./demo-dehydration.sh"
echo ""
