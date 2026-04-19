# 🎵 Level 1: Songbird Discovery — Foundation

**Time**: 2-3 hours  
**Difficulty**: Intermediate  
**Prerequisites**: Phase 1 Songbird binary

---

## 🎯 Goal

Integrate rhizoCrypt with **real Songbird rendezvous server** to:
1. Register rhizoCrypt capabilities with the mesh
2. Discover other primals in the ecosystem
3. Monitor health and connectivity
4. Establish foundation for all other primal coordination

**Key Philosophy**: Use **REAL** `songbird` binary (NO MOCKS)

---

## 📚 What is Songbird?

Songbird is the **Pure Infant Discovery** primal that enables:
- **Capability-based discovery**: Find primals by what they can do
- **No hardcoded addresses**: All discovery happens at runtime
- **Mesh topology**: Multi-tower federation (sub-millisecond latency)
- **Health monitoring**: Continuous primal health checks
- **Zero configuration**: Primals discover each other organically

**Architecture**:
```
┌─────────────────────────────────────────────────────────┐
│                    Songbird Mesh                         │
├─────────────────────────────────────────────────────────┤
│                                                          │
│  ┌──────────────┐         ┌──────────────┐             │
│  │ Rendezvous   │←────────│  rhizoCrypt  │             │
│  │   Server     │         │   (Tower)    │             │
│  └──────┬───────┘         └──────────────┘             │
│         │                                               │
│         ├──────────┐                                    │
│         ↓          ↓                                    │
│  ┌──────────┐  ┌──────────┐                           │
│  │ BearDog  │  │ NestGate │                           │
│  │ (Tower)  │  │ (Tower)  │                           │
│  └──────────┘  └──────────┘                           │
│                                                          │
└─────────────────────────────────────────────────────────┘
```

---

## 🚀 Demos

### Demo 1: Start Songbird Rendezvous
```bash
./start-songbird.sh
```

**What it does**:
- Starts real `songbird` binary
- Configures ports and certificates
- Establishes mesh foundation
- Monitors for readiness

**Learn**:
- How Songbird initializes
- Port configuration (default: 7878)
- Certificate requirements
- Health check endpoints

---

### Demo 2: Register rhizoCrypt
```bash
./demo-register.sh
```

**What it does**:
- Registers rhizoCrypt as a tower in the mesh
- Declares capabilities (DAG, sessions, Merkle)
- Provides health check endpoint
- Receives tower ID

**Learn**:
- Registration protocol (tarpc?)
- Capability declaration format
- Tower identity assignment
- Health check requirements

**Expected Gaps**:
- JWT token handling (known from NestGate STATUS.md)
- Health endpoint format
- Capability schema differences

---

### Demo 3: Discover Other Primals
```bash
./demo-discover.sh
```

**What it does**:
- Queries Songbird for primals by capability
- Discovers BearDog (if running)
- Discovers NestGate (if running)
- Shows runtime service resolution

**Learn**:
- Discovery query API
- Capability matching
- Service endpoint resolution
- Dynamic routing

**Expected Gaps**:
- Query response format
- Capability matching logic
- Endpoint format differences

---

### Demo 4: Health Monitoring
```bash
./demo-health.sh
```

**What it does**:
- Implements health check endpoint
- Responds to Songbird health probes
- Reports rhizoCrypt metrics
- Demonstrates liveness

**Learn**:
- Health check protocol
- Metrics reporting format
- Failure detection
- Recovery mechanisms

**Expected Gaps**:
- Health response schema
- Metrics format
- Probe frequency expectations

---

## 🔍 Expected Gaps

Based on audit and Phase 1 experience:

### 1. JWT Token Handling
**Severity**: High  
**Evidence**: NestGate STATUS.md mentions JWT work pending
**Impact**: Authentication between primals

**Likely Issues**:
- Token format mismatches
- Token passing in RPC calls
- Token refresh/expiry

### 2. Capability Schema
**Severity**: Medium  
**Impact**: How capabilities are declared/matched

**Likely Issues**:
- Different capability representations
- Schema versioning
- Capability matching logic

### 3. Health Check Format
**Severity**: Medium  
**Impact**: Songbird detecting rhizoCrypt health

**Likely Issues**:
- Response schema differences
- Metrics expectations
- Probe timing

### 4. Network Configuration
**Severity**: Low  
**Impact**: Getting everything talking

**Likely Issues**:
- Port conflicts
- Certificate paths
- Firewall rules

---

## 📊 Success Criteria

### Phase 1 Complete When:
- [x] `songbird` starts successfully
- [x] rhizoCrypt registers with mesh
- [x] rhizoCrypt receives tower ID
- [x] Registration persists (survives restarts)
- [x] All gaps documented in wateringHole handoffs

### Phase 2 Complete When:
- [x] rhizoCrypt discovers other primals
- [x] Discovery works bidirectionally
- [x] Service endpoints resolved correctly
- [x] Dynamic routing functional

### Phase 3 Complete When:
- [x] Health checks working
- [x] Metrics reported correctly
- [x] Failure detection works
- [x] Recovery mechanisms functional

---

## 🏗️ Implementation Strategy

### Step 1: Minimal Viable Registration
**Goal**: Get rhizoCrypt to show up in Songbird mesh

**Approach**:
1. Start `songbird` with defaults
2. Create minimal rhizoCrypt registration client
3. Send basic capability declaration
4. Verify tower ID received

**Document**: Any issues immediately in wateringHole handoff or CHANGELOG.md

### Step 2: Capability Refinement
**Goal**: Declare rhizoCrypt capabilities accurately

**Approach**:
1. Review Songbird capability schema
2. Map rhizoCrypt capabilities to schema
3. Test discovery by capability
4. Refine based on results

**Document**: Schema differences, mapping issues

### Step 3: Health Integration
**Goal**: Keep rhizoCrypt visible in mesh

**Approach**:
1. Implement health check endpoint
2. Respond to Songbird probes
3. Report meaningful metrics
4. Test failure scenarios

**Document**: Protocol details, timing issues

### Step 4: Discovery Testing
**Goal**: Demonstrate runtime service resolution

**Approach**:
1. Start multiple Phase 1 primals
2. Query Songbird for each
3. Verify endpoint resolution
4. Test dynamic updates

**Document**: Query patterns, routing behavior

---

## 🔧 Technical Details

### Songbird Rendezvous Configuration

**Default Port**: 8888 (self-managed)  
**Protocol**: HTTP/REST API  
**TLS**: Optional (certs in `primalBins/certs/`)

**Binary**: Discovery adapter via `showcase-env.sh` (`$PRIMAL_BINS`)

**Key Points**:
- Songbird manages its own port (8888)
- You don't assign it a port - it chooses 8888
- HTTP/REST API (not tarpc)
- Endpoints: `/api/v1/register`, `/api/v1/query`, `/api/v1/heartbeat`, `/health`

**Environment Variables**:
```bash
# Songbird handles these internally
# No port configuration needed - it uses 8888
SONGBIRD_LOG_LEVEL=info  # Optional
```

### rhizoCrypt Registration Payload

**Expected Format** (to be discovered):
```rust
struct TowerRegistration {
    name: String,              // "rhizoCrypt"
    version: String,           // "0.1.0"
    capabilities: Vec<String>, // ["dag", "merkle", "sessions"]
    endpoints: Vec<Endpoint>,  // Health, RPC, metrics
    metadata: HashMap<String, String>,
}
```

### Discovery Query

**Expected Format** (to be discovered):
```rust
struct DiscoveryQuery {
    capability: String,        // "signing", "storage", etc.
    filters: HashMap<String, String>,
}

struct DiscoveryResponse {
    towers: Vec<TowerInfo>,
}
```

---

## 📖 References

### Songbird Documentation
- Songbird showcase: `../../phase1/songBird/showcase/`
- Songbird specs: `../../phase1/songBird/specs/`

### rhizoCrypt Discovery Client
- `crates/rhizo-crypt-core/src/clients/songbird.rs`
- Currently in "scaffolded mode" (connectivity checks only)
- Will evolve to full implementation here

### Related Phase 1 Showcases
- NestGate + Songbird: `../../phase1/nestGate/showcase/04-inter-primal/`
- ToadStool + Songbird: `../../phase1/toadStool/showcase/ecosystem/`

---

## 🎓 Learning Outcomes

After completing this level, you'll understand:
1. **Pure Infant Discovery**: How primals find each other at runtime
2. **Capability-Based Architecture**: Discovery by what, not who
3. **Mesh Topology**: How Songbird federates services
4. **Health Monitoring**: How ecosystem maintains awareness
5. **Integration Gaps**: Real issues discovered through interaction

---

## 🔗 Next Level

Once Songbird foundation is solid:
```bash
cd ../02-beardog-signing
cat README.md
```

**Build on**: Discovery foundation → Add identity/signing

---

*"No hardcoded addresses. Pure capability discovery. Runtime coordination."* 🎵

