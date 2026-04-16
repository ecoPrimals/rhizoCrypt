// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2024–2026 ecoPrimals Project

//! Dehydration pipeline methods for [`RhizoCrypt`].
//!
//! Extracted from the main module to keep the core API surface readable.
//! These methods handle the full dehydration lifecycle: root computation,
//! summary generation, attestation collection, and permanent storage commit.

use super::RhizoCrypt;
use crate::dehydration::{self, Attestation, DehydrationConfig, DehydrationSummary};
use crate::error::{Result, RhizoCryptError};
use crate::event::{AgentRole, EventType};
use crate::merkle::MerkleRoot;
use crate::session::CommitRef;
use crate::types::{SessionId, Timestamp};

/// Per-agent accumulator used while walking the session DAG.
struct AgentAccumulator {
    joined_at: Option<Timestamp>,
    left_at: Option<Timestamp>,
    event_count: u64,
    role: AgentRole,
}

impl AgentAccumulator {
    const fn new() -> Self {
        Self {
            joined_at: None,
            left_at: None,
            event_count: 0,
            role: AgentRole::Participant,
        }
    }
}

impl RhizoCrypt {
    /// Start dehydration of a session with full implementation.
    ///
    /// This method:
    /// 1. Computes the Merkle root of the DAG
    /// 2. Generates a dehydration summary
    /// 3. Collects attestations from participants (if required)
    /// 4. Commits to permanent storage via `PermanentStorageProvider`
    ///
    /// # Errors
    ///
    /// Returns an error if session not found, dehydration fails, or commit fails.
    pub async fn dehydrate(&self, session_id: SessionId) -> Result<MerkleRoot> {
        self.dehydration_status.insert(session_id, dehydration::DehydrationStatus::ComputingRoot);

        let root = self.compute_merkle_root(session_id).await?;

        self.dehydration_status
            .insert(session_id, dehydration::DehydrationStatus::GeneratingSummary);

        let summary = self.generate_dehydration_summary(session_id, root).await?;

        let config = DehydrationConfig::default();
        let attestations = if config.required_attestations.is_empty() {
            Vec::new()
        } else {
            self.collect_attestations(session_id, &summary, &config).await
        };

        let mut summary_with_attestations = summary;
        summary_with_attestations.attestations.extend(attestations);

        self.dehydration_status.insert(session_id, dehydration::DehydrationStatus::Committing);

        let commit_ref = self.commit_to_permanent_storage(&summary_with_attestations).await?;

        self.provenance_notifier.notify_dehydration(&summary_with_attestations).await.ok();

        self.dehydration_status.insert(
            session_id,
            dehydration::DehydrationStatus::Completed {
                commit_ref,
            },
        );

        self.metrics.inc_dehydrations_completed();
        Ok(root)
    }

    /// Generate a dehydration summary for a session.
    ///
    /// Walks the session DAG to extract:
    /// - Actual payload byte totals from the payload store
    /// - Frontier vertices as result entries with serialized event types
    /// - Per-agent summaries with roles and event counts from DAG vertices
    async fn generate_dehydration_summary(
        &self,
        session_id: SessionId,
        merkle_root: MerkleRoot,
    ) -> Result<DehydrationSummary> {
        let session = self.get_session(session_id)?;

        let payload_bytes = match self.payload_store().await {
            Ok(store) => u64::try_from(store.total_bytes().await).unwrap_or(u64::MAX),
            Err(_) => {
                session.vertex_count.saturating_mul(crate::constants::ESTIMATED_BYTES_PER_VERTEX)
            }
        };

        let mut results = Vec::new();
        for vertex_id in &session.frontier {
            if let Ok(vertex) = self.get_vertex(session_id, *vertex_id).await {
                let value = serde_json::to_value(&vertex.event_type).unwrap_or_default();
                let result = dehydration::ResultEntry {
                    result_type: vertex.event_type.name().to_string(),
                    key: vertex_id.to_string(),
                    value,
                    source_vertex: *vertex_id,
                    payload_ref: vertex.payload,
                };
                results.push(result);
            }
        }

        let agents = self.build_agent_summaries(session_id, &session).await;

        let session_type_str =
            serde_json::to_string(&session.session_type).unwrap_or_else(|_| "General".to_string());

        let mut summary = dehydration::DehydrationSummaryBuilder::new(
            session_id,
            session_type_str,
            session.created_at,
            merkle_root,
        )
        .with_outcome(crate::event::SessionOutcome::Success)
        .with_vertex_count(session.vertex_count)
        .with_payload_bytes(payload_bytes);

        for result in results {
            summary = summary.with_result(result);
        }
        for agent in agents {
            summary = summary.with_agent(agent);
        }

        Ok(summary.build())
    }

    /// Build per-agent summaries by walking the session DAG for join/leave events.
    async fn build_agent_summaries(
        &self,
        session_id: SessionId,
        session: &crate::session::Session,
    ) -> Vec<dehydration::AgentSummary> {
        let all_vertices =
            self.query_vertices(session_id, None, None, None).await.unwrap_or_default();

        let mut agents: std::collections::HashMap<crate::types::Did, AgentAccumulator> =
            std::collections::HashMap::new();

        for vertex in &all_vertices {
            if let Some(ref agent) = vertex.agent {
                let acc = agents.entry(agent.clone()).or_insert_with(AgentAccumulator::new);
                acc.event_count += 1;

                match &vertex.event_type {
                    EventType::AgentJoin {
                        role,
                    } => {
                        if acc.joined_at.is_none() {
                            acc.joined_at = Some(vertex.timestamp);
                        }
                        acc.role = role.clone();
                    }
                    EventType::AgentLeave {
                        ..
                    } => {
                        acc.left_at = Some(vertex.timestamp);
                    }
                    _ => {}
                }
            }
        }

        for did in &session.agents {
            agents.entry(did.clone()).or_insert_with(|| AgentAccumulator {
                joined_at: Some(session.created_at),
                left_at: None,
                event_count: 0,
                role: AgentRole::Participant,
            });
        }

        agents
            .into_iter()
            .map(|(did, acc)| {
                let role_str = match &acc.role {
                    AgentRole::Owner => "owner",
                    AgentRole::Participant => "participant",
                    AgentRole::Observer => "observer",
                    AgentRole::Custom(s) => s.as_str(),
                };
                dehydration::AgentSummary {
                    agent: did,
                    joined_at: acc.joined_at.unwrap_or(session.created_at),
                    left_at: acc.left_at,
                    event_count: acc.event_count,
                    role: role_str.to_string(),
                }
            })
            .collect()
    }

    /// Collect attestations from session participants via capability-based signing.
    ///
    /// Discovers a `SigningProvider` at runtime through the shared discovery
    /// registry and requests attestations from each required attester. Returns
    /// whatever attestations could be collected within the configured timeout.
    /// If no signing provider is discoverable, returns an empty set
    /// (attestations are optional for standalone deployments).
    async fn collect_attestations(
        &self,
        session_id: SessionId,
        summary: &DehydrationSummary,
        config: &DehydrationConfig,
    ) -> Vec<Attestation> {
        self.dehydration_status.insert(
            session_id,
            dehydration::DehydrationStatus::CollectingAttestations {
                collected: 0,
                required: config.required_attestations.len(),
            },
        );

        let signing_client = match crate::clients::SigningClient::discover(&self.discovery_registry)
            .await
        {
            Ok(client) => client,
            Err(e) => {
                tracing::debug!(error = %e, "No signing provider available, skipping attestations");
                return Vec::new();
            }
        };

        let mut attestations = Vec::new();
        for attester in &config.required_attestations {
            match signing_client.request_attestation(attester, summary).await {
                Ok(attestation) => {
                    attestations.push(attestation);
                    self.dehydration_status.insert(
                        session_id,
                        dehydration::DehydrationStatus::CollectingAttestations {
                            collected: attestations.len(),
                            required: config.required_attestations.len(),
                        },
                    );
                }
                Err(e) => {
                    tracing::warn!(
                        attester = %attester,
                        error = %e,
                        "Failed to collect attestation, continuing"
                    );
                }
            }
        }

        attestations
    }

    /// Commit dehydration summary to permanent storage.
    ///
    /// Uses capability-based discovery — any `PermanentStorageProvider` works.
    /// Falls back to a local reference when no provider is available,
    /// allowing dehydration to complete in standalone deployments.
    async fn commit_to_permanent_storage(&self, summary: &DehydrationSummary) -> Result<CommitRef> {
        use crate::clients::PermanentStorageClient;

        match PermanentStorageClient::discover(&self.discovery_registry).await {
            Ok(client) => {
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
                tracing::warn!(
                    session_id = %summary.session_id,
                    error = %e,
                    "No permanent storage provider available, creating local reference"
                );

                Ok(CommitRef {
                    spine_id: format!("local-{}", summary.session_id),
                    entry_hash: *summary.merkle_root.as_bytes(),
                    index: 0,
                })
            }
        }
    }

    /// Get dehydration status for a session (lock-free).
    #[must_use]
    pub fn get_dehydration_status(&self, session_id: SessionId) -> dehydration::DehydrationStatus {
        self.dehydration_status
            .get(&session_id)
            .map_or(dehydration::DehydrationStatus::Pending, |entry| entry.value().clone())
    }
}
