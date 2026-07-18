// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2024–2026 ecoPrimals Project

//! `rhizoCrypt` service library — shared logic for the `UniBin` entry point.
//!
//! Extracts server startup, discovery registration, and CLI types so they can
//! be tested without spawning a subprocess.

#![cfg_attr(not(test), forbid(unsafe_code))]

mod client;
mod config;
mod discovery;
mod doctor;
#[cfg(unix)]
mod neural_api;
mod shutdown;
mod startup;
#[cfg(unix)]
mod uds;

pub use client::run_client;
pub use config::{has_explicit_tcp_config, resolve_bind_addr};
pub use discovery::register_with_discovery;
pub use doctor::{
    DoctorCheck, DoctorCheckError, check_dag_engine, check_discovery_connectivity,
    check_storage_backend, run_doctor,
};
pub use startup::{run_server, run_server_with_ready};

#[cfg(test)]
pub(crate) use shutdown::shutdown_signal;

/// `UniBin` exit codes per the Architecture Standard.
pub mod exit_codes {
    /// Success.
    pub const SUCCESS: i32 = 0;
    /// General error.
    pub const GENERAL_ERROR: i32 = 1;
    /// Configuration error.
    pub const CONFIG_ERROR: i32 = 2;
    /// Network error.
    pub const NETWORK_ERROR: i32 = 3;
    /// Interrupted (SIGINT).
    pub const INTERRUPTED: i32 = 130;
}

use clap::Subcommand;
use thiserror::Error;

/// Client operations for interacting with a running rhizoCrypt server.
#[derive(Clone, Debug, Subcommand)]
pub enum ClientOperation {
    /// Check server health.
    Health,
    /// List active sessions.
    ListSessions,
    /// Get service metrics.
    Metrics,
}

/// Service-level error type.
#[derive(Debug, Error)]
pub enum ServiceError {
    /// Configuration error.
    #[error("configuration error: {0}")]
    Config(String),

    /// RPC server error.
    #[error("rpc server error: {0}")]
    Rpc(#[from] std::io::Error),

    /// Discovery registration failed.
    #[error("discovery registration failed: {0}")]
    Discovery(String),

    /// Address parse error.
    #[error("address parse error: {0}")]
    AddrParse(#[from] std::net::AddrParseError),

    /// Storage backend error (doctor check).
    #[error("storage error: {0}")]
    Storage(String),
}

impl ServiceError {
    /// Map this error to a `UniBin` exit code.
    #[must_use]
    pub const fn exit_code(&self) -> i32 {
        use crate::exit_codes;
        match self {
            Self::Config(_) | Self::AddrParse(_) | Self::Storage(_) => exit_codes::CONFIG_ERROR,
            Self::Rpc(_) | Self::Discovery(_) => exit_codes::NETWORK_ERROR,
        }
    }
}

/// Print version information.
pub fn print_version() {
    println!("rhizoCrypt v{}", env!("CARGO_PKG_VERSION"));
    println!("Edition: 2024");
    println!("License: AGPL-3.0-or-later");
    println!("Architecture: UniBin / ecoBin (Pure Rust)");
}

/// Print status information.
pub fn print_status() {
    println!("rhizoCrypt v{}", env!("CARGO_PKG_VERSION"));
    println!("Status: Not connected (use `rhizocrypt server` to start)");
    println!("License: AGPL-3.0-or-later");
}

#[cfg(test)]
#[expect(clippy::unwrap_used, clippy::expect_used, reason = "test code")]
#[path = "lib_tests.rs"]
mod tests;

#[cfg(test)]
#[expect(clippy::unwrap_used, clippy::expect_used, reason = "test code")]
#[path = "lib_tests_startup.rs"]
mod tests_startup;

#[cfg(test)]
#[expect(clippy::unwrap_used, clippy::expect_used, reason = "test code")]
#[path = "lib_tests_lifecycle.rs"]
mod tests_lifecycle;
