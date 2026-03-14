# 🔐 rhizoCrypt — Live Phase 1 Integration

**Purpose**: Demonstrate rhizoCrypt integration with **REAL Phase 1 binaries** (NO MOCKS)  
**Philosophy**: "Interactions show us gaps in our evolution"  
**Bins Location**: `../../bins/`

---

## 🎯 Goal

Integrate rhizoCrypt with real Phase 1 primals to:
1. **Discover capabilities** via Songbird
2. **Sign vertices** via BearDog  
3. **Store payloads** via NestGate
4. **Track compute** via ToadStool
5. **Route AI** via Squirrel
6. **Document every gap** discovered

---

## 📁 Structure

```
01-inter-primal-live/
├── 01-songbird-discovery/    # Foundation: Capability discovery
│   ├── start-songbird.sh      # Start ../../../bins/songbird-rendezvous
│   ├── demo-register.sh       # Register rhizoCrypt capabilities
│   ├── demo-discover.sh       # Find other primals
│   └── demo-health.sh         # Health monitoring
│
├── 02-beardog-signing/        # Identity: DID & Signatures
│   ├── start-beardog.sh       # Start ../../../bins/beardog
│   ├── demo-did-verify.sh     # Verify DID
│   ├── demo-sign-vertex.sh    # Sign vertex with real HSM
│   └── demo-multi-agent.sh    # Multi-DID session
│
├── 03-nestgate-storage/       # Storage: Content-addressed payloads
│   ├── start-nestgate.sh      # Start ../../../bins/nestgate
│   ├── demo-store-payload.sh  # Store DAG payload
│   ├── demo-retrieve.sh       # Retrieve by hash
│   └── demo-zfs-magic.sh      # NestGate ZFS snapshots
│
├── 04-toadstool-compute/      # Compute: GPU/ML event tracking
│   ├── start-toadstool.sh     # Start ../../../bins/toadstool-byob-server
│   ├── demo-gpu-task.sh       # GPU task → DAG events
│   ├── demo-ml-training.sh    # ML session capture
│   └── demo-gaming-events.sh  # Gaming session tracking
│
├── 05-squirrel-ai/            # AI: MCP routing
│   ├── start-squirrel.sh      # Start ../../../bins/squirrel
│   ├── demo-mcp-session.sh    # MCP session → DAG
│   └── demo-multi-provider.sh # Track provider routing
│
└── 05-complete-workflows/     # Full ecosystem coordination
    ├── demo-supply-chain.sh   # Supply chain provenance
    ├── demo-ml-pipeline.sh    # ML pipeline with provenance
    ├── demo-federated-identity.sh  # Federated identity flow
    └── demo-document-workflow.sh   # Document lifecycle
```

---

## 🚀 Quick Start

### Prerequisites

**Phase 1 Binaries** (already available):
```bash
ls -lh ../../../bins/
# beardog
# nestgate
# nestgate-client
# songbird-cli
# songbird-orchestrator
# songbird-rendezvous  ← We'll use this first
# squirrel
# squirrel-cli
# toadstool-byob-server
# toadstool-cli
```

### Start with Songbird Discovery

```bash
cd 01-songbird-discovery
./start-songbird.sh          # Start rendezvous server
./demo-register.sh           # Register rhizoCrypt
./demo-discover.sh           # Find other primals
```

---

## 🔍 Gap Discovery

As we integrate with each primal, we'll document gaps in:

**`GAPS_DISCOVERED.md`** (created as we go):
- Songbird: JWT handling, registration stability
- BearDog: HSM integration, signature formats
- NestGate: JWT config, payload limits, compression
- ToadStool: Event stream performance, GPU metadata
- Squirrel: MCP event format, provider metadata
- LoamSpine (future): Dehydration format, slice checkout

**Each gap documented as**:
```markdown
## Gap: [Name]
**Primal**: [Which one]
**Severity**: Critical/High/Medium/Low
**Expected**: [What we thought]
**Actual**: [What happened]
**Fix**: [What needs to change]
**Status**: Open/Fixed
```

---

## ⚡ Progressive Build

### Phase 1: Foundation (Songbird)
**Time**: 2-3 hours  
**Goal**: rhizoCrypt discovers and registers with mesh

**Success Criteria**:
- [x] Songbird rendezvous runs
- [x] rhizoCrypt registers capabilities
- [x] rhizoCrypt discovers other primals
- [x] Health checks work

**Gaps Expected**:
- JWT token handling
- Registration persistence
- Health report format

---

### Phase 2: Identity (BearDog)
**Time**: 2-3 hours  
**Goal**: Real DID verification and vertex signing

**Success Criteria**:
- [x] BearDog HSM operational
- [x] DIDs verified
- [x] Vertices signed with real signatures
- [x] Multi-agent sessions work

**Gaps Expected**:
- HSM configuration
- Signature format compatibility
- DID resolution speed

---

### Phase 3: Storage (NestGate)
**Time**: 2-3 hours  
**Goal**: Content-addressed payload storage

**Success Criteria**:
- [x] NestGate ZFS operational
- [x] Payloads stored and retrieved
- [x] Content-addressing works
- [x] Compression coordination

**Gaps Expected**:
- JWT configuration (known from STATUS.md)
- Payload size limits
- Compression format coordination

---

### Phase 4: Compute (ToadStool)
**Time**: 2-3 hours  
**Goal**: GPU/ML compute event tracking

**Success Criteria**:
- [x] ToadStool BYOB server running
- [x] Compute events captured in DAG
- [x] GPU metadata tracked
- [x] Biome lifecycle recorded

**Gaps Expected**:
- Event stream performance
- GPU metadata format
- High-frequency event handling

---

### Phase 5: AI (Squirrel)
**Time**: 1-2 hours  
**Goal**: MCP session routing

**Success Criteria**:
- [x] Squirrel MCP server running
- [x] AI routing decisions captured
- [x] Multi-provider sessions tracked
- [x] Privacy metadata preserved

**Gaps Expected**:
- MCP event format
- Provider metadata structure

---

### Phase 6: Complete Workflow
**Time**: 2-3 hours  
**Goal**: All primals coordinated

**Success Criteria**:
- [x] All Phase 1 primals running together
- [x] rhizoCrypt coordinates full workflow
- [x] Gaming + ML + Storage scenario works
- [x] Performance acceptable

**Gaps Expected**:
- Multi-primal coordination overhead
- Network performance
- Error propagation

---

## 🎓 Learning Goals

### What We'll Learn

1. **Real API Gaps**: Where our APIs don't match Phase 1
2. **Performance Bottlenecks**: Where coordination slows down
3. **Format Mismatches**: Data formats that don't align
4. **Configuration Complexity**: What's hard to set up
5. **Error Handling**: Where errors aren't clear

### What We'll Improve

1. **rhizoCrypt APIs**: Update based on real usage
2. **Integration Clients**: Fix bugs discovered
3. **Documentation**: Add real examples
4. **Configuration**: Simplify based on pain points
5. **Error Messages**: Make them actionable

---

## 🏆 Success Metrics

### Phase 1 Success When:
- [x] Songbird rendezvous operational
- [x] rhizoCrypt registers successfully
- [x] Discovery works bidirectionally
- [x] All gaps documented

### Phase 2 Success When:
- [x] BearDog signs vertices
- [x] Signatures verify correctly
- [x] Multi-agent sessions work
- [x] Performance acceptable

### Phase 3 Success When:
- [x] NestGate stores payloads
- [x] Content-addressing works
- [x] ZFS snapshots coordinate
- [x] Compression works

### Phase 4 Success When:
- [x] ToadStool events captured
- [x] GPU metadata tracked
- [x] High-frequency events handled
- [x] Biome lifecycle recorded

### Phase 5 Success When:
- [x] Squirrel sessions tracked
- [x] MCP routing captured
- [x] Privacy preserved

### Phase 6 Success When:
- [x] All primals coordinate
- [x] Real-world scenario works
- [x] Performance good enough
- [x] All gaps documented

---

## 📝 Gap Documentation Template

For each gap, create entry in `GAPS_DISCOVERED.md`:

```markdown
## Gap #N: [Short Description]

**Discovered**: 2025-12-24  
**Primal**: Songbird  
**Severity**: High  
**Demo**: demo-register.sh

### Expected Behavior
rhizoCrypt should register with Songbird mesh and receive confirmation.

### Actual Behavior
Registration succeeds but health checks fail with JWT validation error.

### Root Cause
rhizoCrypt's health endpoint doesn't include JWT token in response.
NestGate's STATUS.md shows this is a known issue.

### Impact
- Health monitoring doesn't work
- Songbird can't verify rhizoCrypt is alive
- Could lead to routing failures

### Fix Required
**In rhizoCrypt**:
- Add JWT token support to health endpoint
- Read JWT config from environment

**In Showcase**:
- Document JWT configuration in README
- Add environment variable example

### Status
- [ ] Fix implemented
- [ ] Tests added
- [ ] Documentation updated
- [ ] Deployed

### References
- NestGate STATUS.md mentions JWT pending
- Songbird health check spec: ...
```

---

## Related Documents

- [rhizoCrypt README](../../README.md) — Project overview
- [CHANGELOG](../../CHANGELOG.md) — Version history
- [specs/](../../specs/) — Formal specifications

---

## 🎯 Next Steps

1. **Start Here**: `cd 01-songbird-discovery`
2. **Run**: `./start-songbird.sh`
3. **Test**: `./demo-register.sh`
4. **Document**: Any gaps in `GAPS_DISCOVERED.md`
5. **Iterate**: Fix, test, document, repeat

---

*"Real bins, not mocks. Real gaps, not assumptions. Real learning, not theory."* 🔐

