// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2024–2026 ecoPrimals Project

//! Tests for the permanent storage capability client.

#![expect(clippy::unwrap_used, clippy::expect_used, reason = "test code")]

use super::*;
use crate::dehydration::DehydrationSummary;
use crate::discovery::{DiscoveryRegistry, ServiceEndpoint};
use crate::event::SessionOutcome;
use crate::merkle::MerkleRoot;
use crate::slice::{ResolutionOutcome, SliceOrigin};
use crate::transport::TransportEndpoint;
use crate::types::{ContentHash, Did, SessionId, Timestamp, VertexId};
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;

async fn discover_permanent_client(host: &str, port: u16, service: &str) -> PermanentStorageClient {
    let registry = DiscoveryRegistry::new("test-primal");
    registry
        .register_endpoint(ServiceEndpoint::new(
            service.to_owned(),
            TransportEndpoint::tcp(host, port),
            vec![Capability::PermanentCommit],
        ))
        .await;
    PermanentStorageClient::discover(&registry).await.unwrap()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct CommitRequest {
    summary: DehydrationSummary,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct VerifyCommitRequest {
    commit_ref: CommitRef,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct GetCommitRequest {
    commit_ref: CommitRef,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct CheckoutSliceRequest {
    spine_id: String,
    entry_hash: [u8; 32],
    holder: Did,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ResolveSliceRequest {
    slice: Slice,
    outcome: ResolutionOutcome,
}

fn make_commit_ref() -> CommitRef {
    CommitRef {
        spine_id: "test-spine".to_string(),
        entry_hash: [1u8; 32],
        index: 42,
    }
}

fn make_dehydration_summary() -> DehydrationSummary {
    DehydrationSummary {
        session_id: SessionId::new(uuid::Uuid::now_v7()),
        session_type: "test".to_string(),
        created_at: Timestamp::now(),
        resolved_at: Timestamp::now(),
        outcome: SessionOutcome::Success,
        merkle_root: MerkleRoot::new(ContentHash::from([0u8; 32])),
        vertex_count: 5,
        payload_bytes: 100,
        results: vec![],
        agents: vec![],
        attestations: vec![],
    }
}

fn make_slice_origin() -> SliceOrigin {
    SliceOrigin {
        spine_id: "test-spine".to_string(),
        entry_hash: ContentHash::from([2u8; 32]),
        entry_index: 10,
        certificate_id: None,
        owner: Did::new("did:key:owner"),
    }
}

#[test]
fn test_commit_request_serialization() {
    let summary = make_dehydration_summary();
    let request = CommitRequest {
        summary: summary.clone(),
    };
    let serialized = serde_json::to_string(&request).unwrap();
    let deserialized: CommitRequest = serde_json::from_str(&serialized).unwrap();
    assert_eq!(deserialized.summary.session_type, summary.session_type);
}

#[test]
fn test_commit_response_serialization() {
    let commit_ref = make_commit_ref();
    let response = CommitResponse {
        commit_ref: commit_ref.clone(),
    };
    let serialized = serde_json::to_string(&response).unwrap();
    let deserialized: CommitResponse = serde_json::from_str(&serialized).unwrap();
    assert_eq!(deserialized.commit_ref.spine_id, commit_ref.spine_id);
    assert_eq!(deserialized.commit_ref.index, 42);
}

#[test]
fn test_verify_commit_request_serialization() {
    let commit_ref = make_commit_ref();
    let request = VerifyCommitRequest {
        commit_ref: commit_ref.clone(),
    };
    let serialized = serde_json::to_string(&request).unwrap();
    let deserialized: VerifyCommitRequest = serde_json::from_str(&serialized).unwrap();
    assert_eq!(deserialized.commit_ref.spine_id, commit_ref.spine_id);
}

#[test]
fn test_verify_commit_response_serialization() {
    let response = VerifyCommitResponse {
        valid: true,
    };
    let serialized = serde_json::to_string(&response).unwrap();
    let deserialized: VerifyCommitResponse = serde_json::from_str(&serialized).unwrap();
    assert!(deserialized.valid);

    let response_false = VerifyCommitResponse {
        valid: false,
    };
    let serialized = serde_json::to_string(&response_false).unwrap();
    let deserialized: VerifyCommitResponse = serde_json::from_str(&serialized).unwrap();
    assert!(!deserialized.valid);
}

#[test]
fn test_get_commit_request_serialization() {
    let commit_ref = make_commit_ref();
    let request = GetCommitRequest {
        commit_ref: commit_ref.clone(),
    };
    let serialized = serde_json::to_string(&request).unwrap();
    let deserialized: GetCommitRequest = serde_json::from_str(&serialized).unwrap();
    assert_eq!(deserialized.commit_ref.spine_id, commit_ref.spine_id);
}

#[test]
fn test_get_commit_response_serialization() {
    let response = GetCommitResponse {
        summary: Some(make_dehydration_summary()),
    };
    let serialized = serde_json::to_string(&response).unwrap();
    let deserialized: GetCommitResponse = serde_json::from_str(&serialized).unwrap();
    assert!(deserialized.summary.is_some());

    let response_none = GetCommitResponse {
        summary: None,
    };
    let serialized = serde_json::to_string(&response_none).unwrap();
    let deserialized: GetCommitResponse = serde_json::from_str(&serialized).unwrap();
    assert!(deserialized.summary.is_none());
}

#[test]
fn test_checkout_slice_request_serialization() {
    let request = CheckoutSliceRequest {
        spine_id: "spine-1".to_string(),
        entry_hash: [3u8; 32],
        holder: Did::new("did:key:holder"),
    };
    let serialized = serde_json::to_string(&request).unwrap();
    let deserialized: CheckoutSliceRequest = serde_json::from_str(&serialized).unwrap();
    assert_eq!(deserialized.spine_id, "spine-1");
    assert_eq!(deserialized.entry_hash, [3u8; 32]);
}

#[test]
fn test_checkout_slice_response_serialization() {
    let origin = make_slice_origin();
    let response = CheckoutSliceResponse {
        origin: origin.clone(),
    };
    let serialized = serde_json::to_string(&response).unwrap();
    let deserialized: CheckoutSliceResponse = serde_json::from_str(&serialized).unwrap();
    assert_eq!(deserialized.origin.spine_id, origin.spine_id);
}

#[test]
fn test_resolve_slice_request_serialization() {
    use crate::slice::{SliceBuilder, SliceMode};

    let origin = make_slice_origin();
    let slice = SliceBuilder::new(
        origin,
        Did::new("did:key:holder"),
        SliceMode::Copy {
            allow_recopy: false,
        },
        SessionId::new(uuid::Uuid::now_v7()),
        VertexId::from_bytes(b"checkout"),
    )
    .build();
    let outcome = ResolutionOutcome::ReturnedUnchanged;

    let request = ResolveSliceRequest {
        slice: slice.clone(),
        outcome,
    };
    let serialized = serde_json::to_string(&request).unwrap();
    let deserialized: ResolveSliceRequest = serde_json::from_str(&serialized).unwrap();
    assert_eq!(deserialized.slice.id, slice.id);
}

#[test]
fn test_resolve_slice_response_serialization() {
    let response = ResolveSliceResponse {};
    let serialized = serde_json::to_string(&response).unwrap();
    let _deserialized: ResolveSliceResponse = serde_json::from_str(&serialized).unwrap();
}

#[tokio::test]
async fn test_permanent_storage_client_discover_tcp_endpoint() {
    let client = discover_permanent_client("127.0.0.1", 9700, "test-loamspine").await;
    assert!(client.endpoint().contains("127.0.0.1:9700"));
    assert_eq!(client.service_name(), Some("test-loamspine"));
}

#[tokio::test]
async fn test_permanent_storage_client_discover() {
    let registry = Arc::new(DiscoveryRegistry::new("test-primal"));

    // Register a permanent storage provider
    let addr: SocketAddr = "127.0.0.1:9700".parse().unwrap();
    let endpoint = ServiceEndpoint::new(
        "test-loamspine",
        addr.into(),
        vec![Capability::PermanentCommit, Capability::SliceCheckout],
    );
    registry.register_endpoint(endpoint).await;

    // Discover should find the provider
    let result = PermanentStorageClient::discover(&registry).await;
    assert!(result.is_ok());

    let client = result.unwrap();
    assert!(client.endpoint().contains("127.0.0.1:9700"));
    assert_eq!(client.service_name(), Some("test-loamspine"));
}

#[tokio::test]
async fn test_permanent_storage_client_discover_no_provider() {
    let registry = Arc::new(DiscoveryRegistry::new("test-primal"));

    // No providers registered
    let result = PermanentStorageClient::discover(&registry).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_permanent_storage_client_multiple_providers() {
    let registry = Arc::new(DiscoveryRegistry::new("test-primal"));

    // Register multiple providers
    let addr1: SocketAddr = "127.0.0.1:9700".parse().unwrap();
    let endpoint1 =
        ServiceEndpoint::new("ledger-primary", addr1.into(), vec![Capability::PermanentCommit]);
    registry.register_endpoint(endpoint1).await;

    let addr2: SocketAddr = "127.0.0.1:9701".parse().unwrap();
    let endpoint2 =
        ServiceEndpoint::new("ledger-secondary", addr2.into(), vec![Capability::PermanentCommit]);
    registry.register_endpoint(endpoint2).await;

    // Should discover one of them
    let result = PermanentStorageClient::discover(&registry).await;
    assert!(result.is_ok());

    let client = result.unwrap();
    // Should connect to one of the providers
    assert!(
        client.endpoint().contains("127.0.0.1:9700")
            || client.endpoint().contains("127.0.0.1:9701")
    );
}

#[tokio::test]
async fn test_permanent_storage_client_clone() {
    let client1 = discover_permanent_client("127.0.0.1", 9700, "test-loamspine").await;
    let client2 = client1.clone();

    assert_eq!(client1.endpoint(), client2.endpoint());
}

#[tokio::test]
async fn test_permanent_storage_client_debug() {
    let client = discover_permanent_client("127.0.0.1", 9700, "test-loamspine").await;
    let debug_str = format!("{client:?}");
    assert!(debug_str.contains("PermanentStorageClient"));
}

#[tokio::test]
async fn test_permanent_storage_client_concurrent_discovery() {
    let registry = Arc::new(DiscoveryRegistry::new("test-primal"));

    // Register provider
    let addr: SocketAddr = "127.0.0.1:9700".parse().unwrap();
    let endpoint =
        ServiceEndpoint::new("ledger-provider", addr.into(), vec![Capability::PermanentCommit]);
    registry.register_endpoint(endpoint).await;

    // Discover concurrently
    let registry1 = registry.clone();
    let registry2 = registry.clone();

    let handle1 = tokio::spawn(async move { PermanentStorageClient::discover(&registry1).await });

    let handle2 = tokio::spawn(async move { PermanentStorageClient::discover(&registry2).await });

    let result1 = handle1.await.unwrap();
    let result2 = handle2.await.unwrap();

    assert!(result1.is_ok());
    assert!(result2.is_ok());
}

#[tokio::test]
async fn test_permanent_storage_client_discover_with_slice_capability() {
    let registry = Arc::new(DiscoveryRegistry::new("test-primal"));

    // Register provider with both commit and slice capabilities
    let addr: SocketAddr = "127.0.0.1:9700".parse().unwrap();
    let endpoint = ServiceEndpoint::new(
        "full-loamspine",
        addr.into(),
        vec![Capability::PermanentCommit, Capability::SliceCheckout, Capability::SliceResolution],
    );
    registry.register_endpoint(endpoint).await;

    let result = PermanentStorageClient::discover(&registry).await;
    assert!(result.is_ok());

    let client = result.unwrap();
    assert_eq!(client.service_name(), Some("full-loamspine"));
}
