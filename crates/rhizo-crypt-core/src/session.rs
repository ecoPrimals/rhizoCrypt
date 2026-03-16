// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2024–2026 ecoPrimals Project

//! Session management for RhizoCrypt.
//!
//! A session is a scoped DAG with a defined lifecycle.

use crate::types::{Did, SessionId, SliceId, Timestamp, VertexId};
use hashbrown::{HashMap, HashSet};
use serde::{Deserialize, Serialize};
use std::time::Duration;

/// A RhizoCrypt session (scoped DAG with lifecycle).
#[derive(Clone, Debug)]
pub struct Session {
    /// Unique session identifier.
    pub id: SessionId,

    /// Human-readable session name.
    pub name: Option<String>,

    /// Session type (determines event types, policies).
    pub session_type: SessionType,

    /// Session configuration.
    pub config: SessionConfig,

    /// Genesis timestamp.
    pub created_at: Timestamp,

    /// Session state.
    pub state: SessionState,

    /// Genesis vertices (roots with no parents).
    pub genesis: HashSet<VertexId>,

    /// Frontier vertices (tips with no children).
    pub frontier: HashSet<VertexId>,

    /// Total vertex count.
    pub vertex_count: u64,

    /// Active slices in this session.
    pub slices: HashMap<SliceId, SliceRef>,

    /// Agents participating in this session.
    pub agents: HashSet<Did>,
}

impl Session {
    /// Check if the session is accepting events.
    #[must_use]
    pub const fn is_active(&self) -> bool {
        self.state.is_active()
    }

    /// Check if the session is in a terminal state.
    #[must_use]
    pub const fn is_terminal(&self) -> bool {
        self.state.is_terminal()
    }

    /// Add a vertex to the frontier, removing consumed parents.
    pub fn update_frontier(&mut self, new_vertex: VertexId, parents: &[VertexId]) {
        // Remove parents from frontier (they now have children)
        for parent in parents {
            self.frontier.remove(parent);
        }
        // Add new vertex to frontier
        self.frontier.insert(new_vertex);
        self.vertex_count += 1;

        // If no parents, this is a genesis vertex
        if parents.is_empty() {
            self.genesis.insert(new_vertex);
        }
    }

    /// Add an agent to the session.
    pub fn add_agent(&mut self, agent: Did) {
        self.agents.insert(agent);
    }

    /// Add a slice to the session.
    pub fn add_slice(&mut self, slice_id: SliceId, slice_ref: SliceRef) {
        self.slices.insert(slice_id, slice_ref);
    }

    /// Transition to resolving state.
    ///
    /// # Errors
    ///
    /// Returns an error if the session is not active.
    pub fn begin_resolve(&mut self) -> Result<(), SessionStateError> {
        if !self.is_active() {
            return Err(SessionStateError::NotActive {
                current: self.state.name().to_string(),
            });
        }
        self.state = SessionState::Resolving {
            started_at: Timestamp::now(),
        };
        Ok(())
    }

    /// Transition to committed state.
    ///
    /// # Errors
    ///
    /// Returns an error if the session is not resolving.
    pub fn commit(&mut self, loam_ref: LoamCommitRef) -> Result<(), SessionStateError> {
        if !matches!(self.state, SessionState::Resolving { .. }) {
            return Err(SessionStateError::InvalidTransition {
                from: self.state.name().to_string(),
                to: "committed".to_string(),
            });
        }
        self.state = SessionState::Committed {
            loam_ref,
            committed_at: Timestamp::now(),
        };
        Ok(())
    }

    /// Transition to discarded state.
    pub fn discard(&mut self, reason: DiscardReason) {
        self.state = SessionState::Discarded {
            reason,
            discarded_at: Timestamp::now(),
        };
    }
}

/// Session state.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum SessionState {
    /// Actively accepting events.
    Active,

    /// Paused (no new events, can resume).
    Paused {
        /// Reason for pausing.
        reason: String,
    },

    /// Preparing for resolution.
    Resolving {
        /// When resolution started.
        started_at: Timestamp,
    },

    /// Committed to LoamSpine.
    Committed {
        /// LoamSpine commit reference.
        loam_ref: LoamCommitRef,
        /// When committed.
        committed_at: Timestamp,
    },

    /// Discarded without commit.
    Discarded {
        /// Reason for discarding.
        reason: DiscardReason,
        /// When discarded.
        discarded_at: Timestamp,
    },

    /// Garbage collected.
    Expired {
        /// When expired.
        expired_at: Timestamp,
    },
}

impl SessionState {
    /// Check if session is accepting events.
    #[must_use]
    pub const fn is_active(&self) -> bool {
        matches!(self, Self::Active)
    }

    /// Check if session is in a terminal state.
    #[must_use]
    pub const fn is_terminal(&self) -> bool {
        matches!(self, Self::Committed { .. } | Self::Discarded { .. } | Self::Expired { .. })
    }

    /// Get the state name.
    #[must_use]
    pub const fn name(&self) -> &'static str {
        match self {
            Self::Active => "active",
            Self::Paused {
                ..
            } => "paused",
            Self::Resolving {
                ..
            } => "resolving",
            Self::Committed {
                ..
            } => "committed",
            Self::Discarded {
                ..
            } => "discarded",
            Self::Expired {
                ..
            } => "expired",
        }
    }
}

/// Session state transition error.
#[derive(Debug, thiserror::Error)]
pub enum SessionStateError {
    /// Session is not active.
    #[error("session is not active: {current}")]
    NotActive {
        /// Current state.
        current: String,
    },

    /// Invalid state transition.
    #[error("invalid state transition from {from} to {to}")]
    InvalidTransition {
        /// Current state.
        from: String,
        /// Target state.
        to: String,
    },
}

/// Reason for discarding a session.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum DiscardReason {
    /// Manual discard by user.
    Manual {
        /// Reason provided.
        reason: String,
    },
    /// Session timed out.
    Timeout,
    /// Limit exceeded.
    LimitExceeded {
        /// Which limit.
        limit: String,
    },
    /// Error during processing.
    Error {
        /// Error message.
        message: String,
    },
}

/// LoamSpine commit reference.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct LoamCommitRef {
    /// Spine identifier.
    pub spine_id: String,
    /// Entry hash.
    pub entry_hash: [u8; 32],
    /// Entry index.
    pub index: u64,
}

/// Reference to a slice in the session.
#[derive(Clone, Debug)]
pub struct SliceRef {
    /// Slice identifier.
    pub id: SliceId,
    /// Checkout vertex.
    pub checkout_vertex: VertexId,
    /// When checked out.
    pub checked_out_at: Timestamp,
}

/// Session type.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum SessionType {
    /// Gaming session (raid, match, etc.).
    Gaming {
        /// Game identifier.
        game_id: String,
    },
    /// Scientific experiment.
    Experiment {
        /// Protocol identifier.
        protocol_id: String,
    },
    /// Collaborative document editing.
    Collaboration {
        /// Workspace identifier.
        workspace_id: String,
    },
    /// General-purpose session.
    #[default]
    General,
    /// Custom domain.
    Custom {
        /// Domain name.
        domain: String,
    },
}

/// Session configuration.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SessionConfig {
    /// Maximum session duration.
    pub max_duration: Duration,

    /// Maximum vertices before forced resolution.
    pub max_vertices: u64,

    /// Maximum payload bytes.
    pub max_payload_bytes: u64,

    /// Require signatures for all events.
    pub require_all_signatures: bool,

    /// Event types that require signatures.
    pub signature_required_events: HashSet<String>,

    /// Automatic dehydration on resolve.
    pub auto_dehydrate: bool,

    /// Session owner.
    pub owner: Did,

    /// Agents allowed to append (None = open).
    pub allowed_agents: Option<HashSet<Did>>,
}

impl Default for SessionConfig {
    fn default() -> Self {
        Self {
            max_duration: Duration::from_secs(3600), // 1 hour
            max_vertices: 100_000,
            max_payload_bytes: 1024 * 1024 * 1024, // 1 GB
            require_all_signatures: false,
            signature_required_events: HashSet::new(),
            auto_dehydrate: true,
            owner: Did::default(),
            allowed_agents: None,
        }
    }
}

/// Builder for creating sessions.
#[derive(Clone, Debug)]
pub struct SessionBuilder {
    name: Option<String>,
    session_type: SessionType,
    config: SessionConfig,
}

impl SessionBuilder {
    /// Create a new session builder.
    #[must_use]
    pub fn new(session_type: SessionType) -> Self {
        Self {
            name: None,
            session_type,
            config: SessionConfig::default(),
        }
    }

    /// Set session name.
    #[must_use]
    pub fn with_name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }

    /// Set session owner.
    #[must_use]
    pub fn with_owner(mut self, owner: Did) -> Self {
        self.config.owner = owner;
        self
    }

    /// Set max duration.
    #[must_use]
    pub const fn with_max_duration(mut self, duration: Duration) -> Self {
        self.config.max_duration = duration;
        self
    }

    /// Set max vertices.
    #[must_use]
    pub const fn with_max_vertices(mut self, max: u64) -> Self {
        self.config.max_vertices = max;
        self
    }

    /// Require all signatures.
    #[must_use]
    pub const fn require_all_signatures(mut self) -> Self {
        self.config.require_all_signatures = true;
        self
    }

    /// Set auto dehydrate.
    #[must_use]
    pub const fn with_auto_dehydrate(mut self, auto: bool) -> Self {
        self.config.auto_dehydrate = auto;
        self
    }

    /// Build the session.
    #[must_use]
    pub fn build(self) -> Session {
        Session {
            id: SessionId::now(),
            name: self.name,
            session_type: self.session_type,
            config: self.config,
            created_at: Timestamp::now(),
            state: SessionState::Active,
            genesis: HashSet::new(),
            frontier: HashSet::new(),
            vertex_count: 0,
            slices: HashMap::new(),
            agents: HashSet::new(),
        }
    }
}

#[cfg(test)]
#[expect(clippy::unwrap_used, reason = "test code")]
mod tests {
    use super::*;

    #[test]
    fn test_session_builder() {
        let session = SessionBuilder::new(SessionType::General)
            .with_name("Test Session")
            .with_owner(Did::new("did:key:test"))
            .with_max_vertices(1000)
            .build();

        assert_eq!(session.name, Some("Test Session".to_string()));
        assert!(session.is_active());
        assert!(!session.is_terminal());
        assert_eq!(session.vertex_count, 0);
    }

    #[test]
    fn test_session_state_transitions() {
        let mut session = SessionBuilder::new(SessionType::General).build();

        assert!(session.is_active());

        // Begin resolve
        session.begin_resolve().unwrap();
        assert!(!session.is_active());
        assert!(matches!(session.state, SessionState::Resolving { .. }));

        // Commit
        let loam_ref = LoamCommitRef {
            spine_id: "test".to_string(),
            entry_hash: [0u8; 32],
            index: 1,
        };
        session.commit(loam_ref).unwrap();
        assert!(session.is_terminal());
    }

    #[test]
    fn test_session_discard() {
        let mut session = SessionBuilder::new(SessionType::General).build();

        session.discard(DiscardReason::Timeout);
        assert!(session.is_terminal());
        assert!(matches!(session.state, SessionState::Discarded { .. }));
    }

    #[test]
    fn test_session_frontier_update() {
        let mut session = SessionBuilder::new(SessionType::General).build();

        let v1 = VertexId::from_bytes(b"vertex1");
        session.update_frontier(v1, &[]);
        assert!(session.genesis.contains(&v1));
        assert!(session.frontier.contains(&v1));
        assert_eq!(session.vertex_count, 1);

        let v2 = VertexId::from_bytes(b"vertex2");
        session.update_frontier(v2, &[v1]);
        assert!(!session.frontier.contains(&v1));
        assert!(session.frontier.contains(&v2));
        assert_eq!(session.vertex_count, 2);
    }

    #[test]
    fn test_session_type_default() {
        assert_eq!(SessionType::default(), SessionType::General);
    }

    #[test]
    fn test_session_with_parent() {
        let mut session = SessionBuilder::new(SessionType::General).build();
        let v1 = VertexId::from_bytes(b"parent1");
        let v2 = VertexId::from_bytes(b"parent2");
        session.update_frontier(v1, &[]);
        session.update_frontier(v2, &[]);
        let v3 = VertexId::from_bytes(b"child");
        session.update_frontier(v3, &[v1, v2]);
        assert!(!session.frontier.contains(&v1));
        assert!(!session.frontier.contains(&v2));
        assert!(session.frontier.contains(&v3));
        assert!(session.genesis.contains(&v1));
        assert!(session.genesis.contains(&v2));
        assert!(!session.genesis.contains(&v3));
    }

    #[test]
    fn test_session_with_max_vertices() {
        let session = SessionBuilder::new(SessionType::General).with_max_vertices(500).build();
        assert_eq!(session.config.max_vertices, 500);
    }

    #[test]
    fn test_session_with_ttl() {
        let session = SessionBuilder::new(SessionType::General)
            .with_max_duration(Duration::from_secs(7200))
            .build();
        assert_eq!(session.config.max_duration, Duration::from_secs(7200));
    }

    #[test]
    fn test_session_add_agent() {
        let mut session = SessionBuilder::new(SessionType::General).build();
        let agent1 = Did::new("did:key:agent1");
        let agent2 = Did::new("did:key:agent2");
        session.add_agent(agent1.clone());
        session.add_agent(agent2.clone());
        session.add_agent(agent1.clone());
        assert!(session.agents.contains(&agent1));
        assert!(session.agents.contains(&agent2));
        assert_eq!(session.agents.len(), 2);
    }

    #[test]
    fn test_session_vertex_count() {
        let mut session = SessionBuilder::new(SessionType::General).build();
        assert_eq!(session.vertex_count, 0);
        let v1 = VertexId::from_bytes(b"v1");
        session.update_frontier(v1, &[]);
        assert_eq!(session.vertex_count, 1);
        let v2 = VertexId::from_bytes(b"v2");
        session.update_frontier(v2, &[v1]);
        assert_eq!(session.vertex_count, 2);
    }

    #[test]
    fn test_session_frontier_operations() {
        let mut session = SessionBuilder::new(SessionType::General).build();
        let v1 = VertexId::from_bytes(b"a");
        let v2 = VertexId::from_bytes(b"b");
        let v3 = VertexId::from_bytes(b"c");
        let v4 = VertexId::from_bytes(b"d");
        session.update_frontier(v1, &[]);
        session.update_frontier(v2, &[]);
        session.update_frontier(v3, &[v1]);
        session.update_frontier(v4, &[v2, v3]);
        assert_eq!(session.frontier.len(), 1);
        assert!(session.frontier.contains(&v4));
        assert_eq!(session.vertex_count, 4);
        assert_eq!(session.genesis.len(), 2);
    }

    #[test]
    fn test_session_serialization() {
        let state = SessionState::Resolving {
            started_at: Timestamp::now(),
        };
        let json = serde_json::to_string(&state).unwrap();
        let parsed: SessionState = serde_json::from_str(&json).unwrap();
        assert!(matches!(parsed, SessionState::Resolving { .. }));

        let discard = DiscardReason::LimitExceeded {
            limit: "vertices".to_string(),
        };
        let json = serde_json::to_string(&discard).unwrap();
        let parsed: DiscardReason = serde_json::from_str(&json).unwrap();
        assert!(matches!(parsed, DiscardReason::LimitExceeded { limit } if limit == "vertices"));
    }

    #[test]
    fn test_loam_commit_ref_serialization() {
        let loam_ref = LoamCommitRef {
            spine_id: "spine-42".to_string(),
            entry_hash: [1u8; 32],
            index: 99,
        };
        let json = serde_json::to_string(&loam_ref).unwrap();
        let parsed: LoamCommitRef = serde_json::from_str(&json).unwrap();
        assert_eq!(loam_ref.spine_id, parsed.spine_id);
        assert_eq!(loam_ref.entry_hash, parsed.entry_hash);
        assert_eq!(loam_ref.index, parsed.index);
    }
}
