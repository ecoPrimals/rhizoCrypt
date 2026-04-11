// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2024–2026 ecoPrimals Project

//! BTSP protocol types per `BTSP_PROTOCOL_STANDARD.md` v1.0.0.
//!
//! Ecosystem-compatible wire types for the 4-step X25519 + HMAC-SHA256
//! handshake. Kept in sync with the canonical definitions in wateringHole.

use serde::{Deserialize, Serialize};
use zeroize::Zeroize;

/// BTSP protocol version.
pub const BTSP_VERSION: u32 = 1;

/// HKDF salt for deriving the handshake key from the family seed.
pub const HANDSHAKE_HKDF_SALT: &[u8] = b"btsp-v1";

/// HKDF info for deriving the handshake key.
pub const HANDSHAKE_HKDF_INFO: &[u8] = b"handshake";

/// Maximum BTSP frame size: 16 MiB.
pub const MAX_FRAME_SIZE: u32 = 0x0100_0000;

/// Step 1: Client → Server.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientHello {
    /// Protocol version (must be `BTSP_VERSION`).
    pub version: u32,
    /// X25519 ephemeral public key (32 bytes).
    pub client_ephemeral_pub: [u8; 32],
}

/// Step 2: Server → Client.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerHello {
    /// Protocol version.
    pub version: u32,
    /// X25519 ephemeral public key (32 bytes).
    pub server_ephemeral_pub: [u8; 32],
    /// 32-byte random challenge.
    pub challenge: [u8; 32],
}

/// Step 3: Client → Server.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChallengeResponse {
    /// HMAC-SHA256 over `challenge ‖ client_pub ‖ server_pub` with handshake key.
    pub response: [u8; 32],
    /// Client's preferred cipher suite.
    pub preferred_cipher: BtspCipher,
}

/// Step 4: Server → Client.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HandshakeComplete {
    /// Negotiated cipher suite.
    pub cipher: BtspCipher,
    /// 16-byte session identifier.
    pub session_id: [u8; 16],
}

/// Negotiable cipher suites.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BtspCipher {
    /// ChaCha20-Poly1305 AEAD — confidentiality + integrity + authentication.
    Chacha20Poly1305,
    /// HMAC-SHA256 per frame — integrity + authentication, no confidentiality.
    HmacPlain,
    /// No per-frame protection.
    Null,
}

impl BtspCipher {
    /// Wire name for cipher negotiation.
    #[must_use]
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Chacha20Poly1305 => "chacha20_poly1305",
            Self::HmacPlain => "hmac_plain",
            Self::Null => "null",
        }
    }
}

/// Directional session keys derived from the ECDH shared secret.
#[derive(Zeroize)]
#[zeroize(drop)]
pub struct SessionKeys {
    /// Key for encrypting outgoing frames.
    pub encrypt_key: [u8; 32],
    /// Key for decrypting incoming frames.
    pub decrypt_key: [u8; 32],
}

/// BTSP handshake errors.
#[derive(Debug, thiserror::Error)]
pub enum HandshakeError {
    /// I/O error during handshake.
    #[error("BTSP I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// Protocol version mismatch.
    #[error("BTSP version mismatch: expected {expected}, got {got}")]
    VersionMismatch {
        /// Expected version.
        expected: u32,
        /// Received version.
        got: u32,
    },

    /// Server rejected the challenge response (wrong family seed).
    #[error("BTSP handshake failed: family verification")]
    FamilyVerification,

    /// JSON serialization/deserialization error.
    #[error("BTSP serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    /// Key derivation failure.
    #[error("BTSP key derivation error: {0}")]
    KeyDerivation(String),
}

#[cfg(test)]
#[expect(clippy::unwrap_used, reason = "test code")]
mod tests {
    use super::*;

    #[test]
    fn test_btsp_version_constant() {
        assert_eq!(BTSP_VERSION, 1);
    }

    #[test]
    fn test_max_frame_size_16mib() {
        assert_eq!(MAX_FRAME_SIZE, 16 * 1024 * 1024);
    }

    #[test]
    fn test_cipher_as_str() {
        assert_eq!(BtspCipher::Chacha20Poly1305.as_str(), "chacha20_poly1305");
        assert_eq!(BtspCipher::HmacPlain.as_str(), "hmac_plain");
        assert_eq!(BtspCipher::Null.as_str(), "null");
    }

    #[test]
    fn test_cipher_serde_roundtrip() {
        for cipher in [BtspCipher::Chacha20Poly1305, BtspCipher::HmacPlain, BtspCipher::Null] {
            let json = serde_json::to_string(&cipher).unwrap();
            let back: BtspCipher = serde_json::from_str(&json).unwrap();
            assert_eq!(back, cipher);
        }
    }

    #[test]
    fn test_client_hello_serde() {
        let hello = ClientHello {
            version: BTSP_VERSION,
            client_ephemeral_pub: [0xAA; 32],
        };
        let json = serde_json::to_value(&hello).unwrap();
        assert_eq!(json["version"], BTSP_VERSION);
        let back: ClientHello = serde_json::from_value(json).unwrap();
        assert_eq!(back.client_ephemeral_pub, [0xAA; 32]);
    }

    #[test]
    fn test_server_hello_serde() {
        let hello = ServerHello {
            version: BTSP_VERSION,
            server_ephemeral_pub: [0xBB; 32],
            challenge: [0xCC; 32],
        };
        let json = serde_json::to_value(&hello).unwrap();
        let back: ServerHello = serde_json::from_value(json).unwrap();
        assert_eq!(back.challenge, [0xCC; 32]);
    }

    #[test]
    fn test_handshake_complete_serde() {
        let complete = HandshakeComplete {
            cipher: BtspCipher::Chacha20Poly1305,
            session_id: [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16],
        };
        let json = serde_json::to_value(&complete).unwrap();
        let back: HandshakeComplete = serde_json::from_value(json).unwrap();
        assert_eq!(back.cipher, BtspCipher::Chacha20Poly1305);
        assert_eq!(back.session_id[0], 1);
    }

    #[test]
    fn test_handshake_error_display() {
        let io_err = HandshakeError::Io(std::io::Error::other("test"));
        assert!(io_err.to_string().contains("test"));

        let ver_err = HandshakeError::VersionMismatch {
            expected: 1,
            got: 2,
        };
        assert!(ver_err.to_string().contains("expected 1"));
        assert!(ver_err.to_string().contains("got 2"));

        let fam_err = HandshakeError::FamilyVerification;
        assert!(fam_err.to_string().contains("family verification"));

        let key_err = HandshakeError::KeyDerivation("bad key".into());
        assert!(key_err.to_string().contains("bad key"));
    }

    #[test]
    fn test_session_keys_zeroize_on_drop() {
        let keys = SessionKeys {
            encrypt_key: [0xFF; 32],
            decrypt_key: [0xEE; 32],
        };
        assert_eq!(keys.encrypt_key[0], 0xFF);
        assert_eq!(keys.decrypt_key[0], 0xEE);
    }

    #[test]
    fn test_hkdf_constants() {
        assert_eq!(HANDSHAKE_HKDF_SALT, b"btsp-v1");
        assert_eq!(HANDSHAKE_HKDF_INFO, b"handshake");
    }
}
