// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2024–2026 ecoPrimals Project

//! Witness chain round-trip e2e tests.
//!
//! Validates the store -> witness mapping -> wire format -> deserialize chain
//! without requiring live trio peers.  Covers the primalSpring audit action:
//! "validate witness chain under NUCLEUS mesh (store → witness → verify round-trip)."

#![allow(clippy::unwrap_used, clippy::expect_used)]

use rhizo_crypt_core::{
    MerkleRoot,
    dehydration::{AgentSummary, Attestation, AttestationStatement, DehydrationSummaryBuilder},
    dehydration_wire::{DehydrationWireSummary, WireWitnessRef},
    event::SessionOutcome,
    types::{Did, SessionId, Timestamp},
};

fn build_summary_with_attestations() -> rhizo_crypt_core::dehydration::DehydrationSummary {
    let session_id = SessionId::now();
    let merkle_root = MerkleRoot::new([42u8; 32]);
    let created_at = Timestamp::now();
    let attester = Did::new("did:key:z6MkWitness");

    let signature_bytes: bytes::Bytes = bytes::Bytes::from_static(&[
        0xde, 0xad, 0xbe, 0xef, 0x01, 0x23, 0x45, 0x67, 0x89, 0xab, 0xcd, 0xef, 0xfe, 0xdc, 0xba,
        0x98, 0x76, 0x54, 0x32, 0x10, 0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77, 0x88, 0x99, 0xaa,
        0xbb, 0xcc,
    ]);

    DehydrationSummaryBuilder::new(session_id, "witness-test", created_at, merkle_root)
        .with_outcome(SessionOutcome::Success)
        .with_vertex_count(10)
        .with_agent(AgentSummary {
            agent: attester.clone(),
            joined_at: created_at,
            left_at: None,
            event_count: 5,
            role: "author".to_string(),
        })
        .with_attestation(Attestation {
            attester,
            statement: AttestationStatement::SessionSummary {
                summary_hash: [1u8; 32],
            },
            signature: signature_bytes,
            witnessed_at: created_at,
            verified: true,
        })
        .build()
}

#[test]
fn wire_summary_witnesses_populated_from_attestations() {
    let summary = build_summary_with_attestations();
    assert_eq!(summary.attestations.len(), 1);

    let wire: DehydrationWireSummary = (&summary).into();
    assert_eq!(wire.witnesses.len(), 1, "attestation should map to one witness");

    let w = &wire.witnesses[0];
    assert_eq!(w.agent, "did:key:z6MkWitness");
    assert_eq!(w.kind, "signature");
    assert_eq!(w.encoding, "hex");
    assert_eq!(w.algorithm.as_deref(), Some("ed25519"));
    assert_eq!(w.tier.as_deref(), Some("local"));
    assert!(w.witnessed_at > 0, "witnessed_at should be non-zero");
    assert!(!w.evidence.is_empty(), "evidence should contain hex-encoded signature");
}

#[test]
fn wire_summary_json_roundtrip_fidelity() {
    let summary = build_summary_with_attestations();
    let wire: DehydrationWireSummary = (&summary).into();

    let json = serde_json::to_string(&wire).expect("serialize to JSON");
    let parsed: DehydrationWireSummary =
        serde_json::from_str(&json).expect("deserialize from JSON");

    assert_eq!(parsed.source_primal, wire.source_primal);
    assert_eq!(parsed.session_id, wire.session_id);
    assert_eq!(parsed.merkle_root, wire.merkle_root);
    assert_eq!(parsed.vertex_count, wire.vertex_count);
    assert_eq!(parsed.witnesses.len(), wire.witnesses.len());

    let orig = &wire.witnesses[0];
    let rt = &parsed.witnesses[0];
    assert_eq!(rt.agent, orig.agent);
    assert_eq!(rt.kind, orig.kind);
    assert_eq!(rt.evidence, orig.evidence);
    assert_eq!(rt.witnessed_at, orig.witnessed_at);
    assert_eq!(rt.encoding, orig.encoding);
    assert_eq!(rt.algorithm, orig.algorithm);
    assert_eq!(rt.tier, orig.tier);
    assert_eq!(rt.context, orig.context);
}

#[test]
fn signature_witness_evidence_hex_decodable() {
    let summary = build_summary_with_attestations();
    let wire: DehydrationWireSummary = (&summary).into();
    let w = &wire.witnesses[0];

    assert_eq!(w.kind, "signature");
    let decoded = hex::decode(&w.evidence).expect("evidence should be valid hex");
    assert_eq!(decoded.len(), 32, "decoded evidence should match original signature length");
    assert_eq!(w.algorithm.as_deref(), Some("ed25519"));
}

#[test]
fn non_crypto_witness_kinds_discriminate_correctly() {
    let checkpoint = WireWitnessRef {
        agent: "system".to_string(),
        kind: "checkpoint".to_string(),
        evidence: "state-hash-abc".to_string(),
        witnessed_at: 1_000_000,
        encoding: "utf8".to_string(),
        algorithm: None,
        tier: None,
        context: Some("game:tick:42".to_string()),
    };
    assert_eq!(checkpoint.kind, "checkpoint");
    assert!(checkpoint.algorithm.is_none());

    let marker = WireWitnessRef {
        agent: "system".to_string(),
        kind: "marker".to_string(),
        evidence: String::new(),
        witnessed_at: 2_000_000,
        encoding: "none".to_string(),
        algorithm: None,
        tier: None,
        context: Some("conversation:thread:xyz".to_string()),
    };
    assert_eq!(marker.kind, "marker");
    assert!(marker.algorithm.is_none());
    assert!(marker.evidence.is_empty());

    let hash_witness = WireWitnessRef {
        agent: "observer".to_string(),
        kind: "hash".to_string(),
        evidence: hex::encode([0xffu8; 32]),
        witnessed_at: 3_000_000,
        encoding: "hex".to_string(),
        algorithm: None,
        tier: Some("local".to_string()),
        context: None,
    };
    assert_eq!(hash_witness.kind, "hash");
    assert!(hash_witness.algorithm.is_none());
    let decoded = hex::decode(&hash_witness.evidence).expect("hash evidence should be valid hex");
    assert_eq!(decoded.len(), 32);
}

#[test]
fn non_crypto_witness_json_roundtrip() {
    let witnesses = vec![
        WireWitnessRef {
            agent: "system".to_string(),
            kind: "checkpoint".to_string(),
            evidence: "state-snap".to_string(),
            witnessed_at: 1_000_000,
            encoding: "utf8".to_string(),
            algorithm: None,
            tier: None,
            context: Some("game:tick:42".to_string()),
        },
        WireWitnessRef {
            agent: "system".to_string(),
            kind: "marker".to_string(),
            evidence: String::new(),
            witnessed_at: 2_000_000,
            encoding: "none".to_string(),
            algorithm: None,
            tier: None,
            context: None,
        },
    ];

    for w in &witnesses {
        let json = serde_json::to_string(w).expect("serialize witness");
        let parsed: WireWitnessRef = serde_json::from_str(&json).expect("deserialize witness");
        assert_eq!(parsed.kind, w.kind);
        assert_eq!(parsed.evidence, w.evidence);
        assert_eq!(parsed.algorithm, w.algorithm);
        assert_eq!(parsed.context, w.context);
    }
}

#[test]
fn multiple_attestations_produce_multiple_witnesses() {
    let session_id = SessionId::now();
    let merkle_root = MerkleRoot::new([7u8; 32]);
    let created_at = Timestamp::now();

    let attester_a = Did::new("did:key:z6MkAlpha");
    let attester_b = Did::new("did:key:z6MkBeta");

    let summary =
        DehydrationSummaryBuilder::new(session_id, "multi-witness", created_at, merkle_root)
            .with_outcome(SessionOutcome::Success)
            .with_vertex_count(20)
            .with_attestation(Attestation {
                attester: attester_a,
                statement: AttestationStatement::SessionSummary {
                    summary_hash: [2u8; 32],
                },
                signature: bytes::Bytes::from_static(b"sig-alpha-0123456789abcdef"),
                witnessed_at: created_at,
                verified: true,
            })
            .with_attestation(Attestation {
                attester: attester_b,
                statement: AttestationStatement::MerkleRoot {
                    root: [3u8; 32],
                },
                signature: bytes::Bytes::from_static(b"sig-beta-fedcba9876543210"),
                witnessed_at: created_at,
                verified: true,
            })
            .build();

    let wire: DehydrationWireSummary = (&summary).into();
    assert_eq!(wire.witnesses.len(), 2);

    assert_eq!(wire.witnesses[0].agent, "did:key:z6MkAlpha");
    assert_eq!(wire.witnesses[1].agent, "did:key:z6MkBeta");

    let json = serde_json::to_string(&wire).expect("serialize");
    let parsed: DehydrationWireSummary = serde_json::from_str(&json).expect("roundtrip");
    assert_eq!(parsed.witnesses.len(), 2);
}
