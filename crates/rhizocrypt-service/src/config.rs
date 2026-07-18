// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2024–2026 ecoPrimals Project

//! Bind address resolution and TCP transport configuration.

use rhizo_crypt_core::constants;
use rhizo_crypt_core::safe_env::SafeEnv;
use std::net::SocketAddr;

use crate::ServiceError;

/// Resolve the bind address from CLI overrides + environment.
///
/// # Errors
///
/// Returns [`ServiceError::AddrParse`] if the resulting host:port cannot be parsed.
pub fn resolve_bind_addr(
    port_override: Option<u16>,
    host_override: Option<String>,
) -> Result<SocketAddr, ServiceError> {
    let default_port = if SafeEnv::is_development() {
        constants::DEFAULT_RPC_PORT
    } else {
        constants::PRODUCTION_RPC_PORT
    };
    let port = port_override.unwrap_or_else(|| SafeEnv::get_rpc_port(default_port));
    let host = host_override.unwrap_or_else(SafeEnv::get_rpc_host);
    let addr: SocketAddr = format!("{host}:{port}").parse()?;
    Ok(addr)
}

/// Check whether TCP transports were explicitly requested via environment.
///
/// Returns `true` when any TCP-related env var is set. Used to implement
/// opt-in TCP: when no CLI flag and no env var requests TCP, only the UDS
/// socket is started (Provenance Trio standard).
#[must_use]
pub fn has_explicit_tcp_config() -> bool {
    SafeEnv::get_optional(SafeEnv::RHIZOCRYPT_PORT).is_some()
        || SafeEnv::get_optional(SafeEnv::RHIZOCRYPT_RPC_PORT).is_some()
        || SafeEnv::get_optional(SafeEnv::RHIZOCRYPT_JSONRPC_PORT).is_some()
}
