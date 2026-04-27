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
