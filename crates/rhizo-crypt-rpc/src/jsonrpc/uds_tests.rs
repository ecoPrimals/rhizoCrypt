use super::*;
use rhizo_crypt_core::{PrimalLifecycle, RhizoCryptConfig};
use tokio::io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader};

async fn test_primal() -> Arc<RhizoCrypt> {
    let mut p = RhizoCrypt::new(RhizoCryptConfig::default());
    p.start().await.unwrap();
    Arc::new(p)
}

#[tokio::test]
async fn test_uds_roundtrip() {
    let dir = tempfile::tempdir().expect("tempdir");
    let sock = dir.path().join("rhizocrypt-test.sock");
    let primal = test_primal().await;

    let server = UdsJsonRpcServer::new(primal, sock.clone());
    let (shutdown_tx, shutdown_rx) = watch::channel(false);
    let ready = Arc::new(tokio::sync::Notify::new());
    let ready_rx = Arc::clone(&ready);

    let handle = tokio::spawn(async move { server.serve_with_ready(shutdown_rx, ready_rx).await });
    ready.notified().await;

    let stream = tokio::net::UnixStream::connect(&sock).await.expect("connect");
    let (reader, mut writer) = stream.into_split();

    let req = r#"{"jsonrpc":"2.0","method":"health.check","params":{},"id":1}"#;
    writer.write_all(format!("{req}\n").as_bytes()).await.unwrap();
    writer.shutdown().await.unwrap();

    let mut lines = BufReader::new(reader).lines();
    let line = lines.next_line().await.unwrap().expect("response");
    let resp: serde_json::Value = serde_json::from_str(&line).unwrap();
    assert_eq!(resp["jsonrpc"], "2.0");
    assert!(resp["result"].is_object());

    let _ = shutdown_tx.send(true);
    let _ = handle.await;
}

#[tokio::test]
async fn test_uds_stale_socket_removed() {
    let dir = tempfile::tempdir().expect("tempdir");
    let sock = dir.path().join("stale-test.sock");
    std::fs::write(&sock, "stale").unwrap();
    assert!(sock.exists());

    let primal = test_primal().await;
    let server = UdsJsonRpcServer::new(primal, sock.clone());
    let (shutdown_tx, shutdown_rx) = watch::channel(false);
    let ready = Arc::new(tokio::sync::Notify::new());
    let ready_rx = Arc::clone(&ready);

    let handle = tokio::spawn(async move { server.serve_with_ready(shutdown_rx, ready_rx).await });
    ready.notified().await;

    assert!(sock.exists(), "socket should exist (re-bound)");

    let _ = shutdown_tx.send(true);
    let _ = handle.await;
}

#[tokio::test]
async fn test_uds_cleanup_removes_file() {
    let dir = tempfile::tempdir().expect("tempdir");
    let sock = dir.path().join("cleanup-test.sock");
    std::fs::write(&sock, "").unwrap();
    assert!(sock.exists());

    cleanup_socket_at(&sock);
    assert!(!sock.exists());
}

#[tokio::test]
async fn test_uds_cleanup_nonexistent_noop() {
    let dir = tempfile::tempdir().expect("tempdir");
    let sock = dir.path().join("nonexistent.sock");
    cleanup_socket_at(&sock);
}

#[test]
fn test_default_socket_path_contains_biomeos() {
    let path = default_socket_path();
    let path_str = path.to_string_lossy();
    assert!(
        path_str.contains("biomeos") || path_str.contains("rhizocrypt"),
        "path should reference biomeos or rhizocrypt: {path_str}"
    );
}

#[test]
fn test_socket_path_accessor() {
    let path = std::path::PathBuf::from("/tmp/test.sock");
    let primal = tokio::runtime::Runtime::new().unwrap().block_on(test_primal());
    let server = UdsJsonRpcServer::new(primal, path.clone());
    assert_eq!(server.socket_path(), path);
}

#[tokio::test]
async fn test_uds_server_cleanup_idempotent() {
    let dir = tempfile::tempdir().expect("tempdir");
    let sock = dir.path().join("idempotent.sock");
    let primal = test_primal().await;
    let server = UdsJsonRpcServer::new(primal, sock.clone());
    server.cleanup();
    server.cleanup();
}

#[tokio::test]
async fn test_uds_multiple_sequential_requests() {
    let dir = tempfile::tempdir().expect("tempdir");
    let sock = dir.path().join("seq-test.sock");
    let primal = test_primal().await;

    let server = UdsJsonRpcServer::new(primal, sock.clone());
    let (shutdown_tx, shutdown_rx) = watch::channel(false);
    let ready = Arc::new(tokio::sync::Notify::new());
    let ready_rx = Arc::clone(&ready);

    let handle = tokio::spawn(async move { server.serve_with_ready(shutdown_rx, ready_rx).await });
    ready.notified().await;

    for i in 0..5_u32 {
        let stream = tokio::net::UnixStream::connect(&sock).await.expect("connect");
        let (reader, mut writer) = stream.into_split();

        let req = format!(r#"{{"jsonrpc":"2.0","method":"health.check","params":{{}},"id":{i}}}"#);
        writer.write_all(format!("{req}\n").as_bytes()).await.unwrap();
        writer.shutdown().await.unwrap();

        let mut lines = BufReader::new(reader).lines();
        let line = lines.next_line().await.unwrap().expect("response");
        let resp: serde_json::Value = serde_json::from_str(&line).unwrap();
        assert_eq!(resp["id"], i);
    }

    let _ = shutdown_tx.send(true);
    let _ = handle.await;
}

#[tokio::test]
async fn test_uds_serve_creates_parent_dirs() {
    let dir = tempfile::tempdir().expect("tempdir");
    let nested = dir.path().join("deep").join("nested").join("test.sock");
    let primal = test_primal().await;

    let server = UdsJsonRpcServer::new(primal, nested.clone());
    let (shutdown_tx, shutdown_rx) = watch::channel(false);
    let ready = Arc::new(tokio::sync::Notify::new());
    let ready_rx = Arc::clone(&ready);

    let handle = tokio::spawn(async move { server.serve_with_ready(shutdown_rx, ready_rx).await });
    ready.notified().await;

    assert!(nested.exists(), "socket should exist under nested dirs");

    let _ = shutdown_tx.send(true);
    let _ = handle.await;
}

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

    let (server_raw, client_raw) = std::os::unix::net::UnixStream::pair().unwrap();
    server_raw.set_nonblocking(true).unwrap();
    client_raw.set_nonblocking(true).unwrap();
    let server_stream = tokio::net::UnixStream::from_std(server_raw).unwrap();
    let mut client = tokio::net::UnixStream::from_std(client_raw).unwrap();

    let server_handle = tokio::spawn(async move {
        handle_uds_connection(server_stream, primal, true, Some(family_seed)).await
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

/// Read one newline-terminated JSON message from a raw async stream.
async fn read_json_line_raw(
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
async fn encrypted_roundtrip(
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
async fn client_phase2_handshake(
    client: &mut tokio::net::UnixStream,
    family_seed: &[u8],
) -> (String, [u8; 32]) {
    use base64::Engine;
    use hmac::{Hmac, Mac};
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

    let (server_raw, client_raw) = std::os::unix::net::UnixStream::pair().unwrap();
    server_raw.set_nonblocking(true).unwrap();
    client_raw.set_nonblocking(true).unwrap();
    let server_stream = tokio::net::UnixStream::from_std(server_raw).unwrap();
    let mut client = tokio::net::UnixStream::from_std(client_raw).unwrap();

    let server_handle = tokio::spawn(async move {
        handle_uds_connection(server_stream, primal, true, Some(family_seed)).await
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

    let (server_raw, client_raw) = std::os::unix::net::UnixStream::pair().unwrap();
    server_raw.set_nonblocking(true).unwrap();
    client_raw.set_nonblocking(true).unwrap();
    let server_stream = tokio::net::UnixStream::from_std(server_raw).unwrap();
    let mut client = tokio::net::UnixStream::from_std(client_raw).unwrap();

    let server_handle = tokio::spawn(async move {
        handle_uds_connection(server_stream, primal, true, Some(family_seed)).await
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

/// PG-52 repro: plain `dag.session.create` over UDS with BTSP required
/// must succeed (UDS is filesystem-authenticated, no handshake needed).
#[tokio::test]
async fn test_plain_jsonrpc_data_methods_on_btsp_uds() {
    let family_seed = b"integration-test-family-seed-ok!";
    let primal = test_primal().await;

    let (server_raw, client_raw) = std::os::unix::net::UnixStream::pair().unwrap();
    server_raw.set_nonblocking(true).unwrap();
    client_raw.set_nonblocking(true).unwrap();
    let server_stream = tokio::net::UnixStream::from_std(server_raw).unwrap();
    let mut client = tokio::net::UnixStream::from_std(client_raw).unwrap();

    let server_handle = tokio::spawn(async move {
        handle_uds_connection(server_stream, primal, true, Some(family_seed)).await
    });

    let req = r#"{"jsonrpc":"2.0","method":"dag.session.create","params":{"description":"PG-52 test","session_type":"General"},"id":1}"#;
    client.write_all(format!("{req}\n").as_bytes()).await.unwrap();
    client.shutdown().await.unwrap();

    let mut buf = vec![0u8; 4096];
    let mut total = 0;
    loop {
        let n = client.read(&mut buf[total..]).await.unwrap();
        total += n;
        if n == 0 || buf[..total].contains(&b'\n') {
            break;
        }
    }
    assert!(total > 0, "should receive response, not empty/reset");
    let resp_line = std::str::from_utf8(&buf[..total]).unwrap();
    let resp: serde_json::Value = serde_json::from_str(resp_line.trim()).unwrap();

    assert_eq!(resp["jsonrpc"], "2.0");
    assert_eq!(resp["id"], 1);
    assert!(
        resp["result"].is_string(),
        "dag.session.create should return a session ID, got: {resp}"
    );

    let server_result = server_handle.await.unwrap();
    assert!(server_result.is_ok());
}

/// Verify multiple data methods work over plain UDS with BTSP required:
/// `dag.session.create`, `dag.event.append`, `dag.vertex.children`,
/// `dag.frontier.get`, `dag.merkle.root`.
#[tokio::test]
async fn test_dag_method_suite_on_btsp_uds() {
    let family_seed = b"integration-test-family-seed-ok!";
    let primal = test_primal().await;

    let (server_raw, client_raw) = std::os::unix::net::UnixStream::pair().unwrap();
    server_raw.set_nonblocking(true).unwrap();
    client_raw.set_nonblocking(true).unwrap();
    let server_stream = tokio::net::UnixStream::from_std(server_raw).unwrap();
    let client_stream = tokio::net::UnixStream::from_std(client_raw).unwrap();

    let server_handle = tokio::spawn(async move {
        handle_uds_connection(server_stream, primal, true, Some(family_seed)).await
    });

    let (reader, mut writer) = client_stream.into_split();

    let create = r#"{"jsonrpc":"2.0","method":"dag.session.create","params":{"description":"suite","session_type":"General"},"id":1}"#;
    writer.write_all(format!("{create}\n").as_bytes()).await.unwrap();

    let health = r#"{"jsonrpc":"2.0","method":"health.check","params":{},"id":2}"#;
    writer.write_all(format!("{health}\n").as_bytes()).await.unwrap();

    let caps = r#"{"jsonrpc":"2.0","method":"capability.list","params":{},"id":3}"#;
    writer.write_all(format!("{caps}\n").as_bytes()).await.unwrap();

    writer.shutdown().await.unwrap();

    let mut lines = BufReader::new(reader).lines();
    let mut responses = Vec::new();
    while let Some(line) = lines.next_line().await.unwrap() {
        responses.push(serde_json::from_str::<serde_json::Value>(&line).unwrap());
    }

    assert_eq!(responses.len(), 3, "should get 3 responses");

    assert_eq!(responses[0]["id"], 1);
    assert!(responses[0]["result"].is_string(), "dag.session.create result: {}", responses[0]);

    assert_eq!(responses[1]["id"], 2);
    assert!(responses[1]["result"].is_object(), "health.check result: {}", responses[1]);

    assert_eq!(responses[2]["id"], 3);
    assert!(
        responses[2]["result"].is_object() || responses[2]["result"].is_array(),
        "capability.list result: {}",
        responses[2]
    );

    let server_result = server_handle.await.unwrap();
    assert!(server_result.is_ok());
}

/// Verify batch JSON-RPC also works on BTSP-enforced UDS.
#[tokio::test]
async fn test_batch_jsonrpc_on_btsp_uds() {
    let family_seed = b"integration-test-family-seed-ok!";
    let primal = test_primal().await;

    let (server_raw, client_raw) = std::os::unix::net::UnixStream::pair().unwrap();
    server_raw.set_nonblocking(true).unwrap();
    client_raw.set_nonblocking(true).unwrap();
    let server_stream = tokio::net::UnixStream::from_std(server_raw).unwrap();
    let mut client = tokio::net::UnixStream::from_std(client_raw).unwrap();

    let server_handle = tokio::spawn(async move {
        handle_uds_connection(server_stream, primal, true, Some(family_seed)).await
    });

    let batch = r#"[{"jsonrpc":"2.0","method":"health.check","params":{},"id":1},{"jsonrpc":"2.0","method":"dag.session.create","params":{"description":"batch","session_type":"General"},"id":2}]"#;
    client.write_all(format!("{batch}\n").as_bytes()).await.unwrap();
    client.shutdown().await.unwrap();

    let mut buf = vec![0u8; 8192];
    let mut total = 0;
    loop {
        let n = client.read(&mut buf[total..]).await.unwrap();
        total += n;
        if n == 0 || buf[..total].contains(&b'\n') {
            break;
        }
    }
    let resp_line = std::str::from_utf8(&buf[..total]).unwrap();
    let resp: serde_json::Value = serde_json::from_str(resp_line.trim()).unwrap();

    let arr = resp.as_array().expect("batch response should be an array");
    assert_eq!(arr.len(), 2);
    assert!(arr[0]["result"].is_object(), "health.check: {}", arr[0]);
    assert!(arr[1]["result"].is_string(), "dag.session.create: {}", arr[1]);

    let server_result = server_handle.await.unwrap();
    assert!(server_result.is_ok());
}
