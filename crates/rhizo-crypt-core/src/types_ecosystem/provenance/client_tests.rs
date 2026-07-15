// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2024–2026 ecoPrimals Project

//! Tests for the provenance notifier client.

#![expect(clippy::unwrap_used, clippy::expect_used, reason = "test code")]

use std::net::SocketAddr;

use super::*;
use crate::MerkleRoot;
use crate::types::{Did, Timestamp, VertexId};
use crate::types_ecosystem::provenance::VertexRef;

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_notifier_creation() {
    let config = ProvenanceProviderConfig::default();
    let notifier = ProvenanceNotifier::new(config);
    assert_eq!(notifier.state().await, ClientState::Disconnected);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_notify_without_connection() {
    let notifier = ProvenanceNotifier::new(ProvenanceProviderConfig::default());
    // Should succeed silently when not connected
    let result = notifier.notify_session_commit(SessionId::now()).await;
    assert!(result.is_ok());
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_notifier_with_discovery() {
    let registry = Arc::new(DiscoveryRegistry::new("test"));
    let notifier = ProvenanceNotifier::with_discovery(registry);
    assert_eq!(notifier.state().await, ClientState::Disconnected);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_notifier_connect_with_push_address() {
    let config = ProvenanceProviderConfig::with_push_address("127.0.0.1:9900");
    let notifier = ProvenanceNotifier::new(config);

    let result = notifier.connect().await;
    assert!(result.is_ok());
    assert_eq!(notifier.state().await, ClientState::Connected);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_notifier_connect_invalid_address() {
    let config = ProvenanceProviderConfig::with_push_address("invalid-address");
    let notifier = ProvenanceNotifier::new(config);

    let result = notifier.connect().await;
    assert!(result.is_err());
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_notifier_connect_no_address() {
    let notifier = ProvenanceNotifier::new(ProvenanceProviderConfig::default());

    // Should succeed with warning (provenance provider is optional)
    let result = notifier.connect().await;
    assert!(result.is_ok());
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_notify_session_commit_connected() {
    let config = ProvenanceProviderConfig::with_push_address("127.0.0.1:9900");
    let notifier = ProvenanceNotifier::new(config);
    notifier.connect().await.unwrap();

    let result = notifier.notify_session_commit(SessionId::now()).await;
    assert!(result.is_ok());
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_notify_dehydration_without_connection() {
    use crate::dehydration::{AgentSummary, DehydrationSummaryBuilder};
    use crate::event::SessionOutcome;

    let notifier = ProvenanceNotifier::new(ProvenanceProviderConfig::default());

    let summary = DehydrationSummaryBuilder::new(
        SessionId::now(),
        "test",
        Timestamp::now(),
        MerkleRoot::new([0u8; 32]),
    )
    .with_outcome(SessionOutcome::Success)
    .with_vertex_count(5)
    .with_agent(AgentSummary {
        agent: Did::new("did:key:test"),
        joined_at: Timestamp::now(),
        left_at: None,
        event_count: 3,
        role: "author".to_string(),
    })
    .build();

    let result = notifier.notify_dehydration(&summary).await;
    assert!(result.is_ok());
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_notify_dehydration_connected_no_server() {
    use crate::dehydration::{AgentSummary, DehydrationSummaryBuilder};
    use crate::event::SessionOutcome;

    let config = ProvenanceProviderConfig::with_push_address("127.0.0.1:19901");
    let notifier = ProvenanceNotifier::new(config);
    notifier.connect().await.unwrap();

    let summary = DehydrationSummaryBuilder::new(
        SessionId::now(),
        "test",
        Timestamp::now(),
        MerkleRoot::new([0u8; 32]),
    )
    .with_outcome(SessionOutcome::Success)
    .with_vertex_count(5)
    .with_agent(AgentSummary {
        agent: Did::new("did:key:test"),
        joined_at: Timestamp::now(),
        left_at: None,
        event_count: 3,
        role: "author".to_string(),
    })
    .build();

    // Non-fatal: should succeed even when provider is unreachable
    let result = notifier.notify_dehydration(&summary).await;
    assert!(result.is_ok());
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_notify_provenance_without_connection() {
    let notifier = ProvenanceNotifier::new(ProvenanceProviderConfig::default());
    let chain = ProvenanceChain::new();

    // Should succeed silently when not connected
    let result = notifier.notify_provenance(&chain).await;
    assert!(result.is_ok());
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_notify_provenance_connected_no_server() {
    let config = ProvenanceProviderConfig::with_push_address("127.0.0.1:19904");
    let notifier = ProvenanceNotifier::new(config);
    notifier.connect().await.unwrap();

    let mut chain = ProvenanceChain::new();
    chain.add_vertex(VertexRef {
        session_id: SessionId::now(),
        vertex_id: VertexId::from_bytes(b"v1"),
        event_type: "test".to_string(),
        agent: Some(Did::new("did:key:test")),
        timestamp: Timestamp::now(),
        payload_ref: None,
    });

    // Non-fatal: should succeed even when provider is unreachable
    let result = notifier.notify_provenance(&chain).await;
    assert!(result.is_ok());
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_send_jsonrpc_provenance_with_mock_server() {
    use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};

    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();

    let server_handle = tokio::spawn(async move {
        let (stream, _) = listener.accept().await.unwrap();
        let (reader, mut writer) = stream.into_split();
        let mut buf_reader = BufReader::new(reader);
        let mut request = String::new();
        buf_reader.read_line(&mut request).await.unwrap();

        let parsed: serde_json::Value = serde_json::from_str(&request).unwrap();
        assert_eq!(parsed["method"], "contribution.record_provenance");
        assert!(parsed["params"]["vertices"].is_array());

        let response = r#"{"jsonrpc":"2.0","result":"ok","id":1}"#;
        writer.write_all(format!("{response}\n").as_bytes()).await.unwrap();
    });

    let config = ProvenanceProviderConfig::with_push_address(addr.to_string());
    let notifier = ProvenanceNotifier::new(config);
    notifier.connect().await.unwrap();

    let mut chain = ProvenanceChain::new();
    chain.add_vertex(VertexRef {
        session_id: SessionId::now(),
        vertex_id: VertexId::from_bytes(b"v1"),
        event_type: "test".to_string(),
        agent: Some(Did::new("did:key:test")),
        timestamp: Timestamp::now(),
        payload_ref: None,
    });

    let result = notifier.notify_provenance(&chain).await;
    assert!(result.is_ok());

    server_handle.await.unwrap();
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_endpoint_management() {
    let config = ProvenanceProviderConfig::with_push_address("127.0.0.1:9900");
    let notifier = ProvenanceNotifier::new(config);

    // Initially no endpoint
    assert!(notifier.endpoint().await.is_none());

    // Connect
    notifier.connect().await.unwrap();

    // Should have endpoint
    let endpoint = notifier.endpoint().await;
    assert!(endpoint.is_some());
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_connect_via_discovery_registry() {
    use crate::discovery::ServiceEndpoint;

    let registry = Arc::new(DiscoveryRegistry::new("test"));
    let addr: SocketAddr = "127.0.0.1:19902".parse().unwrap();
    registry
        .register_endpoint(ServiceEndpoint::new(
            "provenance-test",
            addr.into(),
            vec![Capability::ProvenanceQuery],
        ))
        .await;

    let notifier = ProvenanceNotifier::with_discovery(registry);
    let result = notifier.connect().await;
    assert!(result.is_ok());
    assert_eq!(notifier.state().await, ClientState::Connected);
    let expected = crate::transport::TransportEndpoint::tcp(addr.ip().to_string(), addr.port());
    assert_eq!(notifier.endpoint().await, Some(expected));
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_send_jsonrpc_success_with_mock_server() {
    use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};

    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();

    let server_handle = tokio::spawn(async move {
        let (stream, _) = listener.accept().await.unwrap();
        let (reader, mut writer) = stream.into_split();
        let mut buf_reader = BufReader::new(reader);
        let mut request = String::new();
        buf_reader.read_line(&mut request).await.unwrap();
        let response = r#"{"jsonrpc":"2.0","result":"ok","id":1}"#;
        writer.write_all(format!("{response}\n").as_bytes()).await.unwrap();
    });

    let config = ProvenanceProviderConfig::with_push_address(addr.to_string());
    let notifier = ProvenanceNotifier::new(config);
    notifier.connect().await.unwrap();

    let result = notifier.notify_session_commit(SessionId::now()).await;
    assert!(result.is_ok());

    server_handle.await.unwrap();
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_send_jsonrpc_dehydration_with_mock_server() {
    use crate::dehydration::{AgentSummary, DehydrationSummaryBuilder};
    use crate::event::SessionOutcome;
    use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};

    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();

    let server_handle = tokio::spawn(async move {
        let (stream, _) = listener.accept().await.unwrap();
        let (reader, mut writer) = stream.into_split();
        let mut buf_reader = BufReader::new(reader);
        let mut request = String::new();
        buf_reader.read_line(&mut request).await.unwrap();
        let response = r#"{"jsonrpc":"2.0","result":"ok","id":1}"#;
        writer.write_all(format!("{response}\n").as_bytes()).await.unwrap();
    });

    let config = ProvenanceProviderConfig::with_push_address(addr.to_string());
    let notifier = ProvenanceNotifier::new(config);
    notifier.connect().await.unwrap();

    let summary = DehydrationSummaryBuilder::new(
        SessionId::now(),
        "test",
        Timestamp::now(),
        MerkleRoot::new([0u8; 32]),
    )
    .with_outcome(SessionOutcome::Success)
    .with_vertex_count(5)
    .with_agent(AgentSummary {
        agent: Did::new("did:key:test"),
        joined_at: Timestamp::now(),
        left_at: None,
        event_count: 3,
        role: "author".to_string(),
    })
    .build();

    let result = notifier.notify_dehydration(&summary).await;
    assert!(result.is_ok());

    server_handle.await.unwrap();
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_send_jsonrpc_connection_refused() {
    let config = ProvenanceProviderConfig::with_push_address("127.0.0.1:19903");
    let notifier = ProvenanceNotifier::new(config);
    notifier.connect().await.unwrap();

    // Non-fatal: should succeed even when connection is refused
    let result = notifier.notify_session_commit(SessionId::now()).await;
    assert!(result.is_ok());
}
