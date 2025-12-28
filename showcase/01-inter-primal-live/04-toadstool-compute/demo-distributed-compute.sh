#!/bin/bash
# Demo: Distributed Compute Across Multiple Regions
# Prerequisites: Understanding of GPU provenance
# Expected: Multi-region coordination with global provenance DAG

set -euo pipefail

# Colors
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m'

echo -e "${BLUE}════════════════════════════════════════════════════════════════${NC}"
echo -e "${BLUE}  🌍 Demo: Distributed Compute Provenance${NC}"
echo -e "${BLUE}════════════════════════════════════════════════════════════════${NC}"
echo ""

# Scenario
echo -e "${YELLOW}📝 Scenario: Global ML Inference Deployment${NC}"
echo -e "${BLUE}   Deploy inference across 3 regions for low latency${NC}"
echo -e "${BLUE}   Track performance, cost, and SLAs per region${NC}"
echo ""

# Region topology
echo -e "${YELLOW}📝 Step 1: Multi-region topology${NC}"
cat <<'EOF'

┌────────────────────────────────────────────────────────────────┐
│  Global Compute Deployment                                     │
├────────────────────────────────────────────────────────────────┤
│                                                                │
│  🌎 US-West (us-west-2)                                        │
│     ├─ Node 1: did:toadstool:us-west:node-1 (4× GPU)          │
│     └─ Node 2: did:toadstool:us-west:node-2 (4× GPU)          │
│                                                                │
│  🌍 EU-Central (eu-central-1)                                  │
│     ├─ Node 1: did:toadstool:eu-central:node-1 (4× GPU)       │
│     └─ Node 2: did:toadstool:eu-central:node-2 (4× GPU)       │
│                                                                │
│  🌏 Asia-Pacific (ap-southeast-1)                              │
│     ├─ Node 1: did:toadstool:ap-southeast:node-1 (4× GPU)     │
│     └─ Node 2: did:toadstool:ap-southeast:node-2 (4× GPU)     │
│                                                                │
│  Total: 24 GPUs across 3 regions                              │
│                                                                │
└────────────────────────────────────────────────────────────────┘

EOF
echo -e "${GREEN}✓ Geo-distributed GPU cluster with regional DIDs${NC}"

# Inference workflow
echo -e "\n${YELLOW}📝 Step 2: Distributed inference workflow${NC}"
cat <<'EOF'

┌────────────────────────────────────────────────────────────────┐
│  Request Flow (User in Europe)                                 │
├────────────────────────────────────────────────────────────────┤
│                                                                │
│  ① User Request                                               │
│     └─ Location: Berlin, Germany                              │
│     └─ Model: gpt-4-turbo (inference)                         │
│     └─ Routing: Nearest region (EU-Central)                   │
│                                                                │
│  ② Vertex: Request Received                                   │
│     └─ Agent: did:toadstool:eu-central:load-balancer         │
│     └─ Timestamp: 2025-12-28T15:23:45Z                        │
│     └─ Client IP: 192.168.1.100 (Berlin)                      │
│                                                                │
│  ③ Vertex: GPU Assignment                                     │
│     └─ Agent: did:toadstool:eu-central:node-1                │
│     └─ GPU: did:toadstool:eu-central:node-1:gpu-2            │
│     └─ Parent: [request-received]                             │
│                                                                │
│  ④ Vertex: Inference Execution                                │
│     └─ Agent: did:toadstool:eu-central:node-1:gpu-2          │
│     └─ Duration: 127ms                                        │
│     └─ Tokens: 150                                            │
│     └─ Cost: $0.0045                                          │
│     └─ Parent: [gpu-assignment]                               │
│                                                                │
│  ⑤ Vertex: Response Sent                                      │
│     └─ Agent: did:toadstool:eu-central:load-balancer         │
│     └─ Total latency: 142ms (meets SLA: <200ms)              │
│     └─ Parent: [inference-execution]                          │
│                                                                │
└────────────────────────────────────────────────────────────────┘

EOF
echo -e "${GREEN}✓ Complete request trace with regional attribution${NC}"

# Performance comparison
echo -e "\n${YELLOW}📝 Step 3: Regional performance comparison${NC}"
cat <<'EOF'

┌────────────────────────────────────────────────────────────────┐
│  Regional Performance (24-hour period)                         │
├─────────────┬──────────┬──────────┬──────────┬────────────────┤
│  Region     │ Requests │ Latency  │ Cost     │ Availability   │
├─────────────┼──────────┼──────────┼──────────┼────────────────┤
│ US-West     │ 45,230   │ 98ms     │ $203.54  │ 99.98%  ✅     │
│ EU-Central  │ 38,450   │ 105ms    │ $182.12  │ 99.95%  ✅     │
│ AP-SE       │ 28,120   │ 112ms    │ $145.23  │ 99.92%  ⚠️     │
├─────────────┼──────────┼──────────┼──────────┼────────────────┤
│ Total       │ 111,800  │ 104ms    │ $530.89  │ 99.95%  ✅     │
└─────────────┴──────────┴──────────┴──────────┴────────────────┘

Insights:
  ✅ US-West: Best performance (lowest latency, highest uptime)
  ✅ EU-Central: Good performance, meets SLA
  ⚠️  AP-SE: Slightly below SLA (99.92% vs 99.95% target)

EOF
echo -e "${GREEN}✓ Per-region metrics enable optimization${NC}"

# Cost breakdown
echo -e "\n${YELLOW}📝 Step 4: Regional cost breakdown${NC}"
cat <<'EOF'

┌────────────────────────────────────────────────────────────────┐
│  Cost Accounting by Region (24 hours)                          │
├─────────────┬──────────┬──────────┬──────────┬────────────────┤
│  Region     │ Compute  │ Network  │ Storage  │ Total          │
├─────────────┼──────────┼──────────┼──────────┼────────────────┤
│ US-West     │ $185.30  │ $15.24   │ $3.00    │ $203.54        │
│ EU-Central  │ $165.45  │ $13.67   │ $3.00    │ $182.12        │
│ AP-SE       │ $132.10  │ $10.13   │ $3.00    │ $145.23        │
├─────────────┼──────────┼──────────┼──────────┼────────────────┤
│ Total       │ $482.85  │ $39.04   │ $9.00    │ $530.89        │
└─────────────┴──────────┴──────────┴──────────┴────────────────┘

Cost per Request:
  US-West:    $0.0045 per request
  EU-Central: $0.0047 per request
  AP-SE:      $0.0052 per request  ⚠️ Higher cost

Recommendation: Increase capacity in AP-SE to reduce per-request cost

EOF
echo -e "${GREEN}✓ Regional cost analysis guides resource allocation${NC}"

# Global provenance DAG
echo -e "\n${YELLOW}📝 Step 5: Global provenance DAG${NC}"
cat <<'EOF'

┌────────────────────────────────────────────────────────────────┐
│  Single DAG Captures All Regions                               │
├────────────────────────────────────────────────────────────────┤
│                                                                │
│                    Session Genesis                             │
│                          │                                     │
│         ┌────────────────┼────────────────┐                   │
│         │                │                │                   │
│    🌎 US-West      🌍 EU-Central    🌏 AP-SE                  │
│         │                │                │                   │
│    [Request 1]      [Request 2]      [Request 3]              │
│         │                │                │                   │
│    [GPU Exec 1]     [GPU Exec 2]     [GPU Exec 3]             │
│         │                │                │                   │
│    [Response 1]     [Response 2]     [Response 3]             │
│         │                │                │                   │
│         └────────────────┴────────────────┘                   │
│                          │                                     │
│                   Merkle Root                                  │
│                  (Global Proof)                                │
│                                                                │
│  Benefits:                                                     │
│  • Single cryptographic proof for ALL regions                 │
│  • Cross-region request comparison                            │
│  • Global audit trail                                         │
│  • SLA verification across regions                            │
│                                                                │
└────────────────────────────────────────────────────────────────┘

EOF
echo -e "${GREEN}✓ One DAG unifies all distributed compute${NC}"

# SLA verification
echo -e "\n${YELLOW}📝 Step 6: SLA verification (cryptographic proofs)${NC}"
cat <<'EOF'

// Verify SLA compliance with Merkle proofs

// Claim: "99.95% uptime in EU-Central"
let uptime_proof = session.generate_merkle_proof(
    vertices_for_region("eu-central")
).await?;

// Verify: 38,450 successful / 38,470 total = 99.948% ✅
let successful = uptime_proof.vertices
    .iter()
    .filter(|v| v.metadata["status"] == "success")
    .count();

assert_eq!(successful, 38_450);
assert_eq!(uptime_proof.vertices.len(), 38_470);
let uptime_pct = (successful as f64 / uptime_proof.vertices.len() as f64) * 100.0;
assert!(uptime_pct >= 99.95); // ✅ SLA met

// Claim: "Average latency < 200ms"
let latencies: Vec<f64> = uptime_proof.vertices
    .iter()
    .map(|v| v.metadata["duration_ms"].as_f64().unwrap())
    .collect();

let avg_latency = latencies.iter().sum::<f64>() / latencies.len() as f64;
assert!(avg_latency < 200.0); // ✅ 105ms, SLA met

// Merkle root: Single cryptographic proof for all claims
let merkle_root = session.compute_merkle_root().await?;
println!("Global proof: {}", merkle_root);

EOF
echo -e "${GREEN}✓ Cryptographic SLA verification (tamper-proof)${NC}"

# Benefits
echo -e "\n${YELLOW}📝 Step 7: Benefits of distributed provenance${NC}"
echo -e "${BLUE}   1. Regional Optimization${NC}"
echo -e "      → Identify best-performing regions"
echo -e "      → Allocate resources efficiently"
echo -e ""
echo -e "${BLUE}   2. SLA Compliance${NC}"
echo -e "      → Cryptographic proof of uptime"
echo -e "      → Latency verification per region"
echo -e ""
echo -e "${BLUE}   3. Cost Management${NC}"
echo -e "      → Per-region cost breakdown"
echo -e "      → Identify cost optimization opportunities"
echo -e ""
echo -e "${BLUE}   4. Debugging${NC}"
echo -e "      → Trace requests across regions"
echo -e "      → Identify regional issues"
echo -e ""
echo -e "${BLUE}   5. Global Audit Trail${NC}"
echo -e "      → Single DAG for all regions"
echo -e "      → Unified provenance and compliance"
echo -e ""

# Final summary
echo -e "\n${GREEN}✅ Demo complete!${NC}"
echo -e "\n${BLUE}What you learned:${NC}"
echo "  • Geo-distributed compute with regional DIDs"
echo "  • Single DAG captures all regions"
echo "  • Per-region performance and cost metrics"
echo "  • Cryptographic SLA verification"
echo "  • Global Merkle root for unified provenance"
echo ""
echo -e "${BLUE}Real-world applications:${NC}"
echo "  • Multi-region ML inference"
echo "  • SLA monitoring and compliance"
echo "  • Cost optimization across regions"
echo "  • Regulatory compliance (data locality)"
echo ""
echo -e "${BLUE}Next steps:${NC}"
echo "  • See: ../05-complete-workflows/demo-ml-pipeline.sh"
echo "  • Explore: Complete end-to-end scenarios"
echo ""
echo -e "${BLUE}════════════════════════════════════════════════════════════════${NC}"
