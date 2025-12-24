//! Slice semantics for LoamSpine state checkout.
//!
//! A slice is a reference to LoamSpine state that is temporarily "lifted" into
//! a RhizoCrypt DAG for asynchronous operations. The slice carries information
//! about how it should resolve back to permanence.

use crate::types::{ContentHash, Did, SessionId, SliceId, Timestamp, VertexId};
use serde::{Deserialize, Serialize};
use std::time::Duration;

/// A slice of LoamSpine state checked out into RhizoCrypt.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Slice {
    /// Unique slice identifier.
    pub id: SliceId,

    /// Origin in LoamSpine.
    pub origin: SliceOrigin,

    /// Current holder of the slice.
    pub holder: Did,

    /// Original owner (always the origin owner).
    pub owner: Did,

    /// Slice mode (determines behavior).
    pub mode: SliceMode,

    /// Resolution routing.
    pub resolution_route: ResolutionRoute,

    /// When the slice was checked out.
    pub checked_out_at: Timestamp,

    /// When the slice expires (if applicable).
    pub expires_at: Option<Timestamp>,

    /// Slice constraints.
    pub constraints: SliceConstraints,

    /// Current slice state.
    pub state: SliceState,

    /// Session the slice is active in.
    pub session_id: SessionId,

    /// Vertex that checked out the slice.
    pub checkout_vertex: VertexId,
}

impl Slice {
    /// Check if the slice has expired.
    #[must_use]
    pub fn is_expired(&self) -> bool {
        self.expires_at
            .is_some_and(|expires_at| Timestamp::now().as_nanos() > expires_at.as_nanos())
    }

    /// Check if the slice is still active.
    #[must_use]
    pub const fn is_active(&self) -> bool {
        matches!(self.state, SliceState::Active)
    }

    /// Check if the slice is resolved.
    #[must_use]
    pub const fn is_resolved(&self) -> bool {
        matches!(self.state, SliceState::Resolved { .. })
    }

    /// Check if the slice can be re-sliced.
    #[must_use]
    pub const fn can_reslice(&self) -> bool {
        self.constraints.allow_reslice
    }
}

/// Origin of a slice in LoamSpine.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SliceOrigin {
    /// Source spine identifier.
    pub spine_id: String,

    /// Entry hash in the spine.
    pub entry_hash: ContentHash,

    /// Entry index in the spine.
    pub entry_index: u64,

    /// Certificate ID (if slice is of a certificate).
    pub certificate_id: Option<String>,

    /// Owner of the entry.
    pub owner: Did,
}

/// Slice mode determines the behavior and resolution semantics.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum SliceMode {
    /// Copy mode - local use only, cannot lineage back up.
    ///
    /// Use case: Give a friend a game to play locally, no network effects.
    Copy {
        /// Whether the copy can be further copied.
        allow_recopy: bool,
    },

    /// Loan mode - borrower has use rights, auto-returns on expiry/condition.
    ///
    /// Use case: Lend game to friend for weekend.
    Loan {
        /// Loan terms.
        terms: LoanTerms,
        /// Whether borrower can sub-loan.
        allow_subloan: bool,
    },

    /// Consignment mode - temporary possession, ownership never transfers.
    ///
    /// Use case: Auction house holds item until sale, then routes to buyer.
    Consignment {
        /// The consignee (temporary holder).
        consignee: Did,
        /// Conditions that trigger resolution.
        resolution_triggers: Vec<ResolutionTrigger>,
    },

    /// Escrow mode - held pending multi-party agreement.
    ///
    /// Use case: Trade between players, both items held until both confirm.
    Escrow {
        /// Parties involved.
        parties: Vec<Did>,
        /// Required confirmations for release.
        required_confirmations: u32,
        /// Current confirmations.
        confirmations: Vec<Did>,
    },

    /// Waypoint mode - anchors to holder's local spine, then returns.
    ///
    /// Use case: Friend's local spine tracks their use, then resolves back.
    Waypoint {
        /// The waypoint spine.
        waypoint_spine: String,
    },

    /// Transfer mode - full ownership transfer on resolution.
    ///
    /// Use case: Sale, gift, permanent transfer.
    Transfer {
        /// New owner on resolution.
        new_owner: Did,
    },
}

impl SliceMode {
    /// Check if this slice can lineage back to origin.
    #[must_use]
    pub const fn can_lineage_back(&self) -> bool {
        !matches!(self, Self::Copy { .. })
    }

    /// Get a descriptive name for the mode.
    #[must_use]
    pub const fn name(&self) -> &'static str {
        match self {
            Self::Copy {
                ..
            } => "copy",
            Self::Loan {
                ..
            } => "loan",
            Self::Consignment {
                ..
            } => "consignment",
            Self::Escrow {
                ..
            } => "escrow",
            Self::Waypoint {
                ..
            } => "waypoint",
            Self::Transfer {
                ..
            } => "transfer",
        }
    }
}

/// Loan terms for loan mode slices.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct LoanTerms {
    /// Maximum loan duration.
    pub max_duration: Option<Duration>,

    /// Grace period after expiry before auto-return.
    pub grace_period: Option<Duration>,

    /// Whether to automatically return on expiry.
    pub auto_return: bool,
}

impl Default for LoanTerms {
    fn default() -> Self {
        Self {
            max_duration: Some(Duration::from_secs(7 * 24 * 3600)), // 1 week
            grace_period: Some(Duration::from_secs(24 * 3600)),     // 1 day
            auto_return: true,
        }
    }
}

/// Resolution route determines where the slice goes on resolution.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ResolutionRoute {
    /// Return to origin spine unchanged.
    ReturnToOrigin,

    /// Commit new state to origin spine.
    CommitToOrigin {
        /// Include summary in commit.
        include_summary: bool,
    },

    /// Route to a different spine (for transfers).
    RouteToSpine {
        /// Target spine ID.
        target_spine: String,
    },

    /// Route through waypoint, then back to origin.
    WaypointReturn {
        /// Waypoint spine.
        waypoint_spine: String,
    },

    /// Conditional routing based on resolution outcome.
    Conditional {
        /// Conditions and their routes.
        conditions: Vec<ConditionalRouteEntry>,
        /// Default route if no conditions match.
        default: Box<ResolutionRoute>,
    },
}

/// A conditional route entry.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConditionalRouteEntry {
    /// Condition to evaluate.
    pub condition: ResolutionCondition,
    /// Route to take if condition is met.
    pub route: ResolutionRoute,
}

/// Conditions for conditional routing.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ResolutionCondition {
    /// DAG resolved with success outcome.
    SessionSuccess,
    /// DAG resolved with rollback.
    SessionRollback,
    /// Session timed out.
    SessionTimeout,
    /// Specific event occurred in DAG.
    EventOccurred {
        /// Event type to match.
        event_type: String,
    },
    /// External trigger received.
    ExternalTrigger {
        /// Trigger identifier.
        trigger_id: String,
    },
    /// All parties confirmed (for escrow).
    AllPartiesConfirmed,
    /// Loan expired.
    LoanExpired,
    /// Owner recalled the slice.
    OwnerRecall,
}

/// Trigger for resolution.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ResolutionTrigger {
    /// Specific event type occurred.
    EventType(String),
    /// Timeout elapsed.
    Timeout(Duration),
    /// External trigger.
    External(String),
}

/// Constraints on slice operations.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct SliceConstraints {
    /// Maximum duration the slice can exist.
    pub max_duration: Option<Duration>,

    /// Whether slice can be re-sliced (sub-lending).
    pub allow_reslice: bool,

    /// Maximum depth of re-slicing.
    pub max_reslice_depth: Option<u32>,

    /// Forbidden operations.
    pub forbidden_operations: Vec<String>,
}

/// Slice lifecycle state.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum SliceState {
    /// Slice is active in a session.
    Active,

    /// Slice is anchored at a waypoint.
    Anchored {
        /// Waypoint spine.
        waypoint_spine: String,
        /// Anchor entry hash.
        anchor_entry: ContentHash,
    },

    /// Slice is resolving.
    Resolving {
        /// When resolution started.
        started_at: Timestamp,
    },

    /// Slice has been resolved.
    Resolved {
        /// Resolution outcome.
        outcome: ResolutionOutcome,
        /// When resolved.
        resolved_at: Timestamp,
    },
}

/// Outcome of slice resolution.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ResolutionOutcome {
    /// Returned to origin unchanged.
    ReturnedUnchanged,

    /// Committed new state to origin.
    Committed {
        /// New entry hash.
        new_entry: ContentHash,
    },

    /// Transferred to new spine/owner.
    Transferred {
        /// New spine ID.
        new_spine: String,
        /// New entry hash.
        new_entry: ContentHash,
        /// New owner.
        new_owner: Did,
    },

    /// Anchored at waypoint.
    Anchored {
        /// Waypoint spine.
        waypoint_spine: String,
        /// Waypoint entry.
        waypoint_entry: ContentHash,
    },

    /// Consumed (deleted).
    Consumed,
}

/// Builder for creating slices.
#[derive(Clone, Debug)]
pub struct SliceBuilder {
    origin: SliceOrigin,
    holder: Did,
    mode: SliceMode,
    resolution_route: ResolutionRoute,
    constraints: SliceConstraints,
    session_id: SessionId,
    checkout_vertex: VertexId,
    expires_at: Option<Timestamp>,
}

impl SliceBuilder {
    /// Create a new slice builder.
    #[must_use]
    pub fn new(
        origin: SliceOrigin,
        holder: Did,
        mode: SliceMode,
        session_id: SessionId,
        checkout_vertex: VertexId,
    ) -> Self {
        Self {
            origin,
            holder,
            mode,
            resolution_route: ResolutionRoute::ReturnToOrigin,
            constraints: SliceConstraints::default(),
            session_id,
            checkout_vertex,
            expires_at: None,
        }
    }

    /// Set the resolution route.
    #[must_use]
    pub fn with_resolution_route(mut self, route: ResolutionRoute) -> Self {
        self.resolution_route = route;
        self
    }

    /// Set the constraints.
    #[must_use]
    pub fn with_constraints(mut self, constraints: SliceConstraints) -> Self {
        self.constraints = constraints;
        self
    }

    /// Set the expiration time.
    #[must_use]
    pub const fn with_expires_at(mut self, expires_at: Timestamp) -> Self {
        self.expires_at = Some(expires_at);
        self
    }

    /// Set expiration as duration from now.
    #[must_use]
    #[allow(clippy::cast_possible_truncation)]
    pub fn expires_in(mut self, duration: Duration) -> Self {
        let now = Timestamp::now();
        let expires = Timestamp::from_nanos(now.as_nanos() + duration.as_nanos() as u64);
        self.expires_at = Some(expires);
        self
    }

    /// Build the slice.
    #[must_use]
    pub fn build(self) -> Slice {
        let now = Timestamp::now();
        Slice {
            id: SliceId::now(),
            origin: self.origin.clone(),
            holder: self.holder,
            owner: self.origin.owner,
            mode: self.mode,
            resolution_route: self.resolution_route,
            checked_out_at: now,
            expires_at: self.expires_at,
            constraints: self.constraints,
            state: SliceState::Active,
            session_id: self.session_id,
            checkout_vertex: self.checkout_vertex,
        }
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;

    fn make_origin() -> SliceOrigin {
        SliceOrigin {
            spine_id: "spine-123".to_string(),
            entry_hash: [1u8; 32],
            entry_index: 42,
            certificate_id: None,
            owner: Did::new("did:key:owner"),
        }
    }

    #[test]
    fn test_slice_builder() {
        let origin = make_origin();
        let holder = Did::new("did:key:holder");
        let session_id = SessionId::now();
        let checkout_vertex = VertexId::from_bytes(b"checkout");

        let slice = SliceBuilder::new(
            origin,
            holder.clone(),
            SliceMode::Loan {
                terms: LoanTerms::default(),
                allow_subloan: false,
            },
            session_id,
            checkout_vertex,
        )
        .with_resolution_route(ResolutionRoute::ReturnToOrigin)
        .build();

        assert!(slice.is_active());
        assert!(!slice.is_resolved());
        assert!(!slice.is_expired());
        assert_eq!(slice.holder, holder);
        assert_eq!(slice.origin.spine_id, "spine-123");
    }

    #[test]
    fn test_slice_mode_lineage() {
        assert!(!SliceMode::Copy {
            allow_recopy: false
        }
        .can_lineage_back());
        assert!(SliceMode::Loan {
            terms: LoanTerms::default(),
            allow_subloan: false
        }
        .can_lineage_back());
        assert!(SliceMode::Transfer {
            new_owner: Did::new("did:key:new")
        }
        .can_lineage_back());
    }

    #[test]
    fn test_slice_mode_names() {
        assert_eq!(
            SliceMode::Copy {
                allow_recopy: true
            }
            .name(),
            "copy"
        );
        assert_eq!(
            SliceMode::Escrow {
                parties: vec![],
                required_confirmations: 2,
                confirmations: vec![]
            }
            .name(),
            "escrow"
        );
    }

    #[test]
    fn test_resolution_route() {
        let route = ResolutionRoute::Conditional {
            conditions: vec![ConditionalRouteEntry {
                condition: ResolutionCondition::SessionSuccess,
                route: ResolutionRoute::CommitToOrigin {
                    include_summary: true,
                },
            }],
            default: Box::new(ResolutionRoute::ReturnToOrigin),
        };

        if let ResolutionRoute::Conditional {
            conditions,
            ..
        } = &route
        {
            assert_eq!(conditions.len(), 1);
        } else {
            panic!("Expected Conditional route");
        }
    }

    #[test]
    fn test_loan_terms_default() {
        let terms = LoanTerms::default();
        assert!(terms.auto_return);
        assert!(terms.max_duration.is_some());
        assert!(terms.grace_period.is_some());
    }

    #[test]
    fn test_slice_constraints_default() {
        let constraints = SliceConstraints::default();
        assert!(!constraints.allow_reslice);
        assert!(constraints.forbidden_operations.is_empty());
    }
}
