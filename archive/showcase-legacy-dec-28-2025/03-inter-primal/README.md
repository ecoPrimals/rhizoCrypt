# 🔐 rhizoCrypt Inter-Primal Demos

**Phase 3: Inter-Primal Integration**

---

## 📋 What You'll Learn

- Capability-based discovery via Songbird
- DID verification and vertex signing via BearDog
- Payload storage via NestGate
- Permanent commits via LoamSpine
- Compute event integration via ToadStool
- Provenance queries via SweetGrass

---

## 🎯 Philosophy

rhizoCrypt doesn't hardcode primal addresses. Instead:

1. **Self-Knowledge Only**: rhizoCrypt knows what it does, not who else exists
2. **Runtime Discovery**: Primals are discovered via Songbird capabilities
3. **Graceful Degradation**: Missing primals don't crash the system

---

## 🚀 Quick Start

```bash
# Requires Songbird running (for discovery)
cd songbird-discovery && ./demo-discovery.sh

# With BearDog for signing
cd beardog-signing && ./demo-signing.sh

# With NestGate for payloads
cd nestgate-payloads && ./demo-payloads.sh

# Full ecosystem
./demo-full-ecosystem.sh
```

---

## 📁 Available Demos

### `songbird-discovery/`
**Status:** ✅ Scaffolded

Demonstrates capability-based discovery:
- Register rhizoCrypt capabilities
- Discover sibling primals
- Query by capability type
- Handle discovery failures gracefully

### `beardog-signing/`
**Status:** ✅ Scaffolded

Demonstrates DID and cryptographic operations:
- Verify vertex author DIDs
- Sign dehydration summaries
- Request attestations
- Generate BTSP headers

### `nestgate-payloads/`
**Status:** ✅ Scaffolded

Demonstrates content storage:
- Store session payloads
- Content-addressed retrieval
- Streaming large payloads
- Payload deduplication

### `loamspine-commits/`
**Status:** ✅ Scaffolded

Demonstrates permanent storage:
- Commit dehydrated sessions
- Query committed data
- Verify commit integrity
- Provenance tracking

### `toadstool-events/`
**Status:** ✅ Scaffolded (new!)

Demonstrates compute event integration:
- Capture ToadStool compute tasks
- Track task lifecycle
- Record compute receipts
- Link to session DAGs

### `sweetgrass-queries/`
**Status:** ✅ Scaffolded (new!)

Demonstrates provenance queries:
- Query session attribution
- Track agent contributions
- Verify provenance chains
- Cross-session lineage

---

## 🏗️ Integration Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                        rhizoCrypt                            │
│                    (Ephemeral DAG Engine)                    │
├─────────────────────────────────────────────────────────────┤
│                                                              │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐       │
│  │   Songbird   │  │   BearDog    │  │   NestGate   │       │
│  │  Discovery   │  │   Signing    │  │   Payloads   │       │
│  └──────┬───────┘  └──────┬───────┘  └──────┬───────┘       │
│         │                 │                 │                │
│         ▼                 ▼                 ▼                │
│  ┌─────────────────────────────────────────────────────┐    │
│  │              DiscoveryRegistry                       │    │
│  │      (Capability → ServiceEndpoint mapping)          │    │
│  └─────────────────────────────────────────────────────┘    │
│                           │                                  │
│  ┌────────────────────────┼────────────────────────┐        │
│  │                        ▼                        │        │
│  │             Integration Layer                    │        │
│  │  BearDogClient • NestGateClient • LoamSpineClient        │
│  │  SongbirdClient • ToadStoolClient • SweetGrassClient     │
│  └─────────────────────────────────────────────────────┘    │
│                                                              │
└─────────────────────────────────────────────────────────────┘
                              │
                              ▼
                    ┌──────────────────┐
                    │    LoamSpine     │
                    │    (Permanent    │
                    │     Storage)     │
                    └──────────────────┘
```

---

## 🔧 Capability Types

rhizoCrypt discovers primals by capability:

| Capability | Primal | Purpose |
|------------|--------|---------|
| `Orchestration` | Songbird | Service coordination |
| `DidVerification` | BearDog | Identity & signing |
| `PayloadStorage` | NestGate | Content storage |
| `PermanentCommit` | LoamSpine | Long-term storage |
| `Compute` | ToadStool | Task execution |
| `Provenance` | SweetGrass | Attribution queries |
| `EphemeralDag` | rhizoCrypt | Session DAG (self) |

---

## 💡 Graceful Degradation

If a primal is unavailable:

```rust
match discovery.get_endpoint(Capability::DidVerification) {
    Some(endpoint) => {
        // Use BearDog
        beardog_client.sign_vertex(vertex).await?
    }
    None => {
        // Continue without signing
        log::warn!("BearDog unavailable, skipping signature");
        Ok(UnsignedVertex(vertex))
    }
}
```

---

## 🔐 Security

- All inter-primal communication uses BTSP (BearDog)
- No hardcoded secrets
- Capabilities are verified at registration
- Rate limiting protects all endpoints

---

## 🔗 Next Steps

After understanding inter-primal integration:
1. Explore `../04-complete-workflow/` for full dehydration
2. Review `../../specs/INTEGRATION_SPECIFICATION.md`
3. See `../../specs/DEHYDRATION_PROTOCOL.md`

---

*rhizoCrypt: Capability-based discovery, graceful degradation*

