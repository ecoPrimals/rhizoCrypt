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

use crate::constants::{DEFAULT_RPC_HOST, DEFAULT_SOCKET_DIR, SOCKET_FILE_EXTENSION};

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
        std::env::temp_dir().join("ecoPrimals")
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
        PlatformKind::Android => TransportHint::AbstractSocket(format!("ecoPrimals.{primal_name}")),
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
///   `/run/ecoPrimals` on Linux, `/tmp/ecoPrimals` elsewhere.
/// - **Android**: Returns `None` (use abstract sockets).
/// - **Windows**: Returns `None` (use named pipes or TCP).
/// - **General fallback**: `/tmp/ecoPrimals`.
#[must_use]
pub fn socket_dir() -> Option<PathBuf> {
    if cfg!(target_os = "android") || cfg!(target_os = "windows") {
        return None;
    }

    if let Ok(runtime_dir) = std::env::var("XDG_RUNTIME_DIR") {
        let path = Path::new(&runtime_dir).join("ecoPrimals");
        return Some(path);
    }

    Some(unix_socket_dir_fallback())
}

/// Constructs the full socket path for a primal, or `None` if path-based
/// sockets are not available on this platform.
///
/// Returns `{socket_dir}/{name}.sock` when [`socket_dir()`] is `Some`.
#[must_use]
pub fn socket_path_for_primal(name: &str) -> Option<PathBuf> {
    let dir = socket_dir()?;
    let filename = format!("{name}{SOCKET_FILE_EXTENSION}");
    Some(dir.join(filename))
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
                assert_eq!(dir, runtime_path.join("ecoPrimals"));
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
                assert_eq!(result, Some(std::env::temp_dir().join("ecoPrimals")));
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
            assert_eq!(name, "ecoPrimals.rhizoCrypt");
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

        let abstract1 = TransportHint::AbstractSocket("ecoPrimals.test".to_string());
        let abstract2 = TransportHint::AbstractSocket("ecoPrimals.test".to_string());
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
            assert_eq!(dir, PathBuf::from("ecoPrimals"));
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
        let unix = TransportHint::UnixSocket(PathBuf::from("/run/ecoPrimals/rhizoCrypt.sock"));
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

        let abstract_sock = TransportHint::AbstractSocket("ecoPrimals.test".to_string());
        let debug_str = format!("{abstract_sock:?}");
        assert!(debug_str.contains("AbstractSocket"));
        assert!(debug_str.contains("ecoPrimals.test"));
    }

    #[test]
    fn test_transport_hint_unix_socket_equality() {
        let p1 = PathBuf::from("/run/ecoPrimals/a.sock");
        let p2 = PathBuf::from("/run/ecoPrimals/a.sock");
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
                assert_eq!(name, "ecoPrimals.");
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
        assert_eq!(hint, TransportHint::AbstractSocket("ecoPrimals.rhizoCrypt".to_string()));
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
        let path = PathBuf::from("/tmp/ecoPrimals/test.sock");
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
            assert_eq!(dir, std::env::temp_dir().join("ecoPrimals"));
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
        let hint = TransportHint::AbstractSocket("ecoPrimals.test".to_string());
        let cloned = hint.clone();
        assert_eq!(hint, cloned);

        let tcp = TransportHint::Tcp {
            host: "localhost".to_string(),
            port: 3000,
        };
        let tcp_cloned = tcp.clone();
        assert_eq!(tcp, tcp_cloned);
    }
}
