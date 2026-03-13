// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2024–2026 ecoPrimals Project

//! Songbird client submodules.

mod client;
mod config;
mod connection;
mod discovery;

// Re-export main types (same public API)
pub use client::SongbirdClient;
pub use config::SongbirdConfig;
