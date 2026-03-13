#!/bin/bash
#
# 🐻 Start BearDog - Live Integration
#
# Verifies BearDog binary and prepares HSM environment
# NO MOCKS - This is live integration!
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
║        🐻 BearDog - Live Integration 🐻                  ║
║                                                           ║
║  Starting REAL Phase 1 binary (NO MOCKS)                 ║
╚══════════════════════════════════════════════════════════╝
EOF
echo -e "${NC}"

# Get paths (portable)
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/../../.." && pwd)"
BINS_DIR="${PRIMAL_BINS_DIR:-$REPO_ROOT/../../primalBins}"
BEARDOG_BIN="$BINS_DIR/beardog"

# Verify binary exists
if [ ! -f "$BEARDOG_BIN" ]; then
    error "BearDog binary not found at: $BEARDOG_BIN"
    echo ""
    info "Expected location: ../../../bins/beardog"
    info "Please ensure Phase 1 binaries are built and copied to bins/"
    echo ""
    exit 1
fi

success "Found BearDog binary: $BEARDOG_BIN"

# Make executable
chmod +x "$BEARDOG_BIN" 2>/dev/null || true

# Create log directory
LOG_DIR="$SCRIPT_DIR/logs"
mkdir -p "$LOG_DIR"

echo ""
log "Checking BearDog status..."
echo ""

# Show BearDog status
"$BEARDOG_BIN" status 2>&1 | tee "$LOG_DIR/beardog-status.log"

echo ""
log "Discovering available HSMs..."
echo ""

# Discover HSMs
"$BEARDOG_BIN" hsm discover 2>&1 | tee "$LOG_DIR/hsm-discover.log"

echo ""
success "BearDog environment ready!"
echo ""

info "Binary: $BEARDOG_BIN"
info "Version: $(\"$BEARDOG_BIN\" --version 2>&1 || echo 'v0.9.0')"
info "Logs: $LOG_DIR/"
echo ""

success "You can now run the demos:"
echo "  ./demo-real-signing.sh       - Sign with BearDog HSM"
echo "  ./demo-real-verification.sh  - Verify signatures"
echo "  ./demo-real-multi-agent.sh   - Multi-agent sessions"
echo ""

exit 0

