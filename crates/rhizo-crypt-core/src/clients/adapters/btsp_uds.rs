// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2024–2026 ecoPrimals Project

//! BTSP-aware Unix domain socket adapter.
//!
//! Connects to bearDog (or other BTSP-strict peers) over UDS with the 4-step
//! BTSP handshake, then exchanges JSON-RPC 2.0 messages as NDJSON lines.

use super::{BoxFuture, ProtocolAdapter};
use crate::btsp_client::perform_client_handshake;
use crate::error::{IpcErrorPhase, Result, RhizoCryptError};
use std::fmt;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::UnixStream;

/// Unix domain socket adapter with BTSP strict-mode handshake.
pub struct BtspUnixAdapter {
    socket_path: PathBuf,
    timeout: Duration,
    request_id: AtomicU64,
}

impl BtspUnixAdapter {
    /// Create a new BTSP Unix socket adapter.
    ///
    /// # Errors
    ///
    /// Returns error if the socket path is invalid.
    pub fn new(socket_path: impl AsRef<Path>) -> Result<Self> {
        Ok(Self {
            socket_path: socket_path.as_ref().to_path_buf(),
            timeout: crate::constants::CONNECTION_TIMEOUT,
            request_id: AtomicU64::new(1),
        })
    }

    fn next_id(&self) -> u64 {
        self.request_id.fetch_add(1, Ordering::Relaxed)
    }

    /// Connect, handshake, send JSON-RPC, read response (all per-call).
    async fn call_with_handshake(&self, request: &serde_json::Value) -> Result<String> {
        let display_path = self.socket_path.display();

        let mut stream = tokio::time::timeout(self.timeout, UnixStream::connect(&self.socket_path))
            .await
            .map_err(|_| {
                RhizoCryptError::ipc(IpcErrorPhase::Connect, format!("timed out: {display_path}"))
            })?
            .map_err(|e| {
                RhizoCryptError::ipc(IpcErrorPhase::Connect, format!("{display_path}: {e}"))
            })?;

        perform_client_handshake(&mut stream).await.map_err(|e| {
            RhizoCryptError::ipc(IpcErrorPhase::Connect, format!("BTSP handshake failed: {e}"))
        })?;

        let req_json = serde_json::to_string(request)
            .map_err(|e| RhizoCryptError::integration(format!("serialize: {e}")))?;
        stream
            .write_all(req_json.as_bytes())
            .await
            .map_err(|e| RhizoCryptError::ipc(IpcErrorPhase::Write, format!("{e}")))?;
        stream
            .write_all(b"\n")
            .await
            .map_err(|e| RhizoCryptError::ipc(IpcErrorPhase::Write, format!("{e}")))?;
        stream
            .flush()
            .await
            .map_err(|e| RhizoCryptError::ipc(IpcErrorPhase::Write, format!("{e}")))?;

        let mut reader = BufReader::new(stream);
        let mut line = String::new();
        tokio::time::timeout(self.timeout, reader.read_line(&mut line))
            .await
            .map_err(|_| RhizoCryptError::ipc(IpcErrorPhase::Read, "response timed out"))?
            .map_err(|e| RhizoCryptError::ipc(IpcErrorPhase::Read, format!("{e}")))?;

        super::unix_socket::UnixSocketAdapter::parse_json_rpc_response(line.trim().as_bytes())
    }

    async fn attempt_handshake(&self) -> bool {
        if !self.socket_path.exists() {
            return false;
        }

        let Ok(Ok(mut stream)) =
            tokio::time::timeout(self.timeout, UnixStream::connect(&self.socket_path)).await
        else {
            return false;
        };

        perform_client_handshake(&mut stream).await.is_ok()
    }
}

impl fmt::Debug for BtspUnixAdapter {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("BtspUnixAdapter")
            .field("socket_path", &self.socket_path)
            .field("protocol", &"btsp-unix")
            .finish_non_exhaustive()
    }
}

impl ProtocolAdapter for BtspUnixAdapter {
    fn protocol(&self) -> &'static str {
        "btsp-unix"
    }

    fn call_json<'a>(
        &'a self,
        method: &'a str,
        args_json: &'a str,
    ) -> BoxFuture<'a, Result<String>> {
        Box::pin(async move {
            let params: serde_json::Value = serde_json::from_str(args_json)
                .unwrap_or_else(|_| serde_json::Value::String(args_json.to_owned()));

            let request = serde_json::json!({
                "jsonrpc": crate::constants::JSONRPC_VERSION,
                "method": method,
                "params": params,
                "id": self.next_id()
            });

            tracing::debug!(
                method = %method,
                socket = %self.socket_path.display(),
                "BTSP Unix adapter calling method"
            );

            self.call_with_handshake(&request).await
        })
    }

    fn call_oneway_json<'a>(
        &'a self,
        method: &'a str,
        args_json: &'a str,
    ) -> BoxFuture<'a, Result<()>> {
        Box::pin(async move {
            let params: serde_json::Value = serde_json::from_str(args_json)
                .unwrap_or_else(|_| serde_json::Value::String(args_json.to_owned()));

            let request = serde_json::json!({
                "jsonrpc": crate::constants::JSONRPC_VERSION,
                "method": method,
                "params": params,
            });

            tracing::debug!(
                method = %method,
                socket = %self.socket_path.display(),
                "BTSP Unix adapter sending notification"
            );

            if let Err(e) = self.call_with_handshake(&request).await {
                tracing::warn!(
                    method = %method,
                    error = %e,
                    "Notification delivery failed (fire-and-forget)"
                );
            }
            Ok(())
        })
    }

    fn is_healthy(&self) -> BoxFuture<'_, bool> {
        Box::pin(async move { self.attempt_handshake().await })
    }

    fn endpoint(&self) -> &str {
        self.socket_path.to_str().unwrap_or("btsp-unix-socket")
    }
}

#[cfg(test)]
#[expect(clippy::unwrap_used, reason = "test code")]
mod tests {
    use super::*;

    #[test]
    fn test_btsp_uds_adapter_new() {
        let adapter = BtspUnixAdapter::new("/tmp/btsp-test.sock").unwrap();
        assert_eq!(adapter.protocol(), "btsp-unix");
        assert_eq!(adapter.endpoint(), "/tmp/btsp-test.sock");
    }

    #[test]
    fn test_btsp_uds_adapter_debug() {
        let adapter = BtspUnixAdapter::new("/tmp/btsp-test.sock").unwrap();
        let debug = format!("{adapter:?}");
        assert!(debug.contains("BtspUnixAdapter"));
        assert!(debug.contains("/tmp/btsp-test.sock"));
    }

    #[tokio::test]
    async fn test_btsp_uds_nonexistent_socket() {
        let adapter =
            BtspUnixAdapter::new("/tmp/nonexistent_ecoPrimal_btsp_socket_12345.sock").unwrap();
        let result = adapter.call_json("test", "{}").await;
        assert!(result.is_err());
    }
}
