// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2024–2026 ecoPrimals Project

//! tarpc adapter for capability clients
//!
//! This adapter allows capability-based clients (SigningProvider, StorageProvider, etc.)
//! to communicate with rhizoCrypt or other primals using the tarpc protocol.

use crate::clients::adapters::ProtocolAdapter;
use crate::constants::CONNECTION_TIMEOUT;
use crate::error::{Result, RhizoCryptError};
use async_trait::async_trait;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;

#[cfg(feature = "live-clients")]
use serde_json::Value;
#[cfg(feature = "live-clients")]
use tokio::time::timeout;

// ============================================================================
// ecoPrimals JSON-RPC tarpc Trait (live-clients only)
// ============================================================================

/// Standard JSON-RPC tarpc trait for ecoPrimals interop.
///
/// Any primal that implements this trait can interoperate with the TarpcAdapter.
/// The adapter calls `call(method, params)` with JSON-serialized arguments and
/// receives a JSON-serialized response.
#[cfg(feature = "live-clients")]
#[tarpc::service]
pub trait EcoPrimalJsonRpc {
    /// Execute a JSON-RPC style call.
    ///
    /// # Arguments
    ///
    /// * `method` - Method name (e.g., "sign", "store", "commit")
    /// * `params` - JSON-serialized parameters
    ///
    /// # Returns
    ///
    /// JSON-serialized result on success, or error string on failure.
    async fn call(method: String, params: String) -> std::result::Result<String, String>;
}

// ============================================================================
// TarpcConnection (feature-gated)
// ============================================================================

/// Internal connection wrapper holding the tarpc client handle.
#[cfg(feature = "live-clients")]
#[derive(Debug)]
struct TarpcConnection {
    /// Spawned tarpc client for EcoPrimalJsonRpc.
    client: EcoPrimalJsonRpcClient,
}

/// Stub connection when live-clients feature is disabled.
#[cfg(not(feature = "live-clients"))]
#[derive(Debug, Clone)]
struct TarpcConnection {
    /// Placeholder for stub mode.
    _stub: (),
}

// ============================================================================
// TarpcAdapter
// ============================================================================

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
/// - Works with ANY tarpc service that implements `EcoPrimalJsonRpc`
/// - Capability-based: doesn't hardcode specific service types
/// - Discovery-driven: services found at runtime
///
/// ## Usage
///
/// ```no_run
/// # use rhizo_crypt_core::clients::adapters::{tarpc::TarpcAdapter, ProtocolAdapter};
/// # tokio::runtime::Runtime::new().unwrap().block_on(async {
/// let adapter = TarpcAdapter::new("localhost:7777")?;
/// let args_json = r#"{"data":[],"signer":"did:key:test"}"#.to_string();
/// let _result_json = adapter.call_json("sign", args_json).await?;
/// # Ok::<(), rhizo_crypt_core::error::RhizoCryptError>(())
/// # });
/// ```
///
/// ## Feature Gate
///
/// When the `live-clients` feature is disabled, `call_json` and `call_oneway_json`
/// return an error indicating the feature is required. Adapter creation and
/// health checks still work.
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
    /// Connection state (lazy-initialized, cached).
    connection: Arc<RwLock<Option<TarpcConnection>>>,
    /// Timeout for RPC calls.
    pub timeout_duration: Duration,
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
        let addr: SocketAddr = if let Ok(addr) = addr_str.parse() {
            addr
        } else {
            // Not an IP address, try to resolve hostname
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
            timeout_duration: CONNECTION_TIMEOUT,
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
            .map_err(|e| {
                RhizoCryptError::integration(format!("Failed to resolve tarpc address: {e}"))
            })?
            .next()
            .ok_or_else(|| {
                RhizoCryptError::integration("No addresses resolved for tarpc endpoint")
            })?;

        tracing::info!(endpoint = %socket_addr, "Connecting to tarpc service");

        Ok(Self {
            endpoint: socket_addr.to_string(),
            addr: socket_addr,
            connection: Arc::new(RwLock::new(None)),
            timeout_duration: CONNECTION_TIMEOUT,
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
            #[cfg(feature = "live-clients")]
            {
                tracing::debug!(endpoint = %self.addr, "Establishing tarpc connection");

                let transport = tarpc::serde_transport::tcp::connect(
                    self.addr,
                    tarpc::tokio_serde::formats::Bincode::default,
                )
                .await
                .map_err(|e| {
                    RhizoCryptError::integration(format!(
                        "Failed to connect to tarpc service at {}: {e}",
                        self.addr
                    ))
                })?;

                let client =
                    EcoPrimalJsonRpcClient::new(tarpc::client::Config::default(), transport)
                        .spawn();

                *conn = Some(TarpcConnection {
                    client,
                });

                tracing::info!(endpoint = %self.addr, "tarpc connection established");
            }

            #[cfg(not(feature = "live-clients"))]
            {
                tracing::debug!(endpoint = %self.addr, "tarpc stub connection (live-clients disabled)");
                *conn = Some(TarpcConnection {
                    _stub: (),
                });
            }
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
        #[cfg(not(feature = "live-clients"))]
        {
            let _ = (method, args_json);
            return Err(RhizoCryptError::integration(
                "tarpc support requires 'live-clients' feature",
            ));
        }

        #[cfg(feature = "live-clients")]
        {
            self.ensure_connected().await?;

            // Parse args for validation
            let _args: Value = serde_json::from_str(&args_json)
                .map_err(|e| RhizoCryptError::integration(format!("Invalid JSON args: {e}")))?;

            tracing::debug!(
                method = %method,
                endpoint = %self.addr,
                "Calling tarpc method"
            );

            let method = method.to_string();
            let conn = self.connection.read().await;
            let tarpc_conn = conn
                .as_ref()
                .ok_or_else(|| RhizoCryptError::integration("tarpc connection lost"))?;

            let call_future = tarpc_conn.client.call(
                tarpc::context::current(),
                method.clone(),
                args_json.clone(),
            );

            let result = timeout(self.timeout_duration, call_future).await.map_err(|_| {
                RhizoCryptError::integration(format!(
                    "tarpc call timed out after {:?}: method={}, endpoint={}",
                    self.timeout_duration, method, self.addr
                ))
            })?;

            result
                .map_err(|e| RhizoCryptError::integration(format!("tarpc call failed: {e}")))?
                .map_err(|e| RhizoCryptError::integration(format!("Remote error: {e}")))
        }
    }

    async fn call_oneway_json(&self, method: &str, args_json: String) -> Result<()> {
        #[cfg(not(feature = "live-clients"))]
        {
            let _ = (method, args_json);
            return Err(RhizoCryptError::integration(
                "tarpc support requires 'live-clients' feature",
            ));
        }

        #[cfg(feature = "live-clients")]
        {
            // EcoPrimalJsonRpc has no oneway method; fire-and-forget by spawning
            // and discarding the response future.
            self.ensure_connected().await?;

            let _args: Value = serde_json::from_str(&args_json)
                .map_err(|e| RhizoCryptError::integration(format!("Invalid JSON args: {e}")))?;

            tracing::debug!(
                method = %method,
                endpoint = %self.addr,
                "Calling tarpc method (oneway)"
            );

            let method = method.to_string();
            let conn = self.connection.read().await;
            let tarpc_conn = conn
                .as_ref()
                .ok_or_else(|| RhizoCryptError::integration("tarpc connection lost"))?;

            // Clone client handle for spawn; tarpc client is a cheap cloneable handle.
            let client = tarpc_conn.client.clone();
            tokio::spawn(async move {
                let _ = client.call(tarpc::context::current(), method, args_json).await;
            });

            Ok(())
        }
    }

    async fn is_healthy(&self) -> bool {
        match self.ensure_connected().await {
            Ok(()) => {
                tracing::debug!(endpoint = %self.addr, "tarpc adapter connection healthy");
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
        let adapter =
            TarpcAdapter::new("127.0.0.1:7777").unwrap().with_timeout(Duration::from_secs(5));
        assert_eq!(adapter.timeout_duration, Duration::from_secs(5));
    }

    #[tokio::test]
    async fn test_ensure_connected() {
        let adapter = TarpcAdapter::new("127.0.0.1:7777").unwrap();
        // When live-clients disabled: stub connection; when enabled: real TCP (may fail)
        let result = adapter.ensure_connected().await;
        #[cfg(feature = "live-clients")]
        {
            // With live-clients, connection may fail (no server) or succeed
            let _ = result;
        }
        #[cfg(not(feature = "live-clients"))]
        {
            assert!(result.is_ok());
            assert!(adapter.connection.read().await.is_some());
        }
    }

    #[tokio::test]
    async fn test_is_healthy() {
        let adapter = TarpcAdapter::new("127.0.0.1:7777").unwrap();
        #[cfg(not(feature = "live-clients"))]
        {
            assert!(adapter.is_healthy().await);
        }
        #[cfg(feature = "live-clients")]
        {
            // With live-clients, health depends on whether 127.0.0.1:7777 is reachable
            let _ = adapter.is_healthy().await;
        }
    }

    #[tokio::test]
    async fn test_call_json_feature_gated() {
        let adapter = TarpcAdapter::new("127.0.0.1:7777").unwrap();
        let result = adapter.call_json("test_method", "{}".to_string()).await;
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        #[cfg(not(feature = "live-clients"))]
        assert!(err.contains("live-clients"), "expected feature-gate error: {err}");
        #[cfg(feature = "live-clients")]
        {
            // With live-clients: connection error or timeout (no server)
            assert!(
                err.contains("live-clients")
                    || err.contains("connect")
                    || err.contains("timed out")
                    || err.contains("tarpc"),
                "expected connection/timeout/feature error: {err}"
            );
        }
    }

    #[tokio::test]
    async fn test_call_json_timeout() {
        let adapter =
            TarpcAdapter::new("127.0.0.1:7777").unwrap().with_timeout(Duration::from_millis(1));
        let result = adapter.call_json("test_method", "{}".to_string()).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_connect_async() {
        let result = TarpcAdapter::connect("127.0.0.1:7777").await;
        assert!(result.is_ok());
        let adapter = result.unwrap();
        assert_eq!(adapter.protocol(), "tarpc");
    }
}
