#!/bin/bash
#
# 🎵 Start Songbird Rendezvous Server
#
# Starts the REAL songbird-rendezvous binary from Phase 1
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
║        🎵 Songbird Rendezvous - Live Integration 🎵      ║
║                                                           ║
║  Starting REAL Phase 1 binary (NO MOCKS)                 ║
╚══════════════════════════════════════════════════════════╝
EOF
echo -e "${NC}"

# Get paths (portable)
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/../../.." && pwd)"
BINS_DIR="${PRIMAL_BINS_DIR:-$REPO_ROOT/../../primalBins}"
SONGBIRD_BIN="${SONGBIRD_BIN:-$BINS_DIR/songbird}"

# Verify binary exists
if [ ! -f "$SONGBIRD_BIN" ]; then
    error "Songbird binary not found at: $SONGBIRD_BIN"
    echo ""
    info "Expected location: $SONGBIRD_BIN"
    info "Please ensure Phase 1 binaries are built and copied to bins/"
    echo ""
    exit 1
fi

success "Found Songbird binary: $SONGBIRD_BIN"

# Check if already running
if lsof -Pi :8888 -sTCP:LISTEN -t >/dev/null 2>&1; then
    warn "Songbird rendezvous already running on port 8888"
    echo ""
    info "To stop existing instance:"
    echo "  pkill -f songbird"
    echo ""
    info "To continue with existing instance:"
    echo "  Press Ctrl+C to cancel, or wait 5 seconds to kill and restart..."
    sleep 5
    pkill -f songbird || true
    sleep 2
fi

# Configuration
export SONGBIRD_PORT=8888  # Real port (discovered from logs)
export SONGBIRD_LOG_LEVEL=info
export RUST_LOG=songbird=info

echo ""
log "Configuration:"
echo "  Port: $SONGBIRD_PORT (HTTP/REST API)"
echo "  Log Level: $SONGBIRD_LOG_LEVEL"
echo "  Binary: $SONGBIRD_BIN"
echo ""

# Create log directory
LOG_DIR="$SCRIPT_DIR/logs"
mkdir -p "$LOG_DIR"
LOG_FILE="$LOG_DIR/songbird-rendezvous.log"

log "Starting Songbird tower (rendezvous mode)..."
echo ""

# Start tower in background
"$SONGBIRD_BIN" tower start --port $SONGBIRD_PORT > "$LOG_FILE" 2>&1 &
SONGBIRD_PID=$!

echo "$SONGBIRD_PID" > "$SCRIPT_DIR/.songbird.pid"

log "Songbird PID: $SONGBIRD_PID"
log "Log file: $LOG_FILE"
echo ""

# Wait for startup
log "Waiting for Songbird to be ready..."
ATTEMPTS=0
MAX_ATTEMPTS=30

while [ $ATTEMPTS -lt $MAX_ATTEMPTS ]; do
    if lsof -Pi :8888 -sTCP:LISTEN -t >/dev/null 2>&1; then
        echo ""
        success "Songbird rendezvous is ready!"
        echo ""
        info "Listening on: 0.0.0.0:8888 (HTTP/REST API)"
        info "Status: RUNNING"
        info "PID: $SONGBIRD_PID"
        echo ""
        info "API Endpoints:"
        echo "  POST http://localhost:8888/api/v1/register"
        echo "  POST http://localhost:8888/api/v1/query"
        echo "  POST http://localhost:8888/api/v1/connect"
        echo "  WS   ws://localhost:8888/ws/:session_id"
        echo ""
        
        success "You can now run the demos:"
        echo "  ./demo-register.sh    - Register rhizoCrypt"
        echo "  ./demo-discover.sh    - Discover other primals"
        echo "  ./demo-health.sh      - Health monitoring"
        echo ""
        
        info "To stop Songbird:"
        echo "  ./stop-songbird.sh"
        echo ""
        
        info "To view logs:"
        echo "  tail -f $LOG_FILE"
        echo ""
        
        exit 0
    fi
    
    # Check if process died
    if ! kill -0 $SONGBIRD_PID 2>/dev/null; then
        echo ""
        error "Songbird process died during startup!"
        echo ""
        error "Last 20 lines of log:"
        tail -20 "$LOG_FILE"
        echo ""
        exit 1
    fi
    
    echo -n "."
    sleep 1
    ATTEMPTS=$((ATTEMPTS + 1))
done

echo ""
error "Songbird failed to start after ${MAX_ATTEMPTS} seconds"
echo ""
error "Last 20 lines of log:"
tail -20 "$LOG_FILE"
echo ""

# Kill the process
kill $SONGBIRD_PID 2>/dev/null || true

exit 1

