// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2024–2026 ecoPrimals Project

//! Event types for the RhizoCrypt DAG.
//!
//! Events are the domain-specific actions that are recorded in the DAG.

use crate::types::{Did, SliceId};
use serde::{Deserialize, Serialize};

/// Event type identifier.
///
/// Defines the type of action that occurred in the DAG.
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
    /// Slice checked out from LoamSpine.
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

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;

    #[test]
    fn test_event_type_domain() {
        assert_eq!(EventType::SessionStart.domain(), "session");
        assert_eq!(
            EventType::AgentJoin {
                role: AgentRole::Participant
            }
            .domain(),
            "agent"
        );
        assert_eq!(
            EventType::DataCreate {
                schema: None
            }
            .domain(),
            "data"
        );
        assert_eq!(
            EventType::ItemLoot {
                item_type: "weapon".into()
            }
            .domain(),
            "gaming"
        );
        assert_eq!(
            EventType::ExperimentStart {
                protocol: "test".into()
            }
            .domain(),
            "science"
        );
        assert_eq!(
            EventType::Custom {
                domain: "custom".into(),
                event_name: "test".into()
            }
            .domain(),
            "custom"
        );
    }

    #[test]
    fn test_event_type_name() {
        assert_eq!(EventType::SessionStart.name(), "session_start");
        assert_eq!(
            EventType::SessionEnd {
                outcome: SessionOutcome::Success
            }
            .name(),
            "session_end"
        );
    }

    #[test]
    fn test_session_outcome_serialization() {
        let outcome = SessionOutcome::Failure {
            reason: "test error".into(),
        };
        let json = serde_json::to_string(&outcome).unwrap();
        let parsed: SessionOutcome = serde_json::from_str(&json).unwrap();
        assert_eq!(outcome, parsed);
    }
}
