// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2024–2026 ecoPrimals Project

//! Fuzz target for Vertex CBOR deserialization.
//!
//! Ensures arbitrary byte sequences do not cause panics when parsed as Vertex.

#![no_main]

use libfuzzer_sys::fuzz_target;
use rhizo_crypt_core::Vertex;

fuzz_target!(|data: &[u8]| {
    // Attempt CBOR deserialization from arbitrary bytes — must not panic
    let _: Result<Vertex, _> = ciborium::de::from_reader(data);
});
