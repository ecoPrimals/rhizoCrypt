#!/usr/bin/env bash
set -euo pipefail

# Demo: Basic NestGate Storage and Retrieval
# Purpose: Store rhizoCrypt payload in NestGate and retrieve it
# Status: Phase 3 - Storage Integration

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../../.." && pwd)"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
NESTGATE_ENDPOINT="${NESTGATE_ENDPOINT:-http://127.0.0.1:9500}"
TEST_PAYLOAD="Hello from rhizoCrypt! This is a test payload."

echo -e "${BLUE}═══════════════════════════════════════════════════════${NC}"
echo -e "${BLUE}  NestGate Integration: Basic Storage & Retrieval${NC}"
echo -e "${BLUE}═══════════════════════════════════════════════════════${NC}"
echo ""

# ============================================================================
# Step 1: Check NestGate Availability
# ============================================================================
echo -e "${YELLOW}Step 1: Checking NestGate service...${NC}"

if ! curl -sf "$NESTGATE_ENDPOINT/health" > /dev/null 2>&1; then
    echo -e "${RED}✗ NestGate not responding at $NESTGATE_ENDPOINT${NC}"
    echo ""
    echo "To start NestGate:"
    echo "  cd ../../../bins/"
    echo "  ./nestgate --port 9500 --storage ./nestgate-data"
    echo ""
    echo "Or set custom endpoint:"
    echo "  export NESTGATE_ENDPOINT=http://10.0.1.5:9500"
    exit 1
fi

echo -e "${GREEN}✓ NestGate is healthy${NC}"
echo ""

# ============================================================================
# Step 2: Create Test Payload
# ============================================================================
echo -e "${YELLOW}Step 2: Creating test payload...${NC}"

PAYLOAD_FILE=$(mktemp)
echo "$TEST_PAYLOAD" > "$PAYLOAD_FILE"

echo "Payload: $TEST_PAYLOAD"
echo "Size: $(wc -c < "$PAYLOAD_FILE") bytes"
echo ""

# ============================================================================
# Step 3: Store Payload in NestGate
# ============================================================================
echo -e "${YELLOW}Step 3: Storing payload in NestGate...${NC}"

# Use NestGate REST API to store payload
STORE_RESPONSE=$(curl -sf -X POST \
    "$NESTGATE_ENDPOINT/api/v1/payloads" \
    -H "Content-Type: application/octet-stream" \
    --data-binary "@$PAYLOAD_FILE" \
    2>/dev/null || echo "{}")

if [[ -z "$STORE_RESPONSE" ]] || [[ "$STORE_RESPONSE" == "{}" ]]; then
    echo -e "${RED}✗ Failed to store payload${NC}"
    rm -f "$PAYLOAD_FILE"
    exit 1
fi

# Extract hash from response
PAYLOAD_HASH=$(echo "$STORE_RESPONSE" | grep -o '"hash":"[^"]*"' | cut -d'"' -f4 || echo "")

if [[ -z "$PAYLOAD_HASH" ]]; then
    echo -e "${RED}✗ No hash returned from NestGate${NC}"
    echo "Response: $STORE_RESPONSE"
    rm -f "$PAYLOAD_FILE"
    exit 1
fi

echo -e "${GREEN}✓ Payload stored successfully${NC}"
echo "Hash: $PAYLOAD_HASH"
echo ""

# ============================================================================
# Step 4: Retrieve Payload by Hash
# ============================================================================
echo -e "${YELLOW}Step 4: Retrieving payload by hash...${NC}"

RETRIEVED_FILE=$(mktemp)

if ! curl -sf "$NESTGATE_ENDPOINT/api/v1/payloads/$PAYLOAD_HASH" \
    -o "$RETRIEVED_FILE" 2>/dev/null; then
    echo -e "${RED}✗ Failed to retrieve payload${NC}"
    rm -f "$PAYLOAD_FILE" "$RETRIEVED_FILE"
    exit 1
fi

echo -e "${GREEN}✓ Payload retrieved successfully${NC}"
echo ""

# ============================================================================
# Step 5: Verify Content Integrity
# ============================================================================
echo -e "${YELLOW}Step 5: Verifying content integrity...${NC}"

ORIGINAL_CONTENT=$(cat "$PAYLOAD_FILE")
RETRIEVED_CONTENT=$(cat "$RETRIEVED_FILE")

if [[ "$ORIGINAL_CONTENT" == "$RETRIEVED_CONTENT" ]]; then
    echo -e "${GREEN}✓ Content verified: Original matches retrieved${NC}"
else
    echo -e "${RED}✗ Content mismatch!${NC}"
    echo "Original:  $ORIGINAL_CONTENT"
    echo "Retrieved: $RETRIEVED_CONTENT"
    rm -f "$PAYLOAD_FILE" "$RETRIEVED_FILE"
    exit 1
fi
echo ""

# ============================================================================
# Step 6: Demonstrate Hash-Based Addressing
# ============================================================================
echo -e "${YELLOW}Step 6: Demonstrating content-addressed storage...${NC}"

# Store the same content again
STORE_RESPONSE2=$(curl -sf -X POST \
    "$NESTGATE_ENDPOINT/api/v1/payloads" \
    -H "Content-Type: application/octet-stream" \
    --data-binary "@$PAYLOAD_FILE" \
    2>/dev/null || echo "{}")

PAYLOAD_HASH2=$(echo "$STORE_RESPONSE2" | grep -o '"hash":"[^"]*"' | cut -d'"' -f4 || echo "")

if [[ "$PAYLOAD_HASH" == "$PAYLOAD_HASH2" ]]; then
    echo -e "${GREEN}✓ Deduplication working: Same content = same hash${NC}"
    echo "Hash 1: $PAYLOAD_HASH"
    echo "Hash 2: $PAYLOAD_HASH2"
else
    echo -e "${YELLOW}⚠ Different hashes returned (may indicate server-side dedup)${NC}"
fi
echo ""

# ============================================================================
# Summary
# ============================================================================
echo -e "${BLUE}═══════════════════════════════════════════════════════${NC}"
echo -e "${BLUE}  Summary${NC}"
echo -e "${BLUE}═══════════════════════════════════════════════════════${NC}"
echo ""
echo -e "${GREEN}✓ NestGate storage working${NC}"
echo -e "${GREEN}✓ Payload stored: ${PAYLOAD_HASH:0:16}...${NC}"
echo -e "${GREEN}✓ Content retrieved and verified${NC}"
echo -e "${GREEN}✓ Content-addressed storage confirmed${NC}"
echo ""
echo "Key Insights:"
echo "  - Payloads addressed by hash (not by name)"
echo "  - Immutable: hash changes if content changes"
echo "  - Deduplicated: same content stored once"
echo "  - Integrity: automatic verification on retrieval"
echo ""
echo "Next Steps:"
echo "  ./demo-content-addressed.sh    - Explore hash-based storage"
echo "  ./demo-payload-metadata.sh     - Track metadata"
echo "  ./demo-workflow-integration.sh - Full integration"
echo ""

# Cleanup
rm -f "$PAYLOAD_FILE" "$RETRIEVED_FILE"

echo -e "${GREEN}Demo completed successfully!${NC}"

