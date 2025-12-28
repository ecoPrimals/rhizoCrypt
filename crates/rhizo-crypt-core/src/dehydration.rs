//! Dehydration protocol for committing DAG sessions to LoamSpine.
//!
//! Dehydration is the process of committing a RhizoCrypt DAG session to
//! permanent storage in LoamSpine. The name reflects the biological metaphor:
//! the ephemeral, water-rich rhizome layer dries and compresses into the
//! permanent loam fossil record.

use crate::event::SessionOutcome;
use crate::merkle::MerkleRoot;
use crate::session::LoamCommitRef;
use crate::types::{ContentHash, Did, PayloadRef, SessionId, Timestamp, VertexId};
use serde::{Deserialize, Serialize};

/// Summary of a dehydrated session for LoamSpine commit.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DehydrationSummary {
    /// Session identifier.
    pub session_id: SessionId,

    /// Session type identifier.
    pub session_type: String,

    /// When the session was created.
    pub created_at: Timestamp,

    /// When the session was resolved.
    pub resolved_at: Timestamp,

    /// Session outcome.
    pub outcome: SessionOutcome,

    /// Merkle root of the session.
    pub merkle_root: MerkleRoot,

    /// Total number of vertices.
    pub vertex_count: u64,

    /// Total payload bytes.
    pub payload_bytes: u64,

    /// Extracted results (domain-specific).
    pub results: Vec<ResultEntry>,

    /// Agent participation summary.
    pub agents: Vec<AgentSummary>,

    /// Signatures attesting to the summary.
    pub attestations: Vec<Attestation>,
}

impl DehydrationSummary {
    /// Check if the summary has all required attestations.
    #[must_use]
    pub fn has_required_attestations(&self, required: &[Did]) -> bool {
        required
            .iter()
            .all(|did| self.attestations.iter().any(|a| &a.attester == did && a.verified))
    }

    /// Get the summary hash for signing.
    #[must_use]
    pub fn compute_hash(&self) -> ContentHash {
        // Hash key fields for attestation
        let mut hasher = blake3::Hasher::new();
        hasher.update(self.session_id.as_uuid().as_bytes());
        hasher.update(&self.merkle_root.0);
        hasher.update(&self.resolved_at.as_nanos().to_le_bytes());
        hasher.finalize().into()
    }
}

/// A result entry extracted from the session.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ResultEntry {
    /// Result type.
    pub result_type: String,

    /// Result key (domain-specific identifier).
    pub key: String,

    /// Result value as JSON.
    pub value: serde_json::Value,

    /// Reference to source vertex.
    pub source_vertex: VertexId,

    /// Reference to payload (if large).
    pub payload_ref: Option<PayloadRef>,
}

/// Summary of an agent's participation in the session.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AgentSummary {
    /// Agent DID.
    pub agent: Did,

    /// When the agent joined.
    pub joined_at: Timestamp,

    /// When the agent left (if applicable).
    pub left_at: Option<Timestamp>,

    /// Number of events created by this agent.
    pub event_count: u64,

    /// Role in the session.
    pub role: String,
}

/// Attestation from a participant.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Attestation {
    /// Attester DID.
    pub attester: Did,

    /// What is being attested.
    pub statement: AttestationStatement,

    /// Signature over the statement.
    pub signature: Vec<u8>,

    /// When the attestation was made.
    pub attested_at: Timestamp,

    /// Whether the signature has been verified.
    pub verified: bool,
}

/// What an attestation claims.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum AttestationStatement {
    /// Attests that the session summary is correct.
    SessionSummary {
        /// Hash of the summary.
        summary_hash: ContentHash,
    },

    /// Attests that the Merkle root is correct.
    MerkleRoot {
        /// The Merkle root.
        root: ContentHash,
    },

    /// Attests to a specific result.
    Result {
        /// Result key.
        key: String,
        /// Result hash.
        value_hash: ContentHash,
    },

    /// Custom attestation.
    Custom {
        /// Statement type.
        statement_type: String,
        /// Statement data.
        data: ContentHash,
    },
}

/// Dehydration configuration.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct DehydrationConfig {
    /// Include full vertices in summary.
    pub include_vertices: bool,

    /// Include payloads in summary.
    pub include_payloads: bool,

    /// Generate Merkle proofs for specific vertices.
    pub generate_proofs_for: Vec<VertexId>,

    /// Required attestations before commit.
    pub required_attestations: Vec<Did>,

    /// Timeout for attestation collection (seconds).
    pub attestation_timeout_secs: Option<u64>,

    /// Require all attestations before committing.
    pub require_all_attestations: bool,
}

impl DehydrationConfig {
    /// Create a minimal dehydration config.
    #[must_use]
    pub const fn minimal() -> Self {
        Self {
            include_vertices: false,
            include_payloads: false,
            generate_proofs_for: Vec::new(),
            required_attestations: Vec::new(),
            attestation_timeout_secs: Some(60),
            require_all_attestations: false,
        }
    }

    /// Create a full dehydration config.
    #[must_use]
    pub const fn full() -> Self {
        Self {
            include_vertices: true,
            include_payloads: true,
            generate_proofs_for: Vec::new(),
            required_attestations: Vec::new(),
            attestation_timeout_secs: Some(300),
            require_all_attestations: false,
        }
    }
}

/// Status of a dehydration operation.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum DehydrationStatus {
    /// Not started.
    Pending,

    /// Computing Merkle root.
    ComputingRoot,

    /// Generating summary.
    GeneratingSummary,

    /// Collecting attestations.
    CollectingAttestations {
        /// Collected so far.
        collected: usize,
        /// Required total.
        required: usize,
    },

    /// Committing to LoamSpine.
    Committing,

    /// Successfully completed.
    Completed {
        /// LoamSpine commit reference.
        commit_ref: LoamCommitRef,
    },

    /// Failed.
    Failed {
        /// Error message.
        error: String,
    },
}

impl DehydrationStatus {
    /// Check if dehydration is complete.
    #[must_use]
    pub const fn is_complete(&self) -> bool {
        matches!(self, Self::Completed { .. })
    }

    /// Check if dehydration failed.
    #[must_use]
    pub const fn is_failed(&self) -> bool {
        matches!(self, Self::Failed { .. })
    }

    /// Check if dehydration is in progress.
    #[must_use]
    pub const fn is_in_progress(&self) -> bool {
        matches!(
            self,
            Self::ComputingRoot
                | Self::GeneratingSummary
                | Self::CollectingAttestations { .. }
                | Self::Committing
        )
    }
}

/// Builder for creating dehydration summaries.
#[derive(Clone, Debug)]
pub struct DehydrationSummaryBuilder {
    session_id: SessionId,
    session_type: String,
    created_at: Timestamp,
    resolved_at: Timestamp,
    outcome: SessionOutcome,
    merkle_root: MerkleRoot,
    vertex_count: u64,
    payload_bytes: u64,
    results: Vec<ResultEntry>,
    agents: Vec<AgentSummary>,
    attestations: Vec<Attestation>,
}

impl DehydrationSummaryBuilder {
    /// Create a new builder.
    #[must_use]
    pub fn new(
        session_id: SessionId,
        session_type: impl Into<String>,
        created_at: Timestamp,
        merkle_root: MerkleRoot,
    ) -> Self {
        Self {
            session_id,
            session_type: session_type.into(),
            created_at,
            resolved_at: Timestamp::now(),
            outcome: SessionOutcome::Success,
            merkle_root,
            vertex_count: 0,
            payload_bytes: 0,
            results: Vec::new(),
            agents: Vec::new(),
            attestations: Vec::new(),
        }
    }

    /// Set the outcome.
    #[must_use]
    pub fn with_outcome(mut self, outcome: SessionOutcome) -> Self {
        self.outcome = outcome;
        self
    }

    /// Set the vertex count.
    #[must_use]
    pub const fn with_vertex_count(mut self, count: u64) -> Self {
        self.vertex_count = count;
        self
    }

    /// Set the payload bytes.
    #[must_use]
    pub const fn with_payload_bytes(mut self, bytes: u64) -> Self {
        self.payload_bytes = bytes;
        self
    }

    /// Add a result entry.
    #[must_use]
    pub fn with_result(mut self, result: ResultEntry) -> Self {
        self.results.push(result);
        self
    }

    /// Add an agent summary.
    #[must_use]
    pub fn with_agent(mut self, agent: AgentSummary) -> Self {
        self.agents.push(agent);
        self
    }

    /// Add an attestation.
    #[must_use]
    pub fn with_attestation(mut self, attestation: Attestation) -> Self {
        self.attestations.push(attestation);
        self
    }

    /// Build the summary.
    #[must_use]
    pub fn build(self) -> DehydrationSummary {
        DehydrationSummary {
            session_id: self.session_id,
            session_type: self.session_type,
            created_at: self.created_at,
            resolved_at: self.resolved_at,
            outcome: self.outcome,
            merkle_root: self.merkle_root,
            vertex_count: self.vertex_count,
            payload_bytes: self.payload_bytes,
            results: self.results,
            agents: self.agents,
            attestations: self.attestations,
        }
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;

    #[test]
    fn test_dehydration_summary_builder() {
        let session_id = SessionId::now();
        let merkle_root = MerkleRoot::ZERO;
        let created_at = Timestamp::now();

        let summary = DehydrationSummaryBuilder::new(session_id, "gaming", created_at, merkle_root)
            .with_outcome(SessionOutcome::Success)
            .with_vertex_count(1000)
            .with_payload_bytes(1024 * 1024)
            .build();

        assert_eq!(summary.vertex_count, 1000);
        assert_eq!(summary.payload_bytes, 1024 * 1024);
        assert!(summary.attestations.is_empty());
    }

    #[test]
    fn test_dehydration_status() {
        assert!(!DehydrationStatus::Pending.is_complete());
        assert!(!DehydrationStatus::Pending.is_in_progress());

        assert!(DehydrationStatus::ComputingRoot.is_in_progress());

        let commit_ref = LoamCommitRef {
            spine_id: "test".to_string(),
            entry_hash: [0u8; 32],
            index: 1,
        };
        assert!(DehydrationStatus::Completed {
            commit_ref
        }
        .is_complete());

        assert!(DehydrationStatus::Failed {
            error: "test".to_string()
        }
        .is_failed());
    }

    #[test]
    fn test_dehydration_config() {
        let minimal = DehydrationConfig::minimal();
        assert!(!minimal.include_vertices);
        assert!(!minimal.include_payloads);

        let full = DehydrationConfig::full();
        assert!(full.include_vertices);
        assert!(full.include_payloads);
    }

    #[test]
    fn test_attestation_statement() {
        let stmt = AttestationStatement::SessionSummary {
            summary_hash: [1u8; 32],
        };

        if let AttestationStatement::SessionSummary {
            summary_hash,
        } = stmt
        {
            assert_eq!(summary_hash, [1u8; 32]);
        } else {
            panic!("Expected SessionSummary");
        }
    }

    #[test]
    fn test_has_required_attestations() {
        let session_id = SessionId::now();
        let merkle_root = MerkleRoot::ZERO;
        let created_at = Timestamp::now();

        let attester = Did::new("did:key:attester");
        let attestation = Attestation {
            attester: attester.clone(),
            statement: AttestationStatement::SessionSummary {
                summary_hash: [0u8; 32],
            },
            signature: vec![],
            attested_at: Timestamp::now(),
            verified: true,
        };

        let summary = DehydrationSummaryBuilder::new(session_id, "test", created_at, merkle_root)
            .with_attestation(attestation)
            .build();

        assert!(summary.has_required_attestations(&[attester]));
        assert!(!summary.has_required_attestations(&[Did::new("did:key:other")]));
    }
}
