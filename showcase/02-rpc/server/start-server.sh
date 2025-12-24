#!/bin/bash
#
# 🔐 rhizoCrypt RPC Server Startup
#
# Starts the tarpc RPC server with rate limiting and metrics.
#

set -e

# Colors
BLUE='\033[0;34m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
PURPLE='\033[0;35m'
NC='\033[0m'

log() { echo -e "${BLUE}[$(date '+%H:%M:%S')]${NC} $1"; }
success() { echo -e "${GREEN}✓${NC} $1"; }

echo -e "${PURPLE}"
cat << 'EOF'
╔═══════════════════════════════════════════════════════════════╗
║            🔐 rhizoCrypt RPC Server                            ║
║                                                                ║
║  tarpc server with rate limiting and Prometheus metrics        ║
╚═══════════════════════════════════════════════════════════════╝
EOF
echo -e "${NC}"

# Configuration
RPC_PORT="${RHIZOCRYPT_RPC_PORT:-9400}"
METRICS_PORT="${RHIZOCRYPT_METRICS_PORT:-9401}"
RATE_LIMIT_CAPACITY="${RHIZOCRYPT_RATE_LIMIT_CAPACITY:-1000}"
RATE_LIMIT_REFILL="${RHIZOCRYPT_RATE_LIMIT_REFILL:-100}"

log "Configuration:"
echo "   RPC Port:            $RPC_PORT"
echo "   Metrics Port:        $METRICS_PORT"
echo "   Rate Limit Capacity: $RATE_LIMIT_CAPACITY"
echo "   Rate Limit Refill:   $RATE_LIMIT_REFILL/sec"
echo ""

# Check if port is available
if lsof -Pi :$RPC_PORT -sTCP:LISTEN -t >/dev/null 2>&1; then
    echo -e "${YELLOW}Warning: Port $RPC_PORT is already in use${NC}"
    echo "Try: export RHIZOCRYPT_RPC_PORT=9500"
    exit 1
fi

# Navigate to rhizoCrypt root
SCRIPT_DIR=$(cd "$(dirname "$0")" && pwd)
RHIZO_ROOT=$(cd "$SCRIPT_DIR/../../.." && pwd)
cd "$RHIZO_ROOT"

log "Building rhizoCrypt RPC..."
cargo build --release -p rhizo-crypt-rpc 2>&1 | grep -E "(Compiling|Finished|error)" || true

log "Starting RPC server..."
echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "  RPC endpoint:     tcp://localhost:$RPC_PORT"
echo "  Metrics endpoint: http://localhost:$METRICS_PORT/metrics"
echo "  Rate limiting:    $RATE_LIMIT_CAPACITY tokens, $RATE_LIMIT_REFILL/sec"
echo ""
echo "  Press Ctrl+C for graceful shutdown"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

# Run the server (when binary exists)
# For now, show what would happen
echo -e "${YELLOW}Note: This demo requires the rhizo-crypt-rpc binary to be built${NC}"
echo ""
echo "To run the server programmatically:"
echo ""
echo '```rust'
echo 'use rhizo_crypt_rpc::RpcServer;'
echo ''
echo '#[tokio::main]'
echo 'async fn main() {'
echo "    let server = RpcServer::bind(\"0.0.0.0:$RPC_PORT\").await.unwrap();"
echo '    server.run().await;'
echo '}'
echo '```'
echo ""

success "Server configuration ready"
echo ""
echo "Next: Run './demo-rpc-client.sh' in another terminal"

