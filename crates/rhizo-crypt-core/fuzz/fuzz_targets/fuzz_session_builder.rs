// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2024–2026 ecoPrimals Project

//! Fuzz target for session construction.
//!
//! Exercises `SessionBuilder` with arbitrary configuration values to ensure
//! no panics from edge-case inputs (zero durations, extreme limits, etc.).

#![no_main]

use arbitrary::Arbitrary;
use libfuzzer_sys::fuzz_target;
use rhizo_crypt_core::session::{SessionBuilder, SessionType};
use std::time::Duration;

#[derive(Debug, Arbitrary)]
struct FuzzSessionConfig {
    session_type_idx: u8,
    name: Option<String>,
    max_duration_secs: u64,
    max_vertices: u64,
    max_payload_bytes: u64,
    require_signatures: bool,
}

fuzz_target!(|config: FuzzSessionConfig| {
    let session_type = match config.session_type_idx % 4 {
        0 => SessionType::Annotation,
        1 => SessionType::Computation,
        2 => SessionType::Collaboration,
        _ => SessionType::Governance,
    };

    let mut builder = SessionBuilder::new(session_type);

    if let Some(name) = &config.name {
        builder = builder.with_name(name);
    }

    builder = builder.with_max_duration(Duration::from_secs(config.max_duration_secs));

    let session = builder.build();

    // Session must always be constructable without panic
    let _ = session.id;
    let _ = session.name;
});
