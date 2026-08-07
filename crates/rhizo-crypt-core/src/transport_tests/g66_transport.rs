// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2024–2026 ecoPrimals Project

//! G66 Transport Abstraction tests — `TransportListener`, `platform_default`,
//! `from_env_or_default`, `is_local`, and cross-transport I/O.

use super::*;

// ── TransportEndpoint::is_local ──────────────────────────────────

#[test]
fn test_is_local_uds() {
    assert!(TransportEndpoint::uds("/run/test.sock").is_local());
}

#[test]
fn test_is_local_tcp_localhost_ipv4() {
    assert!(TransportEndpoint::tcp("127.0.0.1", 7700).is_local());
}

#[test]
fn test_is_local_tcp_localhost_ipv6() {
    assert!(TransportEndpoint::tcp("::1", 7700).is_local());
}

#[test]
fn test_is_local_tcp_localhost_name() {
    assert!(TransportEndpoint::tcp("localhost", 7700).is_local());
}

#[test]
fn test_is_local_tcp_remote() {
    assert!(!TransportEndpoint::tcp("192.168.1.100", 7700).is_local());
}

#[test]
fn test_is_local_mesh_relay() {
    let ep = TransportEndpoint::MeshRelay {
        peer_id: "peer".into(),
        capability: "cap".into(),
    };
    assert!(!ep.is_local());
}

// ── TransportEndpoint::platform_default ──────────────────────────

#[cfg(unix)]
#[test]
fn test_platform_default_returns_uds_on_unix() {
    let ep = TransportEndpoint::platform_default("rhizocrypt", 7700);
    assert!(matches!(ep, TransportEndpoint::Uds { .. }));
}

// ── TransportEndpoint::from_env_or_default ───────────────────────

#[test]
fn test_from_env_json_override() {
    let json = r#"{"transport":"tcp","host":"10.0.0.5","port":9300}"#;
    temp_env::with_vars([("TRANSPORT_ENDPOINT", Some(json))], || {
        let ep = TransportEndpoint::from_env_or_default("rhizocrypt", "RHIZOCRYPT", 7700);
        assert_eq!(ep.tcp_addr(), Some(("10.0.0.5", 9300)));
    });
}

#[test]
fn test_from_env_address_override() {
    temp_env::with_vars(
        [("TRANSPORT_ENDPOINT", None::<&str>), ("RHIZOCRYPT_ADDRESS", Some("192.168.1.5:8800"))],
        || {
            let ep = TransportEndpoint::from_env_or_default("rhizocrypt", "RHIZOCRYPT", 7700);
            assert_eq!(ep.tcp_addr(), Some(("192.168.1.5", 8800)));
        },
    );
}

#[test]
fn test_from_env_falls_back_to_platform_default() {
    temp_env::with_vars(
        [("TRANSPORT_ENDPOINT", None::<&str>), ("RHIZOCRYPT_ADDRESS", None::<&str>)],
        || {
            let ep = TransportEndpoint::from_env_or_default("rhizocrypt", "RHIZOCRYPT", 7700);
            // On Unix: UDS; on non-Unix: TCP localhost:7700
            assert!(ep.is_local());
        },
    );
}

// ── TransportStream methods ──────────────────────────────────────

#[cfg(unix)]
#[tokio::test]
async fn test_transport_stream_supports_peer_cred_uds() {
    let dir = tempfile::tempdir().unwrap();
    let sock = dir.path().join("peercred.sock");
    let listener = tokio::net::UnixListener::bind(&sock).unwrap();
    let ep = TransportEndpoint::uds(sock.to_str().unwrap());

    let (stream, _) =
        tokio::join!(connect_transport(&ep), async { listener.accept().await.unwrap() });
    let stream = stream.unwrap();
    assert!(stream.supports_peer_cred());
    assert!(stream.is_local());
}

#[tokio::test]
async fn test_transport_stream_tcp_no_peer_cred() {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let ep = TransportEndpoint::from(addr);

    let (stream, _) =
        tokio::join!(connect_transport(&ep), async { listener.accept().await.unwrap() });
    let stream = stream.unwrap();
    assert!(!stream.supports_peer_cred());
    assert!(stream.is_local());
}

// ── TransportListener ────────────────────────────────────────────

#[tokio::test]
async fn test_transport_listener_tcp_accept_roundtrip() {
    use tokio::io::{AsyncReadExt, AsyncWriteExt};

    let tcp_listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let bound_addr = tcp_listener.local_addr().unwrap();
    let listener = TransportListener::Tcp(tcp_listener);

    let server = tokio::spawn(async move {
        let mut stream = listener.accept().await.unwrap();
        let mut buf = [0u8; 5];
        stream.read_exact(&mut buf).await.unwrap();
        buf
    });

    let connect_ep = TransportEndpoint::from(bound_addr);
    let mut client = connect_transport(&connect_ep).await.unwrap();
    client.write_all(b"hello").await.unwrap();
    client.flush().await.unwrap();

    let received = server.await.unwrap();
    assert_eq!(&received, b"hello");
}

#[cfg(unix)]
#[tokio::test]
async fn test_transport_listener_uds_bind_accept() {
    use tokio::io::{AsyncReadExt, AsyncWriteExt};

    let dir = tempfile::tempdir().unwrap();
    let sock = dir.path().join("g66_listener.sock");
    let ep = TransportEndpoint::uds(sock.to_str().unwrap());

    let listener = TransportListener::bind(&ep).await.unwrap();

    let server = tokio::spawn(async move {
        let mut stream = listener.accept().await.unwrap();
        assert!(stream.supports_peer_cred());
        let mut buf = [0u8; 4];
        stream.read_exact(&mut buf).await.unwrap();
        buf
    });

    let mut client = connect_transport(&ep).await.unwrap();
    client.write_all(b"g66!").await.unwrap();
    client.flush().await.unwrap();

    let received = server.await.unwrap();
    assert_eq!(&received, b"g66!");
}

#[tokio::test]
async fn test_transport_listener_bind_tcp() {
    let ep = TransportEndpoint::tcp("127.0.0.1", 0);
    let result = TransportListener::bind(&ep).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_transport_listener_bind_mesh_relay_fails() {
    let ep = TransportEndpoint::MeshRelay {
        peer_id: "p".into(),
        capability: "c".into(),
    };
    let err = TransportListener::bind(&ep).await.unwrap_err();
    assert_eq!(err.kind(), std::io::ErrorKind::Unsupported);
}

#[test]
fn test_transport_listener_debug() {
    let _ = format!("{:?}", TransportEndpoint::tcp("127.0.0.1", 7700));
}
