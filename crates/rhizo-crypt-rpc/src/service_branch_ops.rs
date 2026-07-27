// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2024–2026 ecoPrimals Project

//! Branch, diff, merge, and federate operation implementations for tarpc.
//!
//! Extracted from `service.rs` to keep production modules under 700L.

use crate::error::RpcError;
use crate::service::{RhizoCryptRpcServer, sign_vertex_if_available};
use crate::service_types::{
    BranchRequest, BranchResponse, DiffRequest, DiffResponse, FederateRequest, FederateResponse,
    MergeRequest,
};
use rhizo_crypt_core::{VertexBuilder, VertexId};

impl RhizoCryptRpcServer {
    pub(crate) async fn impl_branch_session(
        &self,
        request: BranchRequest,
    ) -> Result<BranchResponse, RpcError> {
        let parent_session_id = request.session_id;
        let (session_id, vertex_count) = self
            .primal
            .branch_session(parent_session_id, request.checkout_vertex, request.name)
            .await
            .map_err(RpcError::from)?;

        Ok(BranchResponse {
            session_id,
            vertex_count,
            parent_session_id,
        })
    }

    pub(crate) async fn impl_diff_sessions(
        &self,
        request: DiffRequest,
    ) -> Result<DiffResponse, RpcError> {
        let (only_in_base, only_in_other, common_count) = self
            .primal
            .diff_sessions(request.base_session_id, request.other_session_id)
            .await
            .map_err(RpcError::from)?;

        Ok(DiffResponse {
            only_in_base,
            only_in_other,
            common_count,
        })
    }

    pub(crate) async fn impl_merge_branches(
        &self,
        request: MergeRequest,
    ) -> Result<VertexId, RpcError> {
        let MergeRequest {
            session_id,
            parents,
            event_type,
            agent,
            metadata,
        } = request;

        let mut builder = VertexBuilder::new(event_type).with_parents(parents.clone());
        if let Some(agent) = agent {
            builder = builder.with_agent(agent);
        }
        for (k, v) in metadata {
            builder = builder.with_metadata(k, v);
        }
        let mut vertex = builder.build();
        sign_vertex_if_available(&self.primal, &mut vertex).await;

        self.primal.merge_branches(session_id, parents, vertex).await.map_err(RpcError::from)
    }

    pub(crate) async fn impl_federate(
        &self,
        request: FederateRequest,
    ) -> Result<FederateResponse, RpcError> {
        let signing_client = if request.verify_signatures {
            self.primal.signing_client().await
        } else {
            None
        };

        let mut accepted = Vec::with_capacity(request.vertices.len());
        let mut rejected: u64 = 0;

        for mut vertex in request.vertices {
            if request.verify_signatures
                && let Some(client) = &signing_client
                && vertex.signature.is_some()
            {
                match client.verify_vertex(&vertex).await {
                    Ok(true) => {}
                    Ok(false) => {
                        tracing::warn!("federate: vertex signature invalid — rejecting");
                        rejected += 1;
                        continue;
                    }
                    Err(e) => {
                        tracing::warn!(
                            error = %e,
                            "federate: signature verification failed — rejecting"
                        );
                        rejected += 1;
                        continue;
                    }
                }
            }

            if let Some(gate) = &request.source_gate {
                vertex.metadata.insert(
                    "source_gate".into(),
                    rhizo_crypt_core::vertex::MetadataValue::from(gate.as_str()),
                );
            }

            accepted.push(vertex);
        }

        let (imported, skipped, frontier) = self
            .primal
            .federate_vertices(request.session_id, accepted)
            .await
            .map_err(RpcError::from)?;

        Ok(FederateResponse {
            imported,
            skipped,
            rejected,
            frontier,
        })
    }
}
