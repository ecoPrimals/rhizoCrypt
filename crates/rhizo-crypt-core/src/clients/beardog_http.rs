// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2024–2026 ecoPrimals Project

//! BearDog HTTP API types and client.
//!
//! This module contains the HTTP request/response types for connecting
//! to the BearDog signing service via its REST API.
//!
//! ## Feature Gate
//!
//! This module is only compiled when the `live-clients` feature is enabled.
//! Without the feature, the BearDog client operates in scaffolded mode.

use serde::{Deserialize, Serialize};

/// Default cryptographic key type for signing operations.
const DEFAULT_KEY_TYPE: &str = "ed25519";

// ============================================================================
// HTTP Request/Response Types (mirrors BearDog's API)
// ============================================================================

/// Sign request for BearDog HTTP API.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HttpSignRequest {
    /// Data to sign (base64 encoded).
    pub data: String,
    /// Key type for signing.
    #[serde(default = "default_key_type")]
    pub key_type: String,
}

fn default_key_type() -> String {
    DEFAULT_KEY_TYPE.to_string()
}

/// Sign response from BearDog HTTP API.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HttpSignResponse {
    /// Base64-encoded signature.
    pub signature: String,
    /// Whether signing succeeded.
    #[serde(default = "default_true")]
    pub success: bool,
}

const fn default_true() -> bool {
    true
}

/// Verify request for BearDog HTTP API.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HttpVerifyRequest {
    /// Data that was signed (base64 encoded).
    pub data: String,
    /// Signature to verify (base64 encoded).
    pub signature: String,
}

/// Verify response from BearDog HTTP API.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HttpVerifyResponse {
    /// Whether the signature is valid.
    pub valid: bool,
}

/// DID resolution request.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HttpResolveDidRequest {
    /// DID to resolve.
    pub did: String,
}

/// DID document from BearDog.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HttpDidDocument {
    /// DID identifier.
    pub id: String,
    /// Verification methods.
    #[serde(default)]
    pub verification_method: Vec<HttpVerificationMethod>,
    /// Authentication references.
    #[serde(default)]
    pub authentication: Vec<String>,
    /// Controller DID.
    pub controller: Option<String>,
}

/// Verification method in DID document.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HttpVerificationMethod {
    /// Method ID.
    pub id: String,
    /// Method type (e.g., Ed25519VerificationKey2020).
    #[serde(rename = "type")]
    pub method_type: String,
    /// Controller DID.
    pub controller: String,
    /// Public key in multibase format.
    #[serde(rename = "publicKeyMultibase")]
    pub public_key_multibase: Option<String>,
}

/// Health check response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HttpHealthResponse {
    /// Service status.
    pub status: String,
    /// Service version.
    #[serde(default)]
    pub version: String,
}

// ============================================================================
// HTTP Client
// ============================================================================

/// HTTP client for BearDog API.
#[derive(Clone)]
pub struct BearDogHttpClient {
    client: reqwest::Client,
    base_url: String,
}

impl BearDogHttpClient {
    /// Create a new HTTP client for BearDog.
    ///
    /// # Arguments
    ///
    /// * `base_url` - Base URL of the BearDog service (e.g., "http://127.0.0.1:8080")
    /// * `timeout_ms` - Request timeout in milliseconds
    ///
    /// # Errors
    ///
    /// Returns an error if the HTTP client cannot be built.
    pub fn new(base_url: impl Into<String>, timeout_ms: u64) -> Result<Self, reqwest::Error> {
        let base_url = base_url.into();
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_millis(timeout_ms))
            .build()?;

        Ok(Self {
            client,
            base_url,
        })
    }

    /// Sign data using BearDog.
    ///
    /// Returns the signature as `bytes::Bytes` for zero-copy downstream use.
    ///
    /// # Errors
    ///
    /// Returns error if the HTTP request fails or response cannot be parsed.
    pub async fn sign(&self, data: &[u8]) -> Result<bytes::Bytes, BearDogHttpError> {
        use base64::{Engine, engine::general_purpose::STANDARD};

        let request = HttpSignRequest {
            data: STANDARD.encode(data),
            key_type: DEFAULT_KEY_TYPE.to_string(),
        };

        let response = self
            .client
            .post(format!("{}/ai/sign", self.base_url))
            .json(&request)
            .send()
            .await
            .map_err(BearDogHttpError::Request)?;

        if !response.status().is_success() {
            return Err(BearDogHttpError::Status(response.status().as_u16()));
        }

        let sign_response: HttpSignResponse =
            response.json().await.map_err(BearDogHttpError::Parse)?;

        if !sign_response.success {
            return Err(BearDogHttpError::SigningFailed);
        }

        STANDARD
            .decode(&sign_response.signature)
            .map(bytes::Bytes::from)
            .map_err(|_| BearDogHttpError::InvalidSignature)
    }

    /// Verify a signature using BearDog.
    ///
    /// # Errors
    ///
    /// Returns error if the HTTP request fails or response cannot be parsed.
    pub async fn verify(&self, data: &[u8], signature: &[u8]) -> Result<bool, BearDogHttpError> {
        use base64::{Engine, engine::general_purpose::STANDARD};

        let request = HttpVerifyRequest {
            data: STANDARD.encode(data),
            signature: STANDARD.encode(signature),
        };

        let response = self
            .client
            .post(format!("{}/ai/verify", self.base_url))
            .json(&request)
            .send()
            .await
            .map_err(BearDogHttpError::Request)?;

        if !response.status().is_success() {
            return Err(BearDogHttpError::Status(response.status().as_u16()));
        }

        let verify_response: HttpVerifyResponse =
            response.json().await.map_err(BearDogHttpError::Parse)?;

        Ok(verify_response.valid)
    }

    /// Check BearDog health.
    ///
    /// # Errors
    ///
    /// Returns error if the health check fails.
    pub async fn health(&self) -> Result<HttpHealthResponse, BearDogHttpError> {
        let response = self
            .client
            .get(format!("{}/ai/health", self.base_url))
            .send()
            .await
            .map_err(BearDogHttpError::Request)?;

        if !response.status().is_success() {
            return Err(BearDogHttpError::Status(response.status().as_u16()));
        }

        response.json().await.map_err(BearDogHttpError::Parse)
    }
}

/// Errors from BearDog HTTP client.
#[derive(Debug)]
pub enum BearDogHttpError {
    /// HTTP request failed.
    Request(reqwest::Error),
    /// Non-success HTTP status.
    Status(u16),
    /// Failed to parse response.
    Parse(reqwest::Error),
    /// Signing operation failed.
    SigningFailed,
    /// Invalid signature format.
    InvalidSignature,
}

impl std::fmt::Display for BearDogHttpError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Request(e) => write!(f, "HTTP request failed: {e}"),
            Self::Status(code) => write!(f, "HTTP status {code}"),
            Self::Parse(e) => write!(f, "Failed to parse response: {e}"),
            Self::SigningFailed => write!(f, "Signing operation failed"),
            Self::InvalidSignature => write!(f, "Invalid signature format"),
        }
    }
}

impl std::error::Error for BearDogHttpError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Request(e) | Self::Parse(e) => Some(e),
            _ => None,
        }
    }
}

#[cfg(test)]
#[expect(clippy::unwrap_used, reason = "test code")]
mod tests {
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
}
