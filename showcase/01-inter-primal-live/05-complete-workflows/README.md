# 05: Complete End-to-End Workflows

**Real-world scenarios integrating all primals**

## Overview

These demos showcase complete workflows that integrate `rhizoCrypt` with multiple Phase 1 primals, demonstrating real-world use cases where provenance, storage, compute, and signing work together seamlessly.

## Architecture

```
┌─────────────────────────────────────────────────┐
│              rhizoCrypt (Orchestrator)          │
│                 Ephemeral DAG Engine            │
└────────┬────────┬────────┬────────┬─────────────┘
         │        │        │        │
         ▼        ▼        ▼        ▼
    ┌────────┐ ┌─────────┐ ┌──────────┐ ┌────────┐
    │Songbird│ │NestGate │ │ToadStool │ │BearDog │
    │Discover│ │Storage  │ │Compute   │ │Signing │
    └────────┘ └─────────┘ └──────────┘ └────────┘
```

## Demos

### 1. ML Pipeline (`demo-ml-pipeline.sh`)
**Scenario:** Complete machine learning training workflow with full provenance.

**Workflow:**
1. **Data Preparation** (NestGate)
   - Upload ImageNet dataset (12.5 GB)
   - Content-addressed storage
   - Validation

2. **Training** (ToadStool)
   - ResNet-50 model
   - 8× NVIDIA A100 GPUs
   - 100 epochs, 6.2 hours

3. **Model Storage** (NestGate)
   - Store trained model (98.4 MB)
   - Content-addressed

4. **Model Signing** (BearDog)
   - Cryptographic signature
   - Release authentication

5. **Validation & Approval**
   - QA testing
   - Team lead approval

**Agents:** Data Engineer, ML Engineer, ToadStool, ML Ops, BearDog HSM, QA Engineer, Team Lead

**Run:**
```bash
./demo-ml-pipeline.sh
```

---

### 2. Document Management (`demo-document-workflow.sh`)
**Scenario:** Multi-party contract negotiation with full audit trail.

**Workflow:**
1. **Initial Draft** (Vendor)
   - Service agreement v1
   - Stored in NestGate

2. **Client Edits**
   - Add price and payment terms
   - New version stored

3. **Legal Review** (2 Lawyers)
   - Add liability clauses
   - Jurisdiction selection
   - Approval

4. **Signatures** (BearDog HSM)
   - Client signature
   - Vendor signature
   - Timestamped

**Agents:** Vendor Rep, Client Rep, Lawyer 1, Lawyer 2, Client HSM, Vendor HSM

**Benefits:**
- Complete version history
- Multi-party collaboration
- Cryptographic signatures
- Dispute resolution capability

**Run:**
```bash
./demo-document-workflow.sh
```

---

### 3. Supply Chain (`demo-supply-chain.sh`)
**Scenario:** Farm-to-table coffee bean provenance.

**Workflow:**
1. **Harvesting** (Ethiopia)
   - 500 kg Heirloom variety
   - Organic certified
   - Location & farmer attribution

2. **Processing**
   - Washed, sun-dried
   - 14 days processing
   - Final weight: 420 kg

3. **Quality Control**
   - Grade 1 classification
   - Cupping score: 87/100
   - Export approval

4. **Shipping**
   - Container tracking
   - Djibouti → Rotterdam
   - 17-day transit

5. **Roasting** (Amsterdam)
   - Medium roast
   - Roast profile stored (NestGate)

6. **Packaging**
   - 1,600 packages × 250g
   - Compostable packaging
   - QR code per package

7. **Authentication** (BearDog)
   - Batch signature
   - QR code verification

**Agents:** Farmer, Processor, Inspector, Exporter, Roaster, Packager, BearDog

**Consumer Experience:**
- Scan QR code
- View complete provenance
- Verify authenticity

**Run:**
```bash
./demo-supply-chain.sh
```

---

### 4. Federated Identity (`demo-federated-identity.sh`)
**Scenario:** Cross-organization research collaboration.

**Workflow:**
1. **Project Setup**
   - 3 organizations (University, Lab, Agency)
   - 6 researchers
   - Role-based access grants

2. **Data Collection**
   - University A: Temperature data (1M records)
   - Lab B: Precipitation imagery (50K images)
   - Agency C: Policy data (5K records)
   - All stored in NestGate

3. **Cross-Org Compute** (ToadStool)
   - Lab B analyzes University A's data
   - Authorized cross-org access

4. **Joint Publication**
   - Multi-author paper
   - 3 organizations co-author

5. **Multi-Org Signatures** (BearDog)
   - Each org signs via own HSM
   - Joint authentication

**Agents:** Project Coordinator, 6 Researchers (3 orgs), 3 BearDog HSMs

**Benefits:**
- No central identity provider
- Organizational sovereignty
- Cross-org collaboration
- Fine-grained access control

**Run:**
```bash
./demo-federated-identity.sh
```

---

## Key Patterns

### Workflow Orchestration
```rust
// rhizoCrypt orchestrates, doesn't embed
let session = SessionBuilder::new(SessionType::General)
    .with_name("workflow-name")
    .with_owner(coordinator)
    .build();

// Phase 1: Data (NestGate)
let data_vertex = create_data_vertex(payload);

// Phase 2: Compute (ToadStool)
let compute_vertex = request_compute(data_vertex);

// Phase 3: Sign (BearDog)
let signed_vertex = sign_result(compute_vertex);

// Resolve with full provenance
let resolution = resolve_session(session_id);
```

### Multi-Primal Integration
- **Discovery:** Songbird (capability-based)
- **Storage:** NestGate (content-addressed)
- **Compute:** ToadStool (GPU/ML)
- **Signing:** BearDog (HSM)
- **Provenance:** rhizoCrypt (DAG)

### Zero-Knowledge Discovery
```rust
// No hardcoded endpoints
let storage_client = discover_capability("storage").await?;
let compute_client = discover_capability("compute").await?;
let signing_client = discover_capability("signing").await?;
```

## Benefits Across All Workflows

| Aspect | Benefit |
|--------|---------|
| **Provenance** | Complete audit trail, cryptographic proof |
| **Collaboration** | Multi-agent, multi-org workflows |
| **Storage** | Content-addressed, deduplicated (NestGate) |
| **Compute** | GPU/ML with full attribution (ToadStool) |
| **Trust** | Cryptographic signatures (BearDog) |
| **Discovery** | Zero-knowledge, capability-based (Songbird) |
| **Sovereignty** | No central authority, federated |

## Real-World Impact

### ML Reproducibility
- Dataset → Model lineage
- Training provenance
- Cost and carbon tracking
- Model cards with full history

### Legal Compliance
- Immutable document history
- Multi-party signatures
- Dispute resolution
- Regulatory audit trails

### Consumer Trust
- Supply chain transparency
- Anti-counterfeiting
- Quality verification
- Fair trade attribution

### Research Collaboration
- Cross-org data sharing
- Federated identity
- Joint publications
- Attribution and credit

## Technical Highlights

### Session Management
- Long-running workflows (days/weeks)
- Multi-phase orchestration
- Checkpoint and resume
- Final resolution with Merkle proof

### Agent Attribution
- Every action attributed to DID
- Human agents, services, hardware
- Cross-organizational identity
- No central authority

### Storage Integration
- Large payloads separate from DAG
- Content-addressed (deduplication)
- Version history maintained
- ZFS features (compression, snapshots)

### Compute Integration
- GPU-level attribution
- Performance metrics
- Cost tracking
- Distributed execution

### Signature Integration
- HSM-backed signing
- Multi-party signatures
- Timestamped
- Algorithm flexibility (Ed25519, etc.)

## Running All Demos

```bash
# Complete ML workflow
./demo-ml-pipeline.sh

# Document collaboration
./demo-document-workflow.sh

# Supply chain tracking
./demo-supply-chain.sh

# Federated research
./demo-federated-identity.sh
```

## Next Steps

Explore the full showcase:
```bash
cd ../..
cat SHOWCASE_COMPLETE.md
```

---

**No mocks. Real primals. Real workflows. Complete provenance.**

**This is the ecoPrimals ecosystem in action!** 🌱

