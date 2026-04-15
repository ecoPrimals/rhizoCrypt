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
