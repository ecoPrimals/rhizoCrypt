// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2024–2026 ecoPrimals Project

//! Discovery registration and capability manifest publishing.

use rhizo_crypt_core::constants;
use std::net::SocketAddr;
use tracing::{info, warn};

use crate::ServiceError;

/// Register this primal with the configured discovery adapter.
///
/// The discovery adapter is the one bootstrap address a primal needs.
/// All other primals are discovered at runtime via capability queries.
/// This function is adapter-agnostic — any compatible endpoint
/// accepting `register` + `heartbeat` JSON-RPC methods will work.
///
/// Returns the connected client so the caller can populate the engine's
/// discovery registry with eagerly-resolved peer endpoints.
///
/// # Errors
///
/// Returns [`ServiceError::Discovery`] if registration or heartbeat setup fails.
pub async fn register_with_discovery(
    discovery_addr: &str,
    our_addr: SocketAddr,
) -> Result<rhizo_crypt_core::clients::songbird::DiscoveryClient, ServiceError> {
    use rhizo_crypt_core::clients::songbird::{DiscoveryClient, DiscoveryConfig};

    let mut config = DiscoveryConfig::new();
    config.address = std::borrow::Cow::Owned(discovery_addr.to_owned());
    let client = DiscoveryClient::new(config);

    client
        .connect()
        .await
        .map_err(|e| ServiceError::Discovery(format!("discovery adapter connect: {e}")))?;

    let our_endpoint = format!("{}://{our_addr}", constants::DISCOVERY_ENDPOINT_SCHEME);
    let result = client
        .register(&our_endpoint)
        .await
        .map_err(|e| ServiceError::Discovery(format!("discovery adapter register: {e}")))?;

    if !result.success {
        return Err(ServiceError::Discovery(format!(
            "discovery registration rejected: {}",
            result.message
        )));
    }

    client
        .start_heartbeat()
        .await
        .map_err(|e| ServiceError::Discovery(format!("discovery adapter heartbeat: {e}")))?;

    Ok(client)
}

/// Publish a capability manifest so sibling primals (and springs) can discover
/// rhizoCrypt via file-based capability lookup (PG-32).
///
/// Writes `$XDG_RUNTIME_DIR/biomeos/rhizocrypt.json` with the UDS socket path
/// and all capabilities from `METHOD_CATALOG`. Logs on failure but does not
/// abort — the service still runs, just without manifest-based discoverability.
#[cfg(unix)]
pub async fn publish_capability_manifest(
    socket_path: &std::path::Path,
    tcp_addr: Option<SocketAddr>,
) {
    use rhizo_crypt_core::discovery::{PrimalManifest, publish_manifest};
    use rhizo_crypt_core::niche::CAPABILITIES;

    let manifest = PrimalManifest {
        primal: rhizo_crypt_core::niche::PRIMAL_ID.to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        socket: socket_path.display().to_string(),
        address: tcp_addr.map(|a| a.to_string()),
        capabilities: CAPABILITIES.iter().map(|s| (*s).to_string()).collect(),
    };

    match publish_manifest(&manifest).await {
        Ok(path) => info!(path = %path.display(), "Capability manifest published"),
        Err(e) => warn!(error = %e, "Failed to publish capability manifest (discovery degraded)"),
    }
}
