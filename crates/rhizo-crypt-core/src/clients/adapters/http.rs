// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2024–2026 ecoPrimals Project

//! HTTP/REST protocol adapter (pure Rust — hyper/tower stack).
//!
//! Provides a generic HTTP adapter for calling REST APIs from capability clients.
//! Uses `hyper-util` for outbound connections (ecoBin compliant, no reqwest/ring).

use super::ProtocolAdapter;
use crate::error::{Result, RhizoCryptError};
use async_trait::async_trait;
use std::fmt;

pub use eco_http::EcoHttpClient;

/// Lightweight HTTP client built on hyper-util (pure Rust, no reqwest).
///
/// Used by all outbound HTTP adapters and primal-specific clients.
pub mod eco_http {
    use bytes::Bytes;
    use http_body_util::{BodyExt, Full};
    use hyper::Uri;
    use hyper_util::client::legacy::Client;
    use hyper_util::rt::TokioExecutor;
    use std::time::Duration;

    /// Pure-Rust HTTP client for ecosystem IPC.
    #[derive(Clone)]
    pub struct EcoHttpClient {
        client: Client<hyper_util::client::legacy::connect::HttpConnector, Full<Bytes>>,
        timeout: Duration,
    }

    impl EcoHttpClient {
        /// Create a client with the given timeout.
        #[must_use]
        pub fn new(timeout: Duration) -> Self {
            let client = Client::builder(TokioExecutor::new()).build_http();
            Self {
                client,
                timeout,
            }
        }

        /// POST JSON to a URL, returning `(status_code, body_text)`.
        ///
        /// # Errors
        ///
        /// Returns error on connection failure or body read failure.
        pub async fn post_json(&self, url: &str, body: &str) -> Result<(u16, String)> {
            let uri: Uri = url.parse().map_err(|e| err(format!("Invalid URL: {e}")))?;

            let req = hyper::Request::builder()
                .method(hyper::Method::POST)
                .uri(uri)
                .header("Content-Type", "application/json")
                .body(Full::new(Bytes::from(body.to_owned())))
                .map_err(|e| err(format!("Failed to build request: {e}")))?;

            let response = tokio::time::timeout(self.timeout, self.client.request(req))
                .await
                .map_err(|_| err("HTTP request timed out".to_string()))?
                .map_err(|e| err(format!("HTTP request failed: {e}")))?;

            let status = response.status().as_u16();
            let body_bytes = response
                .into_body()
                .collect()
                .await
                .map_err(|e| err(format!("Failed to read response: {e}")))?
                .to_bytes();
            let text = String::from_utf8_lossy(&body_bytes).into_owned();

            Ok((status, text))
        }

        /// GET a URL, returning `(status_code, body_text)`.
        ///
        /// # Errors
        ///
        /// Returns error on connection failure.
        pub async fn get(&self, url: &str) -> Result<(u16, String)> {
            let uri: Uri = url.parse().map_err(|e| err(format!("Invalid URL: {e}")))?;

            let req = hyper::Request::builder()
                .method(hyper::Method::GET)
                .uri(uri)
                .body(Full::new(Bytes::new()))
                .map_err(|e| err(format!("Failed to build request: {e}")))?;

            let response = tokio::time::timeout(self.timeout, self.client.request(req))
                .await
                .map_err(|_| err("HTTP request timed out".to_string()))?
                .map_err(|e| err(format!("HTTP request failed: {e}")))?;

            let status = response.status().as_u16();
            let body_bytes = response
                .into_body()
                .collect()
                .await
                .map_err(|e| err(format!("Failed to read response: {e}")))?
                .to_bytes();
            let text = String::from_utf8_lossy(&body_bytes).into_owned();

            Ok((status, text))
        }

        /// Validate a URL string.
        ///
        /// # Errors
        ///
        /// Returns error if the URL is invalid.
        pub fn validate_url(url: &str) -> Result<String> {
            let _: Uri = url.parse().map_err(|e| err(format!("Invalid URL: {e}")))?;
            Ok(url.trim_end_matches('/').to_string())
        }
    }

    impl std::fmt::Debug for EcoHttpClient {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            f.debug_struct("EcoHttpClient").field("timeout", &self.timeout).finish_non_exhaustive()
        }
    }

    type Result<T> = std::result::Result<T, crate::error::RhizoCryptError>;

    fn err(msg: String) -> crate::error::RhizoCryptError {
        crate::error::RhizoCryptError::integration(msg)
    }
}

/// HTTP protocol adapter.
///
/// Communicates with services via HTTP/REST APIs using hyper (pure Rust).
#[derive(Clone)]
pub struct HttpAdapter {
    base_url: String,
    client: EcoHttpClient,
}

impl HttpAdapter {
    /// Create a new HTTP adapter.
    ///
    /// # Arguments
    ///
    /// * `base_url` - Base URL of the service (e.g., <http://localhost:9500>)
    ///
    /// # Errors
    ///
    /// Returns error if base URL is invalid.
    pub fn new(base_url: &str) -> Result<Self> {
        let base_url = EcoHttpClient::validate_url(base_url)?;
        let client = EcoHttpClient::new(std::time::Duration::from_secs(30));
        Ok(Self {
            base_url,
            client,
        })
    }

    /// Build full URL for a method.
    fn build_url(&self, method: &str) -> String {
        format!("{}{}/{}", self.base_url, crate::constants::API_VERSION_PREFIX, method)
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

    async fn call_json(&self, method: &str, args_json: &str) -> Result<String> {
        let url = self.build_url(method);

        tracing::debug!(url = %url, method = method, "HTTP adapter calling method");

        let (status, body) = self.client.post_json(&url, args_json).await?;

        if !(200..300).contains(&status) {
            return Err(RhizoCryptError::integration(format!(
                "HTTP request failed with status {status}: {body}"
            )));
        }

        Ok(body)
    }

    async fn call_oneway_json(&self, method: &str, args_json: &str) -> Result<()> {
        let url = self.build_url(method);

        tracing::debug!(url = %url, method = method, "HTTP adapter calling method (oneway)");

        let (status, _) = self.client.post_json(&url, args_json).await?;

        if !(200..300).contains(&status) {
            return Err(RhizoCryptError::integration(format!(
                "HTTP request failed with status {status}"
            )));
        }

        Ok(())
    }

    async fn is_healthy(&self) -> bool {
        let health_url = format!("{}/health", self.base_url);
        self.client.get(&health_url).await.is_ok_and(|(status, _)| (200..300).contains(&status))
    }

    fn endpoint(&self) -> &str {
        &self.base_url
    }
}

#[cfg(test)]
#[expect(clippy::unwrap_used, reason = "test code")]
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
        assert_eq!(adapter.protocol(), "http");
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

    #[test]
    fn test_http_adapter_debug() {
        let adapter = HttpAdapter::new("http://localhost:9500").unwrap();
        let debug_str = format!("{adapter:?}");
        assert!(debug_str.contains("HttpAdapter"));
        assert!(debug_str.contains("http://localhost:9500"));
        assert!(debug_str.contains("http"));
    }

    #[test]
    fn test_http_adapter_clone() {
        let adapter = HttpAdapter::new("http://localhost:9500").unwrap();
        let cloned = adapter.clone();
        assert_eq!(cloned.endpoint(), adapter.endpoint());
        assert_eq!(cloned.protocol(), adapter.protocol());
    }

    #[test]
    fn test_build_url_nested_method() {
        let adapter = HttpAdapter::new("http://localhost:9500").unwrap();
        let url = adapter.build_url("dag/session/create");
        assert!(url.contains("/api/v1/dag/session/create"));
    }

    #[tokio::test]
    async fn test_is_healthy_unreachable() {
        let adapter = HttpAdapter::new("http://127.0.0.1:1").unwrap();
        assert!(!adapter.is_healthy().await);
    }

    #[tokio::test]
    async fn test_call_json_unreachable() {
        let adapter = HttpAdapter::new("http://127.0.0.1:1").unwrap();
        let result = adapter.call_json("test", "{}").await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("HTTP"));
    }

    #[tokio::test]
    async fn test_call_oneway_json_unreachable() {
        let adapter = HttpAdapter::new("http://127.0.0.1:1").unwrap();
        let result = adapter.call_oneway_json("test", "{}").await;
        assert!(result.is_err());
    }
}
