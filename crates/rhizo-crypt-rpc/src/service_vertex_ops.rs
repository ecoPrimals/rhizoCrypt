// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2024–2026 ecoPrimals Project

//! Vertex and event operation implementations for tarpc `RhizoCryptRpc`.
//!
//! Extracted from `service.rs` to keep production modules under 700L.
//! Included via `#[path]` in `service.rs`.

use crate::error::RpcError;
use crate::service::{RhizoCryptRpcServer, parse_payload_ref, sign_vertex_if_available};
use crate::service_types::{
    AppendEventRequest, BatchDehydrateResult, PipelineIngestRequest, PipelineIngestResponse,
    QueryRequest,
};
use rhizo_crypt_core::{
    DagStore, MerkleProof, MerkleRoot, SessionBuilder, SessionId, SessionTreeHash, Vertex,
    VertexBuilder, VertexId,
};

impl RhizoCryptRpcServer {
    pub(crate) async fn impl_append_event(
        &self,
        request: AppendEventRequest,
    ) -> Result<VertexId, RpcError> {
        let mut builder = VertexBuilder::new(request.event_type);

        if let Some(agent) = request.agent {
            builder = builder.with_agent(agent);
        }

        for parent in request.parents {
            builder = builder.with_parent(parent);
        }

        for (key, value) in request.metadata {
            builder = builder.with_metadata(key, value);
        }

        if let Some(payload) = request.payload_ref.and_then(|r| parse_payload_ref(&r)) {
            builder = builder.with_payload(payload);
        }

        let mut vertex = builder.build();
        sign_vertex_if_available(&self.primal, &mut vertex).await;
        self.primal.append_vertex(request.session_id, vertex).await.map_err(RpcError::from)
    }

    pub(crate) async fn impl_append_batch(
        &self,
        requests: Vec<AppendEventRequest>,
    ) -> Result<Vec<VertexId>, RpcError> {
        if requests.is_empty() {
            return Ok(Vec::new());
        }

        let all_same_session = requests.windows(2).all(|w| w[0].session_id == w[1].session_id);

        if all_same_session {
            let session_id = requests[0].session_id;
            let mut vertices = Vec::with_capacity(requests.len());
            for request in requests {
                let mut builder = VertexBuilder::new(request.event_type);
                if let Some(agent) = request.agent {
                    builder = builder.with_agent(agent);
                }
                for parent in request.parents {
                    builder = builder.with_parent(parent);
                }
                for (key, value) in request.metadata {
                    builder = builder.with_metadata(key, value);
                }
                if let Some(payload) = request.payload_ref.and_then(|r| parse_payload_ref(&r)) {
                    builder = builder.with_payload(payload);
                }
                let mut vertex = builder.build();
                sign_vertex_if_available(&self.primal, &mut vertex).await;
                vertices.push(vertex);
            }
            return self
                .primal
                .append_vertices_batch(session_id, vertices)
                .await
                .map_err(RpcError::from);
        }

        let mut results = Vec::with_capacity(requests.len());
        for request in requests {
            let mut builder = VertexBuilder::new(request.event_type);
            if let Some(agent) = request.agent {
                builder = builder.with_agent(agent);
            }
            for parent in request.parents {
                builder = builder.with_parent(parent);
            }
            for (key, value) in request.metadata {
                builder = builder.with_metadata(key, value);
            }
            if let Some(payload) = request.payload_ref.and_then(|r| parse_payload_ref(&r)) {
                builder = builder.with_payload(payload);
            }
            let mut vertex = builder.build();
            sign_vertex_if_available(&self.primal, &mut vertex).await;
            let id = self
                .primal
                .append_vertex(request.session_id, vertex)
                .await
                .map_err(RpcError::from)?;
            results.push(id);
        }
        Ok(results)
    }

    pub(crate) async fn impl_get_vertex(
        &self,
        session_id: SessionId,
        vertex_id: VertexId,
    ) -> Result<Vertex, RpcError> {
        tracing::debug!("get_vertex called: session={session_id:?}, vertex={vertex_id:?}");
        let result = self.primal.get_vertex(session_id, vertex_id).await;
        tracing::debug!("get_vertex result: {result:?}");
        result.map_err(RpcError::from)
    }

    pub(crate) fn impl_get_frontier(
        &self,
        session_id: SessionId,
    ) -> Result<Vec<VertexId>, RpcError> {
        let session = self.primal.get_session(session_id).map_err(RpcError::from)?;
        Ok(session.frontier.into_iter().collect())
    }

    pub(crate) fn impl_get_genesis(
        &self,
        session_id: SessionId,
    ) -> Result<Vec<VertexId>, RpcError> {
        let session = self.primal.get_session(session_id).map_err(RpcError::from)?;
        Ok(session.genesis.into_iter().collect())
    }

    pub(crate) async fn impl_query_vertices(
        &self,
        request: QueryRequest,
    ) -> Result<Vec<Vertex>, RpcError> {
        let event_types = request.event_types;
        let agent = request.agent;
        let limit = request.limit.map(|l| usize::try_from(l).unwrap_or(usize::MAX));

        self.primal
            .query_vertices(request.session_id, event_types.as_deref(), agent.as_ref(), limit)
            .await
            .map_err(RpcError::from)
    }

    pub(crate) async fn impl_get_children(
        &self,
        session_id: SessionId,
        vertex_id: VertexId,
    ) -> Result<Vec<VertexId>, RpcError> {
        let dag_store = self.primal.dag_store().await.map_err(RpcError::from)?;
        dag_store.get_children(session_id, vertex_id).await.map_err(RpcError::from)
    }

    pub(crate) async fn impl_get_merkle_root(
        &self,
        session_id: SessionId,
    ) -> Result<MerkleRoot, RpcError> {
        self.primal.compute_merkle_root(session_id).await.map_err(RpcError::from)
    }

    pub(crate) async fn impl_session_tree_hash(
        &self,
        session_id: SessionId,
    ) -> Result<SessionTreeHash, RpcError> {
        self.primal.session_tree_hash(session_id).await.map_err(RpcError::from)
    }

    pub(crate) async fn impl_get_merkle_proof(
        &self,
        session_id: SessionId,
        vertex_id: VertexId,
    ) -> Result<MerkleProof, RpcError> {
        self.primal.generate_merkle_proof(session_id, vertex_id).await.map_err(RpcError::from)
    }

    pub(crate) async fn impl_verify_proof(&self, proof: MerkleProof) -> Result<bool, RpcError> {
        let session_id = self
            .primal
            .session_for_vertex(proof.vertex_id)
            .ok_or_else(|| RpcError::VertexNotFound(proof.vertex_id.to_string()))?;

        let vertex =
            self.primal.get_vertex(session_id, proof.vertex_id).await.map_err(RpcError::from)?;

        Ok(proof.verify(&vertex))
    }

    pub(crate) async fn impl_dehydrate_batch(
        &self,
        session_ids: Vec<SessionId>,
    ) -> Result<Vec<BatchDehydrateResult>, RpcError> {
        let pairs = self.primal.dehydrate_batch(session_ids).await;

        Ok(pairs
            .into_iter()
            .map(|(session_id, result)| match result {
                Ok(root) => BatchDehydrateResult {
                    session_id,
                    merkle_root: Some(hex::encode(root.0)),
                    success: true,
                    error: None,
                },
                Err(e) => BatchDehydrateResult {
                    session_id,
                    merkle_root: None,
                    success: false,
                    error: Some(e.to_string()),
                },
            })
            .collect())
    }

    pub(crate) async fn impl_pipeline_ingest(
        &self,
        request: PipelineIngestRequest,
    ) -> Result<PipelineIngestResponse, RpcError> {
        let mut builder = SessionBuilder::new(request.session_type);
        if let Some(desc) = request.description {
            builder = builder.with_name(desc);
        }
        let session = builder.build();
        let session_id = self.primal.create_session(session).map_err(RpcError::from)?;

        let mut vertices = Vec::with_capacity(request.events.len());
        for event in request.events {
            let mut vb = VertexBuilder::new(event.event_type);
            if let Some(agent) = event.agent {
                vb = vb.with_agent(agent);
            }
            for parent in event.parents {
                vb = vb.with_parent(parent);
            }
            for (key, value) in event.metadata {
                vb = vb.with_metadata(key, value);
            }
            if let Some(payload) = event.payload_ref.and_then(|r| parse_payload_ref(&r)) {
                vb = vb.with_payload(payload);
            }
            let mut vertex = vb.build();
            sign_vertex_if_available(&self.primal, &mut vertex).await;
            vertices.push(vertex);
        }

        let vertex_ids = self
            .primal
            .append_vertices_batch(session_id, vertices)
            .await
            .map_err(RpcError::from)?;

        let appended = vertex_ids.len() as u64;

        let merkle_root = if request.dehydrate {
            match self.primal.dehydrate(session_id).await {
                Ok(root) => Some(hex::encode(root.0)),
                Err(e) => {
                    tracing::warn!(
                        error = %e,
                        %session_id,
                        "pipeline_ingest: dehydration failed (vertices still committed)"
                    );
                    None
                }
            }
        } else {
            None
        };

        Ok(PipelineIngestResponse {
            session_id,
            vertex_ids,
            merkle_root,
            appended,
        })
    }
}
