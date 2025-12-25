#!/usr/bin/env bash
# Demo: BearDog HSM Discovery
#
# Discovers available HSMs and tests their capabilities

set -e

BEARDOG_BIN="../../../../bins/beardog"

echo "🐻 BearDog HSM Discovery Demo"
echo "=============================="
echo ""

# Check BearDog binary
if [ ! -x "$BEARDOG_BIN" ]; then
    echo "❌ BearDog binary not found at $BEARDOG_BIN"
    exit 1
fi

echo "✅ BearDog binary found: $BEARDOG_BIN"
echo ""

# Show BearDog status
echo "📊 BearDog Status:"
echo "─────────────────"
$BEARDOG_BIN status
echo ""

# Discover HSMs
echo "🔍 Discovering HSMs..."
echo "─────────────────────"
$BEARDOG_BIN hsm discover
echo ""

# Show HSM capabilities
echo "💪 HSM Capabilities:"
echo "────────────────────"
$BEARDOG_BIN hsm capabilities || echo "Note: HSM capabilities require HSM to be configured"
echo ""

# Test HSM (if available)
echo "🧪 Testing HSM:"
echo "───────────────"
$BEARDOG_BIN hsm test || echo "Note: HSM test requires HSM to be configured (SoftHSM2 recommended)"
echo ""

echo "╔════════════════════════════════════════════════════════╗"
echo "║  ✅ HSM Discovery Complete                             ║"
echo "╚════════════════════════════════════════════════════════╝"
echo ""
echo "🎓 Key Learnings:"
echo "  • BearDog supports multiple HSM types"
echo "  • SoftHSM2 is recommended for development"
echo "  • HSM discovery is automatic"
echo "  • Capabilities are vendor-agnostic"
echo ""
echo "📝 Next Steps:"
echo "  1. Install SoftHSM2 (if needed): sudo apt install softhsm2"
echo "  2. Run: ./demo-generate-keys.sh"
echo ""

