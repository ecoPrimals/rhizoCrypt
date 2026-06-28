// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2024–2026 ecoPrimals Project

//! UDS server error handling, edge cases, and dev-mode paths.

#![expect(clippy::unwrap_used, clippy::expect_used, reason = "test code")]

use super::tests_support::{client_phase2_handshake, encrypted_roundtrip, test_primal};
use super::*;
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader};
use tokio::sync::watch;

/// Spawn `handle_uds_connection` on a socket pair for direct handler testing.
async fn spawn_handler(
    btsp_required: bool,
    family_seed: Option<&[u8]>,
) -> (tokio::net::UnixStream, tokio::task::JoinHandle<std::io::Result<()>>) {
    let primal = test_primal().await;
    let server = crate::service::RhizoCryptRpcServer::new(primal);
    let (server_raw, client_raw) = std::os::unix::net::UnixStream::pair().unwrap();
    server_raw.set_nonblocking(true).unwrap();
    client_raw.set_nonblocking(true).unwrap();
    let server_stream = tokio::net::UnixStream::from_std(server_raw).unwrap();
    let client = tokio::net::UnixStream::from_std(client_raw).unwrap();
    let seed = family_seed.map(<[u8]>::to_vec);
    let handle = tokio::spawn(async move {
        handle_uds_connection(server_stream, server, btsp_required, seed.as_deref()).await
    });
    (client, handle)
}

#[tokio::test]
async fn test_btsp_required_no_family_seed_rejects() {
    let (mut client, handle) = spawn_handler(true, None).await;

    let req = r#"{"jsonrpc":"2.0","method":"health.check","params":{},"id":1}"#;
    client.write_all(format!("{req}\n").as_bytes()).await.unwrap();
    client.shutdown().await.unwrap();

    let server_result = handle.await.unwrap();
    assert!(server_result.is_err(), "missing seed should reject: {server_result:?}");
}

#[tokio::test]
async fn test_empty_connection_immediate_eof() {
    let (client, handle) = spawn_handler(false, None).await;
    drop(client);

    let server_result = handle.await.unwrap();
    assert!(server_result.is_ok(), "immediate EOF should be clean: {server_result:?}");
}

#[tokio::test]
async fn test_length_prefixed_btsp_handshake_failure() {
    let family_seed = b"integration-test-family-seed-ok!";
    let (mut client, handle) = spawn_handler(true, Some(family_seed)).await;

    // Non-JSON first byte routes to length-prefixed BTSP; garbage frame fails handshake.
    client.write_all(&[0xFF, 0x00, 0x00, 0x00, 0x05, b'g', b'a', b'r', b'b', b'g']).await.unwrap();
    client.shutdown().await.unwrap();

    let server_result = handle.await.unwrap();
    assert!(server_result.is_err(), "invalid BTSP frame should fail: {server_result:?}");
}

#[tokio::test]
async fn test_dev_mode_raw_jsonrpc_no_btsp() {
    let (mut client, handle) = spawn_handler(false, None).await;

    let req = r#"{"jsonrpc":"2.0","method":"health.check","params":{},"id":7}"#;
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
    assert_eq!(resp["id"], 7);
    assert!(resp["result"].is_object());

    let server_result = handle.await.unwrap();
    assert!(server_result.is_ok());
}

#[tokio::test]
async fn test_dev_mode_chains_probe_bytes_back_into_stream() {
    let (mut client, handle) = spawn_handler(false, None).await;

    // First two bytes of JSON are probed then chained back (non-beacon leftover path).
    let req = r#"{"jsonrpc":"2.0","method":"health.check","params":{},"id":8}"#;
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
    assert_eq!(resp["id"], 8);

    let server_result = handle.await.unwrap();
    assert!(server_result.is_ok());
}

#[tokio::test]
async fn test_consume_mito_beacon_partial_single_byte_eof() {
    let (mut client, handle) = spawn_handler(false, None).await;
    client.write_all(b"X").await.unwrap();
    client.shutdown().await.unwrap();

    let server_result = handle.await.unwrap();
    assert!(server_result.is_ok());
}

#[tokio::test]
async fn test_serve_after_handshake_eof_after_handshake() {
    let family_seed = b"integration-test-family-seed-ok!";
    let (mut client, handle) = spawn_handler(true, Some(family_seed)).await;

    let _ = client_phase2_handshake(&mut client, family_seed).await;
    client.shutdown().await.unwrap();

    let server_result = handle.await.unwrap();
    assert!(server_result.is_ok(), "EOF after handshake should be clean: {server_result:?}");
}

#[tokio::test]
async fn test_serve_after_handshake_non_json_first_line() {
    let family_seed = b"integration-test-family-seed-ok!";
    let (mut client, handle) = spawn_handler(true, Some(family_seed)).await;

    let _ = client_phase2_handshake(&mut client, family_seed).await;
    client.write_all(b"not-valid-json\n").await.unwrap();
    let req = r#"{"jsonrpc":"2.0","method":"health.check","params":{},"id":9}"#;
    client.write_all(format!("{req}\n").as_bytes()).await.unwrap();
    client.shutdown().await.unwrap();

    let mut lines = BufReader::new(&mut client).lines();
    let mut responses = Vec::new();
    while let Some(line) = lines.next_line().await.unwrap() {
        responses.push(serde_json::from_str::<serde_json::Value>(&line).unwrap());
    }
    assert!(
        responses.iter().any(|r| r.get("id") == Some(&serde_json::json!(9))),
        "health.check should succeed after non-JSON first line: {responses:?}"
    );

    let server_result = handle.await.unwrap();
    assert!(server_result.is_ok());
}

#[tokio::test]
async fn test_phase3_negotiate_unsupported_cipher_fallback() {
    use base64::Engine;

    let family_seed = b"integration-test-family-seed-ok!";
    let (mut client, handle) = spawn_handler(true, Some(family_seed)).await;

    let (session_id, _handshake_key) = client_phase2_handshake(&mut client, family_seed).await;

    let b64 = base64::engine::general_purpose::STANDARD;
    let client_nonce = {
        let mut n = [0u8; 32];
        rand::RngCore::fill_bytes(&mut rand::thread_rng(), &mut n);
        n
    };
    let negotiate_req = serde_json::json!({
        "jsonrpc": "2.0",
        "method": "btsp.negotiate",
        "params": {
            "session_id": session_id,
            "ciphers": ["aes-256-gcm"],
            "client_nonce": b64.encode(client_nonce),
        },
        "id": 1
    });
    client.write_all(format!("{negotiate_req}\n").as_bytes()).await.unwrap();

    let req = r#"{"jsonrpc":"2.0","method":"health.check","params":{},"id":2}"#;
    client.write_all(format!("{req}\n").as_bytes()).await.unwrap();
    client.shutdown().await.unwrap();

    let mut lines = BufReader::new(&mut client).lines();
    let mut responses = Vec::new();
    while let Some(line) = lines.next_line().await.unwrap() {
        responses.push(serde_json::from_str::<serde_json::Value>(&line).unwrap());
    }
    let health = responses
        .iter()
        .find(|r| r.get("id") == Some(&serde_json::json!(2)))
        .expect("plaintext health.check after null cipher fallback");
    assert!(health["result"].is_object());

    let server_result = handle.await.unwrap();
    assert!(server_result.is_ok());
}

#[tokio::test]
async fn test_phase3_negotiate_session_mismatch_fallback() {
    use base64::Engine;

    let family_seed = b"integration-test-family-seed-ok!";
    let (mut client, handle) = spawn_handler(true, Some(family_seed)).await;

    let (_session_id, _handshake_key) = client_phase2_handshake(&mut client, family_seed).await;

    let b64 = base64::engine::general_purpose::STANDARD;
    let client_nonce = {
        let mut n = [0u8; 32];
        rand::RngCore::fill_bytes(&mut rand::thread_rng(), &mut n);
        n
    };
    let negotiate_req = serde_json::json!({
        "jsonrpc": "2.0",
        "method": "btsp.negotiate",
        "params": {
            "session_id": "00000000000000000000000000000000",
            "ciphers": ["chacha20-poly1305"],
            "client_nonce": b64.encode(client_nonce),
        },
        "id": 1
    });
    client.write_all(format!("{negotiate_req}\n").as_bytes()).await.unwrap();

    let req = r#"{"jsonrpc":"2.0","method":"health.check","params":{},"id":3}"#;
    client.write_all(format!("{req}\n").as_bytes()).await.unwrap();
    client.shutdown().await.unwrap();

    let mut lines = BufReader::new(&mut client).lines();
    let mut responses = Vec::new();
    while let Some(line) = lines.next_line().await.unwrap() {
        responses.push(serde_json::from_str::<serde_json::Value>(&line).unwrap());
    }
    let health = responses
        .iter()
        .find(|r| r.get("id") == Some(&serde_json::json!(3)))
        .expect("plaintext health.check after negotiate error fallback");
    assert!(health["result"].is_object());

    let server_result = handle.await.unwrap();
    assert!(server_result.is_ok());
}

#[tokio::test]
async fn test_encrypted_connection_malformed_json_returns_parse_error() {
    use base64::Engine;

    let family_seed = b"integration-test-family-seed-ok!";
    let (mut client, handle) = spawn_handler(true, Some(family_seed)).await;

    let (session_id, handshake_key) = client_phase2_handshake(&mut client, family_seed).await;

    let b64 = base64::engine::general_purpose::STANDARD;
    let client_nonce = {
        let mut n = [0u8; 32];
        rand::RngCore::fill_bytes(&mut rand::thread_rng(), &mut n);
        n
    };
    let negotiate_req = serde_json::json!({
        "jsonrpc": "2.0",
        "method": "btsp.negotiate",
        "params": {
            "session_id": session_id,
            "ciphers": ["chacha20-poly1305"],
            "client_nonce": b64.encode(client_nonce),
        },
        "id": 1
    });
    client.write_all(format!("{negotiate_req}\n").as_bytes()).await.unwrap();

    let mut buf = vec![0u8; 4096];
    let negotiate_resp = super::tests_support::read_json_line_raw(&mut client, &mut buf).await;
    let server_nonce =
        b64.decode(negotiate_resp["result"]["server_nonce"].as_str().unwrap()).unwrap();
    let keys =
        crate::btsp::phase3::Phase3Keys::derive(&handshake_key, &client_nonce, &server_nonce, true)
            .unwrap();

    let bad_resp = encrypted_roundtrip(&mut client, &keys, b"{not valid json").await;
    assert_eq!(bad_resp["error"]["code"], -32700);

    let good_resp = encrypted_roundtrip(
        &mut client,
        &keys,
        br#"{"jsonrpc":"2.0","method":"health.check","params":{},"id":4}"#,
    )
    .await;
    assert_eq!(good_resp["id"], 4);
    assert!(good_resp["result"].is_object());

    client.shutdown().await.unwrap();
    let server_result = handle.await.unwrap();
    assert!(server_result.is_ok());
}

#[test]
fn test_cleanup_socket_at_directory_logs_no_panic() {
    let dir = tempfile::tempdir().expect("tempdir");
    let sock = dir.path().join("not-a-socket");
    std::fs::create_dir(&sock).unwrap();
    cleanup_socket_at(&sock);
    assert!(sock.exists(), "directory should remain when cleanup cannot remove it");
}

#[test]
fn test_uds_server_dev_mode_via_insecure_env() {
    let dir = tempfile::tempdir().expect("tempdir");
    let sock = dir.path().join("dev-mode.sock");

    temp_env::with_vars(
        [
            ("BIOMEOS_INSECURE", Some("1")),
            ("FAMILY_ID", None::<&str>),
            ("RHIZOCRYPT_FAMILY_ID", None),
        ],
        || {
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async {
                let primal = test_primal().await;
                let server = UdsJsonRpcServer::new(primal, sock.clone());
                let (shutdown_tx, shutdown_rx) = watch::channel(false);
                let ready = Arc::new(tokio::sync::Notify::new());
                let ready_rx = Arc::clone(&ready);

                let handle =
                    tokio::spawn(
                        async move { server.serve_with_ready(shutdown_rx, ready_rx).await },
                    );
                ready.notified().await;

                let stream = tokio::net::UnixStream::connect(&sock).await.expect("connect");
                let (reader, mut writer) = stream.into_split();
                let req = r#"{"jsonrpc":"2.0","method":"health.check","params":{},"id":1}"#;
                writer.write_all(format!("{req}\n").as_bytes()).await.unwrap();
                writer.shutdown().await.unwrap();

                let mut lines = BufReader::new(reader).lines();
                let line = lines.next_line().await.unwrap().expect("response");
                let resp: serde_json::Value = serde_json::from_str(&line).unwrap();
                assert!(resp["result"].is_object());

                let _ = shutdown_tx.send(true);
                let _ = handle.await;
            });
        },
    );
}

#[tokio::test]
async fn test_uds_serve_without_ready_notify() {
    let dir = tempfile::tempdir().expect("tempdir");
    let sock = dir.path().join("serve-only.sock");
    let primal = test_primal().await;

    let server = UdsJsonRpcServer::new(primal, sock.clone());
    let (shutdown_tx, shutdown_rx) = watch::channel(false);

    let handle = tokio::spawn(async move { server.serve(shutdown_rx).await });

    for _ in 0..100 {
        if tokio::net::UnixStream::connect(&sock).await.is_ok() {
            break;
        }
        tokio::task::yield_now().await;
    }

    let stream = tokio::net::UnixStream::connect(&sock).await.expect("connect");
    drop(stream);

    let _ = shutdown_tx.send(true);
    let result = handle.await.unwrap();
    assert!(result.is_ok());
    assert!(!sock.exists(), "socket cleaned up after serve()");
}

fn capability_symlink_path(dir: &std::path::Path) -> std::path::PathBuf {
    dir.join(format!(
        "{}{}",
        rhizo_crypt_core::niche::DOMAIN,
        rhizo_crypt_core::constants::SOCKET_FILE_EXTENSION
    ))
}

#[test]
fn test_uds_remove_capability_symlink_skips_foreign_target() {
    let dir = tempfile::tempdir().expect("tempdir");
    let sock = dir.path().join("server.sock");
    let symlink = capability_symlink_path(dir.path());
    let other = dir.path().join("other.sock");
    std::fs::write(&other, "").unwrap();
    std::os::unix::fs::symlink(&other, &symlink).unwrap();

    let rt = tokio::runtime::Runtime::new().unwrap();
    let primal = rt.block_on(test_primal());
    let server = UdsJsonRpcServer::new(primal, sock);
    server.cleanup();

    assert!(symlink.exists(), "foreign capability symlink must not be removed");
    assert_eq!(std::fs::read_link(&symlink).unwrap(), other);
}

#[tokio::test]
async fn test_uds_capability_symlink_creation_fails_when_dag_sock_is_directory() {
    let dir = tempfile::tempdir().expect("tempdir");
    let sock = dir.path().join("rhizocrypt-test.sock");
    let symlink = capability_symlink_path(dir.path());
    std::fs::create_dir(&symlink).unwrap();

    let primal = test_primal().await;
    let server = UdsJsonRpcServer::new(primal, sock.clone());
    let (shutdown_tx, shutdown_rx) = watch::channel(false);
    let ready = Arc::new(tokio::sync::Notify::new());
    let ready_rx = Arc::clone(&ready);

    let handle = tokio::spawn(async move { server.serve_with_ready(shutdown_rx, ready_rx).await });
    ready.notified().await;

    assert!(sock.exists(), "primary socket should bind even when symlink creation fails");
    assert!(symlink.is_dir(), "blocked dag.sock path should remain a directory");

    let stream = tokio::net::UnixStream::connect(&sock).await.expect("connect");
    drop(stream);

    let _ = shutdown_tx.send(true);
    let _ = handle.await;
}

#[test]
fn test_uds_serve_btsp_production_mode_with_family_seed() {
    let dir = tempfile::tempdir().expect("tempdir");
    let sock = dir.path().join("btsp-prod.sock");

    temp_env::with_vars(
        [
            ("BIOMEOS_INSECURE", None::<&str>),
            ("FAMILY_ID", Some("test-family")),
            ("FAMILY_SEED", Some("integration-test-family-seed-ok!")),
            ("RHIZOCRYPT_FAMILY_ID", None::<&str>),
            ("RHIZOCRYPT_FAMILY_SEED", None::<&str>),
        ],
        || {
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async {
                let primal = test_primal().await;
                let server = UdsJsonRpcServer::new(primal, sock.clone());
                let (shutdown_tx, shutdown_rx) = watch::channel(false);
                let ready = Arc::new(tokio::sync::Notify::new());
                let ready_rx = Arc::clone(&ready);

                let handle =
                    tokio::spawn(
                        async move { server.serve_with_ready(shutdown_rx, ready_rx).await },
                    );
                ready.notified().await;

                let stream = tokio::net::UnixStream::connect(&sock).await.expect("connect");
                let (reader, mut writer) = stream.into_split();
                let req = r#"{"jsonrpc":"2.0","method":"health.check","params":{},"id":1}"#;
                writer.write_all(format!("{req}\n").as_bytes()).await.unwrap();
                writer.shutdown().await.unwrap();

                let mut lines = BufReader::new(reader).lines();
                let line = lines.next_line().await.unwrap().expect("response");
                let resp: serde_json::Value = serde_json::from_str(&line).unwrap();
                assert!(resp["result"].is_object());

                let _ = shutdown_tx.send(true);
                let _ = handle.await;
            });
        },
    );
}

#[test]
fn test_uds_serve_btsp_production_mode_warns_missing_family_seed() {
    let dir = tempfile::tempdir().expect("tempdir");
    let sock = dir.path().join("btsp-no-seed.sock");

    temp_env::with_vars(
        [
            ("BIOMEOS_INSECURE", None::<&str>),
            ("FAMILY_ID", Some("test-family")),
            ("FAMILY_SEED", None::<&str>),
            ("RHIZOCRYPT_FAMILY_ID", None::<&str>),
            ("RHIZOCRYPT_FAMILY_SEED", None::<&str>),
        ],
        || {
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async {
                let primal = test_primal().await;
                let server = UdsJsonRpcServer::new(primal, sock.clone());
                let (shutdown_tx, shutdown_rx) = watch::channel(false);
                let ready = Arc::new(tokio::sync::Notify::new());
                let ready_rx = Arc::clone(&ready);

                let handle =
                    tokio::spawn(
                        async move { server.serve_with_ready(shutdown_rx, ready_rx).await },
                    );
                ready.notified().await;

                let stream = tokio::net::UnixStream::connect(&sock).await.expect("connect");
                drop(stream);

                let _ = shutdown_tx.send(true);
                let _ = handle.await;
            });
        },
    );
}

#[test]
fn test_uds_serve_rejects_connection_when_btsp_required_without_seed() {
    let dir = tempfile::tempdir().expect("tempdir");
    let sock = dir.path().join("btsp-reject.sock");

    temp_env::with_vars(
        [
            ("BIOMEOS_INSECURE", None::<&str>),
            ("FAMILY_ID", Some("test-family")),
            ("FAMILY_SEED", None::<&str>),
            ("RHIZOCRYPT_FAMILY_ID", None::<&str>),
            ("RHIZOCRYPT_FAMILY_SEED", None::<&str>),
        ],
        || {
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async {
                let primal = test_primal().await;
                let server = UdsJsonRpcServer::new(primal, sock.clone());
                let (shutdown_tx, shutdown_rx) = watch::channel(false);
                let ready = Arc::new(tokio::sync::Notify::new());
                let ready_rx = Arc::clone(&ready);

                let handle =
                    tokio::spawn(
                        async move { server.serve_with_ready(shutdown_rx, ready_rx).await },
                    );
                ready.notified().await;

                let mut client = tokio::net::UnixStream::connect(&sock).await.expect("connect");
                let req = r#"{"jsonrpc":"2.0","method":"health.check","params":{},"id":1}"#;
                client.write_all(format!("{req}\n").as_bytes()).await.unwrap();
                client.shutdown().await.unwrap();

                let _ = shutdown_tx.send(true);
                let _ = handle.await;
            });
        },
    );
}

#[tokio::test]
async fn test_btsp_required_empty_connection_eof() {
    let (client, handle) = spawn_handler(true, Some(b"integration-test-family-seed-ok!")).await;
    drop(client);

    let server_result = handle.await.unwrap();
    assert!(server_result.is_ok(), "immediate EOF with seed should be clean: {server_result:?}");
}

#[tokio::test]
async fn test_detect_btsp_json_line_eof_before_newline() {
    let family_seed = b"integration-test-family-seed-ok!";
    let (mut client, handle) = spawn_handler(true, Some(family_seed)).await;

    client.write_all(b"{").await.unwrap();
    client.shutdown().await.unwrap();

    let server_result = handle.await.unwrap();
    assert!(server_result.is_ok(), "EOF mid-line should be clean: {server_result:?}");
}

#[tokio::test]
async fn test_phase3_negotiate_invalid_client_nonce_fallback() {
    let family_seed = b"integration-test-family-seed-ok!";
    let (mut client, handle) = spawn_handler(true, Some(family_seed)).await;

    let (session_id, _handshake_key) = client_phase2_handshake(&mut client, family_seed).await;

    let negotiate_req = serde_json::json!({
        "jsonrpc": "2.0",
        "method": "btsp.negotiate",
        "params": {
            "session_id": session_id,
            "ciphers": ["chacha20-poly1305"],
            "client_nonce": "not-valid-base64!!!",
        },
        "id": 1
    });
    client.write_all(format!("{negotiate_req}\n").as_bytes()).await.unwrap();

    let req = r#"{"jsonrpc":"2.0","method":"health.check","params":{},"id":2}"#;
    client.write_all(format!("{req}\n").as_bytes()).await.unwrap();
    client.shutdown().await.unwrap();

    let mut lines = BufReader::new(&mut client).lines();
    let mut responses = Vec::new();
    while let Some(line) = lines.next_line().await.unwrap() {
        responses.push(serde_json::from_str::<serde_json::Value>(&line).unwrap());
    }
    let health = responses
        .iter()
        .find(|r| r.get("id") == Some(&serde_json::json!(2)))
        .expect("plaintext health.check after negotiate parse error fallback");
    assert!(health["result"].is_object());

    let server_result = handle.await.unwrap();
    assert!(server_result.is_ok());
}
