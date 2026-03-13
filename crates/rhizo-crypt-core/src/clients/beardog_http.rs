// SPDX-License-Identifier: AGPL-3.0-only
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
    "ed25519".to_string()
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
    /// # Errors
    ///
    /// Returns error if the HTTP request fails or response cannot be parsed.
    pub async fn sign(&self, data: &[u8]) -> Result<Vec<u8>, BearDogHttpError> {
        use base64::{engine::general_purpose::STANDARD, Engine};

        let request = HttpSignRequest {
            data: STANDARD.encode(data),
            key_type: "ed25519".to_string(),
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

        STANDARD.decode(&sign_response.signature).map_err(|_| BearDogHttpError::InvalidSignature)
    }

    /// Verify a signature using BearDog.
    ///
    /// # Errors
    ///
    /// Returns error if the HTTP request fails or response cannot be parsed.
    pub async fn verify(&self, data: &[u8], signature: &[u8]) -> Result<bool, BearDogHttpError> {
        use base64::{engine::general_purpose::STANDARD, Engine};

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
