#!/usr/bin/env bash
# Demo: Session Lifecycle - Create → Grow → Resolve → Expire
#
# This demo shows the complete lifecycle of a rhizoCrypt session

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR"

echo "🔐 rhizoCrypt Demo: Session Lifecycle"
echo "======================================"
echo ""
echo "Sessions in rhizoCrypt have a defined lifecycle:"
echo "  Created → Active → Resolved → Expired"
echo ""
echo "This demonstrates the 'Philosophy of Forgetting' - sessions"
echo "are ephemeral by default, only dehydrated summaries persist."
echo ""

# Build if needed
if [ ! -f "target/debug/demo-session-lifecycle" ]; then
    echo "Building demo (first run)..."
    cargo build --bin demo-session-lifecycle --quiet
    echo ""
fi

# Run the demo
cargo run --bin demo-session-lifecycle --quiet

echo ""
echo "✅ Demo complete!"
echo ""
echo "Next: Try ./demo-ephemeral-persistent.sh to compare session types"
