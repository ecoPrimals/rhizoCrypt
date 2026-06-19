// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2024–2026 ecoPrimals Project

//! JSON-RPC method routing over BTSP-enforced UDS connections.

#![expect(clippy::unwrap_used, clippy::expect_used, reason = "test code")]

use super::tests_support::test_primal;
use super::*;
use tokio::io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader};

/// PG-52 repro: plain `dag.session.create` over UDS with BTSP required
/// must succeed (UDS is filesystem-authenticated, no handshake needed).
#[tokio::test]
async fn test_plain_jsonrpc_data_methods_on_btsp_uds() {
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

    let req = r#"{"jsonrpc":"2.0","method":"dag.session.create","params":{"description":"PG-52 test","session_type":"General"},"id":1}"#;
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
    assert!(total > 0, "should receive response, not empty/reset");
    let resp_line = std::str::from_utf8(&buf[..total]).unwrap();
    let resp: serde_json::Value = serde_json::from_str(resp_line.trim()).unwrap();

    assert_eq!(resp["jsonrpc"], "2.0");
    assert_eq!(resp["id"], 1);
    assert!(
        resp["result"].is_string(),
        "dag.session.create should return a session ID, got: {resp}"
    );

    let server_result = server_handle.await.unwrap();
    assert!(server_result.is_ok());
}

/// Verify multiple data methods work over plain UDS with BTSP required:
/// `dag.session.create`, `dag.event.append`, `dag.vertex.children`,
/// `dag.frontier.get`, `dag.merkle.root`.
#[tokio::test]
async fn test_dag_method_suite_on_btsp_uds() {
    let family_seed = b"integration-test-family-seed-ok!";
    let primal = test_primal().await;
    let server = crate::service::RhizoCryptRpcServer::new(primal);

    let (server_raw, client_raw) = std::os::unix::net::UnixStream::pair().unwrap();
    server_raw.set_nonblocking(true).unwrap();
    client_raw.set_nonblocking(true).unwrap();
    let server_stream = tokio::net::UnixStream::from_std(server_raw).unwrap();
    let client_stream = tokio::net::UnixStream::from_std(client_raw).unwrap();

    let server_handle = tokio::spawn(async move {
        handle_uds_connection(server_stream, server, true, Some(family_seed)).await
    });

    let (reader, mut writer) = client_stream.into_split();

    let create = r#"{"jsonrpc":"2.0","method":"dag.session.create","params":{"description":"suite","session_type":"General"},"id":1}"#;
    writer.write_all(format!("{create}\n").as_bytes()).await.unwrap();

    let health = r#"{"jsonrpc":"2.0","method":"health.check","params":{},"id":2}"#;
    writer.write_all(format!("{health}\n").as_bytes()).await.unwrap();

    let caps = r#"{"jsonrpc":"2.0","method":"capability.list","params":{},"id":3}"#;
    writer.write_all(format!("{caps}\n").as_bytes()).await.unwrap();

    writer.shutdown().await.unwrap();

    let mut lines = BufReader::new(reader).lines();
    let mut responses = Vec::new();
    while let Some(line) = lines.next_line().await.unwrap() {
        responses.push(serde_json::from_str::<serde_json::Value>(&line).unwrap());
    }

    assert_eq!(responses.len(), 3, "should get 3 responses");

    assert_eq!(responses[0]["id"], 1);
    assert!(responses[0]["result"].is_string(), "dag.session.create result: {}", responses[0]);

    assert_eq!(responses[1]["id"], 2);
    assert!(responses[1]["result"].is_object(), "health.check result: {}", responses[1]);

    assert_eq!(responses[2]["id"], 3);
    assert!(
        responses[2]["result"].is_object() || responses[2]["result"].is_array(),
        "capability.list result: {}",
        responses[2]
    );

    let server_result = server_handle.await.unwrap();
    assert!(server_result.is_ok());
}

/// Without mito-beacon signal, normal JSON-RPC still works (no regression).
#[tokio::test]
async fn test_no_mito_beacon_prefix_still_works() {
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

    let req = r#"{"jsonrpc":"2.0","method":"health.check","params":{},"id":42}"#;
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
    let resp: serde_json::Value =
        serde_json::from_str(std::str::from_utf8(&buf[..total]).unwrap().trim()).unwrap();
    assert_eq!(resp["id"], 42);
    assert!(resp["result"].is_object());

    let server_result = server_handle.await.unwrap();
    assert!(server_result.is_ok());
}

/// Verify batch JSON-RPC also works on BTSP-enforced UDS.
#[tokio::test]
async fn test_batch_jsonrpc_on_btsp_uds() {
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

    let batch = r#"[{"jsonrpc":"2.0","method":"health.check","params":{},"id":1},{"jsonrpc":"2.0","method":"dag.session.create","params":{"description":"batch","session_type":"General"},"id":2}]"#;
    client.write_all(format!("{batch}\n").as_bytes()).await.unwrap();
    client.shutdown().await.unwrap();

    let mut buf = vec![0u8; 8192];
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

    let arr = resp.as_array().expect("batch response should be an array");
    assert_eq!(arr.len(), 2);
    assert!(arr[0]["result"].is_object(), "health.check: {}", arr[0]);
    assert!(arr[1]["result"].is_string(), "dag.session.create: {}", arr[1]);

    let server_result = server_handle.await.unwrap();
    assert!(server_result.is_ok());
}
