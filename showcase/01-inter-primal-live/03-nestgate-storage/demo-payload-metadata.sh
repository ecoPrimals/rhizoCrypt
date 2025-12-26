#!/usr/bin/env bash
set -euo pipefail

# Demo: Payload Metadata in NestGate
# Purpose: Demonstrate metadata tracking (content-type, size, timestamps)
# Status: Phase 3 - Storage Integration

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../../.." && pwd)"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

# Configuration
NESTGATE_ENDPOINT="${NESTGATE_ENDPOINT:-http://127.0.0.1:9500}"

echo -e "${BLUE}═══════════════════════════════════════════════════════${NC}"
echo -e "${BLUE}  NestGate: Payload Metadata Tracking${NC}"
echo -e "${BLUE}═══════════════════════════════════════════════════════${NC}"
echo ""

# ============================================================================
# Step 1: Verify NestGate
# ============================================================================
echo -e "${YELLOW}Step 1: Verifying NestGate service...${NC}"

if ! curl -sf "$NESTGATE_ENDPOINT/health" > /dev/null 2>&1; then
    echo -e "${RED}✗ NestGate not available${NC}"
    exit 1
fi

echo -e "${GREEN}✓ NestGate ready${NC}"
echo ""

# ============================================================================
# Step 2: Store Payload with Metadata
# ============================================================================
echo -e "${YELLOW}Step 2: Storing payload with metadata...${NC}"

PAYLOAD_FILE=$(mktemp)
cat > "$PAYLOAD_FILE" << 'EOF'
{
  "session_id": "session-abc123",
  "event_type": "sensor.reading",
  "data": {
    "temperature": 22.5,
    "humidity": 58.3,
    "pressure": 1013.25,
    "timestamp": "2025-12-26T10:30:00Z"
  }
}
EOF

PAYLOAD_SIZE=$(wc -c < "$PAYLOAD_FILE")

echo "Payload size: $PAYLOAD_SIZE bytes"
echo "Content-Type: application/json"
echo ""

# Store with explicit content-type
STORE_RESPONSE=$(curl -sf -X POST \
    "$NESTGATE_ENDPOINT/api/v1/payloads" \
    -H "Content-Type: application/json" \
    -H "X-Metadata: rhizoCrypt-session" \
    --data-binary "@$PAYLOAD_FILE")

PAYLOAD_HASH=$(echo "$STORE_RESPONSE" | grep -o '"hash":"[^"]*"' | cut -d'"' -f4)

echo -e "${GREEN}✓ Payload stored with metadata${NC}"
echo "Hash: $PAYLOAD_HASH"
echo ""

# ============================================================================
# Step 3: Query Metadata (without downloading payload)
# ============================================================================
echo -e "${YELLOW}Step 3: Querying payload metadata...${NC}"

# Use HEAD request to get metadata without downloading
METADATA_RESPONSE=$(curl -sI "$NESTGATE_ENDPOINT/api/v1/payloads/$PAYLOAD_HASH" 2>/dev/null)

if [[ -n "$METADATA_RESPONSE" ]]; then
    echo -e "${GREEN}✓ Metadata retrieved (without downloading payload)${NC}"
    echo ""
    echo "Metadata:"
    echo "$METADATA_RESPONSE" | grep -i "content-type:" || echo "  Content-Type: (not set)"
    echo "$METADATA_RESPONSE" | grep -i "content-length:" || echo "  Content-Length: (not set)"
    echo "$METADATA_RESPONSE" | grep -i "last-modified:" || echo "  Last-Modified: (not set)"
    echo "$METADATA_RESPONSE" | grep -i "etag:" || echo "  ETag: (hash)"
else
    echo -e "${YELLOW}⚠ HEAD request not supported, using GET${NC}"
fi
echo ""

# ============================================================================
# Step 4: Store Multiple Payloads with Different Types
# ============================================================================
echo -e "${YELLOW}Step 4: Storing multiple content types...${NC}"

# JSON
JSON_FILE=$(mktemp)
echo '{"type":"json","data":[1,2,3]}' > "$JSON_FILE"
JSON_RESP=$(curl -sf -X POST "$NESTGATE_ENDPOINT/api/v1/payloads" \
    -H "Content-Type: application/json" --data-binary "@$JSON_FILE")
JSON_HASH=$(echo "$JSON_RESP" | grep -o '"hash":"[^"]*"' | cut -d'"' -f4)

# CBOR (binary)
CBOR_FILE=$(mktemp)
echo -ne '\xA2\x64type\x64cbor\x64data\x83\x01\x02\x03' > "$CBOR_FILE"
CBOR_RESP=$(curl -sf -X POST "$NESTGATE_ENDPOINT/api/v1/payloads" \
    -H "Content-Type: application/cbor" --data-binary "@$CBOR_FILE")
CBOR_HASH=$(echo "$CBOR_RESP" | grep -o '"hash":"[^"]*"' | cut -d'"' -f4)

# Text
TEXT_FILE=$(mktemp)
echo "Plain text session notes" > "$TEXT_FILE"
TEXT_RESP=$(curl -sf -X POST "$NESTGATE_ENDPOINT/api/v1/payloads" \
    -H "Content-Type: text/plain" --data-binary "@$TEXT_FILE")
TEXT_HASH=$(echo "$TEXT_RESP" | grep -o '"hash":"[^"]*"' | cut -d'"' -f4)

echo -e "${GREEN}✓ Multiple content types stored${NC}"
echo "  JSON: $JSON_HASH (application/json)"
echo "  CBOR: $CBOR_HASH (application/cbor)"
echo "  Text: $TEXT_HASH (text/plain)"
echo ""

# ============================================================================
# Step 5: Demonstrate Metadata-Only Queries
# ============================================================================
echo -e "${YELLOW}Step 5: Metadata-only queries...${NC}"

echo "Checking payload sizes (no download):"
for hash in "$JSON_HASH" "$CBOR_HASH" "$TEXT_HASH"; do
    METADATA=$(curl -sI "$NESTGATE_ENDPOINT/api/v1/payloads/$hash" 2>/dev/null || echo "")
    if [[ -n "$METADATA" ]]; then
        SIZE=$(echo "$METADATA" | grep -i "content-length:" | awk '{print $2}' | tr -d '\r\n')
        TYPE=$(echo "$METADATA" | grep -i "content-type:" | awk '{print $2}' | tr -d '\r\n')
        echo "  ${hash:0:16}... - $SIZE bytes ($TYPE)"
    fi
done
echo ""

# ============================================================================
# Step 6: Demonstrate Conditional Requests (ETags)
# ============================================================================
echo -e "${YELLOW}Step 6: Testing conditional requests (ETags)...${NC}"

# Get ETag from initial request
RESPONSE_WITH_ETAG=$(curl -sI "$NESTGATE_ENDPOINT/api/v1/payloads/$PAYLOAD_HASH" 2>/dev/null)
ETAG=$(echo "$RESPONSE_WITH_ETAG" | grep -i "etag:" | awk '{print $2}' | tr -d '\r\n' | tr -d '"')

if [[ -n "$ETAG" ]]; then
    echo "ETag for payload: $ETAG"
    
    # Make conditional request with If-None-Match
    STATUS=$(curl -s -o /dev/null -w "%{http_code}" \
        -H "If-None-Match: \"$ETAG\"" \
        "$NESTGATE_ENDPOINT/api/v1/payloads/$PAYLOAD_HASH")
    
    if [[ "$STATUS" == "304" ]]; then
        echo -e "${GREEN}✓ Conditional request working (304 Not Modified)${NC}"
        echo "  Benefit: Avoids unnecessary re-download"
    else
        echo -e "${YELLOW}⚠ Got status $STATUS (expected 304)${NC}"
    fi
else
    echo -e "${YELLOW}⚠ ETags not supported by server${NC}"
fi
echo ""

# ============================================================================
# Step 7: Demonstrate Metadata for Rehydration
# ============================================================================
echo -e "${YELLOW}Step 7: Using metadata for efficient rehydration...${NC}"

echo "Scenario: Rehydrate session with 3 vertices"
echo ""
echo "Step A: Query metadata for all payload hashes"
echo "  → Determine total size before downloading"
echo ""
echo "Payload inventory:"
TOTAL_SIZE=0
for hash in "$JSON_HASH" "$CBOR_HASH" "$TEXT_HASH"; do
    META=$(curl -sI "$NESTGATE_ENDPOINT/api/v1/payloads/$hash" 2>/dev/null || echo "")
    SIZE=$(echo "$META" | grep -i "content-length:" | awk '{print $2}' | tr -d '\r\n')
    if [[ -n "$SIZE" ]]; then
        echo "  ${hash:0:16}... - $SIZE bytes"
        TOTAL_SIZE=$((TOTAL_SIZE + SIZE))
    fi
done
echo "  Total: $TOTAL_SIZE bytes"
echo ""
echo "Step B: Download only needed payloads"
echo "  → Skip cached payloads using ETags"
echo "  → Download missing payloads in parallel"
echo ""
echo -e "${GREEN}✓ Metadata enables efficient rehydration${NC}"
echo ""

# ============================================================================
# Summary
# ============================================================================
echo -e "${BLUE}═══════════════════════════════════════════════════════${NC}"
echo -e "${BLUE}  Metadata Features${NC}"
echo -e "${BLUE}═══════════════════════════════════════════════════════${NC}"
echo ""
echo -e "${GREEN}✓ Content-Type tracking${NC}  - Preserve payload semantics"
echo -e "${GREEN}✓ Size information${NC}       - Plan downloads efficiently"
echo -e "${GREEN}✓ Timestamps${NC}             - Track storage time"
echo -e "${GREEN}✓ ETags${NC}                  - Avoid redundant transfers"
echo -e "${GREEN}✓ Metadata-only queries${NC}  - Query without downloading"
echo ""
echo "Benefits for rhizoCrypt:"
echo "  • Efficient rehydration (query sizes first)"
echo "  • Conditional requests (use ETags)"
echo "  • Content-type preservation (JSON, CBOR, etc.)"
echo "  • Minimal network transfer"
echo ""
echo "Next Steps:"
echo "  ./demo-workflow-integration.sh - Full workflow demo"
echo ""

# Cleanup
rm -f "$PAYLOAD_FILE" "$JSON_FILE" "$CBOR_FILE" "$TEXT_FILE"

echo -e "${GREEN}Demo completed successfully!${NC}"

