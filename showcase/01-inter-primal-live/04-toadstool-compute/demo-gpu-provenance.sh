#!/bin/bash
# Demo: GPU Hardware Provenance with ToadStool
# Prerequisites: Understanding of DAG-driven compute
# Expected: Hardware-level attribution and cost tracking

set -euo pipefail

# Colors
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m'

echo -e "${BLUE}════════════════════════════════════════════════════════════════${NC}"
echo -e "${BLUE}  🎮 Demo: GPU Hardware Provenance${NC}"
echo -e "${BLUE}════════════════════════════════════════════════════════════════${NC}"
echo ""

# Scenario setup
echo -e "${YELLOW}📝 Scenario: Multi-GPU Training with Cost Tracking${NC}"
echo -e "${BLUE}   Training a large language model across 8 GPUs${NC}"
echo -e "${BLUE}   Tracking: Performance, utilization, power, cost${NC}"
echo ""

# GPU inventory
echo -e "${YELLOW}📝 Step 1: GPU Inventory (Hardware DIDs)${NC}"
cat <<'EOF'

┌─────────────────────────────────────────────────────────────┐
│  GPU Cluster Inventory                                      │
├─────────────────────────────────────────────────────────────┤
│  GPU 0: did:toadstool:node-1:gpu-0 (NVIDIA A100, 80GB)     │
│  GPU 1: did:toadstool:node-1:gpu-1 (NVIDIA A100, 80GB)     │
│  GPU 2: did:toadstool:node-1:gpu-2 (NVIDIA A100, 80GB)     │
│  GPU 3: did:toadstool:node-1:gpu-3 (NVIDIA A100, 80GB)     │
│  GPU 4: did:toadstool:node-2:gpu-0 (NVIDIA A100, 80GB)     │
│  GPU 5: did:toadstool:node-2:gpu-1 (NVIDIA A100, 80GB)     │
│  GPU 6: did:toadstool:node-2:gpu-2 (NVIDIA A100, 80GB)     │
│  GPU 7: did:toadstool:node-2:gpu-3 (NVIDIA A100, 80GB)     │
└─────────────────────────────────────────────────────────────┘

EOF
echo -e "${GREEN}✓ Each GPU has a unique DID for attribution${NC}"

# Training workflow
echo -e "\n${YELLOW}📝 Step 2: Training workflow${NC}"
cat <<'EOF'

┌─────────────────────────────────────────────────────────────┐
│  LLM Training Session                                       │
│  Model: 7B parameters                                       │
│  Dataset: 100GB text corpus                                 │
│  Strategy: Data parallel across 8 GPUs                      │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│  ① Genesis Vertex                                          │
│     └─ Session start: did:key:ml-researcher                │
│                                                             │
│  ② Data Loading (8 parallel vertices)                      │
│     ├─ GPU 0 shard (12.5GB) ← did:toadstool:node-1:gpu-0  │
│     ├─ GPU 1 shard (12.5GB) ← did:toadstool:node-1:gpu-1  │
│     ├─ ... (GPUs 2-6)                                      │
│     └─ GPU 7 shard (12.5GB) ← did:toadstool:node-2:gpu-3  │
│                                                             │
│  ③ Training Step 1 (8 parallel vertices)                   │
│     ├─ GPU 0: Forward pass, backprop, gradient            │
│     ├─ GPU 1: Forward pass, backprop, gradient            │
│     ├─ ... (GPUs 2-6)                                      │
│     └─ GPU 7: Forward pass, backprop, gradient            │
│                                                             │
│  ④ Gradient Sync (coordinator vertex)                      │
│     └─ Aggregate gradients from all 8 GPUs                 │
│                                                             │
│  ⑤ Training Steps 2-1000 (repeated)                        │
│                                                             │
│  ⑥ Model Export                                            │
│     └─ Final weights with complete provenance              │
│                                                             │
└─────────────────────────────────────────────────────────────┘

EOF
echo -e "${GREEN}✓ Complete DAG captures all GPU operations${NC}"

# Performance metrics
echo -e "\n${YELLOW}📝 Step 3: Per-GPU performance metrics${NC}"
cat <<'EOF'

┌────────────────────────────────────────────────────────────────┐
│  GPU Performance Metrics (Training Step 1)                     │
├────┬────────┬─────────┬───────┬────────┬───────────┬──────────┤
│ ID │ Agent  │ Util(%) │ Mem   │ Power  │ Duration  │ Cost     │
├────┼────────┼─────────┼───────┼────────┼───────────┼──────────┤
│ 0  │ gpu-0  │ 98.2%   │ 76GB  │ 345W   │ 1.23s     │ $0.0034  │
│ 1  │ gpu-1  │ 97.8%   │ 75GB  │ 342W   │ 1.25s     │ $0.0034  │
│ 2  │ gpu-2  │ 98.1%   │ 76GB  │ 344W   │ 1.24s     │ $0.0034  │
│ 3  │ gpu-3  │ 94.2%   │ 73GB  │ 330W   │ 1.28s     │ $0.0035  │ ⚠️
│ 4  │ gpu-4  │ 97.9%   │ 75GB  │ 343W   │ 1.25s     │ $0.0034  │
│ 5  │ gpu-5  │ 98.0%   │ 76GB  │ 344W   │ 1.24s     │ $0.0034  │
│ 6  │ gpu-6  │ 97.7%   │ 75GB  │ 341W   │ 1.26s     │ $0.0035  │
│ 7  │ gpu-7  │ 98.3%   │ 76GB  │ 346W   │ 1.23s     │ $0.0034  │
└────┴────────┴─────────┴───────┴────────┴───────────┴──────────┘

EOF
echo -e "${GREEN}✓ Identified: GPU 3 underperforming (94% vs 98% average)${NC}"
echo -e "${YELLOW}  → Action: Investigate thermal throttling on GPU 3${NC}"

# Cost accounting
echo -e "\n${YELLOW}📝 Step 4: Cost and energy accounting${NC}"
cat <<'EOF'

┌────────────────────────────────────────────────────────────────┐
│  Training Session Cost Breakdown                               │
├────────────────────────────────────────────────────────────────┤
│  Duration:        3.5 hours                                    │
│  GPU Hours:       28.0 (8 GPUs × 3.5h)                         │
│  Total Cost:      $98.00                                       │
│  Cost per GPU:    $12.25                                       │
├────────────────────────────────────────────────────────────────┤
│  Energy Usage:    96.6 kWh                                     │
│  Carbon:          48.3 kg CO₂                                  │
│  Efficiency:      3.45 kWh per GPU-hour                        │
└────────────────────────────────────────────────────────────────┘

Per-GPU Breakdown:
  GPU 0: $12.50 (28.2 kWh, 14.1 kg CO₂)  
  GPU 1: $12.25 (28.0 kWh, 14.0 kg CO₂)
  GPU 2: $12.30 (28.1 kWh, 14.1 kg CO₂)
  GPU 3: $11.80 (27.2 kWh, 13.6 kg CO₂) ⚠️ Underutilized
  GPU 4: $12.25 (28.0 kWh, 14.0 kg CO₂)
  GPU 5: $12.30 (28.1 kWh, 14.1 kg CO₂)
  GPU 6: $12.20 (27.9 kWh, 14.0 kg CO₂)
  GPU 7: $12.40 (28.3 kWh, 14.2 kg CO₂)

EOF
echo -e "${GREEN}✓ Complete cost attribution per hardware unit${NC}"

# Vertex metadata example
echo -e "\n${YELLOW}📝 Step 5: Vertex metadata (GPU performance)${NC}"
cat <<'EOF'

// GPU vertex capturing hardware metrics
let gpu_vertex = session.create_vertex(
    EventType::DataUpdate { schema: None },
    did!("did:toadstool:node-1:gpu-0"),
    vec![data_loading_vertex],
    json!({
        // Hardware identification
        "gpu_id": "GPU-0",
        "gpu_model": "NVIDIA A100",
        "memory_gb": 80,
        "cuda_version": "12.0",
        
        // Performance metrics
        "utilization_pct": 98.2,
        "memory_used_gb": 76,
        "temperature_c": 75,
        "power_watts": 345,
        "clock_mhz": 1410,
        
        // Timing
        "duration_sec": 1.23,
        "timestamp": "2025-12-28T10:15:32Z",
        
        // Cost accounting
        "cost_usd": 0.0034,
        "kwh": 0.118,
        "carbon_kg": 0.059,
        
        // Training specifics
        "batch_size": 32,
        "gradient_norm": 2.34,
        "loss": 0.456
    })
).await?;

EOF
echo -e "${GREEN}✓ Rich metadata enables debugging and optimization${NC}"

# Benefits
echo -e "\n${YELLOW}📝 Step 6: Benefits of hardware provenance${NC}"
echo -e "${BLUE}   1. Reproducibility${NC}"
echo -e "      → Know exactly which GPUs were used"
echo -e "      → Reproduce results on same hardware"
echo -e ""
echo -e "${BLUE}   2. Debugging${NC}"
echo -e "      → Identify underperforming GPUs"
echo -e "      → Compare performance across hardware"
echo -e ""
echo -e "${BLUE}   3. Cost Optimization${NC}"
echo -e "      → Track GPU usage per user/project"
echo -e "      → Fair billing and chargeback"
echo -e ""
echo -e "${BLUE}   4. Environmental Impact${NC}"
echo -e "      → Carbon footprint per model"
echo -e "      → Optimize for green compute"
echo -e ""
echo -e "${BLUE}   5. Compliance${NC}"
echo -e "      → Audit trail for AI training"
echo -e "      → Hardware certification tracking"
echo -e ""

# Final summary
echo -e "\n${GREEN}✅ Demo complete!${NC}"
echo -e "\n${BLUE}What you learned:${NC}"
echo "  • Every GPU gets a unique DID for attribution"
echo "  • Per-GPU metrics: utilization, power, cost, carbon"
echo "  • Identify performance bottlenecks (GPU 3 underperforming)"
echo "  • Complete cost and energy accounting"
echo "  • Merkle proofs for hardware-level provenance"
echo ""
echo -e "${BLUE}Real-world applications:${NC}"
echo "  • ML training cost tracking"
echo "  • GPU cluster optimization"
echo "  • Environmental reporting"
echo "  • Compliance and auditing"
echo ""
echo -e "${BLUE}Next steps:${NC}"
echo "  • Try: ./demo-distributed-compute.sh (multi-region)"
echo "  • See: ../05-complete-workflows/demo-ml-pipeline.sh"
echo ""
echo -e "${BLUE}════════════════════════════════════════════════════════════════${NC}"
