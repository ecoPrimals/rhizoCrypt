// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2024–2026 ecoPrimals Project

//! BTSP server-side handshake implementation.
//!
//! Performs the 4-step handshake as the **accepting** side per
//! `BTSP_PROTOCOL_STANDARD.md`:
//!
//! 1. Receive `ClientHello` (client's ephemeral X25519 public key)
//! 2. Send `ServerHello` (server public key + random challenge)
//! 3. Receive and verify `ChallengeResponse` (HMAC proof of family membership)
//! 4. Send `HandshakeComplete` (negotiated cipher + session ID)
//!
//! After handshake, derives session keys via ECDH + HKDF.

use hmac::{Hmac, Mac};
use rand::RngCore;
use sha2::Sha256;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use x25519_dalek::{EphemeralSecret, PublicKey};

use base64::Engine;

use super::framing;
use super::types::{
    BTSP_VERSION, BtspCipher, ChallengeResponse, ChallengeResponseWire, ClientHello,
    ClientHelloWire, HANDSHAKE_HKDF_INFO, HANDSHAKE_HKDF_SALT, HandshakeComplete,
    HandshakeCompleteWire, HandshakeError, ServerHello, ServerHelloWire, SessionKeys,
};

type HmacSha256 = Hmac<Sha256>;

/// Derive the handshake key from a family seed using HKDF-SHA256.
fn derive_handshake_key(family_seed: &[u8]) -> Result<[u8; 32], HandshakeError> {
    use hkdf::Hkdf;

    let hk = Hkdf::<Sha256>::new(Some(HANDSHAKE_HKDF_SALT), family_seed);
    let mut okm = [0u8; 32];
    hk.expand(HANDSHAKE_HKDF_INFO, &mut okm)
        .map_err(|e| HandshakeError::KeyDerivation(format!("handshake HKDF: {e}")))?;
    Ok(okm)
}

/// Derive directional session keys (server perspective: encrypt = server→client).
fn derive_session_keys(
    shared_secret: &[u8; 32],
    session_id: &[u8; 16],
) -> Result<SessionKeys, HandshakeError> {
    use hkdf::Hkdf;

    let hk = Hkdf::<Sha256>::new(Some(session_id), shared_secret);
    let mut encrypt_key = [0u8; 32];
    let mut decrypt_key = [0u8; 32];

    hk.expand(b"client-decrypt", &mut encrypt_key)
        .map_err(|e| HandshakeError::KeyDerivation(format!("encrypt key: {e}")))?;
    hk.expand(b"client-encrypt", &mut decrypt_key)
        .map_err(|e| HandshakeError::KeyDerivation(format!("decrypt key: {e}")))?;

    Ok(SessionKeys {
        encrypt_key,
        decrypt_key,
    })
}

/// Completed BTSP session with negotiated parameters.
pub struct BtspSession {
    /// Negotiated cipher suite.
    pub cipher: BtspCipher,
    /// Session identifier (16 bytes).
    pub session_id: [u8; 16],
    /// Directional session keys.
    pub keys: SessionKeys,
    /// Handshake key preserved for Phase 3 negotiation (HKDF IKM).
    pub handshake_key: [u8; 32],
}

/// BTSP server — accepts and verifies handshakes on incoming connections.
pub struct BtspServer;

impl BtspServer {
    /// Perform the server-side BTSP handshake.
    ///
    /// Runs the 4-step X25519 + HMAC-SHA256 protocol. On success, returns
    /// a [`BtspSession`] with negotiated cipher and derived keys.
    ///
    /// # Errors
    ///
    /// Returns [`HandshakeError`] on version mismatch, family verification
    /// failure, or I/O errors.
    pub async fn accept_handshake<S>(
        stream: &mut S,
        family_seed: &[u8],
    ) -> Result<BtspSession, HandshakeError>
    where
        S: AsyncReadExt + AsyncWriteExt + Unpin,
    {
        let handshake_key = derive_handshake_key(family_seed)?;

        // Step 1: Receive ClientHello
        let hello_bytes = framing::read_frame(stream).await?;
        let client_hello: ClientHello = serde_json::from_slice(&hello_bytes)?;

        if client_hello.version != BTSP_VERSION {
            return Err(HandshakeError::VersionMismatch {
                expected: BTSP_VERSION,
                got: client_hello.version,
            });
        }

        // Generate server ephemeral keypair
        let server_secret = EphemeralSecret::random_from_rng(rand::thread_rng());
        let server_public = PublicKey::from(&server_secret);

        // Generate 32-byte random challenge
        let mut challenge = [0u8; 32];
        rand::thread_rng().fill_bytes(&mut challenge);

        // Step 2: Send ServerHello
        let server_hello = ServerHello {
            version: BTSP_VERSION,
            server_ephemeral_pub: *server_public.as_bytes(),
            challenge,
        };
        let sh_bytes = serde_json::to_vec(&server_hello)?;
        framing::write_frame(stream, &sh_bytes).await?;

        // Step 3: Receive and verify ChallengeResponse
        let cr_bytes = framing::read_frame(stream).await?;
        let challenge_response: ChallengeResponse = serde_json::from_slice(&cr_bytes)?;

        let mut mac = HmacSha256::new_from_slice(&handshake_key)
            .map_err(|e| HandshakeError::KeyDerivation(format!("HMAC init: {e}")))?;
        mac.update(&challenge);
        mac.update(&client_hello.client_ephemeral_pub);
        mac.update(server_public.as_bytes());

        mac.verify_slice(&challenge_response.response)
            .map_err(|_| HandshakeError::FamilyVerification)?;

        // Generate session ID
        let mut session_id = [0u8; 16];
        rand::thread_rng().fill_bytes(&mut session_id);

        let cipher = challenge_response.preferred_cipher;

        // Step 4: Send HandshakeComplete
        let complete = HandshakeComplete {
            cipher,
            session_id,
        };
        let complete_bytes = serde_json::to_vec(&complete)?;
        framing::write_frame(stream, &complete_bytes).await?;

        // Derive session keys via ECDH
        let client_public = PublicKey::from(client_hello.client_ephemeral_pub);
        let shared_secret = server_secret.diffie_hellman(&client_public);

        let keys = derive_session_keys(shared_secret.as_bytes(), &session_id)?;

        Ok(BtspSession {
            cipher,
            session_id,
            keys,
            handshake_key,
        })
    }

    /// Perform the server-side BTSP handshake using **JSON-line** framing.
    ///
    /// This is the interop path for primalSpring and other springs that send
    /// `{"protocol":"btsp","version":1,...}\n` instead of length-prefixed
    /// binary frames. The crypto is identical — only the wire encoding
    /// differs (base64 strings instead of raw byte arrays, newline-delimited
    /// instead of 4-byte length prefix).
    ///
    /// `client_hello_line` is the already-read first line (consumed during
    /// protocol auto-detection in the UDS accept loop).
    ///
    /// # Errors
    ///
    /// Returns [`HandshakeError`] on version mismatch, family verification
    /// failure, base64 decode errors, or I/O errors.
    pub async fn accept_handshake_jsonline<S>(
        stream: &mut S,
        family_seed: &[u8],
        client_hello_line: &[u8],
    ) -> Result<BtspSession, HandshakeError>
    where
        S: AsyncReadExt + AsyncWriteExt + Unpin,
    {
        let b64 = base64::engine::general_purpose::STANDARD;
        let handshake_key = derive_handshake_key(family_seed)?;

        // Step 1: Parse the already-read ClientHello (JSON-line)
        let client_hello: ClientHelloWire = serde_json::from_slice(client_hello_line)?;

        if client_hello.version != BTSP_VERSION {
            return Err(HandshakeError::VersionMismatch {
                expected: BTSP_VERSION,
                got: client_hello.version,
            });
        }

        let client_pub_bytes = client_hello.decode_pub()?;

        // Generate server ephemeral keypair
        let server_secret = EphemeralSecret::random_from_rng(rand::thread_rng());
        let server_public = PublicKey::from(&server_secret);

        // Generate 32-byte random challenge
        let mut challenge = [0u8; 32];
        rand::thread_rng().fill_bytes(&mut challenge);

        // Generate session ID early (primalSpring expects it in ServerHello)
        let mut session_id = [0u8; 16];
        rand::thread_rng().fill_bytes(&mut session_id);
        let session_id_hex = hex::encode(session_id);

        // Step 2: Send ServerHello (JSON-line)
        let server_hello = ServerHelloWire {
            version: BTSP_VERSION,
            server_ephemeral_pub: b64.encode(server_public.as_bytes()),
            challenge: b64.encode(challenge),
            session_id: session_id_hex.clone(),
        };
        let sh_bytes = serde_json::to_vec(&server_hello)?;
        framing::write_json_line(stream, &sh_bytes).await?;

        // Step 3: Receive and verify ChallengeResponse (JSON-line)
        let cr_bytes = framing::read_json_line(stream).await?;

        // primalSpring may send an error instead of a challenge response
        if let Ok(err_wire) = serde_json::from_slice::<super::types::HandshakeErrorWire>(&cr_bytes)
            && !err_wire.error.is_empty()
        {
            return Err(HandshakeError::KeyDerivation(format!(
                "client sent error: {}",
                err_wire.reason
            )));
        }

        let cr: ChallengeResponseWire = serde_json::from_slice(&cr_bytes)?;
        let response_bytes = cr.decode_response()?;

        let mut mac = HmacSha256::new_from_slice(&handshake_key)
            .map_err(|e| HandshakeError::KeyDerivation(format!("HMAC init: {e}")))?;
        mac.update(&challenge);
        mac.update(&client_pub_bytes);
        mac.update(server_public.as_bytes());

        mac.verify_slice(&response_bytes).map_err(|_| HandshakeError::FamilyVerification)?;

        // Resolve cipher from string
        let cipher = match cr.preferred_cipher.as_str() {
            "chacha20_poly1305" => BtspCipher::Chacha20Poly1305,
            "hmac_plain" => BtspCipher::HmacPlain,
            _ => BtspCipher::Null,
        };

        // Step 4: Send HandshakeComplete (JSON-line)
        let complete = HandshakeCompleteWire {
            cipher: cipher.as_str().to_owned(),
            session_id: session_id_hex,
        };
        let complete_bytes = serde_json::to_vec(&complete)?;
        framing::write_json_line(stream, &complete_bytes).await?;

        // Derive session keys via ECDH
        let client_public = PublicKey::from(client_pub_bytes);
        let shared_secret = server_secret.diffie_hellman(&client_public);

        let keys = derive_session_keys(shared_secret.as_bytes(), &session_id)?;

        Ok(BtspSession {
            cipher,
            session_id,
            keys,
            handshake_key,
        })
    }

    /// Send a handshake failure error frame (length-prefixed) and close.
    ///
    /// # Errors
    ///
    /// Returns I/O errors from the underlying stream.
    pub async fn send_handshake_error<S>(stream: &mut S) -> std::io::Result<()>
    where
        S: AsyncWriteExt + Unpin,
    {
        let error_msg = serde_json::to_vec(
            &serde_json::json!({"error": "handshake_failed", "reason": "family_verification"}),
        )
        .unwrap_or_else(|_| br#"{"error":"handshake_failed"}"#.to_vec());
        framing::write_frame(stream, &error_msg).await
    }

    /// Send a handshake failure as a JSON-line with the actual error reason.
    ///
    /// # Errors
    ///
    /// Returns I/O errors from the underlying stream.
    pub async fn send_handshake_error_jsonline<S>(
        stream: &mut S,
        reason: &str,
    ) -> std::io::Result<()>
    where
        S: AsyncWriteExt + Unpin,
    {
        let error_msg =
            serde_json::to_vec(&serde_json::json!({"error": "handshake_failed", "reason": reason}))
                .unwrap_or_else(|_| br#"{"error":"handshake_failed"}"#.to_vec());
        framing::write_json_line(stream, &error_msg).await
    }
}

#[cfg(test)]
#[expect(clippy::expect_used, reason = "test code")]
#[path = "server_tests.rs"]
mod tests;
