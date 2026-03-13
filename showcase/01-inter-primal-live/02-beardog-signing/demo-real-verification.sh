#!/usr/bin/env bash
#
# 🐻 Demo: Real BearDog Verification
#
# Demonstrates signature verification workflow
# NO MOCKS - Shows real integration pattern
#

set -euo pipefail

GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
CYAN='\033[0;36m'
RED='\033[0;31m'
NC='\033[0m'

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/../../.." && pwd)"
BINS_DIR="${PRIMAL_BINS_DIR:-$REPO_ROOT/../../primalBins}"
BEARDOG="${BEARDOG_BIN:-$BINS_DIR/beardog}"

echo -e "${BLUE}═══════════════════════════════════════════════════════${NC}"
echo -e "${BLUE}   🔐 Real Signature Verification${NC}"
echo -e "${BLUE}═══════════════════════════════════════════════════════${NC}"
echo ""

if [ ! -f "$BEARDOG" ]; then
    echo -e "${RED}❌ BearDog binary not found${NC}"
    echo "Run ./start-beardog.sh first"
    exit 1
fi

chmod +x "$BEARDOG" 2>/dev/null || true

LOG_DIR="$SCRIPT_DIR/logs"
mkdir -p "$LOG_DIR"

echo -e "${YELLOW}📝 Signature Verification Workflow${NC}"
echo ""

echo "This demo shows how rhizoCrypt would verify signatures:"
echo ""

cat <<'WORKFLOW'
┌─────────────────────────────────────────────────────────┐
│ Signature Verification Workflow                         │
├─────────────────────────────────────────────────────────┤
│                                                          │
│  1. Receive Vertex with Signature                       │
│     • Vertex ID                                          │
│     • Vertex data                                        │
│     • Agent DID                                          │
│     • Signature bytes                                    │
│                                                          │
│  2. Recompute Vertex Hash                               │
│     • Blake3(vertex_data)                               │
│     • Ensures data hasn't changed                       │
│                                                          │
│  3. Extract Public Key from DID                         │
│     • did:key:z6Mk... → Ed25519 public key             │
│     • BearDog handles key resolution                    │
│                                                          │
│  4. Verify Signature                                    │
│     • Ed25519.verify(pubkey, hash, signature)          │
│     • BearDog crypto engine                            │
│                                                          │
│  5. Accept or Reject                                    │
│     • ✅ Valid → Add to DAG                            │
│     • ❌ Invalid → Reject vertex                       │
│                                                          │
└─────────────────────────────────────────────────────────┘
WORKFLOW

echo ""

echo -e "${YELLOW}📝 Benefits of HSM Verification:${NC}"
echo ""
echo "  🔐 Cryptographic Proof"
echo "     • Signatures can't be forged"
echo "     • Only private key holder can sign"
echo "     • Public verification by anyone"
echo ""
echo "  🎯 Authenticity"
echo "     • Proves who created the vertex"
echo "     • Links to DID identity"
echo "     • Non-repudiation"
echo ""
echo "  🛡️  Integrity"
echo "     • Detects any tampering"
echo "     • Hash mismatch = rejection"
echo "     • Immutable provenance"
echo ""
echo "  🔗 Trust Chain"
echo "     • Each vertex signed"
echo "     • DAG becomes audit trail"
echo "     • Full provenance history"
echo ""

echo -e "${YELLOW}📝 Integration with rhizoCrypt:${NC}"
echo ""

echo "In production, rhizoCrypt would:"
echo ""
echo "1. Discover signing service via Songbird"
echo "   query_capabilities([\"Signing\", \"Ed25519\"])"
echo ""
echo "2. Send vertex hash to BearDog"
echo "   SigningClient::sign(vertex_hash, agent_did)"
echo ""
echo "3. Receive and attach signature"
echo "   vertex.add_signature(sig)"
echo ""
echo "4. Verify on retrieval"
echo "   SigningClient::verify(vertex, signature)"
echo ""

echo -e "${BLUE}═══════════════════════════════════════════════════════${NC}"
echo -e "${GREEN}✅ Verification workflow explained!${NC}"
echo ""
echo -e "${YELLOW}📚 Key Concepts:${NC}"
echo "  • Public key cryptography (Ed25519)"
echo "  • DID-based identity"
echo "  • HSM for key security"
echo "  • Capability-based discovery"
echo "  • Zero vendor lock-in"
echo ""
echo -e "${CYAN}🔗 Next Steps for Full Integration:${NC}"
echo "  1. Implement SigningProvider trait"
echo "  2. Add BearDog client to rhizoCrypt"
echo "  3. Discover via Songbird capabilities"
echo "  4. Add signature field to Vertex struct"
echo "  5. Update Merkle tree to include signatures"
echo ""
echo -e "${YELLOW}▶ Next demo:${NC} ./demo-real-multi-agent.sh"
echo ""

