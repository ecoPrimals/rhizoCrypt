#!/bin/bash
#
# 🎵 Stop Songbird Rendezvous Server
#

set -e

# Colors
GREEN='\033[0;32m'
BLUE='\033[0;34m'
RED='\033[0;31m'
NC='\033[0m'

log() { echo -e "${BLUE}[$(date '+%H:%M:%S')]${NC} $1"; }
success() { echo -e "${GREEN}✓${NC} $1"; }
error() { echo -e "${RED}✗${NC} $1"; }

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PID_FILE="$SCRIPT_DIR/.songbird.pid"

if [ -f "$PID_FILE" ]; then
    PID=$(cat "$PID_FILE")
    
    if kill -0 "$PID" 2>/dev/null; then
        log "Stopping Songbird (PID: $PID)..."
        kill "$PID"
        sleep 2
        
        if kill -0 "$PID" 2>/dev/null; then
            log "Forcefully stopping..."
            kill -9 "$PID" 2>/dev/null || true
        fi
        
        success "Songbird stopped"
    else
        log "Songbird not running (stale PID file)"
    fi
    
    rm -f "$PID_FILE"
else
    log "No PID file found, checking for running instances..."
    pkill -f songbird-rendezvous || true
    success "Cleanup complete"
fi

echo ""

