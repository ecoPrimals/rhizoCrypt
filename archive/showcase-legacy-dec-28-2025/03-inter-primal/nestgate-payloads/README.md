# 🔐 rhizoCrypt NestGate Payload Demos

**Phase 3: Inter-Primal — Payload Storage & Retrieval**

---

## 📋 What You'll Learn

- Storing large payloads in NestGate
- Content-addressed payload references
- Retrieving payloads by reference
- Separating structure (rhizoCrypt) from content (NestGate)

---

## 🚀 Quick Start

```bash
# Run the payload storage demo
./demo-payload-storage.sh
```

---

## 🔗 Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                        rhizoCrypt                                │
│  ┌───────────────────────────────────────────────────────────┐  │
│  │                    DAG (Vertices)                          │  │
│  │  [v1] ─→ [v2] ─→ [v3]                                     │  │
│  │    │       │       │                                       │  │
│  │  ref_1   ref_2   ref_3  ← PayloadRef (content addresses)  │  │
│  └───────────────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────────────┘
                           │
                           │ Store/Retrieve
                           ▼
┌─────────────────────────────────────────────────────────────────┐
│                        NestGate                                  │
│  ┌───────────────────────────────────────────────────────────┐  │
│  │                  Content Store                             │  │
│  │  ref_1 → [Large Image Data]                               │  │
│  │  ref_2 → [Audio File]                                     │  │
│  │  ref_3 → [Model Weights]                                  │  │
│  └───────────────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────────────┘
```

---

## 💡 Why Separate?

| Aspect | rhizoCrypt | NestGate |
|--------|------------|----------|
| Stores | Structure, Events | Large Content |
| Size | Small (metadata) | Large (payloads) |
| Lifetime | Ephemeral | Persistent |
| Addressing | Vertex IDs | Content Hash |

---

## 📜 PayloadRef

```rust
pub struct PayloadRef {
    /// Content-addressed hash of payload
    pub content_hash: ContentHash,
    /// Size in bytes
    pub size: u64,
}
```

Key properties:
- **Immutable**: Content can't change for same hash
- **Verifiable**: Anyone can verify content matches hash
- **Efficient**: Store once, reference many times

---

*rhizoCrypt: Structure in DAG, Content in NestGate*

