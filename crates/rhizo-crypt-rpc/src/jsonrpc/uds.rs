// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2024–2026 ecoPrimals Project

//! Unix domain socket JSON-RPC 2.0 listener.
//!
//! Binds a `UnixListener` at the ecosystem-standard socket path
//! (`$XDG_RUNTIME_DIR/biomeos/rhizocrypt.sock`) and serves newline-delimited
//! JSON-RPC connections using the shared handler from [`super::newline`].
//!
//! Follows the ecosystem UDS pattern:
//! - Create parent directories if missing
//! - Remove stale socket before binding
//! - Accept loop with graceful shutdown
//! - Cleanup socket file on stop

use rhizo_crypt_core::RhizoCrypt;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::net::UnixListener;
use tokio::sync::watch;
use tracing::{info, warn};

/// Unix domain socket JSON-RPC server.
///
/// Serves newline-delimited JSON-RPC 2.0 over a Unix socket, sharing the
/// same request handler as the TCP and HTTP transports.
pub struct UdsJsonRpcServer {
    primal: Arc<RhizoCrypt>,
    socket_path: PathBuf,
}

impl UdsJsonRpcServer {
    /// Create a new UDS server that will bind to `socket_path`.
    #[must_use]
    pub const fn new(primal: Arc<RhizoCrypt>, socket_path: PathBuf) -> Self {
        Self {
            primal,
            socket_path,
        }
    }

    /// Start serving JSON-RPC over the Unix socket.
    ///
    /// Creates parent directories, removes any stale socket file, binds the
    /// listener, and accepts connections until the `shutdown` receiver signals.
    ///
    /// # Errors
    ///
    /// Returns `std::io::Error` if directory creation, socket removal, or
    /// binding fails.
    pub async fn serve(&self, shutdown: watch::Receiver<bool>) -> Result<(), std::io::Error> {
        self.serve_inner(shutdown, None).await
    }

    /// Start serving and signal `ready` once the listener is bound.
    ///
    /// Identical to [`serve`](Self::serve) but notifies the provided
    /// [`tokio::sync::Notify`] after the socket is ready to accept connections.
    ///
    /// # Errors
    ///
    /// Returns `std::io::Error` if binding fails.
    pub async fn serve_with_ready(
        &self,
        shutdown: watch::Receiver<bool>,
        ready: Arc<tokio::sync::Notify>,
    ) -> Result<(), std::io::Error> {
        self.serve_inner(shutdown, Some(ready)).await
    }

    async fn serve_inner(
        &self,
        mut shutdown: watch::Receiver<bool>,
        ready: Option<Arc<tokio::sync::Notify>>,
    ) -> Result<(), std::io::Error> {
        self.prepare_socket_path()?;

        let listener = UnixListener::bind(&self.socket_path)?;
        info!(path = %self.socket_path.display(), "JSON-RPC 2.0 UDS listening");

        if let Some(notify) = ready {
            notify.notify_one();
        }

        loop {
            tokio::select! {
                result = listener.accept() => {
                    match result {
                        Ok((stream, _addr)) => {
                            let primal = Arc::clone(&self.primal);
                            tokio::spawn(async move {
                                if let Err(e) = super::newline::handle_newline_connection(stream, primal).await {
                                    warn!(error = %e, "UDS connection error");
                                }
                            });
                        }
                        Err(e) => {
                            warn!(error = %e, "UDS accept error");
                        }
                    }
                }
                _ = shutdown.changed() => {
                    info!("UDS listener shutting down");
                    break;
                }
            }
        }

        self.cleanup();
        Ok(())
    }

    /// Remove the socket file (idempotent).
    pub fn cleanup(&self) {
        cleanup_socket_at(&self.socket_path);
    }

    /// Socket path this server will bind to.
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
}

/// Remove a socket file at the given path (idempotent, logs on failure).
pub fn cleanup_socket_at(path: &Path) {
    if path.exists() {
        if let Err(e) = std::fs::remove_file(path) {
            warn!(path = %path.display(), error = %e, "failed to clean up UDS socket");
        } else {
            info!(path = %path.display(), "cleaned up UDS socket");
        }
    }
}

/// Resolve the default socket path for rhizoCrypt.
///
/// Uses `rhizo_crypt_core::transport::socket_path_for_primal` which
/// respects `$XDG_RUNTIME_DIR/biomeos/` per the ecosystem standard.
/// Falls back to the primal name `"rhizocrypt"` (lowercase, matching
/// the IPC compliance matrix).
#[must_use]
pub fn default_socket_path() -> PathBuf {
    rhizo_crypt_core::transport::socket_path_for_primal("rhizocrypt")
        .unwrap_or_else(|| PathBuf::from("/tmp/biomeos/rhizocrypt.sock"))
}

#[cfg(test)]
#[expect(clippy::unwrap_used, clippy::expect_used, reason = "test code")]
mod tests {
    use super::*;
    use rhizo_crypt_core::{PrimalLifecycle, RhizoCryptConfig};
    use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};

    async fn test_primal() -> Arc<RhizoCrypt> {
        let mut p = RhizoCrypt::new(RhizoCryptConfig::default());
        p.start().await.unwrap();
        Arc::new(p)
    }

    #[tokio::test]
    async fn test_uds_roundtrip() {
        let dir = tempfile::tempdir().expect("tempdir");
        let sock = dir.path().join("rhizocrypt-test.sock");
        let primal = test_primal().await;

        let server = UdsJsonRpcServer::new(primal, sock.clone());
        let (shutdown_tx, shutdown_rx) = watch::channel(false);
        let ready = Arc::new(tokio::sync::Notify::new());
        let ready_rx = Arc::clone(&ready);

        let handle =
            tokio::spawn(async move { server.serve_with_ready(shutdown_rx, ready_rx).await });
        ready.notified().await;

        let stream = tokio::net::UnixStream::connect(&sock).await.expect("connect");
        let (reader, mut writer) = stream.into_split();

        let req = r#"{"jsonrpc":"2.0","method":"health.check","params":{},"id":1}"#;
        writer.write_all(format!("{req}\n").as_bytes()).await.unwrap();
        writer.shutdown().await.unwrap();

        let mut lines = BufReader::new(reader).lines();
        let line = lines.next_line().await.unwrap().expect("response");
        let resp: serde_json::Value = serde_json::from_str(&line).unwrap();
        assert_eq!(resp["jsonrpc"], "2.0");
        assert!(resp["result"].is_object());

        let _ = shutdown_tx.send(true);
        let _ = handle.await;
    }

    #[tokio::test]
    async fn test_uds_stale_socket_removed() {
        let dir = tempfile::tempdir().expect("tempdir");
        let sock = dir.path().join("stale-test.sock");
        std::fs::write(&sock, "stale").unwrap();
        assert!(sock.exists());

        let primal = test_primal().await;
        let server = UdsJsonRpcServer::new(primal, sock.clone());
        let (shutdown_tx, shutdown_rx) = watch::channel(false);
        let ready = Arc::new(tokio::sync::Notify::new());
        let ready_rx = Arc::clone(&ready);

        let handle =
            tokio::spawn(async move { server.serve_with_ready(shutdown_rx, ready_rx).await });
        ready.notified().await;

        assert!(sock.exists(), "socket should exist (re-bound)");

        let _ = shutdown_tx.send(true);
        let _ = handle.await;
    }

    #[tokio::test]
    async fn test_uds_cleanup_removes_file() {
        let dir = tempfile::tempdir().expect("tempdir");
        let sock = dir.path().join("cleanup-test.sock");
        std::fs::write(&sock, "").unwrap();
        assert!(sock.exists());

        cleanup_socket_at(&sock);
        assert!(!sock.exists());
    }

    #[tokio::test]
    async fn test_uds_cleanup_nonexistent_noop() {
        let dir = tempfile::tempdir().expect("tempdir");
        let sock = dir.path().join("nonexistent.sock");
        cleanup_socket_at(&sock);
    }

    #[test]
    fn test_default_socket_path_contains_biomeos() {
        let path = default_socket_path();
        let path_str = path.to_string_lossy();
        assert!(
            path_str.contains("biomeos") || path_str.contains("rhizocrypt"),
            "path should reference biomeos or rhizocrypt: {path_str}"
        );
    }
}
