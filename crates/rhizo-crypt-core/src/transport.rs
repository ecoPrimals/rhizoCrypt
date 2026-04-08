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
mod tests {
    use super::*;

    #[test]
    fn test_socket_dir_respects_xdg_runtime_dir() {
        let temp = tempfile::tempdir().expect("temp dir");
        let runtime_path = temp.path().to_path_buf();
        let path_str = runtime_path.to_str().unwrap();
        temp_env::with_vars([("XDG_RUNTIME_DIR", Some(path_str))], || {
            let result = socket_dir();
            if cfg!(target_os = "android") || cfg!(target_os = "windows") {
                assert!(result.is_none(), "Android/Windows should return None");
            } else {
                let dir = result.expect("Unix-like should return Some");
                assert_eq!(dir, runtime_path.join("biomeos"));
            }
        });
    }

    #[test]
    fn test_socket_dir_fallback_without_xdg() {
        temp_env::with_vars([("XDG_RUNTIME_DIR", None::<&str>)], || {
            let result = socket_dir();
            if cfg!(target_os = "android") || cfg!(target_os = "windows") {
                assert!(result.is_none());
            } else if cfg!(target_os = "linux") {
                assert_eq!(result, Some(PathBuf::from(DEFAULT_SOCKET_DIR)));
            } else {
                assert_eq!(result, Some(std::env::temp_dir().join("biomeos")));
            }
        });
    }

    #[test]
    fn test_socket_path_for_primal_unix_like() {
        if cfg!(target_os = "android") || cfg!(target_os = "windows") {
            let result = socket_path_for_primal("rhizoCrypt");
            assert!(result.is_none());
            return;
        }

        let temp = tempfile::tempdir().expect("temp dir");
        let path_str = temp.path().to_str().unwrap();
        temp_env::with_vars([("XDG_RUNTIME_DIR", Some(path_str))], || {
            let path = socket_path_for_primal("rhizoCrypt").expect("should return path");
            assert!(path.ends_with("rhizoCrypt.sock"));
            assert_eq!(path.file_name().unwrap(), "rhizoCrypt.sock");
        });
    }

    #[test]
    fn test_socket_path_for_primal_uses_socket_extension() {
        if cfg!(target_os = "android") || cfg!(target_os = "windows") {
            return;
        }

        let path = socket_path_for_primal("testPrimal").expect("should return path");
        assert!(path.to_string_lossy().ends_with(".sock"));
    }

    #[test]
    fn test_transport_hint_android_abstract() {
        if !cfg!(target_os = "android") {
            return;
        }

        let hint = preferred_transport("rhizoCrypt", 9400);
        assert!(matches!(hint, TransportHint::AbstractSocket(_)));
        if let TransportHint::AbstractSocket(name) = hint {
            assert_eq!(name, "biomeos.rhizoCrypt");
        }
    }

    #[test]
    fn test_transport_hint_windows_tcp() {
        if !cfg!(target_os = "windows") {
            return;
        }

        let hint = preferred_transport("rhizoCrypt", 9400);
        assert!(matches!(hint, TransportHint::Tcp { .. }));
        if let TransportHint::Tcp {
            host,
            port,
        } = hint
        {
            assert_eq!(host, "127.0.0.1");
            assert_eq!(port, 9400);
        }
    }

    #[test]
    fn test_transport_hint_unix_socket_when_available() {
        if cfg!(target_os = "android") || cfg!(target_os = "windows") {
            return;
        }

        let temp = tempfile::tempdir().expect("temp dir");
        let path_str = temp.path().to_str().unwrap();
        temp_env::with_vars([("XDG_RUNTIME_DIR", Some(path_str))], || {
            let hint = preferred_transport("rhizoCrypt", 9400);
            assert!(matches!(hint, TransportHint::UnixSocket(_)));
            if let TransportHint::UnixSocket(path) = hint {
                assert!(path.ends_with("rhizoCrypt.sock"));
            }
        });
    }

    #[test]
    fn test_transport_hint_tcp_fallback() {
        if cfg!(target_os = "android") || cfg!(target_os = "windows") {
            return;
        }

        temp_env::with_vars([("XDG_RUNTIME_DIR", None::<&str>)], || {
            let hint = preferred_transport("rhizoCrypt", 9400);
            match &hint {
                TransportHint::UnixSocket(path) => {
                    assert!(
                        path.starts_with(Path::new(DEFAULT_SOCKET_DIR))
                            || path.starts_with(std::env::temp_dir())
                    );
                }
                TransportHint::Tcp {
                    host,
                    port,
                } => {
                    assert_eq!(host, "127.0.0.1");
                    assert_eq!(*port, 9400);
                }
                TransportHint::AbstractSocket(_) => panic!("Unexpected AbstractSocket on Unix"),
            }
        });
    }

    #[test]
    fn test_transport_hint_equality() {
        let tcp1 = TransportHint::Tcp {
            host: "127.0.0.1".to_string(),
            port: 9400,
        };
        let tcp2 = TransportHint::Tcp {
            host: "127.0.0.1".to_string(),
            port: 9400,
        };
        assert_eq!(tcp1, tcp2);

        let abstract1 = TransportHint::AbstractSocket("biomeos.test".to_string());
        let abstract2 = TransportHint::AbstractSocket("biomeos.test".to_string());
        assert_eq!(abstract1, abstract2);
    }

    #[test]
    fn test_socket_dir_xdg_runtime_dir_empty_string() {
        if cfg!(target_os = "android") || cfg!(target_os = "windows") {
            return;
        }
        temp_env::with_vars([("XDG_RUNTIME_DIR", Some(""))], || {
            let result = socket_dir();
            let dir = result.expect("Empty XDG_RUNTIME_DIR still yields Some on Unix");
            assert_eq!(dir, PathBuf::from("biomeos"));
        });
    }

    #[test]
    fn test_socket_path_for_primal_empty_name() {
        if cfg!(target_os = "android") || cfg!(target_os = "windows") {
            return;
        }
        let path = socket_path_for_primal("").expect("should return path");
        assert!(path.to_string_lossy().ends_with(".sock"));
        assert_eq!(path.file_name().unwrap(), ".sock");
    }

    #[test]
    fn test_socket_path_for_primal_special_characters() {
        if cfg!(target_os = "android") || cfg!(target_os = "windows") {
            return;
        }
        let path = socket_path_for_primal("my-primal.v2").expect("should return path");
        assert!(path.to_string_lossy().ends_with("my-primal.v2.sock"));
        assert_eq!(path.file_name().unwrap(), "my-primal.v2.sock");
    }

    #[test]
    fn test_transport_hint_debug_format() {
        let unix = TransportHint::UnixSocket(PathBuf::from("/run/biomeos/rhizoCrypt.sock"));
        let debug_str = format!("{unix:?}");
        assert!(debug_str.contains("UnixSocket"));
        assert!(debug_str.contains("rhizoCrypt.sock"));

        let tcp = TransportHint::Tcp {
            host: "127.0.0.1".to_string(),
            port: 9400,
        };
        let debug_str = format!("{tcp:?}");
        assert!(debug_str.contains("Tcp"));
        assert!(debug_str.contains("127.0.0.1"));
        assert!(debug_str.contains("9400"));

        let abstract_sock = TransportHint::AbstractSocket("biomeos.test".to_string());
        let debug_str = format!("{abstract_sock:?}");
        assert!(debug_str.contains("AbstractSocket"));
        assert!(debug_str.contains("biomeos.test"));
    }

    #[test]
    fn test_transport_hint_unix_socket_equality() {
        let p1 = PathBuf::from("/run/biomeos/a.sock");
        let p2 = PathBuf::from("/run/biomeos/a.sock");
        let u1 = TransportHint::UnixSocket(p1);
        let u2 = TransportHint::UnixSocket(p2);
        assert_eq!(u1, u2);
    }

    #[test]
    fn test_preferred_transport_empty_primal_name() {
        if cfg!(target_os = "android") {
            let hint = preferred_transport("", 9400);
            assert!(matches!(hint, TransportHint::AbstractSocket(_)));
            if let TransportHint::AbstractSocket(name) = hint {
                assert_eq!(name, "biomeos.");
            }
            return;
        }
        if cfg!(target_os = "windows") {
            let hint = preferred_transport("", 9400);
            assert!(matches!(hint, TransportHint::Tcp { .. }));
            return;
        }
        let hint = preferred_transport("", 9400);
        match hint {
            TransportHint::UnixSocket(p) => assert!(p.to_string_lossy().ends_with(".sock")),
            TransportHint::Tcp {
                host,
                port,
            } => {
                assert_eq!(host, "127.0.0.1");
                assert_eq!(port, 9400);
            }
            TransportHint::AbstractSocket(_) => panic!("Unexpected AbstractSocket on Unix"),
        }
    }

    #[test]
    fn test_socket_path_for_primal_none_on_unsupported_platform() {
        if !cfg!(target_os = "android") && !cfg!(target_os = "windows") {
            return;
        }
        assert!(socket_path_for_primal("rhizoCrypt").is_none());
        assert!(socket_path_for_primal("any").is_none());
    }

    #[test]
    fn test_platform_kind_current() {
        let kind = PlatformKind::current();
        if cfg!(target_os = "android") {
            assert_eq!(kind, PlatformKind::Android);
        } else if cfg!(target_os = "windows") {
            assert_eq!(kind, PlatformKind::Windows);
        } else {
            assert_eq!(kind, PlatformKind::Unix);
        }
    }

    #[test]
    fn test_preferred_transport_android_platform() {
        let hint = preferred_transport_with_platform("rhizoCrypt", 9400, PlatformKind::Android);
        assert_eq!(hint, TransportHint::AbstractSocket("biomeos.rhizoCrypt".to_string()));
    }

    #[test]
    fn test_preferred_transport_windows_platform() {
        let hint = preferred_transport_with_platform("rhizoCrypt", 9400, PlatformKind::Windows);
        assert_eq!(
            hint,
            TransportHint::Tcp {
                host: DEFAULT_RPC_HOST.to_string(),
                port: 9400,
            }
        );
    }

    #[test]
    fn test_preferred_transport_unix_platform() {
        let hint = preferred_transport_with_platform("rhizoCrypt", 9400, PlatformKind::Unix);
        match hint {
            TransportHint::UnixSocket(path) => {
                assert!(path.to_string_lossy().contains("rhizoCrypt.sock"));
            }
            TransportHint::Tcp {
                ..
            } => {
                // Acceptable fallback when socket_dir returns None
            }
            TransportHint::AbstractSocket(_) => panic!("Unix platform should not use abstract"),
        }
    }

    #[test]
    fn test_unix_transport_from_socket_path_some() {
        let path = PathBuf::from("/tmp/biomeos/test.sock");
        let hint = unix_transport_from_socket_path(Some(path.clone()), 9400);
        assert_eq!(hint, TransportHint::UnixSocket(path));
    }

    #[test]
    fn test_unix_transport_from_socket_path_none_falls_back_to_tcp() {
        let hint = unix_transport_from_socket_path(None, 8080);
        assert_eq!(
            hint,
            TransportHint::Tcp {
                host: DEFAULT_RPC_HOST.to_string(),
                port: 8080,
            }
        );
    }

    #[test]
    fn test_unix_socket_dir_fallback() {
        let dir = unix_socket_dir_fallback();
        if cfg!(target_os = "linux") {
            assert_eq!(dir, PathBuf::from(DEFAULT_SOCKET_DIR));
        } else {
            assert_eq!(dir, std::env::temp_dir().join("biomeos"));
        }
    }

    #[test]
    fn test_platform_kind_clone_and_copy() {
        let kind = PlatformKind::Unix;
        let copied = kind;
        let cloned = kind;
        assert_eq!(kind, copied);
        assert_eq!(kind, cloned);
    }

    #[test]
    fn test_transport_hint_clone() {
        let hint = TransportHint::AbstractSocket("biomeos.test".to_string());
        let cloned = hint.clone();
        assert_eq!(hint, cloned);

        let tcp = TransportHint::Tcp {
            host: "localhost".to_string(),
            port: 3000,
        };
        let tcp_cloned = tcp.clone();
        assert_eq!(tcp, tcp_cloned);
    }

    // ====================================================================
    // BTSP Phase 1: Family-scoped socket naming + environment guard
    // ====================================================================

    #[test]
    fn test_read_family_id_from_ecosystem_var() {
        temp_env::with_vars(
            [("FAMILY_ID", Some("acme-prod")), ("RHIZOCRYPT_FAMILY_ID", None::<&str>)],
            || {
                let fid = read_family_id("RHIZOCRYPT");
                assert_eq!(fid.as_deref(), Some("acme-prod"));
            },
        );
    }

    #[test]
    fn test_read_family_id_primal_override_takes_precedence() {
        temp_env::with_vars(
            [("FAMILY_ID", Some("eco-wide")), ("RHIZOCRYPT_FAMILY_ID", Some("primal-specific"))],
            || {
                let fid = read_family_id("RHIZOCRYPT");
                assert_eq!(fid.as_deref(), Some("primal-specific"));
            },
        );
    }

    #[test]
    fn test_read_family_id_none_when_unset() {
        temp_env::with_vars(
            [("FAMILY_ID", None::<&str>), ("RHIZOCRYPT_FAMILY_ID", None::<&str>)],
            || {
                assert!(read_family_id("RHIZOCRYPT").is_none());
            },
        );
    }

    #[test]
    fn test_read_family_id_default_treated_as_none() {
        temp_env::with_vars([("FAMILY_ID", Some("default"))], || {
            assert!(read_family_id("RHIZOCRYPT").is_none());
        });
    }

    #[test]
    fn test_read_family_id_empty_treated_as_none() {
        temp_env::with_vars([("FAMILY_ID", Some(""))], || {
            assert!(read_family_id("RHIZOCRYPT").is_none());
        });
    }

    #[test]
    fn test_read_family_id_whitespace_trimmed() {
        temp_env::with_vars([("FAMILY_ID", Some("  acme  "))], || {
            let fid = read_family_id("RHIZOCRYPT");
            assert_eq!(fid.as_deref(), Some("acme"));
        });
    }

    #[test]
    fn test_is_biomeos_insecure_truthy_values() {
        for val in &["1", "true", "yes"] {
            temp_env::with_vars([("BIOMEOS_INSECURE", Some(*val))], || {
                assert!(is_biomeos_insecure(), "Expected insecure for '{val}'");
            });
        }
    }

    #[test]
    fn test_is_biomeos_insecure_falsy_values() {
        for val in &["0", "false", "no", ""] {
            temp_env::with_vars([("BIOMEOS_INSECURE", Some(*val))], || {
                assert!(!is_biomeos_insecure(), "Expected secure for '{val}'");
            });
        }
        temp_env::with_vars([("BIOMEOS_INSECURE", None::<&str>)], || {
            assert!(!is_biomeos_insecure(), "Expected secure when unset");
        });
    }

    #[test]
    fn test_btsp_guard_ok_production() {
        temp_env::with_vars(
            [("FAMILY_ID", Some("acme-prod")), ("BIOMEOS_INSECURE", None::<&str>)],
            || {
                assert!(btsp_env_guard("RHIZOCRYPT").is_ok());
            },
        );
    }

    #[test]
    fn test_btsp_guard_ok_development() {
        temp_env::with_vars([("FAMILY_ID", None::<&str>), ("BIOMEOS_INSECURE", Some("1"))], || {
            assert!(btsp_env_guard("RHIZOCRYPT").is_ok());
        });
    }

    #[test]
    fn test_btsp_guard_ok_neither_set() {
        temp_env::with_vars(
            [("FAMILY_ID", None::<&str>), ("BIOMEOS_INSECURE", None::<&str>)],
            || {
                assert!(btsp_env_guard("RHIZOCRYPT").is_ok());
            },
        );
    }

    #[test]
    fn test_btsp_guard_rejects_conflict() {
        temp_env::with_vars(
            [("FAMILY_ID", Some("acme-prod")), ("BIOMEOS_INSECURE", Some("1"))],
            || {
                let result = btsp_env_guard("RHIZOCRYPT");
                assert_eq!(result, Err(BtspConfigError::FamilyInsecureConflict));
                assert!(result.unwrap_err().to_string().contains("BTSP conflict"));
            },
        );
    }

    #[test]
    fn test_btsp_guard_default_family_not_conflict() {
        temp_env::with_vars(
            [("FAMILY_ID", Some("default")), ("BIOMEOS_INSECURE", Some("1"))],
            || {
                assert!(btsp_env_guard("RHIZOCRYPT").is_ok(), "default is not a real FAMILY_ID");
            },
        );
    }

    #[test]
    fn test_family_scoped_socket_path_with_family() {
        if cfg!(target_os = "android") || cfg!(target_os = "windows") {
            return;
        }
        let temp = tempfile::tempdir().expect("temp dir");
        let path_str = temp.path().to_str().unwrap();
        temp_env::with_vars(
            [
                ("XDG_RUNTIME_DIR", Some(path_str)),
                ("FAMILY_ID", Some("acme-42")),
                ("RHIZOCRYPT_FAMILY_ID", None::<&str>),
            ],
            || {
                let path = family_scoped_socket_path("rhizocrypt", "RHIZOCRYPT").unwrap();
                assert!(
                    path.to_string_lossy().ends_with("rhizocrypt-acme-42.sock"),
                    "Expected family-scoped path, got: {path:?}"
                );
            },
        );
    }

    #[test]
    fn test_family_scoped_socket_path_without_family() {
        if cfg!(target_os = "android") || cfg!(target_os = "windows") {
            return;
        }
        let temp = tempfile::tempdir().expect("temp dir");
        let path_str = temp.path().to_str().unwrap();
        temp_env::with_vars(
            [
                ("XDG_RUNTIME_DIR", Some(path_str)),
                ("FAMILY_ID", None::<&str>),
                ("RHIZOCRYPT_FAMILY_ID", None::<&str>),
            ],
            || {
                let path = family_scoped_socket_path("rhizocrypt", "RHIZOCRYPT").unwrap();
                assert!(
                    path.to_string_lossy().ends_with("rhizocrypt.sock"),
                    "Expected unscoped path, got: {path:?}"
                );
            },
        );
    }

    #[test]
    fn test_family_scoped_socket_primal_override() {
        if cfg!(target_os = "android") || cfg!(target_os = "windows") {
            return;
        }
        let temp = tempfile::tempdir().expect("temp dir");
        let path_str = temp.path().to_str().unwrap();
        temp_env::with_vars(
            [
                ("XDG_RUNTIME_DIR", Some(path_str)),
                ("FAMILY_ID", Some("eco-wide")),
                ("RHIZOCRYPT_FAMILY_ID", Some("override-99")),
            ],
            || {
                let path = family_scoped_socket_path("rhizocrypt", "RHIZOCRYPT").unwrap();
                assert!(
                    path.to_string_lossy().ends_with("rhizocrypt-override-99.sock"),
                    "Primal-specific FAMILY_ID should take precedence, got: {path:?}"
                );
            },
        );
    }
}
