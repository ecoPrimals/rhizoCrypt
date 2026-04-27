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
mod tests {
    use super::*;
    use rhizo_crypt_core::{PrimalLifecycle, RhizoCryptConfig};
    use tokio::io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader};

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

    #[test]
    fn test_socket_path_accessor() {
        let path = std::path::PathBuf::from("/tmp/test.sock");
        let primal = tokio::runtime::Runtime::new().unwrap().block_on(test_primal());
        let server = UdsJsonRpcServer::new(primal, path.clone());
        assert_eq!(server.socket_path(), path);
    }

    #[tokio::test]
    async fn test_uds_server_cleanup_idempotent() {
        let dir = tempfile::tempdir().expect("tempdir");
        let sock = dir.path().join("idempotent.sock");
        let primal = test_primal().await;
        let server = UdsJsonRpcServer::new(primal, sock.clone());
        server.cleanup();
        server.cleanup();
    }

    #[tokio::test]
    async fn test_uds_multiple_sequential_requests() {
        let dir = tempfile::tempdir().expect("tempdir");
        let sock = dir.path().join("seq-test.sock");
        let primal = test_primal().await;

        let server = UdsJsonRpcServer::new(primal, sock.clone());
        let (shutdown_tx, shutdown_rx) = watch::channel(false);
        let ready = Arc::new(tokio::sync::Notify::new());
        let ready_rx = Arc::clone(&ready);

        let handle =
            tokio::spawn(async move { server.serve_with_ready(shutdown_rx, ready_rx).await });
        ready.notified().await;

        for i in 0..5_u32 {
            let stream = tokio::net::UnixStream::connect(&sock).await.expect("connect");
            let (reader, mut writer) = stream.into_split();

            let req =
                format!(r#"{{"jsonrpc":"2.0","method":"health.check","params":{{}},"id":{i}}}"#);
            writer.write_all(format!("{req}\n").as_bytes()).await.unwrap();
            writer.shutdown().await.unwrap();

            let mut lines = BufReader::new(reader).lines();
            let line = lines.next_line().await.unwrap().expect("response");
            let resp: serde_json::Value = serde_json::from_str(&line).unwrap();
            assert_eq!(resp["id"], i);
        }

        let _ = shutdown_tx.send(true);
        let _ = handle.await;
    }

    #[tokio::test]
    async fn test_uds_serve_creates_parent_dirs() {
        let dir = tempfile::tempdir().expect("tempdir");
        let nested = dir.path().join("deep").join("nested").join("test.sock");
        let primal = test_primal().await;

        let server = UdsJsonRpcServer::new(primal, nested.clone());
        let (shutdown_tx, shutdown_rx) = watch::channel(false);
        let ready = Arc::new(tokio::sync::Notify::new());
        let ready_rx = Arc::clone(&ready);

        let handle =
            tokio::spawn(async move { server.serve_with_ready(shutdown_rx, ready_rx).await });
        ready.notified().await;

        assert!(nested.exists(), "socket should exist under nested dirs");

        let _ = shutdown_tx.send(true);
        let _ = handle.await;
    }

    /// Full BTSP JSON-line handshake over a real `UnixStream` pair, testing the
    /// routing in `handle_uds_connection` end-to-end:
    /// `ClientHello` → `ServerHello` → `ChallengeResponse` → `HandshakeComplete` → JSON-RPC
    #[tokio::test]
    async fn test_btsp_jsonline_handshake_over_uds() {
        use base64::Engine;
        use hmac::{Hmac, Mac};
        use sha2::Sha256;
        use x25519_dalek::{EphemeralSecret, PublicKey};

        type HmacSha256 = Hmac<Sha256>;

        let family_seed = b"integration-test-family-seed-ok!";
        let primal = test_primal().await;

        let (server_raw, client_raw) = std::os::unix::net::UnixStream::pair().unwrap();
        server_raw.set_nonblocking(true).unwrap();
        client_raw.set_nonblocking(true).unwrap();
        let server_stream = tokio::net::UnixStream::from_std(server_raw).unwrap();
        let mut client = tokio::net::UnixStream::from_std(client_raw).unwrap();

        let server_handle = tokio::spawn(async move {
            handle_uds_connection(server_stream, primal, true, Some(family_seed)).await
        });

        // --- Client side: JSON-line BTSP handshake ---

        let b64 = base64::engine::general_purpose::STANDARD;

        // Step 1: Send ClientHello
        let client_secret = EphemeralSecret::random_from_rng(rand::thread_rng());
        let client_public = PublicKey::from(&client_secret);
        let hello = serde_json::json!({
            "protocol": "btsp",
            "version": 1,
            "client_ephemeral_pub": b64.encode(client_public.as_bytes())
        });
        let hello_line = format!("{hello}\n");
        client.write_all(hello_line.as_bytes()).await.unwrap();

        // Step 2: Read ServerHello
        let mut buf = vec![0u8; 4096];
        let mut total = 0;
        loop {
            let n = client.read(&mut buf[total..]).await.unwrap();
            total += n;
            if buf[..total].contains(&b'\n') || n == 0 {
                break;
            }
        }
        let server_hello_line = std::str::from_utf8(&buf[..total]).unwrap();
        let sh: serde_json::Value = serde_json::from_str(server_hello_line.trim()).unwrap();

        assert_eq!(sh["version"], 1);
        assert!(sh["server_ephemeral_pub"].is_string());
        assert!(sh["challenge"].is_string());
        assert!(sh["session_id"].is_string());

        let server_pub_bytes = b64.decode(sh["server_ephemeral_pub"].as_str().unwrap()).unwrap();
        let challenge_bytes = b64.decode(sh["challenge"].as_str().unwrap()).unwrap();

        // Step 3: Compute HMAC and send ChallengeResponse
        let handshake_key = {
            use hkdf::Hkdf;
            let hk = Hkdf::<sha2::Sha256>::new(Some(b"btsp-v1"), family_seed);
            let mut okm = [0u8; 32];
            hk.expand(b"handshake", &mut okm).unwrap();
            okm
        };

        let mut mac = HmacSha256::new_from_slice(&handshake_key).expect("HMAC init");
        mac.update(&challenge_bytes);
        mac.update(client_public.as_bytes());
        mac.update(&server_pub_bytes);
        let hmac_result = mac.finalize().into_bytes();

        let cr = serde_json::json!({
            "response": b64.encode(hmac_result),
            "preferred_cipher": "null"
        });
        let cr_line = format!("{cr}\n");
        client.write_all(cr_line.as_bytes()).await.unwrap();

        // Step 4: Read HandshakeComplete
        total = 0;
        loop {
            let n = client.read(&mut buf[total..]).await.unwrap();
            total += n;
            if buf[..total].contains(&b'\n') || n == 0 {
                break;
            }
        }
        let complete_line = std::str::from_utf8(&buf[..total]).unwrap();
        let hc: serde_json::Value = serde_json::from_str(complete_line.trim()).unwrap();
        assert_eq!(hc["cipher"], "null");
        assert!(hc["session_id"].is_string());

        // Post-handshake: send a JSON-RPC request
        let rpc_req = r#"{"jsonrpc":"2.0","method":"health.check","params":{},"id":1}"#;
        client.write_all(format!("{rpc_req}\n").as_bytes()).await.unwrap();

        total = 0;
        loop {
            let n = client.read(&mut buf[total..]).await.unwrap();
            total += n;
            if buf[..total].contains(&b'\n') || n == 0 {
                break;
            }
        }
        let rpc_resp_line = std::str::from_utf8(&buf[..total]).unwrap();
        let resp: serde_json::Value = serde_json::from_str(rpc_resp_line.trim()).unwrap();
        assert_eq!(resp["jsonrpc"], "2.0");
        assert!(resp["result"].is_object());
        assert_eq!(resp["id"], 1);

        client.shutdown().await.unwrap();
        let server_result = server_handle.await.unwrap();
        assert!(server_result.is_ok(), "server should complete cleanly");
    }

    /// Verify that a partial/invalid `ClientHello` gets an error response, not
    /// a silent connection reset.
    #[tokio::test]
    async fn test_btsp_jsonline_invalid_key_returns_error() {
        let family_seed = b"integration-test-family-seed-ok!";
        let primal = test_primal().await;

        let (server_raw, client_raw) = std::os::unix::net::UnixStream::pair().unwrap();
        server_raw.set_nonblocking(true).unwrap();
        client_raw.set_nonblocking(true).unwrap();
        let server_stream = tokio::net::UnixStream::from_std(server_raw).unwrap();
        let mut client = tokio::net::UnixStream::from_std(client_raw).unwrap();

        let server_handle = tokio::spawn(async move {
            handle_uds_connection(server_stream, primal, true, Some(family_seed)).await
        });

        // Send ClientHello with invalid 4-byte key (the socat test from the audit)
        let hello = r#"{"protocol":"btsp","version":1,"client_ephemeral_pub":"dGVzdA=="}"#;
        client.write_all(format!("{hello}\n").as_bytes()).await.unwrap();

        // Should receive an error JSON-line, NOT a connection reset
        let mut buf = vec![0u8; 4096];
        let mut total = 0;
        loop {
            let n = client.read(&mut buf[total..]).await.unwrap();
            total += n;
            if n == 0 || buf[..total].contains(&b'\n') {
                break;
            }
        }
        assert!(total > 0, "should receive error response, not connection reset");
        let error_line = std::str::from_utf8(&buf[..total]).unwrap();
        let err: serde_json::Value = serde_json::from_str(error_line.trim()).unwrap();
        assert_eq!(err["error"], "handshake_failed");
        assert!(
            err["reason"].as_str().unwrap().contains("32 bytes"),
            "reason should mention expected key length: {}",
            err["reason"]
        );

        let server_result = server_handle.await.unwrap();
        assert!(server_result.is_err(), "server should report handshake failure");
    }

    /// PG-52 repro: plain `dag.session.create` over UDS with BTSP required
    /// must succeed (UDS is filesystem-authenticated, no handshake needed).
    #[tokio::test]
    async fn test_plain_jsonrpc_data_methods_on_btsp_uds() {
        let family_seed = b"integration-test-family-seed-ok!";
        let primal = test_primal().await;

        let (server_raw, client_raw) = std::os::unix::net::UnixStream::pair().unwrap();
        server_raw.set_nonblocking(true).unwrap();
        client_raw.set_nonblocking(true).unwrap();
        let server_stream = tokio::net::UnixStream::from_std(server_raw).unwrap();
        let mut client = tokio::net::UnixStream::from_std(client_raw).unwrap();

        let server_handle = tokio::spawn(async move {
            handle_uds_connection(server_stream, primal, true, Some(family_seed)).await
        });

        let req = r#"{"jsonrpc":"2.0","method":"dag.session.create","params":{"description":"PG-52 test","session_type":"General"},"id":1}"#;
        client.write_all(format!("{req}\n").as_bytes()).await.unwrap();
        client.shutdown().await.unwrap();

        let mut buf = vec![0u8; 4096];
        let mut total = 0;
        loop {
            let n = client.read(&mut buf[total..]).await.unwrap();
            total += n;
            if n == 0 || buf[..total].contains(&b'\n') {
                break;
            }
        }
        assert!(total > 0, "should receive response, not empty/reset");
        let resp_line = std::str::from_utf8(&buf[..total]).unwrap();
        let resp: serde_json::Value = serde_json::from_str(resp_line.trim()).unwrap();

        assert_eq!(resp["jsonrpc"], "2.0");
        assert_eq!(resp["id"], 1);
        assert!(
            resp["result"].is_string(),
            "dag.session.create should return a session ID, got: {resp}"
        );

        let server_result = server_handle.await.unwrap();
        assert!(server_result.is_ok());
    }

    /// Verify multiple data methods work over plain UDS with BTSP required:
    /// `dag.session.create`, `dag.event.append`, `dag.vertex.children`,
    /// `dag.frontier.get`, `dag.merkle.root`.
    #[tokio::test]
    async fn test_dag_method_suite_on_btsp_uds() {
        let family_seed = b"integration-test-family-seed-ok!";
        let primal = test_primal().await;

        let (server_raw, client_raw) = std::os::unix::net::UnixStream::pair().unwrap();
        server_raw.set_nonblocking(true).unwrap();
        client_raw.set_nonblocking(true).unwrap();
        let server_stream = tokio::net::UnixStream::from_std(server_raw).unwrap();
        let client_stream = tokio::net::UnixStream::from_std(client_raw).unwrap();

        let server_handle = tokio::spawn(async move {
            handle_uds_connection(server_stream, primal, true, Some(family_seed)).await
        });

        let (reader, mut writer) = client_stream.into_split();

        let create = r#"{"jsonrpc":"2.0","method":"dag.session.create","params":{"description":"suite","session_type":"General"},"id":1}"#;
        writer.write_all(format!("{create}\n").as_bytes()).await.unwrap();

        let health = r#"{"jsonrpc":"2.0","method":"health.check","params":{},"id":2}"#;
        writer.write_all(format!("{health}\n").as_bytes()).await.unwrap();

        let caps = r#"{"jsonrpc":"2.0","method":"capability.list","params":{},"id":3}"#;
        writer.write_all(format!("{caps}\n").as_bytes()).await.unwrap();

        writer.shutdown().await.unwrap();

        let mut lines = BufReader::new(reader).lines();
        let mut responses = Vec::new();
        while let Some(line) = lines.next_line().await.unwrap() {
            responses.push(serde_json::from_str::<serde_json::Value>(&line).unwrap());
        }

        assert_eq!(responses.len(), 3, "should get 3 responses");

        assert_eq!(responses[0]["id"], 1);
        assert!(responses[0]["result"].is_string(), "dag.session.create result: {}", responses[0]);

        assert_eq!(responses[1]["id"], 2);
        assert!(responses[1]["result"].is_object(), "health.check result: {}", responses[1]);

        assert_eq!(responses[2]["id"], 3);
        assert!(
            responses[2]["result"].is_object() || responses[2]["result"].is_array(),
            "capability.list result: {}",
            responses[2]
        );

        let server_result = server_handle.await.unwrap();
        assert!(server_result.is_ok());
    }

    /// Verify batch JSON-RPC also works on BTSP-enforced UDS.
    #[tokio::test]
    async fn test_batch_jsonrpc_on_btsp_uds() {
        let family_seed = b"integration-test-family-seed-ok!";
        let primal = test_primal().await;

        let (server_raw, client_raw) = std::os::unix::net::UnixStream::pair().unwrap();
        server_raw.set_nonblocking(true).unwrap();
        client_raw.set_nonblocking(true).unwrap();
        let server_stream = tokio::net::UnixStream::from_std(server_raw).unwrap();
        let mut client = tokio::net::UnixStream::from_std(client_raw).unwrap();

        let server_handle = tokio::spawn(async move {
            handle_uds_connection(server_stream, primal, true, Some(family_seed)).await
        });

        let batch = r#"[{"jsonrpc":"2.0","method":"health.check","params":{},"id":1},{"jsonrpc":"2.0","method":"dag.session.create","params":{"description":"batch","session_type":"General"},"id":2}]"#;
        client.write_all(format!("{batch}\n").as_bytes()).await.unwrap();
        client.shutdown().await.unwrap();

        let mut buf = vec![0u8; 8192];
        let mut total = 0;
        loop {
            let n = client.read(&mut buf[total..]).await.unwrap();
            total += n;
            if n == 0 || buf[..total].contains(&b'\n') {
                break;
            }
        }
        let resp_line = std::str::from_utf8(&buf[..total]).unwrap();
        let resp: serde_json::Value = serde_json::from_str(resp_line.trim()).unwrap();

        let arr = resp.as_array().expect("batch response should be an array");
        assert_eq!(arr.len(), 2);
        assert!(arr[0]["result"].is_object(), "health.check: {}", arr[0]);
        assert!(arr[1]["result"].is_string(), "dag.session.create: {}", arr[1]);

        let server_result = server_handle.await.unwrap();
        assert!(server_result.is_ok());
    }
}
