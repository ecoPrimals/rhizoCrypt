// session.rs - Session management for scoped DAGs
//
// Sessions provide isolation and lifecycle management for ephemeral DAGs.
// Each session is a self-contained workspace that can be:
// - Created (new session ID)
// - Appended to (add vertices)
// - Resolved (success/failure/timeout)
// - Dehydrated (committed to LoamSpine)
// - Expired (garbage collected)

use crate::vertex::{VertexId};
pub use crate::vertex::SessionId;
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// A session - a scoped, ephemeral DAG workspace.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    /// Unique session identifier.
    pub id: SessionId,

    /// Session metadata (name, description, tags).
    pub metadata: SessionMetadata,

    /// Session state (active, resolved, expired).
    pub state: SessionState,

    /// When the session was created.
    pub created_at: DateTime<Utc>,

    /// When the session was last modified.
    pub updated_at: DateTime<Utc>,

    /// When the session expires (if set).
    pub expires_at: Option<DateTime<Utc>>,

    /// The genesis vertex(s) of this session.
    /// 
    /// Most sessions have one genesis, but multiple are allowed.
    pub genesis_vertices: Vec<VertexId>,

    /// The tip vertices of this session (no children yet).
    /// 
    /// These are the "active" vertices where new events can be appended.
    pub tip_vertices: Vec<VertexId>,

    /// The resolution vertices (if resolved).
    /// 
    /// These are the final vertices when the session concludes.
    pub resolution_vertices: Vec<VertexId>,
}

/// Session metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionMetadata {
    /// Human-readable session name.
    pub name: Option<String>,

    /// Session description.
    pub description: Option<String>,

    /// Session creator (DID).
    pub creator: Option<String>, // TODO: Use BearDog DID type

    /// Session tags (for querying/filtering).
    #[serde(default)]
    pub tags: Vec<String>,

    /// Custom metadata fields.
    #[serde(default)]
    pub custom: HashMap<String, serde_json::Value>,
}

/// Session state.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SessionState {
    /// Session is active (accepting new vertices).
    Active,

    /// Session is resolved (successfully completed).
    Resolved {
        /// When the session was resolved.
        resolved_at: DateTime<Utc>,
    },

    /// Session failed (error occurred).
    Failed {
        /// When the session failed.
        failed_at: DateTime<Utc>,
        /// Error message.
        reason: String,
    },

    /// Session expired (timeout).
    Expired {
        /// When the session expired.
        expired_at: DateTime<Utc>,
    },
}

impl Session {
    /// Create a new session.
    #[must_use]
    pub fn new(metadata: SessionMetadata) -> Self {
        let now = Utc::now();
        Self {
            id: SessionId::new(),
            metadata,
            state: SessionState::Active,
            created_at: now,
            updated_at: now,
            expires_at: None,
            genesis_vertices: Vec::new(),
            tip_vertices: Vec::new(),
            resolution_vertices: Vec::new(),
        }
    }

    /// Create a new session with a specific ID.
    #[must_use]
    pub fn with_id(id: SessionId, metadata: SessionMetadata) -> Self {
        let now = Utc::now();
        Self {
            id,
            metadata,
            state: SessionState::Active,
            created_at: now,
            updated_at: now,
            expires_at: None,
            genesis_vertices: Vec::new(),
            tip_vertices: Vec::new(),
            resolution_vertices: Vec::new(),
        }
    }

    /// Create a new session with an expiration time.
    #[must_use]
    pub fn with_expiration(metadata: SessionMetadata, duration: Duration) -> Self {
        let now = Utc::now();
        let expires_at = now + duration;
        Self {
            id: SessionId::new(),
            metadata,
            state: SessionState::Active,
            created_at: now,
            updated_at: now,
            expires_at: Some(expires_at),
            genesis_vertices: Vec::new(),
            tip_vertices: Vec::new(),
            resolution_vertices: Vec::new(),
        }
    }

    /// Add a genesis vertex to the session.
    pub fn add_genesis(&mut self, vertex_id: VertexId) {
        self.genesis_vertices.push(vertex_id);
        self.tip_vertices.push(vertex_id);
        self.updated_at = Utc::now();
    }

    /// Update the tip vertices after appending a new vertex.
    /// 
    /// Removes parents from tips, adds the new vertex.
    pub fn update_tips(&mut self, new_vertex: VertexId, parents: &[VertexId]) {
        // Remove parents from tips (they're no longer tips)
        self.tip_vertices.retain(|v| !parents.contains(v));

        // Add the new vertex as a tip
        self.tip_vertices.push(new_vertex);

        self.updated_at = Utc::now();
    }

    /// Resolve the session successfully.
    pub fn resolve(&mut self, resolution_vertices: Vec<VertexId>) {
        self.state = SessionState::Resolved {
            resolved_at: Utc::now(),
        };
        self.resolution_vertices = resolution_vertices;
        self.updated_at = Utc::now();
    }

    /// Mark the session as failed.
    pub fn fail(&mut self, reason: impl Into<String>) {
        self.state = SessionState::Failed {
            failed_at: Utc::now(),
            reason: reason.into(),
        };
        self.updated_at = Utc::now();
    }

    /// Mark the session as expired.
    pub fn expire(&mut self) {
        self.state = SessionState::Expired {
            expired_at: Utc::now(),
        };
        self.updated_at = Utc::now();
    }

    /// Check if the session is active.
    #[must_use]
    pub fn is_active(&self) -> bool {
        matches!(self.state, SessionState::Active)
    }

    /// Check if the session is resolved.
    #[must_use]
    pub fn is_resolved(&self) -> bool {
        matches!(self.state, SessionState::Resolved { .. })
    }

    /// Check if the session has expired.
    #[must_use]
    pub fn is_expired(&self) -> bool {
        if let Some(expires_at) = self.expires_at {
            if Utc::now() > expires_at {
                return true;
            }
        }
        matches!(self.state, SessionState::Expired { .. })
    }

    /// Get the session age (time since creation).
    #[must_use]
    pub fn age(&self) -> Duration {
        Utc::now() - self.created_at
    }

    /// Get the session lifetime (created → now or resolved/failed/expired).
    #[must_use]
    pub fn lifetime(&self) -> Duration {
        match self.state {
            SessionState::Active => Utc::now() - self.created_at,
            SessionState::Resolved { resolved_at } => resolved_at - self.created_at,
            SessionState::Failed { failed_at, .. } => failed_at - self.created_at,
            SessionState::Expired { expired_at } => expired_at - self.created_at,
        }
    }
}

impl Default for SessionMetadata {
    fn default() -> Self {
        Self {
            name: None,
            description: None,
            creator: None,
            tags: Vec::new(),
            custom: HashMap::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_session_is_active() {
        let session = Session::new(SessionMetadata::default());

        assert!(session.is_active());
        assert!(!session.is_resolved());
        assert!(!session.is_expired());
    }

    #[test]
    fn test_add_genesis() {
        let mut session = Session::new(SessionMetadata::default());
        let vertex_id = VertexId::from_bytes([1u8; 32]);

        session.add_genesis(vertex_id);

        assert_eq!(session.genesis_vertices.len(), 1);
        assert_eq!(session.tip_vertices.len(), 1);
        assert_eq!(session.genesis_vertices[0], vertex_id);
    }

    #[test]
    fn test_update_tips() {
        let mut session = Session::new(SessionMetadata::default());

        let v1 = VertexId::from_bytes([1u8; 32]);
        let v2 = VertexId::from_bytes([2u8; 32]);
        let v3 = VertexId::from_bytes([3u8; 32]);

        session.add_genesis(v1);
        assert_eq!(session.tip_vertices, vec![v1]);

        session.update_tips(v2, &[v1]);
        assert_eq!(session.tip_vertices, vec![v2]);

        session.update_tips(v3, &[v2]);
        assert_eq!(session.tip_vertices, vec![v3]);
    }

    #[test]
    fn test_merge_creates_multiple_parents() {
        let mut session = Session::new(SessionMetadata::default());

        let v1 = VertexId::from_bytes([1u8; 32]);
        let v2 = VertexId::from_bytes([2u8; 32]);
        let merge = VertexId::from_bytes([3u8; 32]);

        session.add_genesis(v1);
        session.tip_vertices.push(v2); // Simulate another branch

        assert_eq!(session.tip_vertices.len(), 2);

        session.update_tips(merge, &[v1, v2]);

        assert_eq!(session.tip_vertices.len(), 1);
        assert_eq!(session.tip_vertices[0], merge);
    }

    #[test]
    fn test_resolve_session() {
        let mut session = Session::new(SessionMetadata::default());
        let resolution = VertexId::from_bytes([1u8; 32]);

        session.resolve(vec![resolution]);

        assert!(!session.is_active());
        assert!(session.is_resolved());
        assert_eq!(session.resolution_vertices.len(), 1);
    }

    #[test]
    fn test_fail_session() {
        let mut session = Session::new(SessionMetadata::default());

        session.fail("Something went wrong");

        assert!(!session.is_active());
        assert!(!session.is_resolved());
        assert!(matches!(session.state, SessionState::Failed { .. }));
    }

    #[test]
    fn test_session_with_expiration() {
        let duration = Duration::seconds(10);
        let session = Session::with_expiration(SessionMetadata::default(), duration);

        assert!(session.expires_at.is_some());
        assert!(!session.is_expired()); // Not yet
    }

    #[test]
    fn test_session_age() {
        let session = Session::new(SessionMetadata::default());
        let age = session.age();

        assert!(age.num_milliseconds() >= 0);
        assert!(age.num_seconds() < 1); // Just created
    }
}

