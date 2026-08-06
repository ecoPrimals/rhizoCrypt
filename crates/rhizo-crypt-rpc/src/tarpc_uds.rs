// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2024–2026 ecoPrimals Project

//! tarpc binary UDS server (G64 C2 dual-socket pattern).
//!
//! Serves the tarpc binary protocol over a Unix domain socket at
//! `{primal}.tarpc.sock` alongside the JSON-RPC socket at `{primal}.sock`.
//! This eliminates serde/JSON overhead for intra-gate primal-to-primal
//! composition while keeping JSON-RPC for discovery and diagnostics.

use crate::service::{RhizoCryptRpc, RhizoCryptRpcServer};
use futures_util::StreamExt;
use rhizo_crypt_core::RhizoCrypt;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use tarpc::server::{self, Channel};
use tarpc::tokio_serde::formats::Bincode;
use tokio::sync::watch;
use tracing::{debug, info, warn};

/// tarpc binary UDS server.
///
/// Manages the tarpc binary listener lifecycle on a `.tarpc.sock` path,
/// serving the same [`RhizoCryptRpc`] trait as the TCP server but over
/// a length-delimited bincode UDS transport for sub-ms latency.
pub struct TarpcUdsServer {
    primal: Arc<RhizoCrypt>,
    socket_path: PathBuf,
    shutdown_tx: watch::Sender<bool>,
    shutdown_rx: watch::Receiver<bool>,
    is_running: Arc<AtomicBool>,
    ready_notify: Arc<tokio::sync::Notify>,
}

impl TarpcUdsServer {
    /// Create a new tarpc UDS server that will bind to `socket_path`.
    #[must_use]
    pub fn new(primal: Arc<RhizoCrypt>, socket_path: PathBuf) -> Self {
        let (shutdown_tx, shutdown_rx) = watch::channel(false);
        Self {
            primal,
            socket_path,
            shutdown_tx,
            shutdown_rx,
            is_running: Arc::new(AtomicBool::new(false)),
            ready_notify: Arc::new(tokio::sync::Notify::new()),
        }
    }

    /// Start serving tarpc over the Unix socket.
    ///
    /// Creates parent directories, removes any stale socket file, binds the
    /// listener, and accepts connections until `shutdown()` is called.
    ///
    /// # Errors
    ///
    /// Returns `std::io::Error` if directory creation, socket removal, or
    /// binding fails.
    pub async fn serve(self) -> Result<(), std::io::Error> {
        self.prepare_socket_path()?;

        let listener =
            tarpc::serde_transport::unix::listen(&self.socket_path, Bincode::default).await?;
        info!(path = %self.socket_path.display(), "tarpc binary UDS listening (C2 dual-socket)");

        self.is_running.store(true, Ordering::SeqCst);
        self.ready_notify.notify_one();
        let is_running = Arc::clone(&self.is_running);
        let mut shutdown_rx = self.shutdown_rx.clone();

        let shared_server = RhizoCryptRpcServer::new(Arc::clone(&self.primal));
        let incoming = listener.filter_map(|r| async { r.ok() });

        tokio::select! {
            () = incoming.for_each(|transport| {
                let server = shared_server.clone();

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
                info!("tarpc UDS server shutting down gracefully");
            }
        }

        is_running.store(false, Ordering::SeqCst);
        self.cleanup();
        info!("tarpc UDS server stopped");

        Ok(())
    }

    /// Signal the server to shut down gracefully.
    pub fn shutdown(&self) {
        if self.shutdown_tx.send(true).is_err() {
            warn!("tarpc UDS server already shut down or shutdown channel closed");
        }
    }

    /// Get a clone of the shutdown sender for external signal handling.
    #[must_use]
    pub fn shutdown_sender(&self) -> watch::Sender<bool> {
        self.shutdown_tx.clone()
    }

    /// Check if the server is currently running.
    #[inline]
    #[must_use]
    pub fn is_running(&self) -> bool {
        self.is_running.load(Ordering::SeqCst)
    }

    /// Wait until the server has bound its socket and is accepting connections.
    pub async fn wait_ready(&self) {
        if self.is_running() {
            return;
        }
        self.ready_notify.notified().await;
    }

    /// Get a cloneable readiness notifier.
    #[must_use]
    pub fn ready_notifier(&self) -> Arc<tokio::sync::Notify> {
        Arc::clone(&self.ready_notify)
    }

    /// Get the socket path.
    #[must_use]
    pub fn socket_path(&self) -> &Path {
        &self.socket_path
    }

    fn prepare_socket_path(&self) -> Result<(), std::io::Error> {
        if let Some(parent) = self.socket_path.parent()
            && !parent.exists()
        {
            std::fs::create_dir_all(parent)?;
        }
        if self.socket_path.exists() {
            std::fs::remove_file(&self.socket_path)?;
        }
        Ok(())
    }

    fn cleanup(&self) {
        if self.socket_path.exists() {
            if let Err(e) = std::fs::remove_file(&self.socket_path) {
                warn!(path = %self.socket_path.display(), error = %e, "failed to clean up tarpc UDS socket");
            } else {
                debug!(path = %self.socket_path.display(), "cleaned up tarpc UDS socket");
            }
        }
    }
}

/// Resolve the default tarpc UDS path for rhizoCrypt.
///
/// Follows the same family-scoped directory chain as
/// [`crate::jsonrpc::uds::default_socket_path`] but uses `.tarpc.sock`.
#[must_use]
pub fn default_tarpc_socket_path() -> PathBuf {
    use rhizo_crypt_core::constants::{BIOMEOS_SOCKET_SUBDIR, TARPC_SOCKET_FILE_EXTENSION};
    use rhizo_crypt_core::transport::family_scoped_tarpc_socket_path;

    let id = rhizo_crypt_core::niche::PRIMAL_ID;
    family_scoped_tarpc_socket_path(id, rhizo_crypt_core::niche::ENV_PREFIX).unwrap_or_else(|| {
        std::env::temp_dir()
            .join(BIOMEOS_SOCKET_SUBDIR)
            .join(format!("{id}{TARPC_SOCKET_FILE_EXTENSION}"))
    })
}

#[cfg(test)]
#[expect(clippy::unwrap_used, reason = "test code")]
mod tests {
    use super::*;
    use rhizo_crypt_core::{PrimalLifecycle, RhizoCryptConfig};
    use tarpc::tokio_serde::formats::Bincode;

    #[test]
    fn test_default_tarpc_socket_path_has_tarpc_extension() {
        let path = default_tarpc_socket_path();
        let name = path.file_name().unwrap().to_str().unwrap();
        assert!(name.ends_with(".tarpc.sock"), "expected .tarpc.sock, got {name}");
        assert!(name.starts_with("rhizocrypt"), "expected rhizocrypt prefix, got {name}");
    }

    #[tokio::test]
    async fn test_tarpc_uds_roundtrip() {
        let dir = tempfile::tempdir().unwrap();
        let sock = dir.path().join("test.tarpc.sock");

        let mut primal = RhizoCrypt::new(RhizoCryptConfig::default());
        primal.start().await.unwrap();
        let primal = Arc::new(primal);

        let server = TarpcUdsServer::new(Arc::clone(&primal), sock.clone());
        let ready = server.ready_notifier();
        let shutdown = server.shutdown_sender();
        let server_handle = tokio::spawn(async move { server.serve().await });

        ready.notified().await;

        let transport =
            tarpc::serde_transport::unix::connect(&sock, Bincode::default).await.unwrap();
        let client =
            crate::service::RhizoCryptRpcClient::new(tarpc::client::Config::default(), transport)
                .spawn();

        let health = client.health(tarpc::context::current()).await.unwrap().unwrap();
        assert!(health.healthy);

        let _ = shutdown.send(true);
        let _ = server_handle.await;
    }

    #[tokio::test]
    async fn test_tarpc_uds_multiple_operations() {
        use crate::service_types::CreateSessionRequest;
        use rhizo_crypt_core::{EventType, SessionType};

        let dir = tempfile::tempdir().unwrap();
        let sock = dir.path().join("multi.tarpc.sock");

        let mut primal = RhizoCrypt::new(RhizoCryptConfig::default());
        primal.start().await.unwrap();
        let primal = Arc::new(primal);

        let server = TarpcUdsServer::new(Arc::clone(&primal), sock.clone());
        let ready = server.ready_notifier();
        let shutdown = server.shutdown_sender();
        let server_handle = tokio::spawn(async move { server.serve().await });

        ready.notified().await;

        let transport =
            tarpc::serde_transport::unix::connect(&sock, Bincode::default).await.unwrap();
        let client =
            crate::service::RhizoCryptRpcClient::new(tarpc::client::Config::default(), transport)
                .spawn();

        let session_id = client
            .create_session(
                tarpc::context::current(),
                CreateSessionRequest {
                    session_type: SessionType::General,
                    description: None,
                    parent_session: None,
                    max_vertices: None,
                    ttl_seconds: None,
                },
            )
            .await
            .unwrap()
            .unwrap();

        let event_id = client
            .append_event(
                tarpc::context::current(),
                crate::service_types::AppendEventRequest {
                    session_id,
                    event_type: EventType::SessionStart,
                    agent: None,
                    parents: vec![],
                    metadata: vec![],
                    payload_ref: None,
                },
            )
            .await
            .unwrap()
            .unwrap();

        let sessions = client.list_sessions(tarpc::context::current()).await.unwrap().unwrap();
        assert!(!sessions.is_empty());

        let session_info =
            client.get_session(tarpc::context::current(), session_id).await.unwrap().unwrap();
        assert_eq!(session_info.vertex_count, 1);

        let merkle =
            client.get_merkle_root(tarpc::context::current(), session_id).await.unwrap().unwrap();
        assert!(!merkle.to_string().is_empty());

        let mut vertex = client
            .get_vertex(tarpc::context::current(), session_id, event_id)
            .await
            .unwrap()
            .unwrap();
        assert_eq!(vertex.id().unwrap(), event_id);

        let _ = shutdown.send(true);
        let _ = server_handle.await;
    }
}
