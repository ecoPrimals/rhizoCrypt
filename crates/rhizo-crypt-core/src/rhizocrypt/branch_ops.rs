// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2024–2026 ecoPrimals Project

//! Branch / Diff / Merge / Federate operations (Wave 60).
//!
//! Extracted from `mod.rs` to keep the main `RhizoCrypt` module under
//! the 800-line production threshold. These methods enable version control
//! through DAG composition for rootPulse signal graphs.

use crate::error::{Result, RhizoCryptError};
use crate::store::DagStore;
use crate::types::{SessionId, VertexId};
use crate::vertex::Vertex;
use std::collections::{HashMap, HashSet};

use super::RhizoCrypt;

impl RhizoCrypt {
    /// Create a new session branched from `parent_session_id` at `checkout_vertex`.
    ///
    /// Copies all ancestor vertices (from `checkout_vertex` back to genesis)
    /// into the new session, preserving DAG structure.
    ///
    /// # Errors
    ///
    /// Returns an error if the parent session or checkout vertex is not found,
    /// or the primal is not running.
    pub async fn branch_session(
        &self,
        parent_session_id: SessionId,
        checkout_vertex: VertexId,
        name: Option<String>,
    ) -> Result<(SessionId, u64)> {
        if !self.state.is_running() {
            return Err(RhizoCryptError::internal("primal not running"));
        }

        let parent = self.get_session(parent_session_id)?;
        let dag_store = self.dag_store().await?;

        let all_vertices = dag_store.get_all_vertices(parent_session_id).await?;

        let mut id_map: HashMap<VertexId, &Vertex> = HashMap::new();
        for v in &all_vertices {
            if let Ok(vid) = v.compute_id() {
                id_map.insert(vid, v);
            }
        }

        if !id_map.contains_key(&checkout_vertex) {
            return Err(RhizoCryptError::vertex_not_found(checkout_vertex));
        }

        // Walk backward from checkout_vertex to collect the ancestor subgraph
        let mut to_visit = vec![checkout_vertex];
        let mut visited = HashSet::new();
        while let Some(vid) = to_visit.pop() {
            if !visited.insert(vid) {
                continue;
            }
            if let Some(v) = id_map.get(&vid) {
                for parent_id in &v.parents {
                    if !visited.contains(parent_id) {
                        to_visit.push(*parent_id);
                    }
                }
            }
        }

        let mut builder = crate::session::SessionBuilder::new(parent.session_type.clone());
        if let Some(n) = name {
            builder = builder.with_name(n);
        }
        let new_session = builder.build();
        let new_session_id = self.create_session(new_session)?;

        let mut copied: u64 = 0;
        let ordered: Vec<&Vertex> = all_vertices
            .iter()
            .filter(|v| v.compute_id().is_ok_and(|id| visited.contains(&id)))
            .collect();

        for v in &ordered {
            let cloned = v.clone_for_branch();
            self.append_vertex(new_session_id, cloned).await?;
            copied += 1;
        }

        Ok((new_session_id, copied))
    }

    /// Compute the structural diff between two sessions.
    ///
    /// Returns vertex IDs unique to each session and the count of shared vertices.
    ///
    /// # Errors
    ///
    /// Returns an error if either session is not found.
    pub async fn diff_sessions(
        &self,
        base_session_id: SessionId,
        other_session_id: SessionId,
    ) -> Result<(Vec<VertexId>, Vec<VertexId>, u64)> {
        let base_vertices = self.get_all_vertices(base_session_id).await?;
        let other_vertices = self.get_all_vertices(other_session_id).await?;

        let base_ids: HashSet<VertexId> =
            base_vertices.iter().filter_map(|v| v.compute_id().ok()).collect();
        let other_ids: HashSet<VertexId> =
            other_vertices.iter().filter_map(|v| v.compute_id().ok()).collect();

        let only_in_base: Vec<VertexId> = base_ids.difference(&other_ids).copied().collect();
        let only_in_other: Vec<VertexId> = other_ids.difference(&base_ids).copied().collect();
        let common = base_ids.intersection(&other_ids).count() as u64;

        Ok((only_in_base, only_in_other, common))
    }

    /// Create a merge vertex that joins multiple frontier tips.
    ///
    /// Validates that all parent IDs are current frontier vertices, then
    /// appends a merge vertex with those parents. The frontier collapses
    /// to a single tip.
    ///
    /// # Errors
    ///
    /// Returns an error if the session is not found, not active, or any
    /// parent is not in the current frontier.
    pub async fn merge_branches(
        &self,
        session_id: SessionId,
        parent_ids: Vec<VertexId>,
        merge_vertex: Vertex,
    ) -> Result<VertexId> {
        if parent_ids.len() < 2 {
            return Err(RhizoCryptError::InvalidVertex(
                "merge requires at least 2 parent vertices".into(),
            ));
        }

        let session = self.get_session(session_id)?;
        for pid in &parent_ids {
            if !session.frontier.contains(pid) {
                return Err(RhizoCryptError::InvalidVertex(format!(
                    "parent {pid} is not in the session frontier",
                )));
            }
        }

        self.append_vertex(session_id, merge_vertex).await
    }

    /// Import vertices from a remote peer into a local session (federation).
    ///
    /// Diff-based: vertices whose IDs already exist in the local session are
    /// skipped. New vertices are appended in the order provided (caller should
    /// provide topologically sorted vertices for correct parent relationships).
    ///
    /// # Errors
    ///
    /// Returns an error if the session is not found or not active.
    pub async fn federate_vertices(
        &self,
        session_id: SessionId,
        remote_vertices: Vec<Vertex>,
    ) -> Result<(u64, u64, Vec<VertexId>)> {
        if !self.state.is_running() {
            return Err(RhizoCryptError::internal("primal not running"));
        }

        let _ = self.get_session(session_id)?;
        let dag_store = self.dag_store().await?;

        let mut imported: u64 = 0;
        let mut skipped: u64 = 0;

        for vertex in remote_vertices {
            let vid = vertex.compute_id()?;
            if dag_store.exists(session_id, vid).await? {
                skipped += 1;
                continue;
            }
            self.append_vertex(session_id, vertex).await?;
            imported += 1;
        }

        let session = self.get_session(session_id)?;
        let frontier: Vec<VertexId> = session.frontier.into_iter().collect();

        Ok((imported, skipped, frontier))
    }
}
