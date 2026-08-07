// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2024–2026 ecoPrimals Project

//! Platform-agnostic transport layer (G66 Transport Abstraction).
//!
//! Provides:
//! - [`TransportEndpoint`] — ecosystem-standard wire type for describing how
//!   to reach a service. Serde-tagged JSON, wire-compatible with the
//!   `sourDough`/`songBird`/`cellMembrane` canonical format.
//! - [`TransportStream`] — transport-agnostic connected stream.
//! - [`TransportListener`] — server-side transport abstraction.
//! - [`connect_transport`] — connect to a service via its resolved endpoint.
//! - Socket path utilities and BTSP helpers.

mod connect;
mod endpoint;
mod listener;
pub mod platform;
mod stream;

pub use connect::{JsonRpcTransportError, connect_transport, send_jsonrpc_request};
pub use endpoint::TransportEndpoint;
pub use listener::TransportListener;
pub use platform::{PlatformAccess, is_symlink_to, platform_link, set_platform_permissions};
pub use stream::TransportStream;

use std::path::{Path, PathBuf};

use crate::constants::{
    BIOMEOS_SOCKET_SUBDIR, DEFAULT_SOCKET_DIR, SOCKET_FILE_EXTENSION, TARPC_SOCKET_FILE_EXTENSION,
};
use crate::safe_env::SafeEnv;

// ============================================================================
// Socket path utilities
// ============================================================================

/// Fallback directory when `XDG_RUNTIME_DIR` is unset (Unix-like, non-Android).
fn unix_socket_dir_fallback() -> PathBuf {
    if cfg!(target_os = "linux") {
        PathBuf::from(DEFAULT_SOCKET_DIR)
    } else {
        std::env::temp_dir().join(BIOMEOS_SOCKET_SUBDIR)
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

    if let Some(runtime_dir) = SafeEnv::get_optional(SafeEnv::XDG_RUNTIME_DIR) {
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

/// Constructs a family-scoped tarpc UDS path (G64 C2 dual-socket pattern).
///
/// Returns `{socket_dir}/{name}[-{family_id}].tarpc.sock`. This socket
/// carries tarpc binary framing alongside the JSON-RPC `.sock`.
///
/// Returns `None` on platforms without path-based sockets.
#[must_use]
pub fn family_scoped_tarpc_socket_path(name: &str, primal_env_prefix: &str) -> Option<PathBuf> {
    let dir = socket_dir()?;
    let family_id = read_family_id(primal_env_prefix);
    let filename = family_id.map_or_else(
        || format!("{name}{TARPC_SOCKET_FILE_EXTENSION}"),
        |fid| format!("{name}-{fid}{TARPC_SOCKET_FILE_EXTENSION}"),
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
    let val =
        SafeEnv::get_optional(&primal_key).or_else(|| SafeEnv::get_optional(SafeEnv::FAMILY_ID))?;
    let val = val.trim().to_string();
    if val.is_empty() || val == "default" {
        None
    } else {
        Some(val)
    }
}

// ============================================================================
// BTSP environment helpers
// ============================================================================

/// Returns `true` when `BIOMEOS_INSECURE` is set to a truthy value (`1`, `true`, `yes`).
#[must_use]
pub fn is_biomeos_insecure() -> bool {
    SafeEnv::get_optional(SafeEnv::BIOMEOS_INSECURE)
        .is_some_and(|v| matches!(v.trim(), "1" | "true" | "yes"))
}

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

// ============================================================================
// Platform-specific utilities
// ============================================================================

/// Probe whether a Unix domain socket is alive by attempting a connection.
///
/// A `connect()` on a live listener succeeds in microseconds; a stale socket
/// file from a crashed process returns `ECONNREFUSED` immediately. This is
/// the ecosystem-standard liveness check (v1.1.5) — prefer over
/// `Path::exists` which cannot distinguish live from stale sockets.
#[cfg(unix)]
#[must_use]
pub fn socket_is_alive(path: &std::path::Path) -> bool {
    use std::os::unix::net::UnixStream;
    path.exists() && UnixStream::connect(path).is_ok()
}

#[cfg(test)]
#[expect(clippy::unwrap_used, reason = "test code")]
#[path = "../transport_tests/mod.rs"]
mod tests;
