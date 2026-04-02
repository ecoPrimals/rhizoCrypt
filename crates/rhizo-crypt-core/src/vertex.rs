// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2024–2026 ecoPrimals Project

//! Vertex data structure and builder.
//!
//! A vertex is a single event in the `RhizoCrypt` DAG. Each vertex is
//! content-addressed by its Blake3 hash and linked to parent vertices.

use bytes::Bytes;

use crate::event::EventType;
use crate::types::{Did, PayloadRef, Signature, Timestamp, VertexId};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// A single event in the `RhizoCrypt` DAG.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Vertex {
    /// Content-addressed identifier (computed from content).
    #[serde(skip)]
    id: Option<VertexId>,

    /// References to parent vertices (empty for genesis).
    pub parents: Vec<VertexId>,

    /// Timestamp of vertex creation.
    pub timestamp: Timestamp,

    /// The agent that created this vertex (Decentralized Identifier).
    pub agent: Option<Did>,

    /// Optional cryptographic signature from agent.
    pub signature: Option<Signature>,

    /// Event type identifier.
    pub event_type: EventType,

    /// Event payload reference (content-addressed).
    pub payload: Option<PayloadRef>,

    /// Inline metadata (small key-value pairs).
    #[serde(default)]
    pub metadata: HashMap<String, MetadataValue>,
}

impl Vertex {
    /// Compute the vertex ID from its content.
    ///
    /// The ID is the Blake3 hash of the canonical CBOR representation.
    ///
    /// # Errors
    ///
    /// Returns an error if serialization fails.
    pub fn compute_id(&self) -> crate::error::Result<VertexId> {
        let bytes = self.to_canonical_bytes()?;
        Ok(VertexId::from_bytes(&bytes))
    }

    /// Get or compute the vertex ID.
    ///
    /// # Errors
    ///
    /// Returns an error if serialization fails.
    pub fn id(&mut self) -> crate::error::Result<VertexId> {
        if let Some(id) = self.id {
            Ok(id)
        } else {
            let id = self.compute_id()?;
            self.id = Some(id);
            Ok(id)
        }
    }

    /// Get the vertex ID if already computed.
    #[must_use]
    pub const fn cached_id(&self) -> Option<VertexId> {
        self.id
    }

    /// Serialize to canonical CBOR bytes (for hashing).
    ///
    /// Returns `Bytes` for zero-copy downstream use (signing, storage, hashing).
    ///
    /// # Errors
    ///
    /// Returns an error if CBOR serialization fails.
    pub fn to_canonical_bytes(&self) -> crate::error::Result<bytes::Bytes> {
        let serializable = SerializableVertex {
            parents: &self.parents,
            timestamp: self.timestamp,
            agent: self.agent.as_ref(),
            event_type: &self.event_type,
            payload: self.payload.as_ref(),
            metadata: &self.metadata,
        };

        let mut buf = Vec::new();
        ciborium::into_writer(&serializable, &mut buf).map_err(|e| {
            crate::error::RhizoCryptError::internal(format!("vertex CBOR serialization: {e}"))
        })?;
        Ok(bytes::Bytes::from(buf))
    }

    /// Deserialize from CBOR bytes.
    ///
    /// # Errors
    ///
    /// Returns an error if deserialization fails.
    pub fn from_cbor_bytes(bytes: &[u8]) -> crate::error::Result<Self> {
        ciborium::from_reader(bytes).map_err(|e| {
            crate::error::RhizoCryptError::internal(format!("Failed to deserialize vertex: {e}"))
        })
    }

    /// Check if this is a genesis vertex (no parents).
    #[must_use]
    pub const fn is_genesis(&self) -> bool {
        self.parents.is_empty()
    }

    /// Check if this vertex is signed.
    #[must_use]
    pub const fn is_signed(&self) -> bool {
        self.signature.is_some()
    }

    /// Get metadata value by key.
    #[must_use]
    pub fn get_metadata(&self, key: &str) -> Option<&MetadataValue> {
        self.metadata.get(key)
    }
}

/// Serializable form of a vertex (excludes cached ID and signature).
#[derive(Serialize)]
struct SerializableVertex<'a> {
    parents: &'a [VertexId],
    timestamp: Timestamp,
    agent: Option<&'a Did>,
    event_type: &'a EventType,
    payload: Option<&'a PayloadRef>,
    metadata: &'a HashMap<String, MetadataValue>,
}

/// Generic metadata value type.
///
/// Note: Uses default tagged representation for bincode compatibility.
/// JSON will include the variant name (e.g., `{"String": "value"}`).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum MetadataValue {
    /// Null value.
    Null,
    /// Boolean value.
    Bool(bool),
    /// Integer value.
    Int(i64),
    /// Float value.
    Float(f64),
    /// String value.
    String(String),
    /// Bytes value (zero-copy via `bytes::Bytes`).
    Bytes(Bytes),
    /// Array value (boxed to reduce enum size disparity).
    Array(Box<Vec<Self>>),
    /// Object value (boxed to reduce enum size disparity).
    Object(Box<HashMap<String, Self>>),
}

impl From<bool> for MetadataValue {
    fn from(v: bool) -> Self {
        Self::Bool(v)
    }
}

impl From<i64> for MetadataValue {
    fn from(v: i64) -> Self {
        Self::Int(v)
    }
}

impl From<f64> for MetadataValue {
    fn from(v: f64) -> Self {
        Self::Float(v)
    }
}

impl From<String> for MetadataValue {
    fn from(v: String) -> Self {
        Self::String(v)
    }
}

impl From<&str> for MetadataValue {
    fn from(v: &str) -> Self {
        Self::String(v.to_string())
    }
}

/// Builder for creating vertices.
#[derive(Clone, Debug)]
pub struct VertexBuilder {
    parents: Vec<VertexId>,
    timestamp: Option<Timestamp>,
    agent: Option<Did>,
    event_type: EventType,
    payload: Option<PayloadRef>,
    metadata: HashMap<String, MetadataValue>,
}

impl VertexBuilder {
    /// Create a new builder with required event type.
    #[must_use]
    pub fn new(event_type: EventType) -> Self {
        Self {
            parents: Vec::new(),
            timestamp: None,
            agent: None,
            event_type,
            payload: None,
            metadata: HashMap::new(),
        }
    }

    /// Add a parent vertex.
    #[must_use]
    pub fn with_parent(mut self, parent: VertexId) -> Self {
        self.parents.push(parent);
        self
    }

    /// Add multiple parents.
    #[must_use]
    pub fn with_parents(mut self, parents: impl IntoIterator<Item = VertexId>) -> Self {
        self.parents.extend(parents);
        self
    }

    /// Set the timestamp (defaults to now).
    #[must_use]
    pub const fn with_timestamp(mut self, timestamp: Timestamp) -> Self {
        self.timestamp = Some(timestamp);
        self
    }

    /// Set the agent.
    #[must_use]
    pub fn with_agent(mut self, agent: Did) -> Self {
        self.agent = Some(agent);
        self
    }

    /// Set the payload reference.
    #[must_use]
    pub const fn with_payload(mut self, payload: PayloadRef) -> Self {
        self.payload = Some(payload);
        self
    }

    /// Add metadata.
    #[must_use]
    pub fn with_metadata(
        mut self,
        key: impl Into<String>,
        value: impl Into<MetadataValue>,
    ) -> Self {
        self.metadata.insert(key.into(), value.into());
        self
    }

    /// Build the vertex.
    #[must_use]
    pub fn build(self) -> Vertex {
        Vertex {
            id: None,
            parents: self.parents,
            timestamp: self.timestamp.unwrap_or_else(Timestamp::now),
            agent: self.agent,
            signature: None,
            event_type: self.event_type,
            payload: self.payload,
            metadata: self.metadata,
        }
    }
}

#[cfg(test)]
#[expect(clippy::unwrap_used, clippy::expect_used, reason = "test code")]
mod tests {
    use super::*;

    #[test]
    fn test_vertex_builder() {
        let vertex = VertexBuilder::new(EventType::SessionStart)
            .with_agent(Did::new("did:key:test"))
            .with_metadata("key", "value")
            .build();

        assert!(vertex.is_genesis());
        assert!(!vertex.is_signed());
        assert!(vertex.agent.is_some());
        assert_eq!(vertex.get_metadata("key"), Some(&MetadataValue::String("value".to_string())));
    }

    #[test]
    fn test_vertex_with_parents() {
        let parent_id = VertexId::from_bytes(b"parent");
        let vertex = VertexBuilder::new(EventType::DataCreate {
            schema: None,
        })
        .with_parent(parent_id)
        .build();

        assert!(!vertex.is_genesis());
        assert_eq!(vertex.parents.len(), 1);
        assert_eq!(vertex.parents[0], parent_id);
    }

    #[test]
    fn test_vertex_id_computation() {
        let mut vertex1 = VertexBuilder::new(EventType::SessionStart).build();
        let mut vertex2 = VertexBuilder::new(EventType::SessionStart).build();

        // Different timestamps should produce different IDs
        let id1 = vertex1.id().unwrap();
        let id2 = vertex2.id().unwrap();

        // IDs should be cached
        assert_eq!(vertex1.cached_id(), Some(id1));
        assert_eq!(vertex2.cached_id(), Some(id2));
    }

    #[test]
    fn test_vertex_canonical_bytes() {
        let vertex = VertexBuilder::new(EventType::SessionStart)
            .with_timestamp(Timestamp::from_nanos(12345))
            .build();

        let bytes = vertex.to_canonical_bytes().unwrap();
        assert!(!bytes.is_empty());

        // Same vertex should produce same bytes
        let vertex2 = VertexBuilder::new(EventType::SessionStart)
            .with_timestamp(Timestamp::from_nanos(12345))
            .build();
        assert_eq!(bytes, vertex2.to_canonical_bytes().unwrap());
    }

    #[test]
    fn test_metadata_value_from() {
        assert_eq!(MetadataValue::from(true), MetadataValue::Bool(true));
        assert_eq!(MetadataValue::from(42i64), MetadataValue::Int(42));
        assert_eq!(MetadataValue::from(1.618f64), MetadataValue::Float(1.618));
        assert_eq!(MetadataValue::from("test"), MetadataValue::String("test".to_string()));
    }

    #[test]
    fn test_vertex_bincode_serialization() {
        // Test that Vertex serializes correctly with bincode (used by tarpc)
        let vertex =
            VertexBuilder::new(EventType::SessionStart).with_metadata("key", "value").build();

        let bytes = bincode::serialize(&vertex).expect("serialize vertex");
        let restored: Vertex = bincode::deserialize(&bytes).expect("deserialize vertex");

        assert_eq!(vertex.event_type, restored.event_type);
        assert_eq!(vertex.timestamp, restored.timestamp);
        assert_eq!(vertex.parents, restored.parents);
        assert_eq!(vertex.metadata.len(), restored.metadata.len());
    }

    #[test]
    fn test_vertex_with_complex_event_bincode() {
        // Test complex event types serialize correctly
        use crate::types::Did;

        let vertex = VertexBuilder::new(EventType::DataTransfer {
            to: Did::new("did:example:recipient"),
        })
        .with_metadata("amount", 100i64)
        .build();

        let bytes = bincode::serialize(&vertex).expect("serialize");
        let restored: Vertex = bincode::deserialize(&bytes).expect("deserialize");

        assert_eq!(vertex.event_type, restored.event_type);
    }
}
