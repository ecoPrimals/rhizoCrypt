#!/bin/bash
# Demo: Complete ML Pipeline with Multi-Primal Integration
# Prerequisites: Understanding of all primals (rhizoCrypt, ToadStool, BearDog, NestGate, LoamSpine)
# Expected: End-to-end ML training with full provenance

set -euo pipefail

# Colors
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
CYAN='\033[0;36m'
PURPLE='\033[0;35m'
NC='\033[0m'

echo -e "${BLUE}════════════════════════════════════════════════════════════════${NC}"
echo -e "${BLUE}  🤖 Complete ML Pipeline: Multi-Primal Integration${NC}"
echo -e "${BLUE}════════════════════════════════════════════════════════════════${NC}"
echo ""

# Workflow overview
echo -e "${YELLOW}📝 Scenario: Train ML model with full provenance${NC}"
echo -e "${BLUE}   Dataset → Preprocessing → Training → Validation → Storage${NC}"
echo -e "${BLUE}   All steps tracked in rhizoCrypt DAG${NC}"
echo ""

# Architecture
echo -e "${YELLOW}📝 Step 1: Multi-primal architecture${NC}"
cat <<'EOF'

┌─────────────────────────────────────────────────────────────────┐
│  Complete ML Pipeline (Multi-Primal)                            │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  ① Data Acquisition                                            │
│     └─ NestGate: Fetch MNIST dataset                           │
│        └─ Agent: did:nestgate:data-provider                    │
│        └─ Payload: 60,000 training images                      │
│                                                                 │
│  ② Provenance Tracking                                         │
│     └─ rhizoCrypt: Create session, track DAG                   │
│        └─ Agent: did:key:ml-researcher                         │
│        └─ Event: dataset-loaded                                │
│                                                                 │
│  ③ Preprocessing                                               │
│     └─ ToadStool: Normalize, augment, split                    │
│        └─ Agent: did:toadstool:preprocessor                    │
│        └─ Event: preprocessing-complete                        │
│                                                                 │
│  ④ Training                                                    │
│     └─ ToadStool: GPU training (8 epochs)                      │
│        └─ Agent: did:toadstool:gpu-0                           │
│        └─ Event: training-step-N (one per epoch)               │
│                                                                 │
│  ⑤ Signature                                                   │
│     └─ BearDog: Sign final model weights                       │
│        └─ Agent: did:beardog:hsm-1                             │
│        └─ Event: model-signed                                  │
│                                                                 │
│  ⑥ Storage                                                     │
│     └─ NestGate: Store model + metadata                        │
│        └─ Agent: did:nestgate:model-registry                   │
│        └─ Event: model-stored                                  │
│                                                                 │
│  ⑦ Dehydration                                                 │
│     └─ LoamSpine: Commit session for reproducibility           │
│        └─ Agent: did:loamspine:archiver                        │
│        └─ Event: session-committed                             │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘

EOF
echo -e "${GREEN}✓ 5 primals working together!${NC}"

# DAG structure
echo -e "\n${YELLOW}📝 Step 2: DAG structure (provenance chain)${NC}"
cat <<'EOF'

Session DAG:
════════════════════════════════════════════════════════════════
                      Genesis (Session Start)
                            │
                            ↓
                   ① Dataset Loaded (NestGate)
                            │
                            ↓
                   ② Preprocessing (ToadStool)
                            │
                    ┌───────┴───────┐
                    │               │
            ③ Training Epoch 1   Training Epoch 2
            (ToadStool GPU)      (ToadStool GPU)
                    │               │
                    └───────┬───────┘
                            │
                    ... (Epochs 3-8) ...
                            │
                            ↓
                    ④ Model Weights (ToadStool)
                            │
                            ↓
                    ⑤ Model Signed (BearDog)
                            │
                            ↓
                    ⑥ Model Stored (NestGate)
                            │
                            ↓
                    ⑦ Session Committed (LoamSpine)

Merkle Root: Single cryptographic proof of entire pipeline!

EOF
echo -e "${GREEN}✓ Complete provenance from raw data to final model${NC}"

# Step-by-step workflow
echo -e "\n${YELLOW}📝 Step 3: Execute workflow (conceptual)${NC}"

echo -e "\n${CYAN}  Step 3.1: Fetch dataset from NestGate${NC}"
cat <<'EOF'

// Discover NestGate via capabilities
let storage = CapabilityRegistry::discover("StorageProvider")
    .with_capability("payload-management")
    .await?;

// Fetch MNIST dataset
let dataset_payload = storage.fetch_payload(
    "mnist-training-60k"
).await?;

// Track in rhizoCrypt
let dataset_vertex = session.create_vertex(
    EventType::DataUpdate { schema: Some("dataset") },
    did!("did:nestgate:data-provider"),
    vec![genesis_vertex],
    json!({
        "dataset": "MNIST",
        "size": 60_000,
        "source": "NestGate",
        "payload_hash": dataset_payload.hash(),
    })
).await?;

EOF
echo -e "${GREEN}✓ Step 3.1 complete: Dataset loaded${NC}"

echo -e "\n${CYAN}  Step 3.2: Preprocess with ToadStool${NC}"
cat <<'EOF'

// Discover ToadStool compute
let compute = CapabilityRegistry::discover("ComputeProvider")
    .with_capability("data-preprocessing")
    .await?;

// Submit preprocessing job
let preprocess_job = compute.submit_job(JobSpec {
    name: "mnist-preprocessing",
    script: "normalize.py",
    inputs: vec![dataset_payload],
    resources: Resources { cpus: 4, memory_gb: 8 },
}).await?;

// Track in rhizoCrypt
let preprocess_vertex = session.create_vertex(
    EventType::DataUpdate { schema: Some("preprocessing") },
    did!("did:toadstool:preprocessor"),
    vec![dataset_vertex],
    json!({
        "job_id": preprocess_job.id,
        "operations": ["normalize", "augment", "split"],
        "train_size": 50_000,
        "val_size": 10_000,
    })
).await?;

EOF
echo -e "${GREEN}✓ Step 3.2 complete: Data preprocessed${NC}"

echo -e "\n${CYAN}  Step 3.3: Train model with ToadStool GPU${NC}"
cat <<'EOF'

// Submit training job (8 epochs)
let training_job = compute.submit_job(JobSpec {
    name: "mnist-training",
    script: "train.py",
    inputs: vec![preprocess_job.output],
    resources: Resources { gpus: 1, memory_gb: 16 },
    params: json!({
        "model": "cnn",
        "epochs": 8,
        "batch_size": 128,
        "learning_rate": 0.001,
    }),
}).await?;

// Track each epoch in DAG
let mut parent = preprocess_vertex;
for epoch in 1..=8 {
    let epoch_vertex = session.create_vertex(
        EventType::DataUpdate { schema: Some("training-epoch") },
        did!("did:toadstool:gpu-0"),
        vec![parent],
        json!({
            "epoch": epoch,
            "loss": 0.5 - (epoch as f64 * 0.05),  // Simulated
            "accuracy": 0.7 + (epoch as f64 * 0.03),
            "duration_sec": 45,
            "gpu_utilization": 0.95,
        })
    ).await?;
    parent = epoch_vertex;
}

let final_model_vertex = parent;

EOF
echo -e "${GREEN}✓ Step 3.3 complete: Model trained (8 epochs)${NC}"

echo -e "\n${CYAN}  Step 3.4: Sign model with BearDog${NC}"
cat <<'EOF'

// Discover BearDog HSM
let signing = CapabilityRegistry::discover("SigningProvider")
    .with_capability("hsm-signing")
    .await?;

// Sign model weights
let model_weights_hash = compute.get_job_output_hash(training_job.id).await?;
let signature = signing.sign(
    model_weights_hash.as_bytes(),
    &did!("did:key:ml-researcher")
).await?;

// Track signature in DAG
let signature_vertex = session.create_vertex(
    EventType::DataUpdate { schema: Some("model-signature") },
    did!("did:beardog:hsm-1"),
    vec![final_model_vertex],
    json!({
        "model_hash": model_weights_hash,
        "signature": signature.to_string(),
        "signer": "did:key:ml-researcher",
        "timestamp": Utc::now(),
    })
).await?;

EOF
echo -e "${GREEN}✓ Step 3.4 complete: Model signed${NC}"

echo -e "\n${CYAN}  Step 3.5: Store model in NestGate${NC}"
cat <<'EOF'

// Store model + metadata
let model_payload = Payload {
    content: training_job.output,
    metadata: json!({
        "model_type": "cnn",
        "dataset": "MNIST",
        "accuracy": 0.98,
        "signature": signature.to_string(),
        "provenance_session": session.id(),
    }),
};

let storage_id = storage.store_payload(model_payload).await?;

// Track storage in DAG
let storage_vertex = session.create_vertex(
    EventType::DataUpdate { schema: Some("model-storage") },
    did!("did:nestgate:model-registry"),
    vec![signature_vertex],
    json!({
        "storage_id": storage_id,
        "model_name": "mnist-cnn-v1",
        "version": "1.0.0",
        "registry": "NestGate",
    })
).await?;

EOF
echo -e "${GREEN}✓ Step 3.5 complete: Model stored${NC}"

echo -e "\n${CYAN}  Step 3.6: Commit session to LoamSpine${NC}"
cat <<'EOF'

// Dehydrate session for permanent provenance
let summary = session.generate_summary().await?;
let merkle_root = session.compute_merkle_root().await?;

// Discover LoamSpine permanent storage
let permanent = CapabilityRegistry::discover("PermanentStorageProvider")
    .with_capability("provenance-archival")
    .await?;

// Commit to LoamSpine
let commit_id = permanent.commit_session(CommitRequest {
    session_id: session.id(),
    summary,
    merkle_root,
    metadata: json!({
        "workflow": "ml-training",
        "model": "mnist-cnn-v1",
        "dataset": "MNIST",
        "timestamp": Utc::now(),
    }),
}).await?;

// Track commit in DAG (final vertex!)
let commit_vertex = session.create_vertex(
    EventType::DataUpdate { schema: Some("session-commit") },
    did!("did:loamspine:archiver"),
    vec![storage_vertex],
    json!({
        "commit_id": commit_id,
        "merkle_root": merkle_root,
        "vertex_count": session.vertex_count().await?,
        "storage": "LoamSpine",
    })
).await?;

EOF
echo -e "${GREEN}✓ Step 3.6 complete: Session committed to LoamSpine${NC}"

# Provenance benefits
echo -e "\n${YELLOW}📝 Step 4: Provenance benefits${NC}"
cat <<'EOF'

What we can now prove:
═════════════════════════════════════════════════════════════════
  ✅ Dataset source: MNIST from NestGate (content hash verified)
  ✅ Preprocessing: Exact steps (normalize, augment, split)
  ✅ Training: Every epoch tracked (loss, accuracy, GPU used)
  ✅ Signature: Model signed by ML researcher (BearDog HSM)
  ✅ Storage: Model stored in NestGate (immutable)
  ✅ Provenance: Full DAG committed to LoamSpine (permanent)

Reproducibility:
  → Checkout session slice from LoamSpine
  → Re-run training with same dataset
  → Verify results match (cryptographically)

Auditability:
  → Show complete provenance chain
  → Prove no tampering (Merkle root)
  → Attribute work to specific GPUs/agents

Compliance:
  → Model cards with full lineage
  → Regulatory audit trail
  → Fair use verification

EOF
echo -e "${GREEN}✓ Complete ML provenance: Dataset → Model → Archive${NC}"

# Query examples
echo -e "\n${YELLOW}📝 Step 5: Query provenance${NC}"
cat <<'EOF'

// Question: What dataset was used?
let dataset_vertex = session.find_vertices_by_schema("dataset").await?;
println!("Dataset: {}", dataset_vertex[0].metadata["dataset"]);
// → "MNIST"

// Question: Which GPU trained this model?
let training_vertices = session.find_vertices_by_schema("training-epoch").await?;
for v in training_vertices {
    println!("Epoch {}: GPU {}, Accuracy {}",
        v.metadata["epoch"],
        v.agent,  // → did:toadstool:gpu-0
        v.metadata["accuracy"]
    );
}

// Question: Who signed this model?
let sig_vertex = session.find_vertices_by_schema("model-signature").await?;
println!("Signed by: {}", sig_vertex[0].metadata["signer"]);
// → "did:key:ml-researcher"

// Question: Can I reproduce this?
let commit_vertex = session.find_vertices_by_schema("session-commit").await?;
let commit_id = commit_vertex[0].metadata["commit_id"];

// Checkout from LoamSpine
let historical_session = loamspine.checkout_session(commit_id).await?;
// → Full DAG restored, can re-run!

EOF
echo -e "${GREEN}✓ Complete audit trail, fully queryable${NC}"

# Final summary
echo -e "\n${GREEN}✅ Complete ML Pipeline Demo!${NC}"
echo -e "\n${BLUE}Primals used:${NC}"
echo "  🔐 rhizoCrypt: DAG provenance tracking"
echo "  🍄 ToadStool: Data preprocessing + GPU training"
echo "  🐻 BearDog: Model signature (HSM)"
echo "  🏰 NestGate: Dataset + model storage"
echo "  🌾 LoamSpine: Permanent provenance archive"
echo ""
echo -e "${BLUE}Workflow steps:${NC}"
echo "  ① Fetch dataset (NestGate)"
echo "  ② Preprocess data (ToadStool)"
echo "  ③ Train model 8 epochs (ToadStool GPU)"
echo "  ④ Sign model (BearDog HSM)"
echo "  ⑤ Store model (NestGate)"
echo "  ⑥ Commit provenance (LoamSpine)"
echo ""
echo -e "${BLUE}Benefits:${NC}"
echo "  • Complete reproducibility"
echo "  • Cryptographic audit trail"
echo "  • Regulatory compliance"
echo "  • Fair attribution (per-GPU)"
echo "  • Permanent provenance"
echo ""
echo -e "${CYAN}🏆 This is the power of ecoPrimals!${NC}"
echo -e "${CYAN}   5 primals, 1 workflow, complete provenance${NC}"
echo ""
echo -e "${BLUE}Next steps:${NC}"
echo "  • Try: demo-document-collaboration.sh"
echo "  • Try: demo-supply-chain.sh"
echo "  • See: ../../README.md for more workflows"
echo ""
echo -e "${BLUE}════════════════════════════════════════════════════════════════${NC}"
