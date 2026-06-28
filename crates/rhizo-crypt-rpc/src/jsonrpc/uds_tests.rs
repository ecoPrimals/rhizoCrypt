// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2024–2026 ecoPrimals Project

//! UDS server lifecycle and basic connection tests.

#![expect(clippy::unwrap_used, clippy::expect_used, reason = "test code")]

use super::tests_support::test_primal;
use super::*;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};

#[tokio::test]
async fn test_uds_roundtrip() {
    let dir = tempfile::tempdir().expect("tempdir");
    let sock = dir.path().join("rhizocrypt-test.sock");
    let primal = test_primal().await;

    let server = UdsJsonRpcServer::new(primal, sock.clone());
    let (shutdown_tx, shutdown_rx) = watch::channel(false);
    let ready = Arc::new(tokio::sync::Notify::new());
    let ready_rx = Arc::clone(&ready);

    let handle = tokio::spawn(async move { server.serve_with_ready(shutdown_rx, ready_rx).await });
    ready.notified().await;

    let stream = tokio::net::UnixStream::connect(&sock).await.expect("connect");
    let (reader, mut writer) = stream.into_split();

    let req = r#"{"jsonrpc":"2.0","method":"health.check","params":{},"id":1}"#;
    writer.write_all(format!("{req}\n").as_bytes()).await.unwrap();
    writer.shutdown().await.unwrap();

    let mut lines = BufReader::new(reader).lines();
    let line = lines.next_line().await.unwrap().expect("response");
    let resp: serde_json::Value = serde_json::from_str(&line).unwrap();
    assert_eq!(resp["jsonrpc"], "2.0");
    assert!(resp["result"].is_object());

    let _ = shutdown_tx.send(true);
    let _ = handle.await;
}

#[tokio::test]
async fn test_uds_stale_socket_removed() {
    let dir = tempfile::tempdir().expect("tempdir");
    let sock = dir.path().join("stale-test.sock");
    std::fs::write(&sock, "stale").unwrap();
    assert!(sock.exists());

    let primal = test_primal().await;
    let server = UdsJsonRpcServer::new(primal, sock.clone());
    let (shutdown_tx, shutdown_rx) = watch::channel(false);
    let ready = Arc::new(tokio::sync::Notify::new());
    let ready_rx = Arc::clone(&ready);

    let handle = tokio::spawn(async move { server.serve_with_ready(shutdown_rx, ready_rx).await });
    ready.notified().await;

    assert!(sock.exists(), "socket should exist (re-bound)");

    let _ = shutdown_tx.send(true);
    let _ = handle.await;
}

#[tokio::test]
async fn test_uds_cleanup_removes_file() {
    let dir = tempfile::tempdir().expect("tempdir");
    let sock = dir.path().join("cleanup-test.sock");
    std::fs::write(&sock, "").unwrap();
    assert!(sock.exists());

    cleanup_socket_at(&sock);
    assert!(!sock.exists());
}

#[tokio::test]
async fn test_uds_cleanup_nonexistent_noop() {
    let dir = tempfile::tempdir().expect("tempdir");
    let sock = dir.path().join("nonexistent.sock");
    cleanup_socket_at(&sock);
}

#[test]
fn test_default_socket_path_contains_biomeos() {
    let path = default_socket_path();
    let path_str = path.to_string_lossy();
    assert!(
        path_str.contains("biomeos") || path_str.contains("rhizocrypt"),
        "path should reference biomeos or rhizocrypt: {path_str}"
    );
}

#[test]
fn test_socket_path_accessor() {
    let path = std::path::PathBuf::from("/tmp/test.sock");
    let primal = tokio::runtime::Runtime::new().unwrap().block_on(test_primal());
    let server = UdsJsonRpcServer::new(primal, path.clone());
    assert_eq!(server.socket_path(), path);
}

#[tokio::test]
async fn test_uds_server_cleanup_idempotent() {
    let dir = tempfile::tempdir().expect("tempdir");
    let sock = dir.path().join("idempotent.sock");
    let primal = test_primal().await;
    let server = UdsJsonRpcServer::new(primal, sock.clone());
    server.cleanup();
    server.cleanup();
}

#[tokio::test]
async fn test_uds_multiple_sequential_requests() {
    let dir = tempfile::tempdir().expect("tempdir");
    let sock = dir.path().join("seq-test.sock");
    let primal = test_primal().await;

    let server = UdsJsonRpcServer::new(primal, sock.clone());
    let (shutdown_tx, shutdown_rx) = watch::channel(false);
    let ready = Arc::new(tokio::sync::Notify::new());
    let ready_rx = Arc::clone(&ready);

    let handle = tokio::spawn(async move { server.serve_with_ready(shutdown_rx, ready_rx).await });
    ready.notified().await;

    for i in 0..5_u32 {
        let stream = tokio::net::UnixStream::connect(&sock).await.expect("connect");
        let (reader, mut writer) = stream.into_split();

        let req = format!(r#"{{"jsonrpc":"2.0","method":"health.check","params":{{}},"id":{i}}}"#);
        writer.write_all(format!("{req}\n").as_bytes()).await.unwrap();
        writer.shutdown().await.unwrap();

        let mut lines = BufReader::new(reader).lines();
        let line = lines.next_line().await.unwrap().expect("response");
        let resp: serde_json::Value = serde_json::from_str(&line).unwrap();
        assert_eq!(resp["id"], i);
    }

    let _ = shutdown_tx.send(true);
    let _ = handle.await;
}

#[tokio::test]
async fn test_uds_serve_creates_parent_dirs() {
    let dir = tempfile::tempdir().expect("tempdir");
    let nested = dir.path().join("deep").join("nested").join("test.sock");
    let primal = test_primal().await;

    let server = UdsJsonRpcServer::new(primal, nested.clone());
    let (shutdown_tx, shutdown_rx) = watch::channel(false);
    let ready = Arc::new(tokio::sync::Notify::new());
    let ready_rx = Arc::clone(&ready);

    let handle = tokio::spawn(async move { server.serve_with_ready(shutdown_rx, ready_rx).await });
    ready.notified().await;

    assert!(nested.exists(), "socket should exist under nested dirs");

    let _ = shutdown_tx.send(true);
    let _ = handle.await;
}

fn capability_symlink_path(dir: &std::path::Path) -> std::path::PathBuf {
    dir.join(format!(
        "{}{}",
        rhizo_crypt_core::niche::DOMAIN,
        rhizo_crypt_core::constants::SOCKET_FILE_EXTENSION
    ))
}

#[tokio::test]
async fn test_uds_capability_symlink_created_and_removed_on_shutdown() {
    let dir = tempfile::tempdir().expect("tempdir");
    let sock = dir.path().join("rhizocrypt-test.sock");
    let symlink = capability_symlink_path(dir.path());
    let primal = test_primal().await;

    let server = UdsJsonRpcServer::new(primal, sock.clone());
    let (shutdown_tx, shutdown_rx) = watch::channel(false);
    let ready = Arc::new(tokio::sync::Notify::new());
    let ready_rx = Arc::clone(&ready);

    let handle = tokio::spawn(async move { server.serve_with_ready(shutdown_rx, ready_rx).await });
    ready.notified().await;

    assert!(symlink.is_symlink(), "capability symlink should exist during serve");
    assert_eq!(std::fs::read_link(&symlink).unwrap(), sock);

    let _ = shutdown_tx.send(true);
    let _ = handle.await;
    assert!(!symlink.exists(), "capability symlink should be removed on shutdown");
    assert!(!sock.exists(), "primary socket should be removed on shutdown");
}

#[tokio::test]
async fn test_uds_capability_symlink_replaces_stale_existing() {
    let dir = tempfile::tempdir().expect("tempdir");
    let sock = dir.path().join("rhizocrypt-test.sock");
    let symlink = capability_symlink_path(dir.path());
    let other = dir.path().join("other.sock");
    std::fs::write(&other, "").unwrap();
    std::os::unix::fs::symlink(&other, &symlink).unwrap();
    assert_eq!(std::fs::read_link(&symlink).unwrap(), other);

    let primal = test_primal().await;
    let server = UdsJsonRpcServer::new(primal, sock.clone());
    let (shutdown_tx, shutdown_rx) = watch::channel(false);
    let ready = Arc::new(tokio::sync::Notify::new());
    let ready_rx = Arc::clone(&ready);

    let handle = tokio::spawn(async move { server.serve_with_ready(shutdown_rx, ready_rx).await });
    ready.notified().await;

    assert_eq!(std::fs::read_link(&symlink).unwrap(), sock);

    let _ = shutdown_tx.send(true);
    let _ = handle.await;
}
