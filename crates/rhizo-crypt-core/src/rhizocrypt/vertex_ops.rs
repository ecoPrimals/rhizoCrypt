// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2024–2026 ecoPrimals Project

//! Vertex, query, and Merkle operations for `RhizoCrypt`.
//!
//! Extracted from `mod.rs` to keep production modules under 700L.

use crate::error::{Result, RhizoCryptError};
use crate::event::EventType;
use crate::merkle::{MerkleProof, MerkleRoot};
use crate::store::DagStore;
use crate::types::{Did, SessionId, VertexId};
use crate::vertex::Vertex;

use super::RhizoCrypt;

impl RhizoCrypt {
    // ========================================================================
    // Vertex Operations
    // ========================================================================

    /// Append a vertex to a session (fine-grained locking).
    ///
    /// # Errors
    ///
    /// Returns an error if the session is not found or not active.
    pub async fn append_vertex(&self, session_id: SessionId, vertex: Vertex) -> Result<VertexId> {
        if !self.state.is_running() {
            return Err(RhizoCryptError::internal("primal not running"));
        }

        // Fine-grained lock: only lock this specific session
        let mut session_entry = self
            .sessions
            .get_mut(&session_id)
            .ok_or_else(|| RhizoCryptError::session_not_found(session_id))?;

        let session = session_entry.value_mut();

        if !session.is_active() {
            return Err(RhizoCryptError::internal("session not active"));
        }

        // Track the agent
        if let Some(ref agent) = vertex.agent {
            session.add_agent(agent.clone());
        }

        // Compute ID once, update frontier, then release session lock
        let parents = vertex.parents.clone();
        let mut vertex = vertex;
        let vertex_id = vertex.id()?;
        session.update_frontier(vertex_id, &parents);

        // Release session lock before expensive DAG operation
        drop(session_entry);

        // Store the vertex and index it for O(1) lookup
        let dag_store = self.dag_store().await?;
        dag_store.put_vertex(session_id, vertex).await?;
        self.vertex_session_index.insert(vertex_id, session_id);
        self.metrics.inc_vertices_appended();

        Ok(vertex_id)
    }

    /// Get a vertex by ID.
    ///
    /// # Errors
    ///
    /// Returns an error if the vertex is not found.
    pub async fn get_vertex(&self, session_id: SessionId, vertex_id: VertexId) -> Result<Vertex> {
        let dag_store = self.dag_store().await?;
        dag_store
            .get_vertex(session_id, vertex_id)
            .await?
            .ok_or_else(|| RhizoCryptError::vertex_not_found(vertex_id))
    }

    /// Get all vertices for a session in topological order.
    ///
    /// # Errors
    ///
    /// Returns an error if primal not running.
    pub async fn get_all_vertices(&self, session_id: SessionId) -> Result<Vec<Vertex>> {
        let dag_store = self.dag_store().await?;
        dag_store.get_all_vertices(session_id).await
    }

    /// Query vertices with filters.
    ///
    /// # Errors
    ///
    /// Returns an error if primal not running.
    pub async fn query_vertices(
        &self,
        session_id: SessionId,
        event_types: Option<&[EventType]>,
        agent: Option<&Did>,
        limit: Option<usize>,
    ) -> Result<Vec<Vertex>> {
        let vertices = self.get_all_vertices(session_id).await?;

        let filtered: Vec<Vertex> = vertices
            .into_iter()
            .filter(|v| {
                // Filter by event type
                if let Some(types) = event_types
                    && !types.contains(&v.event_type)
                {
                    return false;
                }
                if let Some(a) = agent
                    && v.agent.as_ref() != Some(a)
                {
                    return false;
                }
                true
            })
            .take(limit.unwrap_or(usize::MAX))
            .collect();

        self.metrics.inc_queries_executed();
        Ok(filtered)
    }

    // ========================================================================
    // Merkle Operations
    // ========================================================================

    /// Compute Merkle root for a session.
    ///
    /// # Errors
    ///
    /// Returns an error if the session is not found or primal not running.
    pub async fn compute_merkle_root(&self, session_id: SessionId) -> Result<MerkleRoot> {
        let _ = self.get_session(session_id)?;
        let dag_store = self.dag_store().await?;
        let vertices = dag_store.get_all_vertices(session_id).await?;
        MerkleRoot::compute(&vertices)
    }

    /// Generate Merkle proof for a vertex.
    ///
    /// # Errors
    ///
    /// Returns an error if the session or vertex is not found.
    pub async fn generate_merkle_proof(
        &self,
        session_id: SessionId,
        vertex_id: VertexId,
    ) -> Result<MerkleProof> {
        let dag_store = self.dag_store().await?;
        let vertices = dag_store.get_all_vertices(session_id).await?;

        if vertices.is_empty() {
            return Err(RhizoCryptError::vertex_not_found(vertex_id));
        }

        let root = MerkleRoot::compute(&vertices)?;
        let ids: Vec<VertexId> =
            vertices.iter().map(Vertex::compute_id).collect::<std::result::Result<Vec<_>, _>>()?;
        let position = ids
            .iter()
            .position(|id| *id == vertex_id)
            .ok_or_else(|| RhizoCryptError::vertex_not_found(vertex_id))?;

        MerkleProof::generate(&vertices, position, root)
    }
}
