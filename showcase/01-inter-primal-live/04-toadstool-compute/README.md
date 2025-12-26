# 04: ToadStool Compute Integration

**Integration with ToadStool for GPU/compute provenance**

## Overview

ToadStool is the compute primal, providing GPU resources, ML frameworks, and distributed compute orchestration. `rhizoCrypt` integrates with ToadStool to track provenance of compute-intensive workloads, attributing work to specific hardware and agents.

## Architecture

```
┌─────────────────┐         ┌─────────────────┐
│   rhizoCrypt    │         │    ToadStool    │
│  (Provenance)   │         │    (Compute)    │
├─────────────────┤         ├─────────────────┤
│ • Workflow DAG  │  track  │ • GPU execution │
│ • Agent events  │────────>│ • ML frameworks │
│ • Results       │         │ • Distributed   │
│ • Audit trail   │         │ • Metrics       │
└─────────────────┘         └─────────────────┘
```

## Demos

### 1. DAG-Driven Compute (`demo-dag-compute.sh`)
**Concept:** Complete ML training workflow with provenance.

- Dataset preparation
- Compute request
- ToadStool execution
- Result validation
- Full lineage captured

**Run:**
```bash
./demo-dag-compute.sh
```

### 2. GPU Provenance (`demo-gpu-provenance.sh`)
**Concept:** Hardware-level attribution for multi-GPU training.

- Each GPU gets a DID
- Performance metrics tracked
- Cost and carbon accounting
- Multi-GPU coordination

**Run:**
```bash
./demo-gpu-provenance.sh
```

### 3. Distributed Compute (`demo-distributed-compute.sh`)
**Concept:** Geo-distributed compute orchestration.

- Multi-region deployment
- Per-node attribution
- Global provenance in single DAG
- SLA and compliance tracking

**Run:**
```bash
./demo-distributed-compute.sh
```

## Key Patterns

### Compute Request Pattern
```rust
// Create compute request vertex
let request = VertexBuilder::new(EventType::DataUpdate { schema: None })
    .with_agent(Did::new("did:key:researcher"))
    .with_parent(dataset_vertex_id)
    .with_metadata("compute_type", "gpu")
    .with_metadata("framework", "pytorch")
    .with_metadata("gpus", "8")
    .build();
```

### Hardware Attribution
```rust
// Each GPU creates a vertex
let gpu_vertex = VertexBuilder::new(EventType::DataUpdate { schema: None })
    .with_agent(Did::new("did:toadstool:gpu-7"))
    .with_parent(request_id)
    .with_metadata("gpu_model", "NVIDIA A100")
    .with_metadata("utilization", "98%")
    .with_metadata("power_watts", "350")
    .build();
```

### Result Tracking
```rust
// Capture compute results
let result = VertexBuilder::new(EventType::DataUpdate { schema: None })
    .with_agent(Did::new("did:toadstool:coordinator"))
    .with_parent(compute_vertex_id)
    .with_payload(model_weights)
    .with_metadata("accuracy", "0.95")
    .with_metadata("duration_sec", "127")
    .with_metadata("gpu_hours", "0.035")
    .build();
```

## Benefits

| Aspect | Benefit |
|--------|---------|
| **Reproducibility** | Full training lineage captured |
| **Accountability** | Per-GPU attribution |
| **Cost Tracking** | GPU hours, energy, carbon |
| **Debugging** | Identify bottlenecks |
| **Compliance** | Audit trail for ML workflows |
| **Fairness** | Transparent resource allocation |

## Real-World Use Cases

1. **ML Training Provenance**
   - Dataset → Model lineage
   - Reproducible experiments
   - Model cards with full history

2. **Distributed Inference**
   - Multi-region deployments
   - Per-node performance
   - SLA verification

3. **Cost Accounting**
   - GPU hours per user/project
   - Energy consumption
   - Fair billing

4. **Environmental Impact**
   - Carbon footprint tracking
   - Green compute metrics
   - Sustainability reporting

5. **Hardware Debugging**
   - GPU performance comparison
   - Identify failing hardware
   - Optimize resource allocation

## Technical Details

### Agent DIDs for Hardware
Every compute resource gets a DID:
- `did:toadstool:gpu-7` - Specific GPU
- `did:toadstool:node-42` - Compute node
- `did:toadstool:us-west:node-5` - Geo-located node

### Metrics Tracking
```rust
// Performance metrics
.with_metadata("utilization", "98%")
.with_metadata("memory_gb", "80")
.with_metadata("power_watts", "350")
.with_metadata("temperature_c", "75")

// Cost metrics
.with_metadata("gpu_hours", "0.035")
.with_metadata("cost_usd", "1.23")
.with_metadata("kwh", "11.76")
.with_metadata("carbon_kg", "5.88")
```

### Capability-Based Discovery
```rust
// No hardcoded ToadStool endpoints
let compute_client = CapabilityClient::for_compute()
    .discover() // Find ToadStool via Songbird
    .with_capability("gpu")
    .with_framework("pytorch")
    .await?;

// Submit job
compute_client.submit_job(job_spec).await?;
```

## Provenance Hierarchy

```
Session (Training Experiment)
├─ Dataset Preparation (Scientist)
├─ Compute Request (ML Pipeline)
├─ GPU Execution
│  ├─ GPU 0 (did:toadstool:gpu-0)
│  ├─ GPU 1 (did:toadstool:gpu-1)
│  ├─ ...
│  └─ GPU 7 (did:toadstool:gpu-7)
├─ Result Aggregation (Coordinator)
└─ Validation (Scientist)

Merkle Root: Single cryptographic proof of entire workflow
```

## Next Steps

Explore complete end-to-end workflows:
```bash
cd ../05-complete-workflows
./demo-ml-pipeline.sh
```

---

**No mocks. Real ToadStool binary. Hardware-level provenance.**
