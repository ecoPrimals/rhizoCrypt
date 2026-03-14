# RhizoCrypt — Slice Semantics Specification

**Version**: 0.2.0  
**Status**: Draft  
**Last Updated**: December 22, 2025

---

## 1. Overview

This document defines the **Slice Semantics** system—the mechanism by which LoamSpine state can be "checked out" into a RhizoCrypt DAG for asynchronous operations, and how that state resolves back to permanence.

Slices are the key innovation enabling reversible transactions, lending, consignment, and other complex ownership patterns in the ecoPrimals ecosystem.

---

## 2. The Rhizo-Loam Layer Model

```
════════════════════════════════════════════════════════════════════
                    THE RHIZO-LOAM LAYER CAKE
════════════════════════════════════════════════════════════════════

        ┌─────────────────────────────────────────────────────┐
        │                  gAIa COMMONS                       │
        │           (Global permanent anchor)                 │
        └─────────────────────────┬───────────────────────────┘
                                  │
        ┌─────────────────────────┼───────────────────────────┐
        │                         │                           │
        ▼                         ▼                           ▼
    ┌────────┐              ┌────────┐                  ┌────────┐
    │ Spine A│              │ Spine B│                  │ Spine C│
    │(Linear)│              │(Linear)│                  │(Linear)│
    └───┬────┘              └───┬────┘                  └───┬────┘
        │                       │                           │
        │ LOAM LAYER ═══════════╪═══════════════════════════╪═════
        │ (Permanent)           │                           │
        │                       │                           │
        ▼                       ▼                           ▼
    ┌────────┐              ┌────────┐                  ┌────────┐
    │  DAG   │◄────────────►│  DAG   │◄────────────────►│  DAG   │
    │(Branch)│   slice      │(Branch)│    slice         │(Branch)│
    └────────┘   transit    └────────┘    transit       └────────┘
        │                       │                           │
        │ RHIZO LAYER ══════════╪═══════════════════════════╪═════
        │ (Ephemeral)           │                           │
        │                       │                           │
        ▼                       ▼                           ▼
    ┌────────┐              ┌────────┐                  ┌────────┐
    │Waypoint│              │Waypoint│                  │Waypoint│
    │ Spine  │              │ Spine  │                  │ Spine  │
    └────────┘              └────────┘                  └────────┘
        │                       │                           │
        │ WAYPOINT LAYER ═══════╪═══════════════════════════╪═════
        │ (Local permanence,    │                           │
        │  no upward propagation)                           │
        
════════════════════════════════════════════════════════════════════
```

| Layer | Persistence | Propagation | Purpose |
|-------|-------------|-------------|---------|
| gAIa Commons | Eternal | Global | SOVEREIGN SCIENCE anchor |
| Canonical Spines | Permanent | Federated | Personal/org truth |
| RhizoCrypt DAGs | Ephemeral | N/A | Async operations |
| Waypoint Spines | Local permanent | **No upward** | Borrowed state |

---

## 3. Slice Data Structures

### 3.1 Slice Reference

```rust
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Unique slice identifier
pub type SliceId = Uuid;

/// A slice of LoamSpine state checked out into RhizoCrypt
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Slice {
    /// Unique slice identifier
    pub id: SliceId,
    
    /// Origin in LoamSpine
    pub origin: SliceOrigin,
    
    /// Current holder of the slice
    pub holder: Did,
    
    /// Original owner (always the origin owner)
    pub owner: Did,
    
    /// Slice mode (determines behavior)
    pub mode: SliceMode,
    
    /// Resolution routing
    pub resolution_route: ResolutionRoute,
    
    /// Checkout timestamp
    pub checked_out_at: u64,
    
    /// Expiration timestamp (if applicable)
    pub expires_at: Option<u64>,
    
    /// Constraints on slice usage
    pub constraints: SliceConstraints,
    
    /// Current state
    pub state: SliceState,
    
    /// Session containing this slice
    pub session_id: SessionId,
    
    /// Vertex that created this slice
    pub checkout_vertex: VertexId,
}

/// Origin of a slice in LoamSpine
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SliceOrigin {
    /// Source spine ID
    pub spine_id: SpineId,
    
    /// Source entry hash
    pub entry_hash: EntryHash,
    
    /// Source entry index
    pub entry_index: u64,
    
    /// Certificate ID (if slice is of a certificate)
    pub certificate_id: Option<CertificateId>,
    
    /// Original owner DID
    pub owner: Did,
}

/// Current state of a slice
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum SliceState {
    /// Active in a DAG session
    Active {
        session_id: SessionId,
    },
    
    /// Anchored at a waypoint spine
    Anchored {
        waypoint_spine: SpineId,
        anchor_entry: EntryHash,
    },
    
    /// Being resolved
    Resolving {
        started_at: u64,
    },
    
    /// Resolution complete
    Resolved {
        resolution: ResolutionOutcome,
        resolved_at: u64,
    },
}
```

### 3.2 Slice Modes

```rust
/// Slice mode determines behavior and resolution
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum SliceMode {
    /// COPY: Local use only, disconnected from lineage
    /// 
    /// Use case: Give a friend a game to play offline
    /// Cannot: Verify against network, transfer with provenance, return with history
    Copy {
        /// Whether the copy can be further copied
        allow_recopy: bool,
    },
    
    /// LOAN: Borrower has temporary use rights
    /// 
    /// Use case: Lend game to friend for weekend
    /// Returns: Automatically on expiry or manual return
    Loan {
        /// Loan terms
        terms: LoanTerms,
        /// Whether borrower can sub-loan
        allow_subloan: bool,
    },
    
    /// CONSIGNMENT: Holder has possession, owner retains ownership
    /// 
    /// Use case: Auction house holds item until sale
    /// Returns: Based on resolution triggers (sale, timeout, cancel)
    Consignment {
        /// The consignee (temporary holder)
        consignee: Did,
        /// Events that trigger resolution
        resolution_triggers: Vec<ResolutionTrigger>,
    },
    
    /// ESCROW: Held pending multi-party agreement
    /// 
    /// Use case: Trade between players
    /// Returns: When all parties confirm or timeout
    Escrow {
        /// Parties involved in escrow
        parties: Vec<Did>,
        /// Required confirmations to release
        required_confirmations: usize,
        /// Current confirmations
        confirmations: HashSet<Did>,
    },
    
    /// WAYPOINT: Anchors to holder's local spine for local operations
    /// 
    /// Use case: Friend's local spine tracks their usage, then returns
    /// Returns: When waypoint session resolves
    Waypoint {
        /// The waypoint spine ID
        waypoint_spine: SpineId,
        /// Return conditions
        return_conditions: ReturnConditions,
    },
    
    /// TRANSFER: Full ownership transfer on resolution
    /// 
    /// Use case: Sale, gift, permanent transfer
    /// Does not return: Ownership changes permanently
    Transfer {
        /// New owner on resolution
        new_owner: Did,
        /// Conditions for transfer to complete
        conditions: Option<TransferConditions>,
    },
}

impl SliceMode {
    /// Check if this mode allows lineage back to origin
    pub fn can_lineage_back(&self) -> bool {
        !matches!(self, Self::Copy { .. })
    }
    
    /// Check if this mode transfers ownership
    pub fn transfers_ownership(&self) -> bool {
        matches!(self, Self::Transfer { .. })
    }
    
    /// Check if holder is temporary
    pub fn is_temporary_hold(&self) -> bool {
        matches!(
            self,
            Self::Loan { .. } | Self::Consignment { .. } | Self::Escrow { .. } | Self::Waypoint { .. }
        )
    }
    
    /// Get the primary holder DID
    pub fn holder(&self) -> Option<&Did> {
        match self {
            Self::Consignment { consignee, .. } => Some(consignee),
            Self::Transfer { new_owner, .. } => Some(new_owner),
            _ => None,
        }
    }
}
```

### 3.3 Loan Terms

```rust
/// Terms for a loan slice
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LoanTerms {
    /// Loan duration (None = indefinite until manual return)
    pub duration: Option<Duration>,
    
    /// Grace period after expiry before auto-recall
    pub grace_period: Option<Duration>,
    
    /// Automatic return on expiry
    pub auto_return: bool,
    
    /// Use restrictions
    pub restrictions: Vec<UseRestriction>,
    
    /// Return conditions
    pub return_conditions: ReturnConditions,
}

/// Restrictions on slice usage
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum UseRestriction {
    /// Can only use in specific session types
    SessionTypeOnly(Vec<SessionType>),
    
    /// Can only use in specific geographic regions
    GeoFence(Vec<GeoRegion>),
    
    /// Maximum usage duration per session
    MaxSessionDuration(Duration),
    
    /// Maximum total usage time
    MaxTotalUsage(Duration),
    
    /// Specific operations forbidden
    ForbiddenOperations(Vec<String>),
    
    /// Specific operations required for certain actions
    RequiredAttestations(HashMap<String, Vec<Did>>),
}

/// Conditions for returning a slice
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ReturnConditions {
    /// Require certain operations before return
    pub required_operations: Vec<String>,
    
    /// Require attestation from specific parties
    pub required_attestations: Vec<Did>,
    
    /// Include usage summary in return
    pub include_usage_summary: bool,
    
    /// Propagate usage to origin spine
    pub propagate_usage: PropagationPolicy,
}
```

### 3.4 Resolution Routing

```rust
/// How a slice resolves when its session ends
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum ResolutionRoute {
    /// Return to origin spine unchanged
    ReturnToOrigin,
    
    /// Commit new state to origin spine
    CommitToOrigin {
        /// New entry to append
        entry_type: EntryType,
        /// Include usage summary
        include_summary: bool,
    },
    
    /// Route to a different spine (for transfers)
    RouteToSpine {
        /// Target spine ID
        target_spine: SpineId,
        /// Entry to create
        entry_type: EntryType,
    },
    
    /// Route through waypoint, then back to origin
    WaypointReturn {
        /// Waypoint spine to anchor
        waypoint_spine: SpineId,
        /// Entry for waypoint
        waypoint_entry: EntryType,
        /// Update for origin (if any)
        origin_update: Option<EntryType>,
    },
    
    /// Conditional routing based on outcome
    Conditional {
        /// Condition → Route mappings
        conditions: Vec<ConditionalRoute>,
        /// Default route if no condition matches
        default: Box<ResolutionRoute>,
    },
}

/// A conditional route
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ConditionalRoute {
    /// Condition that triggers this route
    pub condition: ResolutionCondition,
    /// Route to take if condition is met
    pub route: ResolutionRoute,
}

/// Conditions for resolution routing
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum ResolutionCondition {
    /// Session resolved with success
    SessionSuccess,
    
    /// Session resolved with failure/rollback
    SessionRollback,
    
    /// Session timed out
    SessionTimeout,
    
    /// Specific event occurred in DAG
    EventOccurred {
        event_type: String,
        payload_match: Option<PayloadMatcher>,
    },
    
    /// External trigger received
    ExternalTrigger {
        trigger_id: String,
    },
    
    /// All parties confirmed (for escrow)
    AllPartiesConfirmed,
    
    /// Loan term expired
    LoanExpired,
    
    /// Owner recalled slice
    OwnerRecall,
}

/// Resolution outcome
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum ResolutionOutcome {
    /// Returned to origin unchanged
    ReturnedUnchanged,
    
    /// Committed update to origin
    CommittedToOrigin {
        entry_hash: EntryHash,
    },
    
    /// Transferred to new spine
    Transferred {
        new_spine: SpineId,
        new_entry: EntryHash,
        new_owner: Did,
    },
    
    /// Anchored at waypoint
    AnchoredAtWaypoint {
        waypoint_spine: SpineId,
        waypoint_entry: EntryHash,
    },
    
    /// Consumed (e.g., one-time use)
    Consumed,
}
```

---

## 4. Slice Operations

### 4.1 Checkout Flow

```rust
/// Slice checkout request
#[derive(Clone, Debug)]
pub struct CheckoutRequest {
    /// Source spine
    pub spine_id: SpineId,
    /// Entry to slice
    pub entry_hash: EntryHash,
    /// Slice mode
    pub mode: SliceMode,
    /// Destination session
    pub session_id: SessionId,
    /// Resolution route
    pub resolution_route: ResolutionRoute,
    /// Constraints
    pub constraints: SliceConstraints,
    /// Requester DID
    pub requester: Did,
}

/// Slice checkout response
#[derive(Clone, Debug)]
pub struct CheckoutResponse {
    /// Created slice
    pub slice: Slice,
    /// Checkout vertex ID
    pub vertex_id: VertexId,
}

/// Checkout a slice from LoamSpine into a RhizoCrypt session
pub async fn checkout_slice(
    request: CheckoutRequest,
    loamspine: &impl PermanentStorageProvider,
    session_manager: &SessionManager,
    beardog: &impl SigningProvider,
) -> Result<CheckoutResponse, RhizoCryptError> {
    // 1. Validate requester has permission
    let permissions = beardog
        .check_permissions(&request.requester, &request.spine_id, "slice:checkout")
        .await
        .map_err(|e| RhizoCryptError::capability_provider("signing", e.to_string()))?;
    
    if !permissions.allowed {
        return Err(RhizoCryptError::SlicePermissionDenied);
    }
    
    // 2. Validate session is active
    let session = session_manager.get_session(request.session_id).await?;
    if !session.state.is_active() {
        return Err(RhizoCryptError::SessionNotActive(request.session_id));
    }
    
    // 3. Get origin entry from LoamSpine
    let origin_entry = loamspine
        .get_entry(&request.spine_id, &request.entry_hash)
        .await
        .map_err(|e| RhizoCryptError::capability_provider("permanent_storage", e.to_string()))?
        .ok_or(RhizoCryptError::SliceNotFound(SliceId::nil()))?;
    
    // 4. Create slice
    let slice_id = SliceId::now_v7();
    let slice = Slice {
        id: slice_id,
        origin: SliceOrigin {
            spine_id: request.spine_id.clone(),
            entry_hash: request.entry_hash,
            entry_index: origin_entry.index,
            certificate_id: extract_certificate_id(&origin_entry),
            owner: origin_entry.committer.clone(),
        },
        holder: request.requester.clone(),
        owner: origin_entry.committer.clone(),
        mode: request.mode,
        resolution_route: request.resolution_route,
        checked_out_at: current_timestamp_nanos(),
        expires_at: compute_expiry(&request.mode),
        constraints: request.constraints,
        state: SliceState::Active { session_id: request.session_id },
        session_id: request.session_id,
        checkout_vertex: VertexId::default(), // Set after vertex creation
    };
    
    // 5. Create checkout vertex in session
    let vertex = VertexBuilder::new(EventType::SliceCheckout {
        slice_id,
        mode: slice.mode.clone(),
    })
    .with_agent(request.requester)
    .with_metadata("origin_spine", request.spine_id.to_string())
    .with_metadata("origin_entry", hex::encode(request.entry_hash))
    .build();
    
    let vertex_id = session_manager.append_vertex(request.session_id, vertex).await?;
    
    // 6. Register slice with session
    session_manager.register_slice(request.session_id, slice.clone()).await?;
    
    Ok(CheckoutResponse {
        slice: Slice { checkout_vertex: vertex_id, ..slice },
        vertex_id,
    })
}
```

### 4.2 Resolution Flow

```rust
/// Resolve all slices in a session
pub async fn resolve_slices(
    session: &Session,
    outcome: &SessionOutcome,
    loamspine: &impl PermanentStorageProvider,
    beardog: &impl SigningProvider,
) -> Result<Vec<SliceResolution>, RhizoCryptError> {
    let mut resolutions = Vec::new();
    
    for (slice_id, slice) in &session.slices {
        let resolution = resolve_single_slice(
            slice,
            outcome,
            loamspine,
            beardog,
        ).await?;
        
        resolutions.push(SliceResolution {
            slice_id: *slice_id,
            outcome: resolution,
        });
    }
    
    Ok(resolutions)
}

/// Resolve a single slice
async fn resolve_single_slice(
    slice: &Slice,
    session_outcome: &SessionOutcome,
    loamspine: &impl PermanentStorageProvider,
    beardog: &impl SigningProvider,
) -> Result<ResolutionOutcome, RhizoCryptError> {
    // Determine actual route based on session outcome and conditions
    let route = evaluate_resolution_route(
        &slice.resolution_route,
        session_outcome,
        slice,
    )?;
    
    match route {
        ResolutionRoute::ReturnToOrigin => {
            // No changes to LoamSpine
            Ok(ResolutionOutcome::ReturnedUnchanged)
        }
        
        ResolutionRoute::CommitToOrigin { entry_type, include_summary } => {
            let summary = if include_summary {
                Some(compute_usage_summary(slice))
            } else {
                None
            };
            
            let entry_hash = loamspine
                .append_entry(
                    &slice.origin.spine_id,
                    entry_type,
                    summary,
                )
                .await
                .map_err(|e| RhizoCryptError::capability_provider("permanent_storage", e.to_string()))?;
            
            Ok(ResolutionOutcome::CommittedToOrigin { entry_hash })
        }
        
        ResolutionRoute::RouteToSpine { target_spine, entry_type } => {
            let entry_hash = loamspine
                .append_entry(&target_spine, entry_type, None)
                .await
                .map_err(|e| RhizoCryptError::capability_provider("permanent_storage", e.to_string()))?;
            
            let new_owner = extract_new_owner(&slice.mode);
            
            Ok(ResolutionOutcome::Transferred {
                new_spine: target_spine,
                new_entry: entry_hash,
                new_owner,
            })
        }
        
        ResolutionRoute::WaypointReturn { waypoint_spine, waypoint_entry, origin_update } => {
            // Anchor at waypoint
            let waypoint_hash = loamspine
                .append_entry(&waypoint_spine, waypoint_entry, None)
                .await
                .map_err(|e| RhizoCryptError::capability_provider("permanent_storage", e.to_string()))?;
            
            // Optionally update origin
            if let Some(update) = origin_update {
                loamspine
                    .append_entry(&slice.origin.spine_id, update, None)
                    .await
                    .map_err(|e| RhizoCryptError::capability_provider("permanent_storage", e.to_string()))?;
            }
            
            Ok(ResolutionOutcome::AnchoredAtWaypoint {
                waypoint_spine,
                waypoint_entry: waypoint_hash,
            })
        }
        
        ResolutionRoute::Conditional { .. } => {
            // Should have been evaluated already
            Err(RhizoCryptError::Internal("Conditional route not evaluated".into()))
        }
    }
}

/// Evaluate conditional routing
fn evaluate_resolution_route(
    route: &ResolutionRoute,
    session_outcome: &SessionOutcome,
    slice: &Slice,
) -> Result<ResolutionRoute, RhizoCryptError> {
    match route {
        ResolutionRoute::Conditional { conditions, default } => {
            for cond in conditions {
                if matches_condition(&cond.condition, session_outcome, slice) {
                    return evaluate_resolution_route(&cond.route, session_outcome, slice);
                }
            }
            evaluate_resolution_route(default, session_outcome, slice)
        }
        other => Ok(other.clone()),
    }
}

/// Check if a condition matches
fn matches_condition(
    condition: &ResolutionCondition,
    session_outcome: &SessionOutcome,
    slice: &Slice,
) -> bool {
    match (condition, session_outcome) {
        (ResolutionCondition::SessionSuccess, SessionOutcome::Success) => true,
        (ResolutionCondition::SessionRollback, SessionOutcome::Rollback) => true,
        (ResolutionCondition::SessionTimeout, SessionOutcome::Timeout) => true,
        (ResolutionCondition::LoanExpired, _) => {
            if let Some(expires) = slice.expires_at {
                current_timestamp_nanos() >= expires
            } else {
                false
            }
        }
        (ResolutionCondition::AllPartiesConfirmed, _) => {
            if let SliceMode::Escrow { parties, confirmations, required_confirmations } = &slice.mode {
                confirmations.len() >= *required_confirmations
            } else {
                false
            }
        }
        _ => false,
    }
}
```

---

## 5. Use Case Examples

### 5.1 Game Item Lending

```rust
// Alice lends a rare sword to Bob for the weekend

// 1. Alice checks out her sword from her LoamSpine
let checkout = checkout_slice(CheckoutRequest {
    spine_id: alice_spine,
    entry_hash: sword_certificate_entry,
    mode: SliceMode::Loan {
        terms: LoanTerms {
            duration: Some(Duration::from_secs(48 * 3600)), // 48 hours
            grace_period: Some(Duration::from_secs(3600)),  // 1 hour grace
            auto_return: true,
            restrictions: vec![],
            return_conditions: ReturnConditions {
                include_usage_summary: true,
                propagate_usage: PropagationPolicy::SummaryOnly,
                ..Default::default()
            },
        },
        allow_subloan: false,
    },
    session_id: lending_session,
    resolution_route: ResolutionRoute::CommitToOrigin {
        entry_type: EntryType::SliceReturn {
            slice_id: SliceId::nil(), // Filled in
            loan_entry: EntryHash::default(),
            waypoint_spine: bob_spine,
            waypoint_summary: None, // Filled during resolution
        },
        include_summary: true,
    },
    constraints: SliceConstraints::default(),
    requester: alice_did,
}).await?;

// 2. Bob uses the sword in his gaming sessions
// (Events recorded in the DAG)

// 3. Loan expires, slice auto-resolves
// Alice's spine receives a SliceReturn entry with Bob's usage summary
// Bob can no longer use the sword
```

### 5.2 Auction Consignment

```rust
// Alice puts a rare item up for auction

// 1. Alice checks out item as consignment to auction house
let checkout = checkout_slice(CheckoutRequest {
    spine_id: alice_spine,
    entry_hash: item_certificate_entry,
    mode: SliceMode::Consignment {
        consignee: auction_house_did,
        resolution_triggers: vec![
            ResolutionTrigger::Event {
                event_type: "auction.sold".to_string(),
            },
            ResolutionTrigger::Event {
                event_type: "auction.cancelled".to_string(),
            },
            ResolutionTrigger::Timeout {
                duration: Duration::from_secs(7 * 24 * 3600), // 7 days
            },
        ],
    },
    session_id: auction_session,
    resolution_route: ResolutionRoute::Conditional {
        conditions: vec![
            ConditionalRoute {
                condition: ResolutionCondition::EventOccurred {
                    event_type: "auction.sold".to_string(),
                    payload_match: None,
                },
                route: ResolutionRoute::RouteToSpine {
                    target_spine: SpineId::placeholder(), // Determined by winning bid
                    entry_type: EntryType::CertificateTransfer {
                        cert_id: CertificateId::placeholder(),
                        from: alice_did.clone(),
                        to: Did::placeholder(), // Winner's DID
                        conditions: None,
                    },
                },
            },
        ],
        default: Box::new(ResolutionRoute::ReturnToOrigin),
    },
    constraints: SliceConstraints::default(),
    requester: alice_did,
}).await?;

// 2. Alice can still USE the item during the auction
// (She keeps playing with her sword while it's listed)

// 3. Auction ends
// - If sold: Item transfers to buyer's spine
// - If not sold: Item returns to Alice unchanged
```

### 5.3 Multi-Party Trade (Escrow)

```rust
// Alice and Bob want to trade items

// 1. Both check out their items into shared escrow session
let alice_checkout = checkout_slice(CheckoutRequest {
    spine_id: alice_spine,
    entry_hash: alice_item_entry,
    mode: SliceMode::Escrow {
        parties: vec![alice_did.clone(), bob_did.clone()],
        required_confirmations: 2,
        confirmations: HashSet::new(),
    },
    session_id: trade_session,
    resolution_route: ResolutionRoute::Conditional {
        conditions: vec![
            ConditionalRoute {
                condition: ResolutionCondition::AllPartiesConfirmed,
                route: ResolutionRoute::RouteToSpine {
                    target_spine: bob_spine,
                    entry_type: EntryType::CertificateTransfer { /* ... */ },
                },
            },
        ],
        default: Box::new(ResolutionRoute::ReturnToOrigin),
    },
    ..
}).await?;

let bob_checkout = checkout_slice(CheckoutRequest {
    // Similar for Bob's item going to Alice
    ..
}).await?;

// 2. Both parties confirm
session_manager.confirm_escrow(trade_session, alice_did).await?;
session_manager.confirm_escrow(trade_session, bob_did).await?;

// 3. Session resolves - both items swap owners atomically
// If either party backs out, both items return to original owners
```

---

## 6. Slice Constraints

```rust
/// Constraints on slice usage
#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct SliceConstraints {
    /// Maximum duration the slice can exist
    pub max_duration: Option<Duration>,
    
    /// Geographic restrictions
    pub geo_fence: Option<GeoFence>,
    
    /// Allowed operations
    pub allowed_operations: Option<HashSet<String>>,
    
    /// Forbidden operations
    pub forbidden_operations: HashSet<String>,
    
    /// Whether slice can be re-sliced (sub-lending)
    pub allow_reslice: bool,
    
    /// Maximum depth of re-slicing
    pub max_reslice_depth: Option<u32>,
    
    /// Current reslice depth
    pub current_reslice_depth: u32,
    
    /// Required attestations for certain operations
    pub attestation_requirements: HashMap<String, Vec<Did>>,
}

impl SliceConstraints {
    /// Check if an operation is allowed
    pub fn is_operation_allowed(&self, operation: &str) -> bool {
        // Check forbidden list first
        if self.forbidden_operations.contains(operation) {
            return false;
        }
        
        // If allowed list exists, operation must be in it
        if let Some(allowed) = &self.allowed_operations {
            return allowed.contains(operation);
        }
        
        // Default: allowed
        true
    }
    
    /// Check if reslicing is allowed
    pub fn can_reslice(&self) -> bool {
        if !self.allow_reslice {
            return false;
        }
        
        if let Some(max_depth) = self.max_reslice_depth {
            return self.current_reslice_depth < max_depth;
        }
        
        true
    }
    
    /// Create constraints for a re-slice
    pub fn for_reslice(&self) -> Self {
        Self {
            current_reslice_depth: self.current_reslice_depth + 1,
            ..self.clone()
        }
    }
}
```

---

## 7. Waypoint Spine Semantics

Waypoint spines are a special type of LoamSpine that:
- Can anchor slices from other spines
- Provide local permanence for borrowed state
- **Cannot propagate upward** to parent spines or global commons

### 7.1 Propagation Policy

```rust
/// Policy for propagating waypoint operations
#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub enum PropagationPolicy {
    /// Never propagate to origin/upstream
    #[default]
    Never,
    
    /// Propagate summary only (e.g., "used for 10 hours")
    SummaryOnly,
    
    /// Propagate specific event types only
    Selective {
        allowed_types: Vec<String>,
    },
    
    /// Full propagation (rare, requires explicit consent)
    Full {
        require_owner_signature: bool,
    },
}

impl PropagationPolicy {
    /// Check if a specific event can propagate
    pub fn can_propagate(&self, event_type: &str) -> bool {
        match self {
            Self::Never => false,
            Self::SummaryOnly => false, // Only summary, not individual events
            Self::Selective { allowed_types } => allowed_types.contains(&event_type.to_string()),
            Self::Full { .. } => true,
        }
    }
}
```

### 7.2 Waypoint Entry Types

When a slice anchors at a waypoint, special entries are created:

```rust
/// Entry types for waypoint spines (in LoamSpine)
pub enum WaypointEntryType {
    /// Slice arrives at this waypoint
    SliceAnchor {
        slice_id: SliceId,
        origin_spine: SpineId,
        origin_entry: EntryHash,
        terms: LoanTerms,
    },
    
    /// Operation performed on anchored slice
    SliceOperation {
        slice_id: SliceId,
        operation: String,
        payload: Option<PayloadRef>,
    },
    
    /// Slice departs this waypoint
    SliceDeparture {
        slice_id: SliceId,
        reason: DepartureReason,
        destination: ResolutionRoute,
        summary: WaypointSummary,
    },
}

/// Summary of waypoint operations
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct WaypointSummary {
    /// Total duration at waypoint
    pub duration: Duration,
    
    /// Number of operations performed
    pub operation_count: u64,
    
    /// Operation types performed
    pub operation_types: Vec<String>,
    
    /// Hash of full operation log
    pub operations_hash: ContentHash,
}
```

---

## 8. Security Considerations

### 8.1 Permission Checks

All slice operations require signing provider permission checks:

| Operation | Required Permission |
|-----------|---------------------|
| Checkout slice | `loamspine:{spine}:slice:checkout` |
| Operate on slice | `rhizocrypt:session:{id}:slice:{slice_id}:operate` |
| Resolve slice | `rhizocrypt:session:{id}:admin` OR owner |
| Recall slice | `loamspine:{spine}:slice:{slice_id}:recall` |

### 8.2 Constraint Enforcement

Constraints are enforced at multiple levels:
- **RhizoCrypt**: Checks before appending vertices
- **Permanent storage provider**: Validates on commit
- **Signing provider**: Policy enforcement

### 8.3 Expiry Enforcement

- Expired slices are automatically resolved
- Background task checks expiry every minute
- Grace period allows for network delays

---

## 9. References

- [RHIZOCRYPT_SPECIFICATION.md](./RHIZOCRYPT_SPECIFICATION.md) — Full specification
- [DATA_MODEL.md](./DATA_MODEL.md) — Core data structures
- [DEHYDRATION_PROTOCOL.md](./DEHYDRATION_PROTOCOL.md) — Commit protocol
- [LoamSpine Specification](../../loamSpine/specs/) — Permanent layer

---

*RhizoCrypt: The memory that knows when to forget.*

