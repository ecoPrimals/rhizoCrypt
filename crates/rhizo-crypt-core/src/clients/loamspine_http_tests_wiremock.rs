// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2024–2026 ecoPrimals Project

#![allow(clippy::unwrap_used)]

use super::*;

#[cfg(feature = "http-clients")]
use crate::dehydration::DehydrationSummaryBuilder;
#[cfg(feature = "http-clients")]
use crate::merkle::MerkleRoot;
#[cfg(feature = "http-clients")]
use crate::slice::{ResolutionOutcome, SliceBuilder, SliceMode, SliceOrigin};
#[cfg(feature = "http-clients")]
use crate::types::{Did, SessionId, Timestamp, VertexId};
#[cfg(feature = "http-clients")]
use wiremock::matchers::{body_string_contains, method, path};
#[cfg(feature = "http-clients")]
use wiremock::{Mock, MockServer, ResponseTemplate};

// ============================================================================
// Wiremock-based HTTP integration tests (require http-clients feature)
// ============================================================================

#[cfg(feature = "http-clients")]
#[tokio::test]
async fn test_wiremock_health_check_success() {
    let mock_server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/rpc"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "jsonrpc": "2.0",
            "result": {"status": "healthy", "version": "0.8.0", "spine_count": 42},
            "id": 1
        })))
        .mount(&mock_server)
        .await;

    let client = LoamSpineHttpClient::new(mock_server.uri()).unwrap();
    let result = client.health_check().await;
    assert!(result.is_ok());
    let hc = result.unwrap();
    assert_eq!(hc.status, "healthy");
    assert_eq!(hc.version, "0.8.0");
    assert_eq!(hc.spine_count, 42);
}

#[cfg(feature = "http-clients")]
#[tokio::test]
async fn test_wiremock_health_check_http_error() {
    let mock_server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/rpc"))
        .respond_with(ResponseTemplate::new(500).set_body_string("Internal Server Error"))
        .mount(&mock_server)
        .await;

    let client = LoamSpineHttpClient::new(mock_server.uri()).unwrap();
    let result = client.health_check().await;
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("500"));
}

#[cfg(feature = "http-clients")]
#[tokio::test]
async fn test_wiremock_health_check_parse_error() {
    let mock_server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/rpc"))
        .respond_with(ResponseTemplate::new(200).set_body_string("not valid json"))
        .mount(&mock_server)
        .await;

    let client = LoamSpineHttpClient::new(mock_server.uri()).unwrap();
    let result = client.health_check().await;
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("parse"));
}

#[cfg(feature = "http-clients")]
#[tokio::test]
async fn test_wiremock_health_check_version_mismatch() {
    let mock_server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/rpc"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "jsonrpc": "1.0",
            "result": {"status": "ok", "version": "0.8.0", "spine_count": 0},
            "id": 1
        })))
        .mount(&mock_server)
        .await;

    let client = LoamSpineHttpClient::new(mock_server.uri()).unwrap();
    let result = client.health_check().await;
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("JSON-RPC version"));
}

#[cfg(feature = "http-clients")]
#[tokio::test]
async fn test_wiremock_health_check_id_mismatch() {
    let mock_server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/rpc"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "jsonrpc": "2.0",
            "result": {"status": "ok", "version": "0.8.0", "spine_count": 0},
            "id": 99
        })))
        .mount(&mock_server)
        .await;

    let client = LoamSpineHttpClient::new(mock_server.uri()).unwrap();
    let result = client.health_check().await;
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("ID mismatch"));
}

#[cfg(feature = "http-clients")]
#[tokio::test]
async fn test_wiremock_method_negotiation_native_to_compat() {
    let mock_server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/rpc"))
        .and(body_string_contains("system.health"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "jsonrpc": "2.0",
            "error": {"code": -32601, "message": "Method not found"},
            "id": 1
        })))
        .mount(&mock_server)
        .await;

    Mock::given(method("POST"))
        .and(path("/rpc"))
        .and(body_string_contains("permanent-storage.healthCheck"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "jsonrpc": "2.0",
            "result": {"status": "healthy", "version": "0.7.0", "spine_count": 10},
            "id": 2
        })))
        .mount(&mock_server)
        .await;

    let client = LoamSpineHttpClient::new(mock_server.uri()).unwrap();
    let result = client.health_check().await;
    assert!(result.is_ok());
    let hc = result.unwrap();
    assert_eq!(hc.version, "0.7.0");
}

#[cfg(feature = "http-clients")]
#[tokio::test]
async fn test_wiremock_commit_session_success() {
    let mock_server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/rpc"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "jsonrpc": "2.0",
            "result": {
                "accepted": true,
                "commit_id": "spine-abc-123",
                "spine_entry_hash": "a1b2c3d4e5f6",
                "entry_index": 0,
                "error": null
            },
            "id": 1
        })))
        .mount(&mock_server)
        .await;

    let client = LoamSpineHttpClient::new(mock_server.uri()).unwrap();
    let summary = DehydrationSummaryBuilder::new(
        SessionId::now(),
        "test",
        Timestamp::now(),
        MerkleRoot::ZERO,
    )
    .build();

    let result = client.commit(&summary).await;
    assert!(result.is_ok());
    let commit_ref = result.unwrap();
    assert_eq!(commit_ref.spine_id, "spine-abc-123");
    assert_eq!(commit_ref.index, 0);
}

#[cfg(feature = "http-clients")]
#[tokio::test]
async fn test_wiremock_commit_session_rejected() {
    let mock_server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/rpc"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "jsonrpc": "2.0",
            "result": {
                "accepted": false,
                "commit_id": null,
                "spine_entry_hash": null,
                "entry_index": null,
                "error": "Quota exceeded"
            },
            "id": 1
        })))
        .mount(&mock_server)
        .await;

    let client = LoamSpineHttpClient::new(mock_server.uri()).unwrap();
    let summary = DehydrationSummaryBuilder::new(
        SessionId::now(),
        "test",
        Timestamp::now(),
        MerkleRoot::ZERO,
    )
    .build();

    let result = client.commit(&summary).await;
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("rejected"));
}

#[cfg(feature = "http-clients")]
#[tokio::test]
async fn test_wiremock_verify_commit_success() {
    let mock_server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/rpc"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "jsonrpc": "2.0",
            "result": true,
            "id": 1
        })))
        .mount(&mock_server)
        .await;

    let client = LoamSpineHttpClient::new(mock_server.uri()).unwrap();
    let commit_ref = CommitRef {
        spine_id: "spine-1".to_string(),
        entry_hash: [1u8; 32],
        index: 0,
    };

    let result = client.verify_commit(&commit_ref).await;
    assert!(result.is_ok());
    assert!(result.unwrap());
}

#[cfg(feature = "http-clients")]
#[tokio::test]
async fn test_wiremock_verify_commit_fallback_to_health() {
    let mock_server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/rpc"))
        .and(body_string_contains("commit.verify"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "jsonrpc": "2.0",
            "error": {"code": -32601, "message": "Method not found"},
            "id": 1
        })))
        .mount(&mock_server)
        .await;

    Mock::given(method("POST"))
        .and(path("/rpc"))
        .and(body_string_contains("permanent-storage.verifyCommit"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "jsonrpc": "2.0",
            "error": {"code": -32601, "message": "Method not found"},
            "id": 2
        })))
        .mount(&mock_server)
        .await;

    // After verify fails with both native and compat, client falls back to health_check.
    // At this point MethodSupport is Compat, so health_check uses permanent-storage.healthCheck.
    Mock::given(method("POST"))
        .and(path("/rpc"))
        .and(body_string_contains("permanent-storage.healthCheck"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "jsonrpc": "2.0",
            "result": {"status": "healthy", "version": "0.8.0", "spine_count": 5},
            "id": 3
        })))
        .mount(&mock_server)
        .await;

    let client = LoamSpineHttpClient::new(mock_server.uri()).unwrap();
    let commit_ref = CommitRef {
        spine_id: "spine-1".to_string(),
        entry_hash: [1u8; 32],
        index: 0,
    };

    let result = client.verify_commit(&commit_ref).await;
    assert!(result.is_ok());
    assert!(result.unwrap());
}

#[cfg(feature = "http-clients")]
#[tokio::test]
async fn test_wiremock_get_commit_success() {
    let mock_server = MockServer::start().await;

    let summary = DehydrationSummaryBuilder::new(
        SessionId::now(),
        "test-session",
        Timestamp::from_nanos(1000),
        MerkleRoot::ZERO,
    )
    .build();

    let result_json = serde_json::json!({
        "summary": serde_json::to_value(&summary).unwrap()
    });

    Mock::given(method("POST"))
        .and(path("/rpc"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "jsonrpc": "2.0",
            "result": result_json,
            "id": 1
        })))
        .mount(&mock_server)
        .await;

    let client = LoamSpineHttpClient::new(mock_server.uri()).unwrap();
    let commit_ref = CommitRef {
        spine_id: "spine-1".to_string(),
        entry_hash: [1u8; 32],
        index: 0,
    };

    let result = client.get_commit(&commit_ref).await;
    assert!(result.is_ok());
    let opt = result.unwrap();
    assert!(opt.is_some());
    let retrieved = opt.unwrap();
    assert_eq!(retrieved.session_type, "test-session");
}

#[cfg(feature = "http-clients")]
#[tokio::test]
async fn test_wiremock_get_commit_not_found_returns_none() {
    let mock_server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/rpc"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "jsonrpc": "2.0",
            "error": {"code": -32000, "message": "Commit not found"},
            "id": 1
        })))
        .mount(&mock_server)
        .await;

    let client = LoamSpineHttpClient::new(mock_server.uri()).unwrap();
    let commit_ref = CommitRef {
        spine_id: "spine-1".to_string(),
        entry_hash: [1u8; 32],
        index: 0,
    };

    let result = client.get_commit(&commit_ref).await;
    assert!(result.is_ok());
    assert!(result.unwrap().is_none());
}

#[cfg(feature = "http-clients")]
#[tokio::test]
async fn test_wiremock_checkout_slice_success() {
    let mock_server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/rpc"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "jsonrpc": "2.0",
            "result": {
                "spine_id": "spine-checkout",
                "entry_index": 7,
                "certificate_id": "cert-xyz",
                "owner_did": "did:key:owner"
            },
            "id": 1
        })))
        .mount(&mock_server)
        .await;

    let client = LoamSpineHttpClient::new(mock_server.uri()).unwrap();
    let spine_id = "spine-checkout";
    let entry_hash = [0u8; 32];
    let holder = Did::new("did:key:holder");

    let result = client.checkout_slice(spine_id, &entry_hash, &holder).await;
    assert!(result.is_ok());
    let origin = result.unwrap();
    assert_eq!(origin.spine_id, "spine-checkout");
    assert_eq!(origin.entry_index, 7);
    assert_eq!(origin.certificate_id.as_deref(), Some("cert-xyz"));
    assert_eq!(origin.owner.as_str(), "did:key:owner");
}

#[cfg(feature = "http-clients")]
#[tokio::test]
async fn test_wiremock_resolve_slice_success() {
    let mock_server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/rpc"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "jsonrpc": "2.0",
            "result": null,
            "id": 1
        })))
        .mount(&mock_server)
        .await;

    let client = LoamSpineHttpClient::new(mock_server.uri()).unwrap();
    let origin = SliceOrigin {
        spine_id: "spine-resolve".to_string(),
        entry_hash: [0u8; 32],
        entry_index: 0,
        certificate_id: None,
        owner: Did::new("did:key:owner"),
    };
    let slice = SliceBuilder::new(
        origin,
        Did::new("did:key:holder"),
        SliceMode::Copy {
            allow_recopy: false,
        },
        SessionId::now(),
        VertexId::from_bytes(b"checkout"),
    )
    .build();

    let outcome = ResolutionOutcome::ReturnedUnchanged;
    let result = client.resolve_slice(&slice, &outcome).await;
    assert!(result.is_ok());
}

#[cfg(feature = "http-clients")]
#[tokio::test]
async fn test_wiremock_resolve_slice_endpoint_unavailable_returns_ok() {
    let mock_server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/rpc"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "jsonrpc": "2.0",
            "error": {"code": -32601, "message": "Method not found"},
            "id": 1
        })))
        .mount(&mock_server)
        .await;

    let client = LoamSpineHttpClient::new(mock_server.uri()).unwrap();
    let origin = SliceOrigin {
        spine_id: "spine-resolve".to_string(),
        entry_hash: [0u8; 32],
        entry_index: 0,
        certificate_id: None,
        owner: Did::new("did:key:owner"),
    };
    let slice = SliceBuilder::new(
        origin,
        Did::new("did:key:holder"),
        SliceMode::Copy {
            allow_recopy: false,
        },
        SessionId::now(),
        VertexId::from_bytes(b"checkout"),
    )
    .build();

    let outcome = ResolutionOutcome::ReturnedUnchanged;
    let result = client.resolve_slice(&slice, &outcome).await;
    assert!(result.is_ok());
}

#[cfg(feature = "http-clients")]
#[tokio::test]
async fn test_wiremock_jsonrpc_other_error() {
    let mock_server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/rpc"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "jsonrpc": "2.0",
            "error": {"code": -32000, "message": "Server error", "data": "extra_details"},
            "id": 1
        })))
        .mount(&mock_server)
        .await;

    let client = LoamSpineHttpClient::new(mock_server.uri()).unwrap();
    let result = client.health_check().await;
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.to_string().contains("Server error"));
    assert!(err.to_string().contains("extra_details"));
}
