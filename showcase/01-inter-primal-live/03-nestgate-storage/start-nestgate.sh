#!/bin/bash
#
# 🏠 Start NestGate - Live Integration
#
# Starts the REAL nestgate service from Phase 1
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
║        🏠 NestGate - Live Integration 🏠                 ║
║                                                           ║
║  Starting REAL Phase 1 binary (NO MOCKS)                 ║
╚══════════════════════════════════════════════════════════╝
EOF
echo -e "${NC}"

# Get paths
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
BINS_DIR="/path/to/ecoPrimals/phase2/bins"
NESTGATE_BIN="$BINS_DIR/nestgate"

# Verify binary exists
if [ ! -f "$NESTGATE_BIN" ]; then
    error "NestGate binary not found at: $NESTGATE_BIN"
    echo ""
    info "Expected location: $BINS_DIR/nestgate"
    info "Please ensure Phase 1 binaries are built and copied to bins/"
    echo ""
    exit 1
fi

success "Found NestGate binary: $NESTGATE_BIN"
chmod +x "$NESTGATE_BIN" 2>/dev/null || true

# Check if already running
NESTGATE_PORT=9500
if lsof -Pi :$NESTGATE_PORT -sTCP:LISTEN -t >/dev/null 2>&1; then
    warn "NestGate already running on port $NESTGATE_PORT"
    echo ""
    info "To stop existing instance:"
    echo "  pkill -f nestgate"
    echo ""
    info "To continue with existing instance:"
    echo "  Press Ctrl+C to cancel, or wait 5 seconds to kill and restart..."
    sleep 5
    pkill -f nestgate || true
    sleep 2
fi

# Configuration
export NESTGATE_API_PORT=$NESTGATE_PORT
export NESTGATE_STORAGE_PATH="/tmp/nestgate-demo"
export NESTGATE_JWT_SECRET="demo-secret-for-rhizocrypt-showcase-not-for-production"
export RUST_LOG=nestgate=info

mkdir -p "$NESTGATE_STORAGE_PATH"

echo ""
log "Configuration:"
echo "  Port: $NESTGATE_API_PORT (HTTP/REST API)"
echo "  Storage: $NESTGATE_STORAGE_PATH"
echo "  Binary: $NESTGATE_BIN"
echo ""

# Create log directory
LOG_DIR="$SCRIPT_DIR/logs"
mkdir -p "$LOG_DIR"
LOG_FILE="$LOG_DIR/nestgate.log"

log "Starting NestGate service..."
echo ""

# Start in background
"$NESTGATE_BIN" service start --port $NESTGATE_API_PORT > "$LOG_FILE" 2>&1 &
NESTGATE_PID=$!

echo "$NESTGATE_PID" > "$SCRIPT_DIR/.nestgate.pid"

log "NestGate PID: $NESTGATE_PID"
log "Log file: $LOG_FILE"
echo ""

# Wait for startup
log "Waiting for NestGate to be ready..."
ATTEMPTS=0
MAX_ATTEMPTS=30

while [ $ATTEMPTS -lt $MAX_ATTEMPTS ]; do
    if lsof -Pi :$NESTGATE_API_PORT -sTCP:LISTEN -t >/dev/null 2>&1; then
        echo ""
        success "NestGate service is ready!"
        echo ""
        info "Listening on: 0.0.0.0:$NESTGATE_API_PORT (HTTP/REST API)"
        info "Status: RUNNING"
        info "PID: $NESTGATE_PID"
        info "Storage: $NESTGATE_STORAGE_PATH"
        echo ""
        info "API Endpoints:"
        echo "  POST http://localhost:$NESTGATE_API_PORT/api/v1/store"
        echo "  GET  http://localhost:$NESTGATE_API_PORT/api/v1/retrieve/:hash"
        echo "  GET  http://localhost:$NESTGATE_API_PORT/health"
        echo ""
        
        success "You can now run the demos:"
        echo "  ./demo-real-storage.sh          - Store payloads"
        echo "  ./demo-real-retrieval.sh        - Retrieve by hash"
        echo "  ./demo-real-content-addressed.sh - Content addressing"
        echo "  ./demo-real-deduplication.sh    - Storage efficiency"
        echo ""
        
        info "To stop NestGate:"
        echo "  ./stop-nestgate.sh"
        echo ""
        
        info "To view logs:"
        echo "  tail -f $LOG_FILE"
        echo ""
        
        exit 0
    fi
    
    # Check if process died
    if ! kill -0 $NESTGATE_PID 2>/dev/null; then
        echo ""
        error "NestGate process died during startup!"
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
error "NestGate failed to start after ${MAX_ATTEMPTS} seconds"
echo ""
error "Last 20 lines of log:"
tail -20 "$LOG_FILE"
echo ""

# Kill the process
kill $NESTGATE_PID 2>/dev/null || true

exit 1

