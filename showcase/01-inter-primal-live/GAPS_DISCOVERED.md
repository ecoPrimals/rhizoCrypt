# 🔍 Gaps Discovered During Live Phase 1 Integration

**Purpose**: Document every gap, mismatch, and issue discovered while integrating rhizoCrypt with real Phase 1 binaries.

**Philosophy**: "Interactions show us gaps in our evolution"

---

## Gap Template

```markdown
## Gap #N: [Short Description]

**Discovered**: YYYY-MM-DD  
**Primal**: [Which Phase 1 primal]  
**Severity**: Critical/High/Medium/Low  
**Demo**: [Which demo file]

### Expected Behavior
[What we thought would happen]

### Actual Behavior
[What actually happened]

### Root Cause
[Why it happened]

### Impact
- [Effect on functionality]
- [Effect on integration]
- [Effect on user experience]

### Fix Required
**In rhizoCrypt**:
- [Changes needed in rhizoCrypt]

**In Phase 1 Primal**:
- [Changes needed in the other primal, if any]

**In Documentation**:
- [Documentation updates needed]

### Status
- [ ] Root cause identified
- [ ] Fix implemented
- [ ] Tests added
- [ ] Documentation updated
- [ ] Verified in integration

### References
- [Links to related issues, docs, code]
```

---

## Discovered Gaps

### Gap #1: Songbird Port and Protocol Mismatch

**Discovered**: 2025-12-24  
**Primal**: Songbird  
**Severity**: High  
**Demo**: `01-songbird-discovery/start-songbird.sh`

#### Expected Behavior
Based on initial assumptions, expected Songbird rendezvous to:
- Listen on port 7878
- Use tarpc protocol
- Require TLS certificates

#### Actual Behavior
Songbird rendezvous actually:
- Listens on port **8888** (not 7878)
- Uses **HTTP/REST API** (not tarpc)
- Endpoints:
  - `POST /api/v1/register` - Register node presence
  - `POST /api/v1/query` - Query for peers
  - `POST /api/v1/connect` - Request connection
  - `WS /ws/:session_id` - Real-time coordination (WebSocket)

#### Root Cause
Documentation/assumption mismatch. The rendezvous server uses HTTP/REST for simplicity and broad compatibility, while tower-to-tower coordination may use tarpc.

#### Impact
- rhizoCrypt Songbird client needs HTTP client (not tarpc client)
- Port configuration wrong in scripts
- Protocol layer mismatch
- **GOOD NEWS**: HTTP/REST is easier to integrate than tarpc!

#### Fix Required

**In rhizoCrypt**:
- Update `crates/rhizo-crypt-core/src/clients/songbird.rs`:
  - Add `reqwest` dependency for HTTP client
  - Implement REST API calls (register, query, connect)
  - Add WebSocket support for real-time coordination
- Update configuration to use port 8888

**In Documentation**:
- Correct port numbers in all docs (7878 → 8888)
- Document HTTP/REST API (not tarpc)
- Add example REST payloads

**In Scripts**:
- Fix port detection (check 8888, not 7878)
- Update environment variables

#### Status
- [x] Root cause identified (HTTP/REST, port 8888)
- [x] Fix path determined (reqwest client)
- [ ] Fix implemented
- [ ] Tests added
- [ ] Documentation updated
- [ ] Verified in integration

#### References
- Songbird log output shows actual API endpoints
- `songbird-rendezvous` binary version: v0.1.0

---

## Gap Summary

| # | Description | Primal | Severity | Status |
|---|-------------|--------|----------|--------|
| 1 | Port 8888 & HTTP/REST (not 7878/tarpc) | Songbird | High | Identified |

---

### Gap #2: Short Session Expiry (60 seconds)

**Discovered**: 2025-12-24  
**Primal**: Songbird  
**Severity**: Medium  
**Demo**: `01-songbird-discovery/demo-register.sh`

#### Expected Behavior
Expected session registration to last for several minutes or require explicit refresh.

#### Actual Behavior
Songbird registration expires after 60 seconds:
```json
{
  "expires_at": "2025-12-24T19:15:17Z"  // 60 seconds from registration
}
```

#### Root Cause
Rendezvous protocol specifies 30-60 second heartbeats for privacy (ephemeral sessions).

#### Impact
- rhizoCrypt must implement heartbeat/refresh mechanism
- Registration not persistent
- Requires background task to maintain presence
- **GOOD**: Aligns with ephemeral, privacy-first design

#### Fix Required

**In rhizoCrypt**:
- Add background heartbeat task (every 30-45 seconds)
- Re-register before expiry
- Handle registration failures gracefully
- Log registration status changes

**Status**:
- [x] Root cause identified (intentional design)
- [ ] Heartbeat mechanism implemented
- [ ] Tests added
- [ ] Documentation updated

#### References
- RENDEZVOUS_PROTOCOL_SPEC.md specifies 30-60s heartbeats
- Aligns with privacy-first, ephemeral session design

---

### Gap #3: Query API Requires All Fields (capabilities_optional, exclude_node_ids)

**Discovered**: 2025-12-24  
**Primal**: Songbird  
**Severity**: Low  
**Demo**: `01-songbird-discovery/demo-discover.sh`

#### Expected Behavior
Based on RENDEZVOUS_PROTOCOL_SPEC.md, optional fields like `capabilities_optional` and `exclude_node_ids` should be optional.

#### Actual Behavior
Query API requires ALL fields even if empty:
- `capabilities_optional`: Required (even if `[]`)
- `exclude_node_ids`: Required (even if `[]`)

#### Root Cause
Implementation uses strict deserialization. Spec shows fields as optional, but Rust serde requires them unless marked with `#[serde(default)]`.

#### Impact
- Must include empty arrays for optional fields
- Minor inconvenience, easy workaround
- **GOOD**: Explicit is better than implicit!

#### Fix Required

**In Scripts**:
- Add `"capabilities_optional": []` to all query payloads ✅
- Add `"exclude_node_ids": []` to all query payloads ✅

#### Status
- [x] Root cause identified
- [x] Fix implemented
- [x] Verified — Discovery working perfectly!

---

## Gap Summary

| # | Description | Primal | Severity | Status |
|---|-------------|--------|----------|--------|
| 1 | Port 8888 & HTTP/REST (not 7878/tarpc) | Songbird | High | ✅ Fixed |
| 2 | Short session expiry (60s heartbeat) | Songbird | Medium | Identified |
| 3 | Query requires all fields (optional arrays) | Songbird | Low | ✅ Fixed |

---

## Gap Statistics

**Total Gaps**: 3  
**By Severity**:
- Critical: 0
- High: 1 (✅ fixed)
- Medium: 1 (planned)
- Low: 1 (✅ fixed)

**By Status**:
- Open: 1 (heartbeat mechanism)
- Fixed: 2 (port/protocol, query fields)
- Verified: 2

**By Status**:
- Open: 0
- In Progress: 0
- Fixed: 0
- Verified: 0

**By Primal**:
- Songbird: 0
- BearDog: 0
- NestGate: 0
- ToadStool: 0
- Squirrel: 0
- LoamSpine: 0 (future)

---

## Lessons Learned

_Will be populated as we progress through integration._

---

*Last Updated: Dec 24, 2025*

