#!/bin/bash
#
# 🔐 Check Phase 1 Primal Status
#

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
PURPLE='\033[0;35m'
NC='\033[0m'

echo -e "${PURPLE}"
cat << 'EOF'
╔═══════════════════════════════════════════════════════════════╗
║          🔐 Phase 1 Primal Status                              ║
╚═══════════════════════════════════════════════════════════════╝
EOF
echo -e "${NC}"

PID_DIR="/tmp/rhizocrypt-primals/pids"
LOG_DIR="/tmp/rhizocrypt-primals"

check_service() {
    local name=$1
    local port=$2
    local pid_file="$PID_DIR/$name.pid"
    
    printf "  %-25s" "$name:"
    
    if [ -f "$pid_file" ]; then
        local pid=$(cat "$pid_file")
        if kill -0 "$pid" 2>/dev/null; then
            # Check if port is responding
            if curl -s --connect-timeout 1 "http://127.0.0.1:$port/health" > /dev/null 2>&1 || \
               curl -s --connect-timeout 1 "http://127.0.0.1:$port/" > /dev/null 2>&1; then
                echo -e "${GREEN}● Running${NC} (PID $pid, port $port, healthy)"
            else
                echo -e "${YELLOW}● Running${NC} (PID $pid, port $port, not responding)"
            fi
        else
            echo -e "${RED}● Stopped${NC} (stale PID file)"
            rm -f "$pid_file"
        fi
    else
        # Check if something is on the port anyway
        if ss -tlnp 2>/dev/null | grep -q ":$port "; then
            echo -e "${YELLOW}● Unknown${NC} (port $port in use, not tracked)"
        else
            echo -e "${RED}● Stopped${NC}"
        fi
    fi
}

echo "Services:"
echo ""
check_service "beardog" 8091
check_service "songbird-orchestrator" 8080
check_service "songbird-rendezvous" 8081
check_service "nestgate" 8092

echo ""
echo "Logs: $LOG_DIR/"
echo ""

# Show recent log entries if any errors
if [ -f "$LOG_DIR/beardog.log" ]; then
    if grep -q "error\|Error\|ERROR" "$LOG_DIR/beardog.log" 2>/dev/null; then
        echo -e "${YELLOW}Recent BearDog errors:${NC}"
        grep -i "error" "$LOG_DIR/beardog.log" | tail -3
        echo ""
    fi
fi

