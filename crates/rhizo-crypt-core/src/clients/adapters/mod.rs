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
//!
//! With a structured endpoint:
//!
//! ```no_run
//! # use rhizo_crypt_core::clients::adapters::{ProtocolAdapter, AdapterFactory};
//! # use rhizo_crypt_core::transport::TransportEndpoint;
//! # tokio::runtime::Runtime::new().unwrap().block_on(async {
//! let endpoint = TransportEndpoint::tcp("127.0.0.1", 9500);
//! let adapter = AdapterFactory::from_transport(&endpoint)?;
//! let _result = adapter.call_json("sign", r#"{"data":[]}"#).await?;
//! # Ok::<(), rhizo_crypt_core::error::RhizoCryptError>(())
//! # });
//! ```

use crate::error::{Result, RhizoCryptError};
use crate::transport::TransportEndpoint;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::future::Future;
use std::pin::Pin;

/// Boxed future for object-safe async trait methods.
///
/// `ProtocolAdapter` is used as `Arc<dyn ProtocolAdapter>` (trait objects),
/// which requires object safety. Native `async fn in dyn Trait` is not yet
/// stable (RFC 3185), so async methods return a pinned boxed future. This
/// replaces the `async-trait` proc macro with explicit desugaring.
pub type BoxFuture<'a, T> = Pin<Box<dyn Future<Output = T> + Send + 'a>>;

#[cfg(feature = "http-clients")]
pub mod http;
pub mod tarpc;
#[cfg(unix)]
pub mod unix_socket;

// Re-exports
#[cfg(feature = "http-clients")]
pub use http::HttpAdapter;
pub use tarpc::TarpcAdapter;
#[cfg(unix)]
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
    fn call_json<'a>(
        &'a self,
        method: &'a str,
        args_json: &'a str,
    ) -> BoxFuture<'a, Result<String>>;

    /// Call a remote method without expecting a response (fire-and-forget).
    fn call_oneway_json<'a>(
        &'a self,
        method: &'a str,
        args_json: &'a str,
    ) -> BoxFuture<'a, Result<()>>;

    /// Check if the adapter is connected/healthy.
    fn is_healthy(&self) -> BoxFuture<'_, bool>;

    /// Get endpoint information.
    fn endpoint(&self) -> &str;
}

/// Helper trait for type-safe calls (convenience wrapper).
pub trait ProtocolAdapterExt: ProtocolAdapter {
    /// Call a remote method with typed arguments.
    fn call<'a, Args, Response>(
        &'a self,
        method: &'a str,
        args: Args,
    ) -> BoxFuture<'a, Result<Response>>
    where
        Args: Serialize + Send + Sync + 'a,
        Response: for<'de> Deserialize<'de> + Send + 'a,
    {
        Box::pin(async move {
            let args_json = serde_json::to_string(&args).map_err(|e| {
                RhizoCryptError::integration(format!("Failed to serialize args: {e}"))
            })?;

            let response_json = self.call_json(method, &args_json).await?;

            serde_json::from_str(&response_json).map_err(|e| {
                RhizoCryptError::integration(format!("Failed to deserialize response: {e}"))
            })
        })
    }

    /// Call a remote method without expecting a response.
    fn call_oneway<'a, Args>(&'a self, method: &'a str, args: Args) -> BoxFuture<'a, Result<()>>
    where
        Args: Serialize + Send + Sync + 'a,
    {
        Box::pin(async move {
            let args_json = serde_json::to_string(&args).map_err(|e| {
                RhizoCryptError::integration(format!("Failed to serialize args: {e}"))
            })?;

            self.call_oneway_json(method, &args_json).await
        })
    }
}

// Blanket implementation for all ProtocolAdapter implementors
impl<T: ProtocolAdapter + ?Sized> ProtocolAdapterExt for T {}

// ============================================================================
// Adapter Factory
// ============================================================================

/// Factory for creating protocol adapters.
///
/// Supports platform-aware transport types:
///
/// | Protocol | Transport | Platform | Use case |
/// |----------|-----------|----------|----------|
/// | `unix://` | Unix socket | Unix only | Local IPC (Tower Atomic) |
/// | `tarpc://` | TCP binary | All | High-perf binary RPC |
/// | `http://` | HTTP/REST | All | Legacy / remote (feature-gated) |
///
/// On non-Unix platforms (Windows), bare paths and `unix://` endpoints fall
/// back to tarpc over TCP loopback.
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
        // Bare path → Unix socket (Unix) or error (non-Unix)
        if endpoint.starts_with('/') || endpoint.starts_with('.') {
            #[cfg(unix)]
            return Ok(Box::new(UnixSocketAdapter::new(endpoint)?));
            #[cfg(not(unix))]
            return Err(RhizoCryptError::integration(format!(
                "Unix socket paths not available on this platform: {endpoint}. \
                 Use tarpc:// or http:// instead."
            )));
        }

        if let Some((protocol, _)) = endpoint.split_once("://") {
            match protocol {
                #[cfg(unix)]
                "unix" => Ok(Box::new(UnixSocketAdapter::from_endpoint(endpoint)?)),
                #[cfg(not(unix))]
                "unix" => Err(RhizoCryptError::integration(
                    "Unix socket transport not available on this platform. \
                     Use tarpc:// or http:// instead.",
                )),
                "tcp" | "tarpc" => Ok(Box::new(TarpcAdapter::new(endpoint)?)),
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

    /// Create an adapter from a structured [`TransportEndpoint`].
    ///
    /// Preferred over [`Self::create`] when the endpoint is already structured
    /// (e.g., from discovery). Avoids the `Display` → re-parse round-trip.
    ///
    /// # Errors
    ///
    /// Returns error if:
    /// - Transport type is unavailable on this platform (e.g., UDS on Windows)
    /// - Protocol is unsupported (`MeshRelay` — not yet implemented)
    pub fn from_transport(endpoint: &TransportEndpoint) -> Result<Box<dyn ProtocolAdapter>> {
        match endpoint {
            #[cfg(unix)]
            TransportEndpoint::Uds {
                path,
            } => Ok(Box::new(UnixSocketAdapter::new(path)?)),
            #[cfg(not(unix))]
            TransportEndpoint::Uds {
                path,
            } => Err(RhizoCryptError::integration(format!(
                "Unix socket transport not available on this platform: {path}. \
                 Use TCP or HTTP instead."
            ))),
            TransportEndpoint::Tcp {
                host,
                port,
            } => {
                let addr = format!("{host}:{port}");
                Ok(Box::new(TarpcAdapter::new(&addr)?))
            }
            TransportEndpoint::MeshRelay {
                peer_id,
                capability,
            } => Err(RhizoCryptError::integration(format!(
                "Mesh relay transport not yet implemented (peer: {peer_id}, cap: {capability}). \
                     Direct transport required."
            ))),
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
    /// Only available on Unix platforms. On Windows, use `tarpc()` for local
    /// IPC over TCP loopback.
    ///
    /// # Errors
    ///
    /// Returns an error if the socket path is invalid.
    #[cfg(unix)]
    pub fn unix(socket_path: &str) -> Result<Box<dyn ProtocolAdapter>> {
        Ok(Box::new(UnixSocketAdapter::new(socket_path)?))
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
#[expect(clippy::unwrap_used, reason = "test code")]
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

    #[cfg(unix)]
    #[test]
    fn test_factory_unix_with_protocol() {
        let adapter = AdapterFactory::create("unix:///run/biomeos/signing.sock").unwrap();
        assert_eq!(adapter.protocol(), "unix");
        assert_eq!(adapter.endpoint(), "/run/biomeos/signing.sock");
    }

    #[cfg(unix)]
    #[test]
    fn test_factory_unix_bare_path() {
        let adapter = AdapterFactory::create("/run/biomeos/signing.sock").unwrap();
        assert_eq!(adapter.protocol(), "unix");
    }

    #[cfg(unix)]
    #[test]
    fn test_factory_unix_relative_path() {
        let adapter = AdapterFactory::create("./sockets/signing.sock").unwrap();
        assert_eq!(adapter.protocol(), "unix");
    }

    #[cfg(unix)]
    #[test]
    fn test_unix_factory_method() {
        let adapter = AdapterFactory::unix("/tmp/test.sock").unwrap();
        assert_eq!(adapter.protocol(), "unix");
        assert_eq!(adapter.endpoint(), "/tmp/test.sock");
    }

    #[cfg(not(unix))]
    #[test]
    fn test_factory_unix_unavailable_on_non_unix() {
        let result = AdapterFactory::create("unix:///run/biomeos/signing.sock");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("not available"));
    }

    #[cfg(not(unix))]
    #[test]
    fn test_factory_bare_path_unavailable_on_non_unix() {
        let result = AdapterFactory::create("/run/biomeos/signing.sock");
        assert!(result.is_err());
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

    #[test]
    fn test_factory_from_transport_tcp() {
        use crate::transport::TransportEndpoint;
        let endpoint = TransportEndpoint::tcp("127.0.0.1", 9600);
        let adapter = AdapterFactory::from_transport(&endpoint).unwrap();
        assert_eq!(adapter.protocol(), "tarpc");
    }

    #[cfg(unix)]
    #[test]
    fn test_factory_from_transport_uds() {
        use crate::transport::TransportEndpoint;
        let endpoint = TransportEndpoint::uds("/tmp/test.sock");
        let adapter = AdapterFactory::from_transport(&endpoint).unwrap();
        assert_eq!(adapter.protocol(), "unix");
    }

    #[cfg(not(unix))]
    #[test]
    fn test_factory_from_transport_uds_unavailable() {
        use crate::transport::TransportEndpoint;
        let endpoint = TransportEndpoint::uds("/tmp/test.sock");
        let result = AdapterFactory::from_transport(&endpoint);
        assert!(result.is_err());
    }

    #[test]
    fn test_factory_from_transport_mesh_relay_unsupported() {
        use crate::transport::TransportEndpoint;
        let endpoint = TransportEndpoint::MeshRelay {
            peer_id: "strand-gate".to_string(),
            capability: "security".to_string(),
        };
        let result = AdapterFactory::from_transport(&endpoint);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Mesh relay"));
    }
}
