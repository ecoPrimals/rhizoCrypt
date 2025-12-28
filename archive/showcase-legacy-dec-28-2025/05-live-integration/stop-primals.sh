#!/bin/bash
#
# 🔐 Stop Phase 1 Primals
#

set -e

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
BLUE='\033[0;34m'
PURPLE='\033[0;35m'
NC='\033[0m'

log() { echo -e "${BLUE}[$(date '+%H:%M:%S')]${NC} $1"; }
success() { echo -e "${GREEN}✓${NC} $1"; }

echo -e "${PURPLE}"
cat << 'EOF'
╔═══════════════════════════════════════════════════════════════╗
║          🔐 Stopping Phase 1 Primals                           ║
╚═══════════════════════════════════════════════════════════════╝
EOF
echo -e "${NC}"

PID_DIR="/tmp/rhizocrypt-primals/pids"

stop_service() {
    local name=$1
    local pid_file="$PID_DIR/$name.pid"
    
    if [ -f "$pid_file" ]; then
        local pid=$(cat "$pid_file")
        if kill -0 "$pid" 2>/dev/null; then
            log "Stopping $name (PID $pid)..."
            kill "$pid" 2>/dev/null || true
            sleep 1
            # Force kill if still running
            if kill -0 "$pid" 2>/dev/null; then
                kill -9 "$pid" 2>/dev/null || true
            fi
            success "$name stopped"
        else
            log "$name not running"
        fi
        rm -f "$pid_file"
    else
        log "$name not tracked"
    fi
}

stop_service "nestgate"
stop_service "songbird-orchestrator"
stop_service "songbird-rendezvous"
stop_service "beardog"

echo ""
success "All primals stopped"

