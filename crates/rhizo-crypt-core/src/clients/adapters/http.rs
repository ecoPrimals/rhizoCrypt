//! HTTP/REST protocol adapter.
//!
//! Provides a generic HTTP adapter for calling REST APIs from capability clients.

use super::ProtocolAdapter;
use crate::error::{Result, RhizoCryptError};
use async_trait::async_trait;
use std::fmt;

/// HTTP protocol adapter.
///
/// Communicates with services via HTTP/REST APIs.
#[derive(Clone)]
pub struct HttpAdapter {
    base_url: String,
    client: reqwest::Client,
}

impl HttpAdapter {
    /// Create a new HTTP adapter.
    ///
    /// # Arguments
    ///
    /// * `base_url` - Base URL of the service (e.g., "http://localhost:9500")
    ///
    /// # Errors
    ///
    /// Returns error if base URL is invalid.
    pub fn new(base_url: &str) -> Result<Self> {
        // Validate URL
        let parsed = base_url
            .parse::<reqwest::Url>()
            .map_err(|e| RhizoCryptError::integration(format!("Invalid URL: {e}")))?;

        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .map_err(|e| {
                RhizoCryptError::integration(format!("Failed to create HTTP client: {e}"))
            })?;

        Ok(Self {
            base_url: parsed.to_string().trim_end_matches('/').to_string(),
            client,
        })
    }

    /// Build full URL for a method.
    fn build_url(&self, method: &str) -> String {
        format!("{}/api/v1/{}", self.base_url, method)
    }
}

impl fmt::Debug for HttpAdapter {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("HttpAdapter")
            .field("base_url", &self.base_url)
            .field("protocol", &"http")
            .finish_non_exhaustive()
    }
}

#[async_trait]
impl ProtocolAdapter for HttpAdapter {
    fn protocol(&self) -> &'static str {
        "http"
    }

    async fn call_json(&self, method: &str, args_json: String) -> Result<String> {
        let url = self.build_url(method);

        tracing::debug!(
            url = %url,
            method = method,
            "HTTP adapter calling method"
        );

        let response = self
            .client
            .post(&url)
            .header("Content-Type", "application/json")
            .body(args_json)
            .send()
            .await
            .map_err(|e| RhizoCryptError::integration(format!("HTTP request failed: {e}")))?;

        // Check status
        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_else(|_| "<no body>".to_string());
            return Err(RhizoCryptError::integration(format!(
                "HTTP request failed with status {status}: {body}"
            )));
        }

        // Get response as JSON string
        response
            .text()
            .await
            .map_err(|e| RhizoCryptError::integration(format!("Failed to read response: {e}")))
    }

    async fn call_oneway_json(&self, method: &str, args_json: String) -> Result<()> {
        let url = self.build_url(method);

        tracing::debug!(
            url = %url,
            method = method,
            "HTTP adapter calling method (oneway)"
        );

        let response = self
            .client
            .post(&url)
            .header("Content-Type", "application/json")
            .body(args_json)
            .send()
            .await
            .map_err(|e| RhizoCryptError::integration(format!("HTTP request failed: {e}")))?;

        // Check status but don't wait for response body
        if !response.status().is_success() {
            let status = response.status();
            return Err(RhizoCryptError::integration(format!(
                "HTTP request failed with status {status}"
            )));
        }

        Ok(())
    }

    async fn is_healthy(&self) -> bool {
        // Try to connect to /health endpoint
        let health_url = format!("{}/health", self.base_url);

        self.client
            .get(&health_url)
            .send()
            .await
            .is_ok_and(|response| response.status().is_success())
    }

    fn endpoint(&self) -> &str {
        &self.base_url
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn test_http_adapter_creation() {
        let adapter = HttpAdapter::new("http://localhost:9500").unwrap();
        assert_eq!(adapter.protocol(), "http");
        assert_eq!(adapter.endpoint(), "http://localhost:9500");
    }

    #[test]
    fn test_http_adapter_https() {
        let adapter = HttpAdapter::new("https://api.example.com").unwrap();
        assert_eq!(adapter.protocol(), "http"); // Still "http" protocol type
        assert_eq!(adapter.endpoint(), "https://api.example.com");
    }

    #[test]
    fn test_http_adapter_invalid_url() {
        let result = HttpAdapter::new("not a url");
        assert!(result.is_err());
    }

    #[test]
    fn test_http_adapter_build_url() {
        let adapter = HttpAdapter::new("http://localhost:9500").unwrap();
        let url = adapter.build_url("sign");
        assert_eq!(url, "http://localhost:9500/api/v1/sign");
    }

    #[test]
    fn test_http_adapter_trailing_slash() {
        let adapter = HttpAdapter::new("http://localhost:9500/").unwrap();
        let url = adapter.build_url("sign");
        assert_eq!(url, "http://localhost:9500/api/v1/sign");
    }
}
