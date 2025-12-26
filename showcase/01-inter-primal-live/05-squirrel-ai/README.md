# 🐿️ rhizoCrypt + Squirrel Integration

**Purpose**: AI provenance tracking with complete training and inference lineage  
**Status**: Phase 5 - AI & Machine Learning Integration  
**Binary**: `../../../bins/squirrel`

---

## 🎯 Goal

Integrate rhizoCrypt with Squirrel to add:
1. **Model Training Lineage** - Track data → training → model provenance
2. **Inference Auditability** - Cryptographic proof of AI decisions
3. **AI Provenance Chain** - Complete lineage from data to decision
4. **Human Dignity** - Ensure AI systems respect human sovereignty

---

## 🐿️ Squirrel Capabilities

Squirrel provides:
- **Training Provenance** (what data trained this model?)
- **Inference Tracking** (what model made this decision?)
- **Model Versioning** (immutable model snapshots)
- **Human-in-Loop** (sovereign approval gates)
- **Explainability** (why did the AI decide this?)
- **Bias Detection** (identify discriminatory patterns)

---

## 📁 Demos

### 1. Model Training Lineage (`demo-training-lineage.sh`)
**What it does**:
- Loads training dataset
- Trains ML model
- Records complete training provenance
- Creates cryptographic audit trail

**Run**:
```bash
./demo-training-lineage.sh
```

**Expected Output**:
```
✅ Dataset: 1000 training examples (sha256:a4b3c2...)
✅ Training started: Model v1.0
✅ Training complete: Accuracy 94.2%, Loss 0.058
✅ Model hash: sha256:f3e4d5...
✅ Provenance vertex created
   - Input data: NestGate payload
   - Training agent: did:key:agent-ml
   - Model output: NestGate model store
   - Signature: BearDog cryptographic proof
```

---

### 2. Inference Auditability (`demo-inference-audit.sh`)
**What it does**:
- Loads trained model
- Makes prediction on new data
- Signs inference result
- Records full inference provenance

**Run**:
```bash
./demo-inference-audit.sh
```

**Expected Output**:
```
✅ Model loaded: model-v1.0 (sha256:f3e4d5...)
✅ Input: [features...] (sha256:b2c3d4...)
✅ Prediction: Class A (confidence: 87.3%)
✅ Inference provenance:
   - Model version: v1.0
   - Input hash: sha256:b2c3d4...
   - Output: {"class": "A", "confidence": 0.873}
   - Signed by: did:key:inference-agent
   - Timestamp: 2025-12-26T10:45:00Z
✅ Complete audit trail recorded
```

---

### 3. AI Provenance Chain (`demo-ai-provenance.sh`)
**What it does**:
- Demonstrates full AI lifecycle
- Data collection → Training → Inference → Decision
- Shows complete cryptographic chain
- Includes human approval gates

**Run**:
```bash
./demo-ai-provenance.sh
```

**Expected Output**:
```
✅ AI Provenance Chain:

┌─────────────────────────────────────────┐
│ 1. Data Collection                      │
│    - Source: sensor network             │
│    - Records: 10,000 samples            │
│    - Hash: sha256:a1b2c3...             │
│    - Signed: did:key:data-collector     │
└─────────────────────────────────────────┘
                 ↓
┌─────────────────────────────────────────┐
│ 2. Human Approval (Sovereign Gate)     │
│    - Approved by: did:key:data-steward  │
│    - Purpose: "Train safety model"      │
│    - Timestamp: 2025-12-26T09:00:00Z    │
└─────────────────────────────────────────┘
                 ↓
┌─────────────────────────────────────────┐
│ 3. Model Training                       │
│    - Algorithm: Random Forest           │
│    - Accuracy: 96.8%                    │
│    - Model: sha256:f3e4d5...            │
│    - Signed: did:key:ml-trainer         │
└─────────────────────────────────────────┘
                 ↓
┌─────────────────────────────────────────┐
│ 4. Model Approval (Sovereign Gate)     │
│    - Approved by: did:key:model-auditor │
│    - Bias check: PASSED                 │
│    - Safety check: PASSED               │
└─────────────────────────────────────────┘
                 ↓
┌─────────────────────────────────────────┐
│ 5. Inference (Real-world Use)          │
│    - Input: real-time sensor data       │
│    - Prediction: "SAFE" (99.2%)         │
│    - Decision hash: sha256:e4f5g6...    │
│    - Signed: did:key:inference-agent    │
└─────────────────────────────────────────┘

✅ Complete provenance chain verified
✅ All signatures valid
✅ Human dignity preserved (2 approval gates)
```

---

## 🔄 Integration Pattern

```
┌─────────────┐
│   Training  │
│    Data     │
│ (NestGate)  │
└──────┬──────┘
       │
       │ 1. Load data
       ▼
┌─────────────┐
│   Squirrel  │
│   Training  │
└──────┬──────┘
       │
       │ 2. Train model
       ▼
┌─────────────┐
│    Model    │
│ (versioned) │
└──────┬──────┘
       │
       │ 3. Store model
       ▼
┌─────────────┐
│  NestGate   │
│Model Storage│
└──────┬──────┘
       │
       │ 4. Sign & record
       ▼
┌─────────────┐
│ rhizoCrypt  │
│  Provenance │
│   Vertex    │
└─────────────┘
```

**Key Insight**: AI provenance requires tracking:
- **Data lineage**: Where did training data come from?
- **Training process**: What algorithm? What hyperparameters?
- **Model version**: Immutable model snapshot
- **Inference**: What inputs produced what outputs?
- **Human approval**: Sovereign gates for data use and model deployment

---

## 🧠 AI Provenance Model

### Training Provenance
```json
{
  "event_type": "ai.model_trained",
  "training_data": {
    "dataset_hash": "sha256:a1b2c3...",
    "record_count": 10000,
    "features": ["temperature", "pressure", "humidity"],
    "source": "nestgate://datasets/sensor-2025-12"
  },
  "training_config": {
    "algorithm": "random_forest",
    "hyperparameters": {
      "n_estimators": 100,
      "max_depth": 10
    },
    "training_duration_secs": 45
  },
  "model_output": {
    "model_hash": "sha256:f3e4d5...",
    "accuracy": 0.968,
    "loss": 0.032,
    "storage": "nestgate://models/safety-v1.0"
  },
  "agent": "did:key:ml-trainer",
  "signature": "...",
  "timestamp": "2025-12-26T10:00:00Z"
}
```

### Inference Provenance
```json
{
  "event_type": "ai.inference",
  "model": {
    "hash": "sha256:f3e4d5...",
    "version": "v1.0",
    "source": "nestgate://models/safety-v1.0"
  },
  "input": {
    "data_hash": "sha256:b2c3d4...",
    "features": [22.5, 1013.25, 58.3]
  },
  "output": {
    "prediction": "SAFE",
    "confidence": 0.992,
    "explanation": "All sensors within normal range"
  },
  "agent": "did:key:inference-agent",
  "signature": "...",
  "timestamp": "2025-12-26T10:45:00Z"
}
```

---

## 🛡️ Human Dignity & Sovereignty

Squirrel enforces **human sovereignty** through:

1. **Approval Gates**: Humans approve data use and model deployment
2. **Explainability**: AI decisions must be explainable
3. **Bias Detection**: Automatic detection of discriminatory patterns
4. **Transparency**: Complete provenance visible to affected humans
5. **Right to Challenge**: Humans can challenge AI decisions

**Example Approval Gate**:
```json
{
  "event_type": "human.approval",
  "approver": "did:key:data-steward",
  "approval_type": "data_use",
  "purpose": "Train safety monitoring model",
  "data_scope": "sensor-network-2025-12",
  "approved_at": "2025-12-26T09:00:00Z",
  "signature": "..."
}
```

---

## 🚀 Prerequisites

### 1. Squirrel Binary
```bash
# Check if squirrel binary exists
ls ../../../bins/squirrel

# If not, it needs to be built from phase1/squirrel
```

### 2. Start Squirrel Service
```bash
# Default port: 9800
../../../bins/squirrel --port 9800 --storage ./squirrel-models
```

### 3. Verify Service
```bash
curl http://localhost:9800/health
# Expected: {"status":"healthy","models":0}
```

---

## 🎓 Learning Progression

1. **Start with `demo-training-lineage.sh`** - Basic training provenance
2. **Then `demo-inference-audit.sh`** - Understand inference tracking
3. **Finally `demo-ai-provenance.sh`** - Complete AI lifecycle

---

## 📝 Notes

- **Immutable**: Models are versioned and immutable
- **Auditable**: Every inference is cryptographically signed
- **Explainable**: AI decisions include explanations
- **Sovereign**: Human approval gates protect dignity
- **Provenance**: Complete lineage from data to decision

---

## 🔍 Troubleshooting

### Squirrel not responding
```bash
netstat -tuln | grep 9800
tail -f squirrel.log
```

### Training fails
```bash
# Check training data
curl http://localhost:9800/api/v1/datasets

# Verify model storage
ls ./squirrel-models/
```

### Inference errors
```bash
# Check model is loaded
curl http://localhost:9800/api/v1/models/{model_hash}

# Verify input format
```

---

## 🔗 Related Demos

- **04-toadstool-compute**: Distribute AI training across agents
- **03-nestgate-storage**: Store models and datasets
- **06-complete-workflow**: Full AI + provenance workflow

---

## 📚 Further Reading

- `specs/INTEGRATION_SPECIFICATION.md` - Squirrel integration
- `specs/HUMAN_DIGNITY.md` - Sovereignty principles
- `specs/AI_PROVENANCE.md` - AI-specific provenance

---

*Last Updated: Dec 26, 2025*

