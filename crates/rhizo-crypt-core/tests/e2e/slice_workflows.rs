// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2024–2026 ecoPrimals Project

#![allow(clippy::unwrap_used, clippy::expect_used)]

use rhizo_crypt_core::slice::{ResolutionOutcome, SliceBuilder, SliceOrigin};
use rhizo_crypt_core::{
    Did, EventType, PrimalLifecycle, RhizoCrypt, RhizoCryptConfig, SessionBuilder, SessionType,
    SliceMode, VertexBuilder,
};

fn make_origin(owner: Did) -> SliceOrigin {
    SliceOrigin {
        spine_id: "spine-test".to_string(),
        entry_hash: [0u8; 32],
        entry_index: 0,
        certificate_id: None,
        owner,
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_checkout_copy_slice_workflow() {
    let config = RhizoCryptConfig::default();
    let mut primal = RhizoCrypt::new(config);
    primal.start().await.expect("primal should start");

    let session = SessionBuilder::new(SessionType::General).build();
    let session_id = primal.create_session(session).expect("should create session");

    let vertex = VertexBuilder::new(EventType::SessionStart).build();
    let vertex_id = primal.append_vertex(session_id, vertex).await.expect("should append vertex");

    let owner = Did::new("did:test:owner");
    let holder = Did::new("did:test:holder");
    let slice = SliceBuilder::new(
        make_origin(owner),
        holder,
        SliceMode::Copy {
            allow_recopy: false,
        },
        session_id,
        vertex_id,
    )
    .build();

    let slice_id = primal.checkout_slice(slice).expect("should checkout slice");

    let retrieved = primal.get_slice(slice_id).expect("should get slice");
    assert!(retrieved.is_active());
    assert_eq!(retrieved.session_id, session_id);
    assert_eq!(retrieved.checkout_vertex, vertex_id);
    assert!(matches!(retrieved.mode, SliceMode::Copy { .. }));

    primal.stop().await.expect("primal should stop");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_checkout_loan_slice_workflow() {
    let config = RhizoCryptConfig::default();
    let mut primal = RhizoCrypt::new(config);
    primal.start().await.expect("primal should start");

    let session = SessionBuilder::new(SessionType::General).build();
    let session_id = primal.create_session(session).expect("should create session");

    let vertex = VertexBuilder::new(EventType::SessionStart).build();
    let vertex_id = primal.append_vertex(session_id, vertex).await.expect("should append vertex");

    let owner = Did::new("did:test:owner");
    let holder = Did::new("did:test:borrower");
    let slice = SliceBuilder::new(
        make_origin(owner),
        holder.clone(),
        SliceMode::Loan {
            terms: rhizo_crypt_core::LoanTerms::default(),
            allow_subloan: false,
        },
        session_id,
        vertex_id,
    )
    .build();

    let slice_id = primal.checkout_slice(slice).expect("should checkout slice");

    let retrieved = primal.get_slice(slice_id).expect("should get slice");
    assert_eq!(retrieved.holder, holder);
    assert!(matches!(retrieved.mode, SliceMode::Loan { .. }));

    primal.stop().await.expect("primal should stop");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_slice_resolution_workflow() {
    let config = RhizoCryptConfig::default();
    let mut primal = RhizoCrypt::new(config);
    primal.start().await.expect("primal should start");

    let session = SessionBuilder::new(SessionType::General).build();
    let session_id = primal.create_session(session).expect("should create session");

    let vertex = VertexBuilder::new(EventType::SessionStart).build();
    let vertex_id = primal.append_vertex(session_id, vertex).await.expect("should append vertex");

    let owner = Did::new("did:test:owner");
    let holder = Did::new("did:test:holder");
    let slice = SliceBuilder::new(
        make_origin(owner),
        holder,
        SliceMode::Copy {
            allow_recopy: false,
        },
        session_id,
        vertex_id,
    )
    .build();

    let slice_id = primal.checkout_slice(slice).expect("should checkout slice");
    assert!(primal.get_slice(slice_id).expect("get slice").is_active());

    primal
        .resolve_slice(slice_id, ResolutionOutcome::ReturnedUnchanged)
        .expect("should resolve slice");

    let resolved = primal.get_slice(slice_id).expect("should get slice");
    assert!(resolved.is_resolved());

    primal.stop().await.expect("primal should stop");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_multiple_slices_isolation() {
    let config = RhizoCryptConfig::default();
    let mut primal = RhizoCrypt::new(config);
    primal.start().await.expect("primal should start");

    let session = SessionBuilder::new(SessionType::General).build();
    let session_id = primal.create_session(session).expect("should create session");

    let vertex = VertexBuilder::new(EventType::SessionStart).build();
    let vertex_id = primal.append_vertex(session_id, vertex).await.expect("should append vertex");

    let owner = Did::new("did:test:owner");
    let holder1 = Did::new("did:test:holder1");
    let holder2 = Did::new("did:test:holder2");

    let slice1 = SliceBuilder::new(
        make_origin(owner.clone()),
        holder1,
        SliceMode::Copy {
            allow_recopy: false,
        },
        session_id,
        vertex_id,
    )
    .build();

    let mut origin2 = make_origin(owner);
    origin2.spine_id = "spine-other".to_string();
    let slice2 = SliceBuilder::new(
        origin2,
        holder2,
        SliceMode::Loan {
            terms: rhizo_crypt_core::LoanTerms::default(),
            allow_subloan: true,
        },
        session_id,
        vertex_id,
    )
    .build();

    let slice_id1 = primal.checkout_slice(slice1).expect("should checkout slice1");
    let slice_id2 = primal.checkout_slice(slice2).expect("should checkout slice2");

    let s1 = primal.get_slice(slice_id1).expect("get slice1");
    let s2 = primal.get_slice(slice_id2).expect("get slice2");

    assert_ne!(slice_id1, slice_id2);
    assert_ne!(s1.id, s2.id);
    assert!(matches!(s1.mode, SliceMode::Copy { .. }));
    assert!(matches!(s2.mode, SliceMode::Loan { .. }));

    primal.resolve_slice(slice_id1, ResolutionOutcome::ReturnedUnchanged).expect("resolve slice1");

    let s2_after = primal.get_slice(slice_id2).expect("get slice2");
    assert!(s2_after.is_active());

    primal.stop().await.expect("primal should stop");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_slice_mode_validation() {
    let config = RhizoCryptConfig::default();
    let mut primal = RhizoCrypt::new(config);
    primal.start().await.expect("primal should start");

    let session = SessionBuilder::new(SessionType::General).build();
    let session_id = primal.create_session(session).expect("should create session");

    let vertex = VertexBuilder::new(EventType::SessionStart).build();
    let vertex_id = primal.append_vertex(session_id, vertex).await.expect("should append vertex");

    let owner = Did::new("did:test:owner");
    let holder = Did::new("did:test:holder");

    let copy_slice = SliceBuilder::new(
        make_origin(owner.clone()),
        holder.clone(),
        SliceMode::Copy {
            allow_recopy: true,
        },
        session_id,
        vertex_id,
    )
    .build();

    let transfer_slice = SliceBuilder::new(
        make_origin(owner),
        Did::new("did:test:new_owner"),
        SliceMode::Transfer {
            new_owner: Did::new("did:test:buyer"),
        },
        session_id,
        vertex_id,
    )
    .build();

    let copy_id = primal.checkout_slice(copy_slice).expect("checkout copy");
    let transfer_id = primal.checkout_slice(transfer_slice).expect("checkout transfer");

    let copy_retrieved = primal.get_slice(copy_id).expect("get copy slice");
    let transfer_retrieved = primal.get_slice(transfer_id).expect("get transfer slice");

    assert!(matches!(
        copy_retrieved.mode,
        SliceMode::Copy {
            allow_recopy: true
        }
    ));
    assert!(matches!(transfer_retrieved.mode, SliceMode::Transfer { .. }));

    primal.stop().await.expect("primal should stop");
}
