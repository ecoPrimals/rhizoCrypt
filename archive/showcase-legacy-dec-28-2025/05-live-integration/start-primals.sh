#!/bin/bash
#
# 🔐 Start Phase 1 Primals for rhizoCrypt Integration
#
# Starts Songbird and NestGate (BearDog is a CLI, not a server)
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
║          🔐 Starting Phase 1 Primals                           ║
║                                                                ║
║  Songbird (discovery) • NestGate (storage)                     ║
╚═══════════════════════════════════════════════════════════════╝
EOF
echo -e "${NC}"

# Paths
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
BINS_DIR="$(cd "$SCRIPT_DIR/../../../bins" && pwd)"
LOG_DIR="/tmp/rhizocrypt-primals"
PID_DIR="$LOG_DIR/pids"

mkdir -p "$LOG_DIR" "$PID_DIR"

# Check binaries exist
log "Checking binaries in $BINS_DIR..."

check_binary() {
    if [ -x "$BINS_DIR/$1" ]; then
        success "$1 found"
        return 0
    else
        warn "$1 not found or not executable"
        return 1
    fi
}

HAVE_SONGBIRD=false
HAVE_NESTGATE=false

check_binary "songbird-orchestrator" && HAVE_SONGBIRD=true
check_binary "nestgate" && HAVE_NESTGATE=true

echo ""
echo "Note: BearDog is a CLI tool, not a server."
echo "      Use ./beardog directly for crypto operations."
echo ""

# Check if already running
check_running() {
    local name=$1
    local pid_file="$PID_DIR/$name.pid"
    if [ -f "$pid_file" ]; then
        local pid=$(cat "$pid_file")
        if kill -0 "$pid" 2>/dev/null; then
            warn "$name already running (PID $pid)"
            return 0
        else
            rm -f "$pid_file"
        fi
    fi
    return 1
}

# Start Songbird Rendezvous first
if $HAVE_SONGBIRD && [ -x "$BINS_DIR/songbird-rendezvous" ]; then
    if ! check_running "songbird-rendezvous"; then
        log "Starting Songbird Rendezvous on port 8888..."
        cd "$BINS_DIR"
        # Rendezvous doesn't take port arg, uses 8888 by default
        ./songbird-rendezvous > "$LOG_DIR/songbird-rendezvous.log" 2>&1 &
        echo $! > "$PID_DIR/songbird-rendezvous.pid"
        sleep 2
        if kill -0 $(cat "$PID_DIR/songbird-rendezvous.pid") 2>/dev/null; then
            success "Songbird Rendezvous started (PID $(cat $PID_DIR/songbird-rendezvous.pid))"
        else
            warn "Songbird Rendezvous may have failed - check $LOG_DIR/songbird-rendezvous.log"
        fi
    fi
fi

# Start Songbird Orchestrator
if $HAVE_SONGBIRD; then
    if ! check_running "songbird-orchestrator"; then
        log "Starting Songbird Orchestrator on port 8080..."
        cd "$BINS_DIR"
        ./songbird-orchestrator --port 8080 > "$LOG_DIR/songbird-orchestrator.log" 2>&1 &
        echo $! > "$PID_DIR/songbird-orchestrator.pid"
        sleep 2
        if kill -0 $(cat "$PID_DIR/songbird-orchestrator.pid") 2>/dev/null; then
            success "Songbird Orchestrator started (PID $(cat $PID_DIR/songbird-orchestrator.pid))"
        else
            warn "Songbird Orchestrator may have failed - check $LOG_DIR/songbird-orchestrator.log"
        fi
    fi
fi

# Start NestGate
if $HAVE_NESTGATE; then
    if ! check_running "nestgate"; then
        log "Starting NestGate on port 8092..."
        cd "$BINS_DIR"
        mkdir -p "$LOG_DIR/nestgate-data"
        # Use 'service start' command with --daemon
        ./nestgate service start --port 8092 --daemon > "$LOG_DIR/nestgate.log" 2>&1 &
        echo $! > "$PID_DIR/nestgate.pid"
        sleep 2
        if kill -0 $(cat "$PID_DIR/nestgate.pid") 2>/dev/null; then
            success "NestGate started (PID $(cat $PID_DIR/nestgate.pid))"
        else
            # NestGate with --daemon might exit immediately but spawn a child
            if ss -tlnp 2>/dev/null | grep -q ":8092 "; then
                success "NestGate started (daemonized on port 8092)"
                rm -f "$PID_DIR/nestgate.pid"  # Don't track daemon PID
            else
                warn "NestGate may have failed - check $LOG_DIR/nestgate.log"
            fi
        fi
    fi
fi

echo ""
log "Primal stack startup complete!"
echo ""
echo "Services:"
[ -f "$PID_DIR/songbird-orchestrator.pid" ] && echo "  • Songbird Orchestrator: http://127.0.0.1:8080"
[ -f "$PID_DIR/songbird-rendezvous.pid" ] && echo "  • Songbird Rendezvous:   http://127.0.0.1:8888"
ss -tlnp 2>/dev/null | grep -q ":8092 " && echo "  • NestGate:              http://127.0.0.1:8092"
echo ""
echo "CLI Tools (in $BINS_DIR/):"
echo "  • beardog       - Crypto operations"
echo "  • toadstool-cli - Compute runtime"
echo "  • squirrel-cli  - AI/MCP"
echo ""
echo "Logs: $LOG_DIR/"
echo ""
echo -e "Run ${GREEN}./stop-primals.sh${NC} to stop all services"
echo -e "Run ${GREEN}./check-status.sh${NC} to check health"
