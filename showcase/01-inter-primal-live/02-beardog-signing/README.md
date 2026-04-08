# 🐻 rhizoCrypt + BearDog Integration

**Purpose**: Add real cryptographic signatures to rhizoCrypt vertices using BearDog HSM  
**Status**: Phase 2 - Identity & Signing  
**Binary**: Signing provider via `showcase-env.sh` (`$PRIMAL_BINS`)

---

## 🎯 Goal

Integrate rhizoCrypt with BearDog to add:
1. **DID Verification** - Verify agent identities
2. **Vertex Signing** - Sign DAG vertices with HSM keys
3. **Multi-Agent Sessions** - Multiple DIDs in one session
4. **Signature Verification** - Verify vertex authenticity

---

## 🔐 BearDog Capabilities

BearDog provides:
- **Universal HSM Integration** (SoftHSM2, TPM, FIDO2, etc.)
- **Genetic Cryptography** (adaptive algorithms)
- **Human Entropy Collection** (sovereign key generation)
- **Ed25519 Signatures** (for vertex signing)
- **Cross-Primal Messaging** (secure coordination)

---

## 📁 Demos

### 1. HSM Discovery (`demo-hsm-discover.sh`)
**What it does**:
- Discovers available HSMs on the system
- Tests HSM capabilities
- Prepares for key generation

**Run**:
```bash
./demo-hsm-discover.sh
```

---

### 2. Key Generation (`demo-generate-keys.sh`)
**What it does**:
- Collects human entropy
- Generates Ed25519 signing key
- Stores key in HSM
- Creates DID from public key

**Run**:
```bash
./demo-generate-keys.sh
```

---

### 3. Sign Vertex (`demo-sign-vertex.sh`)
**What it does**:
- Creates rhizoCrypt session
- Adds vertices to DAG
- Signs vertices with BearDog HSM key
- Verifies signatures
- Demonstrates cryptographic provenance

**Run**:
```bash
./demo-sign-vertex.sh
```

---

### 4. Multi-Agent Session (`demo-multi-agent.sh`)
**What it does**:
- Creates session with multiple agents
- Each agent has own DID and signing key
- Vertices signed by different agents
- Demonstrates multi-party collaboration

**Run**:
```bash
./demo-multi-agent.sh
```

---

## 🔄 Integration Pattern

```
┌─────────────┐
│ rhizoCrypt  │
│   Session   │
└──────┬──────┘
       │
       │ 1. Create vertex
       ▼
┌─────────────┐
│   Vertex    │
│  (unsigned) │
└──────┬──────┘
       │
       │ 2. Sign with BearDog
       ▼
┌─────────────┐
│  BearDog    │
│     HSM     │
└──────┬──────┘
       │
       │ 3. Return signature
       ▼
┌─────────────┐
│   Vertex    │
│   (signed)  │
└──────┬──────┘
       │
       │ 4. Add to DAG
       ▼
┌─────────────┐
│ rhizoCrypt  │
│     DAG     │
└─────────────┘
```

---

## 🎓 Learning Goals

### What We'll Learn
1. **HSM Integration** - How to use hardware security modules
2. **DID Management** - Decentralized identity in practice
3. **Signature Formats** - Ed25519 signature structure
4. **Key Management** - Sovereign key generation and storage
5. **Multi-Agent Coordination** - Multiple DIDs in one session

### What We'll Discover
1. **API Gaps** - Where rhizoCrypt and BearDog don't align
2. **Performance** - Signing overhead on DAG operations
3. **Format Mismatches** - DID and signature format differences
4. **Configuration** - What's hard to set up
5. **Error Handling** - Where errors aren't clear

---

## 📊 Success Criteria

### Phase 2 Complete When:
- [x] BearDog binary operational
- [x] HSM discovered and tested
- [x] Keys generated in HSM
- [x] Vertices signed successfully
- [x] Signatures verify correctly
- [x] Multi-agent sessions work
- [x] All gaps documented

---

## 🔗 Related

- [BearDog User Guide](https://github.com/eastgate-software/beardog)
- Gap tracking: `infra/wateringHole/` handoffs

---

*"Sovereignty through cryptography, trust through verification."* 🐻🔐

