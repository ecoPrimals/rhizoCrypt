#!/bin/bash
#
# 🎵 Demo: Discover Other Primals via Songbird
#
# Demonstrates capability-based discovery:
# - Query Songbird for primals by capability
# - Discover services at runtime (no hardcoding)
# - Show dynamic endpoint resolution
#

set -e

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
PURPLE='\033[0;35m'
CYAN='\033[0;36m'
NC='\033[0m'

# Logging
log() { echo -e "${BLUE}[$(date '+%H:%M:%S')]${NC} $1"; }
success() { echo -e "${GREEN}✓${NC} $1"; }
error() { echo -e "${RED}✗${NC} $1"; }
info() { echo -e "${CYAN}ℹ${NC} $1"; }
warn() { echo -e "${YELLOW}⚠${NC} $1"; }

# Banner
echo -e "${PURPLE}"
cat << 'EOF'
╔══════════════════════════════════════════════════════════╗
║     🔍 Discover Primals via Songbird Mesh 🔍             ║
║                                                           ║
║  Capability-Based Discovery (NO HARDCODED ADDRESSES)     ║
╚══════════════════════════════════════════════════════════╝
EOF
echo -e "${NC}"

# Check if Songbird is running
if ! lsof -Pi :8888 -sTCP:LISTEN -t >/dev/null 2>&1; then
    error "Songbird rendezvous not running on port 8888"
    echo ""
    info "Start it first:"
    echo "  ./start-songbird.sh"
    echo ""
    exit 1
fi

success "Songbird rendezvous detected on port 8888"
echo ""

# First, register ourselves so we're in the mesh
SESSION_ID=$(cat /proc/sys/kernel/random/uuid)
TIMESTAMP=$(date -u +"%Y-%m-%dT%H:%M:%SZ")

log "Step 1: Registering rhizoCrypt in mesh..."
REGISTER_PAYLOAD=$(cat <<JSON
{
  "message_type": "register_presence",
  "version": "1.0",
  "timestamp": "$TIMESTAMP",
  "node_identity": {
    "node_id": "$(cat /proc/sys/kernel/random/uuid)",
    "ephemeral_session_id": "$SESSION_ID",
    "public_key_fingerprint": "sha256:rhizocrypt-demo-$(date +%s)",
    "capabilities": ["dag_engine", "merkle_proofs", "ephemeral_sessions"],
    "protocols": ["tarpc", "http"]
  },
  "network_context": {
    "nat_type": "open",
    "reachability": "direct",
    "connection_quality": "excellent"
  },
  "security": {
    "signature": "demo-mode"
  }
}
JSON
)

curl -s -X POST http://localhost:8888/api/v1/register \
  -H "Content-Type: application/json" \
  -d "$REGISTER_PAYLOAD" > /dev/null

success "rhizoCrypt registered (Session: ${SESSION_ID:0:8}...)"
echo ""

# Now discover by different capabilities
echo -e "${CYAN}═══════════════════════════════════════════════════${NC}"
echo -e "${CYAN}Discovery Queries (Capability-Based)${NC}"
echo -e "${CYAN}═══════════════════════════════════════════════════${NC}"
echo ""

# Query 1: Find DAG engines
log "Query 1: Find primals with 'dag_engine' capability"
QUERY1=$(cat <<JSON
{
  "message_type": "query_peers",
  "version": "1.0",
  "timestamp": "$(date -u +"%Y-%m-%dT%H:%M:%SZ")",
  "requester": {
    "session_id": "$SESSION_ID",
    "signature": "demo"
  },
  "query": {
    "capabilities_required": ["dag_engine"],
    "capabilities_optional": [],
    "exclude_node_ids": [],
    "max_results": 10
  }
}
JSON
)

RESPONSE1=$(curl -s -X POST http://localhost:8888/api/v1/query \
  -H "Content-Type: application/json" \
  -d "$QUERY1")

echo ""
echo -e "${CYAN}Results (DAG engines):${NC}"
echo "$RESPONSE1" | jq '.' 2>/dev/null || echo "$RESPONSE1"
echo ""

PEER_COUNT=$(echo "$RESPONSE1" | jq '.peers | length' 2>/dev/null || echo "0")
if [ "$PEER_COUNT" -gt "0" ]; then
    success "Found $PEER_COUNT peer(s) with DAG capability"
    echo "$RESPONSE1" | jq '.peers[] | "  • Session: \(.ephemeral_session_id[0:8])... Capabilities: \(.capabilities | join(", "))"' -r 2>/dev/null || true
else
    info "No other DAG engines found (expected - only rhizoCrypt running)"
fi
echo ""

# Query 2: Find signing services (BearDog)
log "Query 2: Find primals with 'signing' capability (BearDog)"
QUERY2=$(cat <<JSON
{
  "message_type": "query_peers",
  "version": "1.0",
  "timestamp": "$(date -u +"%Y-%m-%dT%H:%M:%SZ")",
  "requester": {
    "session_id": "$SESSION_ID",
    "signature": "demo"
  },
  "query": {
    "capabilities_required": ["signing"],
    "capabilities_optional": [],
    "exclude_node_ids": [],
    "max_results": 10
  }
}
JSON
)

RESPONSE2=$(curl -s -X POST http://localhost:8888/api/v1/query \
  -H "Content-Type: application/json" \
  -d "$QUERY2")

echo -e "${CYAN}Results (Signing services):${NC}"
echo "$RESPONSE2" | jq '.' 2>/dev/null || echo "$RESPONSE2"
echo ""

SIGNING_COUNT=$(echo "$RESPONSE2" | jq '.peers | length' 2>/dev/null || echo "0")
if [ "$SIGNING_COUNT" -gt "0" ]; then
    success "Found $SIGNING_COUNT signing service(s)"
    info "BearDog detected! Can add real signatures now."
else
    info "No signing services found (BearDog not running)"
    info "To test with BearDog: cd ../02-beardog-signing && ./start-beardog.sh"
fi
echo ""

# Query 3: Find storage services (NestGate)
log "Query 3: Find primals with 'storage' capability (NestGate)"
QUERY3=$(cat <<JSON
{
  "message_type": "query_peers",
  "version": "1.0",
  "timestamp": "$(date -u +"%Y-%m-%dT%H:%M:%SZ")",
  "requester": {
    "session_id": "$SESSION_ID",
    "signature": "demo"
  },
  "query": {
    "capabilities_required": ["storage"],
    "capabilities_optional": [],
    "exclude_node_ids": [],
    "max_results": 10
  }
}
JSON
)

RESPONSE3=$(curl -s -X POST http://localhost:8888/api/v1/query \
  -H "Content-Type: application/json" \
  -d "$QUERY3")

echo -e "${CYAN}Results (Storage services):${NC}"
echo "$RESPONSE3" | jq '.' 2>/dev/null || echo "$RESPONSE3"
echo ""

STORAGE_COUNT=$(echo "$RESPONSE3" | jq '.peers | length' 2>/dev/null || echo "0")
if [ "$STORAGE_COUNT" -gt "0" ]; then
    success "Found $STORAGE_COUNT storage service(s)"
    info "NestGate detected! Can store payloads now."
else
    info "No storage services found (NestGate not running)"
    info "To test with NestGate: cd ../03-nestgate-storage && ./start-nestgate.sh"
fi
echo ""

# Show what just happened
echo -e "${PURPLE}═══════════════════════════════════════════════════${NC}"
echo -e "${PURPLE}What Just Happened?${NC}"
echo -e "${PURPLE}═══════════════════════════════════════════════════${NC}"
echo ""
echo "1. 📋 Registered rhizoCrypt with Songbird"
echo "   → Declared our capabilities (DAG, Merkle, Sessions)"
echo ""
echo "2. 🔍 Queried for DAG engines"
echo "   → Found: $PEER_COUNT peer(s)"
echo "   → rhizoCrypt discovered itself!"
echo ""
echo "3. 🔍 Queried for signing services (BearDog)"
echo "   → Found: $SIGNING_COUNT service(s)"
if [ "$SIGNING_COUNT" -eq "0" ]; then
    echo "   → Not running yet (expected)"
fi
echo ""
echo "4. 🔍 Queried for storage services (NestGate)"
echo "   → Found: $STORAGE_COUNT service(s)"
if [ "$STORAGE_COUNT" -eq "0" ]; then
    echo "   → Not running yet (expected)"
fi
echo ""

echo -e "${CYAN}🔑 Key Concepts:${NC}"
echo ""
echo "  • Capability-Based Discovery"
echo "    Query: \"Find me primals that can SIGN\""
echo "    NOT: \"Find me the primal named 'BearDog'\""
echo ""
echo "  • No Hardcoded Addresses"
echo "    rhizoCrypt doesn't know where BearDog lives"
echo "    Songbird tells us at runtime"
echo ""
echo "  • Pure Infant Discovery"
echo "    Primals have zero knowledge of each other"
echo "    Discovery happens through mesh coordination"
echo ""
echo "  • Dynamic Endpoint Resolution"
echo "    Services can move, restart, scale"
echo "    Mesh always has current state"
echo ""

echo -e "${GREEN}Success!${NC} Capability-based discovery working!"
echo ""
success "Next steps:"
echo "  1. Run ./demo-health.sh to test heartbeat mechanism"
echo "  2. Start BearDog and see it appear in queries"
echo "  3. Start NestGate and discover storage"
echo "  4. Build multi-primal coordination workflows"
echo ""

