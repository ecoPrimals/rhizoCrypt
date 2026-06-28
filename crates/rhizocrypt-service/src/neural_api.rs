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

    let request =
        build_announce_request(&socket_path.display().to_string(), Some(std::process::id()));

    match send_jsonrpc_uds(&neural_socket, &request).await {
        Ok(resp) => match parse_announce_response(&resp) {
            AnnounceResponseOutcome::Registered {
                capabilities,
                methods,
            } => {
                info!(capabilities, methods, "Announced to biomeOS Neural API");
            }
            AnnounceResponseOutcome::Rejected(err) => {
                warn!(error = %err, "biomeOS Neural API rejected announce");
            }
            AnnounceResponseOutcome::NoResult => {}
        },
        Err(e) => {
            debug!(error = %e, socket = %neural_socket.display(), "biomeOS Neural API announce failed (non-fatal)");
        }
    }
}

/// Outcome of a `primal.announce` JSON-RPC response.
#[cfg(unix)]
#[derive(Debug, Clone, PartialEq, Eq)]
enum AnnounceResponseOutcome {
    /// Announce succeeded with registration counts.
    Registered {
        /// Capabilities registered with biomeOS.
        capabilities: u64,
        /// Methods registered with biomeOS.
        methods: u64,
    },
    /// biomeOS returned a JSON-RPC error object.
    Rejected(serde_json::Value),
    /// Response lacked both `result` and `error`.
    NoResult,
}

/// Parse a `primal.announce` JSON-RPC response envelope.
#[cfg(unix)]
#[must_use]
fn parse_announce_response(resp: &serde_json::Value) -> AnnounceResponseOutcome {
    if let Some(result) = resp.get("result") {
        let capabilities =
            result.get("capabilities_registered").and_then(serde_json::Value::as_u64).unwrap_or(0);
        let methods =
            result.get("methods_registered").and_then(serde_json::Value::as_u64).unwrap_or(0);
        return AnnounceResponseOutcome::Registered {
            capabilities,
            methods,
        };
    }

    if let Some(err) = resp.get("error") {
        return AnnounceResponseOutcome::Rejected(err.clone());
    }

    AnnounceResponseOutcome::NoResult
}

/// Build the full `primal.announce` JSON-RPC request envelope.
#[cfg(unix)]
#[must_use]
fn build_announce_request(socket_path: &str, pid: Option<u32>) -> serde_json::Value {
    let params = rhizo_crypt_core::niche::announce_payload(socket_path, pid);

    serde_json::json!({
        "jsonrpc": rhizo_crypt_core::constants::JSONRPC_VERSION,
        "method": "primal.announce",
        "params": params,
        "id": 1,
    })
}

/// Ordered candidate paths for the biomeOS neural-api socket (tiers 2–3).
///
/// Tier 1 (`$NEURAL_API_SOCKET`) is handled separately in
/// [`resolve_neural_api_socket`].
#[cfg(unix)]
#[must_use]
fn neural_api_socket_candidates(
    family_id: &str,
    xdg_runtime_dir: Option<&str>,
) -> Vec<std::path::PathBuf> {
    use std::path::PathBuf;

    let socket_name = format!("neural-api-{family_id}.sock");
    let mut candidates = Vec::with_capacity(2);

    if let Some(xdg) = xdg_runtime_dir {
        candidates
            .push(PathBuf::from(xdg).join(constants::BIOMEOS_SOCKET_SUBDIR).join(&socket_name));
    }

    candidates.push(
        PathBuf::from(constants::POSIX_FALLBACK_TMPDIR)
            .join(constants::BIOMEOS_SOCKET_SUBDIR)
            .join(socket_name),
    );

    candidates
}

/// Resolve the biomeOS neural-api UDS socket via tiered lookup.
///
/// 1. `$NEURAL_API_SOCKET` env override (when `neural_api_socket_env` is set)
/// 2. `$XDG_RUNTIME_DIR/biomeos/neural-api-{family}.sock`
/// 3. `/tmp/biomeos/neural-api-{family}.sock`
///
/// The `exists` predicate is injectable for unit tests.
#[cfg(unix)]
#[must_use]
fn resolve_neural_api_socket<E>(
    neural_api_socket_env: Option<&str>,
    family_id: &str,
    xdg_runtime_dir: Option<&str>,
    exists: E,
) -> Option<std::path::PathBuf>
where
    E: Fn(&std::path::Path) -> bool,
{
    use std::path::PathBuf;

    if let Some(path) = neural_api_socket_env {
        let candidate = PathBuf::from(path);
        if exists(&candidate) {
            return Some(candidate);
        }
    }

    neural_api_socket_candidates(family_id, xdg_runtime_dir)
        .into_iter()
        .find(|candidate| exists(candidate))
}

/// Discover the biomeOS neural-api UDS socket via tiered lookup.
///
/// Uses connect-probe (`socket_is_alive`) instead of `Path::exists` to
/// distinguish live listeners from stale socket files left by crashed
/// processes (ecosystem standard v1.1.5).
#[cfg(unix)]
fn discover_neural_api_socket() -> Option<std::path::PathBuf> {
    let family = SafeEnv::get_or_default(
        SafeEnv::ECOPRIMALS_FAMILY_ID,
        rhizo_crypt_core::constants::DEFAULT_FAMILY_ID,
    );

    resolve_neural_api_socket(
        SafeEnv::get_optional(SafeEnv::NEURAL_API_SOCKET).as_deref(),
        &family,
        SafeEnv::get_optional(SafeEnv::XDG_RUNTIME_DIR).as_deref(),
        rhizo_crypt_core::transport::socket_is_alive,
    )
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

#[cfg(all(test, unix))]
#[expect(clippy::unwrap_used, clippy::expect_used, reason = "test code")]
#[path = "neural_api_tests.rs"]
mod tests;
