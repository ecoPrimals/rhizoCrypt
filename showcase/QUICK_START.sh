#!/bin/bash
#
# 🔐 rhizoCrypt Showcase Quick Start
#
# One command to experience rhizoCrypt capabilities.
#

set -e

SCRIPT_DIR=$(cd "$(dirname "$0")" && pwd)
cd "$SCRIPT_DIR"

# Colors
PURPLE='\033[0;35m'
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
CYAN='\033[0;36m'
NC='\033[0m'

echo -e "${PURPLE}"
cat << 'EOF'
╔═══════════════════════════════════════════════════════════════╗
║                🔐 rhizoCrypt Showcase                          ║
║                                                                ║
║            The Memory That Knows When to Forget                ║
╚═══════════════════════════════════════════════════════════════╝
EOF
echo -e "${NC}"

echo "Welcome to the rhizoCrypt showcase!"
echo ""
echo -e "${CYAN}Phase 1: Isolated (core capabilities)${NC}"
echo -e "  ${GREEN}1${NC}. Session Lifecycle   (5 min)  - Create, grow, query, resolve"
echo -e "  ${GREEN}2${NC}. DAG Operations      (5 min)  - Multi-parent DAG, content-addressing"
echo -e "  ${GREEN}3${NC}. Merkle Proofs       (5 min)  - Tree construction, O(log n) proofs"
echo -e "  ${GREEN}4${NC}. Slice Semantics     (5 min)  - Copy/Loan/Consignment modes"
echo ""
echo -e "${CYAN}Phase 3: Inter-Primal (ecosystem)${NC}"
echo -e "  ${GREEN}5${NC}. Discovery           (5 min)  - Capability-based primal discovery"
echo -e "  ${GREEN}6${NC}. BearDog Signing     (5 min)  - DID verification, signatures"
echo -e "  ${GREEN}7${NC}. NestGate Payloads   (5 min)  - Content-addressed storage"
echo -e "  ${GREEN}8${NC}. LoamSpine Commits   (5 min)  - Permanent storage, checkout"
echo ""
echo -e "${CYAN}Phase 4: Complete Workflows${NC}"
echo -e "  ${GREEN}9${NC}. Dehydration         (10 min) - Session → Merkle → Commit"
echo ""
echo -e "${CYAN}Phase 5: Live Integration (requires ../bins/)${NC}"
echo -e "  ${GREEN}L${NC}. Start Live Primals  (2 min)  - Songbird + NestGate"
echo -e "  ${GREEN}D${NC}. Live Discovery      (5 min)  - Real Songbird connection"
echo -e "  ${GREEN}S${NC}. Live Signing        (5 min)  - Real BearDog CLI"
echo -e "  ${GREEN}X${NC}. Stop Live Primals   (1 min)  - Cleanup"
echo ""
echo -e "  ${GREEN}a${NC}. Run all mock demos"
echo -e "  ${GREEN}q${NC}. Quit"
echo ""

read -p "Select demo (1-9, L/D/S/X, a, or q): " choice

run_demo() {
    local demo_path="$1"
    local demo_name="$2"
    if [ -f "$demo_path" ]; then
        chmod +x "$demo_path"
        echo -e "${YELLOW}━━━ Running: $demo_name ━━━${NC}"
        ./"$demo_path"
    else
        echo -e "${YELLOW}Demo not found: $demo_path${NC}"
    fi
}

case "$choice" in
    1)
        run_demo "01-isolated/sessions/demo-session-lifecycle.sh" "Session Lifecycle"
        ;;
    2)
        run_demo "01-isolated/dag/demo-dag-operations.sh" "DAG Operations"
        ;;
    3)
        run_demo "01-isolated/merkle/demo-merkle-proofs.sh" "Merkle Proofs"
        ;;
    4)
        run_demo "01-isolated/slices/demo-slice-semantics.sh" "Slice Semantics"
        ;;
    5)
        run_demo "03-inter-primal/songbird-discovery/demo-discovery.sh" "Capability Discovery"
        ;;
    6)
        run_demo "03-inter-primal/beardog-signing/demo-signing.sh" "BearDog Signing"
        ;;
    7)
        run_demo "03-inter-primal/nestgate-payloads/demo-payload-storage.sh" "NestGate Payloads"
        ;;
    8)
        run_demo "03-inter-primal/loamspine-commits/demo-loamspine-commit.sh" "LoamSpine Commits"
        ;;
    9)
        run_demo "04-complete-workflow/dehydration/demo-simple-dehydration.sh" "Dehydration Workflow"
        ;;
    L|l)
        run_demo "05-live-integration/start-primals.sh" "Start Live Primals"
        ;;
    D|d)
        run_demo "05-live-integration/demo-live-discovery.sh" "Live Discovery"
        ;;
    S|s)
        run_demo "05-live-integration/demo-live-signing.sh" "Live Signing"
        ;;
    X|x)
        run_demo "05-live-integration/stop-primals.sh" "Stop Live Primals"
        ;;
    a|A)
        echo ""
        echo -e "${BLUE}Running all 9 demos...${NC}"
        echo ""
        
        demos=(
            "01-isolated/sessions/demo-session-lifecycle.sh:Session Lifecycle"
            "01-isolated/dag/demo-dag-operations.sh:DAG Operations"
            "01-isolated/merkle/demo-merkle-proofs.sh:Merkle Proofs"
            "01-isolated/slices/demo-slice-semantics.sh:Slice Semantics"
            "03-inter-primal/songbird-discovery/demo-discovery.sh:Capability Discovery"
            "03-inter-primal/beardog-signing/demo-signing.sh:BearDog Signing"
            "03-inter-primal/nestgate-payloads/demo-payload-storage.sh:NestGate Payloads"
            "03-inter-primal/loamspine-commits/demo-loamspine-commit.sh:LoamSpine Commits"
            "04-complete-workflow/dehydration/demo-simple-dehydration.sh:Dehydration Workflow"
        )
        
        for item in "${demos[@]}"; do
            demo_path="${item%%:*}"
            demo_name="${item##*:}"
            run_demo "$demo_path" "$demo_name"
            echo ""
            echo -e "${GREEN}Press Enter to continue to next demo...${NC}"
            read
        done
        
        echo -e "${GREEN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
        echo -e "${GREEN}✅ All 9 demos completed successfully!${NC}"
        echo -e "${GREEN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
        ;;
    q|Q)
        echo ""
        echo "Goodbye!"
        exit 0
        ;;
    *)
        echo ""
        echo -e "${YELLOW}Invalid choice. Please run again and select 1-9, L/D/S/X, a, or q.${NC}"
        exit 1
        ;;
esac

