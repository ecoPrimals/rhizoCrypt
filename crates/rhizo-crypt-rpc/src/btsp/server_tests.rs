// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2024–2026 ecoPrimals Project

//! Tests for the BTSP server handshake implementation.

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
        handshake_key,
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

/// Minimal primalSpring-style JSON-line BTSP client for testing interop.
async fn client_handshake_jsonline<S>(
    stream: &mut S,
    family_seed: &[u8],
) -> Result<BtspSession, HandshakeError>
where
    S: AsyncReadExt + AsyncWriteExt + Unpin,
{
    let b64 = base64::engine::general_purpose::STANDARD;
    let handshake_key = derive_handshake_key(family_seed)?;

    let client_secret = EphemeralSecret::random_from_rng(rand::thread_rng());
    let client_public = PublicKey::from(&client_secret);

    let hello = ClientHelloWire {
        protocol: "btsp".to_owned(),
        version: BTSP_VERSION,
        client_ephemeral_pub: b64.encode(client_public.as_bytes()),
    };
    let hello_bytes = serde_json::to_vec(&hello)?;
    framing::write_json_line(stream, &hello_bytes).await?;

    let sh_bytes = framing::read_json_line(stream).await?;
    let server_hello: ServerHelloWire = serde_json::from_slice(&sh_bytes)?;

    let challenge = b64
        .decode(&server_hello.challenge)
        .map_err(|e| HandshakeError::KeyDerivation(format!("challenge b64: {e}")))?;
    let server_pub_bytes = b64
        .decode(&server_hello.server_ephemeral_pub)
        .map_err(|e| HandshakeError::KeyDerivation(format!("server pub b64: {e}")))?;

    let mut mac = HmacSha256::new_from_slice(&handshake_key)
        .map_err(|e| HandshakeError::KeyDerivation(format!("HMAC init: {e}")))?;
    mac.update(&challenge);
    mac.update(client_public.as_bytes());
    mac.update(&server_pub_bytes);
    let hmac_result = mac.finalize().into_bytes();

    let cr = ChallengeResponseWire {
        response: b64.encode(hmac_result),
        preferred_cipher: "null".to_owned(),
    };
    let cr_bytes = serde_json::to_vec(&cr)?;
    framing::write_json_line(stream, &cr_bytes).await?;

    let complete_bytes = framing::read_json_line(stream).await?;
    let complete: HandshakeCompleteWire = serde_json::from_slice(&complete_bytes)?;

    let session_id_bytes = hex::decode(&complete.session_id)
        .map_err(|e| HandshakeError::KeyDerivation(format!("session_id hex: {e}")))?;
    let session_id: [u8; 16] = session_id_bytes
        .try_into()
        .map_err(|_| HandshakeError::KeyDerivation("session_id not 16 bytes".into()))?;

    let server_public_arr: [u8; 32] = server_pub_bytes
        .try_into()
        .map_err(|_| HandshakeError::KeyDerivation("server pub not 32 bytes".into()))?;
    let server_public = PublicKey::from(server_public_arr);
    let shared_secret = client_secret.diffie_hellman(&server_public);

    let hk = Hkdf::<Sha256>::new(Some(&session_id), shared_secret.as_bytes());
    let mut encrypt_key = [0u8; 32];
    let mut decrypt_key = [0u8; 32];
    hk.expand(b"client-encrypt", &mut encrypt_key)
        .map_err(|e| HandshakeError::KeyDerivation(format!("{e}")))?;
    hk.expand(b"client-decrypt", &mut decrypt_key)
        .map_err(|e| HandshakeError::KeyDerivation(format!("{e}")))?;

    let cipher = match complete.cipher.as_str() {
        "chacha20_poly1305" => BtspCipher::Chacha20Poly1305,
        "hmac_plain" => BtspCipher::HmacPlain,
        _ => BtspCipher::Null,
    };

    Ok(BtspSession {
        cipher,
        session_id,
        keys: SessionKeys {
            encrypt_key,
            decrypt_key,
        },
        handshake_key,
    })
}

#[tokio::test]
async fn jsonline_handshake_round_trip() {
    let family_seed = b"test-family-seed-for-btsp-test!!";
    let (mut client_stream, mut server_stream) = duplex(8192);

    let server_handle = tokio::spawn(async move {
        let mut first_line = String::new();
        let mut byte = [0u8; 1];
        loop {
            use tokio::io::AsyncReadExt;
            let n = server_stream.read(&mut byte).await.expect("read byte");
            if n == 0 || byte[0] == b'\n' {
                break;
            }
            first_line.push(byte[0] as char);
        }
        BtspServer::accept_handshake_jsonline(
            &mut server_stream,
            family_seed,
            first_line.as_bytes(),
        )
        .await
    });

    let client_result = client_handshake_jsonline(&mut client_stream, family_seed).await;
    let server_result = server_handle.await.expect("server join");

    let client_session = client_result.expect("client handshake");
    let server_session = server_result.expect("server handshake");

    assert_eq!(client_session.cipher, BtspCipher::Null);
    assert_eq!(server_session.cipher, BtspCipher::Null);
    assert_eq!(client_session.session_id, server_session.session_id);

    assert_eq!(client_session.keys.encrypt_key, server_session.keys.decrypt_key);
    assert_eq!(client_session.keys.decrypt_key, server_session.keys.encrypt_key);
}

#[tokio::test]
async fn jsonline_handshake_fails_with_wrong_seed() {
    let (mut client_stream, mut server_stream) = duplex(8192);

    let server_handle = tokio::spawn(async move {
        let mut first_line = String::new();
        let mut byte = [0u8; 1];
        loop {
            use tokio::io::AsyncReadExt;
            let n = server_stream.read(&mut byte).await.expect("read");
            if n == 0 || byte[0] == b'\n' {
                break;
            }
            first_line.push(byte[0] as char);
        }
        BtspServer::accept_handshake_jsonline(
            &mut server_stream,
            b"server-seed-correct!",
            first_line.as_bytes(),
        )
        .await
    });

    let _ = client_handshake_jsonline(&mut client_stream, b"client-seed-WRONG!!!").await;
    let server_result = server_handle.await.expect("server join");
    assert!(server_result.is_err(), "server should reject wrong family seed");
}

#[tokio::test]
async fn send_handshake_error_jsonline_writes_line() {
    let mut buf = Vec::new();
    BtspServer::send_handshake_error_jsonline(&mut buf, "bad_key_length")
        .await
        .expect("send error");
    assert!(!buf.is_empty());
    assert_eq!(buf.last(), Some(&b'\n'));

    let mut cursor = std::io::Cursor::new(buf);
    let line = framing::read_json_line(&mut cursor).await.expect("read line");
    let val: serde_json::Value = serde_json::from_slice(&line).expect("parse json");
    assert_eq!(val["error"], "handshake_failed");
    assert_eq!(val["reason"], "bad_key_length");
}

#[test]
fn wire_types_serde_roundtrip() {
    let b64 = base64::engine::general_purpose::STANDARD;
    let hello = ClientHelloWire {
        protocol: "btsp".to_owned(),
        version: 1,
        client_ephemeral_pub: b64.encode([0xAA; 32]),
    };
    let json = serde_json::to_string(&hello).expect("serialize");
    assert!(json.contains("\"protocol\":\"btsp\""));
    let back: ClientHelloWire = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(back.decode_pub().expect("decode"), [0xAA; 32]);
}
