// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2024–2026 ecoPrimals Project

//! Unix domain socket protocol adapter.
//!
//! Sends JSON-RPC 2.0 requests over Unix domain sockets using HTTP/1.1 framing.
//! This is the canonical ecoPrimals IPC transport for the Tower Atomic pattern:
//!
//! - **Pure Rust** — zero C/TLS dependencies for local IPC
//! - **HTTP-compatible** — works with axum/hyper servers bound to Unix sockets
//! - **Inherently secure** — filesystem permissions replace TLS
//!
//! ## Tower Atomic Pattern
//!
//! ```text
//! rhizoCrypt ──unix socket──► BearDog    (crypto atoms)
//! rhizoCrypt ──unix socket──► Songbird   (HTTP/TLS state machine)
//! rhizoCrypt ──unix socket──► LoamSpine  (permanent storage)
//! rhizoCrypt ──unix socket──► NestGate   (payload storage)
//! ```

use super::ProtocolAdapter;
use crate::error::{Result, RhizoCryptError};
use async_trait::async_trait;
use std::fmt;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::UnixStream;

/// Unix domain socket protocol adapter.
///
/// Communicates with sibling primals via JSON-RPC 2.0 over HTTP/1.1,
/// transported on Unix domain sockets. No TLS required for local IPC.
pub struct UnixSocketAdapter {
    socket_path: PathBuf,
    rpc_path: String,
    timeout: Duration,
    request_id: AtomicU64,
}

impl UnixSocketAdapter {
    /// Create a new Unix socket adapter.
    ///
    /// # Arguments
    ///
    /// * `socket_path` - Path to the Unix domain socket
    ///
    /// # Errors
    ///
    /// Returns error if the socket path is invalid.
    pub fn new(socket_path: impl AsRef<Path>) -> Result<Self> {
        let socket_path = socket_path.as_ref().to_path_buf();

        Ok(Self {
            socket_path,
            rpc_path: crate::constants::JSON_RPC_PATH.to_string(),
            timeout: crate::constants::CONNECTION_TIMEOUT,
            request_id: AtomicU64::new(1),
        })
    }

    /// Create from an endpoint string.
    ///
    /// Accepts:
    /// - `unix:///path/to/socket` (with protocol prefix)
    /// - `/path/to/socket` (bare path)
    ///
    /// # Errors
    ///
    /// Returns error if the path is invalid.
    pub fn from_endpoint(endpoint: &str) -> Result<Self> {
        let path = endpoint.strip_prefix("unix://").unwrap_or(endpoint);
        Self::new(path)
    }

    /// Set custom RPC path (default: "/rpc").
    #[must_use]
    pub fn with_rpc_path(mut self, path: impl Into<String>) -> Self {
        self.rpc_path = path.into();
        self
    }

    /// Set timeout for operations.
    #[must_use]
    pub const fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    fn next_id(&self) -> u64 {
        self.request_id.fetch_add(1, Ordering::Relaxed)
    }

    /// Build HTTP/1.1 POST request header and return bytes (header only; body appended separately).
    #[must_use]
    pub fn build_http_request(path: &str, body_len: usize) -> String {
        format!(
            "POST {path} HTTP/1.1\r\n\
             Host: localhost\r\n\
             Content-Type: application/json\r\n\
             Content-Length: {body_len}\r\n\
             Connection: close\r\n\
             \r\n"
        )
    }

    /// Parse JSON-RPC 2.0 response body into result string.
    /// Returns error for JSON-RPC error objects or missing result.
    pub fn parse_json_rpc_response(body: &[u8]) -> Result<String> {
        let response: serde_json::Value = serde_json::from_slice(body).map_err(|e| {
            RhizoCryptError::integration(format!("Failed to parse JSON-RPC response: {e}"))
        })?;

        if let Some(error) = response.get("error") {
            let code = error.get("code").and_then(serde_json::Value::as_i64).unwrap_or(-1);
            let message =
                error.get("message").and_then(serde_json::Value::as_str).unwrap_or("Unknown error");
            return Err(RhizoCryptError::integration(format!(
                "JSON-RPC error [{code}]: {message}"
            )));
        }

        let result = response.get("result").ok_or_else(|| {
            RhizoCryptError::integration("JSON-RPC response missing 'result' field")
        })?;

        serde_json::to_string(result)
            .map_err(|e| RhizoCryptError::integration(format!("Failed to serialize result: {e}")))
    }

    /// Send an HTTP/1.1 POST over the Unix socket and return the response body.
    async fn http_post(&self, path: &str, body: &[u8]) -> Result<Vec<u8>> {
        let mut stream = tokio::time::timeout(self.timeout, UnixStream::connect(&self.socket_path))
            .await
            .map_err(|_| {
                RhizoCryptError::integration(format!(
                    "Unix socket connection timed out: {}",
                    self.socket_path.display()
                ))
            })?
            .map_err(|e| {
                RhizoCryptError::integration(format!(
                    "Unix socket connection failed at {}: {e}",
                    self.socket_path.display()
                ))
            })?;

        let header = Self::build_http_request(path, body.len());

        stream
            .write_all(header.as_bytes())
            .await
            .map_err(|e| RhizoCryptError::integration(format!("Unix socket write failed: {e}")))?;
        stream
            .write_all(body)
            .await
            .map_err(|e| RhizoCryptError::integration(format!("Unix socket write failed: {e}")))?;

        let mut response_buf = Vec::with_capacity(4096);
        tokio::time::timeout(self.timeout, stream.read_to_end(&mut response_buf))
            .await
            .map_err(|_| RhizoCryptError::integration("Unix socket read timed out"))?
            .map_err(|e| RhizoCryptError::integration(format!("Unix socket read failed: {e}")))?;

        Self::parse_http_response(&response_buf)
    }

    /// Parse an HTTP/1.1 response, extracting the body after status validation.
    fn parse_http_response(raw: &[u8]) -> Result<Vec<u8>> {
        let header_end = raw.windows(4).position(|w| w == b"\r\n\r\n").ok_or_else(|| {
            RhizoCryptError::integration("Malformed HTTP response: no header boundary")
        })?;

        let header_bytes = &raw[..header_end];
        let body = &raw[header_end + 4..];

        let Ok(header_str) = std::str::from_utf8(header_bytes) else {
            return Err(RhizoCryptError::integration(
                "Invalid HTTP response encoding: headers are not valid UTF-8",
            ));
        };

        let status_line = header_str.lines().next().unwrap_or("");
        let status_code: u16 =
            status_line.split_whitespace().nth(1).and_then(|s| s.parse().ok()).unwrap_or(0);

        if !(200..300).contains(&status_code) {
            let body_preview = String::from_utf8_lossy(body);
            return Err(RhizoCryptError::integration(format!(
                "Unix socket HTTP {status_code}: {body_preview}"
            )));
        }

        Ok(body.to_vec())
    }
}

impl fmt::Debug for UnixSocketAdapter {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("UnixSocketAdapter")
            .field("socket_path", &self.socket_path)
            .field("rpc_path", &self.rpc_path)
            .field("protocol", &"unix")
            .finish_non_exhaustive()
    }
}

#[async_trait]
impl ProtocolAdapter for UnixSocketAdapter {
    fn protocol(&self) -> &str {
        "unix"
    }

    async fn call_json(&self, method: &str, args_json: String) -> Result<String> {
        let params: serde_json::Value =
            serde_json::from_str(&args_json).unwrap_or(serde_json::Value::String(args_json));

        let request = serde_json::json!({
            "jsonrpc": "2.0",
            "method": method,
            "params": params,
            "id": self.next_id()
        });

        let body = serde_json::to_vec(&request).map_err(|e| {
            RhizoCryptError::integration(format!("Failed to serialize JSON-RPC request: {e}"))
        })?;

        tracing::debug!(
            method = %method,
            socket = %self.socket_path.display(),
            "Unix socket adapter calling method"
        );

        let response_body = self.http_post(&self.rpc_path, &body).await?;
        Self::parse_json_rpc_response(&response_body)
    }

    async fn call_oneway_json(&self, method: &str, args_json: String) -> Result<()> {
        let params: serde_json::Value =
            serde_json::from_str(&args_json).unwrap_or(serde_json::Value::String(args_json));

        // JSON-RPC 2.0 notification: no `id` field
        let request = serde_json::json!({
            "jsonrpc": "2.0",
            "method": method,
            "params": params,
        });

        let body = serde_json::to_vec(&request).map_err(|e| {
            RhizoCryptError::integration(format!("Failed to serialize JSON-RPC notification: {e}"))
        })?;

        tracing::debug!(
            method = %method,
            socket = %self.socket_path.display(),
            "Unix socket adapter sending notification"
        );

        let _ = self.http_post(&self.rpc_path, &body).await;
        Ok(())
    }

    async fn is_healthy(&self) -> bool {
        self.socket_path.exists() && self.call_json("system.health", "{}".to_string()).await.is_ok()
    }

    fn endpoint(&self) -> &str {
        self.socket_path.to_str().unwrap_or("unix-socket")
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
    fn test_from_endpoint_with_prefix() {
        let adapter = UnixSocketAdapter::from_endpoint("unix:///tmp/primal.sock").unwrap();
        assert_eq!(adapter.socket_path, PathBuf::from("/tmp/primal.sock"));
        assert_eq!(adapter.protocol(), "unix");
    }

    #[test]
    fn test_from_endpoint_bare_path() {
        let adapter = UnixSocketAdapter::from_endpoint("/run/ecoPrimals/beardog.sock").unwrap();
        assert_eq!(adapter.socket_path, PathBuf::from("/run/ecoPrimals/beardog.sock"));
    }

    #[test]
    fn test_with_rpc_path() {
        let adapter =
            UnixSocketAdapter::new("/tmp/test.sock").unwrap().with_rpc_path("/api/v1/rpc");
        assert_eq!(adapter.rpc_path, "/api/v1/rpc");
    }

    #[test]
    fn test_with_timeout() {
        let adapter =
            UnixSocketAdapter::new("/tmp/test.sock").unwrap().with_timeout(Duration::from_secs(5));
        assert_eq!(adapter.timeout, Duration::from_secs(5));
    }

    #[test]
    fn test_parse_http_response_success() {
        let raw = b"HTTP/1.1 200 OK\r\nContent-Type: application/json\r\n\r\n\
                     {\"jsonrpc\":\"2.0\",\"result\":true,\"id\":1}";
        let body = UnixSocketAdapter::parse_http_response(raw).unwrap();
        assert_eq!(body, b"{\"jsonrpc\":\"2.0\",\"result\":true,\"id\":1}");
    }

    #[test]
    fn test_parse_http_response_error_status() {
        let raw = b"HTTP/1.1 500 Internal Server Error\r\n\r\nSomething went wrong";
        let result = UnixSocketAdapter::parse_http_response(raw);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("500"));
    }

    #[test]
    fn test_parse_http_response_malformed() {
        let raw = b"garbage data without headers";
        let result = UnixSocketAdapter::parse_http_response(raw);
        assert!(result.is_err());
    }

    #[test]
    fn test_request_id_increment() {
        let adapter = UnixSocketAdapter::new("/tmp/test.sock").unwrap();
        assert_eq!(adapter.next_id(), 1);
        assert_eq!(adapter.next_id(), 2);
        assert_eq!(adapter.next_id(), 3);
    }

    #[tokio::test]
    async fn test_nonexistent_socket_fails() {
        let adapter =
            UnixSocketAdapter::new("/tmp/nonexistent_ecoPrimal_socket_12345.sock").unwrap();
        let result = adapter.call_json("test", "{}".to_string()).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_is_healthy_nonexistent() {
        let adapter =
            UnixSocketAdapter::new("/tmp/nonexistent_ecoPrimal_socket_12345.sock").unwrap();
        assert!(!adapter.is_healthy().await);
    }

    #[test]
    fn test_debug_format() {
        let adapter = UnixSocketAdapter::new("/tmp/test.sock").unwrap();
        let debug = format!("{adapter:?}");
        assert!(debug.contains("UnixSocketAdapter"));
        assert!(debug.contains("/tmp/test.sock"));
    }

    #[test]
    fn test_build_http_request() {
        let header = UnixSocketAdapter::build_http_request("/rpc", 42);
        assert!(header.starts_with("POST /rpc HTTP/1.1\r\n"));
        assert!(header.contains("Content-Length: 42\r\n"));
        assert!(header.contains("Host: localhost\r\n"));
        assert!(header.ends_with("\r\n\r\n"));

        let header_empty = UnixSocketAdapter::build_http_request("/api", 0);
        assert!(header_empty.contains("Content-Length: 0\r\n"));
    }

    #[test]
    fn test_parse_json_rpc_response_success() {
        let body = br#"{"jsonrpc":"2.0","result":{"ok":true},"id":1}"#;
        let result = UnixSocketAdapter::parse_json_rpc_response(body).unwrap();
        assert_eq!(result, r#"{"ok":true}"#);
    }

    #[test]
    fn test_parse_json_rpc_response_error() {
        let body =
            br#"{"jsonrpc":"2.0","error":{"code":-32601,"message":"Method not found"},"id":1}"#;
        let err = UnixSocketAdapter::parse_json_rpc_response(body).unwrap_err();
        assert!(err.to_string().contains("Method not found"));
        assert!(err.to_string().contains("-32601"));
    }

    #[test]
    fn test_parse_json_rpc_response_malformed() {
        let body = b"not valid json";
        let result = UnixSocketAdapter::parse_json_rpc_response(body);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_json_rpc_response_missing_result() {
        let body = br#"{"jsonrpc":"2.0","id":1}"#;
        let err = UnixSocketAdapter::parse_json_rpc_response(body).unwrap_err();
        assert!(err.to_string().contains("missing 'result'"));
    }

    #[test]
    fn test_parse_http_response_invalid_utf8() {
        let raw = b"HTTP/1.1 200 OK\r\nX-Custom: \xff\xfe\r\n\r\nbody";
        let result = UnixSocketAdapter::parse_http_response(raw);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("UTF-8"));
    }

    #[test]
    fn test_socket_path_resolution() {
        let with_prefix = UnixSocketAdapter::from_endpoint("unix:///var/run/sock.sock").unwrap();
        assert_eq!(with_prefix.socket_path, PathBuf::from("/var/run/sock.sock"));
        assert_eq!(with_prefix.endpoint(), "/var/run/sock.sock");

        let bare = UnixSocketAdapter::from_endpoint("/tmp/bare.sock").unwrap();
        assert_eq!(bare.socket_path, PathBuf::from("/tmp/bare.sock"));
        assert_eq!(bare.endpoint(), "/tmp/bare.sock");
    }
}
