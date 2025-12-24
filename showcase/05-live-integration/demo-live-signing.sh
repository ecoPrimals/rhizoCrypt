#!/bin/bash
#
# 🔐 rhizoCrypt Live Signing Demo
#
# Uses real BearDog CLI for DID and signing operations.
#
# Prerequisites: BearDog binary in ../../bins/
#

set -e

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
PURPLE='\033[0;35m'
NC='\033[0m'

log() { echo -e "${BLUE}[$(date '+%H:%M:%S')]${NC} $1"; }
success() { echo -e "${GREEN}✓${NC} $1"; }
warn() { echo -e "${YELLOW}⚠${NC} $1"; }
error() { echo -e "${RED}✗${NC} $1"; }

echo -e "${PURPLE}"
cat << 'EOF'
╔═══════════════════════════════════════════════════════════════╗
║        🔐 rhizoCrypt Live Signing Demo                         ║
║                                                                ║
║  Using real BearDog CLI for crypto operations                  ║
╚═══════════════════════════════════════════════════════════════╝
EOF
echo -e "${NC}"

# Find BearDog binary
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# Go up from showcase/05-live-integration to rhizoCrypt, then to phase2/bins
BIN_DIR="$(cd "$SCRIPT_DIR/../../../bins" 2>/dev/null && pwd)" || BIN_DIR=""
BEARDOG="${BIN_DIR}/beardog"

if [ ! -f "$BEARDOG" ]; then
    error "BearDog binary not found at $BEARDOG"
    echo ""
    echo "Please ensure BearDog is built and placed in ../../bins/"
    exit 1
fi

success "BearDog CLI found at $BEARDOG"
echo ""

# Show BearDog version/help
log "Checking BearDog capabilities..."
echo ""
$BEARDOG --version 2>/dev/null || $BEARDOG version 2>/dev/null || echo "   (version command not available)"
echo ""

log "Available BearDog commands:"
echo ""
$BEARDOG --help 2>&1 | head -30 || $BEARDOG help 2>&1 | head -30 || echo "   (help not available)"
echo ""

# Create a temporary workspace for the demo
DEMO_DIR="/tmp/rhizocrypt-beardog-demo"
rm -rf "$DEMO_DIR"
mkdir -p "$DEMO_DIR"
cd "$DEMO_DIR"

log "Demo workspace: $DEMO_DIR"
echo ""

# Try to create a DID
log "Creating a test DID..."
echo ""

if $BEARDOG key generate --help 2>&1 | grep -q "generate"; then
    # Generate a key
    $BEARDOG key generate --output "$DEMO_DIR/test-key.json" 2>&1 || true
    if [ -f "$DEMO_DIR/test-key.json" ]; then
        success "Key generated"
        cat "$DEMO_DIR/test-key.json" | head -10
    fi
elif $BEARDOG did create --help 2>&1 | grep -q "create"; then
    # Create DID directly
    $BEARDOG did create --output "$DEMO_DIR/test-did.json" 2>&1 || true
    if [ -f "$DEMO_DIR/test-did.json" ]; then
        success "DID created"
        cat "$DEMO_DIR/test-did.json" | head -10
    fi
else
    warn "Could not determine key generation command"
    echo ""
    echo "Available subcommands:"
    $BEARDOG --help 2>&1 | grep -E "^\s+\w+" | head -20 || true
fi
echo ""

# Try to sign some data
log "Signing vertex data..."
echo ""

# Create test data (simulating a vertex hash)
VERTEX_DATA="rhizocrypt-vertex-demo-data-$(date +%s)"
echo "$VERTEX_DATA" > "$DEMO_DIR/vertex-data.txt"
VERTEX_HASH=$(echo -n "$VERTEX_DATA" | sha256sum | cut -d' ' -f1)

echo "   Vertex data: $VERTEX_DATA"
echo "   SHA256 hash: $VERTEX_HASH"
echo ""

if $BEARDOG sign --help 2>&1 | grep -q "sign"; then
    $BEARDOG sign --input "$DEMO_DIR/vertex-data.txt" --output "$DEMO_DIR/signature.bin" 2>&1 || true
    if [ -f "$DEMO_DIR/signature.bin" ]; then
        success "Data signed"
        echo "   Signature: $(xxd -p "$DEMO_DIR/signature.bin" | head -c 64)..."
    fi
elif $BEARDOG key sign --help 2>&1 | grep -q "sign"; then
    $BEARDOG key sign --data "$VERTEX_HASH" 2>&1 || true
else
    warn "Could not determine signing command"
fi
echo ""

# Show integration architecture
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""
log "rhizoCrypt ↔ BearDog Integration Architecture:"
echo ""
echo "   ┌─────────────────────────────────────────────────┐"
echo "   │                  rhizoCrypt                     │"
echo "   │                                                 │"
echo "   │  When creating a vertex:                        │"
echo "   │    1. Build vertex content                      │"
echo "   │    2. Compute Blake3 hash                       │"
echo "   │    3. Call BearDog CLI to sign ─────────┐       │"
echo "   │    4. Attach signature to vertex         │       │"
echo "   │                                          │       │"
echo "   │  When verifying:                         │       │"
echo "   │    1. Extract signature from vertex      │       │"
echo "   │    2. Call BearDog CLI to verify ───────┤       │"
echo "   │    3. Accept/reject vertex               │       │"
echo "   └──────────────────────────────────────────│───────┘"
echo "                                              │"
echo "                                              ▼"
echo "   ┌─────────────────────────────────────────────────┐"
echo "   │                 BearDog CLI                     │"
echo "   │                                                 │"
echo "   │  beardog key generate  → Create keypair         │"
echo "   │  beardog did create    → Create DID             │"
echo "   │  beardog sign          → Sign data              │"
echo "   │  beardog verify        → Verify signature       │"
echo "   │                                                 │"
echo "   │  Supports:                                      │"
echo "   │    • Ed25519, P-256 curves                      │"
echo "   │    • HSM integration                            │"
echo "   │    • DID document management                    │"
echo "   └─────────────────────────────────────────────────┘"
echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""
success "Live BearDog Signing Demo completed!"
echo ""
echo "Key Takeaways:"
echo "  • Used REAL BearDog CLI (not a mock)"
echo "  • BearDog is a CLI tool, not a server"
echo "  • rhizoCrypt can shell out to BearDog for crypto"
echo "  • All sensitive key operations stay in BearDog"
echo ""

# Cleanup
rm -rf "$DEMO_DIR"
