// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2024–2026 ecoPrimals Project

//! UDS transport integration tests.

use rhizo_crypt_core::discovery::PrimalManifest;
use rhizo_crypt_core::{PrimalLifecycle, RhizoCrypt, RhizoCryptConfig};
use rhizo_crypt_rpc::jsonrpc::uds::UdsJsonRpcServer;
use rhizocrypt_service::run_server_with_ready;
use std::sync::Arc;
use std::time::Duration;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::UnixStream;

/// socat-style validation: raw newline JSON-RPC over UDS.
#[tokio::test]
async fn test_uds_socat_style_health_liveness() {
    let dir = tempfile::tempdir().expect("tempdir");
    let sock = dir.path().join("socat-test.sock");

    let config = RhizoCryptConfig::default();
    let mut primal = RhizoCrypt::new(config);
    primal.start().await.expect("primal should start");
    let primal = Arc::new(primal);

    let uds = UdsJsonRpcServer::new(Arc::clone(&primal), sock.clone());
    let (shutdown_tx, shutdown_rx) = tokio::sync::watch::channel(false);
    let ready = Arc::new(tokio::sync::Notify::new());
    let ready_rx = Arc::clone(&ready);

    let handle = tokio::spawn(async move { uds.serve_with_ready(shutdown_rx, ready_rx).await });
    ready.notified().await;

    let stream = UnixStream::connect(&sock).await.expect("connect");
    let (reader, mut writer) = stream.into_split();

    let req = "{\"jsonrpc\":\"2.0\",\"method\":\"health.liveness\",\"params\":{},\"id\":1}\n";
    writer.write_all(req.as_bytes()).await.unwrap();
    writer.shutdown().await.unwrap();

    let mut lines = BufReader::new(reader).lines();
    let line = lines.next_line().await.unwrap().expect("response");
    let resp: serde_json::Value = serde_json::from_str(&line).unwrap();
    assert_eq!(resp["jsonrpc"], "2.0");
    assert!(resp.get("result").is_some() || resp.get("error").is_some());

    let _ = shutdown_tx.send(true);
    let _ = handle.await;
}

/// Server with --unix creates socket file and cleans up on shutdown.
#[tokio::test]
async fn test_uds_server_lifecycle() {
    let dir = tempfile::tempdir().expect("tempdir");
    let sock = dir.path().join("lifecycle.sock");

    let config = RhizoCryptConfig::default();
    let mut primal = RhizoCrypt::new(config);
    primal.start().await.expect("primal should start");
    let primal = Arc::new(primal);

    let uds = UdsJsonRpcServer::new(primal, sock.clone());
    let (shutdown_tx, shutdown_rx) = tokio::sync::watch::channel(false);
    let ready = Arc::new(tokio::sync::Notify::new());
    let ready_rx = Arc::clone(&ready);

    assert!(!sock.exists(), "socket should not exist before serve");

    let handle = tokio::spawn(async move { uds.serve_with_ready(shutdown_rx, ready_rx).await });
    ready.notified().await;

    assert!(sock.exists(), "socket should exist after serve starts");

    let _ = shutdown_tx.send(true);
    let _ = handle.await;

    assert!(!sock.exists(), "socket should be cleaned up after shutdown");
}

/// Run server with UDS enabled (empty path = default).
#[test]
fn test_run_server_with_uds() {
    let dir = tempfile::tempdir().expect("tempdir");
    let sock = dir.path().join("run-server-uds.sock");

    let rt =
        tokio::runtime::Builder::new_multi_thread().worker_threads(2).enable_all().build().unwrap();

    let ready = Arc::new(tokio::sync::Notify::new());
    let ready_clone = Arc::clone(&ready);
    let sock_str = sock.to_string_lossy().to_string();

    let handle = rt.spawn(async move {
        let _ = run_server_with_ready(
            Some(0),
            Some("127.0.0.1".to_string()),
            Some(sock_str),
            Some(ready_clone),
        )
        .await;
    });

    rt.block_on(async {
        tokio::time::timeout(Duration::from_secs(10), ready.notified())
            .await
            .expect("server should become ready within 10s");

        assert!(sock.exists(), "UDS socket should be created");

        handle.abort();
        let _ = handle.await;
    });
}

/// Composition-load: many concurrent UDS clients issuing JSON-RPC.
///
/// Simulates the composition load that downstream springs (wetSpring,
/// ludoSpring, healthSpring) apply when trio IPC is active. Validates
/// that UDS remains stable under parallel connection pressure.
#[tokio::test]
async fn test_uds_composition_load_concurrent_clients() {
    const CONCURRENT_CLIENTS: usize = 50;

    let dir = tempfile::tempdir().expect("tempdir");
    let sock = dir.path().join("comp-load.sock");

    let config = RhizoCryptConfig::default();
    let mut primal = RhizoCrypt::new(config);
    primal.start().await.expect("primal should start");
    let primal = Arc::new(primal);

    let uds = UdsJsonRpcServer::new(Arc::clone(&primal), sock.clone());
    let (shutdown_tx, shutdown_rx) = tokio::sync::watch::channel(false);
    let ready = Arc::new(tokio::sync::Notify::new());
    let ready_rx = Arc::clone(&ready);

    let handle = tokio::spawn(async move { uds.serve_with_ready(shutdown_rx, ready_rx).await });
    ready.notified().await;

    let mut tasks = Vec::with_capacity(CONCURRENT_CLIENTS);
    for i in 0..CONCURRENT_CLIENTS {
        let sock = sock.clone();
        tasks.push(tokio::spawn(async move {
            let stream = UnixStream::connect(&sock).await.expect("connect");
            let (reader, mut writer) = stream.into_split();

            let req = format!(
                "{{\"jsonrpc\":\"2.0\",\"method\":\"health.check\",\"params\":{{}},\"id\":{i}}}\n"
            );
            writer.write_all(req.as_bytes()).await.unwrap();
            writer.shutdown().await.unwrap();

            let mut lines = BufReader::new(reader).lines();
            let line = lines.next_line().await.unwrap().expect("response");
            let resp: serde_json::Value = serde_json::from_str(&line).unwrap();
            assert_eq!(resp["jsonrpc"], "2.0");
            assert!(resp.get("result").is_some(), "client {i} expected result, got: {resp}");
        }));
    }

    for task in tasks {
        task.await.unwrap();
    }

    let _ = shutdown_tx.send(true);
    let _ = handle.await;
}

/// Composition-load: sustained sequential requests on a single UDS
/// connection (simulates a long-lived spring client).
#[tokio::test]
async fn test_uds_sustained_sequential_requests() {
    const REQUEST_COUNT: usize = 200;

    let dir = tempfile::tempdir().expect("tempdir");
    let sock = dir.path().join("sustained.sock");

    let config = RhizoCryptConfig::default();
    let mut primal = RhizoCrypt::new(config);
    primal.start().await.expect("primal should start");
    let primal = Arc::new(primal);

    let uds = UdsJsonRpcServer::new(Arc::clone(&primal), sock.clone());
    let (shutdown_tx, shutdown_rx) = tokio::sync::watch::channel(false);
    let ready = Arc::new(tokio::sync::Notify::new());
    let ready_rx = Arc::clone(&ready);

    let handle = tokio::spawn(async move { uds.serve_with_ready(shutdown_rx, ready_rx).await });
    ready.notified().await;

    let stream = UnixStream::connect(&sock).await.expect("connect");
    let (reader, mut writer) = tokio::io::split(stream);
    let mut lines = BufReader::new(reader).lines();

    for i in 0..REQUEST_COUNT {
        let req = format!(
            "{{\"jsonrpc\":\"2.0\",\"method\":\"health.liveness\",\"params\":{{}},\"id\":{i}}}\n"
        );
        writer.write_all(req.as_bytes()).await.unwrap();
        writer.flush().await.unwrap();

        let line = lines.next_line().await.unwrap().expect("response");
        let resp: serde_json::Value = serde_json::from_str(&line).unwrap();
        assert_eq!(resp["jsonrpc"], "2.0", "request {i} bad jsonrpc");
    }

    drop(writer);
    let _ = shutdown_tx.send(true);
    let _ = handle.await;
}

/// Graceful shutdown: server stops accepting new connections and cleans
/// up the socket file after existing connections complete.
#[tokio::test]
async fn test_uds_graceful_shutdown_under_load() {
    let dir = tempfile::tempdir().expect("tempdir");
    let sock = dir.path().join("graceful.sock");

    let config = RhizoCryptConfig::default();
    let mut primal = RhizoCrypt::new(config);
    primal.start().await.expect("primal should start");
    let primal = Arc::new(primal);

    let uds = UdsJsonRpcServer::new(Arc::clone(&primal), sock.clone());
    let (shutdown_tx, shutdown_rx) = tokio::sync::watch::channel(false);
    let ready = Arc::new(tokio::sync::Notify::new());
    let ready_rx = Arc::clone(&ready);

    let handle = tokio::spawn(async move { uds.serve_with_ready(shutdown_rx, ready_rx).await });
    ready.notified().await;

    let mut tasks = Vec::with_capacity(10);
    for i in 0..10 {
        let sock = sock.clone();
        tasks.push(tokio::spawn(async move {
            let stream = UnixStream::connect(&sock).await.expect("connect");
            let (reader, mut writer) = stream.into_split();

            let req = format!(
                "{{\"jsonrpc\":\"2.0\",\"method\":\"health.check\",\"params\":{{}},\"id\":{i}}}\n"
            );
            writer.write_all(req.as_bytes()).await.unwrap();
            writer.shutdown().await.unwrap();

            let mut lines = BufReader::new(reader).lines();
            let line = lines.next_line().await.unwrap().expect("response");
            let resp: serde_json::Value = serde_json::from_str(&line).unwrap();
            assert_eq!(resp["jsonrpc"], "2.0");
        }));
    }

    for task in tasks {
        task.await.unwrap();
    }

    let _ = shutdown_tx.send(true);

    let result = tokio::time::timeout(std::time::Duration::from_secs(5), handle)
        .await
        .expect("server should shut down within 5s");
    assert!(result.is_ok(), "server should shut down cleanly");
    assert!(!sock.exists(), "socket should be cleaned up");
}

/// Verify that `PrimalManifest` round-trips correctly with rhizoCrypt's
/// canonical capabilities, validating the PG-32 discovery contract.
///
/// Uses a dedicated runtime to allow `temp_env::with_var` (sync) to wrap
/// async manifest operations without nesting runtimes.
#[test]
fn test_manifest_publish_lifecycle() {
    let dir = tempfile::tempdir().unwrap();
    let biomeos_dir = dir.path().join("biomeos");
    std::fs::create_dir_all(&biomeos_dir).unwrap();

    let manifest_path = biomeos_dir.join("rhizocrypt.json");
    let sock_path = dir.path().join("rhizocrypt.sock");

    let manifest = PrimalManifest {
        primal: "rhizocrypt".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        socket: sock_path.display().to_string(),
        address: None,
        capabilities: rhizo_crypt_core::niche::CAPABILITIES
            .iter()
            .map(|s| (*s).to_string())
            .collect(),
    };

    let rt = tokio::runtime::Runtime::new().unwrap();

    temp_env::with_var("XDG_RUNTIME_DIR", Some(dir.path().to_str().unwrap()), || {
        rt.block_on(async {
            rhizo_crypt_core::discovery::publish_manifest(&manifest)
                .await
                .expect("publish should succeed");
        });
    });

    assert!(manifest_path.exists(), "manifest should exist after publish");

    let contents = std::fs::read_to_string(&manifest_path).unwrap();
    let loaded: PrimalManifest = serde_json::from_str(&contents).unwrap();
    assert_eq!(loaded.primal, "rhizocrypt");
    assert!(!loaded.socket.is_empty(), "socket path should be populated");
    assert!(!loaded.capabilities.is_empty(), "capabilities should be populated");
    assert!(
        loaded.capabilities.iter().any(|c| c.starts_with("dag.")),
        "should advertise dag.* capabilities"
    );

    let first_cap = loaded.capabilities[0].clone();
    temp_env::with_var("XDG_RUNTIME_DIR", Some(dir.path().to_str().unwrap()), || {
        rt.block_on(async {
            let found = rhizo_crypt_core::discovery::discover_by_capability(&first_cap).await;
            assert!(!found.is_empty(), "discover_by_capability should find rhizocrypt");
            assert_eq!(found[0].primal, "rhizocrypt");
        });
    });

    temp_env::with_var("XDG_RUNTIME_DIR", Some(dir.path().to_str().unwrap()), || {
        rt.block_on(async {
            rhizo_crypt_core::discovery::unpublish_manifest("rhizocrypt").await;
        });
    });

    assert!(!manifest_path.exists(), "manifest should be cleaned up after unpublish");
}
