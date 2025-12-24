#!/bin/bash
#
# 🔐 rhizoCrypt Prerequisites Check
#
# Verifies all prerequisites are installed.
#

set -e

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

echo -e "${BLUE}🔐 Checking rhizoCrypt Prerequisites...${NC}"
echo ""

ERRORS=0

# Check Rust
if command -v rustc &> /dev/null; then
    VERSION=$(rustc --version | cut -d' ' -f2)
    echo -e "${GREEN}✓${NC} Rust installed: $VERSION"
else
    echo -e "${RED}✗${NC} Rust not found. Install: https://rustup.rs"
    ERRORS=$((ERRORS + 1))
fi

# Check Cargo
if command -v cargo &> /dev/null; then
    VERSION=$(cargo --version | cut -d' ' -f2)
    echo -e "${GREEN}✓${NC} Cargo installed: $VERSION"
else
    echo -e "${RED}✗${NC} Cargo not found. Install: https://rustup.rs"
    ERRORS=$((ERRORS + 1))
fi

# Check for optional tools
if command -v lsof &> /dev/null; then
    echo -e "${GREEN}✓${NC} lsof installed (for port checking)"
else
    echo -e "${YELLOW}⚠${NC} lsof not found (optional, for port checking)"
fi

if command -v jq &> /dev/null; then
    echo -e "${GREEN}✓${NC} jq installed (for JSON parsing)"
else
    echo -e "${YELLOW}⚠${NC} jq not found (optional, for JSON parsing)"
fi

if command -v curl &> /dev/null; then
    echo -e "${GREEN}✓${NC} curl installed (for HTTP testing)"
else
    echo -e "${YELLOW}⚠${NC} curl not found (optional, for HTTP testing)"
fi

# Check if we can build rhizoCrypt
echo ""
echo -e "${BLUE}Checking rhizoCrypt builds...${NC}"

RHIZO_ROOT=$(cd "$(dirname "$0")/../../.." && pwd)
cd "$RHIZO_ROOT"

if cargo check --workspace 2>/dev/null; then
    echo -e "${GREEN}✓${NC} rhizoCrypt workspace builds"
else
    echo -e "${RED}✗${NC} rhizoCrypt build failed"
    ERRORS=$((ERRORS + 1))
fi

echo ""

if [ $ERRORS -eq 0 ]; then
    echo -e "${GREEN}All prerequisites satisfied!${NC}"
    echo ""
    echo "You can now run: ./QUICK_START.sh"
else
    echo -e "${RED}$ERRORS prerequisite(s) missing.${NC}"
    echo "Please install missing components and try again."
    exit 1
fi

