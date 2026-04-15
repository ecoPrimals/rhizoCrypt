// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2024–2026 ecoPrimals Project

//! Common test utilities and helpers for rhizoCrypt E2E and chaos testing.
//!
//! This module provides shared infrastructure for integration testing across
//! all test suites.

pub mod harness;

/// Chaos testing configuration.
///
/// Available for chaos test suites; not all consumers reference every field.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ChaosConfig {
    /// Network latency to inject (ms).
    pub network_latency_ms: Option<u64>,
    /// Failure injection rate (0.0-1.0).
    pub failure_rate: f64,
    /// Whether to inject random failures.
    pub inject_failures: bool,
    /// Maximum concurrent operations before throttling.
    pub max_concurrent_ops: usize,
}

#[allow(dead_code)]
impl Default for ChaosConfig {
    fn default() -> Self {
        Self {
            network_latency_ms: None,
            failure_rate: 0.1,
            inject_failures: false,
            max_concurrent_ops: 100,
        }
    }
}
