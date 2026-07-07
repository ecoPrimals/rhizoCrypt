// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2024–2026 ecoPrimals Project

//! Tests for `TransportHint`, `PlatformKind`, and `preferred_transport`.

use super::*;

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
