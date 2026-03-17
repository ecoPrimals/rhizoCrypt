// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2024–2026 ecoPrimals Project

//! Test harness for rhizoCrypt integration tests.
//!
//! Provides a standardized way to set up and tear down test environments.

#![allow(dead_code)]
#![allow(clippy::unwrap_used, clippy::expect_used)]

use rhizo_crypt_core::{
    PrimalLifecycle, RhizoCrypt, RhizoCryptConfig, Session, SessionBuilder, SessionType,
};
use std::sync::Arc;
use tokio::sync::RwLock;

/// Test configuration.
#[derive(Debug, Clone)]
pub struct TestConfig {
    /// Base port for test RPC server.
    pub base_port: u16,
    /// Timeout for operations in seconds.
    pub timeout_secs: u64,
    /// Whether to cleanup after tests.
    pub cleanup: bool,
    /// Maximum sessions for testing.
    pub max_sessions: usize,
}

impl Default for TestConfig {
    fn default() -> Self {
        Self {
            base_port: 19400, // High port to avoid conflicts
            timeout_secs: 10,
            cleanup: true,
            max_sessions: 100,
        }
    }
}

/// Test harness for rhizoCrypt.
///
/// Manages lifecycle of a test primal instance.
pub struct TestHarness {
    /// The primal under test.
    primal: Arc<RwLock<RhizoCrypt>>,
    /// Test configuration.
    config: TestConfig,
}

impl TestHarness {
    /// Create a new test harness.
    pub fn new(config: TestConfig) -> Self {
        let primal_config = RhizoCryptConfig {
            max_sessions: config.max_sessions,
            rpc: rhizo_crypt_core::RpcConfig::with_addr("127.0.0.1", config.base_port),
            ..RhizoCryptConfig::default()
        };

        let primal = RhizoCrypt::new(primal_config);

        Self {
            primal: Arc::new(RwLock::new(primal)),
            config,
        }
    }

    /// Start the primal.
    pub async fn start(&self) -> Result<(), Box<dyn std::error::Error>> {
        self.primal.write().await.start().await?;
        Ok(())
    }

    /// Stop the primal.
    pub async fn stop(&self) -> Result<(), Box<dyn std::error::Error>> {
        self.primal.write().await.stop().await?;
        Ok(())
    }

    /// Create a test session.
    #[expect(
        clippy::unused_self,
        reason = "Self parameter for API consistency with other harness methods"
    )]
    pub fn create_test_session(&self, name: &str) -> Session {
        SessionBuilder::new(SessionType::General).with_name(name).build()
    }

    /// Get the primal reference.
    pub fn primal(&self) -> Arc<RwLock<RhizoCrypt>> {
        Arc::clone(&self.primal)
    }

    /// Get the config.
    pub const fn config(&self) -> &TestConfig {
        &self.config
    }
}

impl Drop for TestHarness {
    fn drop(&mut self) {
        // Cleanup is handled by the primal's Drop implementation
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn test_harness_lifecycle() {
        let config = TestConfig::default();
        let harness = TestHarness::new(config);

        harness.start().await.expect("harness should start");

        let session = harness.create_test_session("test-session");
        assert!(session.name.is_some());

        harness.stop().await.expect("harness should stop");
    }
}
