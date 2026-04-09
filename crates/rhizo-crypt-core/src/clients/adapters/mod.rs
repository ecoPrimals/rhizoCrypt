// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2024–2026 ecoPrimals Project

//! Protocol adapters for capability-based clients.
//!
//! This module provides protocol adapters that allow capability clients to
//! communicate with services using different protocols (tarpc, HTTP, gRPC, etc.)
//! without knowing the protocol details.
//!
//! ## Philosophy
//!
//! Capability clients are protocol-agnostic. They describe WHAT they need
//! (signing, storage, etc.) not HOW to communicate. Protocol adapters handle
//! the HOW.
//!
//! ## Usage
//!
//! ```no_run
//! # use rhizo_crypt_core::clients::adapters::{ProtocolAdapter, AdapterFactory};
//! # use rhizo_crypt_core::types::Signature;
//! # tokio::runtime::Runtime::new().unwrap().block_on(async {
//! // Create adapter based on service endpoint
//! let adapter = AdapterFactory::create("127.0.0.1:9500")?;
//!
//! // Use adapter to make calls (protocol-independent)
//! let args_json = r#"{"data":[],"signer":"did:key:test"}"#;
//! let _result_json = adapter.call_json("sign", args_json).await?;
//! # Ok::<(), rhizo_crypt_core::error::RhizoCryptError>(())
//! # });
//! ```

use crate::error::{Result, RhizoCryptError};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::fmt;

#[cfg(feature = "http-clients")]
pub mod http;
pub mod tarpc;
pub mod unix_socket;

// Re-exports
#[cfg(feature = "http-clients")]
pub use http::HttpAdapter;
pub use tarpc::TarpcAdapter;
pub use unix_socket::UnixSocketAdapter;

// ============================================================================
// Protocol Adapter Trait
// ============================================================================

/// Protocol adapter for capability clients.
///
/// Provides a uniform interface for calling remote services regardless of
/// the underlying protocol (tarpc, HTTP, gRPC, WebSocket, etc.).
///
/// Note: This trait uses JSON serialization to be object-safe.
/// For performance-critical paths, consider using concrete adapter types directly.
#[async_trait]
pub trait ProtocolAdapter: Send + Sync + fmt::Debug {
    /// Get the protocol name (http, tarpc, grpc, etc.).
    fn protocol(&self) -> &str;

    /// Call a remote method with JSON-serializable arguments.
    ///
    /// # Arguments
    ///
    /// * `method` - Method name (e.g., "sign", "store", "commit")
    /// * `args_json` - JSON-serialized arguments
    ///
    /// # Returns
    ///
    /// JSON-serialized response or error
    async fn call_json(&self, method: &str, args_json: &str) -> Result<String>;

    /// Call a remote method without expecting a response (fire-and-forget).
    async fn call_oneway_json(&self, method: &str, args_json: &str) -> Result<()>;

    /// Check if the adapter is connected/healthy.
    async fn is_healthy(&self) -> bool;

    /// Get endpoint information.
    fn endpoint(&self) -> &str;
}

/// Helper trait for type-safe calls (convenience wrapper).
#[async_trait]
pub trait ProtocolAdapterExt: ProtocolAdapter {
    /// Call a remote method with typed arguments.
    async fn call<Args, Response>(&self, method: &str, args: Args) -> Result<Response>
    where
        Args: Serialize + Send + Sync,
        Response: for<'de> Deserialize<'de> + Send,
    {
        let args_json = serde_json::to_string(&args)
            .map_err(|e| RhizoCryptError::integration(format!("Failed to serialize args: {e}")))?;

        let response_json = self.call_json(method, &args_json).await?;

        serde_json::from_str(&response_json).map_err(|e| {
            RhizoCryptError::integration(format!("Failed to deserialize response: {e}"))
        })
    }

    /// Call a remote method without expecting a response.
    async fn call_oneway<Args>(&self, method: &str, args: Args) -> Result<()>
    where
        Args: Serialize + Send + Sync,
    {
        let args_json = serde_json::to_string(&args)
            .map_err(|e| RhizoCryptError::integration(format!("Failed to serialize args: {e}")))?;

        self.call_oneway_json(method, &args_json).await
    }
}

// Blanket implementation for all ProtocolAdapter implementors
impl<T: ProtocolAdapter + ?Sized> ProtocolAdapterExt for T {}

// ============================================================================
// Adapter Factory
// ============================================================================

/// Factory for creating protocol adapters.
///
/// Supports three transport types:
///
/// | Protocol | Transport | C deps | Use case |
/// |----------|-----------|--------|----------|
/// | `unix://` | Unix socket | None | Local IPC (Tower Atomic) |
/// | `tarpc://` | TCP binary | None | High-perf binary RPC |
/// | `http://` | HTTP/REST | ring | Legacy / remote (feature-gated) |
pub struct AdapterFactory;

impl AdapterFactory {
    /// Create an adapter based on protocol hint in endpoint.
    ///
    /// # Arguments
    ///
    /// * `endpoint` - Service endpoint:
    ///   - `unix:///run/biomeos/{primal}.sock` — Unix socket IPC
    ///   - `/run/biomeos/{primal}.sock` — bare path → Unix socket
    ///   - `tarpc://{host}:{port}` — tarpc binary protocol
    ///   - `http://{host}:{port}` — HTTP/REST (requires `http-clients` feature)
    ///
    /// # Errors
    ///
    /// Returns error if:
    /// - Protocol is unsupported
    /// - Endpoint format is invalid
    /// - Adapter creation fails
    pub fn create(endpoint: &str) -> Result<Box<dyn ProtocolAdapter>> {
        // Bare path → Unix socket
        if endpoint.starts_with('/') || endpoint.starts_with('.') {
            return Ok(Box::new(UnixSocketAdapter::new(endpoint)?));
        }

        if let Some((protocol, _)) = endpoint.split_once("://") {
            match protocol {
                "unix" => Ok(Box::new(UnixSocketAdapter::from_endpoint(endpoint)?)),
                "tarpc" => Ok(Box::new(TarpcAdapter::new(endpoint)?)),
                #[cfg(feature = "http-clients")]
                "http" | "https" => Ok(Box::new(HttpAdapter::new(endpoint)?)),
                #[cfg(not(feature = "http-clients"))]
                "http" | "https" => Err(RhizoCryptError::integration(
                    "HTTP transport requires 'http-clients' feature. \
                     Use unix:// for local IPC (Tower Atomic pattern).",
                )),
                unsupported => Err(RhizoCryptError::integration(format!(
                    "Unsupported protocol: {unsupported}. Supported: unix, tarpc{}",
                    if cfg!(feature = "http-clients") {
                        ", http, https"
                    } else {
                        ""
                    }
                ))),
            }
        } else {
            // No protocol, no path → host:port assumed
            #[cfg(feature = "http-clients")]
            {
                tracing::warn!(
                    endpoint,
                    "No protocol specified, defaulting to HTTP. \
                     Consider: unix:// for local IPC or http://{endpoint}",
                );
                let http_endpoint = format!("http://{endpoint}");
                Ok(Box::new(HttpAdapter::new(&http_endpoint)?))
            }
            #[cfg(not(feature = "http-clients"))]
            {
                tracing::info!(
                    endpoint,
                    "No protocol specified, defaulting to tarpc (pure Rust). \
                     Consider: unix:// for local IPC or tarpc://{endpoint}",
                );
                Ok(Box::new(TarpcAdapter::new(endpoint)?))
            }
        }
    }

    /// Create an HTTP adapter (requires `http-clients` feature).
    ///
    /// # Errors
    ///
    /// Returns an error if the HTTP adapter cannot be created or feature is disabled.
    #[cfg(feature = "http-clients")]
    pub fn http(endpoint: &str) -> Result<Box<dyn ProtocolAdapter>> {
        Ok(Box::new(HttpAdapter::new(endpoint)?))
    }

    /// Create a tarpc adapter.
    ///
    /// # Errors
    ///
    /// Returns an error if the tarpc adapter cannot be created.
    pub fn tarpc(endpoint: &str) -> Result<Box<dyn ProtocolAdapter>> {
        Ok(Box::new(TarpcAdapter::new(endpoint)?))
    }

    /// Create a Unix socket adapter (Tower Atomic IPC).
    ///
    /// # Errors
    ///
    /// Returns an error if the socket path is invalid.
    pub fn unix(socket_path: &str) -> Result<Box<dyn ProtocolAdapter>> {
        Ok(Box::new(UnixSocketAdapter::new(socket_path)?))
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(feature = "http-clients")]
    #[test]
    fn test_factory_http() {
        let adapter = AdapterFactory::create("http://localhost:9500").unwrap();
        assert_eq!(adapter.protocol(), "http");
    }

    #[cfg(feature = "http-clients")]
    #[test]
    fn test_factory_https() {
        let adapter = AdapterFactory::create("https://api.example.com:443").unwrap();
        assert_eq!(adapter.protocol(), "http");
    }

    #[cfg(feature = "http-clients")]
    #[test]
    fn test_factory_no_protocol_defaults_to_http() {
        let adapter = AdapterFactory::create("localhost:9500").unwrap();
        assert_eq!(adapter.protocol(), "http");
    }

    #[cfg(not(feature = "http-clients"))]
    #[test]
    fn test_factory_http_disabled() {
        let result = AdapterFactory::create("http://localhost:9500");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("http-clients"));
    }

    #[cfg(not(feature = "http-clients"))]
    #[test]
    fn test_factory_no_protocol_defaults_to_tarpc() {
        let adapter = AdapterFactory::create("localhost:9500").unwrap();
        assert_eq!(adapter.protocol(), "tarpc");
    }

    #[test]
    fn test_factory_unsupported_protocol() {
        let result = AdapterFactory::create("grpc://localhost:9500");
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("Unsupported protocol"));
    }

    #[test]
    fn test_factory_tarpc() {
        let adapter = AdapterFactory::create("tarpc://localhost:9600").unwrap();
        assert_eq!(adapter.protocol(), "tarpc");
    }

    #[test]
    fn test_tarpc_factory_method() {
        let adapter = AdapterFactory::tarpc("localhost:7777").unwrap();
        assert_eq!(adapter.protocol(), "tarpc");
        assert_eq!(adapter.endpoint(), "localhost:7777");
    }

    #[test]
    fn test_factory_unix_with_protocol() {
        let adapter = AdapterFactory::create("unix:///run/biomeos/beardog.sock").unwrap();
        assert_eq!(adapter.protocol(), "unix");
        assert_eq!(adapter.endpoint(), "/run/biomeos/beardog.sock");
    }

    #[test]
    fn test_factory_unix_bare_path() {
        let adapter = AdapterFactory::create("/run/biomeos/beardog.sock").unwrap();
        assert_eq!(adapter.protocol(), "unix");
    }

    #[test]
    fn test_factory_unix_relative_path() {
        let adapter = AdapterFactory::create("./sockets/beardog.sock").unwrap();
        assert_eq!(adapter.protocol(), "unix");
    }

    #[test]
    fn test_unix_factory_method() {
        let adapter = AdapterFactory::unix("/tmp/test.sock").unwrap();
        assert_eq!(adapter.protocol(), "unix");
        assert_eq!(adapter.endpoint(), "/tmp/test.sock");
    }

    #[cfg(feature = "http-clients")]
    #[test]
    fn test_http_factory_method() {
        let adapter = AdapterFactory::http("http://localhost:9500").unwrap();
        assert_eq!(adapter.protocol(), "http");
        assert_eq!(adapter.endpoint(), "http://localhost:9500");
    }

    #[test]
    fn test_factory_unsupported_protocol_ws() {
        let result = AdapterFactory::create("ws://localhost:9500");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Unsupported protocol"));
    }

    #[test]
    fn test_factory_unsupported_protocol_ftps() {
        let result = AdapterFactory::create("ftps://localhost:21");
        assert!(result.is_err());
    }
}
