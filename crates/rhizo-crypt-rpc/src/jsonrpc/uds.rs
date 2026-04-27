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

        let btsp_required = crate::btsp::is_btsp_required();
        let family_seed = crate::btsp::read_family_seed("RHIZOCRYPT");

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
                            let primal = Arc::clone(&self.primal);
                            let seed = family_seed.clone();
                            let enforce = btsp_required;
                            tokio::spawn(async move {
                                if let Err(e) = handle_uds_connection(stream, primal, enforce, seed.as_deref()).await {
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

/// Handle a single UDS connection with optional BTSP handshake enforcement.
///
/// When `btsp_required` is `true`, uses first-byte auto-detect with three
/// branches:
///
/// 1. `{` → read the full first line, then:
///    - `"protocol":"btsp"` → **JSON-line BTSP handshake** (primalSpring
///      interop), then serve full JSON-RPC on the authenticated stream
///    - otherwise → **full JSON-RPC** — UDS is filesystem-authenticated and
///      family-scoped (BTSP Phase 1), so all methods are available without a
///      Phase 2 handshake
/// 2. `[` → batch JSON-RPC (no BTSP required on UDS)
/// 3. Any other byte → **length-prefixed BTSP handshake** (internal), then
///    serve full JSON-RPC
///
/// When `btsp_required` is `false` (development mode), the connection serves
/// raw newline-delimited JSON-RPC immediately with all methods.
async fn handle_uds_connection(
    mut stream: tokio::net::UnixStream,
    primal: Arc<RhizoCrypt>,
    btsp_required: bool,
    family_seed: Option<&[u8]>,
) -> std::io::Result<()> {
    use crate::btsp::BtspServer;
    use tokio::io::{AsyncReadExt, AsyncWriteExt};

    if btsp_required {
        let Some(seed) = family_seed else {
            warn!("BTSP required but no FAMILY_SEED — rejecting connection");
            return Err(std::io::Error::other("BTSP: no family seed"));
        };

        let mut first = [0u8; 1];
        let n = stream.read(&mut first).await?;
        if n == 0 {
            return Ok(());
        }

        if first[0] == b'{' {
            // Read the rest of the first line to distinguish BTSP from JSON-RPC.
            let mut first_line = vec![b'{'];
            let mut byte = [0u8; 1];
            loop {
                let n = stream.read(&mut byte).await?;
                if n == 0 {
                    break;
                }
                first_line.push(byte[0]);
                if byte[0] == b'\n' {
                    break;
                }
            }

            let json_end =
                first_line.iter().rposition(|&b| b != b'\n' && b != b'\r').map_or(0, |i| i + 1);
            let json_bytes = &first_line[..json_end];

            let is_btsp = serde_json::from_slice::<serde_json::Value>(json_bytes)
                .ok()
                .and_then(|v| v.get("protocol")?.as_str().map(|s| s == "btsp"))
                .unwrap_or(false);

            let (reader, writer) = stream.into_split();

            if is_btsp {
                debug!("detected BTSP JSON-line handshake (protocol:btsp)");
                let mut rw = tokio::io::join(reader, writer);
                match BtspServer::accept_handshake_jsonline(&mut rw, seed, json_bytes).await {
                    Ok(session) => {
                        debug!(
                            cipher = session.cipher.as_str(),
                            "BTSP JSON-line handshake complete, serving JSON-RPC"
                        );
                        super::newline::handle_newline_connection(rw, primal).await
                    }
                    Err(e) => {
                        warn!(error = %e, "BTSP JSON-line handshake failed");
                        let (_, mut writer) = rw.into_inner();
                        let reason = e.to_string();
                        if let Err(e2) =
                            BtspServer::send_handshake_error_jsonline(&mut writer, &reason).await
                        {
                            debug!(error = %e2, "failed to send BTSP JSON-line error");
                        }
                        let _ = writer.shutdown().await;
                        Err(std::io::Error::other(format!("BTSP JSON-line handshake failed: {e}")))
                    }
                }
            } else {
                debug!("plain JSON-RPC on UDS (filesystem-authenticated, all methods)");
                let chained_reader = first_line.as_slice().chain(reader);
                let joined = tokio::io::join(chained_reader, writer);
                super::newline::handle_newline_connection(joined, primal).await
            }
        } else if first[0] == b'[' {
            debug!("batch JSON-RPC on UDS (filesystem-authenticated, all methods)");
            let (reader, writer) = stream.into_split();
            let chained_reader = (&first[..]).chain(reader);
            let joined = tokio::io::join(chained_reader, writer);
            super::newline::handle_newline_connection(joined, primal).await
        } else {
            // Length-prefixed BTSP handshake (internal binary framing).
            let (reader, writer) = stream.into_split();
            let chained_reader = (&first[..]).chain(reader);
            let mut rw = tokio::io::join(chained_reader, writer);

            match BtspServer::accept_handshake(&mut rw, seed).await {
                Ok(session) => {
                    debug!(
                        cipher = session.cipher.as_str(),
                        "BTSP handshake complete, serving JSON-RPC"
                    );
                    super::newline::handle_newline_connection(rw, primal).await
                }
                Err(e) => {
                    warn!(error = %e, "BTSP handshake failed, dropping connection");
                    let (_, mut writer) = rw.into_inner();
                    if let Err(e2) = BtspServer::send_handshake_error(&mut writer).await {
                        debug!(error = %e2, "failed to send BTSP handshake error to client");
                    }
                    let _ = writer.shutdown().await;
                    Err(std::io::Error::other(format!("BTSP handshake failed: {e}")))
                }
            }
        }
    } else {
        super::newline::handle_newline_connection(stream, primal).await
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

/// Resolve the default socket path for rhizoCrypt (BTSP Phase 1 compliant).
///
/// When `FAMILY_ID` is set, returns `rhizocrypt-{family_id}.sock` per BTSP
/// socket naming convention. When unset, returns `rhizocrypt.sock` (dev mode).
///
/// Uses `$XDG_RUNTIME_DIR/biomeos/` per the ecosystem standard.
/// Falls back to `{temp_dir}/biomeos/` when path-based sockets are unavailable.
#[must_use]
pub fn default_socket_path() -> PathBuf {
    use rhizo_crypt_core::constants::{BIOMEOS_SOCKET_SUBDIR, SOCKET_FILE_EXTENSION};
    use rhizo_crypt_core::transport::{family_scoped_socket_path, read_family_id};

    family_scoped_socket_path("rhizocrypt", "RHIZOCRYPT").unwrap_or_else(|| {
        let family_id = read_family_id("RHIZOCRYPT");
        let stem = family_id.map_or_else(
            || format!("rhizocrypt{SOCKET_FILE_EXTENSION}"),
            |fid| format!("rhizocrypt-{fid}{SOCKET_FILE_EXTENSION}"),
        );
        std::env::temp_dir().join(BIOMEOS_SOCKET_SUBDIR).join(stem)
    })
}

#[cfg(test)]
#[expect(clippy::unwrap_used, clippy::expect_used, reason = "test code")]
#[path = "uds_tests.rs"]
mod tests;
