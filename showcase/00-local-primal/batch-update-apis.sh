#!/usr/bin/env bash
# Automated Batch API Update for Local Showcase
# Applies validated patterns from RootPulse to all demos

set -euo pipefail

GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
CYAN='\033[0;36m'
NC='\033[0m'

echo -e "${BLUE}╔═══════════════════════════════════════════════════════╗${NC}"
echo -e "${BLUE}║    🤖 Automated API Update — Batch Mode             ║${NC}"
echo -e "${BLUE}╚═══════════════════════════════════════════════════════╝${NC}"
echo ""

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR"

# Find all demo shell scripts with Rust code
DEMOS=($(find . -name "demo-*.sh" -type f | sort))

echo -e "${CYAN}Found ${#DEMOS[@]} demo scripts${NC}"
echo ""

UPDATED=0
SKIPPED=0

for demo in "${DEMOS[@]}"; do
    echo -e "${YELLOW}Processing:${NC} $demo"
    
    # Check if it has embedded Rust code
    if ! grep -q "cat.*main.rs" "$demo"; then
        echo -e "  ${BLUE}→${NC} No embedded Rust, skipping"
        SKIPPED=$((SKIPPED + 1))
        continue
    fi
    
    # Create backup
    cp "$demo" "$demo.bak"
    
    # Apply common API fixes using Perl for multi-line
    perl -i -pe '
        # Fix Session::new() to SessionBuilder
        s/Session::new\(/SessionBuilder::new(SessionType::General)\n        .with_name(/g;
        s/SessionType::Ephemeral,?\s*\)/)\n        .with_owner(Did::new("did:key:alice"))\n        .build(/g;
        
        # Fix imports
        s/use rhizo_crypt_core::\{RhizoCrypt, RhizoCryptConfig, Session, SessionType\}/use rhizo_crypt_core::{RhizoCrypt, RhizoCryptConfig, SessionType, PrimalLifecycle, session::SessionBuilder, Did}/g;
        s/use rhizo_crypt_core::\{RhizoCrypt, RhizoCryptConfig, Session, SessionType, Vertex, EventType\}/use rhizo_crypt_core::{RhizoCrypt, RhizoCryptConfig, SessionType, EventType, VertexBuilder, PrimalLifecycle, session::SessionBuilder, Did}/g;
        
        # Fix async calls that should be sync
        s/rhizo\.get_session\(([^)]+)\)\.await\?/rhizo.get_session($1)?/g;
        s/rhizo\.list_sessions\(\)\.await/rhizo.list_sessions()/g;
        
        # Fix SessionType::Ephemeral
        s/SessionType::Ephemeral/SessionType::General/g;
        
        # Make rhizo mutable for start/stop
        s/let rhizo = RhizoCrypt::new/let mut rhizo = RhizoCrypt::new/g;
    ' "$demo"
    
    echo -e "  ${GREEN}✓${NC} Applied API updates"
    UPDATED=$((UPDATED + 1))
done

echo ""
echo -e "${BLUE}═══════════════════════════════════════════════════════${NC}"
echo -e "${GREEN}Batch Update Complete${NC}"
echo ""
echo "  Updated: $UPDATED demos"
echo "  Skipped: $SKIPPED demos (no embedded Rust)"
echo ""
echo -e "${CYAN}Backups created:${NC} *.sh.bak files"
echo -e "${CYAN}Next step:${NC} Run ./test-all-demos.sh to verify"
echo ""

