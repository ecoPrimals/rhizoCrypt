// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2024–2026 ecoPrimals Project

//! Songbird client submodules.

mod client;
mod config;
mod connection;
mod discovery;

// Re-export main types (same public API)
pub use client::SongbirdClient;
pub use config::SongbirdConfig;

/// Capability-neutral alias for the ecosystem discovery adapter.
///
/// Service code should import `DiscoveryClient` so it never depends on a
/// specific primal name. The underlying client sends generic `register` +
/// `heartbeat` JSON-RPC calls.
pub type DiscoveryClient = SongbirdClient;

/// Capability-neutral alias for [`SongbirdConfig`].
pub type DiscoveryConfig = SongbirdConfig;
