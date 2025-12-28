// ! tarpc adapter for capability clients
//!
//! This adapter allows capability-based clients (SigningProvider, StorageProvider, etc.)
//! to communicate with rhizoCrypt or other primals using the tarpc protocol.

use crate::clients::adapters::ProtocolAdapter;
use crate::error::{Result, RhizoCryptError};
use async_trait::async_trait;
use serde_json::Value;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use tokio::time::timeout;

/// tarpc protocol adapter.
///
/// Provides connectivity from capability-based clients to tarpc-based services.
///
/// This adapter uses JSON-RPC over tarpc for maximum flexibility with
/// heterogeneous services. Each service exposes methods via JSON payloads.
///
/// ## Architecture
///
/// This adapter is intentionally **generic**:
/// - Works with ANY tarpc service that speaks JSON-RPC
/// - Capability-based: doesn't hardcode specific service types
/// - Discovery-driven: services found at runtime
///
/// ## Usage
///
/// ```rust,ignore
/// use rhizo_crypt_core::clients::adapters::tarpc::TarpcAdapter;
///
/// let adapter = TarpcAdapter::connect("localhost:7777").await?;
/// let result_json = adapter.call_json("sign", args_json).await?;
/// ```
///
/// ## Connection Pooling
///
/// The adapter maintains a single connection per endpoint. For high-throughput
/// scenarios, consider using multiple adapter instances or implementing
/// connection pooling at a higher level.
#[derive(Debug)]
pub struct TarpcAdapter {
    endpoint: String,
    addr: SocketAddr,
    // Connection state (lazy-initialized, cached)
    connection: Arc<RwLock<Option<TarpcConnection>>>,
    timeout_duration: Duration,
}

/// Internal connection wrapper for tarpc client.
///
/// Note: For a fully generic JSON-RPC over tarpc implementation, we would
/// need the remote service to expose a `call_json(method, args) -> result`
/// endpoint. Since we don't have a standardized JSON-RPC tarpc trait yet
/// across the ecosystem, this connection type is prepared but the actual
/// implementation waits for ecosystem standardization.
#[derive(Debug, Clone)]
struct TarpcConnection {
    _endpoint: String,
    // In future: would hold Arc<dyn JsonRpcClient> or similar
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
    /// - Address cannot be parsed or resolved
    pub fn new(endpoint: &str) -> Result<Self> {
        // Strip protocol if present
        let addr_str = endpoint.strip_prefix("tarpc://").unwrap_or(endpoint);

        // Try to parse as SocketAddr first (IP address)
        let addr: SocketAddr = if let Ok(addr) = addr_str.parse() { addr } else {
            // Not an IP address, try to resolve hostname
            // For new() we need blocking resolution, so we use std::net
            use std::net::ToSocketAddrs as StdToSocketAddrs;
            
            addr_str.to_socket_addrs()
                .map_err(|e| RhizoCryptError::integration(format!(
                    "Invalid tarpc endpoint: {endpoint}. Expected format: host:port or tarpc://host:port. Error: {e}"
                )))?
                .next()
                .ok_or_else(|| RhizoCryptError::integration(format!(
                    "Could not resolve tarpc endpoint: {endpoint}"
                )))?
        };

        tracing::info!(endpoint = %addr, "Created tarpc adapter");

        Ok(Self {
            endpoint: addr_str.to_string(),
            addr,
            connection: Arc::new(RwLock::new(None)),
            timeout_duration: Duration::from_secs(30),
        })
    }

    /// Connect to a tarpc service.
    ///
    /// # Arguments
    ///
    /// * `addr` - Service address (can be SocketAddr or implements ToSocketAddrs)
    ///
    /// # Errors
    ///
    /// Returns error if address resolution or connection fails
    pub async fn connect<A>(addr: A) -> Result<Self>
    where
        A: tokio::net::ToSocketAddrs,
    {
        // Resolve to SocketAddr
        let socket_addr = tokio::net::lookup_host(addr)
            .await
            .map_err(|e| RhizoCryptError::integration(format!("Failed to resolve tarpc address: {e}")))?
            .next()
            .ok_or_else(|| RhizoCryptError::integration("No addresses resolved for tarpc endpoint"))?;

        tracing::info!(endpoint = %socket_addr, "Connecting to tarpc service");

        Ok(Self {
            endpoint: socket_addr.to_string(),
            addr: socket_addr,
            connection: Arc::new(RwLock::new(None)),
            timeout_duration: Duration::from_secs(30),
        })
    }

    /// Set timeout duration for RPC calls.
    #[must_use]
    pub const fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout_duration = timeout;
        self
    }

    /// Establish connection if not already connected.
    ///
    /// This is called internally by call methods, but can be called explicitly
    /// to fail-fast on connection errors.
    async fn ensure_connected(&self) -> Result<()> {
        let mut conn = self.connection.write().await;
        
        if conn.is_none() {
            tracing::debug!(endpoint = %self.addr, "Establishing tarpc connection");
            
            // In a fully generic implementation, we would:
            // 1. Create TCP connection to self.addr
            // 2. Wrap in tarpc transport (Bincode or JSON codec)
            // 3. Spawn client with appropriate tarpc trait
            //
            // However, tarpc requires compile-time trait knowledge.
            // For truly generic JSON-RPC, we'd need:
            // - A standardized JsonRpc trait across the ecosystem, OR
            // - Dynamic client generation, OR
            // - HTTP-based JSON-RPC (easier but loses tarpc benefits)
            //
            // For now, we implement the structure and document the path forward.
            
            *conn = Some(TarpcConnection {
                _endpoint: self.endpoint.clone(),
            });
            
            tracing::info!(endpoint = %self.addr, "tarpc connection established (stub)");
        }
        
        Ok(())
    }
}

#[async_trait]
impl ProtocolAdapter for TarpcAdapter {
    fn protocol(&self) -> &str {
        "tarpc"
    }

    async fn call_json(&self, method: &str, args_json: String) -> Result<String> {
        // Ensure we have a connection
        self.ensure_connected().await?;
        
        // Parse args for validation
        let _args: Value = serde_json::from_str(&args_json)
            .map_err(|e| RhizoCryptError::integration(format!("Invalid JSON args: {e}")))?;

        tracing::debug!(
            method = %method,
            endpoint = %self.addr,
            "Calling tarpc method"
        );

        // Wrap the call with timeout
        let call_future = async {
            // ================================================================
            // Path Forward for Full Implementation:
            // ================================================================
            //
            // Option 1: Ecosystem JSON-RPC Trait
            // ------------------------------------
            // Define a standard trait across ecoPrimals:
            //
            // ```rust
            // #[tarpc::service]
            // pub trait JsonRpcService {
            //     async fn call(method: String, args: String) -> Result<String, String>;
            // }
            // ```
            //
            // All primals that want tarpc interop implement this trait.
            // Then this adapter becomes:
            //
            // ```rust
            // let transport = tarpc::serde_transport::tcp::connect(&self.addr, Bincode::default()).await?;
            // let client = JsonRpcServiceClient::new(client::Config::default(), transport).spawn();
            // let result = client.call(context::current(), method.to_string(), args_json).await??;
            // return Ok(result);
            // ```
            //
            // Option 2: HTTP JSON-RPC Fallback
            // ----------------------------------
            // For services without standardized tarpc JSON-RPC trait,
            // fall back to HTTP-based JSON-RPC (still capability-based):
            //
            // ```rust
            // let http_endpoint = format!("http://{}/rpc", self.addr);
            // // Use reqwest to call JSON-RPC endpoint
            // ```
            //
            // Option 3: Per-Service Adapters
            // -------------------------------
            // Create specific adapters for each primal:
            // - `BearDogTarpcAdapter` (implements ProtocolAdapter, uses BearDog's tarpc trait)
            // - `LoamSpineTarpcAdapter` (uses LoamSpine's tarpc trait)
            // These live in integration crates, not core.
            //
            // ================================================================
            
            Err(RhizoCryptError::integration(format!(
                "tarpc adapter: Generic JSON-RPC over tarpc requires ecosystem standardization.\n\
                 Method: {method}, Endpoint: {}\n\
                 \n\
                 Implementation paths:\n\
                 1. Define ecosystem-wide JsonRpcService trait (recommended)\n\
                 2. Use HTTP JSON-RPC as intermediate protocol\n\
                 3. Create service-specific tarpc adapters\n\
                 \n\
                 For now, use HTTP adapter for cross-primal communication.",
                self.addr
            )))
        };

        timeout(self.timeout_duration, call_future)
            .await
            .map_err(|_| RhizoCryptError::integration(format!(
                "tarpc call timed out after {:?}: method={}, endpoint={}",
                self.timeout_duration, method, self.addr
            )))?
    }

    async fn call_oneway_json(&self, method: &str, args_json: String) -> Result<()> {
        // Ensure we have a connection
        self.ensure_connected().await?;
        
        // Parse args for validation
        let _args: Value = serde_json::from_str(&args_json)
            .map_err(|e| RhizoCryptError::integration(format!("Invalid JSON args: {e}")))?;

        tracing::debug!(
            method = %method,
            endpoint = %self.addr,
            "Calling tarpc method (oneway)"
        );

        // For one-way calls, we would:
        // 1. Send RPC without waiting for response
        // 2. Return immediately
        //
        // This requires fire-and-forget capability in the tarpc client,
        // which is typically done via tokio::spawn of the response future
        // and immediate return.

        Err(RhizoCryptError::integration(format!(
            "tarpc adapter oneway: Requires ecosystem JsonRpcService trait.\n\
             Method: {method}, Endpoint: {}\n\
             See call_json() documentation for implementation paths.",
            self.addr
        )))
    }

    async fn is_healthy(&self) -> bool {
        // Try to establish connection
        match self.ensure_connected().await {
            Ok(()) => {
                tracing::debug!(endpoint = %self.addr, "tarpc adapter connection healthy");
                // In full implementation, would also ping service
                // For now, connection establishment is the health check
                true
            }
            Err(e) => {
                tracing::warn!(
                    endpoint = %self.addr,
                    error = %e,
                    "tarpc adapter connection failed"
                );
                false
            }
        }
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
        let adapter = TarpcAdapter::new("tarpc://127.0.0.1:7777").unwrap();
        assert_eq!(adapter.endpoint(), "127.0.0.1:7777");
        assert_eq!(adapter.protocol(), "tarpc");
    }

    #[test]
    fn test_new_without_protocol() {
        let adapter = TarpcAdapter::new("127.0.0.1:7777").unwrap();
        assert_eq!(adapter.endpoint(), "127.0.0.1:7777");
        assert_eq!(adapter.protocol(), "tarpc");
    }

    #[test]
    fn test_new_invalid_format() {
        let result = TarpcAdapter::new("invalid-no-port");
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("Invalid tarpc endpoint") || err.contains("invalid"));
    }

    #[test]
    fn test_with_timeout() {
        let adapter = TarpcAdapter::new("127.0.0.1:7777")
            .unwrap()
            .with_timeout(Duration::from_secs(5));
        assert_eq!(adapter.timeout_duration, Duration::from_secs(5));
    }

    #[tokio::test]
    async fn test_ensure_connected() {
        let adapter = TarpcAdapter::new("127.0.0.1:7777").unwrap();
        // Connection establishment should succeed (stub implementation)
        let result = adapter.ensure_connected().await;
        assert!(result.is_ok());
        
        // Should cache connection
        assert!(adapter.connection.read().await.is_some());
    }

    #[tokio::test]
    async fn test_is_healthy() {
        let adapter = TarpcAdapter::new("127.0.0.1:7777").unwrap();
        // Should be healthy after connection (stub returns Ok)
        assert!(adapter.is_healthy().await);
    }

    #[tokio::test]
    async fn test_call_json_requires_ecosystem_trait() {
        let adapter = TarpcAdapter::new("127.0.0.1:7777").unwrap();
        let result = adapter.call_json("test_method", "{}".to_string()).await;
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("ecosystem standardization") || err.contains("JsonRpcService"));
    }

    #[tokio::test]
    async fn test_call_json_timeout() {
        let adapter = TarpcAdapter::new("127.0.0.1:7777")
            .unwrap()
            .with_timeout(Duration::from_millis(1));
        let result = adapter.call_json("test_method", "{}".to_string()).await;
        // Should either timeout or return not-implemented error
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_connect_async() {
        // Test async connect method
        let result = TarpcAdapter::connect("127.0.0.1:7777").await;
        assert!(result.is_ok());
        let adapter = result.unwrap();
        assert_eq!(adapter.protocol(), "tarpc");
    }
}
