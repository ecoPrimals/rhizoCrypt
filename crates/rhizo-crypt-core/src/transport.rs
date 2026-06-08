// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2024–2026 ecoPrimals Project

//! Platform-agnostic transport selection (ecoBin v2.0).
//!
//! Provides:
//! - [`TransportEndpoint`] — ecosystem-standard wire type for describing how
//!   to reach a service. Serde-tagged JSON, wire-compatible with the
//!   `sourDough`/`songBird`/`cellMembrane` canonical format.
//! - [`connect_transport`] — connect to a service via its resolved endpoint.
//! - [`TransportStream`] — transport-agnostic connected stream.
//! - [`TransportHint`] — legacy platform detection (ecoBin v2.0).
//! - BTSP helpers (family-scoped socket paths, insecure mode guard).

use std::path::{Path, PathBuf};

use crate::constants::{
    BIOMEOS_SOCKET_SUBDIR, DEFAULT_RPC_HOST, DEFAULT_SOCKET_DIR, SOCKET_FILE_EXTENSION,
};
use crate::safe_env::SafeEnv;

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

/// Returns `true` when `BIOMEOS_INSECURE` is set to a truthy value (`1`, `true`, `yes`).
#[must_use]
pub fn is_biomeos_insecure() -> bool {
    SafeEnv::get_optional(SafeEnv::BIOMEOS_INSECURE)
        .is_some_and(|v| matches!(v.trim(), "1" | "true" | "yes"))
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

// ============================================================================
// TransportEndpoint — ecosystem canonical wire type
// ============================================================================

/// Structured transport endpoint — wire-compatible with the ecosystem standard.
///
/// ```json
/// { "transport": "uds", "path": "/run/membrane/beardog.sock" }
/// { "transport": "tcp", "host": "192.168.1.144", "port": 7700 }
/// { "transport": "mesh_relay", "peer_id": "strand-gate", "capability": "security" }
/// ```
#[derive(Clone, Debug, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
#[serde(tag = "transport")]
pub enum TransportEndpoint {
    /// Unix Domain Socket.
    #[serde(rename = "uds")]
    Uds {
        /// Filesystem path to the socket.
        path: String,
    },
    /// TCP connection.
    #[serde(rename = "tcp")]
    Tcp {
        /// Host address.
        host: String,
        /// TCP port number.
        port: u16,
    },
    /// Mesh relay via Songbird.
    #[serde(rename = "mesh_relay")]
    MeshRelay {
        /// Mesh peer identifier.
        peer_id: String,
        /// Capability being resolved.
        capability: String,
    },
}

impl TransportEndpoint {
    /// Construct a TCP endpoint.
    #[must_use]
    pub fn tcp(host: impl Into<String>, port: u16) -> Self {
        Self::Tcp {
            host: host.into(),
            port,
        }
    }

    /// Returns `(host, port)` if this is a TCP endpoint.
    #[must_use]
    pub fn tcp_addr(&self) -> Option<(&str, u16)> {
        match self {
            Self::Tcp { host, port } => Some((host, *port)),
            _ => None,
        }
    }
}

impl std::fmt::Display for TransportEndpoint {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Uds { path } => write!(f, "unix://{path}"),
            Self::Tcp { host, port } => write!(f, "tcp://{host}:{port}"),
            Self::MeshRelay {
                peer_id,
                capability,
            } => write!(f, "mesh://{peer_id}/{capability}"),
        }
    }
}

// ============================================================================
// TransportStream — transport-agnostic connected stream
// ============================================================================

/// A transport-agnostic connected stream implementing `AsyncRead + AsyncWrite`.
pub enum TransportStream {
    /// Connected Unix domain socket.
    #[cfg(unix)]
    Unix(tokio::net::UnixStream),
    /// Connected TCP stream.
    Tcp(tokio::net::TcpStream),
}

impl tokio::io::AsyncRead for TransportStream {
    fn poll_read(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
        buf: &mut tokio::io::ReadBuf<'_>,
    ) -> std::task::Poll<std::io::Result<()>> {
        match self.get_mut() {
            #[cfg(unix)]
            Self::Unix(s) => std::pin::Pin::new(s).poll_read(cx, buf),
            Self::Tcp(s) => std::pin::Pin::new(s).poll_read(cx, buf),
        }
    }
}

impl tokio::io::AsyncWrite for TransportStream {
    fn poll_write(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
        buf: &[u8],
    ) -> std::task::Poll<std::io::Result<usize>> {
        match self.get_mut() {
            #[cfg(unix)]
            Self::Unix(s) => std::pin::Pin::new(s).poll_write(cx, buf),
            Self::Tcp(s) => std::pin::Pin::new(s).poll_write(cx, buf),
        }
    }

    fn poll_flush(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<std::io::Result<()>> {
        match self.get_mut() {
            #[cfg(unix)]
            Self::Unix(s) => std::pin::Pin::new(s).poll_flush(cx),
            Self::Tcp(s) => std::pin::Pin::new(s).poll_flush(cx),
        }
    }

    fn poll_shutdown(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<std::io::Result<()>> {
        match self.get_mut() {
            #[cfg(unix)]
            Self::Unix(s) => std::pin::Pin::new(s).poll_shutdown(cx),
            Self::Tcp(s) => std::pin::Pin::new(s).poll_shutdown(cx),
        }
    }
}

// ============================================================================
// connect_transport — connect to a resolved endpoint
// ============================================================================

/// Connect to a service via its resolved [`TransportEndpoint`].
///
/// # Errors
///
/// Returns `io::Error` on connection failure. `MeshRelay` endpoints require
/// routing through Songbird and are not directly connectable.
pub async fn connect_transport(endpoint: &TransportEndpoint) -> std::io::Result<TransportStream> {
    match endpoint {
        #[cfg(unix)]
        TransportEndpoint::Uds { path } => {
            let stream = tokio::net::UnixStream::connect(path).await?;
            Ok(TransportStream::Unix(stream))
        }
        #[cfg(not(unix))]
        TransportEndpoint::Uds { path } => Err(std::io::Error::new(
            std::io::ErrorKind::Unsupported,
            format!("UDS not available on this platform for {path}"),
        )),
        TransportEndpoint::Tcp { host, port } => {
            let addr = format!("{host}:{port}");
            let stream = tokio::net::TcpStream::connect(&addr).await?;
            Ok(TransportStream::Tcp(stream))
        }
        TransportEndpoint::MeshRelay {
            peer_id,
            capability,
        } => Err(std::io::Error::new(
            std::io::ErrorKind::Unsupported,
            format!("mesh relay ({peer_id}/{capability}) requires Songbird routing"),
        )),
    }
}

// ============================================================================
// TransportHint — legacy platform detection
// ============================================================================

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
