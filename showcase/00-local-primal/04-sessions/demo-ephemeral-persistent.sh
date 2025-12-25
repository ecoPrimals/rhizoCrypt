#!/usr/bin/env bash
# Demo: Ephemeral vs Persistent Sessions
#
# This demo compares ephemeral and persistent session types

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR"

echo "🔐 rhizoCrypt Demo: Ephemeral vs Persistent Sessions"
echo "======================================================"
echo ""
echo "rhizoCrypt supports two session types:"
echo "  • Ephemeral: Fast, in-memory, discarded on expire"
echo "  • Persistent: Optionally saved, audit trail preserved"
echo ""

# Build if needed
if [ ! -f "target/debug/demo-ephemeral-persistent" ]; then
    echo "Building demo (first run)..."
    cargo build --bin demo-ephemeral-persistent --quiet
    echo ""
fi

# Run the demo
cargo run --bin demo-ephemeral-persistent --quiet

echo ""
echo "✅ Demo complete!"
echo ""
echo "Next: Try ./demo-slices.sh to see slice semantics"
