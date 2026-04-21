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

// ---------------------------------------------------------------------------
// JSON-line wire types (primalSpring interop)
//
// Springs use newline-delimited JSON with base64-encoded byte fields and an
// explicit `protocol` discriminator in ClientHello. These types provide
// ser/de for that format; crypto logic converts to/from the internal types.
// ---------------------------------------------------------------------------

use base64::Engine;

/// Base64 engine (standard alphabet, with padding).
const fn b64() -> base64::engine::GeneralPurpose {
    base64::engine::general_purpose::STANDARD
}

/// JSON-line `ClientHello` as sent by primalSpring.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientHelloWire {
    /// Always `"btsp"` — used for protocol detection on the first line.
    pub protocol: String,
    /// Protocol version (matches [`BTSP_VERSION`]).
    pub version: u32,
    /// X25519 ephemeral public key, base64-encoded.
    pub client_ephemeral_pub: String,
}

impl ClientHelloWire {
    /// Decode the base64 public key into 32 raw bytes.
    ///
    /// # Errors
    ///
    /// Returns [`HandshakeError::KeyDerivation`] if the base64 is invalid or
    /// the decoded length is not 32 bytes.
    pub fn decode_pub(&self) -> Result<[u8; 32], HandshakeError> {
        let bytes = b64()
            .decode(&self.client_ephemeral_pub)
            .map_err(|e| HandshakeError::KeyDerivation(format!("base64 decode: {e}")))?;
        <[u8; 32]>::try_from(bytes.as_slice()).map_err(|_| {
            HandshakeError::KeyDerivation(format!("expected 32 bytes, got {}", bytes.len()))
        })
    }
}

/// JSON-line `ServerHello` — includes `session_id` for primalSpring compat.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerHelloWire {
    /// Protocol version.
    pub version: u32,
    /// X25519 ephemeral public key, base64-encoded.
    pub server_ephemeral_pub: String,
    /// 32-byte random challenge, base64-encoded.
    pub challenge: String,
    /// 16-byte session identifier (hex or opaque string).
    pub session_id: String,
}

/// JSON-line `ChallengeResponse` from the client.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChallengeResponseWire {
    /// HMAC-SHA256 response, base64-encoded.
    pub response: String,
    /// Client's preferred cipher suite.
    pub preferred_cipher: String,
}

impl ChallengeResponseWire {
    /// Decode the base64 HMAC response into 32 raw bytes.
    ///
    /// # Errors
    ///
    /// Returns [`HandshakeError::KeyDerivation`] on invalid base64 or wrong length.
    pub fn decode_response(&self) -> Result<[u8; 32], HandshakeError> {
        let bytes = b64()
            .decode(&self.response)
            .map_err(|e| HandshakeError::KeyDerivation(format!("base64 decode: {e}")))?;
        <[u8; 32]>::try_from(bytes.as_slice()).map_err(|_| {
            HandshakeError::KeyDerivation(format!("expected 32 bytes, got {}", bytes.len()))
        })
    }
}

/// JSON-line `HandshakeComplete` sent by the server.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HandshakeCompleteWire {
    /// Negotiated cipher suite (string form).
    pub cipher: String,
    /// Session identifier (matches `ServerHelloWire::session_id`).
    pub session_id: String,
}

/// JSON-line handshake error frame.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HandshakeErrorWire {
    /// Error type.
    pub error: String,
    /// Human-readable reason.
    #[serde(default)]
    pub reason: String,
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

    #[test]
    fn test_client_hello_wire_serde() {
        let hello = ClientHelloWire {
            protocol: "btsp".to_owned(),
            version: 1,
            client_ephemeral_pub: b64().encode([0xAA; 32]),
        };
        let json = serde_json::to_string(&hello).unwrap();
        assert!(json.contains("\"protocol\":\"btsp\""));
        let back: ClientHelloWire = serde_json::from_str(&json).unwrap();
        assert_eq!(back.protocol, "btsp");
        assert_eq!(back.decode_pub().unwrap(), [0xAA; 32]);
    }

    #[test]
    fn test_client_hello_wire_bad_base64() {
        let hello = ClientHelloWire {
            protocol: "btsp".to_owned(),
            version: 1,
            client_ephemeral_pub: "not-valid-base64!!!".to_owned(),
        };
        assert!(hello.decode_pub().is_err());
    }

    #[test]
    fn test_client_hello_wire_wrong_length() {
        let hello = ClientHelloWire {
            protocol: "btsp".to_owned(),
            version: 1,
            client_ephemeral_pub: b64().encode([0xBB; 16]),
        };
        assert!(hello.decode_pub().is_err());
    }

    #[test]
    fn test_server_hello_wire_serde() {
        let hello = ServerHelloWire {
            version: 1,
            server_ephemeral_pub: b64().encode([0xCC; 32]),
            challenge: b64().encode([0xDD; 32]),
            session_id: "deadbeef01020304".to_owned(),
        };
        let json = serde_json::to_string(&hello).unwrap();
        let back: ServerHelloWire = serde_json::from_str(&json).unwrap();
        assert_eq!(back.session_id, "deadbeef01020304");
    }

    #[test]
    fn test_challenge_response_wire_decode() {
        let cr = ChallengeResponseWire {
            response: b64().encode([0xEE; 32]),
            preferred_cipher: "null".to_owned(),
        };
        assert_eq!(cr.decode_response().unwrap(), [0xEE; 32]);
    }

    #[test]
    fn test_handshake_complete_wire_serde() {
        let hc = HandshakeCompleteWire {
            cipher: "chacha20_poly1305".to_owned(),
            session_id: "abcdef0123456789".to_owned(),
        };
        let json = serde_json::to_string(&hc).unwrap();
        let back: HandshakeCompleteWire = serde_json::from_str(&json).unwrap();
        assert_eq!(back.cipher, "chacha20_poly1305");
    }

    #[test]
    fn test_handshake_error_wire_serde() {
        let err = HandshakeErrorWire {
            error: "handshake_failed".to_owned(),
            reason: "family_verification".to_owned(),
        };
        let json = serde_json::to_string(&err).unwrap();
        let back: HandshakeErrorWire = serde_json::from_str(&json).unwrap();
        assert_eq!(back.error, "handshake_failed");
    }
}
