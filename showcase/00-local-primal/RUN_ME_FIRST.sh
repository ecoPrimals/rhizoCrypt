#!/bin/bash
#
# 🔐 rhizoCrypt - Automated 60-Minute Tour
#
# This script walks you through all rhizoCrypt local capabilities
# with automated demos and pauses for learning.
#

set -e

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
PURPLE='\033[0;35m'
CYAN='\033[0;36m'
NC='\033[0m'

# Logging
log() { echo -e "${BLUE}[$(date '+%H:%M:%S')]${NC} $1"; }
success() { echo -e "${GREEN}✓${NC} $1"; }
info() { echo -e "${CYAN}ℹ${NC} $1"; }
warn() { echo -e "${YELLOW}⚠${NC} $1"; }
pause_for_user() {
    echo ""
    echo -e "${YELLOW}Press ENTER to continue...${NC}"
    read -r
}

# Banner
clear
echo -e "${PURPLE}"
cat << 'EOF'
╔═══════════════════════════════════════════════════════════════════╗
║                                                                   ║
║          🔐 Welcome to the rhizoCrypt Showcase Tour! 🔐           ║
║                                                                   ║
║     Duration: 60 minutes (6 levels, progressive complexity)       ║
║     Philosophy: Master local capabilities before ecosystem        ║
║                                                                   ║
╚═══════════════════════════════════════════════════════════════════╝
EOF
echo -e "${NC}"

echo ""
echo "📚 What is rhizoCrypt?"
echo ""
echo "  rhizoCrypt is an ephemeral DAG engine - the 'memory that knows"
echo "  when to forget'. Unlike traditional databases that preserve"
echo "  everything, rhizoCrypt embraces selective forgetting:"
echo ""
echo "  • Content-addressed DAG (Blake3 hashing)"
echo "  • Session lifecycle (create → grow → resolve → forget)"
echo "  • Merkle proofs (cryptographic integrity)"
echo "  • Slice semantics (6 modes for state management)"
echo "  • Selective permanence (only commits survive)"
echo ""
echo "🎯 Learning Path:"
echo ""
echo "  Level 1: Hello rhizoCrypt      (5 min, Beginner)"
echo "  Level 2: DAG Engine            (10 min, Beginner)"
echo "  Level 3: Merkle Proofs         (10 min, Intermediate)"
echo "  Level 4: Slice Semantics       (15 min, Advanced)"
echo "  Level 5: Performance           (10 min, Expert)"
echo "  Level 6: Real-World Scenarios  (15 min, Expert)"
echo ""
echo "💡 Tips:"
echo "  • Each level builds on previous concepts"
echo "  • Demos use REAL execution (no mocks!)"
echo "  • Take your time - understanding > speed"
echo "  • Experiment after each demo if curious"
echo ""

pause_for_user

# ============================================================================
# Level 1: Hello rhizoCrypt (5 minutes)
# ============================================================================

clear
echo -e "${PURPLE}"
cat << 'EOF'
╔═══════════════════════════════════════════════════════════════════╗
║                    Level 1: Hello rhizoCrypt                       ║
║                       (5 minutes, Beginner)                        ║
╚═══════════════════════════════════════════════════════════════════╝
EOF
echo -e "${NC}"

echo ""
echo "🎯 Goal: Your first session and vertex"
echo ""
echo "You'll learn:"
echo "  • How to create a rhizoCrypt session"
echo "  • What content-addressing means (Blake3)"
echo "  • How to query the DAG"
echo ""
echo "Starting Level 1..."
echo ""

pause_for_user

cd 01-hello-rhizocrypt

if [ -f "demo-first-session.sh" ]; then
    log "Running: Your First Session"
    ./demo-first-session.sh
    pause_for_user
fi

cd ..

# ============================================================================
# Completion Summary
# ============================================================================

clear
echo -e "${GREEN}"
cat << 'EOF'
╔═══════════════════════════════════════════════════════════════════╗
║                                                                   ║
║              🎉 Congratulations! Tour In Progress! 🎉             ║
║                                                                   ║
╚═══════════════════════════════════════════════════════════════════╝
EOF
echo -e "${NC}"

echo ""
echo "✅ Completed:"
echo "  • Level 1: Hello rhizoCrypt"
echo ""
echo "⏳ Remaining:"
echo "  • Level 2: DAG Engine (10 min)"
echo "  • Level 3: Merkle Proofs (10 min)"
echo "  • Level 4: Slice Semantics (15 min)"
echo "  • Level 5: Performance (10 min)"
echo "  • Level 6: Real-World Scenarios (15 min)"
echo ""
warn "Note: Levels 2-6 are under construction!"
echo ""
info "What's Next?"
echo "  1. Explore individual demos in 01-hello-rhizocrypt/"
echo "  2. Read showcase/00-local-primal/00_START_HERE.md"
echo "  3. Check ../01-isolated/ for more DAG demos"
echo "  4. Review ../../specs/ for full documentation"
echo ""
echo "🚀 Ready for ecosystem integration?"
echo "  cd ../03-inter-primal/  - See rhizoCrypt + Songbird + BearDog"
echo ""
echo "Thank you for exploring rhizoCrypt! 🔐"
echo ""

