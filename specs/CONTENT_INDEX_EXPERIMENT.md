# Content Index Experiment — Locality-Sensitive Cross-Session Discovery

**Status**: Proposed Experiment  
**Version**: 0.1.0-draft  
**Last Updated**: March 16, 2026  
**Origin**: Conceptual discussion on hash collision sub-indexing, linear↔branch coexistence

---

## Motivation

rhizoCrypt's DAG stores are session-scoped: every query requires a `session_id`.
This means cross-session patterns — vertices with similar content, similar agent
behavior, or similar event structures — are invisible without full-scan analysis.

Hash tables resolve collisions; this experiment asks: **what if collisions are
the signal?** A locality-sensitive secondary index would intentionally collapse
similar vertices to the same bucket, making cross-session similarity discoverable
at O(1) rather than O(n·m).

### Biological Parallel

In fungal networks, linear hyphae branch into mycelium, and branches anastomose
(reconnect) when they encounter compatible tissue. The recognition mechanism is
chemical similarity — a biological "hash collision." rhizoCrypt's DAG is the
branching mycelium; the content index would be the anastomosis detection layer.

### Historical Parallel

Before paper was abundant, letters were cross-written: text was written
horizontally, then the page was rotated 90° and written over again. Two layers
of information coexist on the same substrate, readable through different access
patterns (orientation). The DAG is one orientation; the content index is the
cross-written layer.

---

## Design

### Non-Goals

- **Not replacing Blake3 VertexIds.** Cryptographic identity stays exact.
- **Not a full-text search engine.** This is structural similarity, not content search.
- **Not cross-primal.** This is a rhizoCrypt-local structure. Other primals
  discover it through `Capability::ContentSimilarity` if/when it matures.

### Core Concept: Reduced-Resolution Hashing

Each vertex has a full Blake3 identity hash (32 bytes, collision-free). The
content index adds a second hash — a **locality-sensitive hash (LSH)** computed
from the vertex's structural properties rather than its full content:

```
LSH input = (event_type, sorted_metadata_keys, agent_prefix, parent_count)
```

This deliberately loses information to create useful "collisions" — vertices
with the same event type, similar metadata structure, and similar topology will
hash to the same bucket.

### Storage Structure

New redb table alongside existing tables:

```
| Table          | Key                | Value                          |
|----------------|--------------------|---------------------------------|
| content_index  | lsh_hash (16 bytes)| packed [(session_id, vertex_id)] |
```

16-byte LSH (128-bit) chosen to balance bucket granularity against storage.
Expected collision rate: ~10-100 vertices per bucket for active systems.

### Query Interface

```rust
/// Find vertices across all sessions with similar structure.
async fn find_similar(&self, vertex: &Vertex) -> Result<Vec<(SessionId, VertexId)>>;

/// Find vertices matching a structural pattern.
async fn find_by_pattern(&self, pattern: ContentPattern) -> Result<Vec<(SessionId, VertexId)>>;
```

### Linear ↔ Branch Lifecycle

The content index bridges the linear and branching views:

```
Session A (branch)  ──vertex──┐
                              ├──► content_index bucket ──► "these are similar"
Session B (branch)  ──vertex──┘
                                        │
                                        ▼
                              dehydration summary
                              (linear, LoamSpine)
```

This enables:
- **Cross-session pattern detection** without scanning
- **Dehydration optimization** — similar sessions may share compression
- **Agent behavior analysis** — same agent, similar events, different sessions
- **Anomaly detection** — vertices that don't match any existing bucket

---

## Feature Gate

```toml
[features]
content-index = ["dep:simhash"]  # or custom LSH implementation
```

The entire experiment is behind a feature gate. No impact on default builds.

---

## Implementation Phases

### Phase 1: LSH Function Design (rhizoCrypt-local)

- Define the locality-sensitive hash function
- Decide on input fields and resolution
- Implement `ContentPattern` type
- Add `content_index` table to redb backend
- Unit tests with synthetic vertex sets

### Phase 2: Query Integration

- Add `find_similar` and `find_by_pattern` to `DagStore` trait (feature-gated)
- JSON-RPC methods: `dag.content.find_similar`, `dag.content.find_by_pattern`
- Performance benchmarks

### Phase 3: Cross-Primal Discovery

- Register `Capability::ContentSimilarity` with Songbird
- Define wire format for similarity queries
- Springs can query rhizoCrypt for structural patterns

---

## Open Questions

1. **LSH algorithm choice**: SimHash, MinHash, or custom? SimHash is fast for
   structural features; MinHash better for set similarity.
2. **Index maintenance**: Populate on `put_vertex` (synchronous) or background
   task (async)?
3. **Bucket eviction**: Should old sessions' entries age out, or persist as
   long as the session data exists?
4. **Privacy**: Does structural similarity leak information about session
   content? Need to analyze what the LSH reveals.

---

## Relation to LoamSpine

LoamSpine is exploring the linear side of this same question — hash collision
resolution within its spine entries. The two experiments are complementary:

- **rhizoCrypt**: Branch-to-branch similarity (cross-session within the DAG)
- **LoamSpine**: Linear collision layering (within the permanent spine)

Both inform the broader question: can intentional hash collisions become a
data science technique for the ecoPrimals ecosystem?

---

## Spring Experiment Guidance

See `wateringHole/CONTENT_SIMILARITY_EXPERIMENT_GUIDE.md` for guidance on how
Springs can participate in this experiment by:
- Producing vertices with known structural patterns
- Querying the content index for cross-session discovery
- Contributing domain-specific LSH input features

---

*Hash collisions aren't bugs — they're anastomosis points in the data mycelium.*
