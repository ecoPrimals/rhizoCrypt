// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2024–2026 ecoPrimals Project

//! Tests for the JSON-RPC method gate (JH-0/JH-1).
//!
//! Split into focused submodules:
//! - `classification` — method classification, scope matching, enforcement, constructors
//! - `verifier_tests` — token verifiers, parsing, scope extraction, `expires_in`
//! - `gate` — bearer extraction, caller context, gate checks, auth responses, BTSP

#![expect(clippy::unwrap_used, reason = "test code")]

use super::*;
use std::sync::Arc;

// ============================================================================
// Shared helpers — used by submodules and `method_gate_tests_provider`
// ============================================================================

pub(super) fn test_gate() -> MethodGate {
    MethodGate::with_noop(EnforcementMode::Permissive)
}

pub(super) fn enforced_gate() -> MethodGate {
    MethodGate::with_noop(EnforcementMode::Enforced)
}

pub(super) fn verified_caller(token: &str) -> CallerContext {
    let verifier = NoopVerifier;
    let mut ctx = CallerContext::with_bearer_token(Some(token.to_owned()), ConnectionOrigin::Unix);
    ctx.verify_token(&verifier);
    ctx
}

pub(super) async fn unused_tcp_port() -> u16 {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    listener.local_addr().unwrap().port()
}

pub(super) async fn spawn_verify_ionic_mock_server(
    result: serde_json::Value,
    max_requests: u32,
) -> (std::net::SocketAddr, Arc<std::sync::atomic::AtomicU32>) {
    use std::sync::atomic::{AtomicU32, Ordering};
    use tokio::io::{AsyncBufReadExt, AsyncWriteExt};

    let hits = Arc::new(AtomicU32::new(0));
    let hits_bg = Arc::clone(&hits);
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();

    tokio::spawn(async move {
        for _ in 0..max_requests {
            let Ok((stream, _)) = listener.accept().await else {
                break;
            };
            hits_bg.fetch_add(1, Ordering::SeqCst);
            let (reader, mut writer) = tokio::io::split(stream);
            let mut buf_reader = tokio::io::BufReader::new(reader);
            let mut line = String::new();
            let _ = buf_reader.read_line(&mut line).await;
            let response = serde_json::json!({
                "jsonrpc": "2.0",
                "id": 1,
                "result": result,
            });
            let payload = format!("{response}\n");
            let _ = writer.write_all(payload.as_bytes()).await;
            let _ = writer.flush().await;
        }
    });

    (addr, hits)
}

pub(super) async fn spawn_verify_ionic_raw_response_server(
    response_line: &str,
) -> std::net::SocketAddr {
    use tokio::io::{AsyncBufReadExt, AsyncWriteExt};

    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let response = format!("{response_line}\n");

    tokio::spawn(async move {
        if let Ok((stream, _)) = listener.accept().await {
            let (reader, mut writer) = tokio::io::split(stream);
            let mut buf_reader = tokio::io::BufReader::new(reader);
            let mut line = String::new();
            let _ = buf_reader.read_line(&mut line).await;
            let _ = writer.write_all(response.as_bytes()).await;
            let _ = writer.flush().await;
        }
    });

    addr
}

pub(super) async fn register_signing_provider(
    registry: &DiscoveryRegistry,
    addr: std::net::SocketAddr,
) {
    use rhizo_crypt_core::discovery::{Capability, ServiceEndpoint};
    use rhizo_crypt_core::transport::TransportEndpoint;

    registry
        .register_endpoint(ServiceEndpoint::new(
            "mock-signer",
            TransportEndpoint::tcp("127.0.0.1", addr.port()),
            vec![Capability::Signing],
        ))
        .await;
}

// ============================================================================
// Submodules
// ============================================================================

#[path = "method_gate_tests_classification.rs"]
mod classification;

#[path = "method_gate_tests_verifier.rs"]
mod verifier_tests;

#[path = "method_gate_tests_gate.rs"]
mod gate;
