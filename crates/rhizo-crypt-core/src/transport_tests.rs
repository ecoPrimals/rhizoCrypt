// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2024–2026 ecoPrimals Project

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
    temp_env::with_vars([("FAMILY_ID", None::<&str>), ("BIOMEOS_INSECURE", None::<&str>)], || {
        assert!(btsp_env_guard("RHIZOCRYPT").is_ok());
    });
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
    temp_env::with_vars([("FAMILY_ID", Some("default")), ("BIOMEOS_INSECURE", Some("1"))], || {
        assert!(btsp_env_guard("RHIZOCRYPT").is_ok(), "default is not a real FAMILY_ID");
    });
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

// ── TransportEndpoint constructors & accessors ───────────────────

#[test]
fn test_transport_endpoint_tcp_constructor() {
    let ep = TransportEndpoint::tcp("myhost", 9400);
    assert_eq!(ep.tcp_addr(), Some(("myhost", 9400)));
}

#[test]
fn test_transport_endpoint_tcp_addr_returns_none_for_uds() {
    let ep = TransportEndpoint::uds("/run/test.sock");
    assert!(ep.tcp_addr().is_none());
}

#[test]
fn test_transport_endpoint_tcp_addr_returns_none_for_mesh_relay() {
    let ep = TransportEndpoint::MeshRelay {
        peer_id: "peer".into(),
        capability: "cap".into(),
    };
    assert!(ep.tcp_addr().is_none());
}

#[test]
fn test_transport_endpoint_uds_constructor() {
    let ep = TransportEndpoint::uds("/tmp/my.sock");
    match ep {
        TransportEndpoint::Uds {
            path,
        } => assert_eq!(path, "/tmp/my.sock"),
        _ => panic!("expected UDS"),
    }
}

// ── TransportEndpoint::try_parse_address ─────────────────────────

#[test]
fn test_try_parse_address_absolute_path() {
    let ep = TransportEndpoint::try_parse_address("/run/eco/rhizocrypt.sock").unwrap();
    assert!(matches!(ep, TransportEndpoint::Uds { path } if path == "/run/eco/rhizocrypt.sock"));
}

#[test]
fn test_try_parse_address_sock_suffix() {
    let ep = TransportEndpoint::try_parse_address("rhizocrypt.sock").unwrap();
    assert!(matches!(ep, TransportEndpoint::Uds { path } if path == "rhizocrypt.sock"));
}

#[test]
fn test_try_parse_address_sock_suffix_case_insensitive() {
    let ep = TransportEndpoint::try_parse_address("myService.SOCK").unwrap();
    assert!(matches!(ep, TransportEndpoint::Uds { .. }));
}

#[test]
fn test_try_parse_address_host_port() {
    let ep = TransportEndpoint::try_parse_address("192.168.1.1:9300").unwrap();
    assert_eq!(ep.tcp_addr(), Some(("192.168.1.1", 9300)));
}

#[test]
fn test_try_parse_address_localhost_port() {
    let ep = TransportEndpoint::try_parse_address("localhost:7700").unwrap();
    assert_eq!(ep.tcp_addr(), Some(("localhost", 7700)));
}

#[test]
fn test_try_parse_address_empty_host_returns_none() {
    assert!(TransportEndpoint::try_parse_address(":8080").is_none());
}

#[test]
fn test_try_parse_address_no_port_returns_none() {
    assert!(TransportEndpoint::try_parse_address("just-a-hostname").is_none());
}

#[test]
fn test_try_parse_address_invalid_port_returns_none() {
    assert!(TransportEndpoint::try_parse_address("host:notaport").is_none());
}

// ── TransportEndpoint::parse_address ─────────────────────────────

#[test]
fn test_parse_address_tcp() {
    let ep = TransportEndpoint::parse_address("myhost:9400");
    assert_eq!(ep.tcp_addr(), Some(("myhost", 9400)));
}

#[test]
fn test_parse_address_uds_with_slash() {
    let ep = TransportEndpoint::parse_address("/run/eco/test.sock");
    assert!(matches!(ep, TransportEndpoint::Uds { .. }));
}

#[test]
fn test_parse_address_sock_suffix_without_slash() {
    let ep = TransportEndpoint::parse_address("mysvc.sock");
    assert!(matches!(ep, TransportEndpoint::Uds { .. }));
}

#[test]
fn test_parse_address_unrecognized_falls_back_to_uds() {
    let ep = TransportEndpoint::parse_address("garbage");
    assert!(matches!(ep, TransportEndpoint::Uds { path } if path == "garbage"));
}

// ── TransportEndpoint Display ────────────────────────────────────

#[test]
fn test_display_uds() {
    let ep = TransportEndpoint::uds("/run/eco/test.sock");
    assert_eq!(ep.to_string(), "unix:///run/eco/test.sock");
}

#[test]
fn test_display_tcp() {
    let ep = TransportEndpoint::tcp("127.0.0.1", 9300);
    assert_eq!(ep.to_string(), "tcp://127.0.0.1:9300");
}

#[test]
fn test_display_mesh_relay() {
    let ep = TransportEndpoint::MeshRelay {
        peer_id: "strand-gate".into(),
        capability: "security".into(),
    };
    assert_eq!(ep.to_string(), "mesh://strand-gate/security");
}

// ── TransportEndpoint From<SocketAddr> ───────────────────────────

#[test]
fn test_from_socket_addr() {
    let addr: std::net::SocketAddr = "192.168.1.100:7700".parse().unwrap();
    let ep = TransportEndpoint::from(addr);
    assert_eq!(ep.tcp_addr(), Some(("192.168.1.100", 7700)));
}

// ── TransportEndpoint serde roundtrip ────────────────────────────

#[test]
fn test_serde_roundtrip_uds() {
    let ep = TransportEndpoint::uds("/run/eco/test.sock");
    let json = serde_json::to_string(&ep).unwrap();
    let back: TransportEndpoint = serde_json::from_str(&json).unwrap();
    assert_eq!(ep, back);
}

#[test]
fn test_serde_roundtrip_tcp() {
    let ep = TransportEndpoint::tcp("localhost", 9300);
    let json = serde_json::to_string(&ep).unwrap();
    let back: TransportEndpoint = serde_json::from_str(&json).unwrap();
    assert_eq!(ep, back);
}

#[test]
fn test_serde_roundtrip_mesh_relay() {
    let ep = TransportEndpoint::MeshRelay {
        peer_id: "peer-1".into(),
        capability: "storage".into(),
    };
    let json = serde_json::to_string(&ep).unwrap();
    let back: TransportEndpoint = serde_json::from_str(&json).unwrap();
    assert_eq!(ep, back);
}

// ── connect_transport: MeshRelay returns Unsupported ─────────────

#[tokio::test]
async fn test_connect_transport_mesh_relay_returns_unsupported() {
    let ep = TransportEndpoint::MeshRelay {
        peer_id: "test-peer".into(),
        capability: "dag".into(),
    };
    let err = connect_transport(&ep).await.unwrap_err();
    assert_eq!(err.kind(), std::io::ErrorKind::Unsupported);
    assert!(err.to_string().contains("discovery routing"));
}

// ── connect_transport: UDS success ───────────────────────────────

#[cfg(unix)]
#[tokio::test]
async fn test_connect_transport_uds_success() {
    let dir = tempfile::tempdir().unwrap();
    let sock_path = dir.path().join("test.sock");

    let listener = tokio::net::UnixListener::bind(&sock_path).unwrap();
    let ep = TransportEndpoint::uds(sock_path.to_str().unwrap());

    let (stream_result, _accept) =
        tokio::join!(connect_transport(&ep), async { listener.accept().await.unwrap() });
    assert!(stream_result.is_ok());
}

// ── connect_transport: UDS failure ───────────────────────────────

#[cfg(unix)]
#[tokio::test]
async fn test_connect_transport_uds_no_listener() {
    let ep = TransportEndpoint::uds("/tmp/nonexistent_rhizo_test_9999.sock");
    assert!(connect_transport(&ep).await.is_err());
}

// ── socket_is_alive ──────────────────────────────────────────────

#[cfg(unix)]
#[tokio::test]
async fn test_socket_is_alive_with_listener() {
    let dir = tempfile::tempdir().unwrap();
    let sock_path = dir.path().join("alive.sock");
    let _listener = tokio::net::UnixListener::bind(&sock_path).unwrap();
    assert!(socket_is_alive(&sock_path));
}

#[cfg(unix)]
#[test]
fn test_socket_is_alive_missing_path() {
    assert!(!socket_is_alive(std::path::Path::new("/tmp/does_not_exist_rhizo.sock")));
}

#[cfg(unix)]
#[test]
fn test_socket_is_alive_stale_socket() {
    let dir = tempfile::tempdir().unwrap();
    let sock_path = dir.path().join("stale.sock");
    std::fs::write(&sock_path, b"").unwrap();
    assert!(!socket_is_alive(&sock_path));
}

// ── JsonRpcTransportError Display ────────────────────────────────

#[test]
fn test_jsonrpc_transport_error_display_connect_timeout() {
    let err = JsonRpcTransportError::ConnectTimeout;
    assert_eq!(err.to_string(), "connection timed out");
    assert!(std::error::Error::source(&err).is_none());
}

#[test]
fn test_jsonrpc_transport_error_display_connect_failed() {
    let io_err = std::io::Error::new(std::io::ErrorKind::ConnectionRefused, "refused");
    let err = JsonRpcTransportError::ConnectFailed(io_err);
    assert!(err.to_string().contains("connection failed"));
    assert!(std::error::Error::source(&err).is_some());
}

#[test]
fn test_jsonrpc_transport_error_display_response_timeout() {
    let err = JsonRpcTransportError::ResponseTimeout;
    assert_eq!(err.to_string(), "response timed out");
    assert!(std::error::Error::source(&err).is_none());
}

#[test]
fn test_jsonrpc_transport_error_display_write() {
    let io_err = std::io::Error::new(std::io::ErrorKind::BrokenPipe, "broken");
    let err = JsonRpcTransportError::Write(io_err);
    assert!(err.to_string().contains("write failed"));
    assert!(std::error::Error::source(&err).is_some());
}

#[test]
fn test_jsonrpc_transport_error_display_read() {
    let io_err = std::io::Error::new(std::io::ErrorKind::UnexpectedEof, "eof");
    let err = JsonRpcTransportError::Read(io_err);
    assert!(err.to_string().contains("read failed"));
    assert!(std::error::Error::source(&err).is_some());
}

#[test]
fn test_jsonrpc_transport_error_display_serialize() {
    let serde_err = serde_json::from_str::<serde_json::Value>("bad").unwrap_err();
    let err = JsonRpcTransportError::Serialize(serde_err);
    assert!(err.to_string().contains("serialize failed"));
    assert!(std::error::Error::source(&err).is_some());
}
