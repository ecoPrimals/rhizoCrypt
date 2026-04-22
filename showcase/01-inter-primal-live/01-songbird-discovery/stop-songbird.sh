#!/usr/bin/env bash
# Stop Songbird
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PID_FILE="$SCRIPT_DIR/logs/songbird.pid"

if [ -f "$PID_FILE" ]; then
    PID=$(cat "$PID_FILE")
    if kill -0 $PID 2>/dev/null; then
        echo "🛑 Stopping songbird (PID: $PID)..."
        kill $PID
        sleep 1
        if kill -0 $PID 2>/dev/null; then
            echo "   Force stopping..."
            kill -9 $PID 2>/dev/null || true
        fi
        echo "✅ Songbird stopped"
    else
        echo "ℹ  Songbird not running"
    fi
    rm -f "$PID_FILE"
else
    echo "ℹ  No PID file found"
fi

# Cleanup logs
mkdir -p "$SCRIPT_DIR/logs"
if [ -f "$SCRIPT_DIR/logs/songbird.log" ]; then
    mv "$SCRIPT_DIR/logs/songbird.log" "$SCRIPT_DIR/logs/songbird.log.old" 2>/dev/null || true
fi

echo "🧹 Cleanup complete"
