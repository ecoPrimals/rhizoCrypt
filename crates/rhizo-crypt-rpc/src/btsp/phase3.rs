// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2024–2026 ecoPrimals Project

//! BTSP Phase 3 — encrypted channel negotiation.
//!
//! After a successful Phase 1/2 handshake, the client may send a
//! `btsp.negotiate` JSON-RPC request to upgrade the connection to an
//! encrypted channel using ChaCha20-Poly1305 AEAD.
//!
//! Wire protocol (primalSpring-compatible):
//!
//! 1. Client sends `btsp.negotiate` as a cleartext JSON-RPC request
//! 2. Server responds with selected cipher + server nonce (cleartext)
//! 3. Both sides derive session keys via HKDF-SHA256
//! 4. All subsequent messages use length-prefixed encrypted framing:
//!    `[4B len BE u32][12B nonce][ciphertext + Poly1305 tag]`
//!
//! If the server doesn't support the offered cipher, it returns
//! `{"cipher":"null"}` and the connection stays plaintext.

use base64::Engine;
use serde::{Deserialize, Serialize};
use zeroize::Zeroize;

use super::types::HandshakeError;

/// Phase 3 cipher selection (wire names use hyphens per primalSpring convention).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Phase3Cipher {
    /// ChaCha20-Poly1305 AEAD — confidentiality + integrity + authentication.
    #[serde(rename = "chacha20-poly1305")]
    ChaCha20Poly1305,
    /// HMAC-SHA256 per frame — integrity + authentication, no confidentiality.
    #[serde(rename = "hmac-plain")]
    HmacPlain,
    /// No per-frame protection (cleartext fallback).
    #[serde(rename = "null")]
    Null,
}

impl Phase3Cipher {
    /// Wire-format name for this cipher.
    #[must_use]
    pub const fn wire_name(&self) -> &'static str {
        match self {
            Self::ChaCha20Poly1305 => "chacha20-poly1305",
            Self::HmacPlain => "hmac-plain",
            Self::Null => "null",
        }
    }
}

/// Parsed params from a `btsp.negotiate` JSON-RPC request.
#[derive(Debug, Deserialize)]
pub struct NegotiateParams {
    /// Session ID from the Phase 1 handshake (hex-encoded).
    pub session_id: String,
    /// Ordered list of ciphers the client supports.
    pub ciphers: Vec<String>,
    /// Client-generated random nonce (base64-encoded).
    pub client_nonce: String,
}

/// Server response for `btsp.negotiate`.
#[derive(Debug, Serialize)]
pub struct NegotiateResult {
    /// Selected cipher suite.
    pub cipher: Phase3Cipher,
    /// Server-generated random nonce (base64-encoded).
    pub server_nonce: String,
}

/// Derived session keys for Phase 3 encrypted channel.
///
/// Both sides derive the same keys from the Phase 1 handshake key and
/// the nonces exchanged during negotiation. The client's encrypt key is
/// the server's decrypt key, and vice versa.
#[derive(Zeroize)]
#[zeroize(drop)]
pub struct Phase3Keys {
    encrypt_key: [u8; 32],
    decrypt_key: [u8; 32],
}

impl Phase3Keys {
    /// Derive Phase 3 session keys from the handshake key and both nonces.
    ///
    /// Uses HKDF-SHA256 with directional labels matching primalSpring:
    /// - `btsp-session-v1-c2s` for client→server traffic
    /// - `btsp-session-v1-s2c` for server→client traffic
    ///
    /// Server sets `is_client = false`; client sets `is_client = true`.
    ///
    /// # Errors
    ///
    /// Returns [`HandshakeError::KeyDerivation`] if HKDF expansion fails.
    pub fn derive(
        handshake_key: &[u8; 32],
        client_nonce: &[u8],
        server_nonce: &[u8],
        is_client: bool,
    ) -> Result<Self, HandshakeError> {
        use hkdf::Hkdf;
        use sha2::Sha256;

        let mut salt = Vec::with_capacity(client_nonce.len() + server_nonce.len());
        salt.extend_from_slice(client_nonce);
        salt.extend_from_slice(server_nonce);

        let hk = Hkdf::<Sha256>::new(Some(&salt), handshake_key);

        let mut c2s = [0u8; 32];
        hk.expand(b"btsp-session-v1-c2s", &mut c2s)
            .map_err(|e| HandshakeError::KeyDerivation(format!("Phase 3 c2s: {e}")))?;

        let mut s2c = [0u8; 32];
        hk.expand(b"btsp-session-v1-s2c", &mut s2c)
            .map_err(|e| HandshakeError::KeyDerivation(format!("Phase 3 s2c: {e}")))?;

        if is_client {
            Ok(Self {
                encrypt_key: c2s,
                decrypt_key: s2c,
            })
        } else {
            Ok(Self {
                encrypt_key: s2c,
                decrypt_key: c2s,
            })
        }
    }

    /// Encrypt plaintext using ChaCha20-Poly1305.
    ///
    /// Returns `[12-byte random nonce][ciphertext + Poly1305 tag]`.
    ///
    /// # Errors
    ///
    /// Returns [`HandshakeError::KeyDerivation`] if AEAD encryption fails.
    pub fn encrypt(&self, plaintext: &[u8]) -> Result<Vec<u8>, HandshakeError> {
        use chacha20poly1305::aead::{Aead, KeyInit};
        use chacha20poly1305::{ChaCha20Poly1305, Nonce};

        let cipher = ChaCha20Poly1305::new((&self.encrypt_key).into());

        let mut nonce_bytes = [0u8; 12];
        rand::RngCore::fill_bytes(&mut rand::thread_rng(), &mut nonce_bytes);
        let nonce = Nonce::from_slice(&nonce_bytes);

        let ciphertext = cipher
            .encrypt(nonce, plaintext)
            .map_err(|e| HandshakeError::KeyDerivation(format!("Phase 3 encrypt: {e}")))?;

        let mut frame = Vec::with_capacity(12 + ciphertext.len());
        frame.extend_from_slice(&nonce_bytes);
        frame.extend_from_slice(&ciphertext);
        Ok(frame)
    }

    /// Decrypt a frame: `[12-byte nonce][ciphertext + Poly1305 tag]`.
    ///
    /// # Errors
    ///
    /// Returns [`HandshakeError::KeyDerivation`] if the frame is too short
    /// or AEAD authentication/decryption fails.
    pub fn decrypt(&self, frame: &[u8]) -> Result<Vec<u8>, HandshakeError> {
        use chacha20poly1305::aead::{Aead, KeyInit};
        use chacha20poly1305::{ChaCha20Poly1305, Nonce};

        if frame.len() < 28 {
            return Err(HandshakeError::KeyDerivation(format!(
                "Phase 3 frame too short: {} bytes (need >= 28)",
                frame.len()
            )));
        }

        let (nonce_bytes, ciphertext) = frame.split_at(12);
        let cipher = ChaCha20Poly1305::new((&self.decrypt_key).into());
        let nonce = Nonce::from_slice(nonce_bytes);

        cipher
            .decrypt(nonce, ciphertext)
            .map_err(|e| HandshakeError::KeyDerivation(format!("Phase 3 decrypt: {e}")))
    }
}

/// Handle a `btsp.negotiate` request on an authenticated stream.
///
/// Reads the negotiate params, selects a cipher, generates a server nonce,
/// and returns the negotiate result. The caller is responsible for switching
/// to encrypted framing after a successful (non-null) negotiation.
///
/// Returns `Ok(Some(keys))` if encryption was negotiated, `Ok(None)` if the
/// client's ciphers aren't supported (null cipher fallback).
///
/// # Errors
///
/// Returns [`HandshakeError`] on serialization, I/O, or key derivation failures.
pub async fn handle_negotiate<S>(
    stream: &mut S,
    handshake_key: &[u8; 32],
    session_id_hex: &str,
    negotiate_json: &serde_json::Value,
) -> Result<Option<Phase3Keys>, HandshakeError>
where
    S: tokio::io::AsyncWriteExt + Unpin,
{
    let b64 = base64::engine::general_purpose::STANDARD;

    let params: NegotiateParams = serde_json::from_value(
        negotiate_json.get("params").cloned().unwrap_or(serde_json::Value::Null),
    )?;

    let request_id = negotiate_json.get("id").cloned();

    if params.session_id != session_id_hex {
        let err_resp = serde_json::json!({
            "jsonrpc": "2.0",
            "error": {
                "code": -32602,
                "message": "session_id mismatch"
            },
            "id": request_id
        });
        let resp_bytes = serde_json::to_vec(&err_resp)?;
        super::framing::write_json_line(stream, &resp_bytes).await?;
        return Ok(None);
    }

    let supports_chacha = params
        .ciphers
        .iter()
        .any(|c| c == "chacha20-poly1305" || c == Phase3Cipher::ChaCha20Poly1305.wire_name());

    if !supports_chacha {
        let result = NegotiateResult {
            cipher: Phase3Cipher::Null,
            server_nonce: String::new(),
        };
        let resp = serde_json::json!({
            "jsonrpc": "2.0",
            "result": result,
            "id": request_id
        });
        let resp_bytes = serde_json::to_vec(&resp)?;
        super::framing::write_json_line(stream, &resp_bytes).await?;
        return Ok(None);
    }

    let client_nonce = b64
        .decode(&params.client_nonce)
        .map_err(|e| HandshakeError::KeyDerivation(format!("client_nonce base64: {e}")))?;

    let mut server_nonce = [0u8; 32];
    rand::RngCore::fill_bytes(&mut rand::thread_rng(), &mut server_nonce);

    let keys = Phase3Keys::derive(handshake_key, &client_nonce, &server_nonce, false)?;

    let result = NegotiateResult {
        cipher: Phase3Cipher::ChaCha20Poly1305,
        server_nonce: b64.encode(server_nonce),
    };
    let resp = serde_json::json!({
        "jsonrpc": "2.0",
        "result": result,
        "id": request_id
    });
    let resp_bytes = serde_json::to_vec(&resp)?;
    super::framing::write_json_line(stream, &resp_bytes).await?;

    tracing::debug!("BTSP Phase 3: ChaCha20-Poly1305 negotiated");
    Ok(Some(keys))
}

#[cfg(test)]
#[expect(clippy::unwrap_used, clippy::expect_used, reason = "test code")]
mod tests {
    use super::*;

    #[test]
    fn phase3_cipher_wire_names() {
        assert_eq!(Phase3Cipher::ChaCha20Poly1305.wire_name(), "chacha20-poly1305");
        assert_eq!(Phase3Cipher::HmacPlain.wire_name(), "hmac-plain");
        assert_eq!(Phase3Cipher::Null.wire_name(), "null");
    }

    #[test]
    fn phase3_cipher_serde_roundtrip() {
        for cipher in [Phase3Cipher::ChaCha20Poly1305, Phase3Cipher::HmacPlain, Phase3Cipher::Null]
        {
            let json = serde_json::to_string(&cipher).unwrap();
            let back: Phase3Cipher = serde_json::from_str(&json).unwrap();
            assert_eq!(back, cipher);
        }
    }

    #[test]
    fn phase3_cipher_serde_uses_hyphens() {
        let json = serde_json::to_string(&Phase3Cipher::ChaCha20Poly1305).unwrap();
        assert_eq!(json, "\"chacha20-poly1305\"");
    }

    #[test]
    fn key_derivation_is_deterministic() {
        let hk = [42u8; 32];
        let cn = [1u8; 32];
        let sn = [2u8; 32];

        let k1 = Phase3Keys::derive(&hk, &cn, &sn, false).unwrap();
        let k2 = Phase3Keys::derive(&hk, &cn, &sn, false).unwrap();

        assert_eq!(k1.encrypt_key, k2.encrypt_key);
        assert_eq!(k1.decrypt_key, k2.decrypt_key);
    }

    #[test]
    fn client_server_keys_are_mirrored() {
        let hk = [42u8; 32];
        let cn = [1u8; 32];
        let sn = [2u8; 32];

        let server_keys = Phase3Keys::derive(&hk, &cn, &sn, false).unwrap();
        let client_keys = Phase3Keys::derive(&hk, &cn, &sn, true).unwrap();

        assert_eq!(server_keys.encrypt_key, client_keys.decrypt_key);
        assert_eq!(server_keys.decrypt_key, client_keys.encrypt_key);
    }

    #[test]
    fn encrypt_decrypt_round_trip() {
        let hk = [42u8; 32];
        let cn = [1u8; 32];
        let sn = [2u8; 32];

        let server_keys = Phase3Keys::derive(&hk, &cn, &sn, false).unwrap();
        let client_keys = Phase3Keys::derive(&hk, &cn, &sn, true).unwrap();

        let plaintext = b"hello BTSP Phase 3";
        let encrypted = server_keys.encrypt(plaintext).unwrap();

        assert_ne!(&encrypted[12..], plaintext.as_slice());
        assert!(encrypted.len() >= 28 + plaintext.len());

        let decrypted = client_keys.decrypt(&encrypted).unwrap();
        assert_eq!(decrypted, plaintext);
    }

    #[test]
    fn decrypt_rejects_short_frame() {
        let hk = [42u8; 32];
        let keys = Phase3Keys::derive(&hk, &[1; 32], &[2; 32], false).unwrap();

        let err = keys.decrypt(&[0u8; 27]);
        assert!(err.is_err());
        assert!(err.unwrap_err().to_string().contains("too short"));
    }

    #[test]
    fn decrypt_rejects_tampered_frame() {
        let hk = [42u8; 32];
        let server = Phase3Keys::derive(&hk, &[1; 32], &[2; 32], false).unwrap();
        let client = Phase3Keys::derive(&hk, &[1; 32], &[2; 32], true).unwrap();

        let mut encrypted = server.encrypt(b"original").unwrap();
        let last = encrypted.len() - 1;
        encrypted[last] ^= 0xFF;

        assert!(client.decrypt(&encrypted).is_err());
    }

    #[test]
    fn different_nonces_produce_different_keys() {
        let hk = [42u8; 32];
        let k1 = Phase3Keys::derive(&hk, &[1; 32], &[2; 32], false).unwrap();
        let k2 = Phase3Keys::derive(&hk, &[3; 32], &[4; 32], false).unwrap();
        assert_ne!(k1.encrypt_key, k2.encrypt_key);
    }

    #[test]
    fn negotiate_params_deserialize() {
        let json = serde_json::json!({
            "session_id": "abc123",
            "ciphers": ["chacha20-poly1305"],
            "client_nonce": "AAAA"
        });
        let params: NegotiateParams = serde_json::from_value(json).unwrap();
        assert_eq!(params.session_id, "abc123");
        assert_eq!(params.ciphers.len(), 1);
    }

    #[test]
    fn negotiate_result_serializes_correctly() {
        let result = NegotiateResult {
            cipher: Phase3Cipher::ChaCha20Poly1305,
            server_nonce: "c2VydmVy".to_owned(),
        };
        let json = serde_json::to_value(&result).unwrap();
        assert_eq!(json["cipher"], "chacha20-poly1305");
        assert_eq!(json["server_nonce"], "c2VydmVy");
    }

    #[test]
    fn negotiate_result_null_cipher() {
        let result = NegotiateResult {
            cipher: Phase3Cipher::Null,
            server_nonce: String::new(),
        };
        let json = serde_json::to_value(&result).unwrap();
        assert_eq!(json["cipher"], "null");
    }

    #[tokio::test]
    async fn handle_negotiate_chacha20() {
        let hk = [42u8; 32];
        let session_id = "deadbeef01020304deadbeef01020304";
        let client_nonce = [7u8; 32];
        let b64 = base64::engine::general_purpose::STANDARD;

        let negotiate_req = serde_json::json!({
            "jsonrpc": "2.0",
            "method": "btsp.negotiate",
            "params": {
                "session_id": session_id,
                "ciphers": ["chacha20-poly1305"],
                "client_nonce": b64.encode(client_nonce),
            },
            "id": 1
        });

        let mut buf = Vec::new();
        let result = handle_negotiate(&mut buf, &hk, session_id, &negotiate_req).await.unwrap();

        assert!(result.is_some(), "should negotiate chacha20-poly1305");

        let resp_line = String::from_utf8(buf).unwrap();
        let resp: serde_json::Value = serde_json::from_str(resp_line.trim()).unwrap();
        assert_eq!(resp["result"]["cipher"], "chacha20-poly1305");
        assert!(!resp["result"]["server_nonce"].as_str().unwrap().is_empty());
    }

    #[tokio::test]
    async fn handle_negotiate_unsupported_cipher() {
        let hk = [42u8; 32];
        let session_id = "deadbeef01020304deadbeef01020304";

        let negotiate_req = serde_json::json!({
            "jsonrpc": "2.0",
            "method": "btsp.negotiate",
            "params": {
                "session_id": session_id,
                "ciphers": ["aes-256-gcm"],
                "client_nonce": "AAAA",
            },
            "id": 1
        });

        let mut buf = Vec::new();
        let result = handle_negotiate(&mut buf, &hk, session_id, &negotiate_req).await.unwrap();

        assert!(result.is_none(), "should fall back to null");

        let resp_line = String::from_utf8(buf).unwrap();
        let resp: serde_json::Value = serde_json::from_str(resp_line.trim()).unwrap();
        assert_eq!(resp["result"]["cipher"], "null");
    }

    #[tokio::test]
    async fn handle_negotiate_session_mismatch() {
        let hk = [42u8; 32];

        let negotiate_req = serde_json::json!({
            "jsonrpc": "2.0",
            "method": "btsp.negotiate",
            "params": {
                "session_id": "wrong-session",
                "ciphers": ["chacha20-poly1305"],
                "client_nonce": "AAAA",
            },
            "id": 1
        });

        let mut buf = Vec::new();
        let result =
            handle_negotiate(&mut buf, &hk, "correct-session", &negotiate_req).await.unwrap();

        assert!(result.is_none(), "should reject mismatched session");

        let resp_line = String::from_utf8(buf).unwrap();
        let resp: serde_json::Value = serde_json::from_str(resp_line.trim()).unwrap();
        assert!(resp.get("error").is_some());
    }

    #[tokio::test]
    async fn full_negotiate_and_encrypted_round_trip() {
        let b64 = base64::engine::general_purpose::STANDARD;
        let handshake_key = [99u8; 32];
        let session_id = "aabbccddee112233aabbccddee112233";
        let client_nonce = [11u8; 32];

        let negotiate_req = serde_json::json!({
            "jsonrpc": "2.0",
            "method": "btsp.negotiate",
            "params": {
                "session_id": session_id,
                "ciphers": ["chacha20-poly1305"],
                "client_nonce": b64.encode(client_nonce),
            },
            "id": 1
        });

        let mut buf = Vec::new();
        let server_keys = handle_negotiate(&mut buf, &handshake_key, session_id, &negotiate_req)
            .await
            .unwrap()
            .expect("should negotiate");

        let resp_line = String::from_utf8(buf).unwrap();
        let resp: serde_json::Value = serde_json::from_str(resp_line.trim()).unwrap();
        let server_nonce_b64 = resp["result"]["server_nonce"].as_str().unwrap();
        let server_nonce = b64.decode(server_nonce_b64).unwrap();

        let client_keys =
            Phase3Keys::derive(&handshake_key, &client_nonce, &server_nonce, true).unwrap();

        let request = b"{\"jsonrpc\":\"2.0\",\"method\":\"health.check\",\"id\":2}";
        let encrypted_req = client_keys.encrypt(request).unwrap();
        let decrypted_req = server_keys.decrypt(&encrypted_req).unwrap();
        assert_eq!(decrypted_req, request);

        let response = b"{\"jsonrpc\":\"2.0\",\"result\":\"ok\",\"id\":2}";
        let encrypted_resp = server_keys.encrypt(response).unwrap();
        let decrypted_resp = client_keys.decrypt(&encrypted_resp).unwrap();
        assert_eq!(decrypted_resp, response);
    }
}
