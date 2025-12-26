# 03: NestGate Storage Integration

**Integration with NestGate for efficient payload storage**

## Overview

NestGate is the storage primal, providing ZFS-based, content-addressed, persistent storage. `rhizoCrypt` integrates with NestGate to separate provenance (DAG) from large data (payloads).

## Architecture

```
┌─────────────────┐         ┌─────────────────┐
│   rhizoCrypt    │         │    NestGate     │
│   (Ephemeral)   │         │  (Persistent)   │
├─────────────────┤         ├─────────────────┤
│ • Provenance    │  refs   │ • Payloads      │
│ • DAG structure │────────>│ • Content hash  │
│ • Agent events  │         │ • ZFS storage   │
│ • Merkle proofs │         │ • Compression   │
└─────────────────┘         └─────────────────┘
```

## Demos

### 1. Payload Storage (`demo-payload-storage.sh`)
**Concept:** Store large payloads efficiently outside the DAG.

- Vertices store small metadata
- Large payloads stored in NestGate
- Content-addressed via Blake3 hash
- Separation of concerns: provenance vs data

**Run:**
```bash
./demo-payload-storage.sh
```

### 2. Content-Addressed Storage (`demo-content-addressed.sh`)
**Concept:** Automatic deduplication via content addressing.

- Same content = same hash
- Multiple users, one copy
- Massive storage savings
- Integrity verification built-in

**Run:**
```bash
./demo-content-addressed.sh
```

### 3. Workflow Integration (`demo-workflow-integration.sh`)
**Concept:** Complete document management workflow.

- DAG tracks all changes
- Storage holds all versions
- Multi-agent collaboration
- Full audit trail + efficient storage

**Run:**
```bash
./demo-workflow-integration.sh
```

## Key Patterns

### Content-Addressed References
```rust
// Hash payload
let payload_hash = blake3::hash(&large_payload);

// Store in NestGate (via StorageClient)
storage_client.store(payload_hash, &large_payload).await?;

// Create vertex with reference
let vertex = VertexBuilder::new(event_type)
    .with_payload_ref(payload_hash)
    .build();
```

### Deduplication
- Same content → same hash → single storage
- Automatic across all users
- No manual coordination needed

### Separation of Concerns
- **rhizoCrypt:** Who, what, when (provenance)
- **NestGate:** Data storage (payloads)
- **Together:** Complete system

## Benefits

| Aspect | Benefit |
|--------|---------|
| **Efficiency** | Small DAG vertices, large payloads separate |
| **Deduplication** | Automatic, content-addressed |
| **Compression** | ZFS features (10-20:1 ratios) |
| **Integrity** | Hash verifies content |
| **Provenance** | DAG links to storage |
| **Scalability** | Unlimited payload sizes |

## Real-World Use Cases

1. **Document Management**
   - DAG: Version history, author attribution
   - Storage: Document files

2. **Media Libraries**
   - DAG: Upload events, permissions
   - Storage: Images, videos

3. **ML Pipelines**
   - DAG: Training provenance
   - Storage: Model files, datasets

4. **Collaborative Workflows**
   - DAG: Edit history, reviews
   - Storage: File versions

## Technical Details

### Content Addressing
- **Hash Function:** Blake3 (fast, secure)
- **Address:** Hash of content
- **Lookup:** O(1) via hash
- **Dedup:** Automatic

### Storage Client
```rust
// Capability-based storage client
let storage_client = CapabilityClient::for_storage()
    .discover() // Find NestGate via Songbird
    .await?;

// Store payload
storage_client.store(hash, data).await?;

// Retrieve payload
let data = storage_client.retrieve(hash).await?;
```

### Zero-Knowledge Discovery
- `rhizoCrypt` has no hardcoded NestGate endpoints
- Discovery via Songbird (capability-based)
- Runtime binding to available storage

## Next Steps

Explore ToadStool compute integration:
```bash
cd ../04-toadstool-compute
./demo-dag-compute.sh
```

---

**No mocks. Real NestGate binary. Capability-based discovery.**
