// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2024–2026 ecoPrimals Project

use super::*;

#[test]
fn test_client_new() {
    let client = NestGateHttpClient::new("http://localhost:9200", 5000).unwrap();
    assert_eq!(client.base_url, "http://localhost:9200");
}

#[test]
fn test_store_blob_request_serde() {
    let req = HttpStoreBlobRequest {
        data: "aGVsbG8=".to_string(),
        content_type: "text/plain".to_string(),
        metadata: std::collections::HashMap::new(),
    };
    let json = serde_json::to_value(&req).unwrap();
    assert_eq!(json["data"], "aGVsbG8=");
    assert_eq!(json["content_type"], "text/plain");
}

#[test]
fn test_store_blob_request_default_content_type() {
    let json = r#"{"data":"aGVsbG8="}"#;
    let req: HttpStoreBlobRequest = serde_json::from_str(json).unwrap();
    assert_eq!(req.content_type, "application/octet-stream");
}

#[test]
fn test_store_blob_response_serde() {
    let json = r#"{"reference":"abc123","size":100}"#;
    let resp: HttpStoreBlobResponse = serde_json::from_str(json).unwrap();
    assert_eq!(resp.reference, "abc123");
    assert_eq!(resp.size, 100);
    assert!(resp.success);
}

#[test]
fn test_store_blob_response_explicit_success() {
    let json = r#"{"reference":"abc123","size":100,"success":false}"#;
    let resp: HttpStoreBlobResponse = serde_json::from_str(json).unwrap();
    assert!(!resp.success);
}

#[test]
fn test_retrieve_blob_response_serde() {
    let json = r#"{"data":"aGVsbG8=","content_type":"text/plain","size":5}"#;
    let resp: HttpRetrieveBlobResponse = serde_json::from_str(json).unwrap();
    assert_eq!(resp.data, "aGVsbG8=");
    assert_eq!(resp.content_type, "text/plain");
    assert_eq!(resp.size, 5);
}

#[test]
fn test_blob_metadata_serde() {
    let json = serde_json::json!({
        "reference": "hash123",
        "content_type": "application/octet-stream",
        "size": 1024,
        "created_at": "2024-01-01T00:00:00Z",
        "metadata": {"key": "value"}
    });
    let meta: HttpBlobMetadata = serde_json::from_value(json).unwrap();
    assert_eq!(meta.reference, "hash123");
    assert_eq!(meta.size, 1024);
    assert_eq!(meta.created_at.as_deref(), Some("2024-01-01T00:00:00Z"));
    assert_eq!(meta.metadata.get("key").unwrap(), "value");
}

#[test]
fn test_blob_metadata_minimal() {
    let json = r#"{"reference":"h","content_type":"a/b","size":0}"#;
    let meta: HttpBlobMetadata = serde_json::from_str(json).unwrap();
    assert!(meta.created_at.is_none());
    assert!(meta.metadata.is_empty());
}

#[test]
fn test_health_response_serde() {
    let json = r#"{"status":"healthy","available_bytes":1000000,"used_bytes":500000}"#;
    let resp: HttpHealthResponse = serde_json::from_str(json).unwrap();
    assert_eq!(resp.status, "healthy");
    assert_eq!(resp.available_bytes, 1_000_000);
    assert_eq!(resp.used_bytes, 500_000);
}

#[test]
fn test_health_response_defaults() {
    let json = r#"{"status":"ok"}"#;
    let resp: HttpHealthResponse = serde_json::from_str(json).unwrap();
    assert_eq!(resp.available_bytes, 0);
    assert_eq!(resp.used_bytes, 0);
}

#[test]
fn test_error_display() {
    assert_eq!(NestGateHttpError::Status(404).to_string(), "HTTP status 404");
    assert_eq!(NestGateHttpError::StoreFailed.to_string(), "Store operation failed");
    assert_eq!(NestGateHttpError::NotFound.to_string(), "Blob not found");
    assert_eq!(NestGateHttpError::InvalidData.to_string(), "Invalid data format");
}

#[test]
fn test_error_source() {
    use std::error::Error;
    assert!(NestGateHttpError::StoreFailed.source().is_none());
    assert!(NestGateHttpError::NotFound.source().is_none());
    assert!(NestGateHttpError::InvalidData.source().is_none());
    assert!(NestGateHttpError::Status(500).source().is_none());
}

#[test]
fn test_store_blob_request_roundtrip() {
    let mut metadata = std::collections::HashMap::new();
    metadata.insert("author".to_string(), "test".to_string());
    let req = HttpStoreBlobRequest {
        data: "dGVzdA==".to_string(),
        content_type: "text/plain".to_string(),
        metadata,
    };
    let json = serde_json::to_string(&req).unwrap();
    let req2: HttpStoreBlobRequest = serde_json::from_str(&json).unwrap();
    assert_eq!(req2.data, req.data);
    assert_eq!(req2.metadata.get("author").unwrap(), "test");
}

// ========================================================================
// Wiremock integration tests (require live-clients feature)
// ========================================================================

#[cfg(feature = "live-clients")]
#[tokio::test]
async fn wiremock_store_success() {
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    let mock_server = MockServer::start().await;
    let base_url = mock_server.uri();

    Mock::given(method("POST"))
        .and(path("/api/v1/blobs"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "reference": "blake3-abc123",
            "size": 11,
            "success": true
        })))
        .mount(&mock_server)
        .await;

    let client = NestGateHttpClient::new(base_url, 5000).unwrap();
    let reference = client.store(b"hello world", None).await.unwrap();
    assert_eq!(reference, "blake3-abc123");
}

#[cfg(feature = "live-clients")]
#[tokio::test]
async fn wiremock_store_with_content_type() {
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    let mock_server = MockServer::start().await;
    let base_url = mock_server.uri();

    Mock::given(method("POST"))
        .and(path("/api/v1/blobs"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "reference": "blake3-xyz",
            "size": 5,
            "success": true
        })))
        .mount(&mock_server)
        .await;

    let client = NestGateHttpClient::new(base_url, 5000).unwrap();
    let reference = client.store(b"hello", Some("text/plain")).await.unwrap();
    assert_eq!(reference, "blake3-xyz");
}

#[cfg(feature = "live-clients")]
#[tokio::test]
async fn wiremock_retrieve_success() {
    use base64::{Engine, engine::general_purpose::STANDARD};
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    let mock_server = MockServer::start().await;
    let base_url = mock_server.uri();

    Mock::given(method("GET"))
        .and(path("/api/v1/blobs/hash123"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "data": STANDARD.encode(b"retrieved data"),
            "content_type": "application/octet-stream",
            "size": 14
        })))
        .mount(&mock_server)
        .await;

    let client = NestGateHttpClient::new(base_url, 5000).unwrap();
    let data = client.retrieve("hash123").await.unwrap();
    assert_eq!(&data[..], b"retrieved data");
}

#[cfg(feature = "live-clients")]
#[tokio::test]
async fn wiremock_retrieve_not_found() {
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    let mock_server = MockServer::start().await;
    let base_url = mock_server.uri();

    Mock::given(method("GET"))
        .and(path("/api/v1/blobs/nonexistent"))
        .respond_with(ResponseTemplate::new(404))
        .mount(&mock_server)
        .await;

    let client = NestGateHttpClient::new(base_url, 5000).unwrap();
    let err = client.retrieve("nonexistent").await.unwrap_err();
    assert!(matches!(err, NestGateHttpError::NotFound));
}

#[cfg(feature = "live-clients")]
#[tokio::test]
async fn wiremock_exists_true() {
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    let mock_server = MockServer::start().await;
    let base_url = mock_server.uri();

    Mock::given(method("GET"))
        .and(path("/api/v1/blobs/exists-ref"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&mock_server)
        .await;

    let client = NestGateHttpClient::new(base_url, 5000).unwrap();
    let exists = client.exists("exists-ref").await.unwrap();
    assert!(exists);
}

#[cfg(feature = "live-clients")]
#[tokio::test]
async fn wiremock_exists_false() {
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    let mock_server = MockServer::start().await;
    let base_url = mock_server.uri();

    Mock::given(method("GET"))
        .and(path("/api/v1/blobs/missing-ref"))
        .respond_with(ResponseTemplate::new(404))
        .mount(&mock_server)
        .await;

    let client = NestGateHttpClient::new(base_url, 5000).unwrap();
    let exists = client.exists("missing-ref").await.unwrap();
    assert!(!exists);
}

#[cfg(feature = "live-clients")]
#[tokio::test]
async fn wiremock_metadata_success() {
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    let mock_server = MockServer::start().await;
    let base_url = mock_server.uri();

    Mock::given(method("GET"))
        .and(path("/api/v1/blobs/ref456/metadata"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "reference": "ref456",
            "content_type": "application/octet-stream",
            "size": 1024,
            "created_at": "2024-01-15T12:00:00Z",
            "metadata": {"author": "test"}
        })))
        .mount(&mock_server)
        .await;

    let client = NestGateHttpClient::new(base_url, 5000).unwrap();
    let meta = client.metadata("ref456").await.unwrap();
    assert_eq!(meta.reference, "ref456");
    assert_eq!(meta.size, 1024);
    assert_eq!(meta.created_at.as_deref(), Some("2024-01-15T12:00:00Z"));
    assert_eq!(meta.metadata.get("author").unwrap(), "test");
}

#[cfg(feature = "live-clients")]
#[tokio::test]
async fn wiremock_metadata_not_found() {
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    let mock_server = MockServer::start().await;
    let base_url = mock_server.uri();

    Mock::given(method("GET"))
        .and(path("/api/v1/blobs/missing/metadata"))
        .respond_with(ResponseTemplate::new(404))
        .mount(&mock_server)
        .await;

    let client = NestGateHttpClient::new(base_url, 5000).unwrap();
    let err = client.metadata("missing").await.unwrap_err();
    assert!(matches!(err, NestGateHttpError::NotFound));
}

#[cfg(feature = "live-clients")]
#[tokio::test]
async fn wiremock_health_success() {
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    let mock_server = MockServer::start().await;
    let base_url = mock_server.uri();

    Mock::given(method("GET"))
        .and(path("/api/v1/health"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "status": "healthy",
            "available_bytes": 1_000_000_000,
            "used_bytes": 500_000_000
        })))
        .mount(&mock_server)
        .await;

    let client = NestGateHttpClient::new(base_url, 5000).unwrap();
    let health = client.health().await.unwrap();
    assert_eq!(health.status, "healthy");
    assert_eq!(health.available_bytes, 1_000_000_000);
    assert_eq!(health.used_bytes, 500_000_000);
}

#[cfg(feature = "live-clients")]
#[tokio::test]
async fn wiremock_store_status_error() {
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    let mock_server = MockServer::start().await;
    let base_url = mock_server.uri();

    Mock::given(method("POST"))
        .and(path("/api/v1/blobs"))
        .respond_with(ResponseTemplate::new(507))
        .mount(&mock_server)
        .await;

    let client = NestGateHttpClient::new(base_url, 5000).unwrap();
    let err = client.store(b"data", None).await.unwrap_err();
    assert!(matches!(err, NestGateHttpError::Status(507)));
}

#[cfg(feature = "live-clients")]
#[tokio::test]
async fn wiremock_store_failure_response() {
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    let mock_server = MockServer::start().await;
    let base_url = mock_server.uri();

    Mock::given(method("POST"))
        .and(path("/api/v1/blobs"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "reference": "",
            "size": 0,
            "success": false
        })))
        .mount(&mock_server)
        .await;

    let client = NestGateHttpClient::new(base_url, 5000).unwrap();
    let err = client.store(b"data", None).await.unwrap_err();
    assert!(matches!(err, NestGateHttpError::StoreFailed));
}

#[cfg(feature = "live-clients")]
#[tokio::test]
async fn wiremock_retrieve_invalid_base64() {
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    let mock_server = MockServer::start().await;
    let base_url = mock_server.uri();

    Mock::given(method("GET"))
        .and(path("/api/v1/blobs/badref"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "data": "!!!invalid-base64!!!",
            "content_type": "application/octet-stream",
            "size": 0
        })))
        .mount(&mock_server)
        .await;

    let client = NestGateHttpClient::new(base_url, 5000).unwrap();
    let err = client.retrieve("badref").await.unwrap_err();
    assert!(matches!(err, NestGateHttpError::InvalidData));
}
