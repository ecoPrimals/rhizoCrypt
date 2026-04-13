# `dag.event.append` — Event Type Reference

**Date**: April 13, 2026
**Version**: 0.14.0-dev
**Canonical source**: `crates/rhizo-crypt-core/src/event.rs` → `pub enum EventType`

---

## Wire Format

`EventType` uses serde's default **externally-tagged** representation:

- **Variants with fields**: `{"VariantName": {"field": value}}`
- **Unit variants (no fields)**: `"VariantName"`

The enum is `#[non_exhaustive]` — new variants may be added without a major version bump.

### Full `dag.event.append` JSON-RPC Request

```json
{
  "jsonrpc": "2.0",
  "method": "dag.event.append",
  "params": {
    "session_id": "019d875d-...",
    "event_type": {"DataCreate": {"schema": "sensor-readings-v2"}},
    "agent": "did:key:z6Mk...",
    "parents": [],
    "metadata": [["source", "sensor_array_north"]],
    "payload_ref": null
  },
  "id": 1
}
```

---

## Variant Reference (27 variants)

### Session Lifecycle

| Variant | Fields | JSON Example |
|---------|--------|-------------|
| `SessionStart` | *(none)* | `"SessionStart"` |
| `SessionEnd` | `outcome: SessionOutcome` | `{"SessionEnd": {"outcome": "Success"}}` |

`SessionOutcome` values: `"Success"`, `"Timeout"`, `"Cancelled"`, `"Rollback"`, `{"Failure": {"reason": "..."}}`

---

### Agent Events

| Variant | Fields | JSON Example |
|---------|--------|-------------|
| `AgentJoin` | `role: AgentRole` | `{"AgentJoin": {"role": "Participant"}}` |
| `AgentLeave` | `reason: LeaveReason` | `{"AgentLeave": {"reason": "Normal"}}` |
| `AgentAction` | `action: String` | `{"AgentAction": {"action": "classify"}}` |

`AgentRole` values: `"Owner"`, `"Participant"`, `"Observer"`, `{"Custom": "analyst"}`

`LeaveReason` values: `"Normal"`, `"Kicked"`, `"Disconnected"`, `"Timeout"`

---

### Data Events

| Variant | Fields | JSON Example |
|---------|--------|-------------|
| `DataCreate` | `schema: Option<String>` | `{"DataCreate": {"schema": "sensor-v2"}}` |
| `DataModify` | `delta_type: String` | `{"DataModify": {"delta_type": "append"}}` |
| `DataDelete` | *(none)* | `"DataDelete"` |
| `DataTransfer` | `to: Did` | `{"DataTransfer": {"to": "did:key:z6Mk..."}}` |

**Note**: `schema` is optional. `{"DataCreate": {"schema": null}}` is valid.

---

### Slice Events

| Variant | Fields | JSON Example |
|---------|--------|-------------|
| `SliceCheckout` | `slice_id: SliceId`, `mode: SliceMode` | `{"SliceCheckout": {"slice_id": "...", "mode": {"Copy": {"allow_recopy": true}}}}` |
| `SliceOperation` | `slice_id: SliceId`, `operation: String` | `{"SliceOperation": {"slice_id": "...", "operation": "transform"}}` |
| `SliceResolve` | `slice_id: SliceId`, `resolution: ResolutionType` | `{"SliceResolve": {"slice_id": "...", "resolution": "CommitToOrigin"}}` |

`SliceMode` values:
- `{"Copy": {"allow_recopy": bool}}`
- `{"Loan": {"allow_subloan": bool}}`
- `{"Consignment": {"consignee": "did:..."}}`
- `{"Escrow": {"required_confirmations": u32}}`
- `{"Waypoint": {"waypoint_spine": "spine-id"}}`
- `{"Transfer": {"new_owner": "did:..."}}`

`ResolutionType` values: `"ReturnToOrigin"`, `"CommitToOrigin"`, `"Consumed"`, `{"RouteToSpine": {"target_spine": "..."}}`

---

### Gaming Domain

| Variant | Fields | JSON Example |
|---------|--------|-------------|
| `GameEvent` | `game_type: String`, `event_name: String` | `{"GameEvent": {"game_type": "extraction", "event_name": "zone_enter"}}` |
| `ItemLoot` | `item_type: String` | `{"ItemLoot": {"item_type": "weapon"}}` |
| `ItemDrop` | *(none)* | `"ItemDrop"` |
| `ItemTransfer` | `to: Did` | `{"ItemTransfer": {"to": "did:key:z6Mk..."}}` |
| `Combat` | `target: Did`, `outcome: String` | `{"Combat": {"target": "did:key:z6Mk...", "outcome": "victory"}}` |
| `Extraction` | `success: bool` | `{"Extraction": {"success": true}}` |

---

### Scientific Domain

| Variant | Fields | JSON Example |
|---------|--------|-------------|
| `ExperimentStart` | `protocol: String` | `{"ExperimentStart": {"protocol": "lattice-qcd-v3"}}` |
| `Observation` | `instrument: String` | `{"Observation": {"instrument": "spectrometer-alpha"}}` |
| `Analysis` | `method: String` | `{"Analysis": {"method": "bayesian-regression"}}` |
| `Result` | `confidence_percent: u8` | `{"Result": {"confidence_percent": 95}}` |

**Note for domain springs**: Use `metadata` on the append request for domain-specific fields
(experiment IDs, observation IDs, etc.) rather than expecting them in the event type itself.
The event type captures the *kind* of action; rich context goes in `metadata` and `payload_ref`.

---

### Collaboration Domain

| Variant | Fields | JSON Example |
|---------|--------|-------------|
| `DocumentEdit` | `operation: String` | `{"DocumentEdit": {"operation": "insert"}}` |
| `CommentAdd` | *(none)* | `"CommentAdd"` |
| `ApprovalGrant` | *(none)* | `"ApprovalGrant"` |
| `ApprovalRevoke` | *(none)* | `"ApprovalRevoke"` |

---

### Custom (any domain)

| Variant | Fields | JSON Example |
|---------|--------|-------------|
| `Custom` | `domain: String`, `event_name: String` | `{"Custom": {"domain": "ecology", "event_name": "species_observation"}}` |

Use `Custom` for domain-specific events that don't fit the built-in variants.
The `domain` field should use the spring's capability domain (e.g., `"ecology"`,
`"genomics"`, `"qcd"`, `"geoscience"`).

---

## Domain Mapping

| Domain | Variants | Primary Consumer |
|--------|----------|-----------------|
| `session` | `SessionStart`, `SessionEnd` | All |
| `agent` | `AgentJoin`, `AgentLeave`, `AgentAction` | All |
| `data` | `DataCreate`, `DataModify`, `DataDelete`, `DataTransfer` | All |
| `slice` | `SliceCheckout`, `SliceOperation`, `SliceResolve` | Storage-aware springs |
| `gaming` | `GameEvent`, `ItemLoot`, `ItemDrop`, `ItemTransfer`, `Combat`, `Extraction` | ludoSpring |
| `science` | `ExperimentStart`, `Observation`, `Analysis`, `Result` | wetSpring, hotSpring, groundSpring, airSpring |
| `collaboration` | `DocumentEdit`, `CommentAdd`, `ApprovalGrant`, `ApprovalRevoke` | esotericWebb |
| `custom` | `Custom` | Any domain spring |

---

## Guidance for Domain Springs

1. **Use built-in variants when they fit.** `ExperimentStart` + `Observation` + `Analysis` + `Result`
   covers the standard science workflow. Put experiment IDs, sample IDs, and other rich context
   in `metadata` key-value pairs on the append request.

2. **Use `Custom` for domain-specific events.** Set `domain` to your spring's capability
   domain so provenance queries can filter by domain.

3. **Unit variants have no fields.** `"DataDelete"`, `"ItemDrop"`, `"CommentAdd"`,
   `"ApprovalGrant"`, `"ApprovalRevoke"` serialize as bare strings, not objects.

4. **`Did` fields are strings.** Format: `"did:key:z6Mk..."` or any valid DID URI.

5. **`SliceId` and `SessionId` are UUID strings.** Format: `"019d875d-..."`.

6. **The enum is `#[non_exhaustive]`.** Your deserialization should handle unknown variants
   gracefully (serde will error on unknown variants by default — use `#[serde(other)]`
   on a catch-all if you need forward compatibility).
