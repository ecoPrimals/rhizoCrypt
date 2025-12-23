// vertex.rs - Core DAG vertex implementation
//
// RhizoCrypt is a content-addressed DAG where every vertex is identified by
// the Blake3 hash of its contents. This provides:
// - Deduplication (same content = same hash)
// - Tamper-evidence (any change = different hash)
// - Provenance tracking (parents referenced by hash)

use blake3::Hash;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// A content-addressed identifier for a vertex.
/// 
/// Uses Blake3 for fast, cryptographically secure hashing.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct VertexId(Hash);

impl VertexId {
    /// Create a new `VertexId` from raw bytes.
    #[must_use]
    pub fn from_bytes(bytes: [u8; 32]) -> Self {
        Self(Hash::from(bytes))
    }

    /// Get the raw bytes of this `VertexId`.
    #[must_use]
    pub fn as_bytes(&self) -> &[u8; 32] {
        self.0.as_bytes()
    }

    /// Convert to a hex string for display.
    #[must_use]
    pub fn to_hex(&self) -> String {
        self.0.to_hex().to_string()
    }
}

impl std::fmt::Display for VertexId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", &self.to_hex()[..8]) // Show first 8 chars
    }
}

// Manual Serialize/Deserialize implementations for VertexId
impl Serialize for VertexId {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_hex())
    }
}

impl<'de> Deserialize<'de> for VertexId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let hex = String::deserialize(deserializer)?;
        let hash = blake3::Hash::from_hex(&hex)
            .map_err(serde::de::Error::custom)?;
        let bytes = *hash.as_bytes(); // Dereference to get [u8; 32]
        Ok(VertexId::from_bytes(bytes))
    }
}

/// A vertex in the RhizoCrypt DAG.
///
/// Vertices are content-addressed: the vertex ID is the Blake3 hash
/// of the serialized `VertexData`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Vertex {
    /// The content-addressed ID of this vertex.
    /// 
    /// This is the Blake3 hash of `data` field.
    pub id: VertexId,

    /// The vertex data (what gets hashed).
    pub data: VertexData,
}

/// The data contained in a vertex.
///
/// This is what gets hashed to produce the `VertexId`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VertexData {
    /// Parent vertices (DAG edges).
    /// 
    /// Empty for genesis vertices, multiple for merge vertices.
    pub parents: Vec<VertexId>,

    /// The session this vertex belongs to.
    pub session_id: SessionId,

    /// When this vertex was created.
    pub timestamp: DateTime<Utc>,

    /// The DID of the entity that created this vertex.
    /// 
    /// Optional - not all vertices have a specific creator.
    pub creator: Option<String>, // TODO: Use BearDog DID type

    /// The event type (domain-specific).
    pub event_type: String,

    /// The event payload (domain-specific).
    pub payload: serde_json::Value,

    /// Optional metadata (tags, annotations, etc.).
    #[serde(default)]
    pub metadata: HashMap<String, serde_json::Value>,
}

impl Vertex {
    /// Create a new vertex from data.
    /// 
    /// Computes the content-addressed ID by hashing the serialized data.
    pub fn new(data: VertexData) -> Result<Self, VertexError> {
        // Serialize the data
        let serialized = serde_json::to_vec(&data)
            .map_err(|e| VertexError::Serialization(e.to_string()))?;

        // Hash it with Blake3
        let hash = blake3::hash(&serialized);
        let id = VertexId(hash);

        Ok(Self { id, data })
    }

    /// Create a new genesis vertex (no parents).
    pub fn genesis(
        session_id: SessionId,
        event_type: impl Into<String>,
        payload: serde_json::Value,
    ) -> Result<Self, VertexError> {
        let data = VertexData {
            parents: Vec::new(),
            session_id,
            timestamp: Utc::now(),
            creator: None,
            event_type: event_type.into(),
            payload,
            metadata: HashMap::new(),
        };
        Self::new(data)
    }

    /// Create a new vertex with a single parent.
    pub fn with_parent(
        parent: VertexId,
        session_id: SessionId,
        event_type: impl Into<String>,
        payload: serde_json::Value,
    ) -> Result<Self, VertexError> {
        let data = VertexData {
            parents: vec![parent],
            session_id,
            timestamp: Utc::now(),
            creator: None,
            event_type: event_type.into(),
            payload,
            metadata: HashMap::new(),
        };
        Self::new(data)
    }

    /// Create a new merge vertex (multiple parents).
    pub fn merge(
        parents: Vec<VertexId>,
        session_id: SessionId,
        event_type: impl Into<String>,
        payload: serde_json::Value,
    ) -> Result<Self, VertexError> {
        if parents.is_empty() {
            return Err(VertexError::InvalidMerge(
                "Merge vertex must have at least one parent".to_string(),
            ));
        }

        let data = VertexData {
            parents,
            session_id,
            timestamp: Utc::now(),
            creator: None,
            event_type: event_type.into(),
            payload,
            metadata: HashMap::new(),
        };
        Self::new(data)
    }

    /// Verify that the vertex ID matches its content.
    /// 
    /// Useful for tamper detection.
    pub fn verify(&self) -> Result<(), VertexError> {
        let serialized = serde_json::to_vec(&self.data)
            .map_err(|e| VertexError::Serialization(e.to_string()))?;
        let computed_hash = blake3::hash(&serialized);
        let computed_id = VertexId(computed_hash);

        if computed_id != self.id {
            return Err(VertexError::InvalidHash {
                expected: self.id,
                computed: computed_id,
            });
        }

        Ok(())
    }

    /// Check if this is a genesis vertex (no parents).
    #[must_use]
    pub fn is_genesis(&self) -> bool {
        self.data.parents.is_empty()
    }

    /// Check if this is a merge vertex (multiple parents).
    #[must_use]
    pub fn is_merge(&self) -> bool {
        self.data.parents.len() > 1
    }
}

/// Session identifier (UUID v4).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SessionId(Uuid);

impl SessionId {
    /// Generate a new random session ID.
    #[must_use]
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }

    /// Create from a UUID.
    #[must_use]
    pub fn from_uuid(uuid: Uuid) -> Self {
        Self(uuid)
    }

    /// Get the inner UUID.
    #[must_use]
    pub fn as_uuid(&self) -> &Uuid {
        &self.0
    }
}

impl Default for SessionId {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for SessionId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Errors that can occur when working with vertices.
#[derive(Debug, thiserror::Error)]
pub enum VertexError {
    /// Failed to serialize vertex data.
    #[error("serialization error: {0}")]
    Serialization(String),

    /// Vertex hash verification failed.
    #[error("invalid hash: expected {expected}, computed {computed}")]
    InvalidHash {
        /// The expected vertex ID.
        expected: VertexId,
        /// The computed vertex ID.
        computed: VertexId,
    },

    /// Invalid merge operation.
    #[error("invalid merge: {0}")]
    InvalidMerge(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vertex_id_is_content_addressed() {
        let session_id = SessionId::new();
        let payload = serde_json::json!({"test": "data"});
        let timestamp = Utc::now();

        // Create two vertices with identical data (including timestamp)
        let data1 = VertexData {
            parents: Vec::new(),
            session_id,
            timestamp,
            creator: None,
            event_type: "test".to_string(),
            payload: payload.clone(),
            metadata: HashMap::new(),
        };

        let data2 = VertexData {
            parents: Vec::new(),
            session_id,
            timestamp,
            creator: None,
            event_type: "test".to_string(),
            payload,
            metadata: HashMap::new(),
        };

        let v1 = Vertex::new(data1).unwrap();
        let v2 = Vertex::new(data2).unwrap();

        // They should have the same ID (content-addressed)
        assert_eq!(v1.id, v2.id, "Identical content should produce identical IDs");
    }

    #[test]
    fn test_different_content_different_id() {
        let session_id = SessionId::new();
        let payload1 = serde_json::json!({"test": "data1"});
        let payload2 = serde_json::json!({"test": "data2"});

        let v1 = Vertex::genesis(session_id, "test", payload1).unwrap();
        let v2 = Vertex::genesis(session_id, "test", payload2).unwrap();

        assert_ne!(v1.id, v2.id, "Different content should produce different IDs");
    }

    #[test]
    fn test_vertex_verification() {
        let session_id = SessionId::new();
        let payload = serde_json::json!({"test": "data"});

        let vertex = Vertex::genesis(session_id, "test", payload).unwrap();

        // Verification should pass
        assert!(vertex.verify().is_ok());
    }

    #[test]
    fn test_genesis_vertex() {
        let session_id = SessionId::new();
        let payload = serde_json::json!({});

        let vertex = Vertex::genesis(session_id, "genesis", payload).unwrap();

        assert!(vertex.is_genesis());
        assert!(!vertex.is_merge());
        assert_eq!(vertex.data.parents.len(), 0);
    }

    #[test]
    fn test_vertex_with_parent() {
        let session_id = SessionId::new();

        let parent = Vertex::genesis(session_id, "parent", serde_json::json!({})).unwrap();
        let child = Vertex::with_parent(parent.id, session_id, "child", serde_json::json!({})).unwrap();

        assert!(!child.is_genesis());
        assert!(!child.is_merge());
        assert_eq!(child.data.parents.len(), 1);
        assert_eq!(child.data.parents[0], parent.id);
    }

    #[test]
    fn test_merge_vertex() {
        let session_id = SessionId::new();

        let p1 = Vertex::genesis(session_id, "p1", serde_json::json!({})).unwrap();
        let p2 = Vertex::genesis(session_id, "p2", serde_json::json!({})).unwrap();
        let merge = Vertex::merge(
            vec![p1.id, p2.id],
            session_id,
            "merge",
            serde_json::json!({}),
        ).unwrap();

        assert!(!merge.is_genesis());
        assert!(merge.is_merge());
        assert_eq!(merge.data.parents.len(), 2);
    }

    #[test]
    fn test_merge_requires_parents() {
        let session_id = SessionId::new();

        let result = Vertex::merge(vec![], session_id, "invalid", serde_json::json!({}));

        assert!(result.is_err());
    }
}

