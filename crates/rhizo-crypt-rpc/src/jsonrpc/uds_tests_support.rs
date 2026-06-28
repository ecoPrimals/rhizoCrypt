// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2024–2026 ecoPrimals Project

//! Shared helpers for UDS integration tests.

#![expect(clippy::unwrap_used, clippy::expect_used, reason = "test code")]

use rhizo_crypt_core::{PrimalLifecycle, RhizoCrypt, RhizoCryptConfig};
use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

pub async fn test_primal() -> Arc<RhizoCrypt> {
    let mut p = RhizoCrypt::new(RhizoCryptConfig::default());
    p.start().await.unwrap();
    Arc::new(p)
}

/// Read one newline-terminated JSON message from a raw async stream.
pub async fn read_json_line_raw(
    client: &mut tokio::net::UnixStream,
    buf: &mut Vec<u8>,
) -> serde_json::Value {
    buf.clear();
    buf.resize(4096, 0);
    let mut total = 0;
    loop {
        let n = client.read(&mut buf[total..]).await.unwrap();
        total += n;
        if buf[..total].contains(&b'\n') || n == 0 {
            break;
        }
    }
    serde_json::from_str(std::str::from_utf8(&buf[..total]).unwrap().trim()).unwrap()
}

/// Send an encrypted JSON-RPC request and read the encrypted response.
pub async fn encrypted_roundtrip(
    client: &mut tokio::net::UnixStream,
    keys: &crate::btsp::phase3::Phase3Keys,
    request: &[u8],
) -> serde_json::Value {
    let encrypted = keys.encrypt(request).unwrap();
    let len: u32 = encrypted.len().try_into().unwrap();
    client.write_all(&len.to_be_bytes()).await.unwrap();
    client.write_all(&encrypted).await.unwrap();
    client.flush().await.unwrap();

    let mut len_buf = [0u8; 4];
    client.read_exact(&mut len_buf).await.unwrap();
    let resp_len = u32::from_be_bytes(len_buf) as usize;
    assert!(resp_len > 28, "encrypted frame must include nonce + tag: {resp_len}");

    let mut resp_frame = vec![0u8; resp_len];
    client.read_exact(&mut resp_frame).await.unwrap();

    let decrypted = keys.decrypt(&resp_frame).unwrap();
    serde_json::from_str(std::str::from_utf8(&decrypted).unwrap()).unwrap()
}

/// Complete the Phase 2 JSON-line BTSP handshake on the client side.
/// Returns `(session_id, handshake_key)`.
pub async fn client_phase2_handshake(
    client: &mut tokio::net::UnixStream,
    family_seed: &[u8],
) -> (String, [u8; 32]) {
    use base64::Engine;
    use hmac::{Hmac, Mac};
    use tokio::io::AsyncWriteExt;
    use x25519_dalek::{EphemeralSecret, PublicKey};

    type HmacSha256 = Hmac<sha2::Sha256>;
    let b64 = base64::engine::general_purpose::STANDARD;

    let client_secret = EphemeralSecret::random_from_rng(rand::thread_rng());
    let client_public = PublicKey::from(&client_secret);
    let hello = serde_json::json!({
        "protocol": "btsp", "version": 1,
        "client_ephemeral_pub": b64.encode(client_public.as_bytes())
    });
    client.write_all(format!("{hello}\n").as_bytes()).await.unwrap();

    let mut buf = vec![0u8; 4096];
    let sh = read_json_line_raw(client, &mut buf).await;
    assert_eq!(sh["version"], 1);

    let server_pub = b64.decode(sh["server_ephemeral_pub"].as_str().unwrap()).unwrap();
    let challenge = b64.decode(sh["challenge"].as_str().unwrap()).unwrap();
    let session_id = sh["session_id"].as_str().unwrap().to_owned();

    let handshake_key = {
        use hkdf::Hkdf;
        let hk = Hkdf::<sha2::Sha256>::new(Some(b"btsp-v1"), family_seed);
        let mut okm = [0u8; 32];
        hk.expand(b"handshake", &mut okm).unwrap();
        okm
    };

    let mut mac = HmacSha256::new_from_slice(&handshake_key).expect("HMAC init");
    mac.update(&challenge);
    mac.update(client_public.as_bytes());
    mac.update(&server_pub);
    let hmac_result = mac.finalize().into_bytes();

    let cr = serde_json::json!({
        "response": b64.encode(hmac_result),
        "preferred_cipher": "null"
    });
    client.write_all(format!("{cr}\n").as_bytes()).await.unwrap();

    let hc = read_json_line_raw(client, &mut buf).await;
    assert_eq!(hc["cipher"], "null");

    (session_id, handshake_key)
}

/// Complete the Phase 2 length-prefixed BTSP handshake on the client side.
pub async fn client_length_prefixed_handshake(
    client: &mut tokio::net::UnixStream,
    family_seed: &[u8],
) -> Result<(), crate::btsp::HandshakeError> {
    use crate::btsp::framing;
    use crate::btsp::types::{
        BTSP_VERSION, BtspCipher, ChallengeResponse, ClientHello, HANDSHAKE_HKDF_INFO,
        HANDSHAKE_HKDF_SALT,
    };
    use hkdf::Hkdf;
    use hmac::{Hmac, Mac};
    use sha2::Sha256;
    use x25519_dalek::{EphemeralSecret, PublicKey};

    type HmacSha256 = Hmac<Sha256>;

    let hk = Hkdf::<Sha256>::new(Some(HANDSHAKE_HKDF_SALT), family_seed);
    let mut handshake_key = [0u8; 32];
    hk.expand(HANDSHAKE_HKDF_INFO, &mut handshake_key)
        .map_err(|e| crate::btsp::HandshakeError::KeyDerivation(format!("handshake HKDF: {e}")))?;

    let client_secret = EphemeralSecret::random_from_rng(rand::thread_rng());
    let client_public = PublicKey::from(&client_secret);

    let hello = ClientHello {
        version: BTSP_VERSION,
        client_ephemeral_pub: *client_public.as_bytes(),
    };
    let hello_bytes = serde_json::to_vec(&hello)?;
    framing::write_frame(client, &hello_bytes).await?;

    let sh_bytes = framing::read_frame(client).await?;
    let server_hello: crate::btsp::types::ServerHello = serde_json::from_slice(&sh_bytes)?;

    let mut mac = HmacSha256::new_from_slice(&handshake_key)
        .map_err(|e| crate::btsp::HandshakeError::KeyDerivation(format!("HMAC init: {e}")))?;
    mac.update(&server_hello.challenge);
    mac.update(client_public.as_bytes());
    mac.update(&server_hello.server_ephemeral_pub);
    let hmac_result = mac.finalize().into_bytes();

    let mut response_bytes = [0u8; 32];
    response_bytes.copy_from_slice(&hmac_result);

    let cr = ChallengeResponse {
        response: response_bytes,
        preferred_cipher: BtspCipher::Null,
    };
    let cr_bytes = serde_json::to_vec(&cr)?;
    framing::write_frame(client, &cr_bytes).await?;

    let complete_bytes = framing::read_frame(client).await?;
    let _complete: crate::btsp::types::HandshakeComplete = serde_json::from_slice(&complete_bytes)?;

    Ok(())
}
