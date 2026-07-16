// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2024–2026 ecoPrimals Project

//! `BearDog` HTTP API types and client.
//!
//! This module contains the HTTP request/response types for connecting
//! to the `BearDog` signing service via its REST API.
//!
//! ## Feature Gate
//!
//! This module is only compiled when the `live-clients` feature is enabled.
//! Without the feature, the `BearDog` client operates in scaffolded mode.

use serde::{Deserialize, Serialize};

/// Default cryptographic key type for signing operations.
const DEFAULT_KEY_TYPE: &str = "ed25519";

// ============================================================================
// HTTP Request/Response Types (mirrors BearDog's API)
// ============================================================================

/// Sign request for `BearDog` HTTP API.
#[cfg_attr(not(test), allow(dead_code))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct HttpSignRequest {
    /// Data to sign (base64 encoded).
    pub data: String,
    /// Key type for signing.
    #[serde(default = "default_key_type")]
    pub key_type: String,
}

fn default_key_type() -> String {
    DEFAULT_KEY_TYPE.to_string()
}

/// Sign response from `BearDog` HTTP API.
#[cfg_attr(not(test), allow(dead_code))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct HttpSignResponse {
    /// Base64-encoded signature.
    pub signature: String,
    /// Whether signing succeeded.
    #[serde(default = "default_true")]
    pub success: bool,
}

const fn default_true() -> bool {
    true
}

/// Verify request for `BearDog` HTTP API.
#[cfg_attr(not(test), allow(dead_code))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct HttpVerifyRequest {
    /// Data that was signed (base64 encoded).
    pub data: String,
    /// Signature to verify (base64 encoded).
    pub signature: String,
}

/// Verify response from `BearDog` HTTP API.
#[cfg_attr(not(test), allow(dead_code))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct HttpVerifyResponse {
    /// Whether the signature is valid.
    pub valid: bool,
}

/// DID resolution request.
#[cfg_attr(not(test), allow(dead_code))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct HttpResolveDidRequest {
    /// DID to resolve.
    pub did: String,
}

/// DID document from `BearDog`.
#[cfg_attr(not(test), allow(dead_code))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct HttpDidDocument {
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
#[cfg_attr(not(test), allow(dead_code))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct HttpVerificationMethod {
    /// Method ID.
    pub id: String,
    /// Method type (e.g., `Ed25519VerificationKey2020`).
    #[serde(rename = "type")]
    pub method_type: String,
    /// Controller DID.
    pub controller: String,
    /// Public key in multibase format.
    #[serde(rename = "publicKeyMultibase")]
    pub public_key_multibase: Option<String>,
}

/// Health check response.
#[cfg_attr(not(test), allow(dead_code))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct HttpHealthResponse {
    /// Service status.
    pub status: String,
    /// Service version.
    #[serde(default)]
    pub version: String,
}

// ============================================================================
// HTTP Client
// ============================================================================

/// HTTP client for `BearDog` API (pure Rust — hyper/tower stack).
#[derive(Clone)]
pub struct SigningHttpClient {
    client: crate::clients::adapters::http::EcoHttpClient,
    base_url: String,
}

impl SigningHttpClient {
    /// Create a new HTTP client for `BearDog`.
    ///
    /// # Errors
    ///
    /// Returns an error if the base URL is invalid.
    pub fn new(base_url: impl Into<String>, timeout_ms: u64) -> Result<Self, SigningHttpError> {
        let base_url = base_url.into();
        let client = crate::clients::adapters::http::EcoHttpClient::new(
            std::time::Duration::from_millis(timeout_ms),
        );
        Ok(Self {
            client,
            base_url,
        })
    }

    /// Sign data using `BearDog`.
    ///
    /// Returns the signature as `bytes::Bytes` for zero-copy downstream use.
    ///
    /// # Errors
    ///
    /// Returns error if the HTTP request fails or response cannot be parsed.
    pub async fn sign(&self, data: &[u8]) -> Result<bytes::Bytes, SigningHttpError> {
        use base64::{Engine, engine::general_purpose::STANDARD};

        let request = HttpSignRequest {
            data: STANDARD.encode(data),
            key_type: DEFAULT_KEY_TYPE.to_string(),
        };

        let body = serde_json::to_string(&request).map_err(|e| {
            SigningHttpError::Transport(format!("Failed to serialize request: {e}"))
        })?;

        let (status, text) = self
            .client
            .post_json(&format!("{}/ai/sign", self.base_url), &body)
            .await
            .map_err(|e| SigningHttpError::Transport(e.to_string()))?;

        if !(200..300).contains(&status) {
            return Err(SigningHttpError::Status(status));
        }

        let sign_response: HttpSignResponse =
            serde_json::from_str(&text).map_err(|e| SigningHttpError::Parse(e.to_string()))?;

        if !sign_response.success {
            return Err(SigningHttpError::SigningFailed);
        }

        STANDARD
            .decode(&sign_response.signature)
            .map(bytes::Bytes::from)
            .map_err(|_| SigningHttpError::InvalidSignature)
    }

    /// Verify a signature using `BearDog`.
    ///
    /// # Errors
    ///
    /// Returns error if the HTTP request fails or response cannot be parsed.
    pub async fn verify(&self, data: &[u8], signature: &[u8]) -> Result<bool, SigningHttpError> {
        use base64::{Engine, engine::general_purpose::STANDARD};

        let request = HttpVerifyRequest {
            data: STANDARD.encode(data),
            signature: STANDARD.encode(signature),
        };

        let body = serde_json::to_string(&request).map_err(|e| {
            SigningHttpError::Transport(format!("Failed to serialize request: {e}"))
        })?;

        let (status, text) = self
            .client
            .post_json(&format!("{}/ai/verify", self.base_url), &body)
            .await
            .map_err(|e| SigningHttpError::Transport(e.to_string()))?;

        if !(200..300).contains(&status) {
            return Err(SigningHttpError::Status(status));
        }

        let verify_response: HttpVerifyResponse =
            serde_json::from_str(&text).map_err(|e| SigningHttpError::Parse(e.to_string()))?;

        Ok(verify_response.valid)
    }

    /// Check `BearDog` health.
    ///
    /// # Errors
    ///
    /// Returns error if the health check fails.
    #[cfg_attr(not(test), allow(dead_code))]
    pub(crate) async fn health(&self) -> Result<HttpHealthResponse, SigningHttpError> {
        let (status, text) = self
            .client
            .get(&format!("{}/ai/health", self.base_url))
            .await
            .map_err(|e| SigningHttpError::Transport(e.to_string()))?;

        if !(200..300).contains(&status) {
            return Err(SigningHttpError::Status(status));
        }

        serde_json::from_str(&text).map_err(|e| SigningHttpError::Parse(e.to_string()))
    }
}

/// Errors from `BearDog` HTTP client.
#[derive(Debug, thiserror::Error)]
pub enum SigningHttpError {
    /// HTTP transport error.
    #[error("HTTP transport: {0}")]
    Transport(String),
    /// Non-success HTTP status.
    #[error("HTTP status {0}")]
    Status(u16),
    /// Failed to parse response.
    #[error("Failed to parse response: {0}")]
    Parse(String),
    /// Signing operation failed.
    #[error("Signing operation failed")]
    SigningFailed,
    /// Invalid signature format.
    #[error("Invalid signature format")]
    InvalidSignature,
}

#[cfg(test)]
#[path = "beardog_http_tests.rs"]
mod tests;
