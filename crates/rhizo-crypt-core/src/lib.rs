//! # RhizoCrypt
//!
//! Core DAG Engine - Ephemeral Working Memory
//!
//! ## Overview
//!
//! RhizoCrypt is part of the ecoPrimals ecosystem. It provides the git-like
//! DAG engine that underlies Phase 2's memory and attribution layer.
//!
//! ## Quick Start
//!
//! ```rust,ignore
//! use rhizo_crypt_core::RhizoCrypt;
//!
//! let primal = RhizoCrypt::new(config);
//! primal.start().await?;
//! ```

#![warn(missing_docs)]
#![warn(clippy::all)]
#![warn(clippy::pedantic)]

pub mod config;
pub mod error;

use sourdough_core::{
    PrimalLifecycle, PrimalHealth, PrimalState,
    HealthStatus, health::HealthReport, PrimalError,
};

/// RhizoCrypt configuration.
pub use config::RhizoCryptConfig;

/// RhizoCrypt errors.
pub use error::RhizoCryptError;

/// The RhizoCrypt primal - Core DAG Engine.
pub struct RhizoCrypt {
    #[allow(dead_code)]
    config: RhizoCryptConfig,
    state: PrimalState,
}

impl RhizoCrypt {
    /// Create a new RhizoCrypt instance.
    #[must_use]
    pub fn new(config: RhizoCryptConfig) -> Self {
        Self {
            config,
            state: PrimalState::Created,
        }
    }
}

impl PrimalLifecycle for RhizoCrypt {
    fn state(&self) -> PrimalState {
        self.state
    }

    async fn start(&mut self) -> Result<(), PrimalError> {
        self.state = PrimalState::Starting;
        tracing::info!("RhizoCrypt starting...");
        
        // TODO: Initialize resources
        
        self.state = PrimalState::Running;
        tracing::info!("RhizoCrypt running");
        Ok(())
    }

    async fn stop(&mut self) -> Result<(), PrimalError> {
        self.state = PrimalState::Stopping;
        tracing::info!("RhizoCrypt stopping...");
        
        // TODO: Clean up resources
        
        self.state = PrimalState::Stopped;
        tracing::info!("RhizoCrypt stopped");
        Ok(())
    }
}

impl PrimalHealth for RhizoCrypt {
    fn health_status(&self) -> HealthStatus {
        if self.state.is_running() {
            HealthStatus::Healthy
        } else {
            HealthStatus::Unhealthy {
                reason: format!("state: {}", self.state),
            }
        }
    }

    async fn health_check(&self) -> Result<HealthReport, PrimalError> {
        Ok(HealthReport::new("RhizoCrypt", env!("CARGO_PKG_VERSION"))
            .with_status(self.health_status()))
    }
}
