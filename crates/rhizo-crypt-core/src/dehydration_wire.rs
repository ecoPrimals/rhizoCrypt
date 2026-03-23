// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2024–2026 ecoPrimals Project

//! Outbound wire types for JSON-RPC dehydration notifications.
//!
//! These structs define rhizoCrypt's own serialization format for the
//! `contribution.record_dehydration` JSON-RPC call to the provenance
//! provider (sweetGrass or any compatible endpoint).
//!
//! Each primal owns its own wire types — the shared contract is the
//! JSON schema on the wire, not a compile-time Rust crate. This
//! eliminates cross-primal compile-time coupling (sovereignty).

use serde::{Deserialize, Serialize};

/// Outbound dehydration summary sent to the provenance provider.
///
/// Serialized as the `params` of a `contribution.record_dehydration`
/// JSON-RPC request. The receiver deserializes with `#[serde(default)]`
/// on optional fields for forward compatibility.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DehydrationWireSummary {
    /// Primal that performed the dehydration.
    pub source_primal: String,
    /// Session that was dehydrated.
    pub session_id: String,
    /// Merkle root of the collapsed DAG (hex).
    pub merkle_root: String,
    /// Total vertices in the original DAG.
    pub vertex_count: u64,
    /// Number of branches explored.
    #[serde(default)]
    pub branch_count: u64,
    /// Total payload bytes.
    #[serde(default)]
    pub payload_bytes: u64,
    /// DIDs of participating agents.
    #[serde(default)]
    pub agents: Vec<String>,
    /// Session creation time (nanoseconds since epoch).
    #[serde(default)]
    pub session_start: u64,
    /// Dehydration time (nanoseconds since epoch).
    #[serde(default)]
    pub dehydrated_at: u64,
    /// Session type identifier (e.g., "experiment").
    #[serde(default)]
    pub session_type: String,
    /// Session outcome as string.
    #[serde(default)]
    pub outcome: String,
    /// Per-agent participation detail.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub agent_summaries: Vec<WireAgentRef>,
    /// Cryptographic attestations.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub attestations: Vec<WireAttestationRef>,
    /// Operations performed during the session.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub operations: Vec<WireOperationRef>,
    /// Frontier hashes (DAG leaf nodes at dehydration time).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub frontier: Vec<String>,
    /// Niche context (e.g., "rootpulse").
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub niche: Option<String>,
    /// Compression ratio if the DAG was compressed.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub compression_ratio: Option<f64>,
}

/// Per-agent participation summary on the wire.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct WireAgentRef {
    /// Agent DID.
    pub agent: String,
    /// When the agent joined (nanoseconds since epoch).
    #[serde(default)]
    pub joined_at: u64,
    /// When the agent left (`None` if still active).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub left_at: Option<u64>,
    /// Events produced by this agent.
    #[serde(default)]
    pub event_count: u64,
    /// Agent role in the session.
    #[serde(default)]
    pub role: String,
}

/// Cryptographic attestation reference on the wire.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct WireAttestationRef {
    /// Attesting agent DID.
    pub agent: String,
    /// Hex-encoded signature.
    pub signature: String,
    /// When the attestation was created (nanoseconds since epoch).
    #[serde(default)]
    pub attested_at: u64,
}

/// A high-level operation recorded during a session (wire format).
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct WireOperationRef {
    /// Operation type (e.g., "create", "modify", "derive", "merge").
    pub op_type: String,
    /// Content hash of the affected artifact.
    pub content_hash: String,
    /// Agent who performed the operation.
    pub agent: String,
    /// When the operation occurred (nanoseconds since epoch).
    #[serde(default)]
    pub timestamp: u64,
    /// Optional description.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn minimal_roundtrip() {
        let payload = serde_json::json!({
            "source_primal": "rhizoCrypt",
            "session_id": "sess-1",
            "merkle_root": "sha256:abc",
            "vertex_count": 10
        });
        let s: DehydrationWireSummary =
            serde_json::from_value(payload).expect("deserialize minimal");
        assert_eq!(s.session_id, "sess-1");
        assert_eq!(s.vertex_count, 10);
        assert_eq!(s.branch_count, 0);
        assert_eq!(s.session_start, 0);
    }

    #[test]
    fn full_roundtrip() {
        let summary = DehydrationWireSummary {
            source_primal: "rhizoCrypt".to_string(),
            session_id: "sess-2".to_string(),
            merkle_root: "sha256:def".to_string(),
            vertex_count: 100,
            branch_count: 5,
            payload_bytes: 4096,
            agents: vec!["did:key:z6MkAlice".to_string()],
            session_start: 1_000_000,
            dehydrated_at: 2_000_000,
            session_type: "rootpulse".to_string(),
            outcome: "Success".to_string(),
            agent_summaries: vec![WireAgentRef {
                agent: "did:key:z6MkAlice".to_string(),
                joined_at: 1_000_000,
                left_at: Some(2_000_000),
                event_count: 42,
                role: "author".to_string(),
            }],
            attestations: vec![WireAttestationRef {
                agent: "did:key:z6MkAlice".to_string(),
                signature: "base64sig==".to_string(),
                attested_at: 2_000_000,
            }],
            operations: vec![WireOperationRef {
                op_type: "create".to_string(),
                content_hash: "sha256:op1".to_string(),
                agent: "did:key:z6MkAlice".to_string(),
                timestamp: 1_500_000,
                description: Some("Initial creation".to_string()),
            }],
            frontier: vec!["sha256:frontier1".to_string()],
            niche: Some("rootpulse".to_string()),
            compression_ratio: Some(0.42),
        };
        let json = serde_json::to_string(&summary).expect("serialize");
        let parsed: DehydrationWireSummary =
            serde_json::from_str(&json).expect("deserialize roundtrip");
        assert_eq!(parsed.vertex_count, 100);
        assert_eq!(parsed.agent_summaries.len(), 1);
        assert_eq!(parsed.attestations.len(), 1);
        assert_eq!(parsed.operations.len(), 1);
    }

    #[test]
    fn extra_fields_tolerated() {
        let payload = serde_json::json!({
            "source_primal": "rhizoCrypt",
            "session_id": "sess-3",
            "merkle_root": "sha256:xyz",
            "vertex_count": 1,
            "unknown_future_field": "should be silently dropped"
        });
        let s: DehydrationWireSummary =
            serde_json::from_value(payload).expect("deserialize with unknown fields");
        assert_eq!(s.source_primal, "rhizoCrypt");
    }
}
