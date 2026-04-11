// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2024–2026 ecoPrimals Project

//! BTSP Phase 2 — `BiomeOS` Transport Security Protocol.
//!
//! Implements the server-side X25519 + HMAC-SHA256 handshake per
//! `BTSP_PROTOCOL_STANDARD.md`. When `FAMILY_ID` is set (production mode),
//! every incoming UDS connection must complete the 4-step handshake before
//! JSON-RPC methods are exposed. Failure → connection refused.
//!
//! Development mode (`BIOMEOS_INSECURE=1`, no `FAMILY_ID`) bypasses the
//! handshake and serves raw newline-delimited JSON-RPC.

pub mod framing;
pub mod server;
pub mod types;

pub use server::{BtspServer, BtspSession};
pub use types::{BtspCipher, HandshakeError};

/// Read the BTSP family seed from the environment.
///
/// Checks:
/// 1. `{PRIMAL_PREFIX}_FAMILY_SEED` (primal-specific override)
/// 2. `FAMILY_SEED` (ecosystem-wide)
///
/// Returns `None` in development mode (no seed configured).
#[must_use]
pub fn read_family_seed(primal_env_prefix: &str) -> Option<Vec<u8>> {
    let primal_key = format!("{primal_env_prefix}_FAMILY_SEED");
    let val = std::env::var(&primal_key).or_else(|_| std::env::var("FAMILY_SEED")).ok()?;
    let val = val.trim().to_string();
    if val.is_empty() {
        None
    } else {
        Some(val.into_bytes())
    }
}

/// Returns `true` when the BTSP handshake is required (production mode).
///
/// Production mode: `FAMILY_ID` is set to a real value (not `"default"`).
/// The caller should also have a `FAMILY_SEED` available for the handshake.
#[must_use]
pub fn is_btsp_required() -> bool {
    rhizo_crypt_core::transport::read_family_id("RHIZOCRYPT").is_some()
}

#[cfg(test)]
#[expect(clippy::unwrap_used, reason = "test code")]
mod tests {
    use super::*;

    #[test]
    fn test_read_family_seed_empty_env() {
        temp_env::with_vars(
            [("RHIZOCRYPT_FAMILY_SEED", None::<&str>), ("FAMILY_SEED", None::<&str>)],
            || {
                assert!(read_family_seed("RHIZOCRYPT").is_none());
            },
        );
    }

    #[test]
    fn test_read_family_seed_primal_override() {
        temp_env::with_vars(
            [
                ("RHIZOCRYPT_FAMILY_SEED", Some("primal-seed")),
                ("FAMILY_SEED", Some("generic-seed")),
            ],
            || {
                let seed = read_family_seed("RHIZOCRYPT").unwrap();
                assert_eq!(seed, b"primal-seed");
            },
        );
    }

    #[test]
    fn test_read_family_seed_generic_fallback() {
        temp_env::with_vars(
            [("RHIZOCRYPT_FAMILY_SEED", None::<&str>), ("FAMILY_SEED", Some("generic-seed"))],
            || {
                let seed = read_family_seed("RHIZOCRYPT").unwrap();
                assert_eq!(seed, b"generic-seed");
            },
        );
    }

    #[test]
    fn test_read_family_seed_whitespace_only_returns_none() {
        temp_env::with_vars(
            [("RHIZOCRYPT_FAMILY_SEED", None::<&str>), ("FAMILY_SEED", Some("   "))],
            || {
                assert!(read_family_seed("RHIZOCRYPT").is_none());
            },
        );
    }

    #[test]
    fn test_is_btsp_required_dev_mode() {
        temp_env::with_vars(
            [("RHIZOCRYPT_FAMILY_ID", None::<&str>), ("FAMILY_ID", None::<&str>)],
            || {
                assert!(!is_btsp_required());
            },
        );
    }
}
