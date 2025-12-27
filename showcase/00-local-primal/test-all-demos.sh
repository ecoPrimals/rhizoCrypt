#!/usr/bin/env bash
# Batch Polish Script for Local Showcase
# Updates all demos to use validated API patterns from RootPulse showcase

set -euo pipefail

GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
NC='\033[0m'

echo -e "${BLUE}╔═══════════════════════════════════════════════════════╗${NC}"
echo -e "${BLUE}║    🔧 Local Showcase Polish — Batch Update          ║${NC}"
echo -e "${BLUE}╚═══════════════════════════════════════════════════════╝${NC}"
echo ""

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR"

# Track results
TOTAL=0
SUCCESS=0
FAILED=0

test_demo() {
    local demo=$1
    echo -e "${YELLOW}Testing:${NC} $demo"
    
    TOTAL=$((TOTAL + 1))
    
    if timeout 60 bash "$demo" > /dev/null 2>&1; then
        echo -e "${GREEN}✓${NC} Works"
        SUCCESS=$((SUCCESS + 1))
        return 0
    else
        echo -e "${YELLOW}⚠${NC}  Needs update"
        FAILED=$((FAILED + 1))
        return 1
    fi
}

echo -e "${BLUE}Phase 1: Testing existing demos...${NC}"
echo ""

# Test Level 0 demos
echo "Level 0: Hello rhizoCrypt"
test_demo "01-hello-rhizocrypt/demo-first-session.sh" || true
test_demo "01-hello-rhizocrypt/demo-first-vertex.sh" || true
test_demo "01-hello-rhizocrypt/demo-query-dag.sh" || true

echo ""
echo "Level 1: DAG Engine"
test_demo "02-dag-engine/demo-genesis.sh" || true
test_demo "02-dag-engine/demo-frontier.sh" || true
test_demo "02-dag-engine/demo-multi-parent.sh" || true

echo ""
echo "Level 2: Merkle Proofs"
test_demo "03-merkle-proofs/demo-content-addressing.sh" || true
test_demo "03-merkle-proofs/demo-merkle-tree.sh" || true

echo ""
echo -e "${BLUE}═══════════════════════════════════════════════════════${NC}"
echo -e "${GREEN}Test Results:${NC}"
echo "  Total demos tested: $TOTAL"
echo "  Working: $SUCCESS"
echo "  Need updates: $FAILED"
echo ""

if [ $SUCCESS -eq $TOTAL ]; then
    echo -e "${GREEN}🎊 All demos working!${NC}"
else
    echo -e "${YELLOW}📝 $FAILED demos need API updates${NC}"
    echo ""
    echo "Recommendation:"
    echo "  1. Review working demos for pattern"
    echo "  2. Update failing demos to match"
    echo "  3. Use SessionBuilder + PrimalLifecycle"
    echo "  4. Remember: get_session() is NOT async"
fi

echo ""

