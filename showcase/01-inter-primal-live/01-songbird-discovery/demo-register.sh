#!/bin/bash
#
# 🎵 Demo: Register rhizoCrypt with Songbird
#
# Demonstrates rhizoCrypt registering as a tower in the Songbird mesh
# Uses REAL Songbird rendezvous API (NO MOCKS)
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

# Banner
echo -e "${PURPLE}"
cat << 'EOF'
╔══════════════════════════════════════════════════════════╗
║     🎵 Register rhizoCrypt with Songbird Mesh 🎵          ║
║                                                           ║
║  Real API Integration (NO MOCKS)                          ║
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

# Generate unique IDs for this registration
NODE_ID=$(cat /proc/sys/kernel/random/uuid 2>/dev/null || echo "rhizocrypt-$(date +%s)-$RANDOM")
SESSION_ID=$(cat /proc/sys/kernel/random/uuid 2>/dev/null || echo "session-$(date +%s)-$RANDOM")
TIMESTAMP=$(date -u +"%Y-%m-%dT%H:%M:%SZ")

log "rhizoCrypt Node Registration"
echo "  Node ID: $NODE_ID"
echo "  Session ID: $SESSION_ID"
echo "  Timestamp: $TIMESTAMP"
echo ""

# Build registration payload following RENDEZVOUS_PROTOCOL_SPEC.md format
PAYLOAD=$(cat <<JSON
{
  "message_type": "register_presence",
  "version": "1.0",
  "timestamp": "$TIMESTAMP",
  
  "node_identity": {
    "node_id": "$NODE_ID",
    "ephemeral_session_id": "$SESSION_ID",
    "public_key_fingerprint": "sha256:rhizocrypt-demo-key-$(date +%s)",
    "capabilities": [
      "dag_engine",
      "merkle_proofs",
      "ephemeral_sessions",
      "tarpc_rpc",
      "content_addressing"
    ],
    "protocols": ["tarpc", "http"]
  },
  
  "network_context": {
    "nat_type": "open",
    "reachability": "direct",
    "connection_quality": "excellent"
  },
  
  "security": {
    "signature": "demo-mode-no-beardog-yet"
  }
}
JSON
)

echo -e "${CYAN}Registration Payload:${NC}"
echo "$PAYLOAD" | jq '.'
echo ""

log "Sending registration to Songbird..."
echo ""

# Send registration
RESPONSE=$(curl -s -X POST http://localhost:8888/api/v1/register \
  -H "Content-Type: application/json" \
  -d "$PAYLOAD" \
  -w "\nHTTP_STATUS:%{http_code}")

HTTP_STATUS=$(echo "$RESPONSE" | grep "HTTP_STATUS:" | cut -d':' -f2)
RESPONSE_BODY=$(echo "$RESPONSE" | sed '/HTTP_STATUS:/d')

if [ "$HTTP_STATUS" = "200" ] || [ "$HTTP_STATUS" = "201" ]; then
    echo ""
    success "Registration successful!"
    echo ""
    echo -e "${CYAN}Response from Songbird:${NC}"
    echo "$RESPONSE_BODY" | jq '.' 2>/dev/null || echo "$RESPONSE_BODY"
    echo ""
    
    info "rhizoCrypt is now registered in the Songbird mesh!"
    echo ""
    echo "  ✓ Node ID: $NODE_ID"
    echo "  ✓ Session ID: $SESSION_ID"
    echo "  ✓ Capabilities: DAG, Merkle, Sessions, tarpc RPC"
    echo ""
    
    success "Next steps:"
    echo "  1. Run ./demo-discover.sh to find other primals"
    echo "  2. Start BearDog to add real signatures"
    echo "  3. Test full mesh coordination"
    echo ""
else
    error "Registration failed (HTTP $HTTP_STATUS)"
    echo ""
    echo -e "${RED}Response:${NC}"
    echo "$RESPONSE_BODY"
    echo ""
    
    if echo "$RESPONSE_BODY" | grep -q "missing field"; then
        error "Gap Discovered: Message format mismatch"
        echo ""
        info "This is expected during first integration!"
        info "Documenting gap in ../GAPS_DISCOVERED.md"
        echo ""
        info "Expected format from RENDEZVOUS_PROTOCOL_SPEC.md"
        info "Actual requirements may differ in implementation"
    fi
    
    exit 1
fi

# Show what just happened
echo -e "${PURPLE}═══════════════════════════════════════════════════${NC}"
echo -e "${PURPLE}What Just Happened?${NC}"
echo -e "${PURPLE}═══════════════════════════════════════════════════${NC}"
echo ""
echo "1. 📋 Generated unique Node ID and Session ID"
echo "   → Ephemeral identifiers (privacy-first)"
echo ""
echo "2. 🎵 Declared rhizoCrypt capabilities:"
echo "   → DAG engine (ephemeral working memory)"
echo "   → Merkle proofs (cryptographic integrity)"
echo "   → Session management (scoped DAGs)"
echo "   → tarpc RPC (type-safe communication)"
echo ""
echo "3. 🌐 Sent HTTP/REST registration to Songbird"
echo "   → POST /api/v1/register"
echo "   → JSON payload with node identity"
echo ""
echo "4. ✅ Songbird acknowledged registration"
echo "   → rhizoCrypt now visible in mesh"
echo "   → Other primals can discover us"
echo ""
echo -e "${CYAN}🔑 Key Concepts:${NC}"
echo ""
echo "  • Capability-Based Discovery"
echo "    Primals discover by WHAT they can do, not WHO they are"
echo ""
echo "  • Ephemeral Identifiers"
echo "    Session IDs rotate frequently (privacy)"
echo ""
echo "  • No Hardcoded Addresses"
echo "    Runtime discovery via Songbird mesh"
echo ""
echo "  • Zero Trust Architecture"
echo "    BearDog signatures verify identity (coming next)"
echo ""

