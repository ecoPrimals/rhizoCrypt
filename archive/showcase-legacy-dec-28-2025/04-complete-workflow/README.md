# 🔐 rhizoCrypt Complete Workflow Demos

**Phase 4: End-to-End Session → Dehydration → Commit**

---

## 📋 What You'll Learn

- Full session lifecycle with multiple agents
- Attestation collection
- Merkle root computation
- Dehydration to summary
- Permanent commit to LoamSpine
- Provenance queries via SweetGrass

---

## 🎯 The Big Picture

rhizoCrypt is the **ephemeral** layer that knows when to **forget**. This workflow shows:

1. **Capture**: Session events become vertices in the DAG
2. **Prove**: Merkle tree proves integrity
3. **Attest**: BearDog signs the summary
4. **Commit**: LoamSpine stores permanently
5. **Query**: SweetGrass tracks provenance

---

## 🚀 Quick Start

```bash
# Simple session capture and commit
./demo-simple-dehydration.sh

# Multi-agent session with attestations
./demo-multi-agent.sh

# Full gaming + ML scenario
./demo-gaming-ml-session.sh
```

---

## 📁 Available Demos

### `dehydration/demo-simple-dehydration.sh`
**Time:** 10 minutes  
**Complexity:** Intermediate

Simple capture → commit workflow:
1. Create a session
2. Add 10 vertices
3. Compute Merkle root
4. Generate summary
5. Commit to LoamSpine

### `dehydration/demo-multi-agent.sh`
**Time:** 15 minutes  
**Complexity:** Advanced

Multi-agent session:
1. Create collaboration session
2. 3 agents contribute vertices
3. Each agent signs their contributions
4. Collect attestations
5. Commit with full provenance

### `dehydration/demo-gaming-ml-session.sh`
**Time:** 30 minutes  
**Complexity:** Advanced

Complete gaming + ML scenario:
1. Gaming session captures player actions
2. ToadStool runs ML training
3. Model checkpoints stored in NestGate
4. Session dehydrates with compute receipts
5. LoamSpine commits full history
6. SweetGrass enables provenance queries

### `provenance/demo-provenance-query.sh`
**Time:** 15 minutes  
**Complexity:** Advanced

Query provenance chains:
1. Load committed session
2. Query by vertex ID
3. Query by agent DID
4. Track attribution across sessions

---

## 🏗️ Dehydration Flow

```
┌─────────────────────────────────────────────────────────────────┐
│                     Active Session                               │
│  ┌─────┐   ┌─────┐   ┌─────┐   ┌─────┐   ┌─────┐               │
│  │ v1  │──▶│ v2  │──▶│ v3  │──▶│ v4  │──▶│ v5  │               │
│  └─────┘   └─────┘   └─────┘   └─────┘   └─────┘               │
│      \                  ▲                  /                     │
│       \                 │                 /                      │
│        └────────────────┴────────────────┘                      │
│                         │                                        │
│                    Merkle Root                                   │
└────────────────────────┬────────────────────────────────────────┘
                         │
                         ▼
              ┌─────────────────────┐
              │  Dehydration Phase  │
              │  • Compute root     │
              │  • Generate summary │
              │  • Collect attests  │
              └──────────┬──────────┘
                         │
                         ▼
┌─────────────────────────────────────────────────────────────────┐
│                   Dehydrated Summary                             │
│  ┌─────────────────────────────────────────────────────────┐    │
│  │  session_id: abc123                                      │    │
│  │  merkle_root: 0xdead...beef                              │    │
│  │  vertex_count: 5                                         │    │
│  │  attestations: [did:beardog:agent1, did:beardog:agent2]  │    │
│  │  payloads: [nestgate://payload1, nestgate://payload2]    │    │
│  └─────────────────────────────────────────────────────────┘    │
└────────────────────────┬────────────────────────────────────────┘
                         │
                         ▼
              ┌─────────────────────┐
              │     LoamSpine       │
              │  Permanent Storage  │
              └─────────────────────┘
```

---

## 📊 Dehydration Stages

| Stage | Description | Components |
|-------|-------------|------------|
| **Active** | Session is live, accepting vertices | rhizoCrypt |
| **Resolving** | Session closing, preparing summary | rhizoCrypt |
| **Attesting** | Collecting agent signatures | BearDog |
| **Committing** | Storing summary permanently | LoamSpine |
| **Committed** | Session is permanent, DAG discarded | LoamSpine |

---

## 🔐 Attestation Flow

```
Agent 1 (DID: did:beardog:alice)
    │
    ▼
┌─────────────────────┐
│ Sign vertices       │
│ signed_v1, signed_v2│
└──────────┬──────────┘
           │
           ▼
┌─────────────────────┐
│ Attestation:        │
│ "I contributed v1,  │
│  v2 to session X"   │
└─────────────────────┘

Agent 2 (DID: did:beardog:bob)
    │
    ▼
┌─────────────────────┐
│ Sign vertices       │
│ signed_v3           │
└──────────┬──────────┘
           │
           ▼
┌─────────────────────┐
│ Attestation:        │
│ "I contributed v3   │
│  to session X"      │
└─────────────────────┘

            ▼
┌─────────────────────────────────┐
│ Dehydrated Summary              │
│ attestations: [alice, bob]      │
│ merkle_root: 0xabc...           │
└─────────────────────────────────┘
```

---

## 🎮 Gaming + ML Scenario

The flagship demo shows rhizoCrypt coordinating:

1. **Gaming Session**: Chess match between players
2. **ML Training**: ToadStool trains on game data
3. **Storage**: NestGate stores model checkpoints
4. **Provenance**: Track which games trained which models

```
Player Actions    ML Training       Model Storage
     │                │                  │
     ▼                ▼                  ▼
┌─────────┐    ┌─────────────┐    ┌───────────┐
│ Chess   │───▶│  ToadStool  │───▶│ NestGate  │
│ Moves   │    │  GPU Train  │    │ Ckpt.bin  │
└─────────┘    └─────────────┘    └───────────┘
     │                │                  │
     └────────────────┼──────────────────┘
                      ▼
              ┌───────────────┐
              │  rhizoCrypt   │
              │  Session DAG  │
              └───────────────┘
                      │
                      ▼
              ┌───────────────┐
              │  LoamSpine    │
              │  Commit       │
              └───────────────┘
                      │
                      ▼
              ┌───────────────┐
              │  SweetGrass   │
              │  Provenance   │
              └───────────────┘
```

---

## 💡 Key Concepts

### Dehydration
Converting live session data to a compact, permanent summary.

### Attestations
Cryptographic signatures from agents proving their contributions.

### Merkle Root
Single hash proving the integrity of all vertices.

### Provenance
The history of who contributed what, when, and why.

---

## 🔗 Related Documentation

- **Dehydration Protocol**: `../../specs/DEHYDRATION_PROTOCOL.md`
- **Integration Spec**: `../../specs/INTEGRATION_SPECIFICATION.md`
- **Data Model**: `../../specs/DATA_MODEL.md`

---

*rhizoCrypt: Capture everything, prove it, commit what matters, forget the rest.*

