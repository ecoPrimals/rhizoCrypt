// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2024–2026 ecoPrimals Project

//! Server lifecycle tests: bind conflicts, discovery registration, graceful
//! shutdown, client operations, and manifest publishing.

use super::*;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;

#[test]
fn test_run_server_tcp_bind_conflict_returns_rpc_error() {
    temp_env::with_vars(
        [
            ("BIOMEOS_INSECURE", Some("1")),
            ("RHIZOCRYPT_PORT", None::<&str>),
            ("RHIZOCRYPT_RPC_PORT", None),
            ("RHIZOCRYPT_JSONRPC_PORT", None),
            ("DISCOVERY_ENDPOINT", None),
            ("RHIZOCRYPT_DISCOVERY_ADAPTER", None),
            ("RUST_LOG", Some("error")),
        ],
        || {
            let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
            let port = listener.local_addr().unwrap().port();

            let rt = tokio::runtime::Builder::new_multi_thread()
                .worker_threads(2)
                .enable_all()
                .build()
                .unwrap();
            let result = rt.block_on(run_server_with_ready(
                Some(port),
                Some("127.0.0.1".to_string()),
                None,
                None,
            ));
            assert!(matches!(result, Err(ServiceError::Rpc(_))));
        },
    );
}

#[test]
fn test_run_server_discovery_register_failure_continues() {
    temp_env::with_vars(
        [
            ("BIOMEOS_INSECURE", Some("1")),
            ("RHIZOCRYPT_PORT", Some("0")),
            ("RHIZOCRYPT_HOST", Some("127.0.0.1")),
            ("RHIZOCRYPT_DISCOVERY_ADAPTER", Some("127.0.0.1:59999")),
            ("DISCOVERY_ENDPOINT", None),
            ("RUST_LOG", Some("error")),
        ],
        || {
            let rt = tokio::runtime::Builder::new_multi_thread()
                .worker_threads(2)
                .enable_all()
                .build()
                .unwrap();
            let ready = Arc::new(tokio::sync::Notify::new());
            let ready_clone = Arc::clone(&ready);
            let handle = rt.spawn(async move {
                let _ = run_server_with_ready(
                    Some(0),
                    Some("127.0.0.1".to_string()),
                    None,
                    Some(ready_clone),
                )
                .await;
            });
            rt.block_on(async {
                tokio::time::timeout(Duration::from_secs(10), ready.notified())
                    .await
                    .expect("server should continue after discovery registration failure");
                handle.abort();
                let _ = handle.await;
            });
        },
    );
}

#[cfg(feature = "live-clients")]
#[test]
fn test_run_server_discovery_registration_success() {
    let rt =
        tokio::runtime::Builder::new_multi_thread().worker_threads(2).enable_all().build().unwrap();
    let discovery_addr = rt.block_on(super::tests_startup::spawn_tarpc_discovery_server(
        super::tests_startup::AcceptingDiscoveryServer,
    ));
    let adapter = discovery_addr.to_string();

    temp_env::with_vars(
        [
            ("BIOMEOS_INSECURE", Some("1")),
            ("RHIZOCRYPT_PORT", Some("0")),
            ("RHIZOCRYPT_HOST", Some("127.0.0.1")),
            ("DISCOVERY_ENDPOINT", None),
            ("RHIZOCRYPT_DISCOVERY_ADAPTER", Some(adapter.as_str())),
            ("RUST_LOG", Some("error")),
        ],
        || {
            let ready = Arc::new(tokio::sync::Notify::new());
            let ready_clone = Arc::clone(&ready);
            let handle = rt.spawn(async move {
                let _ = run_server_with_ready(
                    Some(0),
                    Some("127.0.0.1".to_string()),
                    None,
                    Some(ready_clone),
                )
                .await;
            });
            rt.block_on(async {
                tokio::time::timeout(Duration::from_secs(10), ready.notified())
                    .await
                    .expect("server should become ready with successful discovery registration");
                handle.abort();
                let _ = handle.await;
            });
        },
    );
}

#[cfg(feature = "live-clients")]
#[tokio::test]
async fn test_register_with_discovery_success() {
    let discovery_addr = super::tests_startup::spawn_tarpc_discovery_server(
        super::tests_startup::AcceptingDiscoveryServer,
    )
    .await;
    let our_addr: SocketAddr = "127.0.0.1:9400".parse().unwrap();
    let client = register_with_discovery(&discovery_addr.to_string(), our_addr)
        .await
        .expect("registration should succeed");
    assert!(client.is_connected().await);
}

#[cfg(feature = "live-clients")]
#[tokio::test]
async fn test_register_with_discovery_registration_rejected() {
    let discovery_addr = super::tests_startup::spawn_tarpc_discovery_server(
        super::tests_startup::RejectingDiscoveryServer,
    )
    .await;
    let our_addr: SocketAddr = "127.0.0.1:9400".parse().unwrap();
    let result = register_with_discovery(&discovery_addr.to_string(), our_addr).await;
    let err = result.err().expect("rejected registration should fail");
    match err {
        ServiceError::Discovery(msg) => {
            assert!(msg.contains("registration rejected"));
        }
        other => panic!("expected Discovery error, got: {other}"),
    }
}

#[cfg(unix)]
#[test]
fn test_run_server_uds_only_graceful_shutdown_on_sigint() {
    use nix::sys::signal::{Signal, kill};
    use nix::unistd::Pid;

    let dir = tempfile::tempdir().expect("tempdir");
    let sock = dir.path().join("uds-shutdown.sock");
    let sock_str = sock.to_string_lossy().to_string();
    let pid = Pid::from_raw(i32::try_from(std::process::id()).unwrap());

    temp_env::with_vars(
        [
            ("BIOMEOS_INSECURE", Some("1")),
            ("RHIZOCRYPT_PORT", None::<&str>),
            ("RHIZOCRYPT_RPC_PORT", None),
            ("RHIZOCRYPT_JSONRPC_PORT", None),
            ("DISCOVERY_ENDPOINT", None),
            ("RHIZOCRYPT_DISCOVERY_ADAPTER", None),
            ("RUST_LOG", Some("error")),
        ],
        || {
            let rt = tokio::runtime::Builder::new_multi_thread()
                .worker_threads(2)
                .enable_all()
                .build()
                .unwrap();
            let ready = Arc::new(tokio::sync::Notify::new());
            let ready_clone = Arc::clone(&ready);
            let handle = rt.spawn(async move {
                run_server_with_ready(None, None, Some(sock_str), Some(ready_clone)).await
            });
            rt.block_on(async {
                tokio::time::timeout(Duration::from_secs(10), ready.notified())
                    .await
                    .expect("UDS-only server should become ready");
                let signal_task = tokio::task::spawn_blocking(move || {
                    std::thread::sleep(Duration::from_millis(200));
                    kill(pid, Signal::SIGINT).expect("send SIGINT");
                });
                let result = tokio::time::timeout(Duration::from_secs(10), handle)
                    .await
                    .expect("server should shut down after SIGINT")
                    .expect("server task join");
                assert!(result.is_ok());
                let _ = signal_task.await;
            });
        },
    );
}

#[cfg(unix)]
#[test]
fn test_run_server_tcp_graceful_shutdown_on_sigterm() {
    use nix::sys::signal::{Signal, kill};
    use nix::unistd::Pid;

    let pid = Pid::from_raw(i32::try_from(std::process::id()).unwrap());

    temp_env::with_vars(
        [
            ("BIOMEOS_INSECURE", Some("1")),
            ("RHIZOCRYPT_PORT", None::<&str>),
            ("RHIZOCRYPT_RPC_PORT", None),
            ("RHIZOCRYPT_JSONRPC_PORT", None),
            ("DISCOVERY_ENDPOINT", None),
            ("RHIZOCRYPT_DISCOVERY_ADAPTER", None),
            ("RUST_LOG", Some("error")),
        ],
        || {
            let rt = tokio::runtime::Builder::new_multi_thread()
                .worker_threads(2)
                .enable_all()
                .build()
                .unwrap();
            let ready = Arc::new(tokio::sync::Notify::new());
            let ready_clone = Arc::clone(&ready);
            let handle = rt.spawn(async move {
                run_server_with_ready(
                    Some(0),
                    Some("127.0.0.1".to_string()),
                    None,
                    Some(ready_clone),
                )
                .await
            });
            rt.block_on(async {
                tokio::time::timeout(Duration::from_secs(10), ready.notified())
                    .await
                    .expect("TCP server should become ready");
                let signal_task = tokio::task::spawn_blocking(move || {
                    std::thread::sleep(Duration::from_millis(200));
                    kill(pid, Signal::SIGTERM).expect("send SIGTERM");
                });
                let result = tokio::time::timeout(Duration::from_secs(10), handle)
                    .await
                    .expect("server should shut down after SIGTERM")
                    .expect("server task join");
                assert!(result.is_ok());
                let _ = signal_task.await;
            });
        },
    );
}

#[test]
fn test_resolve_bind_addr_development_default_port() {
    use rhizo_crypt_core::constants;

    temp_env::with_vars(
        [
            ("RHIZOCRYPT_ENV", Some("development")),
            ("RHIZOCRYPT_RPC_PORT", None::<&str>),
            ("RHIZOCRYPT_PORT", None),
            ("RHIZOCRYPT_RPC_HOST", None),
            ("RHIZOCRYPT_HOST", None),
        ],
        || {
            let addr = resolve_bind_addr(None, None).unwrap();
            assert_eq!(addr.port(), constants::DEFAULT_RPC_PORT);
        },
    );
}

#[tokio::test]
async fn test_run_client_invalid_address() {
    let result = run_client("not-a-valid-address", ClientOperation::Health).await;
    assert!(matches!(result, Err(ServiceError::AddrParse(_))));
}

#[tokio::test]
async fn test_run_client_connection_failure() {
    let result = run_client("127.0.0.1:1", ClientOperation::Health).await;
    let err = result.expect_err("connection should fail");
    assert!(matches!(err, ServiceError::Config(_)));
    assert!(err.to_string().contains("Failed to connect"));
}

async fn assert_run_client_succeeds(operation: ClientOperation) {
    use rhizo_crypt_core::{PrimalLifecycle, RhizoCrypt, RhizoCryptConfig};
    use rhizo_crypt_rpc::server::RpcServer;

    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let bound_addr = listener.local_addr().unwrap();
    drop(listener);

    let config = RhizoCryptConfig::default();
    let mut primal = RhizoCrypt::new(config);
    primal.start().await.expect("primal should start");
    let primal = Arc::new(primal);
    let server = RpcServer::new(primal, bound_addr);
    let ready = server.ready_notifier();
    let handle = tokio::spawn(async move { server.serve().await });
    ready.notified().await;

    let addr_str = bound_addr.to_string();
    for _ in 0..50 {
        if run_client(&addr_str, operation.clone()).await.is_ok() {
            handle.abort();
            let _ = handle.await;
            return;
        }
        tokio::task::yield_now().await;
    }
    handle.abort();
    let _ = handle.await;
    panic!("run_client {operation:?} should succeed against running server");
}

#[tokio::test]
async fn test_run_client_health() {
    assert_run_client_succeeds(ClientOperation::Health).await;
}

#[tokio::test]
async fn test_run_client_list_sessions() {
    assert_run_client_succeeds(ClientOperation::ListSessions).await;
}

#[tokio::test]
async fn test_run_client_metrics() {
    assert_run_client_succeeds(ClientOperation::Metrics).await;
}

#[cfg(unix)]
#[test]
fn test_run_server_manifest_publish_failure_is_non_fatal() {
    let file = tempfile::NamedTempFile::new().expect("tempfile");
    let blocked = file.path().to_str().expect("utf-8 path");

    temp_env::with_vars(
        [
            ("XDG_RUNTIME_DIR", Some(blocked)),
            ("BIOMEOS_INSECURE", Some("1")),
            ("RHIZOCRYPT_PORT", None::<&str>),
            ("RHIZOCRYPT_RPC_PORT", None),
            ("RHIZOCRYPT_JSONRPC_PORT", None),
            ("DISCOVERY_ENDPOINT", None),
            ("RHIZOCRYPT_DISCOVERY_ADAPTER", None),
            ("RUST_LOG", Some("error")),
        ],
        || {
            let rt = tokio::runtime::Builder::new_multi_thread()
                .worker_threads(2)
                .enable_all()
                .build()
                .unwrap();
            let ready = Arc::new(tokio::sync::Notify::new());
            let ready_clone = Arc::clone(&ready);
            let dir = tempfile::tempdir().expect("tempdir");
            let sock = dir.path().join("manifest-fail.sock");
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
                    .expect("server should start even when manifest publish fails");
                handle.abort();
                let _ = handle.await;
            });
        },
    );
}
