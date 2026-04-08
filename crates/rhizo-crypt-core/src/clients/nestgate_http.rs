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

/// HTTP client for `NestGate` API (pure Rust — hyper/tower stack).
#[derive(Clone)]
pub struct NestGateHttpClient {
    client: crate::clients::adapters::http::EcoHttpClient,
    base_url: String,
}

impl NestGateHttpClient {
    /// Create a new HTTP client for `NestGate`.
    ///
    /// # Errors
    ///
    /// Returns an error if the base URL is invalid.
    pub fn new(base_url: impl Into<String>, timeout_ms: u64) -> Result<Self, NestGateHttpError> {
        let base_url = base_url.into();
        let client = crate::clients::adapters::http::EcoHttpClient::new(
            std::time::Duration::from_millis(timeout_ms),
        );
        Ok(Self {
            client,
            base_url,
        })
    }

    /// Store a blob in `NestGate`.
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

        let body = serde_json::to_string(&request)
            .map_err(|e| NestGateHttpError::Transport(format!("Serialize: {e}")))?;

        let (status, text) = self
            .client
            .post_json(
                &format!("{}{}/blobs", self.base_url, crate::constants::API_VERSION_PREFIX),
                &body,
            )
            .await
            .map_err(|e| NestGateHttpError::Transport(e.to_string()))?;

        if !(200..300).contains(&status) {
            return Err(NestGateHttpError::Status(status));
        }

        let store_response: HttpStoreBlobResponse =
            serde_json::from_str(&text).map_err(|e| NestGateHttpError::Parse(e.to_string()))?;

        if !store_response.success {
            return Err(NestGateHttpError::StoreFailed);
        }

        Ok(store_response.reference)
    }

    /// Retrieve a blob from `NestGate` by reference.
    ///
    /// # Errors
    ///
    /// Returns error if the HTTP request fails, blob not found, or response cannot be parsed.
    pub async fn retrieve(&self, reference: &str) -> Result<bytes::Bytes, NestGateHttpError> {
        use base64::{Engine, engine::general_purpose::STANDARD};

        let url = format!(
            "{}{}/blobs/{}",
            self.base_url,
            crate::constants::API_VERSION_PREFIX,
            reference
        );

        let (status, text) =
            self.client.get(&url).await.map_err(|e| NestGateHttpError::Transport(e.to_string()))?;

        if status == 404 {
            return Err(NestGateHttpError::NotFound);
        }
        if !(200..300).contains(&status) {
            return Err(NestGateHttpError::Status(status));
        }

        let retrieve_response: HttpRetrieveBlobResponse =
            serde_json::from_str(&text).map_err(|e| NestGateHttpError::Parse(e.to_string()))?;

        STANDARD
            .decode(&retrieve_response.data)
            .map(bytes::Bytes::from)
            .map_err(|_| NestGateHttpError::InvalidData)
    }

    /// Check if a blob exists (GET + status check).
    ///
    /// # Errors
    ///
    /// Returns error if the HTTP request fails.
    pub async fn exists(&self, reference: &str) -> Result<bool, NestGateHttpError> {
        let url = format!(
            "{}{}/blobs/{}",
            self.base_url,
            crate::constants::API_VERSION_PREFIX,
            reference
        );
        let (status, _) =
            self.client.get(&url).await.map_err(|e| NestGateHttpError::Transport(e.to_string()))?;
        Ok((200..300).contains(&status))
    }

    /// Get blob metadata.
    ///
    /// # Errors
    ///
    /// Returns error if the HTTP request fails or blob not found.
    pub async fn metadata(&self, reference: &str) -> Result<HttpBlobMetadata, NestGateHttpError> {
        let url = format!(
            "{}{}/blobs/{}/metadata",
            self.base_url,
            crate::constants::API_VERSION_PREFIX,
            reference
        );
        let (status, text) =
            self.client.get(&url).await.map_err(|e| NestGateHttpError::Transport(e.to_string()))?;

        if status == 404 {
            return Err(NestGateHttpError::NotFound);
        }
        if !(200..300).contains(&status) {
            return Err(NestGateHttpError::Status(status));
        }

        serde_json::from_str(&text).map_err(|e| NestGateHttpError::Parse(e.to_string()))
    }

    /// Check `NestGate` health.
    ///
    /// # Errors
    ///
    /// Returns error if the health check fails.
    pub async fn health(&self) -> Result<HttpHealthResponse, NestGateHttpError> {
        let url = format!("{}{}", self.base_url, crate::constants::HEALTH_CHECK_PATH);
        let (status, text) =
            self.client.get(&url).await.map_err(|e| NestGateHttpError::Transport(e.to_string()))?;

        if !(200..300).contains(&status) {
            return Err(NestGateHttpError::Status(status));
        }

        serde_json::from_str(&text).map_err(|e| NestGateHttpError::Parse(e.to_string()))
    }
}

/// Errors from `NestGate` HTTP client.
#[derive(Debug, thiserror::Error)]
pub enum NestGateHttpError {
    /// HTTP transport error.
    #[error("HTTP transport: {0}")]
    Transport(String),
    /// Non-success HTTP status.
    #[error("HTTP status {0}")]
    Status(u16),
    /// Failed to parse response.
    #[error("Failed to parse response: {0}")]
    Parse(String),
    /// Store operation failed.
    #[error("Store operation failed")]
    StoreFailed,
    /// Blob not found.
    #[error("Blob not found")]
    NotFound,
    /// Invalid data format.
    #[error("Invalid data format")]
    InvalidData,
}

#[cfg(test)]
#[expect(clippy::unwrap_used, reason = "test code")]
#[path = "nestgate_http_tests_wiremock.rs"]
mod tests;
