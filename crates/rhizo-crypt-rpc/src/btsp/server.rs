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

use super::framing;
use super::types::{
    BTSP_VERSION, BtspCipher, ChallengeResponse, ClientHello, HANDSHAKE_HKDF_INFO,
    HANDSHAKE_HKDF_SALT, HandshakeComplete, HandshakeError, ServerHello, SessionKeys,
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
        })
    }

    /// Send a handshake failure error frame and close.
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
}

#[cfg(test)]
#[expect(clippy::expect_used, reason = "test code")]
mod tests {
    use super::*;
    use hkdf::Hkdf;
    use tokio::io::duplex;

    /// Minimal BTSP client for testing the server accept path.
    async fn client_handshake<S>(
        stream: &mut S,
        family_seed: &[u8],
    ) -> Result<BtspSession, HandshakeError>
    where
        S: AsyncReadExt + AsyncWriteExt + Unpin,
    {
        let handshake_key = derive_handshake_key(family_seed)?;

        let client_secret = EphemeralSecret::random_from_rng(rand::thread_rng());
        let client_public = PublicKey::from(&client_secret);

        let hello = ClientHello {
            version: BTSP_VERSION,
            client_ephemeral_pub: *client_public.as_bytes(),
        };
        let hello_bytes = serde_json::to_vec(&hello)?;
        framing::write_frame(stream, &hello_bytes).await?;

        let sh_bytes = framing::read_frame(stream).await?;
        let server_hello: ServerHello = serde_json::from_slice(&sh_bytes)?;

        let mut mac = HmacSha256::new_from_slice(&handshake_key)
            .map_err(|e| HandshakeError::KeyDerivation(format!("HMAC init: {e}")))?;
        mac.update(&server_hello.challenge);
        mac.update(client_public.as_bytes());
        mac.update(&server_hello.server_ephemeral_pub);
        let hmac_result = mac.finalize().into_bytes();

        let mut response_bytes = [0u8; 32];
        response_bytes.copy_from_slice(&hmac_result);

        let cr = ChallengeResponse {
            response: response_bytes,
            preferred_cipher: BtspCipher::Chacha20Poly1305,
        };
        let cr_bytes = serde_json::to_vec(&cr)?;
        framing::write_frame(stream, &cr_bytes).await?;

        let complete_bytes = framing::read_frame(stream).await?;
        let complete: HandshakeComplete = serde_json::from_slice(&complete_bytes)?;

        let server_public = PublicKey::from(server_hello.server_ephemeral_pub);
        let shared_secret = client_secret.diffie_hellman(&server_public);

        let hk = Hkdf::<Sha256>::new(Some(&complete.session_id), shared_secret.as_bytes());
        let mut encrypt_key = [0u8; 32];
        let mut decrypt_key = [0u8; 32];
        hk.expand(b"client-encrypt", &mut encrypt_key)
            .map_err(|e| HandshakeError::KeyDerivation(format!("{e}")))?;
        hk.expand(b"client-decrypt", &mut decrypt_key)
            .map_err(|e| HandshakeError::KeyDerivation(format!("{e}")))?;

        Ok(BtspSession {
            cipher: complete.cipher,
            session_id: complete.session_id,
            keys: SessionKeys {
                encrypt_key,
                decrypt_key,
            },
        })
    }

    #[tokio::test]
    async fn full_handshake_round_trip() {
        let family_seed = b"test-family-seed-for-btsp-test!!";
        let (mut client_stream, mut server_stream) = duplex(8192);

        let server_handle = tokio::spawn(async move {
            BtspServer::accept_handshake(&mut server_stream, family_seed).await
        });

        let client_result = client_handshake(&mut client_stream, family_seed).await;
        let server_result = server_handle.await.expect("server join");

        let client_session = client_result.expect("client handshake");
        let server_session = server_result.expect("server handshake");

        assert_eq!(client_session.cipher, BtspCipher::Chacha20Poly1305);
        assert_eq!(server_session.cipher, BtspCipher::Chacha20Poly1305);
        assert_eq!(client_session.session_id, server_session.session_id);

        assert_eq!(client_session.keys.encrypt_key, server_session.keys.decrypt_key);
        assert_eq!(client_session.keys.decrypt_key, server_session.keys.encrypt_key);
    }

    #[tokio::test]
    async fn handshake_fails_with_wrong_seed() {
        let (mut client_stream, mut server_stream) = duplex(8192);

        let server_handle = tokio::spawn(async move {
            BtspServer::accept_handshake(&mut server_stream, b"server-seed-correct!").await
        });

        let _ = client_handshake(&mut client_stream, b"client-seed-WRONG!!!").await;
        let server_result = server_handle.await.expect("server join");
        assert!(server_result.is_err(), "server should reject wrong family seed");
    }

    #[tokio::test]
    async fn send_handshake_error_writes_frame() {
        let mut buf = Vec::new();
        BtspServer::send_handshake_error(&mut buf).await.expect("send error");
        assert!(!buf.is_empty());

        let mut cursor = std::io::Cursor::new(buf);
        let frame = framing::read_frame(&mut cursor).await.expect("read frame");
        let val: serde_json::Value = serde_json::from_slice(&frame).expect("parse json");
        assert_eq!(val["error"], "handshake_failed");
    }

    #[test]
    fn handshake_key_derivation_deterministic() {
        let seed = b"deterministic-seed-test";
        let k1 = derive_handshake_key(seed).expect("k1");
        let k2 = derive_handshake_key(seed).expect("k2");
        assert_eq!(k1, k2);
    }

    #[test]
    fn different_seeds_produce_different_keys() {
        let k1 = derive_handshake_key(b"seed-alpha").expect("k1");
        let k2 = derive_handshake_key(b"seed-beta").expect("k2");
        assert_ne!(k1, k2);
    }

    #[test]
    fn session_keys_encrypt_differs_from_decrypt() {
        let shared = [42u8; 32];
        let sid = [7u8; 16];
        let keys = derive_session_keys(&shared, &sid).expect("keys");
        assert_ne!(keys.encrypt_key, keys.decrypt_key);
    }

    #[test]
    fn cipher_as_str() {
        assert_eq!(BtspCipher::Chacha20Poly1305.as_str(), "chacha20_poly1305");
        assert_eq!(BtspCipher::HmacPlain.as_str(), "hmac_plain");
        assert_eq!(BtspCipher::Null.as_str(), "null");
    }
}
