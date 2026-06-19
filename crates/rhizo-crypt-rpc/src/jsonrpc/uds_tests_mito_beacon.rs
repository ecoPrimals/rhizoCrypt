// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2024–2026 ecoPrimals Project

//! Mito-beacon signal prefix tests over UDS connections.

#![expect(clippy::unwrap_used, clippy::expect_used, reason = "test code")]

use super::tests_support::test_primal;
use super::*;
use tokio::io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader};

/// Mito-beacon `[0xEC, 0x01]` signal before plain JSON-RPC (dev mode).
#[tokio::test]
async fn test_mito_beacon_signal_jsonrpc_over_uds() {
    let dir = tempfile::tempdir().expect("tempdir");
    let sock = dir.path().join("mito-beacon-test.sock");
    let primal = test_primal().await;

    let server = UdsJsonRpcServer::new(primal, sock.clone());
    let (shutdown_tx, shutdown_rx) = watch::channel(false);
    let ready = Arc::new(tokio::sync::Notify::new());
    let ready_rx = Arc::clone(&ready);

    let handle = tokio::spawn(async move { server.serve_with_ready(shutdown_rx, ready_rx).await });
    ready.notified().await;

    let stream = tokio::net::UnixStream::connect(&sock).await.expect("connect");
    let (reader, mut writer) = stream.into_split();

    writer.write_all(&[0xEC, 0x01]).await.unwrap();
    let req = r#"{"jsonrpc":"2.0","method":"health.check","params":{},"id":1}"#;
    writer.write_all(format!("{req}\n").as_bytes()).await.unwrap();
    writer.shutdown().await.unwrap();

    let mut lines = BufReader::new(reader).lines();
    let line = lines.next_line().await.unwrap().expect("response after mito-beacon signal");
    let resp: serde_json::Value = serde_json::from_str(&line).unwrap();
    assert_eq!(resp["jsonrpc"], "2.0");
    assert!(resp["result"].is_object());
    assert_eq!(resp["id"], 1);

    let _ = shutdown_tx.send(true);
    let _ = handle.await;
}

/// Mito-beacon signal followed by plain JSON-RPC on BTSP-enforced UDS.
#[tokio::test]
async fn test_mito_beacon_btsp_uds_plain_jsonrpc() {
    let family_seed = b"integration-test-family-seed-ok!";
    let primal = test_primal().await;
    let server = crate::service::RhizoCryptRpcServer::new(primal);

    let (server_raw, client_raw) = std::os::unix::net::UnixStream::pair().unwrap();
    server_raw.set_nonblocking(true).unwrap();
    client_raw.set_nonblocking(true).unwrap();
    let server_stream = tokio::net::UnixStream::from_std(server_raw).unwrap();
    let mut client = tokio::net::UnixStream::from_std(client_raw).unwrap();

    let server_handle = tokio::spawn(async move {
        handle_uds_connection(server_stream, server, true, Some(family_seed)).await
    });

    client.write_all(&[0xEC, 0x01]).await.unwrap();
    let req = r#"{"jsonrpc":"2.0","method":"health.check","params":{},"id":1}"#;
    client.write_all(format!("{req}\n").as_bytes()).await.unwrap();
    client.shutdown().await.unwrap();

    let mut buf = vec![0u8; 4096];
    let mut total = 0;
    loop {
        let n = client.read(&mut buf[total..]).await.unwrap();
        total += n;
        if n == 0 || buf[..total].contains(&b'\n') {
            break;
        }
    }
    let resp_line = std::str::from_utf8(&buf[..total]).unwrap();
    let resp: serde_json::Value = serde_json::from_str(resp_line.trim()).unwrap();
    assert_eq!(resp["jsonrpc"], "2.0");
    assert_eq!(resp["id"], 1);
    assert!(resp["result"].is_object(), "health.check through mito-beacon: {resp}");

    let server_result = server_handle.await.unwrap();
    assert!(server_result.is_ok());
}

/// Mito-beacon extended signal `[0xED, 0x00]` accepted (genetics-layer).
#[tokio::test]
async fn test_mito_beacon_extended_signal_accepted() {
    let family_seed = b"integration-test-family-seed-ok!";
    let primal = test_primal().await;
    let server = crate::service::RhizoCryptRpcServer::new(primal);

    let (server_raw, client_raw) = std::os::unix::net::UnixStream::pair().unwrap();
    server_raw.set_nonblocking(true).unwrap();
    client_raw.set_nonblocking(true).unwrap();
    let server_stream = tokio::net::UnixStream::from_std(server_raw).unwrap();
    let mut client = tokio::net::UnixStream::from_std(client_raw).unwrap();

    let server_handle = tokio::spawn(async move {
        handle_uds_connection(server_stream, server, true, Some(family_seed)).await
    });

    client.write_all(&[0xED, 0x00]).await.unwrap();
    let req = r#"{"jsonrpc":"2.0","method":"health.check","params":{},"id":1}"#;
    client.write_all(format!("{req}\n").as_bytes()).await.unwrap();
    client.shutdown().await.unwrap();

    let mut buf = vec![0u8; 4096];
    let mut total = 0;
    loop {
        let n = client.read(&mut buf[total..]).await.unwrap();
        total += n;
        if n == 0 || buf[..total].contains(&b'\n') {
            break;
        }
    }
    let resp_line = std::str::from_utf8(&buf[..total]).unwrap();
    let resp: serde_json::Value = serde_json::from_str(resp_line.trim()).unwrap();
    assert_eq!(resp["jsonrpc"], "2.0");
    assert_eq!(resp["id"], 1);
    assert!(resp["result"].is_object(), "health.check through 0xED signal: {resp}");

    let server_result = server_handle.await.unwrap();
    assert!(server_result.is_ok());
}

/// Mito-beacon with different sub-type `[0xEC, 0x02]` accepted.
#[tokio::test]
async fn test_mito_beacon_subtype_02_accepted() {
    let dir = tempfile::tempdir().expect("tempdir");
    let sock = dir.path().join("mito-subtype-test.sock");
    let primal = test_primal().await;

    let server = UdsJsonRpcServer::new(primal, sock.clone());
    let (shutdown_tx, shutdown_rx) = watch::channel(false);
    let ready = Arc::new(tokio::sync::Notify::new());
    let ready_rx = Arc::clone(&ready);

    let handle = tokio::spawn(async move { server.serve_with_ready(shutdown_rx, ready_rx).await });
    ready.notified().await;

    let stream = tokio::net::UnixStream::connect(&sock).await.expect("connect");
    let (reader, mut writer) = stream.into_split();

    writer.write_all(&[0xEC, 0x02]).await.unwrap();
    let req = r#"{"jsonrpc":"2.0","method":"health.check","params":{},"id":1}"#;
    writer.write_all(format!("{req}\n").as_bytes()).await.unwrap();
    writer.shutdown().await.unwrap();

    let mut lines = BufReader::new(reader).lines();
    let line = lines.next_line().await.unwrap().expect("response after 0xEC 0x02 signal");
    let resp: serde_json::Value = serde_json::from_str(&line).unwrap();
    assert_eq!(resp["jsonrpc"], "2.0");
    assert!(resp["result"].is_object());
    assert_eq!(resp["id"], 1);

    let _ = shutdown_tx.send(true);
    let _ = handle.await;
}
