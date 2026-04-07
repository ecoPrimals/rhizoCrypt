// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2024–2026 ecoPrimals Project

#![expect(clippy::unwrap_used, reason = "test code")]

use super::*;

#[test]
fn test_client_new() {
    let client = BearDogHttpClient::new("http://localhost:8080", 5000).unwrap();
    assert_eq!(client.base_url, "http://localhost:8080");
}

#[test]
fn test_sign_request_serde() {
    let req = HttpSignRequest {
        data: "aGVsbG8=".to_string(),
        key_type: "ed25519".to_string(),
    };
    let json = serde_json::to_value(&req).unwrap();
    assert_eq!(json["data"], "aGVsbG8=");
    assert_eq!(json["key_type"], "ed25519");
}

#[test]
fn test_sign_request_default_key_type() {
    let json = r#"{"data":"aGVsbG8="}"#;
    let req: HttpSignRequest = serde_json::from_str(json).unwrap();
    assert_eq!(req.key_type, "ed25519");
}

#[test]
fn test_sign_response_serde() {
    let json = r#"{"signature":"c2lnbmF0dXJl"}"#;
    let resp: HttpSignResponse = serde_json::from_str(json).unwrap();
    assert_eq!(resp.signature, "c2lnbmF0dXJl");
    assert!(resp.success);
}

#[test]
fn test_sign_response_explicit_failure() {
    let json = r#"{"signature":"","success":false}"#;
    let resp: HttpSignResponse = serde_json::from_str(json).unwrap();
    assert!(!resp.success);
}

#[test]
fn test_verify_request_serde() {
    let req = HttpVerifyRequest {
        data: "ZGF0YQ==".to_string(),
        signature: "c2ln".to_string(),
    };
    let json = serde_json::to_value(&req).unwrap();
    assert_eq!(json["data"], "ZGF0YQ==");
    assert_eq!(json["signature"], "c2ln");
}

#[test]
fn test_verify_response_serde() {
    let json = r#"{"valid":true}"#;
    let resp: HttpVerifyResponse = serde_json::from_str(json).unwrap();
    assert!(resp.valid);

    let json = r#"{"valid":false}"#;
    let resp: HttpVerifyResponse = serde_json::from_str(json).unwrap();
    assert!(!resp.valid);
}

#[test]
fn test_resolve_did_request_serde() {
    let req = HttpResolveDidRequest {
        did: "did:key:test".to_string(),
    };
    let json = serde_json::to_value(&req).unwrap();
    assert_eq!(json["did"], "did:key:test");
}

#[test]
fn test_did_document_serde() {
    let json = serde_json::json!({
        "id": "did:key:abc",
        "verification_method": [{
            "id": "did:key:abc#key-1",
            "type": "Ed25519VerificationKey2020",
            "controller": "did:key:abc",
            "publicKeyMultibase": "z6Mkf5rGM..."
        }],
        "authentication": ["did:key:abc#key-1"],
        "controller": "did:key:abc"
    });
    let doc: HttpDidDocument = serde_json::from_value(json).unwrap();
    assert_eq!(doc.id, "did:key:abc");
    assert_eq!(doc.verification_method.len(), 1);
    assert_eq!(doc.authentication.len(), 1);
    assert_eq!(doc.controller.as_deref(), Some("did:key:abc"));
}

#[test]
fn test_did_document_minimal() {
    let json = r#"{"id":"did:key:abc"}"#;
    let doc: HttpDidDocument = serde_json::from_str(json).unwrap();
    assert!(doc.verification_method.is_empty());
    assert!(doc.authentication.is_empty());
    assert!(doc.controller.is_none());
}

#[test]
fn test_health_response_serde() {
    let json = r#"{"status":"healthy","version":"0.1.0"}"#;
    let resp: HttpHealthResponse = serde_json::from_str(json).unwrap();
    assert_eq!(resp.status, "healthy");
    assert_eq!(resp.version, "0.1.0");
}

#[test]
fn test_health_response_default_version() {
    let json = r#"{"status":"ok"}"#;
    let resp: HttpHealthResponse = serde_json::from_str(json).unwrap();
    assert_eq!(resp.version, "");
}

#[test]
fn test_error_display() {
    assert_eq!(BearDogHttpError::Status(401).to_string(), "HTTP status 401");
    assert_eq!(BearDogHttpError::SigningFailed.to_string(), "Signing operation failed");
    assert_eq!(BearDogHttpError::InvalidSignature.to_string(), "Invalid signature format");
}

#[test]
fn test_error_source() {
    use std::error::Error;
    assert!(BearDogHttpError::SigningFailed.source().is_none());
    assert!(BearDogHttpError::InvalidSignature.source().is_none());
    assert!(BearDogHttpError::Status(500).source().is_none());
}

#[test]
fn test_verification_method_serde() {
    let json = serde_json::json!({
        "id": "did:key:abc#key-1",
        "type": "Ed25519VerificationKey2020",
        "controller": "did:key:abc",
        "publicKeyMultibase": null
    });
    let method: HttpVerificationMethod = serde_json::from_value(json).unwrap();
    assert_eq!(method.method_type, "Ed25519VerificationKey2020");
    assert!(method.public_key_multibase.is_none());
}

// ========================================================================
// Wiremock integration tests (require live-clients feature)
// ========================================================================

#[cfg(feature = "live-clients")]
#[tokio::test]
async fn wiremock_sign_success() {
    use base64::Engine;
    use base64::engine::general_purpose::STANDARD;
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    let mock_server = MockServer::start().await;
    let base_url = mock_server.uri();

    Mock::given(method("POST"))
        .and(path("/ai/sign"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "signature": STANDARD.encode(b"mock-signature-bytes"),
            "success": true
        })))
        .mount(&mock_server)
        .await;

    let client = BearDogHttpClient::new(base_url, 5000).unwrap();
    let data = b"hello world";
    let signature = client.sign(data).await.unwrap();
    assert_eq!(&signature[..], b"mock-signature-bytes");
}

#[cfg(feature = "live-clients")]
#[tokio::test]
async fn wiremock_verify_success() {
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    let mock_server = MockServer::start().await;
    let base_url = mock_server.uri();

    Mock::given(method("POST"))
        .and(path("/ai/verify"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "valid": true
        })))
        .mount(&mock_server)
        .await;

    let client = BearDogHttpClient::new(base_url, 5000).unwrap();
    let valid = client.verify(b"data", b"sig").await.unwrap();
    assert!(valid);
}

#[cfg(feature = "live-clients")]
#[tokio::test]
async fn wiremock_verify_invalid() {
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    let mock_server = MockServer::start().await;
    let base_url = mock_server.uri();

    Mock::given(method("POST"))
        .and(path("/ai/verify"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "valid": false
        })))
        .mount(&mock_server)
        .await;

    let client = BearDogHttpClient::new(base_url, 5000).unwrap();
    let valid = client.verify(b"data", b"sig").await.unwrap();
    assert!(!valid);
}

#[cfg(feature = "live-clients")]
#[tokio::test]
async fn wiremock_health_success() {
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    let mock_server = MockServer::start().await;
    let base_url = mock_server.uri();

    Mock::given(method("GET"))
        .and(path("/ai/health"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "status": "healthy",
            "version": "0.2.0"
        })))
        .mount(&mock_server)
        .await;

    let client = BearDogHttpClient::new(base_url, 5000).unwrap();
    let health = client.health().await.unwrap();
    assert_eq!(health.status, "healthy");
    assert_eq!(health.version, "0.2.0");
}

#[cfg(feature = "live-clients")]
#[tokio::test]
async fn wiremock_sign_status_error() {
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    let mock_server = MockServer::start().await;
    let base_url = mock_server.uri();

    Mock::given(method("POST"))
        .and(path("/ai/sign"))
        .respond_with(ResponseTemplate::new(500))
        .mount(&mock_server)
        .await;

    let client = BearDogHttpClient::new(base_url, 5000).unwrap();
    let err = client.sign(b"data").await.unwrap_err();
    assert!(matches!(err, BearDogHttpError::Status(500)));
}

#[cfg(feature = "live-clients")]
#[tokio::test]
async fn wiremock_sign_failure_response() {
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    let mock_server = MockServer::start().await;
    let base_url = mock_server.uri();

    Mock::given(method("POST"))
        .and(path("/ai/sign"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "signature": "",
            "success": false
        })))
        .mount(&mock_server)
        .await;

    let client = BearDogHttpClient::new(base_url, 5000).unwrap();
    let err = client.sign(b"data").await.unwrap_err();
    assert!(matches!(err, BearDogHttpError::SigningFailed));
}

#[cfg(feature = "live-clients")]
#[tokio::test]
async fn wiremock_sign_invalid_signature_base64() {
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    let mock_server = MockServer::start().await;
    let base_url = mock_server.uri();

    Mock::given(method("POST"))
        .and(path("/ai/sign"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "signature": "!!!invalid-base64!!!",
            "success": true
        })))
        .mount(&mock_server)
        .await;

    let client = BearDogHttpClient::new(base_url, 5000).unwrap();
    let err = client.sign(b"data").await.unwrap_err();
    assert!(matches!(err, BearDogHttpError::InvalidSignature));
}

#[cfg(feature = "live-clients")]
#[tokio::test]
async fn wiremock_health_status_error() {
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    let mock_server = MockServer::start().await;
    let base_url = mock_server.uri();

    Mock::given(method("GET"))
        .and(path("/ai/health"))
        .respond_with(ResponseTemplate::new(503))
        .mount(&mock_server)
        .await;

    let client = BearDogHttpClient::new(base_url, 5000).unwrap();
    let err = client.health().await.unwrap_err();
    assert!(matches!(err, BearDogHttpError::Status(503)));
}
