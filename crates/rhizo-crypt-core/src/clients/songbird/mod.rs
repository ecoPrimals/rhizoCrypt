// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2024–2026 ecoPrimals Project

//! Discovery adapter client — ecosystem service discovery and registration.
//!
//! This module provides the universal discovery adapter used to bootstrap
//! capability-based service resolution. The client sends generic `register`
//! and `heartbeat` JSON-RPC calls to whichever discovery service is deployed.

mod client;
mod config;
mod connection;
mod discovery;

pub use client::DiscoveryClient;
pub use config::DiscoveryConfig;
