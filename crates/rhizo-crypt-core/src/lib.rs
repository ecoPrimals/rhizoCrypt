//! # RhizoCrypt
//!
//! Core DAG Engine - Ephemeral Working Memory
//!
//! ## Overview
//!
//! RhizoCrypt is part of the ecoPrimals ecosystem. It provides the git-like
//! DAG engine that underlies Phase 2's memory and attribution layer.
//!
//! ## Quick Start
//!
//! ```rust,ignore
//! use rhizo_crypt_core::RhizoCrypt;
//!
//! let primal = RhizoCrypt::new(config);
//! primal.start().await?;
//! ```

#![warn(missing_docs)]
#![warn(clippy::all)]
#![warn(clippy::pedantic)]

pub mod config;
pub mod error;
pub mod vertex;
pub mod session;
pub mod store;

use sourdough_core::{
    PrimalLifecycle, PrimalHealth, PrimalState,
    HealthStatus, health::HealthReport, PrimalError,
};

pub use vertex::{Vertex, VertexId, VertexData, VertexError};
pub use session::{Session, SessionId, SessionMetadata, SessionState};
pub use store::{VertexStore, StoreError};

/// RhizoCrypt configuration.
pub use config::RhizoCryptConfig;

/// RhizoCrypt errors.
pub use error::RhizoCryptError;

/// The RhizoCrypt primal - Core DAG Engine.
pub struct RhizoCrypt {
    config: RhizoCryptConfig,
    state: PrimalState,
    /// The vertex store (in-memory DAG).
    store: VertexStore,
}

impl RhizoCrypt {
    /// Create a new RhizoCrypt instance.
    #[must_use]
    pub fn new(config: RhizoCryptConfig) -> Self {
        Self {
            config,
            state: PrimalState::Created,
            store: VertexStore::new(),
        }
    }

    /// Get a reference to the vertex store.
    #[must_use]
    pub fn store(&self) -> &VertexStore {
        &self.store
    }

    /// Create a new session.
    pub fn create_session(&self, metadata: SessionMetadata) -> Result<Session, RhizoCryptError> {
        let session = Session::new(metadata);
        self.store.insert_session(session.clone())
            .map_err(|e| RhizoCryptError::Internal(e.to_string()))?;
        Ok(session)
    }

    /// Append a genesis vertex to a session.
    pub fn append_genesis(
        &self,
        session_id: SessionId,
        event_type: impl Into<String>,
        payload: serde_json::Value,
    ) -> Result<Vertex, RhizoCryptError> {
        // Create the vertex
        let vertex = Vertex::genesis(session_id, event_type, payload)
            .map_err(|e| RhizoCryptError::Internal(e.to_string()))?;

        // Insert into store
        self.store.insert_vertex(vertex.clone())
            .map_err(|e| RhizoCryptError::Internal(e.to_string()))?;

        // Update session
        let mut session = self.store.get_session(&session_id)
            .map_err(|e| RhizoCryptError::Internal(e.to_string()))?
            .ok_or_else(|| RhizoCryptError::Internal(format!("Session not found: {session_id}")))?;

        session.add_genesis(vertex.id);

        self.store.update_session(session)
            .map_err(|e| RhizoCryptError::Internal(e.to_string()))?;

        Ok(vertex)
    }

    /// Append a vertex with a single parent.
    pub fn append(
        &self,
        parent: VertexId,
        session_id: SessionId,
        event_type: impl Into<String>,
        payload: serde_json::Value,
    ) -> Result<Vertex, RhizoCryptError> {
        // Create the vertex
        let vertex = Vertex::with_parent(parent, session_id, event_type, payload)
            .map_err(|e| RhizoCryptError::Internal(e.to_string()))?;

        // Insert into store
        self.store.insert_vertex(vertex.clone())
            .map_err(|e| RhizoCryptError::Internal(e.to_string()))?;

        // Update session tips
        let mut session = self.store.get_session(&session_id)
            .map_err(|e| RhizoCryptError::Internal(e.to_string()))?
            .ok_or_else(|| RhizoCryptError::Internal(format!("Session not found: {session_id}")))?;

        session.update_tips(vertex.id, &[parent]);

        self.store.update_session(session)
            .map_err(|e| RhizoCryptError::Internal(e.to_string()))?;

        Ok(vertex)
    }

    /// Append a merge vertex (multiple parents).
    pub fn append_merge(
        &self,
        parents: Vec<VertexId>,
        session_id: SessionId,
        event_type: impl Into<String>,
        payload: serde_json::Value,
    ) -> Result<Vertex, RhizoCryptError> {
        // Create the vertex
        let vertex = Vertex::merge(parents.clone(), session_id, event_type, payload)
            .map_err(|e| RhizoCryptError::Internal(e.to_string()))?;

        // Insert into store
        self.store.insert_vertex(vertex.clone())
            .map_err(|e| RhizoCryptError::Internal(e.to_string()))?;

        // Update session tips
        let mut session = self.store.get_session(&session_id)
            .map_err(|e| RhizoCryptError::Internal(e.to_string()))?
            .ok_or_else(|| RhizoCryptError::Internal(format!("Session not found: {session_id}")))?;

        session.update_tips(vertex.id, &parents);

        self.store.update_session(session)
            .map_err(|e| RhizoCryptError::Internal(e.to_string()))?;

        Ok(vertex)
    }

    /// Get a vertex by ID.
    pub fn get_vertex(&self, id: &VertexId) -> Result<Option<Vertex>, RhizoCryptError> {
        self.store.get_vertex(id)
            .map_err(|e| RhizoCryptError::Internal(e.to_string()))
    }

    /// Get a session by ID.
    pub fn get_session(&self, id: &SessionId) -> Result<Option<Session>, RhizoCryptError> {
        self.store.get_session(id)
            .map_err(|e| RhizoCryptError::Internal(e.to_string()))
    }

    /// Resolve a session successfully.
    pub fn resolve_session(&self, session_id: SessionId) -> Result<(), RhizoCryptError> {
        let mut session = self.store.get_session(&session_id)
            .map_err(|e| RhizoCryptError::Internal(e.to_string()))?
            .ok_or_else(|| RhizoCryptError::Internal(format!("Session not found: {session_id}")))?;

        let resolution_vertices = session.tip_vertices.clone();
        session.resolve(resolution_vertices);

        self.store.update_session(session)
            .map_err(|e| RhizoCryptError::Internal(e.to_string()))?;

        Ok(())
    }

    /// Get statistics about the RhizoCrypt instance.
    pub fn stats(&self) -> Result<RhizoCryptStats, RhizoCryptError> {
        let vertex_count = self.store.vertex_count()
            .map_err(|e| RhizoCryptError::Internal(e.to_string()))?;
        let session_count = self.store.session_count()
            .map_err(|e| RhizoCryptError::Internal(e.to_string()))?;

        Ok(RhizoCryptStats {
            vertex_count,
            session_count,
        })
    }
}

/// Statistics about a RhizoCrypt instance.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct RhizoCryptStats {
    /// Total number of vertices in the store.
    pub vertex_count: usize,
    /// Total number of sessions.
    pub session_count: usize,
}

impl PrimalLifecycle for RhizoCrypt {
    fn state(&self) -> PrimalState {
        self.state
    }

    async fn start(&mut self) -> Result<(), PrimalError> {
        self.state = PrimalState::Starting;
        tracing::info!("RhizoCrypt starting...");
        
        // Core DAG engine is ready immediately
        // Future: Initialize persistent storage backend
        
        self.state = PrimalState::Running;
        tracing::info!("RhizoCrypt running (in-memory DAG engine)");
        Ok(())
    }

    async fn stop(&mut self) -> Result<(), PrimalError> {
        self.state = PrimalState::Stopping;
        tracing::info!("RhizoCrypt stopping...");
        
        // Future: Flush any pending commits to LoamSpine
        // Future: Close persistent storage connections
        
        self.state = PrimalState::Stopped;
        tracing::info!("RhizoCrypt stopped");
        Ok(())
    }
}

impl PrimalHealth for RhizoCrypt {
    fn health_status(&self) -> HealthStatus {
        if self.state.is_running() {
            HealthStatus::Healthy
        } else {
            HealthStatus::Unhealthy {
                reason: format!("state: {}", self.state),
            }
        }
    }

    async fn health_check(&self) -> Result<HealthReport, PrimalError> {
        let mut report = HealthReport::new("RhizoCrypt", env!("CARGO_PKG_VERSION"))
            .with_status(self.health_status());

        // Add metrics as details
        if let Ok(stats) = self.stats() {
            report = report
                .with_detail("vertices", stats.vertex_count.to_string())
                .with_detail("sessions", stats.session_count.to_string());
        }

        Ok(report)
    }
}
