//! Dehydration implementation for RhizoCrypt.
//!
//! This module handles the complete dehydration workflow:
//! - Summary generation
//! - Attestation collection
//! - Permanent storage commitment

use crate::clients::SigningClient;
use crate::dehydration;
use crate::discovery;
use crate::error::{Result, RhizoCryptError};
use crate::event;
use crate::merkle;
use crate::session;
use crate::types::{SessionId, VertexId};
use crate::RhizoCrypt;

impl RhizoCrypt {
    /// Generate a dehydration summary for a session.
    ///
    /// Extracts key information from the session DAG for permanent storage.
    pub(crate) async fn generate_dehydration_summary(
        &self,
        session_id: SessionId,
        merkle_root: merkle::MerkleRoot,
    ) -> Result<dehydration::DehydrationSummary> {
        // Get session
        let session = self.get_session(session_id)?;

        // Count total payload bytes by iterating vertices
        let payload_bytes = 0u64; // Would need payload store for actual sizes
        let mut results = Vec::new();

        // Collect frontier vertices as results (final outputs)
        for vertex_id in &session.frontier {
            if let Ok(vertex) = self.get_vertex(session_id, *vertex_id).await {
                // Extract result entry from frontier vertex
                let result = dehydration::ResultEntry {
                    result_type: format!("{:?}", vertex.event_type),
                    key: vertex_id.to_string(),
                    value: serde_json::Value::Null, // Would need to fetch from payload store
                    source_vertex: *vertex_id,
                    payload_ref: vertex.payload,
                };
                results.push(result);
            }
        }

        // Build agent summaries
        let agents: Vec<dehydration::AgentSummary> = session
            .agents
            .iter()
            .map(|did| dehydration::AgentSummary {
                agent: did.clone(),
                joined_at: session.created_at,
                left_at: None,
                event_count: 0, // Would track in production
                role: "participant".to_string(),
            })
            .collect();

        // Build summary using builder pattern
        let mut summary = dehydration::DehydrationSummaryBuilder::new(
            session_id,
            format!("{:?}", session.session_type),
            session.created_at,
            merkle_root,
        )
        .with_outcome(event::SessionOutcome::Success)
        .with_vertex_count(session.vertex_count)
        .with_payload_bytes(payload_bytes);

        // Add results
        for result in results {
            summary = summary.with_result(result);
        }

        // Add agents
        for agent in agents {
            summary = summary.with_agent(agent);
        }

        Ok(summary.build())
    }

    /// Collect attestations from session participants.
    ///
    /// Requests cryptographic attestations from required participants using
    /// capability-based SigningProvider discovery. Each attester signs the
    /// dehydration summary hash to attest to the validity of the commitment.
    ///
    /// This implements the multi-party approval pattern for permanent commits.
    pub(crate) async fn collect_attestations(
        &self,
        session_id: SessionId,
        summary: &dehydration::DehydrationSummary,
        config: &dehydration::DehydrationConfig,
    ) -> Result<Vec<dehydration::Attestation>> {
        // If no attestations required, return early
        if config.required_attestations.is_empty() {
            return Ok(Vec::new());
        }

        // Update status
        self.dehydration_status.insert(
            session_id,
            dehydration::DehydrationStatus::CollectingAttestations {
                collected: 0,
                required: config.required_attestations.len(),
            },
        );

        // Compute hash of summary for signing
        let summary_hash = summary.compute_hash();
        let summary_bytes: &[u8] = &summary_hash;

        // Create discovery registry
        let registry = discovery::DiscoveryRegistry::new("rhizoCrypt");

        // Try to discover signing provider (capability-based!)
        let signing_client = match SigningClient::discover(&registry).await {
            Ok(client) => client,
            Err(e) => {
                tracing::warn!(
                    error = %e,
                    "No SigningProvider available for attestations"
                );
                // If attestations are required but no signer available, that's an error
                if config.require_all_attestations {
                    return Err(RhizoCryptError::integration(
                        "Attestations required but no SigningProvider available".to_string(),
                    ));
                }
                // Otherwise, proceed without attestations
                return Ok(Vec::new());
            }
        };

        // Collect attestations sequentially (simpler than concurrent for now)
        let mut attestations = Vec::new();

        for attester_did in &config.required_attestations {
            // Request signature from attester
            match signing_client.sign(summary_bytes, attester_did).await {
                Ok(signature) => {
                    // Verify the signature immediately
                    match signing_client.verify(summary_bytes, &signature, attester_did).await {
                        Ok(true) => {
                            let attestation = dehydration::Attestation {
                                attester: attester_did.clone(),
                                statement: dehydration::AttestationStatement::SessionSummary {
                                    summary_hash,
                                },
                                signature: signature.as_bytes().to_vec(),
                                attested_at: crate::types::Timestamp::now(),
                                verified: true,
                            };
                            tracing::info!(
                                attester = %attester_did,
                                "Attestation received and verified"
                            );
                            attestations.push(attestation);
                        }
                        Ok(false) => {
                            tracing::warn!(
                                attester = %attester_did,
                                "Attestation signature verification failed"
                            );
                        }
                        Err(e) => {
                            tracing::warn!(
                                attester = %attester_did,
                                error = %e,
                                "Failed to verify attestation"
                            );
                        }
                    }
                }
                Err(e) => {
                    tracing::warn!(
                        attester = %attester_did,
                        error = %e,
                        "Failed to request attestation signature"
                    );
                }
            }
        }

        // Update status with collected count
        self.dehydration_status.insert(
            session_id,
            dehydration::DehydrationStatus::CollectingAttestations {
                collected: attestations.len(),
                required: config.required_attestations.len(),
            },
        );

        // Check if we have enough attestations
        if config.require_all_attestations && attestations.len() < config.required_attestations.len()
        {
            return Err(RhizoCryptError::integration(format!(
                "Insufficient attestations: got {}, required {}",
                attestations.len(),
                config.required_attestations.len()
            )));
        }

        tracing::info!(
            collected = attestations.len(),
            required = config.required_attestations.len(),
            "Attestation collection complete"
        );

        Ok(attestations)
    }

    /// Commit dehydration summary to permanent storage.
    ///
    /// Uses capability-based discovery to find PermanentStorageProvider.
    /// Any provider that implements the capability can be used (LoamSpine, IPFS, Arweave, etc.)
    pub(crate) async fn commit_to_permanent_storage(
        &self,
        summary: &dehydration::DehydrationSummary,
    ) -> Result<session::LoamCommitRef> {
        // Create a discovery registry for capability-based lookup
        let registry = discovery::DiscoveryRegistry::new("rhizoCrypt");

        // Try to discover permanent storage provider
        use crate::clients::PermanentStorageClient;
        match PermanentStorageClient::discover(&registry).await {
            Ok(client) => {
                // Commit via discovered provider (capability-based!)
                tracing::info!(
                    session_id = %summary.session_id,
                    merkle_root = %summary.merkle_root,
                    vertex_count = summary.vertex_count,
                    "Committing dehydration to permanent storage"
                );

                client.commit(summary).await.map_err(|e| {
                    RhizoCryptError::integration(format!(
                        "Failed to commit to permanent storage: {e}"
                    ))
                })
            }
            Err(e) => {
                // No permanent storage available - create local reference
                // This allows dehydration to complete even without LoamSpine
                tracing::warn!(
                    session_id = %summary.session_id,
                    error = %e,
                    "No permanent storage provider available, creating local reference"
                );

                Ok(session::LoamCommitRef {
                    spine_id: format!("local-{}", summary.session_id),
                    entry_hash: *summary.merkle_root.as_bytes(),
                    index: 0,
                })
            }
        }
    }

    /// Get dehydration status for a session.
    #[allow(clippy::unused_async)]
    pub async fn get_dehydration_status(
        &self,
        session_id: SessionId,
    ) -> dehydration::DehydrationStatus {
        self.dehydration_status
            .get(&session_id)
            .map_or(dehydration::DehydrationStatus::Pending, |entry| {
                entry.value().clone()
            })
    }
}

