// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2024–2026 ecoPrimals Project

//! Tests for `connect_transport` and `socket_is_alive`.

use super::*;

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

#[cfg(unix)]
#[tokio::test]
async fn test_connect_transport_uds_no_listener() {
    let ep = TransportEndpoint::uds("/tmp/nonexistent_rhizo_test_9999.sock");
    assert!(connect_transport(&ep).await.is_err());
}

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
