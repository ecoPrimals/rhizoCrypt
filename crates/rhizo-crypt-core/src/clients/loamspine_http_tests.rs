// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2024–2026 ecoPrimals Project

#![allow(clippy::unwrap_used)]

use super::*;

#[test]
fn test_client_creation() {
    let client = LoamSpineHttpClient::new("http://localhost:8080").unwrap();
    assert_eq!(client.base_url, "http://localhost:8080/rpc");

    let client2 = LoamSpineHttpClient::new("http://localhost:8080/").unwrap();
    assert_eq!(client2.base_url, "http://localhost:8080/rpc");

    let client3 = LoamSpineHttpClient::new("http://localhost:8080/rpc").unwrap();
    assert_eq!(client3.base_url, "http://localhost:8080/rpc");
}

#[test]
fn test_request_id_increment() {
    let client = LoamSpineHttpClient::new("http://localhost:8080").unwrap();
    assert_eq!(client.next_request_id(), 1);
    assert_eq!(client.next_request_id(), 2);
    assert_eq!(client.next_request_id(), 3);
}

#[test]
fn test_method_negotiation_state() {
    let client = LoamSpineHttpClient::new("http://localhost:8080").unwrap();

    assert_eq!(
        MethodSupport::from_u8(client.native_methods.load(Ordering::Relaxed)),
        MethodSupport::Unknown
    );

    assert_eq!(
        client.resolve_method("commit.session", "permanent-storage.commitSession"),
        "commit.session"
    );

    client.record_support(MethodSupport::Native);
    assert_eq!(
        client.resolve_method("commit.session", "permanent-storage.commitSession"),
        "commit.session"
    );

    client.record_support(MethodSupport::Compat);
    assert_eq!(
        client.resolve_method("commit.session", "permanent-storage.commitSession"),
        "permanent-storage.commitSession"
    );
}

#[test]
fn test_method_support_roundtrip() {
    assert_eq!(MethodSupport::from_u8(0), MethodSupport::Unknown);
    assert_eq!(MethodSupport::from_u8(1), MethodSupport::Native);
    assert_eq!(MethodSupport::from_u8(2), MethodSupport::Compat);
    assert_eq!(MethodSupport::from_u8(255), MethodSupport::Unknown);
}

#[test]
fn test_native_method_names() {
    assert_eq!(methods::native::COMMIT_SESSION, "commit.session");
    assert_eq!(methods::native::VERIFY_COMMIT, "commit.verify");
    assert_eq!(methods::native::GET_COMMIT, "commit.get");
    assert_eq!(methods::native::CHECKOUT_SLICE, "slice.checkout");
    assert_eq!(methods::native::RESOLVE_SLICE, "slice.resolve");
    assert_eq!(methods::native::HEALTH_CHECK, "system.health");
}

#[test]
fn test_compat_method_names() {
    assert_eq!(methods::compat::COMMIT_SESSION, "permanent-storage.commitSession");
    assert_eq!(methods::compat::VERIFY_COMMIT, "permanent-storage.verifyCommit");
}

#[tokio::test]
async fn test_health_check_unavailable() {
    let client = LoamSpineHttpClient::new("http://invalid-endpoint-12345:99999").unwrap();
    let result = client.health_check().await;
    assert!(result.is_err());
}

#[test]
fn test_jsonrpc_response_into_result_success() {
    let response: JsonRpcResponse<String> = JsonRpcResponse::Success {
        jsonrpc: "2.0".to_string(),
        result: "hello".to_string(),
        id: 42,
    };
    let result = response.into_result(42);
    assert_eq!(result.unwrap(), "hello");
}

#[test]
fn test_jsonrpc_response_into_result_version_mismatch() {
    let response: JsonRpcResponse<String> = JsonRpcResponse::Success {
        jsonrpc: "1.0".to_string(),
        result: "hello".to_string(),
        id: 42,
    };
    let result = response.into_result(42);
    assert!(result.is_err());
    match result.unwrap_err() {
        NegotiableError::Other(e) => {
            assert!(e.to_string().contains("Invalid JSON-RPC version"));
        }
        NegotiableError::MethodNotFound => panic!("expected Other"),
    }
}

#[test]
fn test_jsonrpc_response_into_result_id_mismatch() {
    let response: JsonRpcResponse<String> = JsonRpcResponse::Success {
        jsonrpc: "2.0".to_string(),
        result: "hello".to_string(),
        id: 99,
    };
    let result = response.into_result(42);
    assert!(result.is_err());
    match result.unwrap_err() {
        NegotiableError::Other(e) => {
            assert!(e.to_string().contains("ID mismatch"));
        }
        NegotiableError::MethodNotFound => panic!("expected Other"),
    }
}

#[test]
fn test_jsonrpc_response_into_result_method_not_found() {
    let response: JsonRpcResponse<String> = JsonRpcResponse::Error {
        jsonrpc: "2.0".to_string(),
        error: JsonRpcError {
            code: METHOD_NOT_FOUND_CODE,
            message: "Method not found".to_string(),
            data: None,
        },
        id: 1,
    };
    let result = response.into_result(1);
    assert!(matches!(result.unwrap_err(), NegotiableError::MethodNotFound));
}

#[test]
fn test_jsonrpc_response_into_result_other_error() {
    let response: JsonRpcResponse<String> = JsonRpcResponse::Error {
        jsonrpc: "2.0".to_string(),
        error: JsonRpcError {
            code: -32000,
            message: "Server error".to_string(),
            data: Some(serde_json::json!("details")),
        },
        id: 5,
    };
    let result = response.into_result(5);
    match result.unwrap_err() {
        NegotiableError::Other(e) => {
            let msg = e.to_string();
            assert!(msg.contains("Server error"));
            assert!(msg.contains("details"));
        }
        NegotiableError::MethodNotFound => panic!("expected Other"),
    }
}

#[test]
fn test_jsonrpc_response_error_no_data() {
    let response: JsonRpcResponse<String> = JsonRpcResponse::Error {
        jsonrpc: "2.0".to_string(),
        error: JsonRpcError {
            code: -32000,
            message: "Server error".to_string(),
            data: None,
        },
        id: 5,
    };
    let result = response.into_result(5);
    match result.unwrap_err() {
        NegotiableError::Other(e) => {
            let msg = e.to_string();
            assert!(msg.contains("Server error"));
            assert!(!msg.contains("data:"));
        }
        NegotiableError::MethodNotFound => panic!("expected Other"),
    }
}

#[test]
fn test_health_check_response_is_healthy() {
    let healthy_ok = HealthCheckResponse {
        status: "ok".to_string(),
        version: "1.0".to_string(),
        spine_count: 5,
    };
    assert!(healthy_ok.is_healthy());

    let healthy_explicit = HealthCheckResponse {
        status: "healthy".to_string(),
        version: "1.0".to_string(),
        spine_count: 0,
    };
    assert!(healthy_explicit.is_healthy());

    let unhealthy = HealthCheckResponse {
        status: "degraded".to_string(),
        version: "1.0".to_string(),
        spine_count: 0,
    };
    assert!(!unhealthy.is_healthy());
}

#[test]
fn test_negotiable_error_into_rhizo_error() {
    let method_err = NegotiableError::MethodNotFound.into_rhizo_error();
    assert!(method_err.to_string().contains("not found"));

    let other_err = NegotiableError::Other(RhizoCryptError::integration("test")).into_rhizo_error();
    assert!(other_err.to_string().contains("test"));
}

#[test]
fn test_validate_protocol() {
    assert!(JsonRpcResponse::<()>::validate_protocol("2.0", 1, 1).is_ok());
    assert!(JsonRpcResponse::<()>::validate_protocol("1.0", 1, 1).is_err());
    assert!(JsonRpcResponse::<()>::validate_protocol("2.0", 1, 2).is_err());
}

#[test]
fn test_jsonrpc_request_serde() {
    let req = JsonRpcRequest {
        jsonrpc: "2.0",
        method: "test.method",
        params: serde_json::json!({"key": "value"}),
        id: 1,
    };
    let json = serde_json::to_string(&req).unwrap();
    assert!(json.contains("\"jsonrpc\":\"2.0\""));
    assert!(json.contains("\"method\":\"test.method\""));
    assert!(json.contains("\"id\":1"));
}

#[test]
fn test_commit_session_request_serde() {
    let req = CommitSessionRequest {
        session_id: "abc".to_string(),
        merkle_root: "def".to_string(),
        summary: RpcDehydrationSummary {
            session_type: "General".to_string(),
            vertex_count: 10,
            leaf_count: 5,
            started_at: 1000,
            ended_at: 2000,
            outcome: "complete".to_string(),
        },
        committer_did: Some("did:key:test".to_string()),
    };
    let json = serde_json::to_value(&req).unwrap();
    assert_eq!(json["session_id"], "abc");
    assert_eq!(json["summary"]["vertex_count"], 10);
}

#[test]
fn test_checkout_slice_request_serde() {
    let req = CheckoutSliceRequest {
        spine_id: "spine1".to_string(),
        entry_hash: "aabb".to_string(),
        holder_did: "did:key:holder".to_string(),
    };
    let json = serde_json::to_value(&req).unwrap();
    assert_eq!(json["spine_id"], "spine1");
    assert_eq!(json["holder_did"], "did:key:holder");
}

#[test]
fn test_commit_session_response_deserialize() {
    let json = serde_json::json!({
        "accepted": true,
        "commit_id": "cid-123",
        "spine_entry_hash": "hash-456",
        "entry_index": 7,
        "error": null
    });
    let resp: CommitSessionResponse = serde_json::from_value(json).unwrap();
    assert!(resp.accepted);
    assert_eq!(resp.commit_id.unwrap(), "cid-123");
    assert_eq!(resp.entry_index.unwrap(), 7);
}

#[test]
fn test_checkout_slice_response_deserialize() {
    let json = serde_json::json!({
        "spine_id": "s1",
        "entry_index": 3,
        "certificate_id": "cert-1",
        "owner_did": "did:key:owner"
    });
    let resp: CheckoutSliceResponse = serde_json::from_value(json).unwrap();
    assert_eq!(resp.spine_id, "s1");
    assert_eq!(resp.entry_index, 3);
    assert_eq!(resp.owner_did, "did:key:owner");
}

// ============================================================================
// Wiremock-based HTTP integration tests (require http-clients feature)
// ============================================================================

#[cfg(feature = "http-clients")]
#[tokio::test]
async fn test_wiremock_health_check_success() {
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

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
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

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
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

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
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

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
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

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
    use wiremock::matchers::{body_string_contains, method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

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
    use crate::dehydration::DehydrationSummaryBuilder;
    use crate::merkle::MerkleRoot;
    use crate::types::{SessionId, Timestamp};
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

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
    use crate::dehydration::DehydrationSummaryBuilder;
    use crate::merkle::MerkleRoot;
    use crate::types::{SessionId, Timestamp};
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

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
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

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
    use wiremock::matchers::{body_string_contains, method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

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
    use crate::dehydration::DehydrationSummaryBuilder;
    use crate::merkle::MerkleRoot;
    use crate::types::{SessionId, Timestamp};
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

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
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

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
    use crate::types::Did;
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

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
    use crate::slice::{ResolutionOutcome, SliceBuilder, SliceMode, SliceOrigin};
    use crate::types::{Did, SessionId, VertexId};
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

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
    use crate::slice::{ResolutionOutcome, SliceBuilder, SliceMode, SliceOrigin};
    use crate::types::{Did, SessionId, VertexId};
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

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
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

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
