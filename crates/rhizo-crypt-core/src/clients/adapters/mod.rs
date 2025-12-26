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
//! ```ignore
//! use rhizo_crypt_core::clients::adapters::{ProtocolAdapter, AdapterFactory};
//!
//! // Create adapter based on service endpoint metadata
//! let adapter = AdapterFactory::create_for_endpoint(&endpoint)?;
//!
//! // Use adapter to make calls (protocol-independent)
//! let result: Signature = adapter.call("sign", (data, did)).await?;
//! ```

use crate::error::{Result, RhizoCryptError};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::fmt;

pub mod http;
#[cfg(feature = "live-clients")]
pub mod tarpc_adapter;

// Re-exports
pub use http::HttpAdapter;
#[cfg(feature = "live-clients")]
pub use tarpc_adapter::TarpcAdapter;

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
    async fn call_json(&self, method: &str, args_json: String) -> Result<String>;

    /// Call a remote method without expecting a response (fire-and-forget).
    async fn call_oneway_json(&self, method: &str, args_json: String) -> Result<()>;

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
            .map_err(|e| RhizoCryptError::integration(format!("Failed to serialize args: {}", e)))?;
        
        let response_json = self.call_json(method, args_json).await?;
        
        serde_json::from_str(&response_json)
            .map_err(|e| RhizoCryptError::integration(format!("Failed to deserialize response: {}", e)))
    }

    /// Call a remote method without expecting a response.
    async fn call_oneway<Args>(&self, method: &str, args: Args) -> Result<()>
    where
        Args: Serialize + Send + Sync,
    {
        let args_json = serde_json::to_string(&args)
            .map_err(|e| RhizoCryptError::integration(format!("Failed to serialize args: {}", e)))?;
        
        self.call_oneway_json(method, args_json).await
    }
}

// Blanket implementation for all ProtocolAdapter implementors
impl<T: ProtocolAdapter + ?Sized> ProtocolAdapterExt for T {}

// ============================================================================
// Adapter Factory
// ============================================================================

/// Factory for creating protocol adapters.
pub struct AdapterFactory;

impl AdapterFactory {
    /// Create an adapter based on protocol hint in endpoint.
    ///
    /// # Arguments
    ///
    /// * `endpoint` - Service endpoint (e.g., "http://10.0.1.5:9500", "tarpc://10.0.1.6:9600")
    ///
    /// # Errors
    ///
    /// Returns error if:
    /// - Protocol is unsupported
    /// - Endpoint format is invalid
    /// - Adapter creation fails
    pub fn create(endpoint: &str) -> Result<Box<dyn ProtocolAdapter>> {
        // Parse protocol from endpoint
        if let Some((protocol, _)) = endpoint.split_once("://") {
            match protocol {
                "http" | "https" => Ok(Box::new(HttpAdapter::new(endpoint)?)),
                #[cfg(feature = "live-clients")]
                "tarpc" => Ok(Box::new(TarpcAdapter::new(endpoint)?)),
                unsupported => Err(RhizoCryptError::integration(format!(
                    "Unsupported protocol: {}. Supported: http, https{}",
                    unsupported,
                    if cfg!(feature = "live-clients") {
                        ", tarpc"
                    } else {
                        ""
                    }
                ))),
            }
        } else {
            // No protocol specified, try to infer from port or default to HTTP
            tracing::warn!(
                endpoint,
                "No protocol specified, defaulting to HTTP. \
                 Consider using explicit protocol: http://{}",
                endpoint
            );
            let http_endpoint = format!("http://{endpoint}");
            Ok(Box::new(HttpAdapter::new(&http_endpoint)?))
        }
    }

    /// Create an HTTP adapter.
    pub fn http(endpoint: &str) -> Result<Box<dyn ProtocolAdapter>> {
        Ok(Box::new(HttpAdapter::new(endpoint)?))
    }

    /// Create a tarpc adapter (requires `live-clients` feature).
    #[cfg(feature = "live-clients")]
    pub fn tarpc(endpoint: &str) -> Result<Box<dyn ProtocolAdapter>> {
        Ok(Box::new(TarpcAdapter::new(endpoint)?))
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_factory_http() {
        let adapter = AdapterFactory::create("http://localhost:9500").unwrap();
        assert_eq!(adapter.protocol(), "http");
    }

    #[test]
    fn test_factory_https() {
        let adapter = AdapterFactory::create("https://api.example.com:443").unwrap();
        assert_eq!(adapter.protocol(), "http"); // HttpAdapter handles both
    }

    #[test]
    fn test_factory_no_protocol_defaults_to_http() {
        let adapter = AdapterFactory::create("localhost:9500").unwrap();
        assert_eq!(adapter.protocol(), "http");
    }

    #[test]
    fn test_factory_unsupported_protocol() {
        let result = AdapterFactory::create("grpc://localhost:9500");
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("Unsupported protocol"));
    }

    #[cfg(feature = "live-clients")]
    #[test]
    fn test_factory_tarpc() {
        let adapter = AdapterFactory::create("tarpc://localhost:9600").unwrap();
        assert_eq!(adapter.protocol(), "tarpc");
    }
}

