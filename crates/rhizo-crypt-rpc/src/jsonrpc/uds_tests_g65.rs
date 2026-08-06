// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2024–2026 ecoPrimals Project

//! G65 Protocol Negotiation tests.
//!
//! Verifies that the UDS listener correctly handles protocol negotiation:
//! - `PROTOCOLS: tarpc,jsonrpc` → tarpc binary framing on the same stream
//! - `PROTOCOLS: jsonrpc` → JSON-RPC newline-delimited
//! - No negotiation → backward-compatible JSON-RPC

#![expect(clippy::unwrap_used, reason = "test code")]

use super::tests_support::{read_json_line_raw, test_primal};
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};

/// Spin up a UDS listener and return (socket_path, shutdown_sender).
async fn start_uds_server(
    dir: &tempfile::TempDir,
) -> (std::path::PathBuf, tokio::sync::watch::Sender<bool>) {
    let sock = dir.path().join("g65_test.sock");
    let primal = test_primal().await;
    let server = super::UdsJsonRpcServer::new(primal, sock.clone());
    let (shutdown_tx, shutdown_rx) = tokio::sync::watch::channel(false);
    let ready = Arc::new(tokio::sync::Notify::new());
    let ready_clone = Arc::clone(&ready);
    tokio::spawn(async move {
        server.serve_with_ready(shutdown_rx, ready_clone).await.unwrap();
    });
    ready.notified().await;
    (sock, shutdown_tx)
}

#[test]
fn test_g65_negotiate_tarpc_e2e() {
    temp_env::with_vars(
        [
            ("RHIZOCRYPT_FAMILY_ID", None::<&str>),
            ("FAMILY_ID", None::<&str>),
            ("BEARDOG_UDS_REQUIRE_BTSP", None::<&str>),
            ("BTSP_STRICT_MODE", None::<&str>),
            ("BTSP_FAMILY_SEED", None::<&str>),
            ("BIOMEOS_INSECURE", None::<&str>),
        ],
        || {
            let rt = tokio::runtime::Builder::new_multi_thread().enable_all().build().unwrap();
            rt.block_on(async {
                let dir = tempfile::tempdir().unwrap();
                let (sock, shutdown_tx) = start_uds_server(&dir).await;

                let mut stream = tokio::net::UnixStream::connect(&sock).await.unwrap();

                stream.write_all(b"PROTOCOLS: tarpc,jsonrpc\n").await.unwrap();
                stream.flush().await.unwrap();

                let mut reader = BufReader::new(&mut stream);
                let mut response = String::new();
                reader.read_line(&mut response).await.unwrap();
                assert_eq!(response.trim(), "PROTOCOL: tarpc");

                let length_delimited =
                    tokio_util::codec::length_delimited::Builder::new().new_framed(stream);
                let transport = tokio_serde::Framed::new(
                    length_delimited,
                    tarpc::tokio_serde::formats::Bincode::default(),
                );
                let client = crate::service::RhizoCryptRpcClient::new(
                    tarpc::client::Config::default(),
                    transport,
                )
                .spawn();

                let health = client.health(tarpc::context::current()).await.unwrap().unwrap();
                assert!(health.healthy);

                let _ = shutdown_tx.send(true);
            });
        },
    );
}

#[test]
fn test_g65_negotiate_jsonrpc_e2e() {
    temp_env::with_vars(
        [
            ("RHIZOCRYPT_FAMILY_ID", None::<&str>),
            ("FAMILY_ID", None::<&str>),
            ("BEARDOG_UDS_REQUIRE_BTSP", None::<&str>),
            ("BTSP_STRICT_MODE", None::<&str>),
            ("BTSP_FAMILY_SEED", None::<&str>),
            ("BIOMEOS_INSECURE", None::<&str>),
        ],
        || {
            let rt = tokio::runtime::Builder::new_multi_thread().enable_all().build().unwrap();
            rt.block_on(async {
                let dir = tempfile::tempdir().unwrap();
                let (sock, shutdown_tx) = start_uds_server(&dir).await;

                let mut stream = tokio::net::UnixStream::connect(&sock).await.unwrap();

                stream.write_all(b"PROTOCOLS: jsonrpc\n").await.unwrap();
                stream.flush().await.unwrap();

                let mut reader = BufReader::new(&mut stream);
                let mut response = String::new();
                reader.read_line(&mut response).await.unwrap();
                assert_eq!(response.trim(), "PROTOCOL: jsonrpc");

                let request = r#"{"jsonrpc":"2.0","method":"health.check","id":1}"#;
                stream.write_all(format!("{request}\n").as_bytes()).await.unwrap();
                stream.flush().await.unwrap();

                let mut buf = Vec::new();
                let resp = read_json_line_raw(&mut stream, &mut buf).await;
                assert_eq!(resp["result"]["healthy"], true);

                let _ = shutdown_tx.send(true);
            });
        },
    );
}

#[test]
fn test_g65_no_negotiation_backward_compat() {
    temp_env::with_vars(
        [
            ("RHIZOCRYPT_FAMILY_ID", None::<&str>),
            ("FAMILY_ID", None::<&str>),
            ("BEARDOG_UDS_REQUIRE_BTSP", None::<&str>),
            ("BTSP_STRICT_MODE", None::<&str>),
            ("BTSP_FAMILY_SEED", None::<&str>),
            ("BIOMEOS_INSECURE", None::<&str>),
        ],
        || {
            let rt = tokio::runtime::Builder::new_multi_thread().enable_all().build().unwrap();
            rt.block_on(async {
                let dir = tempfile::tempdir().unwrap();
                let (sock, shutdown_tx) = start_uds_server(&dir).await;

                let mut stream = tokio::net::UnixStream::connect(&sock).await.unwrap();

                let request = r#"{"jsonrpc":"2.0","method":"health.check","id":1}"#;
                stream.write_all(format!("{request}\n").as_bytes()).await.unwrap();
                stream.flush().await.unwrap();

                let mut buf = Vec::new();
                let resp = read_json_line_raw(&mut stream, &mut buf).await;
                assert_eq!(resp["result"]["healthy"], true);

                let _ = shutdown_tx.send(true);
            });
        },
    );
}

#[test]
fn test_g65_negotiate_tarpc_then_session_ops() {
    temp_env::with_vars(
        [
            ("RHIZOCRYPT_FAMILY_ID", None::<&str>),
            ("FAMILY_ID", None::<&str>),
            ("BEARDOG_UDS_REQUIRE_BTSP", None::<&str>),
            ("BTSP_STRICT_MODE", None::<&str>),
            ("BTSP_FAMILY_SEED", None::<&str>),
            ("BIOMEOS_INSECURE", None::<&str>),
        ],
        || {
            let rt = tokio::runtime::Builder::new_multi_thread().enable_all().build().unwrap();
            rt.block_on(async {
                use crate::service_types::CreateSessionRequest;
                use rhizo_crypt_core::{EventType, SessionType};

                let dir = tempfile::tempdir().unwrap();
                let (sock, shutdown_tx) = start_uds_server(&dir).await;

                let mut stream = tokio::net::UnixStream::connect(&sock).await.unwrap();

                stream.write_all(b"PROTOCOLS: tarpc,jsonrpc\n").await.unwrap();
                stream.flush().await.unwrap();

                let mut reader = BufReader::new(&mut stream);
                let mut response = String::new();
                reader.read_line(&mut response).await.unwrap();
                assert_eq!(response.trim(), "PROTOCOL: tarpc");

                let length_delimited =
                    tokio_util::codec::length_delimited::Builder::new().new_framed(stream);
                let transport = tokio_serde::Framed::new(
                    length_delimited,
                    tarpc::tokio_serde::formats::Bincode::default(),
                );
                let client = crate::service::RhizoCryptRpcClient::new(
                    tarpc::client::Config::default(),
                    transport,
                )
                .spawn();

                let session_id = client
                    .create_session(
                        tarpc::context::current(),
                        CreateSessionRequest {
                            session_type: SessionType::General,
                            description: None,
                            parent_session: None,
                            max_vertices: None,
                            ttl_seconds: None,
                        },
                    )
                    .await
                    .unwrap()
                    .unwrap();

                let _event_id = client
                    .append_event(
                        tarpc::context::current(),
                        crate::service_types::AppendEventRequest {
                            session_id,
                            event_type: EventType::SessionStart,
                            agent: None,
                            parents: vec![],
                            metadata: vec![],
                            payload_ref: None,
                        },
                    )
                    .await
                    .unwrap()
                    .unwrap();

                let sessions =
                    client.list_sessions(tarpc::context::current()).await.unwrap().unwrap();
                assert!(!sessions.is_empty());

                let _ = shutdown_tx.send(true);
            });
        },
    );
}
