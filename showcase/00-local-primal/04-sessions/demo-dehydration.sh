#!/usr/bin/env bash
# Demo: Dehydration - Commit to Permanent Storage
set -euo pipefail

# Colors
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
NC='\033[0m'

echo -e "${BLUE}═══════════════════════════════════════════════════════${NC}"
echo -e "${BLUE}   🌊 Dehydration: Commit to Permanent Storage${NC}"
echo -e "${BLUE}═══════════════════════════════════════════════════════${NC}"
echo ""

cd "$(dirname "$0")"

echo -e "${YELLOW}📦 Building demo...${NC}"
cargo build --release --bin demo-dehydration 2>&1 | tail -3
echo ""

echo -e "${GREEN}▶ Running dehydration demo...${NC}"
echo ""
cargo run --release --bin demo-dehydration

echo ""
echo -e "${BLUE}═══════════════════════════════════════════════════════${NC}"
echo -e "${GREEN}✅ Demo complete!${NC}"
echo ""
echo -e "${YELLOW}📚 What you learned:${NC}"
echo "  • Dehydration commits DAG results to LoamSpine"
echo "  • Only frontier vertices are committed (efficient)"
echo "  • Merkle root provides integrity proof"
echo "  • Attestations track provenance and agent consent"
echo "  • Ephemeral → Permanent workflow preserves privacy"
echo ""
echo -e "${YELLOW}📖 Read more:${NC} ./README.md"
echo -e "${YELLOW}▶ Next level:${NC} cd ../05-performance && cat README.md"
echo ""
