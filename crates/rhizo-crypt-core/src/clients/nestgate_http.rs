// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2024–2026 ecoPrimals Project

//! `NestGate` HTTP API types and client.
//!
//! This module contains the HTTP request/response types for connecting
//! to the `NestGate` storage service via its REST API.
//!
//! ## Feature Gate
//!
//! This module is only compiled when the `live-clients` feature is enabled.
//! Without the feature, the `NestGate` client operates in scaffolded mode.

use serde::{Deserialize, Serialize};

/// Default MIME content type for untyped binary blobs.
const DEFAULT_CONTENT_TYPE: &str = "application/octet-stream";

// ============================================================================
// HTTP Request/Response Types (mirrors NestGate's API)
// ============================================================================

/// Store blob request.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HttpStoreBlobRequest {
    /// Base64-encoded blob data.
    pub data: String,
    /// Content type.
    #[serde(default = "default_content_type")]
    pub content_type: String,
    /// Optional metadata.
    #[serde(default)]
    pub metadata: std::collections::HashMap<String, String>,
}

fn default_content_type() -> String {
    DEFAULT_CONTENT_TYPE.to_string()
}

/// Store blob response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HttpStoreBlobResponse {
    /// Content-addressed reference (blake3 hash).
    pub reference: String,
    /// Size in bytes.
    pub size: u64,
    /// Whether storage succeeded.
    #[serde(default = "default_true")]
    pub success: bool,
}

const fn default_true() -> bool {
    true
}

/// Retrieve blob response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HttpRetrieveBlobResponse {
    /// Base64-encoded blob data.
    pub data: String,
    /// Content type.
    pub content_type: String,
    /// Size in bytes.
    pub size: u64,
}

/// Blob metadata response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HttpBlobMetadata {
    /// Content reference.
    pub reference: String,
    /// Content type.
    pub content_type: String,
    /// Size in bytes.
    pub size: u64,
    /// Creation timestamp.
    #[serde(default)]
    pub created_at: Option<String>,
    /// Custom metadata.
    #[serde(default)]
    pub metadata: std::collections::HashMap<String, String>,
}

/// Health check response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HttpHealthResponse {
    /// Service status.
    pub status: String,
    /// Available storage in bytes.
    #[serde(default)]
    pub available_bytes: u64,
    /// Used storage in bytes.
    #[serde(default)]
    pub used_bytes: u64,
}

// ============================================================================
// HTTP Client
// ============================================================================

/// HTTP client for `NestGate` API.
#[derive(Clone)]
pub struct NestGateHttpClient {
    client: reqwest::Client,
    base_url: String,
}

impl NestGateHttpClient {
    /// Create a new HTTP client for `NestGate`.
    ///
    /// # Arguments
    ///
    /// * `base_url` - Base URL of the `NestGate` service (e.g., <http://127.0.0.1:9200>)
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

    /// Store a blob in `NestGate`.
    ///
    /// Returns the content-addressed reference (blake3 hash).
    ///
    /// # Errors
    ///
    /// Returns error if the HTTP request fails or response cannot be parsed.
    pub async fn store(
        &self,
        data: &[u8],
        content_type: Option<&str>,
    ) -> Result<String, NestGateHttpError> {
        use base64::{Engine, engine::general_purpose::STANDARD};

        let request = HttpStoreBlobRequest {
            data: STANDARD.encode(data),
            content_type: content_type.unwrap_or(DEFAULT_CONTENT_TYPE).to_string(),
            metadata: std::collections::HashMap::new(),
        };

        let response = self
            .client
            .post(format!("{}{}/blobs", self.base_url, crate::constants::API_VERSION_PREFIX))
            .json(&request)
            .send()
            .await
            .map_err(NestGateHttpError::Request)?;

        if !response.status().is_success() {
            return Err(NestGateHttpError::Status(response.status().as_u16()));
        }

        let store_response: HttpStoreBlobResponse =
            response.json().await.map_err(NestGateHttpError::Parse)?;

        if !store_response.success {
            return Err(NestGateHttpError::StoreFailed);
        }

        Ok(store_response.reference)
    }

    /// Retrieve a blob from `NestGate` by reference.
    ///
    /// Returns the blob as `bytes::Bytes` for zero-copy downstream use.
    ///
    /// # Errors
    ///
    /// Returns error if the HTTP request fails, blob not found, or response cannot be parsed.
    pub async fn retrieve(&self, reference: &str) -> Result<bytes::Bytes, NestGateHttpError> {
        use base64::{Engine, engine::general_purpose::STANDARD};

        let response = self
            .client
            .get(format!(
                "{}{}/blobs/{}",
                self.base_url,
                crate::constants::API_VERSION_PREFIX,
                reference
            ))
            .send()
            .await
            .map_err(NestGateHttpError::Request)?;

        if response.status().as_u16() == 404 {
            return Err(NestGateHttpError::NotFound);
        }

        if !response.status().is_success() {
            return Err(NestGateHttpError::Status(response.status().as_u16()));
        }

        let retrieve_response: HttpRetrieveBlobResponse =
            response.json().await.map_err(NestGateHttpError::Parse)?;

        STANDARD
            .decode(&retrieve_response.data)
            .map(bytes::Bytes::from)
            .map_err(|_| NestGateHttpError::InvalidData)
    }

    /// Check if a blob exists.
    ///
    /// # Errors
    ///
    /// Returns error if the HTTP request fails.
    pub async fn exists(&self, reference: &str) -> Result<bool, NestGateHttpError> {
        let response = self
            .client
            .head(format!(
                "{}{}/blobs/{}",
                self.base_url,
                crate::constants::API_VERSION_PREFIX,
                reference
            ))
            .send()
            .await
            .map_err(NestGateHttpError::Request)?;

        Ok(response.status().is_success())
    }

    /// Get blob metadata.
    ///
    /// # Errors
    ///
    /// Returns error if the HTTP request fails or blob not found.
    pub async fn metadata(&self, reference: &str) -> Result<HttpBlobMetadata, NestGateHttpError> {
        let response = self
            .client
            .get(format!(
                "{}{}/blobs/{}/metadata",
                self.base_url,
                crate::constants::API_VERSION_PREFIX,
                reference
            ))
            .send()
            .await
            .map_err(NestGateHttpError::Request)?;

        if response.status().as_u16() == 404 {
            return Err(NestGateHttpError::NotFound);
        }

        if !response.status().is_success() {
            return Err(NestGateHttpError::Status(response.status().as_u16()));
        }

        response.json().await.map_err(NestGateHttpError::Parse)
    }

    /// Check `NestGate` health.
    ///
    /// # Errors
    ///
    /// Returns error if the health check fails.
    pub async fn health(&self) -> Result<HttpHealthResponse, NestGateHttpError> {
        let response = self
            .client
            .get(format!("{}{}", self.base_url, crate::constants::HEALTH_CHECK_PATH))
            .send()
            .await
            .map_err(NestGateHttpError::Request)?;

        if !response.status().is_success() {
            return Err(NestGateHttpError::Status(response.status().as_u16()));
        }

        response.json().await.map_err(NestGateHttpError::Parse)
    }
}

/// Errors from `NestGate` HTTP client.
#[derive(Debug)]
pub enum NestGateHttpError {
    /// HTTP request failed.
    Request(reqwest::Error),
    /// Non-success HTTP status.
    Status(u16),
    /// Failed to parse response.
    Parse(reqwest::Error),
    /// Store operation failed.
    StoreFailed,
    /// Blob not found.
    NotFound,
    /// Invalid data format.
    InvalidData,
}

impl std::fmt::Display for NestGateHttpError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Request(e) => write!(f, "HTTP request failed: {e}"),
            Self::Status(code) => write!(f, "HTTP status {code}"),
            Self::Parse(e) => write!(f, "Failed to parse response: {e}"),
            Self::StoreFailed => write!(f, "Store operation failed"),
            Self::NotFound => write!(f, "Blob not found"),
            Self::InvalidData => write!(f, "Invalid data format"),
        }
    }
}

impl std::error::Error for NestGateHttpError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Request(e) | Self::Parse(e) => Some(e),
            _ => None,
        }
    }
}

#[cfg(test)]
#[expect(clippy::unwrap_used, reason = "test code")]
#[path = "nestgate_http_tests_wiremock.rs"]
mod tests;
