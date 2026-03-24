#!/usr/bin/env bash
# Demo: Register Presence with Songbird
set -euo pipefail

GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m'

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/../../.." && pwd)"
BINS_DIR="${PRIMAL_BINS_DIR:-$REPO_ROOT/../../primalBins}"
SONGBIRD_BIN="${SONGBIRD_BIN:-$BINS_DIR/songbird-rendezvous}"

echo -e "${BLUE}═══════════════════════════════════════════════════════${NC}"
echo -e "${BLUE}   📡 Register Presence with Songbird${NC}"
echo -e "${BLUE}═══════════════════════════════════════════════════════${NC}"
echo ""

# Check if songbird is running
SONGBIRD_PORT=8888
if ! curl -s http://localhost:$SONGBIRD_PORT/health > /dev/null 2>&1; then
    echo -e "${YELLOW}Starting Songbird...${NC}"
    mkdir -p "$SCRIPT_DIR/logs"
    $SONGBIRD_BIN > "$SCRIPT_DIR/logs/songbird.log" 2>&1 &
    SONGBIRD_PID=$!
    echo $SONGBIRD_PID > "$SCRIPT_DIR/logs/songbird.pid"
    sleep 2
    echo -e "${GREEN}✓${NC} Songbird started (PID: $SONGBIRD_PID)"
else
    echo -e "${GREEN}✓${NC} Songbird already running"
fi
echo ""

echo -e "${YELLOW}📝 Registering rhizoCrypt with Discovery Service...${NC}"
echo ""

# Register using HTTP REST API
cat > /tmp/register.json << 'EOF'
{
  "service_id": "rhizocrypt-demo",
  "capabilities": ["ephemeral-dag", "merkle-proofs", "session-management"],
  "address": "localhost:9400",
  "metadata": {
    "primal": "rhizoCrypt",
    "version": "0.14.0",
    "description": "Ephemeral DAG Engine"
  }
}
EOF

echo "Registration payload:"
cat /tmp/register.json | jq '.'
echo ""

RESPONSE=$(curl -s -X POST http://localhost:$SONGBIRD_PORT/api/v1/register \
  -H "Content-Type: application/json" \
  -d @/tmp/register.json)

echo -e "${GREEN}✓${NC} Registration response:"
echo "$RESPONSE" | jq '.'
echo ""

# Extract session ID if present
if echo "$RESPONSE" | jq -e '.session_id' > /dev/null 2>&1; then
    SESSION_ID=$(echo "$RESPONSE" | jq -r '.session_id')
    echo -e "${GREEN}✓${NC} Session ID: $SESSION_ID"
    echo "$SESSION_ID" > "$SCRIPT_DIR/logs/session_id.txt"
fi

# Query to verify registration
echo ""
echo -e "${YELLOW}🔍 Verifying registration...${NC}"

QUERY_RESPONSE=$(curl -s -X POST http://localhost:$SONGBIRD_PORT/api/v1/query \
  -H "Content-Type: application/json" \
  -d '{"capabilities_required": ["ephemeral-dag"], "capabilities_optional": [], "exclude_node_ids": []}')

echo "$QUERY_RESPONSE" | jq '.'
echo ""

# Check if we're in the results
if echo "$QUERY_RESPONSE" | jq -e '.services[] | select(.service_id == "rhizocrypt-demo")' > /dev/null 2>&1; then
    echo -e "${GREEN}✅ rhizoCrypt successfully registered!${NC}"
else
    echo -e "${YELLOW}⚠️  Registration may need time to propagate${NC}"
fi

echo ""
echo -e "${BLUE}═══════════════════════════════════════════════════════${NC}"
echo -e "${GREEN}✅ Registration demo complete!${NC}"
echo ""
echo -e "${YELLOW}📚 What you learned:${NC}"
echo "  • Services register with capabilities (not names)"
echo "  • Registration includes metadata and address"
echo "  • Discovery is capability-based"
echo "  • Session-based presence tracking"
echo ""
echo -e "${YELLOW}⚠️  Note:${NC} Sessions expire after 60 seconds"
echo "   Next demo shows heartbeat mechanism"
echo ""
echo -e "${YELLOW}▶ Next demo:${NC} ./demo-heartbeat.sh"
echo ""

rm -f /tmp/register.json

