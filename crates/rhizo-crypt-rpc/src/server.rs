// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2024–2026 ecoPrimals Project

//! RPC server implementation.

use crate::service::{RhizoCryptRpc, RhizoCryptRpcServer};
use futures_util::StreamExt;
use rhizo_crypt_core::RhizoCrypt;
use std::net::SocketAddr;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tarpc::server::{self, Channel};
use tarpc::tokio_serde::formats::Bincode;
use tokio::sync::watch;
use tracing::{info, warn};

/// RPC server wrapper.
///
/// Manages the tarpc server lifecycle with graceful shutdown support.
pub struct RpcServer {
    primal: Arc<RhizoCrypt>,
    addr: SocketAddr,
    shutdown_tx: watch::Sender<bool>,
    shutdown_rx: watch::Receiver<bool>,
    is_running: Arc<AtomicBool>,
}

impl RpcServer {
    /// Create a new RPC server.
    #[must_use]
    pub fn new(primal: Arc<RhizoCrypt>, addr: SocketAddr) -> Self {
        let (shutdown_tx, shutdown_rx) = watch::channel(false);
        Self {
            primal,
            addr,
            shutdown_tx,
            shutdown_rx,
            is_running: Arc::new(AtomicBool::new(false)),
        }
    }

    /// Start the RPC server.
    ///
    /// This will bind to the configured address and serve RPC requests.
    /// The server runs until `shutdown()` is called or the process is interrupted.
    ///
    /// # Errors
    ///
    /// Returns `std::io::Error` if binding to the address fails.
    pub async fn serve(self) -> Result<(), std::io::Error> {
        let listener = tarpc::serde_transport::tcp::listen(&self.addr, Bincode::default).await?;
        let local_addr = listener.local_addr();
        info!("rhizoCrypt RPC server listening on {}", local_addr);

        self.is_running.store(true, Ordering::SeqCst);
        let is_running = Arc::clone(&self.is_running);
        let mut shutdown_rx = self.shutdown_rx.clone();

        // Create a stream that stops on shutdown signal
        let incoming = listener.filter_map(|r| async { r.ok() });

        tokio::select! {
            () = incoming.for_each(|transport| {
                let server = RhizoCryptRpcServer::new(Arc::clone(&self.primal));

                async move {
                    let fut = server::BaseChannel::with_defaults(transport)
                        .execute(server.serve())
                        .for_each(|response| async move {
                            response.await;
                        });

                    tokio::spawn(fut);
                }
            }) => {}
            Ok(()) = shutdown_rx.changed() => {
                info!("rhizoCrypt RPC server shutting down gracefully");
            }
        }

        is_running.store(false, Ordering::SeqCst);
        info!("rhizoCrypt RPC server stopped");

        Ok(())
    }

    /// Signal the server to shut down gracefully.
    ///
    /// This will stop accepting new connections. Existing connections
    /// will be allowed to complete their current requests.
    pub fn shutdown(&self) {
        if self.shutdown_tx.send(true).is_err() {
            warn!("Server already shut down or shutdown channel closed");
        }
    }

    /// Get a clone of the shutdown sender for external signal handling.
    ///
    /// Call `send(true)` on the returned sender to trigger graceful shutdown.
    #[must_use]
    pub fn shutdown_sender(&self) -> watch::Sender<bool> {
        self.shutdown_tx.clone()
    }

    /// Check if the server is currently running.
    #[must_use]
    pub fn is_running(&self) -> bool {
        self.is_running.load(Ordering::SeqCst)
    }

    /// Get the server address.
    #[must_use]
    pub const fn addr(&self) -> SocketAddr {
        self.addr
    }

    /// Get a shutdown receiver for external monitoring.
    ///
    /// This can be used to wait for the server to shut down.
    #[must_use]
    pub fn shutdown_receiver(&self) -> watch::Receiver<bool> {
        self.shutdown_rx.clone()
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;
    use rhizo_crypt_core::{PrimalLifecycle, RhizoCryptConfig};
    use std::borrow::Cow;

    #[test]
    fn test_server_creation() {
        let config = RhizoCryptConfig::default();
        let addr = config.rpc.parse_addr().expect("default config should have valid addr");
        let primal = Arc::new(RhizoCrypt::new(config));

        let server = RpcServer::new(primal, addr);
        assert_eq!(server.addr(), addr);
        assert!(!server.is_running());
    }

    #[test]
    fn test_server_custom_port() {
        let mut config = RhizoCryptConfig::default();
        config.rpc.port = 9500;
        config.rpc.host = Cow::Borrowed("0.0.0.0");

        let addr = config.rpc.parse_addr().expect("parse custom addr");
        assert_eq!(addr.port(), 9500);
        assert_eq!(addr.ip().to_string(), "0.0.0.0");
    }

    #[test]
    fn test_shutdown_receiver() {
        let config = RhizoCryptConfig::default();
        let addr = config.rpc.parse_addr().expect("default config should have valid addr");
        let primal = Arc::new(RhizoCrypt::new(config));

        let server = RpcServer::new(primal, addr);
        let mut rx = server.shutdown_receiver();

        // Initially not shut down
        assert!(!*rx.borrow());

        // Signal shutdown
        server.shutdown();

        // Receiver should see the change
        assert!(rx.has_changed().unwrap_or(false) || *rx.borrow_and_update());
    }

    #[test]
    fn test_server_builder_options() {
        use rhizo_crypt_core::RpcConfig;

        // Test RpcConfig::with_addr
        let rpc = RpcConfig::with_addr("127.0.0.1", 9400);
        assert_eq!(rpc.port, 9400);
        assert!(rpc.enabled);
        assert_eq!(rpc.max_connections, 1000);

        // Test RpcConfig::localhost_auto (port 0 for OS assignment)
        let rpc = RpcConfig::localhost_auto();
        assert_eq!(rpc.port, 0);
        assert!(rpc.enabled);

        // Test RhizoCryptConfig builder options
        let config = RhizoCryptConfig::new("TestPrimal")
            .with_max_sessions(500)
            .with_gc_interval(std::time::Duration::from_secs(120));
        assert_eq!(config.name, "TestPrimal");
        assert_eq!(config.max_sessions, 500);
        assert_eq!(config.gc_interval, std::time::Duration::from_secs(120));

        // Create server with custom config
        let addr = config.rpc.parse_addr().expect("parse addr");
        let primal = Arc::new(RhizoCrypt::new(config));
        let server = RpcServer::new(primal, addr);
        assert_eq!(server.addr(), addr);
    }

    #[tokio::test]
    async fn test_server_start_binds_port() {
        let addr = "127.0.0.1:0".parse().expect("parse addr");
        let mut primal = RhizoCrypt::new(RhizoCryptConfig::default());
        primal.start().await.unwrap();
        let primal = Arc::new(primal);

        let server = RpcServer::new(primal, addr);
        let server_addr = server.addr();
        assert_eq!(server_addr.port(), 0);

        // Start server in background and immediately shutdown
        let server_handle = tokio::spawn(async move { server.serve().await });
        tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;

        // Server should have bound (we can't easily verify the port without more setup,
        // but serve() should not error immediately)
        drop(server_handle);
    }

    #[tokio::test]
    async fn test_server_with_metrics() {
        use crate::service::RhizoCryptRpcServer;

        let mut primal = RhizoCrypt::new(RhizoCryptConfig::default());
        primal.start().await.unwrap();
        let primal = Arc::new(primal);

        let server = RhizoCryptRpcServer::new(primal);
        let ctx = tarpc::context::current();
        let metrics = server.clone().metrics(ctx).await.unwrap();

        // All metrics are u64, verify they are accessible
        let _ = metrics.sessions_created;
        let _ = metrics.sessions_resolved;
        let _ = metrics.vertices_appended;
        let _ = metrics.queries_executed;
        let _ = metrics.slices_checked_out;
        let _ = metrics.dehydrations_completed;
    }

    #[test]
    fn test_server_config_defaults() {
        let config = RhizoCryptConfig::default();
        assert_eq!(config.name, rhizo_crypt_core::constants::PRIMAL_NAME);
        assert_eq!(config.max_sessions, 1000);

        let rpc = &config.rpc;
        assert!(rpc.enabled);
        assert_eq!(rpc.max_connections, 1000);

        let rpc_default = rhizo_crypt_core::RpcConfig::with_addr("127.0.0.1", 0);
        let addr = rpc_default.parse_addr().expect("parse");
        assert_eq!(addr.port(), 0);
        assert_eq!(addr.ip().to_string(), "127.0.0.1");
    }
}
