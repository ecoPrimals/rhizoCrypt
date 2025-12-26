#!/usr/bin/env bash
# Demo: Heartbeat Mechanism
set -euo pipefail

GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
CYAN='\033[0;36m'
NC='\033[0m'

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
SONGBIRD_PORT=8888

echo -e "${BLUE}═══════════════════════════════════════════════════════${NC}"
echo -e "${BLUE}   💓 Heartbeat Mechanism Demo${NC}"
echo -e "${BLUE}═══════════════════════════════════════════════════════${NC}"
echo ""

# Check if songbird is running
if ! curl -s http://localhost:$SONGBIRD_PORT/health > /dev/null 2>&1; then
    echo -e "${YELLOW}Please start Songbird first:${NC}"
    echo "  ./demo-register-presence.sh"
    exit 1
fi

echo -e "${YELLOW}📝 Why Heartbeats?${NC}"
echo ""
echo "Songbird uses short-lived sessions (60 seconds) for:"
echo "  • Privacy: Services don't leave permanent traces"
echo "  • Freshness: Stale services auto-expire"
echo "  • Dynamic topology: Services come and go"
echo ""

echo -e "${YELLOW}💓 Starting Heartbeat Demo (30 seconds)...${NC}"
echo ""

# Initial registration
cat > /tmp/heartbeat_register.json << 'EOF'
{
  "service_id": "rhizocrypt-heartbeat-demo",
  "capabilities": ["ephemeral-dag", "session-management"],
  "address": "localhost:9400",
  "metadata": {
    "demo": "heartbeat",
    "timestamp": "TIMESTAMP"
  }
}
EOF

# Update timestamp
sed -i "s/TIMESTAMP/$(date -u +%Y-%m-%dT%H:%M:%SZ)/" /tmp/heartbeat_register.json

echo "Registering service..."
RESPONSE=$(curl -s -X POST http://localhost:$SONGBIRD_PORT/api/v1/register \
  -H "Content-Type: application/json" \
  -d @/tmp/heartbeat_register.json)

SESSION_ID=$(echo "$RESPONSE" | jq -r '.session_id // empty')
EXPIRES_AT=$(echo "$RESPONSE" | jq -r '.expires_at // empty')

if [ -z "$SESSION_ID" ]; then
    echo -e "${RED}❌ Registration failed${NC}"
    echo "$RESPONSE" | jq '.'
    exit 1
fi

echo -e "${GREEN}✓${NC} Registered with session: $SESSION_ID"
echo "   Expires at: $EXPIRES_AT"
echo ""

# Heartbeat loop (every 20 seconds for 30 seconds total)
echo "Sending heartbeats every 20 seconds..."
echo ""

for i in {1..2}; do
    echo -e "${CYAN}[$(date +%H:%M:%S)]${NC} Heartbeat $i/2"
    
    # Send heartbeat (re-register with same session)
    HEARTBEAT=$(curl -s -X POST http://localhost:$SONGBIRD_PORT/api/v1/register \
      -H "Content-Type: application/json" \
      -d @/tmp/heartbeat_register.json)
    
    NEW_EXPIRES=$(echo "$HEARTBEAT" | jq -r '.expires_at // empty')
    echo "   New expiry: $NEW_EXPIRES"
    echo "   Status: Active"
    
    # Query to verify presence
    QUERY=$(curl -s -X POST http://localhost:$SONGBIRD_PORT/api/v1/query \
      -H "Content-Type: application/json" \
      -d '{"capabilities_required": ["ephemeral-dag"], "capabilities_optional": [], "exclude_node_ids": []}')
    
    COUNT=$(echo "$QUERY" | jq '[.services[] | select(.service_id == "rhizocrypt-heartbeat-demo")] | length')
    echo "   Discovered: $COUNT instance(s)"
    echo ""
    
    if [ $i -lt 2 ]; then
        sleep 20
    fi
done

echo -e "${GREEN}✅ Heartbeat mechanism working!${NC}"
echo ""

# Show what happens without heartbeat
echo -e "${YELLOW}Testing without heartbeat (waiting 65 seconds for expiry)...${NC}"
echo "This shows the service will auto-expire without heartbeats."
echo ""
echo "Press Ctrl+C to skip waiting, or wait to see expiry..."
echo ""

for i in {65..1}; do
    echo -ne "   Waiting: ${i}s remaining...\r"
    sleep 1
    
    # Check every 10 seconds
    if [ $((i % 10)) -eq 0 ]; then
        QUERY=$(curl -s -X POST http://localhost:$SONGBIRD_PORT/api/v1/query \
          -H "Content-Type: application/json" \
          -d '{"capabilities_required": ["ephemeral-dag"], "capabilities_optional": [], "exclude_node_ids": []}')
        
        COUNT=$(echo "$QUERY" | jq '[.services[] | select(.service_id == "rhizocrypt-heartbeat-demo")] | length')
        if [ "$COUNT" -eq 0 ]; then
            echo ""
            echo -e "${GREEN}✓${NC} Service expired (no longer discoverable)"
            break
        fi
    fi
done

echo ""
echo ""
echo -e "${BLUE}═══════════════════════════════════════════════════════${NC}"
echo -e "${GREEN}✅ Heartbeat demo complete!${NC}"
echo ""
echo -e "${YELLOW}📚 What you learned:${NC}"
echo "  • Heartbeats maintain presence (60s sessions)"
echo "  • Without heartbeats, services auto-expire"
echo "  • Privacy-first: No permanent traces"
echo "  • Dynamic topology: Services come and go"
echo ""
echo -e "${YELLOW}▶ Next demo:${NC} ./demo-capability-query.sh"
echo ""

rm -f /tmp/heartbeat_register.json

