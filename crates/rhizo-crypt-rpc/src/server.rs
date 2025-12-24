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
    use rhizo_crypt_core::RhizoCryptConfig;

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
        use std::borrow::Cow;

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
}
