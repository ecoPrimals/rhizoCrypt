// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2024–2026 ecoPrimals Project

//! Branch, diff, merge, and federate operation implementations for tarpc.
//!
//! Extracted from `service.rs` to keep production modules under 700L.

use crate::service::{sign_vertex_if_available, RhizoCryptRpcServer};
use crate::service_types::{
    BranchRequest, BranchResponse, DiffRequest, DiffResponse, FederateRequest, FederateResponse,
    MergeRequest,
};
use crate::error::RpcError;
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
        let mut builder =
            VertexBuilder::new(request.event_type).with_parents(request.parents.clone());
        if let Some(agent) = request.agent {
            builder = builder.with_agent(agent);
        }
        for (k, v) in &request.metadata {
            builder = builder.with_metadata(k.clone(), v.clone());
        }
        let mut vertex = builder.build();
        sign_vertex_if_available(&self.primal, &mut vertex).await;

        self.primal
            .merge_branches(request.session_id, request.parents, vertex)
            .await
            .map_err(RpcError::from)
    }

    pub(crate) async fn impl_federate(
        &self,
        request: FederateRequest,
    ) -> Result<FederateResponse, RpcError> {
        let (imported, skipped, frontier) = self
            .primal
            .federate_vertices(request.session_id, request.vertices)
            .await
            .map_err(RpcError::from)?;

        Ok(FederateResponse {
            imported,
            skipped,
            frontier,
        })
    }
}
