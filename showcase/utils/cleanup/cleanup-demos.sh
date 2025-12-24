#!/bin/bash
#
# 🔐 rhizoCrypt Demo Cleanup
#
# Removes all temporary demo files.
#

set -e

# Colors
BLUE='\033[0;34m'
GREEN='\033[0;32m'
NC='\033[0m'

echo -e "${BLUE}🔐 Cleaning up rhizoCrypt demos...${NC}"
echo ""

# Clean up temp directories
DEMO_DIRS=(
    "/tmp/rhizocrypt-session-demo"
    "/tmp/rhizocrypt-dag-demo"
    "/tmp/rhizocrypt-merkle-demo"
    "/tmp/rhizocrypt-discovery-demo"
    "/tmp/rhizocrypt-dehydration-demo"
    "/tmp/rhizocrypt-rpc-demo"
)

for dir in "${DEMO_DIRS[@]}"; do
    if [ -d "$dir" ]; then
        rm -rf "$dir"
        echo "  ✓ Removed $dir"
    fi
done

# Clean up any lingering processes
if pgrep -f "rhizocrypt" > /dev/null 2>&1; then
    echo ""
    echo "  Found running rhizoCrypt processes:"
    pgrep -f "rhizocrypt" || true
    echo ""
    read -p "  Kill these processes? (y/n): " choice
    if [ "$choice" = "y" ]; then
        pkill -f "rhizocrypt" || true
        echo "  ✓ Processes killed"
    fi
fi

echo ""
echo -e "${GREEN}Cleanup complete!${NC}"

