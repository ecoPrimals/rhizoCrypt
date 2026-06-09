// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2024–2026 ecoPrimals Project

//! Neural API announce subsystem.
//!
//! Discovers biomeOS's neural-api UDS socket and publishes a
//! `primal.announce` JSON-RPC call with capabilities, cost hints,
//! and latency estimates. Non-fatal on failure — the primal runs
//! without Neural API routing, falling back to manifest-based discovery.

use rhizo_crypt_core::constants;
use rhizo_crypt_core::safe_env::SafeEnv;
use tracing::{debug, info, warn};

use crate::ServiceError;

/// Announce this primal to biomeOS Neural API for routing weight registration.
///
/// Discovers biomeOS's neural-api socket via tiered lookup, then sends a
/// `primal.announce` JSON-RPC call with capabilities, cost hints, and
/// latency estimates. Non-fatal on failure — the primal runs without
/// Neural API routing, falling back to manifest-based discovery.
#[cfg(unix)]
pub async fn announce_to_biomeos(socket_path: &std::path::Path) {
    let neural_socket = discover_neural_api_socket();
    let Some(neural_socket) = neural_socket else {
        debug!("biomeOS neural-api socket not found (standalone mode)");
        return;
    };

    let params = rhizo_crypt_core::niche::announce_payload(
        &socket_path.display().to_string(),
        Some(std::process::id()),
    );

    let request = serde_json::json!({
        "jsonrpc": "2.0",
        "method": "primal.announce",
        "params": params,
        "id": 1,
    });

    match send_jsonrpc_uds(&neural_socket, &request).await {
        Ok(resp) => {
            if let Some(result) = resp.get("result") {
                let caps = result
                    .get("capabilities_registered")
                    .and_then(serde_json::Value::as_u64)
                    .unwrap_or(0);
                let methods = result
                    .get("methods_registered")
                    .and_then(serde_json::Value::as_u64)
                    .unwrap_or(0);
                info!(capabilities = caps, methods = methods, "Announced to biomeOS Neural API");
            } else if let Some(err) = resp.get("error") {
                warn!(error = %err, "biomeOS Neural API rejected announce");
            }
        }
        Err(e) => {
            debug!(error = %e, socket = %neural_socket.display(), "biomeOS Neural API announce failed (non-fatal)");
        }
    }
}

/// Discover the biomeOS neural-api UDS socket via tiered lookup.
///
/// 1. `$NEURAL_API_SOCKET` env override
/// 2. `$XDG_RUNTIME_DIR/biomeos/neural-api-{family}.sock`
/// 3. `/tmp/biomeos/neural-api-{family}.sock`
#[cfg(unix)]
fn discover_neural_api_socket() -> Option<std::path::PathBuf> {
    use std::path::PathBuf;

    if let Some(path) = SafeEnv::get_optional(SafeEnv::NEURAL_API_SOCKET) {
        let p = PathBuf::from(&path);
        if p.exists() {
            return Some(p);
        }
    }

    let family = SafeEnv::get_or_default(SafeEnv::ECOPRIMALS_FAMILY_ID, "ecoPrimal");
    let socket_name = format!("neural-api-{family}.sock");

    if let Some(xdg) = SafeEnv::get_optional(SafeEnv::XDG_RUNTIME_DIR) {
        let p = PathBuf::from(xdg).join(constants::BIOMEOS_SOCKET_SUBDIR).join(&socket_name);
        if p.exists() {
            return Some(p);
        }
    }

    let p = PathBuf::from(constants::POSIX_FALLBACK_TMPDIR)
        .join(constants::BIOMEOS_SOCKET_SUBDIR)
        .join(&socket_name);
    if p.exists() {
        return Some(p);
    }

    None
}

/// Send a single JSON-RPC request over a UDS and read the response.
#[cfg(unix)]
async fn send_jsonrpc_uds(
    socket_path: &std::path::Path,
    request: &serde_json::Value,
) -> Result<serde_json::Value, ServiceError> {
    use tokio::io::{AsyncBufReadExt, AsyncWriteExt};
    use tokio::net::UnixStream;

    let mut stream = tokio::time::timeout(
        std::time::Duration::from_secs(constants::NEURAL_API_CONNECT_TIMEOUT_SECS),
        UnixStream::connect(socket_path),
    )
    .await
    .map_err(|_| ServiceError::Discovery("neural-api connect timeout".to_owned()))?
    .map_err(|e| ServiceError::Discovery(format!("neural-api connect: {e}")))?;

    let mut payload = serde_json::to_string(request)
        .map_err(|e| ServiceError::Discovery(format!("neural-api serialize: {e}")))?;
    payload.push('\n');
    stream
        .write_all(payload.as_bytes())
        .await
        .map_err(|e| ServiceError::Discovery(format!("neural-api write: {e}")))?;
    stream.flush().await.map_err(|e| ServiceError::Discovery(format!("neural-api flush: {e}")))?;

    let mut reader = tokio::io::BufReader::new(&mut stream);
    let mut line = String::new();
    tokio::time::timeout(
        std::time::Duration::from_secs(constants::NEURAL_API_READ_TIMEOUT_SECS),
        reader.read_line(&mut line),
    )
    .await
    .map_err(|_| ServiceError::Discovery("neural-api read timeout".to_owned()))?
    .map_err(|e| ServiceError::Discovery(format!("neural-api read: {e}")))?;

    serde_json::from_str(line.trim())
        .map_err(|e| ServiceError::Discovery(format!("neural-api parse: {e}")))
}
