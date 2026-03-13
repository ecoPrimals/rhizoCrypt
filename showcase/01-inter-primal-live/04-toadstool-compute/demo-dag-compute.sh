#!/bin/bash
# Demo: DAG-Driven Compute Workflow with ToadStool
# Prerequisites: ToadStool binary available
# Expected: Complete ML workflow with provenance tracking

set -euo pipefail

# Colors
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m'

# Paths
TOADSTOOL_BIN="/path/to/ecoPrimals/primalBins/toadstool-cli"
RHIZO_BIN="/path/to/ecoPrimals/phase2/rhizoCrypt/target/release/rhizocrypt"

echo -e "${BLUE}════════════════════════════════════════════════════════════════${NC}"
echo -e "${BLUE}  🍄 Demo: DAG-Driven Compute with ToadStool${NC}"
echo -e "${BLUE}════════════════════════════════════════════════════════════════${NC}"
echo ""

# Check binaries
echo -e "${YELLOW}📝 Step 1: Check binaries${NC}"
if [ ! -f "$TOADSTOOL_BIN" ]; then
    echo -e "${RED}✗ ToadStool binary not found at: $TOADSTOOL_BIN${NC}"
    echo -e "${YELLOW}  Build ToadStool or check path in primalBins/${NC}"
    exit 1
fi
echo -e "${GREEN}✓ ToadStool binary found${NC}"

if [ ! -f "$RHIZO_BIN" ]; then
    echo -e "${YELLOW}⚠  rhizoCrypt service not built, building now...${NC}"
    cd ../../../ && cargo build --release -p rhizocrypt-service
    cd -
fi
echo -e "${GREEN}✓ rhizoCrypt service ready${NC}"

# Check ToadStool capabilities
echo -e "\n${YELLOW}📝 Step 2: Discover ToadStool capabilities${NC}"
echo -e "${BLUE}   Running: $TOADSTOOL_BIN --help${NC}"
if "$TOADSTOOL_BIN" --help > /tmp/toadstool-help.txt 2>&1; then
    echo -e "${GREEN}✓ ToadStool CLI responsive${NC}"
    head -n 5 /tmp/toadstool-help.txt
else
    echo -e "${RED}✗ ToadStool CLI not responding${NC}"
    echo -e "${YELLOW}  This demo shows the intended integration pattern${NC}"
fi

# Simulate DAG workflow (conceptual demonstration)
echo -e "\n${YELLOW}📝 Step 3: Create compute workflow DAG${NC}"
cat > /tmp/compute-workflow.json <<'EOF'
{
  "workflow": "ml-training",
  "steps": [
    {
      "id": "dataset-prep",
      "type": "data-update",
      "agent": "did:key:researcher",
      "description": "Load and preprocess MNIST dataset"
    },
    {
      "id": "compute-request",
      "type": "data-update", 
      "agent": "did:key:ml-pipeline",
      "parents": ["dataset-prep"],
      "metadata": {
        "compute_type": "gpu",
        "framework": "pytorch",
        "gpus_requested": 1
      }
    },
    {
      "id": "toadstool-execution",
      "type": "data-update",
      "agent": "did:toadstool:gpu-0",
      "parents": ["compute-request"],
      "metadata": {
        "gpu_model": "NVIDIA GPU",
        "utilization": "95%",
        "duration_sec": 45
      }
    },
    {
      "id": "result-validation",
      "type": "data-update",
      "agent": "did:key:researcher",
      "parents": ["toadstool-execution"],
      "metadata": {
        "accuracy": "0.98",
        "validation": "passed"
      }
    }
  ]
}
EOF

echo -e "${GREEN}✓ Workflow DAG defined${NC}"
echo -e "${BLUE}   Workflow structure:${NC}"
echo -e "     dataset-prep → compute-request → toadstool-execution → result-validation"

# Show provenance hierarchy
echo -e "\n${YELLOW}📝 Step 4: Provenance hierarchy${NC}"
cat <<'EOF'

┌─────────────────────────────────────────────────────────┐
│  Session: ML Training Experiment                       │
│  DID: did:key:researcher                               │
├─────────────────────────────────────────────────────────┤
│                                                         │
│  ① Dataset Preparation                                 │
│     └─ Agent: did:key:researcher                       │
│        └─ Event: Load MNIST, normalize, split          │
│                                                         │
│  ② Compute Request                                     │
│     └─ Agent: did:key:ml-pipeline                      │
│        └─ Parents: [dataset-prep]                      │
│        └─ Metadata: {gpu, pytorch, 1 GPU}              │
│                                                         │
│  ③ ToadStool Execution                                 │
│     └─ Agent: did:toadstool:gpu-0                      │
│        └─ Parents: [compute-request]                   │
│        └─ Hardware: NVIDIA GPU @ 95% util              │
│        └─ Duration: 45 seconds                         │
│                                                         │
│  ④ Result Validation                                   │
│     └─ Agent: did:key:researcher                       │
│        └─ Parents: [toadstool-execution]               │
│        └─ Accuracy: 0.98 (98% correct)                 │
│        └─ Status: PASSED                               │
│                                                         │
└─────────────────────────────────────────────────────────┘

EOF
echo -e "${GREEN}✓ Complete provenance chain captured${NC}"

# Key benefits
echo -e "\n${YELLOW}📝 Step 5: Key benefits of DAG-driven compute${NC}"
echo -e "${BLUE}   1. Reproducibility${NC}"
echo -e "      → Full training lineage from raw data to final model"
echo -e "      → Every step cryptographically linked"
echo -e ""
echo -e "${BLUE}   2. Hardware Attribution${NC}"
echo -e "      → Each GPU has a DID (did:toadstool:gpu-N)"
echo -e "      → Track utilization, power, cost per GPU"
echo -e ""
echo -e "${BLUE}   3. Debugging & Optimization${NC}"
echo -e "      → Identify bottlenecks in the DAG"
echo -e "      → Compare performance across runs"
echo -e ""
echo -e "${BLUE}   4. Compliance & Auditing${NC}"
echo -e "      → Merkle proofs for entire workflow"
echo -e "      → Tamper-evident compute records"
echo -e ""

# Integration pattern
echo -e "\n${YELLOW}📝 Step 6: Integration pattern (capability-based)${NC}"
cat <<'EOF'

// No hardcoded ToadStool endpoints!
let compute_client = CapabilityRegistry::discover("ComputeProvider")
    .with_requirement("gpu")
    .with_framework("pytorch")
    .await?;

// Submit job through discovered capability
let job_id = compute_client.submit_job(JobSpec {
    framework: "pytorch",
    script: "train.py",
    resources: Resources { gpus: 1 },
}).await?;

// Track execution in rhizoCrypt DAG
let compute_vertex = session.create_vertex(
    EventType::DataUpdate { schema: None },
    did!("did:toadstool:gpu-0"),
    vec![request_vertex],
    ComputeMetadata {
        job_id,
        gpu_model: "NVIDIA GPU",
        utilization: 0.95,
        duration_sec: 45,
    }
).await?;

EOF
echo -e "${GREEN}✓ Zero vendor lock-in, pure capability discovery${NC}"

# Real-world scenarios
echo -e "\n${YELLOW}📝 Step 7: Real-world use cases${NC}"
echo -e "${BLUE}   ① ML Training${NC}"
echo -e "      Dataset → Preprocessing → Training → Validation → Model"
echo -e "      Full reproducibility for research papers"
echo -e ""
echo -e "${BLUE}   ② Distributed Inference${NC}"
echo -e "      Multi-region GPU deployment with per-node attribution"
echo -e "      SLA verification and cost tracking"
echo -e ""
echo -e "${BLUE}   ③ Cost Accounting${NC}"
echo -e "      GPU hours per user/project"
echo -e "      Energy consumption and carbon footprint"
echo -e ""

# Final summary
echo -e "\n${GREEN}✅ Demo complete!${NC}"
echo -e "\n${BLUE}What you learned:${NC}"
echo "  • rhizoCrypt tracks compute workflow as a DAG"
echo "  • Each GPU/hardware gets a unique DID"
echo "  • Complete provenance from data to model"
echo "  • Capability-based discovery (no hardcoding)"
echo "  • Merkle proofs for tamper-evident compute"
echo ""
echo -e "${YELLOW}⚠️  Current status:${NC}"
echo "  • Pattern demonstrated (conceptual)"
echo "  • ToadStool CLI available but integration API pending"
echo "  • Full working demo requires ToadStool RPC service"
echo ""
echo -e "${BLUE}Next steps:${NC}"
echo "  • Try: ./demo-gpu-provenance.sh (hardware attribution)"
echo "  • Try: ./demo-distributed-compute.sh (multi-node)"
echo "  • See: ../05-complete-workflows/demo-ml-pipeline.sh"
echo ""

# Cleanup
rm -f /tmp/compute-workflow.json /tmp/toadstool-help.txt

echo -e "${BLUE}════════════════════════════════════════════════════════════════${NC}"
