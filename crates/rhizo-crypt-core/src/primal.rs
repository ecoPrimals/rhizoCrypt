// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2024–2026 ecoPrimals Project

//! Primal lifecycle and health traits.
//!
//! These traits define the standard interface for ecoPrimals lifecycle
//! management and health monitoring.

use std::fmt;

/// State of a primal in its lifecycle.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum PrimalState {
    /// Initial state after construction.
    #[default]
    Created,
    /// Primal is starting up.
    Starting,
    /// Primal is running and accepting requests.
    Running,
    /// Primal is stopping.
    Stopping,
    /// Primal has stopped.
    Stopped,
    /// Primal encountered an error.
    Failed,
}

impl PrimalState {
    /// Returns true if the primal is in a running state.
    #[must_use]
    pub const fn is_running(&self) -> bool {
        matches!(self, Self::Running)
    }

    /// Returns true if the primal is in a terminal state.
    #[must_use]
    pub const fn is_terminal(&self) -> bool {
        matches!(self, Self::Stopped | Self::Failed)
    }
}

impl fmt::Display for PrimalState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Created => write!(f, "created"),
            Self::Starting => write!(f, "starting"),
            Self::Running => write!(f, "running"),
            Self::Stopping => write!(f, "stopping"),
            Self::Stopped => write!(f, "stopped"),
            Self::Failed => write!(f, "failed"),
        }
    }
}

/// Error type for primal operations.
#[derive(Debug, thiserror::Error)]
pub enum PrimalError {
    /// Invalid state transition.
    #[error("invalid state transition from {from} to {to}")]
    InvalidTransition {
        /// Current state.
        from: PrimalState,
        /// Attempted state.
        to: PrimalState,
    },

    /// Startup failed.
    #[error("startup failed: {0}")]
    StartupFailed(String),

    /// Shutdown failed.
    #[error("shutdown failed: {0}")]
    ShutdownFailed(String),

    /// Operation failed.
    #[error("operation failed: {0}")]
    OperationFailed(String),
}

/// Lifecycle management for primals.
pub trait PrimalLifecycle {
    /// Returns the current state.
    fn state(&self) -> PrimalState;

    /// Start the primal.
    ///
    /// # Errors
    ///
    /// Returns an error if startup fails.
    fn start(&mut self) -> impl std::future::Future<Output = Result<(), PrimalError>> + Send;

    /// Stop the primal.
    ///
    /// # Errors
    ///
    /// Returns an error if shutdown fails.
    fn stop(&mut self) -> impl std::future::Future<Output = Result<(), PrimalError>> + Send;
}

/// Health status of a primal.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum HealthStatus {
    /// Primal is healthy.
    Healthy,
    /// Primal is degraded but functional.
    Degraded {
        /// Reason for degradation.
        reason: String,
    },
    /// Primal is unhealthy.
    Unhealthy {
        /// Reason for being unhealthy.
        reason: String,
    },
}

impl HealthStatus {
    /// Returns true if the primal is healthy.
    #[must_use]
    pub const fn is_healthy(&self) -> bool {
        matches!(self, Self::Healthy)
    }
}

/// Health report for a primal.
#[derive(Clone, Debug)]
pub struct HealthReport {
    /// Name of the primal.
    pub name: String,
    /// Version of the primal.
    pub version: String,
    /// Current health status.
    pub status: HealthStatus,
    /// Uptime in seconds.
    pub uptime_secs: Option<u64>,
}

impl HealthReport {
    /// Create a new health report.
    #[must_use]
    pub fn new(name: impl Into<String>, version: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            version: version.into(),
            status: HealthStatus::Healthy,
            uptime_secs: None,
        }
    }

    /// Set the health status.
    #[must_use]
    pub fn with_status(mut self, status: HealthStatus) -> Self {
        self.status = status;
        self
    }

    /// Set the uptime.
    #[must_use]
    pub const fn with_uptime(mut self, uptime_secs: u64) -> Self {
        self.uptime_secs = Some(uptime_secs);
        self
    }
}

/// Health monitoring for primals.
pub trait PrimalHealth {
    /// Returns the current health status.
    fn health_status(&self) -> HealthStatus;

    /// Perform a health check and return a report.
    ///
    /// # Errors
    ///
    /// Returns an error if the health check fails.
    fn health_check(
        &self,
    ) -> impl std::future::Future<Output = Result<HealthReport, PrimalError>> + Send;
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;

    #[test]
    fn test_primal_state_display() {
        assert_eq!(PrimalState::Created.to_string(), "created");
        assert_eq!(PrimalState::Running.to_string(), "running");
        assert_eq!(PrimalState::Stopped.to_string(), "stopped");
    }

    #[test]
    fn test_primal_state_predicates() {
        assert!(!PrimalState::Created.is_running());
        assert!(PrimalState::Running.is_running());
        assert!(!PrimalState::Stopped.is_running());

        assert!(!PrimalState::Running.is_terminal());
        assert!(PrimalState::Stopped.is_terminal());
        assert!(PrimalState::Failed.is_terminal());
    }

    #[test]
    fn test_health_status() {
        assert!(HealthStatus::Healthy.is_healthy());
        assert!(!HealthStatus::Degraded {
            reason: "test".into()
        }
        .is_healthy());
        assert!(!HealthStatus::Unhealthy {
            reason: "test".into()
        }
        .is_healthy());
    }

    #[test]
    fn test_health_report_builder() {
        let report = HealthReport::new("TestPrimal", "1.0.0")
            .with_status(HealthStatus::Healthy)
            .with_uptime(3600);

        assert_eq!(report.name, "TestPrimal");
        assert_eq!(report.version, "1.0.0");
        assert!(report.status.is_healthy());
        assert_eq!(report.uptime_secs, Some(3600));
    }
}
