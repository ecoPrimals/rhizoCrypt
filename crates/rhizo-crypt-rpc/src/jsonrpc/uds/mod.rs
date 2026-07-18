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

mod connection;
mod symlinks;

pub(crate) use connection::handle_uds_connection;

use rhizo_crypt_core::RhizoCrypt;
use rhizo_crypt_core::constants::DEFAULT_MAX_CONNECTIONS;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::net::UnixListener;
use tokio::sync::{Semaphore, watch};
use tracing::{debug, info, warn};

/// Unix domain socket JSON-RPC server.
///
/// Serves newline-delimited JSON-RPC 2.0 over a Unix socket, sharing the
/// same request handler as the TCP and HTTP transports.
pub struct UdsJsonRpcServer {
    server: crate::service::RhizoCryptRpcServer,
    socket_path: PathBuf,
}

impl UdsJsonRpcServer {
    /// Create a new UDS server that will bind to `socket_path`.
    #[must_use]
    pub fn new(primal: Arc<RhizoCrypt>, socket_path: PathBuf) -> Self {
        let server = crate::service::RhizoCryptRpcServer::new(primal);
        Self {
            server,
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
        symlinks::create_capability_symlink(&self.socket_path);
        info!(path = %self.socket_path.display(), "JSON-RPC 2.0 UDS listening");

        if let Some(notify) = ready {
            notify.notify_one();
        }

        let btsp_required = crate::btsp::is_btsp_required();
        let family_seed = crate::btsp::read_family_seed(rhizo_crypt_core::niche::ENV_PREFIX);

        if btsp_required {
            if family_seed.is_some() {
                info!("BTSP Phase 2: handshake enforced on every UDS connection");
            } else {
                warn!(
                    "FAMILY_ID is set but FAMILY_SEED is missing — \
                     BTSP handshake will reject all connections"
                );
            }
        } else {
            debug!("BTSP not required (development mode), serving raw JSON-RPC");
        }

        let semaphore = Arc::new(Semaphore::new(DEFAULT_MAX_CONNECTIONS));

        loop {
            tokio::select! {
                result = listener.accept() => {
                    match result {
                        Ok((stream, _addr)) => {
                            let Ok(permit) = Arc::clone(&semaphore).try_acquire_owned() else {
                                warn!("UDS connection rejected: limit reached");
                                drop(stream);
                                continue;
                            };
                            let server = self.server.clone();
                            let seed = family_seed.clone();
                            let enforce = btsp_required;
                            tokio::spawn(async move {
                                if let Err(e) = handle_uds_connection(stream, server, enforce, seed.as_deref()).await {
                                    warn!(error = %e, "UDS connection error");
                                }
                                drop(permit);
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

    /// Remove the socket file and capability symlink (idempotent).
    pub fn cleanup(&self) {
        symlinks::remove_capability_symlink(&self.socket_path);
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
pub(crate) fn cleanup_socket_at(path: &Path) {
    if path.exists() {
        if let Err(e) = std::fs::remove_file(path) {
            warn!(path = %path.display(), error = %e, "failed to clean up UDS socket");
        } else {
            info!(path = %path.display(), "cleaned up UDS socket");
        }
    }
}

/// Resolve the default socket path for rhizoCrypt (BTSP Phase 1 compliant).
///
/// Delegates to [`rhizo_crypt_core::transport::family_scoped_socket_path`]
/// which uses the unified fallback chain:
///
/// 1. `$XDG_RUNTIME_DIR/biomeos/rhizocrypt[-{family}].sock`
/// 2. `/run/biomeos/rhizocrypt[-{family}].sock` (Linux)
/// 3. `{temp_dir}/biomeos/rhizocrypt[-{family}].sock` (other Unix)
///
/// On platforms without path-based sockets (Android, Windows) falls back
/// to `{temp_dir}/biomeos/rhizocrypt.sock` as a best-effort default.
#[must_use]
pub fn default_socket_path() -> PathBuf {
    use rhizo_crypt_core::constants::{BIOMEOS_SOCKET_SUBDIR, SOCKET_FILE_EXTENSION};
    use rhizo_crypt_core::transport::family_scoped_socket_path;

    let id = rhizo_crypt_core::niche::PRIMAL_ID;
    family_scoped_socket_path(id, rhizo_crypt_core::niche::ENV_PREFIX).unwrap_or_else(|| {
        std::env::temp_dir()
            .join(BIOMEOS_SOCKET_SUBDIR)
            .join(format!("{id}{SOCKET_FILE_EXTENSION}"))
    })
}

#[cfg(test)]
#[path = "../uds_tests_support.rs"]
mod tests_support;

#[cfg(test)]
#[path = "../uds_tests.rs"]
mod tests;

#[cfg(test)]
#[path = "../uds_tests_btsp.rs"]
mod tests_btsp;

#[cfg(test)]
#[path = "../uds_tests_jsonrpc.rs"]
mod tests_jsonrpc;

#[cfg(test)]
#[path = "../uds_tests_mito_beacon.rs"]
mod tests_mito_beacon;

#[cfg(test)]
#[path = "../uds_tests_errors.rs"]
mod tests_errors;
