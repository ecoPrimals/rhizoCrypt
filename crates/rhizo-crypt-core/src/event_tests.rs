// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2024–2026 ecoPrimals Project

//! Tests for `EventType`, supporting enums, and cross-gate mesh events.

use super::*;

/// Collect all `EventType` variants for exhaustive testing.
#[allow(
    clippy::too_many_lines,
    reason = "exhaustive variant list requires enumerating all 32 EventType variants"
)]
fn all_event_types() -> Vec<EventType> {
    vec![
        EventType::SessionStart,
        EventType::SessionEnd {
            outcome: SessionOutcome::Success,
        },
        EventType::AgentJoin {
            role: AgentRole::Participant,
        },
        EventType::AgentLeave {
            reason: LeaveReason::Normal,
        },
        EventType::AgentAction {
            action: "test".into(),
        },
        EventType::DataCreate {
            schema: None,
        },
        EventType::DataModify {
            delta_type: "patch".into(),
        },
        EventType::DataDelete,
        EventType::DataTransfer {
            to: Did::new("did:key:recipient"),
        },
        EventType::SliceCheckout {
            slice_id: SliceId::now(),
            mode: SliceMode::Copy {
                allow_recopy: false,
            },
        },
        EventType::SliceOperation {
            slice_id: SliceId::now(),
            operation: "read".into(),
        },
        EventType::SliceResolve {
            slice_id: SliceId::now(),
            resolution: ResolutionType::ReturnToOrigin,
        },
        EventType::GameEvent {
            game_type: "rpg".into(),
            event_name: "level_up".into(),
        },
        EventType::ItemLoot {
            item_type: "weapon".into(),
        },
        EventType::ItemDrop,
        EventType::ItemTransfer {
            to: Did::new("did:key:recipient"),
        },
        EventType::Combat {
            target: Did::new("did:key:target"),
            outcome: "win".into(),
        },
        EventType::Extraction {
            success: true,
        },
        EventType::ExperimentStart {
            protocol: "test".into(),
        },
        EventType::Observation {
            instrument: "microscope".into(),
        },
        EventType::Analysis {
            method: "pca".into(),
        },
        EventType::Result {
            confidence_percent: 95,
        },
        EventType::DocumentEdit {
            operation: "insert".into(),
        },
        EventType::CommentAdd,
        EventType::ApprovalGrant,
        EventType::ApprovalRevoke,
        EventType::TrustIssuerRegistered {
            issuer_fingerprint: "a1b2c3d4e5f6".into(),
            registering_gate: "eastGate".into(),
        },
        EventType::KeyExchangeCompleted {
            local_gate: "strandGate".into(),
            remote_gate: "southGate".into(),
            method: "ed25519_dh".into(),
        },
        EventType::FamilyEnrollment {
            family_id: "ecoPrimal".into(),
            gate: "strandGate".into(),
            primal_count: 3,
        },
        EventType::MeshJoin {
            gate: "strandGate".into(),
            mesh_id: "mesh-001".into(),
        },
        EventType::MeshLeave {
            gate: "strandGate".into(),
            mesh_id: "mesh-001".into(),
            reason: MeshLeaveReason::Graceful,
        },
        EventType::Custom {
            domain: "custom".into(),
            event_name: "custom_event".into(),
        },
    ]
}

#[test]
fn test_all_event_type_names() {
    for event in all_event_types() {
        let name = event.name();
        assert!(!name.is_empty(), "variant {event:?} has empty name");
    }
}

#[test]
fn test_all_event_type_domains() {
    for event in all_event_types() {
        let domain = event.domain();
        assert!(!domain.is_empty(), "variant {event:?} has empty domain");
    }
}

#[test]
fn test_event_type_serialization_roundtrip() {
    for event in all_event_types() {
        let json = serde_json::to_string(&event).unwrap();
        let parsed: EventType = serde_json::from_str(&json).unwrap();
        assert_eq!(event, parsed, "roundtrip failed for {event:?}");
    }
}

#[test]
fn test_session_outcome_all_variants() {
    let variants = vec![
        SessionOutcome::Success,
        SessionOutcome::Failure {
            reason: "error".into(),
        },
        SessionOutcome::Timeout,
        SessionOutcome::Cancelled,
        SessionOutcome::Rollback,
    ];
    for outcome in variants {
        let json = serde_json::to_string(&outcome).unwrap();
        let parsed: SessionOutcome = serde_json::from_str(&json).unwrap();
        assert_eq!(outcome, parsed);
    }
}

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

// === Cross-Gate Mesh Event Tests ===

#[test]
fn test_mesh_event_types_domain() {
    let mesh_events = [
        EventType::TrustIssuerRegistered {
            issuer_fingerprint: "deadbeef".into(),
            registering_gate: "eastGate".into(),
        },
        EventType::KeyExchangeCompleted {
            local_gate: "strandGate".into(),
            remote_gate: "southGate".into(),
            method: "ed25519_dh".into(),
        },
        EventType::FamilyEnrollment {
            family_id: "ecoPrimal".into(),
            gate: "strandGate".into(),
            primal_count: 5,
        },
        EventType::MeshJoin {
            gate: "biomeGate".into(),
            mesh_id: "mesh-lan-001".into(),
        },
        EventType::MeshLeave {
            gate: "biomeGate".into(),
            mesh_id: "mesh-lan-001".into(),
            reason: MeshLeaveReason::Disconnected,
        },
    ];
    for event in &mesh_events {
        assert_eq!(event.domain(), "mesh", "mesh event {event:?} has wrong domain");
    }
}

#[test]
fn test_mesh_event_types_names() {
    assert_eq!(
        EventType::TrustIssuerRegistered {
            issuer_fingerprint: "ab".into(),
            registering_gate: "g".into()
        }
        .name(),
        "trust_issuer_registered"
    );
    assert_eq!(
        EventType::KeyExchangeCompleted {
            local_gate: "a".into(),
            remote_gate: "b".into(),
            method: "ed25519_dh".into()
        }
        .name(),
        "key_exchange_completed"
    );
    assert_eq!(
        EventType::FamilyEnrollment {
            family_id: "f".into(),
            gate: "g".into(),
            primal_count: 1
        }
        .name(),
        "family_enrollment"
    );
    assert_eq!(
        EventType::MeshJoin {
            gate: "g".into(),
            mesh_id: "m".into()
        }
        .name(),
        "mesh_join"
    );
    assert_eq!(
        EventType::MeshLeave {
            gate: "g".into(),
            mesh_id: "m".into(),
            reason: MeshLeaveReason::Graceful
        }
        .name(),
        "mesh_leave"
    );
}

#[test]
fn test_mesh_event_wire_format_trust_issuer() {
    let event = EventType::TrustIssuerRegistered {
        issuer_fingerprint: "a1b2c3d4e5f67890".into(),
        registering_gate: "eastGate".into(),
    };
    let json = serde_json::to_string(&event).unwrap();
    assert!(json.contains("TrustIssuerRegistered"));
    assert!(json.contains("a1b2c3d4e5f67890"));
    assert!(json.contains("eastGate"));
    let parsed: EventType = serde_json::from_str(&json).unwrap();
    assert_eq!(event, parsed);
}

#[test]
fn test_mesh_event_wire_format_key_exchange() {
    let event = EventType::KeyExchangeCompleted {
        local_gate: "strandGate".into(),
        remote_gate: "southGate".into(),
        method: "x25519".into(),
    };
    let json = serde_json::to_string(&event).unwrap();
    let parsed: EventType = serde_json::from_str(&json).unwrap();
    assert_eq!(event, parsed);

    let value: serde_json::Value = serde_json::from_str(&json).unwrap();
    let inner = value.get("KeyExchangeCompleted").unwrap();
    assert_eq!(inner["local_gate"], "strandGate");
    assert_eq!(inner["remote_gate"], "southGate");
    assert_eq!(inner["method"], "x25519");
}

#[test]
fn test_mesh_event_wire_format_family_enrollment() {
    let event = EventType::FamilyEnrollment {
        family_id: "ecoPrimal".into(),
        gate: "strandGate".into(),
        primal_count: 10,
    };
    let json = serde_json::to_string(&event).unwrap();
    let parsed: EventType = serde_json::from_str(&json).unwrap();
    assert_eq!(event, parsed);

    let value: serde_json::Value = serde_json::from_str(&json).unwrap();
    let inner = value.get("FamilyEnrollment").unwrap();
    assert_eq!(inner["primal_count"], 10);
}

#[test]
fn test_mesh_event_wire_format_mesh_join_leave() {
    let join = EventType::MeshJoin {
        gate: "ironGate".into(),
        mesh_id: "glacial-mesh-v1".into(),
    };
    let json = serde_json::to_string(&join).unwrap();
    let parsed: EventType = serde_json::from_str(&json).unwrap();
    assert_eq!(join, parsed);

    for reason in [
        MeshLeaveReason::Graceful,
        MeshLeaveReason::Disconnected,
        MeshLeaveReason::Evicted,
        MeshLeaveReason::TrustRevoked,
    ] {
        let leave = EventType::MeshLeave {
            gate: "ironGate".into(),
            mesh_id: "glacial-mesh-v1".into(),
            reason: reason.clone(),
        };
        let json = serde_json::to_string(&leave).unwrap();
        let parsed: EventType = serde_json::from_str(&json).unwrap();
        assert_eq!(leave, parsed, "roundtrip failed for MeshLeave({reason:?})");
    }
}

#[test]
fn test_mesh_leave_reason_all_variants() {
    let variants = [
        MeshLeaveReason::Graceful,
        MeshLeaveReason::Disconnected,
        MeshLeaveReason::Evicted,
        MeshLeaveReason::TrustRevoked,
    ];
    for reason in variants {
        let json = serde_json::to_string(&reason).unwrap();
        let parsed: MeshLeaveReason = serde_json::from_str(&json).unwrap();
        assert_eq!(reason, parsed);
    }
}
