// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2024–2026 ecoPrimals Project

//! BTSP handshake and Phase 3 encrypted transport tests over UDS.

#![expect(clippy::unwrap_used, clippy::expect_used, reason = "test code")]

use super::tests_support::{
    client_phase2_handshake, encrypted_roundtrip, read_json_line_raw, test_primal,
};
use super::*;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

/// Full BTSP JSON-line handshake over a real `UnixStream` pair, testing the
/// routing in `handle_uds_connection` end-to-end:
/// `ClientHello` → `ServerHello` → `ChallengeResponse` → `HandshakeComplete` → JSON-RPC
#[tokio::test]
async fn test_btsp_jsonline_handshake_over_uds() {
    use base64::Engine;
    use hmac::{Hmac, Mac};
    use sha2::Sha256;
    use x25519_dalek::{EphemeralSecret, PublicKey};

    type HmacSha256 = Hmac<Sha256>;

    let family_seed = b"integration-test-family-seed-ok!";
    let primal = test_primal().await;
    let server = crate::service::RhizoCryptRpcServer::new(primal);

    let (server_raw, client_raw) = std::os::unix::net::UnixStream::pair().unwrap();
    server_raw.set_nonblocking(true).unwrap();
    client_raw.set_nonblocking(true).unwrap();
    let server_stream = tokio::net::UnixStream::from_std(server_raw).unwrap();
    let mut client = tokio::net::UnixStream::from_std(client_raw).unwrap();

    let server_handle = tokio::spawn(async move {
        handle_uds_connection(server_stream, server, true, Some(family_seed)).await
    });

    let b64 = base64::engine::general_purpose::STANDARD;

    let client_secret = EphemeralSecret::random_from_rng(rand::thread_rng());
    let client_public = PublicKey::from(&client_secret);
    let hello = serde_json::json!({
        "protocol": "btsp",
        "version": 1,
        "client_ephemeral_pub": b64.encode(client_public.as_bytes())
    });
    let hello_line = format!("{hello}\n");
    client.write_all(hello_line.as_bytes()).await.unwrap();

    let mut buf = vec![0u8; 4096];
    let mut total = 0;
    loop {
        let n = client.read(&mut buf[total..]).await.unwrap();
        total += n;
        if buf[..total].contains(&b'\n') || n == 0 {
            break;
        }
    }
    let server_hello_line = std::str::from_utf8(&buf[..total]).unwrap();
    let sh: serde_json::Value = serde_json::from_str(server_hello_line.trim()).unwrap();

    assert_eq!(sh["version"], 1);
    assert!(sh["server_ephemeral_pub"].is_string());
    assert!(sh["challenge"].is_string());
    assert!(sh["session_id"].is_string());

    let server_pub_bytes = b64.decode(sh["server_ephemeral_pub"].as_str().unwrap()).unwrap();
    let challenge_bytes = b64.decode(sh["challenge"].as_str().unwrap()).unwrap();

    let handshake_key = {
        use hkdf::Hkdf;
        let hk = Hkdf::<sha2::Sha256>::new(Some(b"btsp-v1"), family_seed);
        let mut okm = [0u8; 32];
        hk.expand(b"handshake", &mut okm).unwrap();
        okm
    };

    let mut mac = HmacSha256::new_from_slice(&handshake_key).expect("HMAC init");
    mac.update(&challenge_bytes);
    mac.update(client_public.as_bytes());
    mac.update(&server_pub_bytes);
    let hmac_result = mac.finalize().into_bytes();

    let cr = serde_json::json!({
        "response": b64.encode(hmac_result),
        "preferred_cipher": "null"
    });
    let cr_line = format!("{cr}\n");
    client.write_all(cr_line.as_bytes()).await.unwrap();

    total = 0;
    loop {
        let n = client.read(&mut buf[total..]).await.unwrap();
        total += n;
        if buf[..total].contains(&b'\n') || n == 0 {
            break;
        }
    }
    let complete_line = std::str::from_utf8(&buf[..total]).unwrap();
    let hc: serde_json::Value = serde_json::from_str(complete_line.trim()).unwrap();
    assert_eq!(hc["cipher"], "null");
    assert!(hc["session_id"].is_string());

    let rpc_req = r#"{"jsonrpc":"2.0","method":"health.check","params":{},"id":1}"#;
    client.write_all(format!("{rpc_req}\n").as_bytes()).await.unwrap();

    total = 0;
    loop {
        let n = client.read(&mut buf[total..]).await.unwrap();
        total += n;
        if buf[..total].contains(&b'\n') || n == 0 {
            break;
        }
    }
    let rpc_resp_line = std::str::from_utf8(&buf[..total]).unwrap();
    let resp: serde_json::Value = serde_json::from_str(rpc_resp_line.trim()).unwrap();
    assert_eq!(resp["jsonrpc"], "2.0");
    assert!(resp["result"].is_object());
    assert_eq!(resp["id"], 1);

    client.shutdown().await.unwrap();
    let server_result = server_handle.await.unwrap();
    assert!(server_result.is_ok(), "server should complete cleanly");
}

/// Guidestone 157/170: Full BTSP Phase 3 encrypted transport integration test.
///
/// Exercises the complete path: handshake → `btsp.negotiate(chacha20-poly1305)` →
/// encrypted `health.check` + `dag.session.create` round-trips over a real
/// `UnixStream` pair, verifying that `serve_after_handshake` switches to
/// `handle_encrypted_connection` and all subsequent frames are encrypted.
#[tokio::test]
async fn test_btsp_phase3_encrypted_transport_over_uds() {
    use base64::Engine;

    let family_seed = b"integration-test-family-seed-ok!";
    let primal = test_primal().await;
    let server = crate::service::RhizoCryptRpcServer::new(primal);

    let (server_raw, client_raw) = std::os::unix::net::UnixStream::pair().unwrap();
    server_raw.set_nonblocking(true).unwrap();
    client_raw.set_nonblocking(true).unwrap();
    let server_stream = tokio::net::UnixStream::from_std(server_raw).unwrap();
    let mut client = tokio::net::UnixStream::from_std(client_raw).unwrap();

    let server_handle = tokio::spawn(async move {
        handle_uds_connection(server_stream, server, true, Some(family_seed)).await
    });

    let (session_id, handshake_key) = client_phase2_handshake(&mut client, family_seed).await;

    // --- Phase 3: btsp.negotiate → chacha20-poly1305 ---

    let b64 = base64::engine::general_purpose::STANDARD;
    let client_nonce = {
        let mut n = [0u8; 32];
        rand::RngCore::fill_bytes(&mut rand::thread_rng(), &mut n);
        n
    };

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
    client.write_all(format!("{negotiate_req}\n").as_bytes()).await.unwrap();

    let mut buf = vec![0u8; 4096];
    let negotiate_resp = read_json_line_raw(&mut client, &mut buf).await;

    assert_eq!(
        negotiate_resp["result"]["cipher"], "chacha20-poly1305",
        "server should select chacha20-poly1305: {negotiate_resp}"
    );
    let server_nonce =
        b64.decode(negotiate_resp["result"]["server_nonce"].as_str().unwrap()).unwrap();

    let client_keys =
        crate::btsp::phase3::Phase3Keys::derive(&handshake_key, &client_nonce, &server_nonce, true)
            .unwrap();

    // --- Encrypted transport: verify multiple requests go through encrypted path ---

    let resp1 = encrypted_roundtrip(
        &mut client,
        &client_keys,
        br#"{"jsonrpc":"2.0","method":"health.check","params":{},"id":2}"#,
    )
    .await;
    assert_eq!(resp1["jsonrpc"], "2.0");
    assert_eq!(resp1["id"], 2);
    assert!(resp1["result"].is_object(), "health.check: {resp1}");

    let resp2 = encrypted_roundtrip(
        &mut client,
        &client_keys,
        br#"{"jsonrpc":"2.0","method":"dag.session.create","params":{"description":"Phase 3 test","session_type":"General"},"id":3}"#,
    )
    .await;
    assert_eq!(resp2["jsonrpc"], "2.0");
    assert_eq!(resp2["id"], 3);
    assert!(resp2["result"].is_string(), "dag.session.create: {resp2}");

    client.shutdown().await.unwrap();
    let server_result = server_handle.await.unwrap();
    assert!(server_result.is_ok(), "server should complete cleanly");
}

/// Verify that a partial/invalid `ClientHello` gets an error response, not
/// a silent connection reset.
#[tokio::test]
async fn test_btsp_jsonline_invalid_key_returns_error() {
    let family_seed = b"integration-test-family-seed-ok!";
    let primal = test_primal().await;
    let server = crate::service::RhizoCryptRpcServer::new(primal);

    let (server_raw, client_raw) = std::os::unix::net::UnixStream::pair().unwrap();
    server_raw.set_nonblocking(true).unwrap();
    client_raw.set_nonblocking(true).unwrap();
    let server_stream = tokio::net::UnixStream::from_std(server_raw).unwrap();
    let mut client = tokio::net::UnixStream::from_std(client_raw).unwrap();

    let server_handle = tokio::spawn(async move {
        handle_uds_connection(server_stream, server, true, Some(family_seed)).await
    });

    let hello = r#"{"protocol":"btsp","version":1,"client_ephemeral_pub":"dGVzdA=="}"#;
    client.write_all(format!("{hello}\n").as_bytes()).await.unwrap();

    let mut buf = vec![0u8; 4096];
    let mut total = 0;
    loop {
        let n = client.read(&mut buf[total..]).await.unwrap();
        total += n;
        if n == 0 || buf[..total].contains(&b'\n') {
            break;
        }
    }
    assert!(total > 0, "should receive error response, not connection reset");
    let error_line = std::str::from_utf8(&buf[..total]).unwrap();
    let err: serde_json::Value = serde_json::from_str(error_line.trim()).unwrap();
    assert_eq!(err["error"], "handshake_failed");
    assert!(
        err["reason"].as_str().unwrap().contains("32 bytes"),
        "reason should mention expected key length: {}",
        err["reason"]
    );

    let server_result = server_handle.await.unwrap();
    assert!(server_result.is_err(), "server should report handshake failure");
}
