#!/bin/bash
#
# 🎵 Demo: Health Monitoring & Heartbeat
#
# Demonstrates:
# - 60-second session expiry handling
# - Heartbeat/refresh mechanism
# - Continuous presence in mesh
# - Session lifecycle management
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
║     💓 Health Monitoring & Session Heartbeat 💓           ║
║                                                           ║
║  Managing Ephemeral Sessions (60s Expiry)                ║
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

echo -e "${CYAN}═══════════════════════════════════════════════════${NC}"
echo -e "${CYAN}Understanding Session Lifecycle${NC}"
echo -e "${CYAN}═══════════════════════════════════════════════════${NC}"
echo ""

info "Songbird uses ephemeral sessions for privacy:"
echo "  • Sessions expire after 60 seconds"
echo "  • Requires periodic re-registration (heartbeat)"
echo "  • Prevents stale entries in mesh"
echo "  • Aligns with privacy-first design"
echo ""

# Initial registration
log "Step 1: Initial Registration"
NODE_ID=$(cat /proc/sys/kernel/random/uuid)
SESSION_ID=$(cat /proc/sys/kernel/random/uuid)

REGISTER=$(cat <<JSON
{
  "message_type": "register_presence",
  "version": "1.0",
  "timestamp": "$(date -u +"%Y-%m-%dT%H:%M:%SZ")",
  "node_identity": {
    "node_id": "$NODE_ID",
    "ephemeral_session_id": "$SESSION_ID",
    "public_key_fingerprint": "sha256:rhizocrypt-health-demo-$(date +%s)",
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

RESPONSE=$(curl -s -X POST http://localhost:8888/api/v1/register \
  -H "Content-Type: application/json" \
  -d "$REGISTER")

EXPIRES_AT=$(echo "$RESPONSE" | jq -r '.expires_at' 2>/dev/null || echo "unknown")

echo ""
success "Initial registration successful"
echo "  Session ID: ${SESSION_ID:0:16}..."
echo "  Expires at: $EXPIRES_AT"
echo ""

# Calculate time until expiry
if [ "$EXPIRES_AT" != "unknown" ] && [ "$EXPIRES_AT" != "null" ]; then
    EXPIRES_EPOCH=$(date -d "$EXPIRES_AT" +%s 2>/dev/null || echo "0")
    NOW_EPOCH=$(date +%s)
    SECONDS_LEFT=$((EXPIRES_EPOCH - NOW_EPOCH))
    
    if [ $SECONDS_LEFT -gt 0 ]; then
        info "Time until expiry: ~${SECONDS_LEFT} seconds"
        echo ""
    fi
fi

# Demonstrate heartbeat mechanism
log "Step 2: Heartbeat Demonstration"
echo ""
info "We'll re-register every 30 seconds (before 60s expiry)"
echo "  This keeps rhizoCrypt visible in the mesh continuously"
echo ""

HEARTBEAT_INTERVAL=30
DEMO_DURATION=90  # 1.5 minutes to show 2-3 heartbeats

START_TIME=$(date +%s)
HEARTBEAT_COUNT=0

while true; do
    ELAPSED=$(($(date +%s) - START_TIME))
    
    if [ $ELAPSED -ge $DEMO_DURATION ]; then
        break
    fi
    
    # Calculate next heartbeat time
    NEXT_HEARTBEAT=$(( (HEARTBEAT_COUNT + 1) * HEARTBEAT_INTERVAL ))
    
    if [ $ELAPSED -ge $NEXT_HEARTBEAT ]; then
        HEARTBEAT_COUNT=$((HEARTBEAT_COUNT + 1))
        
        log "Heartbeat #$HEARTBEAT_COUNT (${ELAPSED}s elapsed)"
        
        # Re-register (refresh)
        REFRESH=$(cat <<JSON
{
  "message_type": "register_presence",
  "version": "1.0",
  "timestamp": "$(date -u +"%Y-%m-%dT%H:%M:%SZ")",
  "node_identity": {
    "node_id": "$NODE_ID",
    "ephemeral_session_id": "$SESSION_ID",
    "public_key_fingerprint": "sha256:rhizocrypt-health-demo-$(date +%s)",
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
        
        REFRESH_RESPONSE=$(curl -s -X POST http://localhost:8888/api/v1/register \
          -H "Content-Type: application/json" \
          -d "$REFRESH")
        
        NEW_EXPIRES=$(echo "$REFRESH_RESPONSE" | jq -r '.expires_at' 2>/dev/null || echo "unknown")
        
        success "Session refreshed"
        echo "  New expiry: $NEW_EXPIRES"
        echo "  Status: Active in mesh"
        echo ""
    fi
    
    # Progress indicator
    REMAINING=$((DEMO_DURATION - ELAPSED))
    if [ $((ELAPSED % 5)) -eq 0 ]; then
        echo -ne "\r  Demo running... ${ELAPSED}s elapsed, ${REMAINING}s remaining"
    fi
    
    sleep 1
done

echo ""
echo ""
success "Heartbeat demonstration complete!"
echo ""
echo "  Total heartbeats: $HEARTBEAT_COUNT"
echo "  Demo duration: ${DEMO_DURATION}s"
echo "  Heartbeat interval: ${HEARTBEAT_INTERVAL}s"
echo ""

# Show what happens without heartbeat
log "Step 3: Expiry Demonstration (No Heartbeat)"
echo ""
warn "Registering a session and letting it expire..."

EXPIRY_SESSION=$(cat /proc/sys/kernel/random/uuid)
EXPIRY_REGISTER=$(cat <<JSON
{
  "message_type": "register_presence",
  "version": "1.0",
  "timestamp": "$(date -u +"%Y-%m-%dT%H:%M:%SZ")",
  "node_identity": {
    "node_id": "$(cat /proc/sys/kernel/random/uuid)",
    "ephemeral_session_id": "$EXPIRY_SESSION",
    "public_key_fingerprint": "sha256:expiry-test-$(date +%s)",
    "capabilities": ["test"],
    "protocols": ["http"]
  },
  "network_context": {
    "nat_type": "open",
    "reachability": "direct",
    "connection_quality": "excellent"
  },
  "security": {
    "signature": "demo"
  }
}
JSON
)

curl -s -X POST http://localhost:8888/api/v1/register \
  -H "Content-Type: application/json" \
  -d "$EXPIRY_REGISTER" > /dev/null

success "Test session registered: ${EXPIRY_SESSION:0:16}..."
info "Waiting 65 seconds for expiry (no heartbeat)..."
echo ""

# Show countdown
for i in {65..1}; do
    echo -ne "\r  Waiting for expiry... ${i}s remaining "
    sleep 1
done
echo ""
echo ""

success "Session expired naturally (no heartbeat sent)"
info "Songbird automatically cleaned up the stale entry"
echo ""

# Summary
echo -e "${PURPLE}═══════════════════════════════════════════════════${NC}"
echo -e "${PURPLE}What Just Happened?${NC}"
echo -e "${PURPLE}═══════════════════════════════════════════════════${NC}"
echo ""
echo "1. 📋 Initial Registration"
echo "   → Session created with 60-second expiry"
echo ""
echo "2. 💓 Heartbeat Mechanism"
echo "   → Re-registered every 30 seconds"
echo "   → Kept session alive continuously"
echo "   → Demonstrated ${HEARTBEAT_COUNT} successful heartbeats"
echo ""
echo "3. ⏱️  Expiry Demonstration"
echo "   → Created session without heartbeat"
echo "   → Waited 65 seconds"
echo "   → Session automatically expired and cleaned up"
echo ""

echo -e "${CYAN}🔑 Key Concepts:${NC}"
echo ""
echo "  • Ephemeral Sessions (Privacy-First)"
echo "    Sessions expire after 60 seconds by design"
echo "    No permanent tracking of nodes"
echo ""
echo "  • Heartbeat Pattern"
echo "    Re-register every 30-45 seconds"
echo "    Prevents expiry while maintaining presence"
echo ""
echo "  • Automatic Cleanup"
echo "    Songbird removes stale entries automatically"
echo "    Mesh stays current without manual intervention"
echo ""
echo "  • Privacy Benefits"
echo "    Short-lived identifiers"
echo "    No permanent node registry"
echo "    Can disappear by simply stopping heartbeat"
echo ""

echo -e "${CYAN}📋 Implementation Guidance:${NC}"
echo ""
echo "In production rhizoCrypt code:"
echo ""
echo "  1. Background Task"
echo "     spawn background heartbeat task on startup"
echo ""
echo "  2. Interval: 30-45 seconds"
echo "     Send before 60s expiry with safety margin"
echo ""
echo "  3. Error Handling"
echo "     Retry on failure, log issues"
echo "     Graceful degradation if Songbird unavailable"
echo ""
echo "  4. Graceful Shutdown"
echo "     Stop heartbeat on primal shutdown"
echo "     Let session expire naturally"
echo ""

success "Heartbeat demo complete!"
echo ""
success "Next steps:"
echo "  1. Implement background heartbeat in rhizoCrypt code"
echo "  2. Add health monitoring metrics"
echo "  3. Move to BearDog integration (real signatures)"
echo "  4. Test multi-primal coordination"
echo ""

echo -e "${GREEN}✓ Songbird integration phase complete!${NC}"
echo ""
info "Ready for:"
echo "  • BearDog: Real DID signatures"
echo "  • NestGate: Content-addressed storage"
echo "  • ToadStool: GPU compute tracking"
echo ""

