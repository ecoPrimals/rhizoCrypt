#!/bin/bash
#
# 🏠 Stop NestGate Service
#

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PID_FILE="$SCRIPT_DIR/.nestgate.pid"

GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m'

echo -e "${YELLOW}🛑 Stopping NestGate service...${NC}"

if [ -f "$PID_FILE" ]; then
    PID=$(cat "$PID_FILE")
    if kill -0 $PID 2>/dev/null; then
        kill $PID
        echo -e "${GREEN}✓${NC} NestGate stopped (PID: $PID)"
    else
        echo -e "${YELLOW}⚠${NC}  NestGate not running (PID: $PID)"
    fi
    rm -f "$PID_FILE"
else
    echo -e "${YELLOW}⚠${NC}  No PID file found"
    
    # Try to kill any running nestgate process
    if pkill -f "nestgate service start" 2>/dev/null; then
        echo -e "${GREEN}✓${NC} Stopped running NestGate process"
    else
        echo -e "${YELLOW}⚠${NC}  No NestGate process found"
    fi
fi

echo ""

