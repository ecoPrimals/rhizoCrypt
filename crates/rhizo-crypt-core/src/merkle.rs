// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2024–2026 ecoPrimals Project

//! Merkle tree structures for cryptographic proofs.
//!
//! This module provides Merkle root computation and proof generation/verification.

use crate::error::{Result, RhizoCryptError};
use crate::types::{hash_pair, ContentHash, VertexId};
use crate::vertex::Vertex;
use serde::{Deserialize, Serialize};

/// Merkle root of a session.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct MerkleRoot(pub ContentHash);

impl MerkleRoot {
    /// The zero Merkle root (empty session).
    pub const ZERO: Self = Self([0u8; 32]);

    /// Create a Merkle root from a hash.
    #[must_use]
    pub const fn new(hash: ContentHash) -> Self {
        Self(hash)
    }

    /// Compute Merkle root from vertices in topological order.
    ///
    /// The vertices must be provided in topological order (parents before children).
    ///
    /// # Errors
    ///
    /// Returns an error if any vertex fails to compute its ID.
    pub fn compute(vertices: &[Vertex]) -> Result<Self> {
        if vertices.is_empty() {
            return Ok(Self::ZERO);
        }

        // Compute leaf hashes
        let mut hashes: Vec<ContentHash> = vertices
            .iter()
            .map(|v| v.compute_id().map(|id| id.0))
            .collect::<std::result::Result<Vec<_>, _>>()?;

        // Pad to power of 2 if needed
        let target_len = hashes.len().next_power_of_two();
        while hashes.len() < target_len {
            hashes.push([0u8; 32]);
        }

        // Build tree bottom-up
        while hashes.len() > 1 {
            hashes = hashes.chunks(2).map(|chunk| hash_pair(&chunk[0], &chunk[1])).collect();
        }

        Ok(Self(hashes[0]))
    }

    /// Get the underlying hash.
    #[must_use]
    pub const fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }

    /// Convert to hex string.
    #[must_use]
    pub fn to_hex(&self) -> String {
        use std::fmt::Write;
        self.0.iter().fold(String::with_capacity(64), |mut s, b| {
            let _ = write!(s, "{b:02x}");
            s
        })
    }
}

impl std::fmt::Debug for MerkleRoot {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "MerkleRoot({})", &self.to_hex()[..16])
    }
}

impl std::fmt::Display for MerkleRoot {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", &self.to_hex()[..16])
    }
}

/// Direction in Merkle tree.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Direction {
    /// Sibling is on the left.
    Left,
    /// Sibling is on the right.
    Right,
}

/// Merkle proof for a vertex.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MerkleProof {
    /// The vertex being proven.
    pub vertex_id: VertexId,

    /// Position in topological order.
    pub position: usize,

    /// Total vertices in session.
    pub total_vertices: usize,

    /// Sibling hashes from leaf to root.
    pub siblings: Vec<(Direction, ContentHash)>,

    /// The Merkle root this proof validates against.
    pub root: MerkleRoot,
}

impl MerkleProof {
    /// Verify this proof against a vertex.
    ///
    /// Returns true if the proof is valid.
    #[must_use]
    pub fn verify(&self, vertex: &Vertex) -> bool {
        let Ok(vertex_hash) = vertex.compute_id() else {
            return false;
        };
        if vertex_hash != self.vertex_id {
            return false;
        }

        let mut current = vertex_hash.0;

        for (direction, sibling) in &self.siblings {
            current = match direction {
                Direction::Left => hash_pair(sibling, &current),
                Direction::Right => hash_pair(&current, sibling),
            };
        }

        current == self.root.0
    }

    /// Generate a Merkle proof for a vertex at a given position.
    ///
    /// # Errors
    ///
    /// Returns an error if the position is out of bounds.
    pub fn generate(vertices: &[Vertex], position: usize, root: MerkleRoot) -> Result<Self> {
        if position >= vertices.len() {
            return Err(RhizoCryptError::InvalidProof(format!(
                "position {position} out of bounds (len: {})",
                vertices.len()
            )));
        }

        let vertex = &vertices[position];
        let vertex_id = vertex.compute_id()?;

        // Compute leaf hashes
        let mut hashes: Vec<ContentHash> = vertices
            .iter()
            .map(|v| v.compute_id().map(|id| id.0))
            .collect::<std::result::Result<Vec<_>, _>>()?;

        let original_len = hashes.len();

        // Pad to power of 2
        let target_len = hashes.len().next_power_of_two();
        while hashes.len() < target_len {
            hashes.push([0u8; 32]);
        }

        // Build proof
        let mut siblings = Vec::new();
        let mut idx = position;

        while hashes.len() > 1 {
            let sibling_idx = if idx.is_multiple_of(2) {
                idx + 1
            } else {
                idx - 1
            };
            let direction = if idx.is_multiple_of(2) {
                Direction::Right
            } else {
                Direction::Left
            };

            siblings.push((direction, hashes[sibling_idx]));

            // Compute next level
            hashes = hashes.chunks(2).map(|chunk| hash_pair(&chunk[0], &chunk[1])).collect();

            idx /= 2;
        }

        Ok(Self {
            vertex_id,
            position,
            total_vertices: original_len,
            siblings,
            root,
        })
    }
}

/// Builder for Merkle tree from vertices.
#[derive(Debug, Default)]
pub struct MerkleTreeBuilder {
    vertices: Vec<Vertex>,
}

impl MerkleTreeBuilder {
    /// Create a new Merkle tree builder.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a vertex.
    pub fn add_vertex(&mut self, vertex: Vertex) {
        self.vertices.push(vertex);
    }

    /// Add multiple vertices.
    pub fn add_vertices(&mut self, vertices: impl IntoIterator<Item = Vertex>) {
        self.vertices.extend(vertices);
    }

    /// Compute the Merkle root.
    ///
    /// # Errors
    ///
    /// Returns an error if any vertex fails to compute its ID.
    pub fn compute_root(&self) -> Result<MerkleRoot> {
        MerkleRoot::compute(&self.vertices)
    }

    /// Generate a proof for a vertex at a given position.
    ///
    /// # Errors
    ///
    /// Returns an error if the position is out of bounds.
    pub fn generate_proof(&self, position: usize) -> Result<MerkleProof> {
        let root = self.compute_root()?;
        MerkleProof::generate(&self.vertices, position, root)
    }

    /// Get the number of vertices.
    #[must_use]
    pub const fn len(&self) -> usize {
        self.vertices.len()
    }

    /// Check if empty.
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.vertices.is_empty()
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;
    use crate::event::EventType;
    use crate::types::Timestamp;
    use crate::vertex::VertexBuilder;

    fn make_vertex(id: u8) -> Vertex {
        VertexBuilder::new(EventType::DataCreate {
            schema: Some(format!("schema{id}")),
        })
        .with_timestamp(Timestamp::from_nanos(u64::from(id)))
        .build()
    }

    #[test]
    fn test_merkle_root_empty() {
        let root = MerkleRoot::compute(&[]).unwrap();
        assert_eq!(root, MerkleRoot::ZERO);
    }

    #[test]
    fn test_merkle_root_single() {
        let vertex = make_vertex(1);
        let root = MerkleRoot::compute(&[vertex]).unwrap();
        assert_ne!(root, MerkleRoot::ZERO);
    }

    #[test]
    fn test_merkle_root_deterministic() {
        let vertices: Vec<Vertex> = (1..=4).map(make_vertex).collect();
        let root1 = MerkleRoot::compute(&vertices).unwrap();
        let root2 = MerkleRoot::compute(&vertices).unwrap();
        assert_eq!(root1, root2);
    }

    #[test]
    fn test_merkle_root_order_matters() {
        let v1 = make_vertex(1);
        let v2 = make_vertex(2);

        let root1 = MerkleRoot::compute(&[v1.clone(), v2.clone()]).unwrap();
        let root2 = MerkleRoot::compute(&[v2, v1]).unwrap();

        assert_ne!(root1, root2);
    }

    #[test]
    fn test_merkle_proof_generate_and_verify() {
        let vertices: Vec<Vertex> = (1..=4).map(make_vertex).collect();
        let root = MerkleRoot::compute(&vertices).unwrap();

        for (i, vertex) in vertices.iter().enumerate() {
            let proof = MerkleProof::generate(&vertices, i, root).unwrap();
            assert!(proof.verify(vertex));
            assert_eq!(proof.position, i);
            assert_eq!(proof.total_vertices, 4);
        }
    }

    #[test]
    fn test_merkle_proof_invalid_position() {
        let vertices: Vec<Vertex> = (1..=4).map(make_vertex).collect();
        let root = MerkleRoot::compute(&vertices).unwrap();

        let result = MerkleProof::generate(&vertices, 10, root);
        assert!(result.is_err());
    }

    #[test]
    fn test_merkle_proof_wrong_vertex() {
        let vertices: Vec<Vertex> = (1..=4).map(make_vertex).collect();
        let root = MerkleRoot::compute(&vertices).unwrap();

        let proof = MerkleProof::generate(&vertices, 0, root).unwrap();

        // Verify with wrong vertex should fail
        let wrong_vertex = make_vertex(99);
        assert!(!proof.verify(&wrong_vertex));
    }

    #[test]
    fn test_merkle_tree_builder() {
        let mut builder = MerkleTreeBuilder::new();
        assert!(builder.is_empty());

        for i in 1..=8 {
            builder.add_vertex(make_vertex(i));
        }

        assert_eq!(builder.len(), 8);

        let root = builder.compute_root().unwrap();
        assert_ne!(root, MerkleRoot::ZERO);

        let proof = builder.generate_proof(3).unwrap();
        assert!(proof.verify(&make_vertex(4)));
    }

    #[test]
    fn test_merkle_root_hex() {
        let vertex = make_vertex(1);
        let root = MerkleRoot::compute(&[vertex]).unwrap();
        let hex = root.to_hex();
        assert_eq!(hex.len(), 64); // 32 bytes = 64 hex chars
    }
}
