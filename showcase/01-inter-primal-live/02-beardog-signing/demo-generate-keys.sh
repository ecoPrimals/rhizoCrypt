#!/usr/bin/env bash
# Demo: Generate Signing Keys with BearDog
#
# Generates Ed25519 signing keys for vertex signing

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/../../.." && pwd)"
BINS_DIR="${PRIMAL_BINS_DIR:-$REPO_ROOT/../../primalBins}"
BEARDOG_BIN="${BEARDOG_BIN:-$BINS_DIR/beardog}"
DEMO_DIR="$(pwd)/beardog-demo-keys"

echo "🐻 BearDog Key Generation Demo"
echo "==============================="
echo ""

# Check BearDog binary
if [ ! -x "$BEARDOG_BIN" ]; then
    echo "❌ BearDog binary not found at $BEARDOG_BIN"
    exit 1
fi

# Create demo directory
mkdir -p "$DEMO_DIR"
cd "$DEMO_DIR"

echo "📁 Working directory: $DEMO_DIR"
echo ""

# Step 1: Collect entropy
echo "🎲 Step 1: Collect Entropy"
echo "──────────────────────────"
echo "BearDog uses human entropy for sovereign key generation."
echo "For this demo, we'll use system entropy (non-interactive)."
echo ""

# Generate entropy seed (simulated human input)
cat > entropy-seed.json << 'EOF'
{
  "entropy_sources": [
    "system_random",
    "timestamp",
    "process_id"
  ],
  "entropy_bits": 256,
  "timestamp": "2025-12-25T00:00:00Z"
}
EOF

echo "✅ Entropy seed created (simulated)"
echo ""

# Step 2: Generate Ed25519 key
echo "🔑 Step 2: Generate Ed25519 Signing Key"
echo "────────────────────────────────────────"
echo "Generating key for vertex signing..."
echo ""

KEY_ID="rhizocrypt-demo-$(date +%s)"

# Note: BearDog key generation typically requires HSM setup
# For demo purposes, we'll show the command pattern
echo "Command pattern:"
echo "  $BEARDOG_BIN key generate \\"
echo "    --key-id $KEY_ID \\"
echo "    --algorithm ed25519 \\"
echo "    --output key-info.json"
echo ""

# Simulate key generation (actual command would require HSM)
cat > key-info.json << EOF
{
  "key_id": "$KEY_ID",
  "algorithm": "ed25519",
  "public_key": "$(openssl rand -hex 32)",
  "created_at": "$(date -Iseconds)",
  "hsm_backed": false,
  "note": "Demo key - not HSM backed"
}
EOF

echo "✅ Key generated (demo mode)"
echo ""

# Step 3: Derive DID
echo "🆔 Step 3: Derive DID from Public Key"
echo "──────────────────────────────────────"

# Extract public key
PUBLIC_KEY=$(jq -r '.public_key' key-info.json)
DID="did:key:z${PUBLIC_KEY:0:44}"

echo "Public Key: $PUBLIC_KEY"
echo "DID: $DID"
echo ""

# Save DID
cat > did-info.json << EOF
{
  "did": "$DID",
  "key_id": "$KEY_ID",
  "public_key": "$PUBLIC_KEY",
  "method": "did:key",
  "created_at": "$(date -Iseconds)"
}
EOF

echo "✅ DID derived and saved"
echo ""

# Step 4: Summary
echo "╔════════════════════════════════════════════════════════╗"
echo "║  ✅ Key Generation Complete                            ║"
echo "╚════════════════════════════════════════════════════════╝"
echo ""
echo "📊 Generated Assets:"
echo "  • Key ID: $KEY_ID"
echo "  • Algorithm: Ed25519"
echo "  • DID: $DID"
echo "  • Files:"
echo "    - entropy-seed.json"
echo "    - key-info.json"
echo "    - did-info.json"
echo ""
echo "🎓 Key Learnings:"
echo "  • BearDog uses Ed25519 for signing"
echo "  • DIDs derived from public keys"
echo "  • HSM backing provides hardware security"
echo "  • Entropy collection ensures sovereignty"
echo ""
echo "📝 Next Steps:"
echo "  Run: ./demo-sign-vertex.sh"
echo ""
echo "💡 For Production:"
echo "  • Use real HSM (SoftHSM2, TPM, FIDO2)"
echo "  • Collect human entropy"
echo "  • Store keys securely"
echo "  • Backup key material"
echo ""

