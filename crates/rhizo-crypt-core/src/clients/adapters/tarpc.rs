// ! tarpc adapter for capability clients
//!
//! This adapter allows capability-based clients (SigningProvider, StorageProvider, etc.)
//! to communicate with rhizoCrypt or other primals using the tarpc protocol.

use crate::error::{Result, RhizoCryptError};
use crate::clients::adapters::{ProtocolAdapter};
use async_trait::async_trait;
use serde_json::Value;
use std::sync::Arc;
use tokio::net::ToSocketAddrs;

/// tarpc protocol adapter.
///
/// Provides connectivity from capability-based clients to tarpc-based services.
///
/// ## Usage
///
/// ```rust,ignore
/// use rhizo_crypt_core::clients::adapters::tarpc::TarpcAdapter;
///
/// let adapter = TarpcAdapter::new("tarpc://localhost:7777").await?;
/// let result_json = adapter.call_json("create_session", args_json).await?;
/// ```
#[derive(Debug)]
pub struct TarpcAdapter {
    endpoint: String,
    // For a full implementation, this would hold a tarpc client connection
    // For now, this is a placeholder that demonstrates the structure
    _placeholder: Arc<()>,
}

impl TarpcAdapter {
    /// Create a new tarpc adapter.
    ///
    /// # Arguments
    ///
    /// * `endpoint` - Service endpoint (e.g., "tarpc://localhost:7777" or "localhost:7777")
    ///
    /// # Errors
    ///
    /// Returns error if:
    /// - Endpoint format is invalid
    /// - Connection cannot be established
    pub fn new(endpoint: &str) -> Result<Self> {
        // Strip protocol if present
        let addr = endpoint
            .strip_prefix("tarpc://")
            .unwrap_or(endpoint);

        // Validate address format (basic check)
        if !addr.contains(':') {
            return Err(RhizoCryptError::integration(format!(
                "Invalid tarpc endpoint: {endpoint}. Expected format: tarpc://host:port or host:port"
            )));
        }

        tracing::info!(endpoint = %addr, "Creating tarpc adapter");

        Ok(Self {
            endpoint: addr.to_string(),
            _placeholder: Arc::new(()),
        })
    }

    /// Connect to a tarpc service.
    ///
    /// # Arguments
    ///
    /// * `endpoint` - Service endpoint
    ///
    /// # Errors
    ///
    /// Returns error if connection fails
    pub async fn connect<A>(addr: A) -> Result<Self>
    where
        A: ToSocketAddrs + std::fmt::Display + Clone,
    {
        let endpoint = addr.to_string();
        
        tracing::info!(endpoint = %endpoint, "Connecting to tarpc service");

        // In a full implementation, this would:
        // 1. Open TCP connection
        // 2. Establish tarpc client
        // 3. Verify connectivity

        // For now, return a placeholder adapter
        Ok(Self {
            endpoint,
            _placeholder: Arc::new(()),
        })
    }
}

#[async_trait]
impl ProtocolAdapter for TarpcAdapter {
    fn protocol(&self) -> &str {
        "tarpc"
    }

    async fn call_json(&self, method: &str, args_json: String) -> Result<String> {
        tracing::debug!(
            method = %method,
            endpoint = %self.endpoint,
            "Calling tarpc method"
        );

        // Parse args for logging
        let _args: Value = serde_json::from_str(&args_json)
            .map_err(|e| RhizoCryptError::integration(format!("Invalid JSON args: {e}")))?;

        // In a full implementation, this would:
        // 1. Route method to appropriate tarpc RPC call
        // 2. Serialize args using bincode (tarpc default)
        // 3. Make RPC call
        // 4. Deserialize response
        // 5. Convert to JSON for uniform interface

        // For now, return a placeholder indicating the adapter is present
        // but requires full tarpc client implementation
        Err(RhizoCryptError::integration(format!(
            "tarpc adapter present but not fully implemented. \
             Method: {method}, Endpoint: {}. \
             To complete: implement tarpc client connection and method routing.",
            self.endpoint
        )))
    }

    async fn call_oneway_json(&self, method: &str, args_json: String) -> Result<()> {
        tracing::debug!(
            method = %method,
            endpoint = %self.endpoint,
            "Calling tarpc method (oneway)"
        );

        // Parse args for validation
        let _args: Value = serde_json::from_str(&args_json)
            .map_err(|e| RhizoCryptError::integration(format!("Invalid JSON args: {e}")))?;

        // In a full implementation, this would:
        // 1. Make fire-and-forget RPC call
        // 2. Not wait for response

        // For now, placeholder
        Err(RhizoCryptError::integration(format!(
            "tarpc adapter oneway calls not fully implemented. \
             Method: {method}, Endpoint: {}",
            self.endpoint
        )))
    }

    async fn is_healthy(&self) -> bool {
        // In a full implementation, this would:
        // 1. Call health endpoint or ping
        // 2. Return true if service responds

        // For now, return false to indicate incomplete implementation
        tracing::warn!(
            endpoint = %self.endpoint,
            "tarpc health check not implemented"
        );
        false
    }

    fn endpoint(&self) -> &str {
        &self.endpoint
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_with_protocol() {
        let adapter = TarpcAdapter::new("tarpc://localhost:7777").unwrap();
        assert_eq!(adapter.endpoint(), "localhost:7777");
        assert_eq!(adapter.protocol(), "tarpc");
    }

    #[test]
    fn test_new_without_protocol() {
        let adapter = TarpcAdapter::new("localhost:7777").unwrap();
        assert_eq!(adapter.endpoint(), "localhost:7777");
        assert_eq!(adapter.protocol(), "tarpc");
    }

    #[test]
    fn test_new_invalid_format() {
        let result = TarpcAdapter::new("invalid-no-port");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Invalid tarpc endpoint"));
    }

    #[tokio::test]
    async fn test_is_healthy_returns_false_for_incomplete_impl() {
        let adapter = TarpcAdapter::new("localhost:7777").unwrap();
        assert!(!adapter.is_healthy().await);
    }

    #[tokio::test]
    async fn test_call_json_returns_incomplete_error() {
        let adapter = TarpcAdapter::new("localhost:7777").unwrap();
        let result = adapter.call_json("test_method", "{}".to_string()).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("not fully implemented"));
    }
}

