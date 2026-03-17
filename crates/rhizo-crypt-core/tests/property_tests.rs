// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2024–2026 ecoPrimals Project

//! Property-Based Tests for rhizoCrypt Core
//!
//! Uses proptest to validate invariants across a wide range of inputs.
//! These tests complement unit tests by exploring edge cases automatically.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use proptest::prelude::*;
use rhizo_crypt_core::{
    SessionType,
    event::EventType,
    merkle::{MerkleRoot, MerkleTreeBuilder},
    session::{SessionBuilder, SessionState},
    types::*,
    vertex::VertexBuilder,
};

// ============================================================================
// Arbitrary Implementations for Core Types
// ============================================================================

/// Generate arbitrary session types.
fn arb_session_type() -> impl Strategy<Value = SessionType> {
    prop_oneof![
        Just(SessionType::General),
        Just(SessionType::Gaming {
            game_id: "test-game".to_string()
        }),
        Just(SessionType::Experiment {
            protocol_id: "test-proto".to_string()
        }),
        Just(SessionType::Collaboration {
            workspace_id: "test-ws".to_string()
        }),
    ]
}

/// Generate arbitrary event types.
fn arb_event_type() -> impl Strategy<Value = EventType> {
    use rhizo_crypt_core::event::{AgentRole, LeaveReason, SessionOutcome};
    prop_oneof![
        Just(EventType::SessionStart),
        Just(EventType::SessionEnd {
            outcome: SessionOutcome::Success
        }),
        any::<Option<String>>().prop_map(|s| EventType::DataCreate {
            schema: s
        }),
        Just(EventType::AgentJoin {
            role: AgentRole::Participant
        }),
        Just(EventType::AgentLeave {
            reason: LeaveReason::Normal
        }),
    ]
}

// ============================================================================
// Vertex ID Properties
// ============================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(1000))]

    /// VertexId should be deterministic based on input bytes
    #[test]
    fn prop_vertex_id_deterministic(data: Vec<u8>) {
        let id1 = VertexId::from_bytes(&data);
        let id2 = VertexId::from_bytes(&data);
        prop_assert_eq!(id1, id2);
    }

    /// Different data should produce different IDs (with high probability)
    #[test]
    fn prop_vertex_id_collision_resistant(
        data1 in prop::collection::vec(any::<u8>(), 1..1000),
        data2 in prop::collection::vec(any::<u8>(), 1..1000)
    ) {
        prop_assume!(data1 != data2);
        let id1 = VertexId::from_bytes(&data1);
        let id2 = VertexId::from_bytes(&data2);
        prop_assert_ne!(id1, id2);
    }

    /// VertexId bytes should be consistent with Display
    #[test]
    fn prop_vertex_id_display_consistent(data in prop::collection::vec(any::<u8>(), 1..1000)) {
        let id = VertexId::from_bytes(&data);
        let display = format!("{id}");
        // Display is truncated to first 16 hex chars (8 bytes)
        prop_assert!(display.chars().all(|c| c.is_ascii_hexdigit()));
        prop_assert_eq!(display.len(), 16);

        // Full hex should be 64 chars
        let full_hex = id.to_hex();
        prop_assert_eq!(full_hex.len(), 64);
        prop_assert!(display == full_hex[..16]);
    }
}

// ============================================================================
// Session ID Properties
// ============================================================================

proptest! {
    /// SessionId::now() should always produce unique IDs
    #[test]
    fn prop_session_id_unique(_dummy: u8) {
        let id1 = SessionId::now();
        let id2 = SessionId::now();
        // UUIDs should be unique (v7 uses timestamp + random)
        prop_assert_ne!(id1, id2);
    }
}

// ============================================================================
// Timestamp Properties
// ============================================================================

proptest! {
    /// Timestamp::now() should be monotonically increasing within resolution
    #[test]
    fn prop_timestamp_monotonic(_dummy: u8) {
        let t1 = Timestamp::now();
        // Small operation to ensure some time passes
        let _ = t1.as_nanos();
        let t2 = Timestamp::now();
        prop_assert!(t2.as_nanos() >= t1.as_nanos());
    }

    /// Timestamp from nanos roundtrip
    #[test]
    fn prop_timestamp_nanos_roundtrip(nanos: u64) {
        let ts = Timestamp::from_nanos(nanos);
        prop_assert_eq!(ts.as_nanos(), nanos);
    }
}

// ============================================================================
// Vertex Properties
// ============================================================================

proptest! {
    /// Vertex ID should be derived from content
    #[test]
    fn prop_vertex_content_addressed(
        event_type in arb_event_type(),
    ) {
        let vertex = VertexBuilder::new(event_type).build();
        // Vertex should have a valid computed ID
        let id = vertex.compute_id().unwrap();
        prop_assert!(!id.as_bytes().iter().all(|&b| b == 0));
    }

    /// Vertex with parents should track them correctly
    #[test]
    fn prop_vertex_parents(
        parent_data in prop::collection::vec(any::<[u8; 32]>(), 0..5),
    ) {
        let parent_ids: Vec<VertexId> = parent_data.iter().map(|d| VertexId::new(*d)).collect();

        let mut builder = VertexBuilder::new(EventType::SessionStart);
        for parent in &parent_ids {
            builder = builder.with_parent(*parent);
        }
        let vertex = builder.build();

        prop_assert_eq!(vertex.parents.len(), parent_ids.len());
        for parent in &parent_ids {
            prop_assert!(vertex.parents.contains(parent));
        }
    }

    /// Vertex metadata should be preserved
    #[test]
    fn prop_vertex_metadata(
        keys in prop::collection::vec(r"[a-zA-Z0-9_]{1,20}", 0..10),
        values in prop::collection::vec(r"[a-zA-Z0-9_]{0,50}", 0..10),
    ) {
        let pairs: Vec<_> = keys.into_iter().zip(values).collect();

        let mut builder = VertexBuilder::new(EventType::SessionStart);
        for (k, v) in &pairs {
            builder = builder.with_metadata(k.as_str(), v.as_str());
        }
        let vertex = builder.build();

        // All keys should be present
        for (k, _) in &pairs {
            prop_assert!(vertex.metadata.contains_key(k));
        }
    }
}

// ============================================================================
// Session Properties
// ============================================================================

proptest! {
    /// Session should start in Active state
    #[test]
    fn prop_session_initial_state(
        session_type in arb_session_type(),
    ) {
        let session = SessionBuilder::new(session_type).build();
        prop_assert!(matches!(session.state, SessionState::Active));
        prop_assert_eq!(session.vertex_count, 0);
        prop_assert!(session.genesis.is_empty());
        prop_assert!(session.frontier.is_empty());
    }

    /// Session with name should preserve it
    #[test]
    fn prop_session_name(
        session_type in arb_session_type(),
        name in r"[a-zA-Z0-9 _-]{0,100}",
    ) {
        let session = SessionBuilder::new(session_type)
            .with_name(&name)
            .build();
        prop_assert_eq!(session.name, Some(name));
    }

    /// Session with max vertices should preserve limit
    #[test]
    fn prop_session_max_vertices(
        session_type in arb_session_type(),
        max_vertices in 1u64..1_000_000,
    ) {
        let session = SessionBuilder::new(session_type)
            .with_max_vertices(max_vertices)
            .build();
        prop_assert_eq!(session.config.max_vertices, max_vertices);
    }
}

// ============================================================================
// Merkle Tree Properties
// ============================================================================

proptest! {
    /// Merkle tree with data should produce non-zero root
    #[test]
    fn prop_merkle_non_empty_root(
        count in 1usize..20,
    ) {
        let mut builder = MerkleTreeBuilder::new();
        for i in 0..count {
            let vertex = VertexBuilder::new(EventType::DataCreate {
                schema: Some(format!("schema-{i}"))
            }).build();
            builder.add_vertex(vertex);
        }
        let root = builder.compute_root().unwrap();

        // Root should not be all zeros
        prop_assert!(!root.as_bytes().iter().all(|&b| b == 0));
    }

    /// Merkle root should be deterministic for same vertices
    #[test]
    fn prop_merkle_deterministic_same_input(count in 1usize..10) {
        // Build same set of vertices twice
        let vertices1: Vec<_> = (0..count)
            .map(|i| {
                VertexBuilder::new(EventType::DataCreate { schema: Some(format!("test-{i}")) })
                    .with_metadata("index", i.to_string())
                    .build()
            })
            .collect();

        // Compute roots from same vertex sequence
        let root1 = MerkleRoot::compute(&vertices1).unwrap();
        let root2 = MerkleRoot::compute(&vertices1).unwrap();

        prop_assert_eq!(root1, root2);
    }

    /// Merkle proofs should be verifiable
    #[test]
    fn prop_merkle_proof_verifiable(count in 1usize..10, index in 0usize..10) {
        let vertices: Vec<_> = (0..count)
            .map(|i| {
                VertexBuilder::new(EventType::DataCreate { schema: Some(format!("test-{i}")) })
                    .build()
            })
            .collect();

        let actual_index = index % count;
        let mut builder = MerkleTreeBuilder::new();
        builder.add_vertices(vertices.clone());

        if let Ok(proof) = builder.generate_proof(actual_index) {
            // Proof should verify against the correct vertex
            let valid = proof.verify(&vertices[actual_index]);
            prop_assert!(valid);
        }
    }
}

// ============================================================================
// Signature Properties
// ============================================================================

proptest! {
    /// Signature should preserve bytes
    #[test]
    fn prop_signature_roundtrip(
        bytes in prop::collection::vec(any::<u8>(), 64..65),
    ) {
        let sig = Signature::new(bytes.clone());
        prop_assert_eq!(sig.as_bytes(), &bytes[..]);
    }
}

// ============================================================================
// PayloadRef Properties
// ============================================================================

proptest! {
    /// PayloadRef from hash should be deterministic
    #[test]
    fn prop_payload_ref_deterministic(
        data: Vec<u8>,
    ) {
        let hash = blake3::hash(&data);
        let ref1 = PayloadRef::from_hash(hash.as_bytes());
        let ref2 = PayloadRef::from_hash(hash.as_bytes());
        prop_assert_eq!(ref1, ref2);
    }
}

// ============================================================================
// Niche / Capability Introspection Properties
// ============================================================================

proptest! {
    /// capability_list() JSON roundtrips through serde without data loss
    #[test]
    fn prop_capability_list_json_roundtrip(_dummy: u8) {
        let list = rhizo_crypt_core::niche::capability_list();
        let json_str = serde_json::to_string(&list).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json_str).unwrap();

        prop_assert_eq!(
            parsed.get("primal").and_then(|v| v.as_str()),
            Some("rhizocrypt")
        );
        prop_assert!(parsed.get("capabilities").unwrap().as_array().is_some());
        prop_assert!(parsed.get("methods").unwrap().as_array().is_some());
        prop_assert!(parsed.get("domains").unwrap().as_array().is_some());
    }
}

// ============================================================================
// IpcErrorPhase Properties
// ============================================================================

proptest! {
    /// is_retriable and is_application_error are mutually exclusive for known phases
    #[test]
    fn prop_ipc_phase_retriable_vs_application(code in -40000i64..0i64) {
        use rhizo_crypt_core::error::IpcErrorPhase;

        let phases = [
            IpcErrorPhase::Connect,
            IpcErrorPhase::Write,
            IpcErrorPhase::Read,
            IpcErrorPhase::InvalidJson,
            IpcErrorPhase::HttpStatus(500),
            IpcErrorPhase::NoResult,
            IpcErrorPhase::JsonRpcError(code),
        ];

        for phase in &phases {
            // No phase should be both retriable and an application error
            prop_assert!(!(phase.is_retriable() && phase.is_application_error()));
        }
    }
}

// ============================================================================
// extract_rpc_error Properties
// ============================================================================

proptest! {
    /// extract_rpc_error returns None for any JSON without an "error" field
    #[test]
    fn prop_extract_rpc_error_none_without_error(result_val in r"[a-z0-9]{1,20}") {
        let response = serde_json::json!({
            "jsonrpc": "2.0",
            "result": result_val,
            "id": 1
        });
        prop_assert!(rhizo_crypt_core::error::extract_rpc_error(&response).is_none());
    }

    /// extract_rpc_error returns Some for any JSON with an "error" field
    #[test]
    fn prop_extract_rpc_error_some_with_error(
        code in -40000i64..0i64,
        msg in r"[a-z ]{1,50}"
    ) {
        let response = serde_json::json!({
            "jsonrpc": "2.0",
            "error": {"code": code, "message": msg},
            "id": 1
        });
        let (extracted_code, extracted_msg) =
            rhizo_crypt_core::error::extract_rpc_error(&response).unwrap();
        prop_assert_eq!(extracted_code, code);
        prop_assert_eq!(extracted_msg, msg);
    }
}

// ============================================================================
// DispatchOutcome Properties
// ============================================================================

proptest! {
    /// DispatchOutcome::Ok always converts to Ok result
    #[test]
    fn prop_dispatch_outcome_ok_roundtrip(val in 0u32..u32::MAX) {
        use rhizo_crypt_core::error::DispatchOutcome;

        let outcome: DispatchOutcome<u32> = DispatchOutcome::Ok(val);
        prop_assert!(outcome.is_ok());
        let result = outcome.into_result().unwrap();
        prop_assert_eq!(result, val);
    }

    /// DispatchOutcome::ApplicationError always converts to Err
    #[test]
    fn prop_dispatch_outcome_app_error(code in -40000i64..0i64) {
        use rhizo_crypt_core::error::DispatchOutcome;

        let outcome: DispatchOutcome<u32> = DispatchOutcome::ApplicationError {
            code,
            message: "test".into(),
        };
        prop_assert!(!outcome.is_ok());
        let err = outcome.into_result().unwrap_err();
        let expected = format!("jsonrpc_{code}");
        prop_assert!(err.to_string().contains(&expected));
    }
}

// ============================================================================
// JSON-RPC Proptest Fuzz (absorbed from airSpring v0.8.7)
// ============================================================================

proptest! {
    /// Any JSON-RPC 2.0 request with a valid method is parseable.
    #[test]
    fn prop_jsonrpc_request_roundtrip(
        method in "[a-z]+\\.[a-z]+\\.[a-z]+",
        id in 1u64..u64::MAX,
    ) {
        let request = serde_json::json!({
            "jsonrpc": "2.0",
            "method": method,
            "params": {},
            "id": id,
        });
        let parsed: serde_json::Value = serde_json::from_str(&request.to_string()).unwrap();
        prop_assert_eq!(parsed["method"].as_str().unwrap(), &method);
        prop_assert_eq!(parsed["id"].as_u64().unwrap(), id);
    }

    /// JSON-RPC error responses with any code/message are extractable.
    #[test]
    fn prop_jsonrpc_error_any_code(code in -50000i64..50000i64) {
        let response = serde_json::json!({
            "jsonrpc": "2.0",
            "error": { "code": code, "message": "fuzz" },
            "id": 1,
        });
        let result = rhizo_crypt_core::error::extract_rpc_error(&response);
        prop_assert!(result.is_some());
        let (extracted_code, _) = result.unwrap();
        prop_assert_eq!(extracted_code, code);
    }

    /// JSON-RPC success responses never extract an error.
    #[test]
    fn prop_jsonrpc_success_no_error(val in 0u32..10000) {
        let response = serde_json::json!({
            "jsonrpc": "2.0",
            "result": val,
            "id": 1,
        });
        prop_assert!(rhizo_crypt_core::error::extract_rpc_error(&response).is_none());
    }

    /// IpcErrorPhase is_retriable and is_application_error are mutually exclusive.
    #[test]
    fn prop_ipc_phase_mutual_exclusion(code in -50000i64..50000i64) {
        use rhizo_crypt_core::error::IpcErrorPhase;
        let phase = IpcErrorPhase::JsonRpcError(code);
        prop_assert!(!(phase.is_retriable() && phase.is_application_error()));
    }

    /// ValidationHarness pass/fail counts always sum to total checks.
    #[test]
    fn prop_validation_harness_counts(
        checks in prop::collection::vec(prop::bool::ANY, 0..50),
    ) {
        let mut v = rhizo_crypt_core::error::ValidationHarness::new("test");
        for (i, passed) in checks.iter().enumerate() {
            v.check(format!("check_{i}"), *passed);
        }
        prop_assert_eq!(v.pass_count() + v.fail_count(), checks.len());
        prop_assert_eq!(v.all_passed(), checks.iter().all(|p| *p));
        let expected_code = u8::from(!checks.iter().all(|p| *p));
        prop_assert_eq!(v.exit_code(), expected_code);
    }

    /// ValidationSink receives same data as finish() would output.
    #[test]
    fn prop_validation_sink_captures(
        checks in prop::collection::vec(prop::bool::ANY, 1..20),
    ) {
        let mut v = rhizo_crypt_core::error::ValidationHarness::new("sink_test");
        for (i, passed) in checks.iter().enumerate() {
            v.check(format!("c{i}"), *passed);
        }
        let mut sink = rhizo_crypt_core::error::StringSink::default();
        let code = v.finish_to(&mut sink);
        prop_assert_eq!(code, v.exit_code());
        let expected_header = format!("{}/{}", v.pass_count(), checks.len());
        prop_assert!(sink.output.contains("sink_test"));
        prop_assert!(sink.output.contains(&expected_header));
    }
}

// ============================================================================
// 4-Format Capability Parsing Properties
// ============================================================================

proptest! {
    /// Format A: flat string array always extracts all capabilities.
    #[test]
    fn prop_capabilities_format_a(
        caps in prop::collection::vec("[a-z]+\\.[a-z]+", 0..20),
    ) {
        let value = serde_json::json!(caps);
        let extracted = rhizo_crypt_core::discovery::extract_capabilities(&value);
        prop_assert_eq!(extracted.len(), caps.len());
    }

    /// Format B: nested objects with "name" field.
    #[test]
    fn prop_capabilities_format_b(
        cap in "[a-z]+\\.[a-z]+"
    ) {
        let value = serde_json::json!([{"name": cap, "version": "1.0"}]);
        let extracted = rhizo_crypt_core::discovery::extract_capabilities(&value);
        prop_assert_eq!(extracted.len(), 1);
        prop_assert_eq!(&extracted[0], &cap);
    }

    /// Format C: wrapper object with "capabilities" key.
    #[test]
    fn prop_capabilities_format_c(
        caps in prop::collection::vec("[a-z]+\\.[a-z]+", 0..10),
    ) {
        let value = serde_json::json!({"capabilities": caps});
        let extracted = rhizo_crypt_core::discovery::extract_capabilities(&value);
        prop_assert_eq!(extracted.len(), caps.len());
    }

    /// Format D: double-nested (wrapper + objects).
    #[test]
    fn prop_capabilities_format_d(
        cap in "[a-z]+\\.[a-z]+"
    ) {
        let value = serde_json::json!({"capabilities": [{"name": cap}]});
        let extracted = rhizo_crypt_core::discovery::extract_capabilities(&value);
        prop_assert_eq!(extracted.len(), 1);
        prop_assert_eq!(&extracted[0], &cap);
    }
}
