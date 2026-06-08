// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2024–2026 ecoPrimals Project

//! Event types for the `RhizoCrypt` DAG.
//!
//! Events are the domain-specific actions recorded in the DAG via
//! `dag.event.append`. The [`EventType`] enum defines 32 variants across
//! 8 domains (session, agent, data, slice, gaming, science, collaboration,
//! mesh) plus a freeform [`Custom`](EventType::Custom) variant for domain
//! springs.
//!
//! ## Wire Format (JSON-RPC)
//!
//! Uses serde's default externally-tagged representation:
//! - Variants with fields: `{"VariantName": {"field": value}}`
//! - Unit variants (no fields): `"VariantName"`
//!
//! ## Canonical Reference
//!
//! See `specs/EVENT_TYPE_REFERENCE.md` for the complete variant list with
//! JSON wire examples, domain mapping, and guidance for domain springs.

use crate::types::{Did, SliceId};
use serde::{Deserialize, Serialize};

/// Event type identifier for `dag.event.append`.
///
/// 32 variants across 8 domains. Uses serde externally-tagged JSON:
/// `{"DataCreate": {"schema": "v2"}}` for variants with fields,
/// `"DataDelete"` for unit variants.
///
/// Domain springs should prefer built-in variants where they fit and use
/// [`Custom`](Self::Custom) for domain-specific events, placing rich
/// context in the request's `metadata` key-value pairs.
///
/// See `specs/EVENT_TYPE_REFERENCE.md` for the complete wire format guide.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub enum EventType {
    // === Session Lifecycle ===
    /// Session started.
    SessionStart,

    /// Session ended.
    SessionEnd {
        /// Outcome of the session.
        outcome: SessionOutcome,
    },

    // === Agent Events ===
    /// Agent joined the session.
    AgentJoin {
        /// Role of the agent.
        role: AgentRole,
    },

    /// Agent left the session.
    AgentLeave {
        /// Reason for leaving.
        reason: LeaveReason,
    },

    /// Agent performed an action.
    AgentAction {
        /// Action description.
        action: String,
    },

    // === Data Events ===
    /// Data created.
    DataCreate {
        /// Optional schema reference.
        schema: Option<String>,
    },

    /// Data modified.
    DataModify {
        /// Type of modification.
        delta_type: String,
    },

    /// Data deleted.
    DataDelete,

    /// Data transferred.
    DataTransfer {
        /// Recipient DID.
        to: Did,
    },

    // === Slice Events ===
    /// Slice checked out from permanent storage.
    SliceCheckout {
        /// Slice identifier.
        slice_id: SliceId,
        /// Slice mode.
        mode: SliceMode,
    },

    /// Operation performed on slice.
    SliceOperation {
        /// Slice identifier.
        slice_id: SliceId,
        /// Operation description.
        operation: String,
    },

    /// Slice resolved.
    SliceResolve {
        /// Slice identifier.
        slice_id: SliceId,
        /// Resolution type.
        resolution: ResolutionType,
    },

    // === Gaming Domain ===
    /// Generic game event.
    GameEvent {
        /// Game type identifier.
        game_type: String,
        /// Event name.
        event_name: String,
    },

    /// Item looted.
    ItemLoot {
        /// Item type.
        item_type: String,
    },

    /// Item dropped.
    ItemDrop,

    /// Item transferred.
    ItemTransfer {
        /// Recipient DID.
        to: Did,
    },

    /// Combat event.
    Combat {
        /// Target DID.
        target: Did,
        /// Combat outcome.
        outcome: String,
    },

    /// Extraction event.
    Extraction {
        /// Whether extraction was successful.
        success: bool,
    },

    // === Scientific Domain ===
    /// Experiment started.
    ExperimentStart {
        /// Protocol identifier.
        protocol: String,
    },

    /// Observation recorded.
    Observation {
        /// Instrument used.
        instrument: String,
    },

    /// Analysis performed.
    Analysis {
        /// Analysis method.
        method: String,
    },

    /// Result recorded.
    Result {
        /// Confidence level as percentage (0 to 100).
        confidence_percent: u8,
    },

    // === Collaboration Domain ===
    /// Document edited.
    DocumentEdit {
        /// Edit operation type.
        operation: String,
    },

    /// Comment added.
    CommentAdd,

    /// Approval granted.
    ApprovalGrant,

    /// Approval revoked.
    ApprovalRevoke,

    // === Mesh (Cross-Gate Trust) ===
    /// A trusted issuer was registered in the gate's issuer registry.
    ///
    /// Records the Ed25519 public key fingerprint and the gate that
    /// enrolled the issuer. Wire: bearDog w135 `TrustedIssuerRegistry`.
    TrustIssuerRegistered {
        /// Ed25519 public key fingerprint (hex-encoded).
        issuer_fingerprint: String,
        /// Gate that registered the issuer.
        registering_gate: String,
    },

    /// An Ed25519 key exchange was completed between two gates.
    ///
    /// Records both gate identifiers and the key exchange method.
    /// Wire: bearDog w135 cross-gate key exchange protocol.
    KeyExchangeCompleted {
        /// Local gate identifier.
        local_gate: String,
        /// Remote gate identifier.
        remote_gate: String,
        /// Key exchange method (e.g. `ed25519_dh`, `x25519`).
        method: String,
    },

    /// A primal family enrolled in the mesh.
    ///
    /// Records the family identifier, the gate it joined through,
    /// and the number of primals in the family.
    FamilyEnrollment {
        /// Family identifier (`ECOPRIMALS_FAMILY_ID`).
        family_id: String,
        /// Gate the family enrolled through.
        gate: String,
        /// Number of primals in the family at enrollment time.
        primal_count: u32,
    },

    /// A gate joined the mesh network.
    MeshJoin {
        /// Gate identifier.
        gate: String,
        /// Mesh network identifier.
        mesh_id: String,
    },

    /// A gate left the mesh network.
    MeshLeave {
        /// Gate identifier.
        gate: String,
        /// Mesh network identifier.
        mesh_id: String,
        /// Reason for leaving.
        reason: MeshLeaveReason,
    },

    // === Custom ===
    /// Custom event type.
    Custom {
        /// Domain name.
        domain: String,
        /// Event name.
        event_name: String,
    },
}

impl EventType {
    /// Get the domain for this event type.
    #[must_use]
    pub fn domain(&self) -> &str {
        match self {
            Self::SessionStart
            | Self::SessionEnd {
                ..
            } => "session",
            Self::AgentJoin {
                ..
            }
            | Self::AgentLeave {
                ..
            }
            | Self::AgentAction {
                ..
            } => "agent",
            Self::DataCreate {
                ..
            }
            | Self::DataModify {
                ..
            }
            | Self::DataDelete
            | Self::DataTransfer {
                ..
            } => "data",
            Self::SliceCheckout {
                ..
            }
            | Self::SliceOperation {
                ..
            }
            | Self::SliceResolve {
                ..
            } => "slice",
            Self::GameEvent {
                ..
            }
            | Self::ItemLoot {
                ..
            }
            | Self::ItemDrop
            | Self::ItemTransfer {
                ..
            }
            | Self::Combat {
                ..
            }
            | Self::Extraction {
                ..
            } => "gaming",
            Self::ExperimentStart {
                ..
            }
            | Self::Observation {
                ..
            }
            | Self::Analysis {
                ..
            }
            | Self::Result {
                ..
            } => "science",
            Self::DocumentEdit {
                ..
            }
            | Self::CommentAdd
            | Self::ApprovalGrant
            | Self::ApprovalRevoke => "collaboration",
            Self::TrustIssuerRegistered {
                ..
            }
            | Self::KeyExchangeCompleted {
                ..
            }
            | Self::FamilyEnrollment {
                ..
            }
            | Self::MeshJoin {
                ..
            }
            | Self::MeshLeave {
                ..
            } => "mesh",
            Self::Custom {
                domain,
                ..
            } => domain,
        }
    }

    /// Get the event name.
    #[must_use]
    pub fn name(&self) -> &str {
        match self {
            Self::SessionStart => "session_start",
            Self::SessionEnd {
                ..
            } => "session_end",
            Self::AgentJoin {
                ..
            } => "agent_join",
            Self::AgentLeave {
                ..
            } => "agent_leave",
            Self::AgentAction {
                ..
            } => "agent_action",
            Self::DataCreate {
                ..
            } => "data_create",
            Self::DataModify {
                ..
            } => "data_modify",
            Self::DataDelete => "data_delete",
            Self::DataTransfer {
                ..
            } => "data_transfer",
            Self::SliceCheckout {
                ..
            } => "slice_checkout",
            Self::SliceOperation {
                ..
            } => "slice_operation",
            Self::SliceResolve {
                ..
            } => "slice_resolve",
            Self::GameEvent {
                event_name,
                ..
            }
            | Self::Custom {
                event_name,
                ..
            } => event_name,
            Self::ItemLoot {
                ..
            } => "item_loot",
            Self::ItemDrop => "item_drop",
            Self::ItemTransfer {
                ..
            } => "item_transfer",
            Self::Combat {
                ..
            } => "combat",
            Self::Extraction {
                ..
            } => "extraction",
            Self::ExperimentStart {
                ..
            } => "experiment_start",
            Self::Observation {
                ..
            } => "observation",
            Self::Analysis {
                ..
            } => "analysis",
            Self::Result {
                ..
            } => "result",
            Self::DocumentEdit {
                ..
            } => "document_edit",
            Self::CommentAdd => "comment_add",
            Self::ApprovalGrant => "approval_grant",
            Self::ApprovalRevoke => "approval_revoke",
            Self::TrustIssuerRegistered {
                ..
            } => "trust_issuer_registered",
            Self::KeyExchangeCompleted {
                ..
            } => "key_exchange_completed",
            Self::FamilyEnrollment {
                ..
            } => "family_enrollment",
            Self::MeshJoin {
                ..
            } => "mesh_join",
            Self::MeshLeave {
                ..
            } => "mesh_leave",
        }
    }
}

/// Session outcome.
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SessionOutcome {
    /// Session completed successfully.
    Success,
    /// Session failed.
    Failure {
        /// Failure reason.
        reason: String,
    },
    /// Session timed out.
    Timeout,
    /// Session was cancelled.
    Cancelled,
    /// Session rolled back.
    Rollback,
}

/// Agent role in a session.
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AgentRole {
    /// Session owner/creator.
    Owner,
    /// Regular participant.
    Participant,
    /// Observer (read-only).
    Observer,
    /// Custom role.
    Custom(String),
}

/// Reason for leaving a session.
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum LeaveReason {
    /// Normal departure.
    Normal,
    /// Kicked by owner.
    Kicked,
    /// Disconnected.
    Disconnected,
    /// Timed out.
    Timeout,
}

/// Slice mode.
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SliceMode {
    /// Copy mode - local use only.
    Copy {
        /// Whether the copy can be re-copied.
        allow_recopy: bool,
    },
    /// Loan mode - temporary use.
    Loan {
        /// Allow sub-loaning.
        allow_subloan: bool,
    },
    /// Consignment mode - held by third party.
    Consignment {
        /// Consignee DID.
        consignee: Did,
    },
    /// Escrow mode - held pending agreement.
    Escrow {
        /// Required confirmations.
        required_confirmations: u32,
    },
    /// Waypoint mode - anchors to local spine.
    Waypoint {
        /// Waypoint spine ID.
        waypoint_spine: String,
    },
    /// Transfer mode - ownership transfer.
    Transfer {
        /// New owner DID.
        new_owner: Did,
    },
}

/// Resolution type for slices.
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ResolutionType {
    /// Return to origin unchanged.
    ReturnToOrigin,
    /// Commit new state to origin.
    CommitToOrigin,
    /// Route to different spine.
    RouteToSpine {
        /// Target spine ID.
        target_spine: String,
    },
    /// Consumed (deleted).
    Consumed,
}

/// Reason a gate left the mesh.
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MeshLeaveReason {
    /// Graceful shutdown.
    Graceful,
    /// Lost connectivity.
    Disconnected,
    /// Evicted by mesh consensus.
    Evicted,
    /// Trust revoked (key compromised or issuer deregistered).
    TrustRevoked,
}

#[cfg(test)]
#[expect(clippy::unwrap_used, reason = "test code")]
#[path = "event_tests.rs"]
mod tests;
