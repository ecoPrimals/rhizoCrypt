#!/bin/bash
# Demo: Intelligent Routing with Squirrel AI
# Prerequisites: Understanding of capability-based discovery
# Expected: AI-powered routing decisions based on DAG context

set -euo pipefail

# Paths (portable)
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/../../.." && pwd)"
BINS_DIR="${PRIMAL_BINS_DIR:-$REPO_ROOT/../../primalBins}"
SQUIRREL_BIN="${SQUIRREL_BIN:-$BINS_DIR/squirrel-cli}"

# Colors
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m'

echo -e "${BLUE}════════════════════════════════════════════════════════════════${NC}"
echo -e "${BLUE}  🐿️  Demo: Intelligent Routing with Squirrel AI${NC}"
echo -e "${BLUE}════════════════════════════════════════════════════════════════${NC}"
echo ""

# Check binary
echo -e "${YELLOW}📝 Step 1: Check Squirrel binary${NC}"
if [ ! -f "$SQUIRREL_BIN" ]; then
    echo -e "${RED}✗ Squirrel binary not found at: $SQUIRREL_BIN${NC}"
    echo -e "${YELLOW}  This demo shows the intended integration pattern${NC}"
else
    echo -e "${GREEN}✓ Squirrel binary found${NC}"
    echo -e "${BLUE}   Testing CLI: $SQUIRREL_BIN --help${NC}"
    "$SQUIRREL_BIN" --help > /tmp/squirrel-help.txt 2>&1 || true
    head -n 3 /tmp/squirrel-help.txt || echo "  (CLI help not available)"
fi

# Scenario
echo -e "\n${YELLOW}📝 Scenario: Multi-User Document Collaboration${NC}"
echo -e "${BLUE}   3 users editing a document concurrently${NC}"
echo -e "${BLUE}   Squirrel optimizes routing for each operation${NC}"
echo ""

# Workflow without AI
echo -e "\n${YELLOW}📝 Step 2: Traditional workflow (no AI)${NC}"
cat <<'EOF'

❌ Without Squirrel AI:
──────────────────────────────────────────────────────────────
  User Edit → Hardcoded Route → BearDog/NestGate/etc
  
  Problems:
  • All edits go to same primal (bottleneck)
  • No load balancing
  • Ignores agent capabilities
  • Fixed routing, no adaptation
  • High latency for remote users

  Example:
    Alice (US) → Edit → BearDog (EU) → 150ms latency ⚠️
    Bob (EU)   → Edit → BearDog (EU) →  20ms latency ✓
    Carol (US) → Edit → BearDog (EU) → 155ms latency ⚠️

EOF
echo -e "${RED}✗ Fixed routing causes performance issues${NC}"

# Workflow with AI
echo -e "\n${YELLOW}📝 Step 3: AI-powered workflow${NC}"
cat <<'EOF'

✅ With Squirrel AI:
──────────────────────────────────────────────────────────────
  User Edit → Squirrel Analysis → Optimal Route
  
  Squirrel considers:
  • Agent location (geo proximity)
  • Current load (which primals busy?)
  • Content type (needs signing? storage?)
  • Historical patterns (what worked before?)
  • Cost optimization (cheapest route)

  Example:
    Alice (US) → Squirrel → BearDog-US  →  25ms latency ✓
    Bob (EU)   → Squirrel → BearDog-EU  →  20ms latency ✓
    Carol (US) → Squirrel → BearDog-US  →  28ms latency ✓

EOF
echo -e "${GREEN}✓ AI routing reduces latency by 80%!${NC}"

# Routing decision process
echo -e "\n${YELLOW}📝 Step 4: Squirrel routing decision process${NC}"
cat <<'EOF'

┌────────────────────────────────────────────────────────────────┐
│  Squirrel AI Routing Decision                                  │
├────────────────────────────────────────────────────────────────┤
│                                                                │
│  Input Context:                                                │
│  ├─ Agent: did:key:alice                                      │
│  ├─ Location: US-West                                         │
│  ├─ Operation: document-edit                                  │
│  ├─ Content size: 4 KB                                        │
│  ├─ Session DAG: 47 vertices, 3 agents                        │
│  └─ Recent pattern: collaborative editing (detected)          │
│                                                                │
│  Analysis:                                                     │
│  ├─ Content type: Needs signing + storage                     │
│  ├─ Geo analysis: US-West region preferred                    │
│  ├─ Load analysis: BearDog-EU at 95% capacity ⚠️             │
│  │                  BearDog-US at 45% capacity ✓             │
│  ├─ Cost analysis: US route $0.0012, EU route $0.0018        │
│  └─ Pattern match: Collaborative edit (confidence: 0.92)      │
│                                                                │
│  Decision:                                                     │
│  ✅ Route to BearDog-US (did:beardog:us-west:node-1)          │
│  ✅ Then store in NestGate-US (did:nestgate:us-west:node-2)   │
│                                                                │
│  Expected latency: 25ms (vs 150ms fixed route)                │
│  Cost savings: 33% ($0.0012 vs $0.0018)                       │
│  Confidence: 94%                                               │
│                                                                │
└────────────────────────────────────────────────────────────────┘

EOF
echo -e "${GREEN}✓ AI considers multiple factors for optimal routing${NC}"

# Code example
echo -e "\n${YELLOW}📝 Step 5: Integration pattern (capability-based)${NC}"
cat <<'EOF'

// Discover Squirrel via capabilities (no hardcoding!)
let ai_router = CapabilityRegistry::discover("AIProvider")
    .with_capability("intelligent-routing")
    .with_capability("pattern-recognition")
    .await?;

// Prepare routing context
let context = RoutingContext {
    session_id: session.id(),
    agent: did!("did:key:alice"),
    operation: "document-edit",
    content_size: 4096,
    location: "us-west-2",
    dag_frontier: session.frontier().await?,
    recent_history: session.recent_vertices(50).await?,
};

// Ask Squirrel for routing decision
let decision = ai_router.route_request(context).await?;

println!("Squirrel recommends: {} (confidence: {}%)", 
    decision.target, decision.confidence * 100.0);

// Apply AI recommendation
match decision.target {
    Target::BearDog(region) => {
        let beardog = discover_beardog_in_region(region).await?;
        beardog.sign_vertex(vertex).await?;
    }
    Target::NestGate(region) => {
        let nestgate = discover_nestgate_in_region(region).await?;
        nestgate.store_payload(payload).await?;
    }
}

// Record outcome for learning
ai_router.record_outcome(WorkflowOutcome {
    decision_id: decision.id,
    success: true,
    latency_ms: 25,
    cost_usd: 0.0012,
    agent_satisfaction: 0.95,
}).await?;

EOF
echo -e "${GREEN}✓ Zero lock-in, pure capability-based discovery${NC}"

# Benefits
echo -e "\n${YELLOW}📝 Step 6: Benefits of AI routing${NC}"
echo -e "${BLUE}   1. Performance${NC}"
echo -e "      → 80% latency reduction (geo-aware routing)"
echo -e "      → Load balancing across primals"
echo -e ""
echo -e "${BLUE}   2. Cost Optimization${NC}"
echo -e "      → Route to cheapest primal"
echo -e "      → 33% cost savings in example"
echo -e ""
echo -e "${BLUE}   3. Adaptability${NC}"
echo -e "      → Learns from outcomes"
echo -e "      → Continuous improvement"
echo -e ""
echo -e "${BLUE}   4. Intelligence${NC}"
echo -e "      → Pattern detection (collaborative editing)"
echo -e "      → Predictive routing"
echo -e ""
echo -e "${BLUE}   5. Flexibility${NC}"
echo -e "      → Works with any primal"
echo -e "      → Graceful fallback if Squirrel unavailable"
echo -e ""

# Privacy considerations
echo -e "\n${YELLOW}📝 Step 7: Privacy-preserving AI${NC}"
cat <<'EOF'

Squirrel only sees what it needs:
────────────────────────────────────────────────────────────────
  ✅ Shared: Agent location, content size, operation type
  ✅ Shared: DAG structure (not content)
  ❌ Hidden: Agent DIDs (anonymized)
  ❌ Hidden: Payload content (truncated)
  ❌ Hidden: Sensitive metadata

// Export privacy-preserving context
let safe_context = session.export_dag()
    .anonymize_agents()       // did:key:alice → agent-1
    .truncate_payloads()      // Remove payload data
    .strip_sensitive_meta()   // Remove PII
    .await?;

ai_router.analyze(safe_context).await?;

EOF
echo -e "${GREEN}✓ Privacy-first AI: Consent-based, minimal data sharing${NC}"

# Provenance
echo -e "\n${YELLOW}📝 Step 8: AI decisions in provenance${NC}"
cat <<'EOF'

Session DAG (includes AI decisions):
────────────────────────────────────────────────────────────────
  ① Edit Request (Alice)
       └─ Agent: did:key:alice
  
  ② Routing Decision (Squirrel AI)  ← AI vertex!
       └─ Agent: did:squirrel:router
       └─ Parent: [edit-request]
       └─ Decision: Route to BearDog-US
       └─ Confidence: 94%
       └─ Factors: [location, load, cost, pattern]
  
  ③ Signature (BearDog-US)
       └─ Agent: did:beardog:us-west:node-1
       └─ Parent: [routing-decision]
  
  ④ Storage (NestGate-US)
       └─ Agent: did:nestgate:us-west:node-2
       └─ Parent: [signature]

✅ AI decisions are part of provenance (cryptographically linked)
✅ Full audit trail: Why was this route chosen?

EOF
echo -e "${GREEN}✓ Transparent AI: All decisions recorded in DAG${NC}"

# Final summary
echo -e "\n${GREEN}✅ Demo complete!${NC}"
echo -e "\n${BLUE}What you learned:${NC}"
echo "  • AI-powered routing reduces latency 80%"
echo "  • Squirrel considers location, load, cost, patterns"
echo "  • Privacy-preserving: minimal data shared"
echo "  • All AI decisions recorded in provenance DAG"
echo "  • Capability-based: zero vendor lock-in"
echo "  • Continuous learning from outcomes"
echo ""
echo -e "${BLUE}Real-world applications:${NC}"
echo "  • Multi-region document collaboration"
echo "  • Load-balanced compute routing"
echo "  • Cost-optimized storage selection"
echo "  • Adaptive workflow orchestration"
echo ""
echo -e "${BLUE}Next steps:${NC}"
echo "  • Try: ./demo-pattern-recognition.sh"
echo "  • Try: ./demo-adaptive-workflows.sh"
echo "  • See: ../05-complete-workflows/demo-ai-collaboration.sh"
echo ""
echo -e "${BLUE}════════════════════════════════════════════════════════════════${NC}"

# Cleanup
rm -f /tmp/squirrel-help.txt

