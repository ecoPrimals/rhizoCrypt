// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2024–2026 ecoPrimals Project

//! Unix domain socket JSON-RPC listener.

use rhizo_crypt_core::RhizoCrypt;
use std::path::PathBuf;
use std::sync::Arc;
use tracing::{error, info};

/// Resolve UDS path from the CLI value.
///
/// Empty string → ecosystem default (`$XDG_RUNTIME_DIR/biomeos/rhizocrypt.sock`).
/// Non-empty → use as-is.
pub fn resolve_uds_path(raw: &str) -> PathBuf {
    if raw.is_empty() {
        rhizo_crypt_rpc::jsonrpc::uds::default_socket_path()
    } else {
        PathBuf::from(raw)
    }
}

/// Start the UDS JSON-RPC listener if `unix_socket` is `Some`.
///
/// `None` = no UDS (test backward-compat). `Some("")` = default ecosystem
/// path. `Some(path)` = custom path. On production Unix, `main.rs` always
/// passes `Some` so UDS is unconditional.
///
/// Returns `(shutdown_sender, Option<socket_path>)` — the socket path is
/// used for manifest publication so springs can discover this primal.
pub fn start_uds_listener(
    unix_socket: Option<&str>,
    primal: &Arc<RhizoCrypt>,
) -> (tokio::sync::watch::Sender<bool>, Option<PathBuf>) {
    let (shutdown_tx, shutdown_rx) = tokio::sync::watch::channel(false);
    let socket_path = unix_socket.map(|raw_path| {
        let path = resolve_uds_path(raw_path);
        info!(path = %path.display(), "Starting UDS JSON-RPC listener");
        let uds_server =
            rhizo_crypt_rpc::jsonrpc::uds::UdsJsonRpcServer::new(Arc::clone(primal), path.clone());
        tokio::spawn(async move {
            if let Err(e) = uds_server.serve(shutdown_rx).await {
                error!(error = %e, "UDS JSON-RPC server error");
            }
        });
        path
    });
    (shutdown_tx, socket_path)
}
