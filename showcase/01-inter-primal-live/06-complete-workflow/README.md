# 🌐 Complete Multi-Primal Workflows

**Purpose**: Demonstrate end-to-end integration of all Phase 1 primals with rhizoCrypt  
**Status**: Phase 6 - Complete Integration  
**Primals**: Songbird + BearDog + NestGate + LoamSpine + ToadStool + Squirrel

---

## 🎯 Goal

Demonstrate complete workflows that integrate ALL primals:
1. **Simple Workflow** - Basic create → sign → store → commit
2. **Multi-Primal Workflow** - Discovery → sign → store → compute → commit → query
3. **AI Workflow** - Complete AI lifecycle with provenance

---

## 🌟 Complete Integration

These workflows demonstrate the **full power** of the ecoPrimals ecosystem:

```
┌──────────────────────────────────────────────────────────────┐
│                    rhizoCrypt Session                        │
│                   (Coordination Layer)                       │
└────────────┬─────────────────────────────────────────────────┘
             │
     ┌───────┴───────┐
     │   Songbird    │  ← Service Discovery
     │  (Registry)   │
     └───────┬───────┘
             │
     ┌───────┴───────────────────────────────────────┐
     │                                               │
┌────▼────┐  ┌─────────┐  ┌─────────┐  ┌──────────┐│
│ BearDog │  │NestGate │  │ToadStool│  │ Squirrel ││
│(Signing)│  │(Storage)│  │(Compute)│  │   (AI)   ││
└────┬────┘  └────┬────┘  └────┬────┘  └────┬─────┘│
     │            │            │            │       │
     └────────────┴────────────┴────────────┴───────┘
                           │
                    ┌──────▼──────┐
                    │  LoamSpine  │  ← Permanent Commit
                    │ (Permanent) │
                    └─────────────┘
```

---

## 📁 Workflows

### 1. Simple Workflow (`demo-simple-workflow.sh`)
**What it does**:
- Creates rhizoCrypt session
- Signs vertices with BearDog
- Stores payloads in NestGate
- Commits to LoamSpine
- Basic end-to-end flow

**Run**:
```bash
./demo-simple-workflow.sh
```

**Flow**:
```
1. Create Session (rhizoCrypt)
   ↓
2. Add Vertices with Data
   ↓
3. Store Payloads (NestGate)
   ↓
4. Sign Vertices (BearDog)
   ↓
5. Commit Session (LoamSpine)
   ↓
6. Verify Provenance
```

---

### 2. Multi-Primal Workflow (`demo-multi-primal.sh`)
**What it does**:
- Discovers all services via Songbird
- Creates complex workflow with compute
- Distributes work via ToadStool
- Signs all operations with BearDog
- Stores all data in NestGate
- Commits permanently to LoamSpine
- Queries provenance via SweetGrass

**Run**:
```bash
./demo-multi-primal.sh
```

**Flow**:
```
1. Discover Services (Songbird)
   ↓
2. Create Session (rhizoCrypt)
   ↓
3. Store Input Data (NestGate)
   ↓
4. Submit Compute Tasks (ToadStool)
   ↓
5. Sign Results (BearDog)
   ↓
6. Store Outputs (NestGate)
   ↓
7. Commit Session (LoamSpine)
   ↓
8. Query Provenance (SweetGrass)
```

---

### 3. AI Workflow (`demo-ai-workflow.sh`)
**What it does**:
- Complete AI lifecycle with full provenance
- Data collection → Training → Inference → Decision
- Human approval gates
- Cryptographic audit trail
- Demonstrates AI sovereignty

**Run**:
```bash
./demo-ai-workflow.sh
```

**Flow**:
```
1. Discover AI Services (Songbird)
   ↓
2. Collect Training Data (NestGate)
   ↓
3. Human Approval Gate (BearDog signature)
   ↓
4. Train Model (Squirrel + ToadStool)
   ↓
5. Model Approval Gate (BearDog signature)
   ↓
6. Store Model (NestGate)
   ↓
7. Run Inference (Squirrel)
   ↓
8. Sign Decision (BearDog)
   ↓
9. Commit Provenance (LoamSpine)
   ↓
10. Query AI Lineage (SweetGrass)
```

---

## 🎯 Key Achievements

These workflows demonstrate:

### ✅ Zero Hardcoding
- All services discovered via Songbird
- No hardcoded addresses or ports
- Infant discovery working perfectly

### ✅ Capability-Based Architecture
- Clients request capabilities, not specific primals
- "I need signing" not "I need BearDog"
- Vendor-agnostic design

### ✅ Complete Provenance
- Every operation cryptographically signed
- Full lineage from input to output
- Immutable audit trail

### ✅ Human Sovereignty
- Approval gates for critical operations
- Explainable AI decisions
- Right to challenge and audit

### ✅ Scalability
- Distributed compute via ToadStool
- Content-addressed storage (deduplication)
- Parallel execution

### ✅ Fault Tolerance
- Graceful degradation
- Retry logic
- Service discovery fallbacks

---

## 🔄 Workflow Patterns

### Pattern 1: Data Pipeline
```
Input Data → Transform → Store → Sign → Commit
```

**Use Cases**:
- ETL pipelines
- Data processing
- Batch jobs

---

### Pattern 2: Compute Workflow
```
Input → Distribute (ToadStool) → Compute → Aggregate → Sign → Commit
```

**Use Cases**:
- Distributed computation
- Map-reduce jobs
- Parallel processing

---

### Pattern 3: AI Lifecycle
```
Data → Approve → Train → Approve → Infer → Sign → Commit → Query
```

**Use Cases**:
- Machine learning
- Model training
- Inference serving

---

## 📊 Provenance Visualization

Each workflow creates a complete provenance DAG:

```
Session Root
    │
    ├─ Vertex 1: Data Collection
    │   ├─ Payload: sha256:a1b2...
    │   ├─ Agent: did:key:collector
    │   └─ Signature: ...
    │
    ├─ Vertex 2: Processing
    │   ├─ Payload: sha256:c3d4...
    │   ├─ Agent: did:key:processor
    │   ├─ Parent: Vertex 1
    │   └─ Signature: ...
    │
    └─ Vertex 3: Output
        ├─ Payload: sha256:e5f6...
        ├─ Agent: did:key:finalizer
        ├─ Parent: Vertex 2
        └─ Signature: ...
```

Every vertex is:
- **Immutable**: Cannot be changed after creation
- **Signed**: Cryptographically attributed to an agent
- **Linked**: References parent vertices (causality)
- **Stored**: Payloads in content-addressed storage
- **Committed**: Permanently recorded in LoamSpine

---

## 🚀 Prerequisites

### All Primals Running
```bash
# Check all services
./check-all-services.sh

# Expected output:
✓ Songbird:  http://localhost:8888
✓ BearDog:   http://localhost:9400
✓ NestGate:  http://localhost:9500
✓ LoamSpine: http://localhost:9600
✓ ToadStool: http://localhost:9700
✓ Squirrel:  http://localhost:9800
```

### Start All Services
```bash
# Use the startup script
./start-all-primals.sh

# Or start individually:
../../../bins/songbird --port 8888 &
../../../bins/beardog --port 9400 &
../../../bins/nestgate --port 9500 &
../../../bins/loamspine --port 9600 &
../../../bins/toadstool --port 9700 &
../../../bins/squirrel --port 9800 &
```

---

## 🎓 Learning Progression

1. **Start with `demo-simple-workflow.sh`** - Basic integration
2. **Then `demo-multi-primal.sh`** - Complex workflows
3. **Finally `demo-ai-workflow.sh`** - AI provenance

---

## 📝 Notes

- **Discovery-first**: Always discover services via Songbird
- **Sign everything**: Use BearDog for all operations
- **Store immutably**: Use NestGate for all data
- **Commit permanently**: Use LoamSpine for finality
- **Query provenance**: Use SweetGrass for lineage

---

## 🔍 Troubleshooting

### Service not available
```bash
# Check which services are running
./check-all-services.sh

# Start missing services
./start-all-primals.sh
```

### Workflow fails
```bash
# Check logs for each primal
tail -f logs/*.log

# Verify connectivity
curl http://localhost:8888/health  # Songbird
curl http://localhost:9400/health  # BearDog
# ... etc
```

---

## 🏆 Success Criteria

A workflow is successful when:
- [x] All services discovered via Songbird
- [x] All operations cryptographically signed
- [x] All data stored in content-addressed storage
- [x] Complete provenance chain created
- [x] Session permanently committed
- [x] Provenance queryable via SweetGrass

---

## 🔗 Related Documentation

- `specs/INTEGRATION_SPECIFICATION.md` - Integration patterns
- `specs/ARCHITECTURE.md` - System architecture
- `specs/DEHYDRATION_PROTOCOL.md` - Session persistence
- `ZERO_HARDCODING_COMPLETE.md` - Discovery architecture

---

*Last Updated: Dec 26, 2025*

**Status**: Documentation complete, ready for implementation

