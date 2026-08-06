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

/// Start both UDS listeners if `unix_socket` is `Some`:
/// 1. JSON-RPC on `{primal}.sock` (existing)
/// 2. tarpc binary on `{primal}.tarpc.sock` (G64 C2 dual-socket)
///
/// `None` = no UDS (test backward-compat). `Some("")` = default ecosystem
/// path. `Some(path)` = custom path. On production Unix, `main.rs` always
/// passes `Some` so UDS is unconditional.
///
/// Returns `(shutdown_sender, tarpc_shutdown_sender, Option<socket_path>)` —
/// the socket path is used for manifest publication so springs can discover
/// this primal.
pub fn start_uds_listener(
    unix_socket: Option<&str>,
    primal: &Arc<RhizoCrypt>,
) -> (tokio::sync::watch::Sender<bool>, tokio::sync::watch::Sender<bool>, Option<PathBuf>) {
    let (shutdown_tx, shutdown_rx) = tokio::sync::watch::channel(false);
    let mut tarpc_shutdown_tx_out = tokio::sync::watch::channel(false).0;
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

        let tarpc_path = resolve_tarpc_uds_path(raw_path);
        info!(path = %tarpc_path.display(), "Starting tarpc binary UDS listener (C2)");
        let tarpc_server = rhizo_crypt_rpc::TarpcUdsServer::new(Arc::clone(primal), tarpc_path);
        tarpc_shutdown_tx_out = tarpc_server.shutdown_sender();
        tokio::spawn(async move {
            if let Err(e) = tarpc_server.serve().await {
                error!(error = %e, "tarpc UDS server error");
            }
        });

        path
    });
    (shutdown_tx, tarpc_shutdown_tx_out, socket_path)
}

/// Resolve tarpc UDS path from the CLI value.
///
/// Empty string → ecosystem default (`$XDG_RUNTIME_DIR/biomeos/rhizocrypt.tarpc.sock`).
/// Non-empty → derive `.tarpc.sock` from the given path.
pub fn resolve_tarpc_uds_path(raw: &str) -> PathBuf {
    if raw.is_empty() {
        rhizo_crypt_rpc::tarpc_uds::default_tarpc_socket_path()
    } else {
        let base = PathBuf::from(raw);
        let stem = base.file_stem().and_then(|s| s.to_str()).unwrap_or("rhizocrypt");
        base.with_file_name(format!("{stem}.tarpc.sock"))
    }
}
