// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2024–2026 ecoPrimals Project

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
    assert!(
        !SliceMode::Copy {
            allow_recopy: false
        }
        .can_lineage_back()
    );
    assert!(
        SliceMode::Loan {
            terms: LoanTerms::default(),
            allow_subloan: false
        }
        .can_lineage_back()
    );
    assert!(
        SliceMode::Transfer {
            new_owner: Did::new("did:key:new")
        }
        .can_lineage_back()
    );
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

#[test]
fn test_slice_state_transitions() {
    let origin = make_origin();
    let holder = Did::new("did:key:holder");
    let session_id = SessionId::now();
    let checkout_vertex = VertexId::from_bytes(b"checkout");

    let mut slice = SliceBuilder::new(
        origin,
        holder,
        SliceMode::Transfer {
            new_owner: Did::new("did:key:new"),
        },
        session_id,
        checkout_vertex,
    )
    .build();

    assert!(slice.is_active());
    assert!(!slice.is_resolved());

    slice.state = SliceState::Resolving {
        started_at: Timestamp::now(),
    };
    assert!(!slice.is_active());
    assert!(!slice.is_resolved());

    slice.state = SliceState::Resolved {
        outcome: ResolutionOutcome::ReturnedUnchanged,
        resolved_at: Timestamp::now(),
    };
    assert!(!slice.is_active());
    assert!(slice.is_resolved());
}

#[test]
fn test_slice_is_active() {
    let origin = make_origin();
    let holder = Did::new("did:key:holder");
    let session_id = SessionId::now();
    let checkout_vertex = VertexId::from_bytes(b"checkout");

    let slice = SliceBuilder::new(
        origin,
        holder,
        SliceMode::Copy {
            allow_recopy: false,
        },
        session_id,
        checkout_vertex,
    )
    .build();

    assert!(slice.is_active());
    assert!(!slice.is_resolved());
}

#[test]
fn test_slice_with_duration() {
    let origin = make_origin();
    let holder = Did::new("did:key:holder");
    let session_id = SessionId::now();
    let checkout_vertex = VertexId::from_bytes(b"checkout");

    let slice = SliceBuilder::new(
        origin,
        holder,
        SliceMode::Loan {
            terms: LoanTerms::default(),
            allow_subloan: false,
        },
        session_id,
        checkout_vertex,
    )
    .expires_in(Duration::from_secs(3600))
    .build();

    assert!(slice.expires_at.is_some());
}

#[test]
fn test_slice_origin_serialization() {
    let origin = SliceOrigin {
        spine_id: "spine-99".to_string(),
        entry_hash: [5u8; 32],
        entry_index: 123,
        certificate_id: Some("cert-1".to_string()),
        owner: Did::new("did:key:owner"),
    };
    let json = serde_json::to_string(&origin).unwrap();
    let parsed: SliceOrigin = serde_json::from_str(&json).unwrap();
    assert_eq!(origin.spine_id, parsed.spine_id);
    assert_eq!(origin.entry_hash, parsed.entry_hash);
    assert_eq!(origin.entry_index, parsed.entry_index);
    assert_eq!(origin.certificate_id, parsed.certificate_id);
}

#[test]
fn test_resolution_outcome_variants() {
    let returned = ResolutionOutcome::ReturnedUnchanged;
    assert_eq!(returned, ResolutionOutcome::ReturnedUnchanged);

    let committed = ResolutionOutcome::Committed {
        new_entry: [2u8; 32],
    };
    assert!(matches!(committed, ResolutionOutcome::Committed { .. }));

    let transferred = ResolutionOutcome::Transferred {
        new_spine: "spine-new".to_string(),
        new_entry: [3u8; 32],
        new_owner: Did::new("did:key:buyer"),
    };
    assert!(matches!(transferred, ResolutionOutcome::Transferred { .. }));

    let anchored = ResolutionOutcome::Anchored {
        waypoint_spine: "waypoint".to_string(),
        waypoint_entry: [4u8; 32],
    };
    assert!(matches!(anchored, ResolutionOutcome::Anchored { .. }));

    let consumed = ResolutionOutcome::Consumed;
    assert_eq!(consumed, ResolutionOutcome::Consumed);
}

#[test]
fn test_slice_constraints_custom() {
    let constraints = SliceConstraints {
        max_duration: Some(Duration::from_secs(86400)),
        allow_reslice: true,
        max_reslice_depth: Some(5),
        forbidden_operations: vec!["delete".to_string(), "transfer".to_string()],
    };
    assert_eq!(constraints.max_duration, Some(Duration::from_secs(86400)));
    assert!(constraints.allow_reslice);
    assert_eq!(constraints.max_reslice_depth, Some(5));
    assert_eq!(constraints.forbidden_operations.len(), 2);
}
