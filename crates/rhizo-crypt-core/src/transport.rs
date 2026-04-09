// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2024–2026 ecoPrimals Project

//! Platform-agnostic transport selection (ecoBin v2.0).
//!
//! Provides runtime transport negotiation across platforms:
//! - Unix domain sockets (Linux, macOS, BSD)
//! - Abstract namespace sockets (Android)
//! - TCP fallback (Windows, unsupported platforms)
//!
//! The transport layer is decoupled from constants so that IPC selection
//! logic and platform detection can evolve independently from static values.

use std::path::{Path, PathBuf};

use crate::constants::{
    BIOMEOS_SOCKET_SUBDIR, DEFAULT_RPC_HOST, DEFAULT_SOCKET_DIR, SOCKET_FILE_EXTENSION,
};

/// Platform bucket for transport selection (used to keep negotiation logic testable on any host).
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum PlatformKind {
    Android,
    Windows,
    Unix,
}

impl PlatformKind {
    const fn current() -> Self {
        if cfg!(target_os = "android") {
            Self::Android
        } else if cfg!(target_os = "windows") {
            Self::Windows
        } else {
            Self::Unix
        }
    }
}

/// Fallback directory when `XDG_RUNTIME_DIR` is unset (Unix-like, non-Android).
fn unix_socket_dir_fallback() -> PathBuf {
    if cfg!(target_os = "linux") {
        PathBuf::from(DEFAULT_SOCKET_DIR)
    } else {
        std::env::temp_dir().join(BIOMEOS_SOCKET_SUBDIR)
    }
}

/// Unix-like transport from an optional socket path (TCP when no path is available).
fn unix_transport_from_socket_path(socket_path: Option<PathBuf>, port: u16) -> TransportHint {
    socket_path.map_or_else(
        || TransportHint::Tcp {
            host: DEFAULT_RPC_HOST.to_string(),
            port,
        },
        TransportHint::UnixSocket,
    )
}

fn preferred_transport_with_platform(
    primal_name: &str,
    port: u16,
    platform: PlatformKind,
) -> TransportHint {
    match platform {
        PlatformKind::Android => {
            TransportHint::AbstractSocket(format!("{BIOMEOS_SOCKET_SUBDIR}.{primal_name}"))
        }
        PlatformKind::Windows => TransportHint::Tcp {
            host: DEFAULT_RPC_HOST.to_string(),
            port,
        },
        PlatformKind::Unix => {
            unix_transport_from_socket_path(socket_path_for_primal(primal_name), port)
        }
    }
}

/// Returns the directory for path-based Unix sockets, or `None` on platforms
/// that use non-path transports (Android abstract sockets, Windows named pipes).
///
/// Platform behavior:
/// - **Linux/macOS/BSD**: Checks `XDG_RUNTIME_DIR` first; falls back to
///   `/run/biomeos` on Linux, `/tmp/biomeos` elsewhere.
/// - **Android**: Returns `None` (use abstract sockets).
/// - **Windows**: Returns `None` (use named pipes or TCP).
/// - **General fallback**: `/tmp/biomeos`.
#[must_use]
pub fn socket_dir() -> Option<PathBuf> {
    if cfg!(target_os = "android") || cfg!(target_os = "windows") {
        return None;
    }

    if let Ok(runtime_dir) = std::env::var("XDG_RUNTIME_DIR") {
        let path = Path::new(&runtime_dir).join(BIOMEOS_SOCKET_SUBDIR);
        return Some(path);
    }

    Some(unix_socket_dir_fallback())
}

/// Constructs the full socket path for a primal, or `None` if path-based
/// sockets are not available on this platform.
///
/// Returns `{socket_dir}/{name}.sock` when [`socket_dir()`] is `Some`.
/// For family-scoped sockets (BTSP Phase 1), use [`family_scoped_socket_path`].
#[must_use]
pub fn socket_path_for_primal(name: &str) -> Option<PathBuf> {
    let dir = socket_dir()?;
    let filename = format!("{name}{SOCKET_FILE_EXTENSION}");
    Some(dir.join(filename))
}

/// Constructs a BTSP Phase 1 family-scoped socket path.
///
/// When `FAMILY_ID` (or `{PRIMAL_ENV_PREFIX}_FAMILY_ID`) is set, returns
/// `{socket_dir}/{name}-{family_id}.sock`. When unset, falls back to
/// `{socket_dir}/{name}.sock` (development mode).
///
/// Returns `None` on platforms without path-based sockets.
#[must_use]
pub fn family_scoped_socket_path(name: &str, primal_env_prefix: &str) -> Option<PathBuf> {
    let dir = socket_dir()?;
    let family_id = read_family_id(primal_env_prefix);
    let filename = family_id.map_or_else(
        || format!("{name}{SOCKET_FILE_EXTENSION}"),
        |fid| format!("{name}-{fid}{SOCKET_FILE_EXTENSION}"),
    );
    Some(dir.join(filename))
}

/// Read `FAMILY_ID` from the environment, checking the primal-specific
/// override first (`{PREFIX}_FAMILY_ID`), then the ecosystem-wide `FAMILY_ID`.
///
/// Returns `None` if unset or the special value `"default"`.
#[must_use]
pub fn read_family_id(primal_env_prefix: &str) -> Option<String> {
    let primal_key = format!("{primal_env_prefix}_FAMILY_ID");
    let val = std::env::var(&primal_key).or_else(|_| std::env::var("FAMILY_ID")).ok()?;
    let val = val.trim().to_string();
    if val.is_empty() || val == "default" {
        None
    } else {
        Some(val)
    }
}

/// Returns `true` when `BIOMEOS_INSECURE` is set to a truthy value (`1`, `true`, `yes`).
#[must_use]
pub fn is_biomeos_insecure() -> bool {
    std::env::var("BIOMEOS_INSECURE").ok().is_some_and(|v| matches!(v.trim(), "1" | "true" | "yes"))
}

/// BTSP Phase 1 environment guard.
///
/// BTSP configuration error.
///
/// Returned when the environment violates BTSP Phase 1 invariants.
#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
pub enum BtspConfigError {
    /// `FAMILY_ID` (production) and `BIOMEOS_INSECURE` (development) are mutually exclusive.
    #[error(
        "BTSP conflict: FAMILY_ID is set (production mode) but BIOMEOS_INSECURE=1 \
         (development mode). These are mutually exclusive. \
         Unset BIOMEOS_INSECURE for production, or unset FAMILY_ID for development."
    )]
    FamilyInsecureConflict,
}

/// Validates that `FAMILY_ID` and `BIOMEOS_INSECURE` are not both set.
/// Per the BTSP protocol standard, this configuration is an error — the
/// primal MUST refuse to start.
///
/// # Errors
///
/// Returns [`BtspConfigError::FamilyInsecureConflict`] when the conflict is detected.
pub fn btsp_env_guard(primal_env_prefix: &str) -> Result<(), BtspConfigError> {
    let family = read_family_id(primal_env_prefix);
    let insecure = is_biomeos_insecure();

    if family.is_some() && insecure {
        return Err(BtspConfigError::FamilyInsecureConflict);
    }

    Ok(())
}

/// Transport hint for primal IPC, selected per-platform per ecoBin v2.0.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum TransportHint {
    /// Path-based Unix domain socket.
    UnixSocket(PathBuf),
    /// TCP connection (fallback or Windows).
    Tcp {
        /// Host to connect to.
        host: String,
        /// Port number.
        port: u16,
    },
    /// Abstract namespace socket (Android, Linux abstract).
    AbstractSocket(String),
}

/// Returns the preferred transport for the current platform.
///
/// Platform selection:
/// - **Linux/macOS/BSD**: Unix socket when path-based sockets are available;
///   otherwise TCP with localhost.
/// - **Android**: Abstract socket.
/// - **Windows**: TCP with localhost.
#[must_use]
pub fn preferred_transport(primal_name: &str, port: u16) -> TransportHint {
    preferred_transport_with_platform(primal_name, port, PlatformKind::current())
}

#[cfg(test)]
#[expect(clippy::unwrap_used, reason = "test code")]
#[path = "transport_tests.rs"]
mod tests;
