// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2024–2026 ecoPrimals Project

//! BTSP — `BiomeOS` Transport Security Protocol.
//!
//! Implements the server-side X25519 + HMAC-SHA256 handshake per
//! `BTSP_PROTOCOL_STANDARD.md`. When `FAMILY_ID` is set (production mode),
//! incoming UDS connections are auto-detected by first byte:
//! - `{`: read first line — if `"protocol":"btsp"` → JSON-line BTSP handshake,
//!   otherwise plain JSON-RPC routed to full handler (filesystem-authenticated).
//! - `[`: batch JSON-RPC routed to full handler (filesystem-authenticated).
//! - Other: length-prefixed BTSP handshake.
//!
//! All successful paths (handshake or filesystem-authenticated) serve the full
//! JSON-RPC method set via `handle_newline_connection`.
//!
//! ## Phase 3 — Encrypted Channel
//!
//! After a successful Phase 2 handshake, the client may send a
//! `btsp.negotiate` JSON-RPC request to upgrade the connection to
//! ChaCha20-Poly1305 encrypted framing. If negotiation succeeds, all
//! subsequent traffic uses length-prefixed encrypted frames. If the client
//! doesn't negotiate or the server returns `{"cipher":"null"}`, the
//! connection stays on cleartext newline-delimited JSON-RPC.
//!
//! Development mode (`BIOMEOS_INSECURE=1`, no `FAMILY_ID`) bypasses the
//! handshake and serves raw newline-delimited JSON-RPC.

pub mod framing;
pub mod phase3;
pub mod server;
pub mod types;

pub use phase3::Phase3Keys;
pub use server::{BtspServer, BtspSession};
pub use types::{BtspCipher, HandshakeError};

/// Read the BTSP family seed from the environment.
///
/// Checks:
/// 1. `{PRIMAL_PREFIX}_FAMILY_SEED` (primal-specific override)
/// 2. `FAMILY_SEED` (ecosystem-wide)
///
/// Seed encoding matches primalSpring's `raw_family_seed_from_env`:
/// - Hex string (len >= 32, even, all hex digits) → raw UTF-8 bytes (NOT hex-decoded)
/// - Valid base64 → decoded bytes
/// - Otherwise → raw UTF-8 bytes
///
/// Returns `None` in development mode (no seed configured).
/// Uses `bytes::Bytes` for O(1) clone when shared across connections.
#[must_use]
pub fn read_family_seed(primal_env_prefix: &str) -> Option<bytes::Bytes> {
    let primal_key = format!("{primal_env_prefix}_FAMILY_SEED");
    let val = std::env::var(&primal_key).or_else(|_| std::env::var("FAMILY_SEED")).ok()?;
    let trimmed = val.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(bytes::Bytes::from(normalize_seed_bytes(trimmed)))
    }
}

/// Normalize seed bytes to match primalSpring's encoding convention.
///
/// The harness generates hex-encoded seeds (64 ASCII chars). All ecosystem
/// primals must interpret them identically:
/// 1. Hex string → use as raw UTF-8 bytes (wire-compat with crypto provider)
/// 2. Valid base64 → decode to raw bytes
/// 3. Anything else → use as raw UTF-8 bytes
fn normalize_seed_bytes(s: &str) -> Vec<u8> {
    use base64::Engine;
    use tracing::debug;

    if is_hex_seed(s) {
        debug!("FAMILY_SEED: using as raw UTF-8 bytes (hex-encoded seed)");
        return s.as_bytes().to_vec();
    }

    if let Ok(decoded) = base64::engine::general_purpose::STANDARD.decode(s)
        && !decoded.is_empty()
    {
        debug!("FAMILY_SEED: decoded from base64 ({} bytes)", decoded.len());
        return decoded;
    }

    debug!("FAMILY_SEED: using as raw UTF-8 bytes (plain string)");
    s.as_bytes().to_vec()
}

/// Returns `true` if the string looks like a hex-encoded seed.
///
/// Must be at least 32 chars, even length, and all hex digits.
fn is_hex_seed(s: &str) -> bool {
    s.len() >= 32 && s.len().is_multiple_of(2) && s.bytes().all(|b| b.is_ascii_hexdigit())
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
                assert_eq!(seed.as_ref(), b"primal-seed");
            },
        );
    }

    #[test]
    fn test_read_family_seed_generic_fallback() {
        temp_env::with_vars(
            [("RHIZOCRYPT_FAMILY_SEED", None::<&str>), ("FAMILY_SEED", Some("generic-seed"))],
            || {
                let seed = read_family_seed("RHIZOCRYPT").unwrap();
                assert_eq!(seed.as_ref(), b"generic-seed");
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

    // --- Seed encoding normalization tests (match primalSpring) ---

    #[test]
    fn test_normalize_hex_seed_uses_raw_utf8() {
        // 64-char hex string (harness-generated): NOT hex-decoded, used as 64 ASCII bytes
        let hex_seed = "a1b2c3d4e5f6a7b8c9d0e1f2a3b4c5d6a1b2c3d4e5f6a7b8c9d0e1f2a3b4c5d6";
        assert_eq!(hex_seed.len(), 64);
        let result = normalize_seed_bytes(hex_seed);
        assert_eq!(result, hex_seed.as_bytes());
        assert_eq!(result.len(), 64, "hex seed should be 64 raw UTF-8 bytes");
    }

    #[test]
    fn test_normalize_base64_seed_decodes() {
        use base64::Engine;
        // 32-byte key encoded as base64: should be DECODED to 32 bytes
        let raw_key = [0x42u8; 32];
        let b64_seed = base64::engine::general_purpose::STANDARD.encode(raw_key);
        let result = normalize_seed_bytes(&b64_seed);
        assert_eq!(result, raw_key.to_vec());
        assert_eq!(result.len(), 32, "base64 seed should decode to 32 bytes");
    }

    #[test]
    fn test_normalize_plain_seed_uses_raw_utf8() {
        // Non-hex, non-base64 string: used as raw UTF-8 bytes
        let plain = "my-family-seed!";
        let result = normalize_seed_bytes(plain);
        assert_eq!(result, plain.as_bytes());
    }

    #[test]
    fn test_normalize_short_hex_uses_base64_or_raw() {
        // Short hex-like string (< 32 chars) is NOT treated as hex seed
        let short = "deadbeef";
        assert!(!is_hex_seed(short));
        let result = normalize_seed_bytes(short);
        // "deadbeef" is valid base64 (decodes to 5 bytes)
        assert_ne!(result, short.as_bytes(), "should base64-decode, not raw");
    }

    /// Cross-primal compatibility: verify that the HKDF derivation from a
    /// harness-generated hex seed produces the same handshake key regardless
    /// of whether you run primalSpring's or rhizoCrypt's code path.
    #[test]
    fn test_cross_primal_hkdf_compatibility() {
        use hkdf::Hkdf;
        use sha2::Sha256;

        // Simulated harness-generated hex seed (64 hex chars = 32 bytes when decoded,
        // but used as 64 raw UTF-8 bytes per primalSpring convention)
        let hex_seed = "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";

        // rhizoCrypt path
        let rhizo_bytes = normalize_seed_bytes(hex_seed);
        assert_eq!(rhizo_bytes.len(), 64);
        assert_eq!(rhizo_bytes, hex_seed.as_bytes());

        // primalSpring path (same logic: hex → raw UTF-8 bytes)
        let spring_bytes = hex_seed.as_bytes().to_vec();
        assert_eq!(rhizo_bytes, spring_bytes, "seed bytes must match primalSpring");

        // HKDF derivation must match: salt="btsp-v1", info="handshake"
        let hk = Hkdf::<Sha256>::new(Some(b"btsp-v1"), &rhizo_bytes);
        let mut rhizo_key = [0u8; 32];
        hk.expand(b"handshake", &mut rhizo_key).unwrap();

        let hk2 = Hkdf::<Sha256>::new(Some(b"btsp-v1"), &spring_bytes);
        let mut spring_key = [0u8; 32];
        hk2.expand(b"handshake", &mut spring_key).unwrap();

        assert_eq!(rhizo_key, spring_key, "handshake keys must be identical");
    }

    /// Cross-primal compatibility for base64-encoded seeds.
    #[test]
    fn test_cross_primal_base64_seed_compatibility() {
        use base64::Engine;

        // A seed that primalSpring would base64-decode
        let raw_key = b"cross-primal-test-seed-32bytes!!";
        let b64_seed = base64::engine::general_purpose::STANDARD.encode(raw_key);

        // Both sides must produce the same bytes
        let rhizo_bytes = normalize_seed_bytes(&b64_seed);
        assert_eq!(rhizo_bytes, raw_key.as_slice(), "base64 seed must decode identically");
    }
}
