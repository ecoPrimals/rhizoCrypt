#!/bin/bash
#
# 🌱 Showcase Environment Variables - Infant Discovery Pattern
#
# This file provides default environment variables for all showcase demos.
# Each script can source this file to get consistent, configurable defaults.
#
# Philosophy: Zero hardcoding, all configuration via environment.
#
# Usage:
#   source "$(dirname "$0")/../../showcase-env.sh"
#

# ============================================================================
# RHIZOCRYPT SERVICE
# ============================================================================

# RhizoCrypt RPC port (0 = OS-assigned, recommended for testing)
export RHIZOCRYPT_PORT="${RHIZOCRYPT_PORT:-0}"

# RhizoCrypt host
export RHIZOCRYPT_HOST="${RHIZOCRYPT_HOST:-127.0.0.1}"

# Development/production mode
export RHIZOCRYPT_ENV="${RHIZOCRYPT_ENV:-development}"

# ============================================================================
# DISCOVERY (Bootstrap Adapter)
# ============================================================================

# Discovery/bootstrap service endpoint (e.g., Songbird)
# If not set, infant mode: service starts with zero knowledge
export RHIZOCRYPT_DISCOVERY_ADAPTER="${RHIZOCRYPT_DISCOVERY_ADAPTER:-}"

# Songbird tower address (when using Songbird as bootstrap)
export SONGBIRD_TOWER="${SONGBIRD_TOWER:-https://localhost:7500}"

# ============================================================================
# CAPABILITY ENDPOINTS (Optional - Discovery Preferred)
# ============================================================================

# Signing capability (e.g., BearDog, YubiKey, HSM)
export SIGNING_ENDPOINT="${SIGNING_ENDPOINT:-}"

# Payload storage capability (e.g., NestGate, S3, local)
export PAYLOAD_STORAGE_ENDPOINT="${PAYLOAD_STORAGE_ENDPOINT:-}"

# Permanent storage capability (e.g., LoamSpine, PostgreSQL)
export PERMANENT_STORAGE_ENDPOINT="${PERMANENT_STORAGE_ENDPOINT:-}"

# Compute orchestration capability (e.g., ToadStool, Kubernetes, Nomad)
export COMPUTE_ENDPOINT="${COMPUTE_ENDPOINT:-}"

# Provenance tracking capability (e.g., SweetGrass, audit-log)
export PROVENANCE_ENDPOINT="${PROVENANCE_ENDPOINT:-}"

# ============================================================================
# PRIMAL BINS (Phase 1 Integration)
# ============================================================================

# Location of Phase 1 primal binaries
export PRIMAL_BINS="${PRIMAL_BINS:-../../../primalBins}"

# ============================================================================
# TIMEOUTS
# ============================================================================

# Connection timeout (seconds)
export RHIZOCRYPT_TIMEOUT="${RHIZOCRYPT_TIMEOUT:-30}"

# Discovery timeout (seconds)
export DISCOVERY_TIMEOUT="${DISCOVERY_TIMEOUT:-10}"

# ============================================================================
# LOGGING
# ============================================================================

# Log directory for demo output
export LOG_DIR="${LOG_DIR:-./logs}"

# Create log directory if it doesn't exist
mkdir -p "$LOG_DIR" 2>/dev/null || true

# ============================================================================
# COLORS (for demo output)
# ============================================================================

export RED='\033[0;31m'
export GREEN='\033[0;32m'
export YELLOW='\033[1;33m'
export BLUE='\033[0;34m'
export PURPLE='\033[0;35m'
export CYAN='\033[0;36m'
export NC='\033[0m' # No Color

# ============================================================================
# HELPER FUNCTIONS
# ============================================================================

log() { echo -e "${BLUE}[$(date '+%H:%M:%S')]${NC} $1"; }
success() { echo -e "${GREEN}✓${NC} $1"; }
warn() { echo -e "${YELLOW}⚠${NC} $1"; }
error() { echo -e "${RED}✗${NC} $1"; }
info() { echo -e "${CYAN}ℹ${NC} $1"; }

# Print environment summary
print_env() {
    echo -e "${PURPLE}🌱 Infant Discovery Environment:${NC}"
    echo "  RHIZOCRYPT_PORT: ${RHIZOCRYPT_PORT}"
    echo "  RHIZOCRYPT_HOST: ${RHIZOCRYPT_HOST}"
    echo "  RHIZOCRYPT_ENV: ${RHIZOCRYPT_ENV}"
    if [ -n "$RHIZOCRYPT_DISCOVERY_ADAPTER" ]; then
        echo "  DISCOVERY: ${RHIZOCRYPT_DISCOVERY_ADAPTER} ✅"
    else
        echo "  DISCOVERY: None (standalone mode)"
    fi
}

# Check if a port is in use
check_port() {
    local port=$1
    if command -v lsof >/dev/null 2>&1; then
        lsof -i ":$port" >/dev/null 2>&1
    elif command -v netstat >/dev/null 2>&1; then
        netstat -tuln | grep -q ":$port "
    else
        # Can't check, assume available
        return 1
    fi
}

# Find an available port starting from a base
find_available_port() {
    local base_port=${1:-9400}
    local port=$base_port
    while check_port "$port"; do
        port=$((port + 1))
    done
    echo "$port"
}

# Wait for service to be ready
wait_for_service() {
    local host=$1
    local port=$2
    local timeout=${3:-30}
    local elapsed=0
    
    while [ $elapsed -lt $timeout ]; do
        if nc -z "$host" "$port" 2>/dev/null || curl -sf "http://${host}:${port}/health" >/dev/null 2>&1; then
            return 0
        fi
        sleep 1
        elapsed=$((elapsed + 1))
    done
    return 1
}

# ============================================================================
# INITIALIZATION
# ============================================================================

# Print a friendly message if sourced interactively
if [ -n "$PS1" ]; then
    success "Showcase environment loaded!"
    print_env
fi

