// store.rs - In-memory DAG storage
//
// The VertexStore is the core storage engine for RhizoCrypt.
// It provides:
// - Fast lookups by VertexId (HashMap)
// - Session isolation
// - DAG traversal operations
// - Garbage collection support
//
// Future: This can be backed by persistent storage (NestGate) or
// distributed across multiple nodes.

use crate::session::{Session, SessionId};
use crate::vertex::{Vertex, VertexId};
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, RwLock};

/// In-memory vertex storage.
///
/// Thread-safe via `Arc<RwLock<...>>` for concurrent access.
#[derive(Debug, Clone)]
pub struct VertexStore {
    /// Vertex storage (content-addressed).
    vertices: Arc<RwLock<HashMap<VertexId, Vertex>>>,

    /// Session storage.
    sessions: Arc<RwLock<HashMap<SessionId, Session>>>,
}

impl VertexStore {
    /// Create a new empty vertex store.
    #[must_use]
    pub fn new() -> Self {
        Self {
            vertices: Arc::new(RwLock::new(HashMap::new())),
            sessions: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Insert a vertex into the store.
    ///
    /// If a vertex with the same ID already exists, it is not overwritten
    /// (content-addressed deduplication).
    ///
    /// Returns `true` if the vertex was inserted, `false` if it already existed.
    pub fn insert_vertex(&self, vertex: Vertex) -> Result<bool, StoreError> {
        let mut vertices = self.vertices.write().map_err(|_| StoreError::LockPoisoned)?;

        if vertices.contains_key(&vertex.id) {
            Ok(false) // Already exists
        } else {
            vertices.insert(vertex.id, vertex);
            Ok(true) // Inserted
        }
    }

    /// Get a vertex by ID.
    pub fn get_vertex(&self, id: &VertexId) -> Result<Option<Vertex>, StoreError> {
        let vertices = self.vertices.read().map_err(|_| StoreError::LockPoisoned)?;
        Ok(vertices.get(id).cloned())
    }

    /// Check if a vertex exists.
    pub fn contains_vertex(&self, id: &VertexId) -> Result<bool, StoreError> {
        let vertices = self.vertices.read().map_err(|_| StoreError::LockPoisoned)?;
        Ok(vertices.contains_key(id))
    }

    /// Get all vertices in a session.
    pub fn get_session_vertices(&self, session_id: &SessionId) -> Result<Vec<Vertex>, StoreError> {
        let vertices = self.vertices.read().map_err(|_| StoreError::LockPoisoned)?;

        let session_vertices: Vec<Vertex> = vertices
            .values()
            .filter(|v| v.data.session_id == *session_id)
            .cloned()
            .collect();

        Ok(session_vertices)
    }

    /// Get the children of a vertex (vertices that have this as a parent).
    pub fn get_children(&self, vertex_id: &VertexId) -> Result<Vec<Vertex>, StoreError> {
        let vertices = self.vertices.read().map_err(|_| StoreError::LockPoisoned)?;

        let children: Vec<Vertex> = vertices
            .values()
            .filter(|v| v.data.parents.contains(vertex_id))
            .cloned()
            .collect();

        Ok(children)
    }

    /// Get the parents of a vertex.
    pub fn get_parents(&self, vertex: &Vertex) -> Result<Vec<Vertex>, StoreError> {
        let vertices = self.vertices.read().map_err(|_| StoreError::LockPoisoned)?;

        let parents: Vec<Vertex> = vertex
            .data
            .parents
            .iter()
            .filter_map(|parent_id| vertices.get(parent_id).cloned())
            .collect();

        Ok(parents)
    }

    /// Topologically sort vertices in a session (parents before children).
    pub fn topological_sort(&self, session_id: &SessionId) -> Result<Vec<Vertex>, StoreError> {
        let session_vertices = self.get_session_vertices(session_id)?;

        let mut sorted = Vec::new();
        let mut visited = HashSet::new();
        let mut visiting = HashSet::new();

        // Build vertex map for fast lookup
        let vertex_map: HashMap<VertexId, Vertex> = session_vertices
            .iter()
            .map(|v| (v.id, v.clone()))
            .collect();

        // Depth-first search with cycle detection
        fn dfs(
            vertex: &Vertex,
            vertex_map: &HashMap<VertexId, Vertex>,
            visited: &mut HashSet<VertexId>,
            visiting: &mut HashSet<VertexId>,
            sorted: &mut Vec<Vertex>,
        ) -> Result<(), StoreError> {
            if visited.contains(&vertex.id) {
                return Ok(());
            }

            if visiting.contains(&vertex.id) {
                return Err(StoreError::CycleDetected);
            }

            visiting.insert(vertex.id);

            // Visit parents first
            for parent_id in &vertex.data.parents {
                if let Some(parent) = vertex_map.get(parent_id) {
                    dfs(parent, vertex_map, visited, visiting, sorted)?;
                }
            }

            visiting.remove(&vertex.id);
            visited.insert(vertex.id);
            sorted.push(vertex.clone());

            Ok(())
        }

        // Process all vertices
        for vertex in &session_vertices {
            dfs(vertex, &vertex_map, &mut visited, &mut visiting, &mut sorted)?;
        }

        Ok(sorted)
    }

    /// Count vertices in the store.
    pub fn vertex_count(&self) -> Result<usize, StoreError> {
        let vertices = self.vertices.read().map_err(|_| StoreError::LockPoisoned)?;
        Ok(vertices.len())
    }

    /// Count sessions in the store.
    pub fn session_count(&self) -> Result<usize, StoreError> {
        let sessions = self.sessions.read().map_err(|_| StoreError::LockPoisoned)?;
        Ok(sessions.len())
    }

    // === Session Management ===

    /// Insert a session into the store.
    pub fn insert_session(&self, session: Session) -> Result<(), StoreError> {
        let mut sessions = self.sessions.write().map_err(|_| StoreError::LockPoisoned)?;
        sessions.insert(session.id, session);
        Ok(())
    }

    /// Get a session by ID.
    pub fn get_session(&self, id: &SessionId) -> Result<Option<Session>, StoreError> {
        let sessions = self.sessions.read().map_err(|_| StoreError::LockPoisoned)?;
        Ok(sessions.get(id).cloned())
    }

    /// Update a session.
    pub fn update_session(&self, session: Session) -> Result<(), StoreError> {
        let mut sessions = self.sessions.write().map_err(|_| StoreError::LockPoisoned)?;

        if !sessions.contains_key(&session.id) {
            return Err(StoreError::SessionNotFound(session.id));
        }

        sessions.insert(session.id, session);
        Ok(())
    }

    /// Remove a session from the store.
    pub fn remove_session(&self, id: &SessionId) -> Result<Option<Session>, StoreError> {
        let mut sessions = self.sessions.write().map_err(|_| StoreError::LockPoisoned)?;
        Ok(sessions.remove(id))
    }

    /// Get all sessions.
    pub fn get_all_sessions(&self) -> Result<Vec<Session>, StoreError> {
        let sessions = self.sessions.read().map_err(|_| StoreError::LockPoisoned)?;
        Ok(sessions.values().cloned().collect())
    }

    /// Garbage collect: Remove vertices from expired sessions.
    pub fn gc_expired_sessions(&self) -> Result<usize, StoreError> {
        let expired: Vec<SessionId> = {
            let sessions = self.sessions.read().map_err(|_| StoreError::LockPoisoned)?;
            sessions
                .values()
                .filter(|s| s.is_expired())
                .map(|s| s.id)
                .collect()
        };

        let mut removed_vertices = 0;

        for session_id in &expired {
            // Get all vertices in this session
            let session_vertices = self.get_session_vertices(session_id)?;

            // Remove them
            let mut vertices = self.vertices.write().map_err(|_| StoreError::LockPoisoned)?;
            for vertex in session_vertices {
                vertices.remove(&vertex.id);
                removed_vertices += 1;
            }

            // Remove the session
            self.remove_session(session_id)?;
        }

        Ok(removed_vertices)
    }
}

impl Default for VertexStore {
    fn default() -> Self {
        Self::new()
    }
}

/// Errors that can occur in the vertex store.
#[derive(Debug, thiserror::Error)]
pub enum StoreError {
    /// Lock is poisoned (should never happen in normal operation).
    #[error("lock poisoned")]
    LockPoisoned,

    /// Cycle detected in DAG (invalid).
    #[error("cycle detected in DAG")]
    CycleDetected,

    /// Session not found.
    #[error("session not found: {0}")]
    SessionNotFound(SessionId),
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::session::SessionMetadata;

    #[test]
    fn test_insert_and_get_vertex() {
        let store = VertexStore::new();
        let session_id = SessionId::new();

        let vertex = Vertex::genesis(session_id, "test", serde_json::json!({})).unwrap();

        let inserted = store.insert_vertex(vertex.clone()).unwrap();
        assert!(inserted);

        let retrieved = store.get_vertex(&vertex.id).unwrap();
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().id, vertex.id);
    }

    #[test]
    fn test_deduplication() {
        let store = VertexStore::new();
        let session_id = SessionId::new();

        let vertex = Vertex::genesis(session_id, "test", serde_json::json!({})).unwrap();

        let first_insert = store.insert_vertex(vertex.clone()).unwrap();
        let second_insert = store.insert_vertex(vertex).unwrap();

        assert!(first_insert); // First insert succeeded
        assert!(!second_insert); // Second insert was deduplicated
        assert_eq!(store.vertex_count().unwrap(), 1);
    }

    #[test]
    fn test_get_session_vertices() {
        let store = VertexStore::new();

        let session1 = SessionId::new();
        let session2 = SessionId::new();

        let v1 = Vertex::genesis(session1, "test", serde_json::json!({})).unwrap();
        let v2 = Vertex::genesis(session2, "test", serde_json::json!({})).unwrap();

        store.insert_vertex(v1).unwrap();
        store.insert_vertex(v2).unwrap();

        let session1_vertices = store.get_session_vertices(&session1).unwrap();
        assert_eq!(session1_vertices.len(), 1);
    }

    #[test]
    fn test_get_children() {
        let store = VertexStore::new();
        let session_id = SessionId::new();

        let parent = Vertex::genesis(session_id, "parent", serde_json::json!({})).unwrap();
        let child = Vertex::with_parent(parent.id, session_id, "child", serde_json::json!({})).unwrap();

        store.insert_vertex(parent.clone()).unwrap();
        store.insert_vertex(child.clone()).unwrap();

        let children = store.get_children(&parent.id).unwrap();
        assert_eq!(children.len(), 1);
        assert_eq!(children[0].id, child.id);
    }

    #[test]
    fn test_topological_sort() {
        let store = VertexStore::new();
        let session_id = SessionId::new();

        let v1 = Vertex::genesis(session_id, "1", serde_json::json!({})).unwrap();
        let v2 = Vertex::with_parent(v1.id, session_id, "2", serde_json::json!({})).unwrap();
        let v3 = Vertex::with_parent(v2.id, session_id, "3", serde_json::json!({})).unwrap();

        store.insert_vertex(v3.clone()).unwrap(); // Insert out of order
        store.insert_vertex(v1.clone()).unwrap();
        store.insert_vertex(v2.clone()).unwrap();

        let sorted = store.topological_sort(&session_id).unwrap();

        assert_eq!(sorted.len(), 3);
        // Parents should come before children
        let v1_pos = sorted.iter().position(|v| v.id == v1.id).unwrap();
        let v2_pos = sorted.iter().position(|v| v.id == v2.id).unwrap();
        let v3_pos = sorted.iter().position(|v| v.id == v3.id).unwrap();

        assert!(v1_pos < v2_pos);
        assert!(v2_pos < v3_pos);
    }

    #[test]
    fn test_session_management() {
        let store = VertexStore::new();
        let session = Session::new(SessionMetadata::default());
        let session_id = session.id;

        store.insert_session(session.clone()).unwrap();

        let retrieved = store.get_session(&session_id).unwrap();
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().id, session_id);

        store.remove_session(&session_id).unwrap();

        let gone = store.get_session(&session_id).unwrap();
        assert!(gone.is_none());
    }
}

