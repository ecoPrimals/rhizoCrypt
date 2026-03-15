// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2024–2026 ecoPrimals Project

//! Fuzz target for Merkle tree computation.
//!
//! Verifies that `MerkleRoot::compute` handles arbitrary vertex configurations
//! without panics and produces consistent results.

#![no_main]

use arbitrary::Arbitrary;
use libfuzzer_sys::fuzz_target;
use rhizo_crypt_core::vertex::VertexBuilder;
use rhizo_crypt_core::{EventType, MerkleRoot};

#[derive(Debug, Arbitrary)]
struct FuzzInput {
    vertex_count: u8,
    parent_indices: Vec<u8>,
    payloads: Vec<Vec<u8>>,
}

fuzz_target!(|input: FuzzInput| {
    let count = (input.vertex_count % 64) as usize;
    if count == 0 {
        let root = MerkleRoot::compute(&[]);
        assert!(root.is_ok());
        return;
    }

    let mut vertices = Vec::with_capacity(count);
    for i in 0..count {
        let mut builder = VertexBuilder::new(EventType::Mutation);

        // Add parent references (only to already-created vertices)
        if i > 0 {
            if let Some(&idx) = input.parent_indices.get(i) {
                let parent_idx = (idx as usize) % i;
                if let Ok(parent_id) = vertices[parent_idx].compute_id() {
                    builder = builder.with_parent(parent_id);
                }
            }
        }

        vertices.push(builder.build());
    }

    // Must not panic regardless of input
    let result = MerkleRoot::compute(&vertices);
    if let Ok(root) = &result {
        // Determinism: same input must always produce the same root
        let result2 = MerkleRoot::compute(&vertices);
        assert_eq!(result.as_ref().ok(), result2.as_ref().ok());
        assert_ne!(root.as_bytes(), &[0u8; 32]);
    }
});
