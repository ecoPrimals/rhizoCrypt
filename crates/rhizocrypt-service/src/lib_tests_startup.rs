// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2024–2026 ecoPrimals Project

//! Service startup, TCP/UDS config, and discovery registration tests.

use super::*;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;

#[cfg(feature = "live-clients")]
async fn spawn_tarpc_discovery_server<S>(server: S) -> SocketAddr
where
    S: rhizo_crypt_core::clients::songbird_rpc::SongbirdRpc + Clone + Send + Sync + 'static,
{
    use futures_util::StreamExt;
    use tarpc::server::{self, Channel};
    use tarpc::tokio_serde::formats::Bincode;

    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    std::thread::spawn(move || {
        let rt = tokio::runtime::Builder::new_current_thread().enable_all().build().unwrap();
        rt.block_on(async move {
            loop {
                let Ok((stream, _)) = listener.accept().await else {
                    break;
                };
                let transport =
                    tarpc::serde_transport::Transport::from((stream, Bincode::default()));
                let channel = server::BaseChannel::with_defaults(transport);
                let s = server.clone();
                channel.execute(s.serve()).for_each(|f| f).await;
            }
        });
    });
    addr
}

#[cfg(feature = "live-clients")]
#[derive(Clone)]
struct AcceptingDiscoveryServer;

#[cfg(feature = "live-clients")]
impl rhizo_crypt_core::clients::songbird_rpc::SongbirdRpc for AcceptingDiscoveryServer {
    async fn discover(
        self,
        _: tarpc::context::Context,
        capability: String,
    ) -> Vec<rhizo_crypt_core::clients::songbird_rpc::RpcServiceInfo> {
        if capability == "signing" {
            vec![rhizo_crypt_core::clients::songbird_rpc::RpcServiceInfo {
                id: "mock-signing-1".to_string(),
                capability: "signing".to_string(),
                endpoint: "127.0.0.1:9500".to_string(),
                status: "healthy".to_string(),
                metadata: None,
            }]
        } else {
            vec![]
        }
    }

    async fn discover_all(
        self,
        _: tarpc::context::Context,
    ) -> Vec<rhizo_crypt_core::clients::songbird_rpc::RpcServiceInfo> {
        vec![]
    }

    async fn register(
        self,
        _: tarpc::context::Context,
        registration: rhizo_crypt_core::clients::songbird_rpc::RpcServiceRegistration,
    ) -> rhizo_crypt_core::clients::songbird_rpc::RpcRegistrationResult {
        rhizo_crypt_core::clients::songbird_rpc::RpcRegistrationResult {
            success: true,
            message: format!("Registered {}", registration.service_id),
        }
    }

    async fn unregister(
        self,
        _: tarpc::context::Context,
        _service_id: String,
    ) -> rhizo_crypt_core::clients::songbird_rpc::RpcRegistrationResult {
        rhizo_crypt_core::clients::songbird_rpc::RpcRegistrationResult {
            success: true,
            message: "Unregistered".to_string(),
        }
    }

    async fn health(
        self,
        _: tarpc::context::Context,
    ) -> rhizo_crypt_core::clients::songbird_rpc::RpcHealthStatus {
        rhizo_crypt_core::clients::songbird_rpc::RpcHealthStatus {
            status: "healthy".to_string(),
            version: "0.1.0-test".to_string(),
            uptime_seconds: 0,
            services_count: 1,
        }
    }

    async fn version(
        self,
        _: tarpc::context::Context,
    ) -> rhizo_crypt_core::clients::songbird_rpc::RpcVersionInfo {
        rhizo_crypt_core::clients::songbird_rpc::RpcVersionInfo {
            version: "0.1.0-test".to_string(),
            protocol: "tarpc-1.0".to_string(),
            capabilities: vec!["discovery".to_string()],
        }
    }
}

#[cfg(feature = "live-clients")]
#[derive(Clone)]
struct RejectingDiscoveryServer;

#[cfg(feature = "live-clients")]
impl rhizo_crypt_core::clients::songbird_rpc::SongbirdRpc for RejectingDiscoveryServer {
    async fn discover(
        self,
        _: tarpc::context::Context,
        _capability: String,
    ) -> Vec<rhizo_crypt_core::clients::songbird_rpc::RpcServiceInfo> {
        vec![]
    }

    async fn discover_all(
        self,
        _: tarpc::context::Context,
    ) -> Vec<rhizo_crypt_core::clients::songbird_rpc::RpcServiceInfo> {
        vec![]
    }

    async fn register(
        self,
        _: tarpc::context::Context,
        _registration: rhizo_crypt_core::clients::songbird_rpc::RpcServiceRegistration,
    ) -> rhizo_crypt_core::clients::songbird_rpc::RpcRegistrationResult {
        rhizo_crypt_core::clients::songbird_rpc::RpcRegistrationResult {
            success: false,
            message: "registration rejected by policy".to_string(),
        }
    }

    async fn unregister(
        self,
        _: tarpc::context::Context,
        _service_id: String,
    ) -> rhizo_crypt_core::clients::songbird_rpc::RpcRegistrationResult {
        rhizo_crypt_core::clients::songbird_rpc::RpcRegistrationResult {
            success: true,
            message: "Unregistered".to_string(),
        }
    }

    async fn health(
        self,
        _: tarpc::context::Context,
    ) -> rhizo_crypt_core::clients::songbird_rpc::RpcHealthStatus {
        rhizo_crypt_core::clients::songbird_rpc::RpcHealthStatus {
            status: "healthy".to_string(),
            version: "0.1.0-test".to_string(),
            uptime_seconds: 0,
            services_count: 0,
        }
    }

    async fn version(
        self,
        _: tarpc::context::Context,
    ) -> rhizo_crypt_core::clients::songbird_rpc::RpcVersionInfo {
        rhizo_crypt_core::clients::songbird_rpc::RpcVersionInfo {
            version: "0.1.0-test".to_string(),
            protocol: "tarpc-1.0".to_string(),
            capabilities: vec!["discovery".to_string()],
        }
    }
}

#[test]
fn test_has_explicit_tcp_config_unset() {
    temp_env::with_vars(
        [
            ("RHIZOCRYPT_PORT", None::<&str>),
            ("RHIZOCRYPT_RPC_PORT", None),
            ("RHIZOCRYPT_JSONRPC_PORT", None),
        ],
        || {
            assert!(!has_explicit_tcp_config());
        },
    );
}

#[test]
fn test_has_explicit_tcp_config_port_env() {
    temp_env::with_vars(
        [
            ("RHIZOCRYPT_PORT", Some("9400")),
            ("RHIZOCRYPT_RPC_PORT", None::<&str>),
            ("RHIZOCRYPT_JSONRPC_PORT", None),
        ],
        || {
            assert!(has_explicit_tcp_config());
        },
    );
}

#[test]
fn test_has_explicit_tcp_config_rpc_port_env() {
    temp_env::with_vars(
        [
            ("RHIZOCRYPT_PORT", None::<&str>),
            ("RHIZOCRYPT_RPC_PORT", Some("9401")),
            ("RHIZOCRYPT_JSONRPC_PORT", None),
        ],
        || {
            assert!(has_explicit_tcp_config());
        },
    );
}

#[test]
fn test_has_explicit_tcp_config_jsonrpc_port_env() {
    temp_env::with_vars(
        [
            ("RHIZOCRYPT_PORT", None::<&str>),
            ("RHIZOCRYPT_RPC_PORT", None),
            ("RHIZOCRYPT_JSONRPC_PORT", Some("9402")),
        ],
        || {
            assert!(has_explicit_tcp_config());
        },
    );
}

#[test]
fn test_run_server_btsp_env_conflict_returns_config_error() {
    temp_env::with_vars(
        [
            ("FAMILY_ID", Some("production-family")),
            ("BIOMEOS_INSECURE", Some("1")),
            ("RHIZOCRYPT_PORT", None::<&str>),
            ("RHIZOCRYPT_RPC_PORT", None),
            ("RHIZOCRYPT_JSONRPC_PORT", None),
        ],
        || {
            let rt = tokio::runtime::Builder::new_current_thread().enable_all().build().unwrap();
            let result = rt.block_on(run_server_with_ready(None, None, None, None));
            assert!(matches!(result, Err(ServiceError::Config(_))));
        },
    );
}

#[cfg(unix)]
#[test]
fn test_run_server_uds_only_mode() {
    let dir = tempfile::tempdir().expect("tempdir");
    let sock = dir.path().join("uds-only.sock");
    let sock_str = sock.to_string_lossy().to_string();

    temp_env::with_vars(
        [
            ("RHIZOCRYPT_PORT", None::<&str>),
            ("RHIZOCRYPT_RPC_PORT", None),
            ("RHIZOCRYPT_JSONRPC_PORT", None),
            ("RHIZOCRYPT_HOST", None),
            ("RHIZOCRYPT_RPC_HOST", None),
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
                let _ = run_server_with_ready(None, None, Some(sock_str), Some(ready_clone)).await;
            });
            rt.block_on(async {
                tokio::time::timeout(Duration::from_secs(10), ready.notified())
                    .await
                    .expect("UDS-only server should become ready");
                assert!(sock.exists(), "UDS socket should exist in UDS-only mode");
                handle.abort();
                let _ = handle.await;
            });
        },
    );
}

#[cfg(unix)]
#[test]
fn test_run_server_tcp_via_jsonrpc_port_env() {
    temp_env::with_vars(
        [
            ("RHIZOCRYPT_JSONRPC_PORT", Some("0")),
            ("RHIZOCRYPT_PORT", None::<&str>),
            ("RHIZOCRYPT_RPC_PORT", None),
            ("RHIZOCRYPT_HOST", Some("127.0.0.1")),
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
            let sock = dir.path().join("tcp-env.sock");
            let sock_str = sock.to_string_lossy().to_string();
            let handle = rt.spawn(async move {
                let _ = run_server_with_ready(None, None, Some(sock_str), Some(ready_clone)).await;
            });
            rt.block_on(async {
                tokio::time::timeout(Duration::from_secs(10), ready.notified())
                    .await
                    .expect("server should become ready with JSONRPC_PORT env");
                handle.abort();
                let _ = handle.await;
            });
        },
    );
}

#[test]
fn test_run_server_no_uds_backward_compat() {
    temp_env::with_vars(
        [
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
                    .expect("TCP-only server should become ready");
                handle.abort();
                let _ = handle.await;
            });
        },
    );
}

#[test]
fn test_run_server_wrapper_delegates() {
    let dir = tempfile::tempdir().expect("tempdir");
    let sock = dir.path().join("wrapper.sock");
    let sock_str = sock.to_string_lossy().to_string();

    temp_env::with_vars(
        [
            ("FAMILY_ID", None::<&str>),
            ("BIOMEOS_INSECURE", Some("1")),
            ("RHIZOCRYPT_PORT", None::<&str>),
            ("RHIZOCRYPT_RPC_PORT", None),
            ("RHIZOCRYPT_JSONRPC_PORT", None),
            ("DISCOVERY_ENDPOINT", None),
            ("RUST_LOG", Some("error")),
        ],
        || {
            let rt = tokio::runtime::Builder::new_multi_thread()
                .worker_threads(2)
                .enable_all()
                .build()
                .unwrap();
            let handle = rt.spawn(async move {
                let _ = run_server(None, None, Some(sock_str)).await;
            });
            rt.block_on(async {
                tokio::time::timeout(Duration::from_secs(5), async {
                    loop {
                        if sock.exists() {
                            break;
                        }
                        tokio::time::sleep(Duration::from_millis(20)).await;
                    }
                })
                .await
                .expect("run_server should start UDS listener");
                handle.abort();
                let _ = handle.await;
            });
        },
    );
}

#[test]
fn test_resolve_bind_addr_production_default_port() {
    use rhizo_crypt_core::constants;

    temp_env::with_vars(
        [
            ("RHIZOCRYPT_ENV", None::<&str>),
            ("RHIZOCRYPT_RPC_PORT", None),
            ("RHIZOCRYPT_PORT", None),
            ("RHIZOCRYPT_RPC_HOST", None),
            ("RHIZOCRYPT_HOST", None),
        ],
        || {
            let addr = resolve_bind_addr(None, None).unwrap();
            assert_eq!(addr.port(), constants::PRODUCTION_RPC_PORT);
        },
    );
}

#[test]
fn test_run_server_invalid_host_returns_addr_parse() {
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
            let rt = tokio::runtime::Builder::new_current_thread().enable_all().build().unwrap();
            let result = rt.block_on(run_server_with_ready(
                Some(9400),
                Some("not-a-valid-host".to_string()),
                None,
                None,
            ));
            assert!(matches!(result, Err(ServiceError::AddrParse(_))));
        },
    );
}

#[test]
fn test_run_server_host_override_enables_tcp() {
    temp_env::with_vars(
        [
            ("BIOMEOS_INSECURE", Some("1")),
            ("RHIZOCRYPT_PORT", None::<&str>),
            ("RHIZOCRYPT_RPC_PORT", None),
            ("RHIZOCRYPT_JSONRPC_PORT", None),
            ("RHIZOCRYPT_HOST", None),
            ("RHIZOCRYPT_RPC_HOST", None),
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
                let _ = run_server_with_ready(
                    None,
                    Some("127.0.0.1".to_string()),
                    None,
                    Some(ready_clone),
                )
                .await;
            });
            rt.block_on(async {
                tokio::time::timeout(Duration::from_secs(10), ready.notified())
                    .await
                    .expect("host override should enable TCP and become ready");
                handle.abort();
                let _ = handle.await;
            });
        },
    );
}

#[test]
fn test_run_server_btsp_family_id_without_insecure() {
    let dir = tempfile::tempdir().expect("tempdir");
    let sock = dir.path().join("family-id.sock");
    let sock_str = sock.to_string_lossy().to_string();

    temp_env::with_vars(
        [
            ("FAMILY_ID", Some("test-family")),
            ("BIOMEOS_INSECURE", None::<&str>),
            ("RHIZOCRYPT_PORT", None),
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
                    .expect("server with FAMILY_ID should start");
                handle.abort();
                let _ = handle.await;
            });
        },
    );
}

#[test]
fn test_run_server_transport_endpoint_env() {
    let dir = tempfile::tempdir().expect("tempdir");
    let sock = dir.path().join("transport-endpoint.sock");
    let sock_str = sock.to_string_lossy().to_string();

    temp_env::with_vars(
        [
            ("BIOMEOS_INSECURE", Some("1")),
            ("TRANSPORT_ENDPOINT", Some(r#"{"transport":"uds","path":"/tmp/test.sock"}"#)),
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
                    .expect("server with TRANSPORT_ENDPOINT should start");
                handle.abort();
                let _ = handle.await;
            });
        },
    );
}

#[cfg(unix)]
#[test]
fn test_run_server_default_uds_path() {
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
                let _ = run_server_with_ready(
                    Some(0),
                    Some("127.0.0.1".to_string()),
                    Some(String::new()),
                    Some(ready_clone),
                )
                .await;
            });
            rt.block_on(async {
                tokio::time::timeout(Duration::from_secs(10), ready.notified())
                    .await
                    .expect("default UDS path should become ready");
                handle.abort();
                let _ = handle.await;
            });
        },
    );
}

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
    let discovery_addr = rt.block_on(spawn_tarpc_discovery_server(AcceptingDiscoveryServer));
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
    let discovery_addr = spawn_tarpc_discovery_server(AcceptingDiscoveryServer).await;
    let our_addr: SocketAddr = "127.0.0.1:9400".parse().unwrap();
    let client = register_with_discovery(&discovery_addr.to_string(), our_addr)
        .await
        .expect("registration should succeed");
    assert!(client.is_connected().await);
}

#[cfg(feature = "live-clients")]
#[tokio::test]
async fn test_register_with_discovery_registration_rejected() {
    let discovery_addr = spawn_tarpc_discovery_server(RejectingDiscoveryServer).await;
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
