#!/usr/bin/env bash
# Demo: Capability-Based Query
set -euo pipefail

GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
CYAN='\033[0;36m'
NC='\033[0m'

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
SONGBIRD_PORT=8888

echo -e "${BLUE}═══════════════════════════════════════════════════════${NC}"
echo -e "${BLUE}   🔍 Capability-Based Query Demo${NC}"
echo -e "${BLUE}═══════════════════════════════════════════════════════${NC}"
echo ""

if ! curl -s http://localhost:$SONGBIRD_PORT/health > /dev/null 2>&1; then
    echo -e "${YELLOW}Please start Songbird first:${NC}"
    echo "  ./demo-register-presence.sh"
    exit 1
fi

echo -e "${YELLOW}📝 Register Multiple Services with Different Capabilities${NC}"
echo ""

# Register rhizoCrypt
curl -s -X POST http://localhost:$SONGBIRD_PORT/api/v1/register \
  -H "Content-Type: application/json" \
  -d '{
    "service_id": "rhizocrypt-1",
    "capabilities": ["ephemeral-dag", "merkle-proofs", "session-management"],
    "address": "localhost:9400",
    "metadata": {"primal": "rhizoCrypt", "instance": "1"}
  }' > /dev/null

echo -e "${GREEN}✓${NC} Registered: rhizocrypt-1 (ephemeral-dag, merkle-proofs, session-management)"

# Register mock storage service
curl -s -X POST http://localhost:$SONGBIRD_PORT/api/v1/register \
  -H "Content-Type: application/json" \
  -d '{
    "service_id": "storage-demo",
    "capabilities": ["storage", "payload-storage"],
    "address": "localhost:9500",
    "metadata": {"primal": "Demo Storage"}
  }' > /dev/null

echo -e "${GREEN}✓${NC} Registered: storage-demo (storage, payload-storage)"

# Register mock compute service
curl -s -X POST http://localhost:$SONGBIRD_PORT/api/v1/register \
  -H "Content-Type: application/json" \
  -d '{
    "service_id": "compute-demo",
    "capabilities": ["compute", "gpu-acceleration"],
    "address": "localhost:9600",
    "metadata": {"primal": "Demo Compute"}
  }' > /dev/null

echo -e "${GREEN}✓${NC} Registered: compute-demo (compute, gpu-acceleration)"

# Register mock signing service
curl -s -X POST http://localhost:$SONGBIRD_PORT/api/v1/register \
  -H "Content-Type: application/json" \
  -d '{
    "service_id": "signing-demo",
    "capabilities": ["signing", "hsm"],
    "address": "localhost:9700",
    "metadata": {"primal": "Demo Signing"}
  }' > /dev/null

echo -e "${GREEN}✓${NC} Registered: signing-demo (signing, hsm)"
echo ""

echo -e "${YELLOW}🔍 Query by Capability (NOT by Name!)${NC}"
echo ""

# Query 1: Find DAG engines
echo -e "${CYAN}Query 1: Need ephemeral-dag capability${NC}"
RESULT=$(curl -s -X POST http://localhost:$SONGBIRD_PORT/api/v1/query \
  -H "Content-Type: application/json" \
  -d '{"capabilities_required": ["ephemeral-dag"], "capabilities_optional": [], "exclude_node_ids": []}')

echo "$RESULT" | jq -r '.services[] | "  → \(.service_id) at \(.address)"'
echo ""

# Query 2: Find storage services
echo -e "${CYAN}Query 2: Need storage capability${NC}"
RESULT=$(curl -s -X POST http://localhost:$SONGBIRD_PORT/api/v1/query \
  -H "Content-Type: application/json" \
  -d '{"capabilities_required": ["storage"], "capabilities_optional": [], "exclude_node_ids": []}')

echo "$RESULT" | jq -r '.services[] | "  → \(.service_id) at \(.address)"'
echo ""

# Query 3: Find compute services
echo -e "${CYAN}Query 3: Need compute capability${NC}"
RESULT=$(curl -s -X POST http://localhost:$SONGBIRD_PORT/api/v1/query \
  -H "Content-Type: application/json" \
  -d '{"capabilities_required": ["compute"], "capabilities_optional": [], "exclude_node_ids": []}')

echo "$RESULT" | jq -r '.services[] | "  → \(.service_id) at \(.address)"'
echo ""

# Query 4: Multiple required capabilities
echo -e "${CYAN}Query 4: Need BOTH ephemeral-dag AND merkle-proofs${NC}"
RESULT=$(curl -s -X POST http://localhost:$SONGBIRD_PORT/api/v1/query \
  -H "Content-Type: application/json" \
  -d '{"capabilities_required": ["ephemeral-dag", "merkle-proofs"], "capabilities_optional": [], "exclude_node_ids": []}')

echo "$RESULT" | jq -r '.services[] | "  → \(.service_id) at \(.address)"'
echo ""

# Query 5: Optional capabilities
echo -e "${CYAN}Query 5: Need signing, prefer HSM (optional)${NC}"
RESULT=$(curl -s -X POST http://localhost:$SONGBIRD_PORT/api/v1/query \
  -H "Content-Type: application/json" \
  -d '{"capabilities_required": ["signing"], "capabilities_optional": ["hsm"], "exclude_node_ids": []}')

echo "$RESULT" | jq -r '.services[] | "  → \(.service_id) [\(.capabilities | join(", "))]"'
echo ""

echo -e "${BLUE}═══════════════════════════════════════════════════════${NC}"
echo -e "${GREEN}✅ Capability query demo complete!${NC}"
echo ""
echo -e "${YELLOW}📚 What you learned:${NC}"
echo "  • Discovery is capability-based (not name-based)"
echo "  • Services declare what they CAN DO"
echo "  • Clients query by WHAT THEY NEED"
echo "  • Multiple providers can offer same capability"
echo "  • Vendor-neutral architecture"
echo ""
echo -e "${YELLOW}🎯 Key Insight:${NC}"
echo "  ❌ BAD:  'Find service named beardog'"
echo "  ✅ GOOD: 'Find service with signing capability'"
echo ""
echo -e "${YELLOW}Benefits:${NC}"
echo "  • Swap providers without code changes"
echo "  • Multiple redundant providers"
echo "  • No vendor lock-in"
echo "  • Federation-ready"
echo ""
echo -e "${CYAN}🎉 Songbird Discovery Complete!${NC}"
echo ""
echo -e "${YELLOW}▶ Next:${NC} BearDog signing integration"
echo "   cd ../02-beardog-signing"
echo ""

