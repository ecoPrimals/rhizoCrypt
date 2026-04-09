// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2024–2026 ecoPrimals Project

use super::*;
use axum::body::{Body, to_bytes};
use axum::http::Request;
use rhizo_crypt_core::{PrimalLifecycle, RhizoCryptConfig};
use tower::util::ServiceExt;

async fn create_test_primal() -> Arc<RhizoCrypt> {
    let config = RhizoCryptConfig::default();
    let mut primal = RhizoCrypt::new(config);
    primal.start().await.expect("start");
    Arc::new(primal)
}

#[test]
fn test_create_router() {
    let primal = Arc::new(RhizoCrypt::new(RhizoCryptConfig::default()));
    let router = JsonRpcServer::router(primal);
    assert_eq!(rhizo_crypt_core::constants::JSON_RPC_PATH, "/rpc");
    // Router is created successfully (no panic)
    drop(router);
}

#[tokio::test]
async fn test_jsonrpc_endpoint_health() {
    let primal = create_test_primal().await;
    let app = JsonRpcServer::router(primal);

    let request_body = serde_json::json!({
        "jsonrpc": "2.0",
        "method": "health.check",
        "params": {},
        "id": 1
    });

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/rpc")
                .header("content-type", "application/json")
                .body(Body::from(serde_json::to_string(&request_body).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), 200);
    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert!(json.get("result").is_some());
    let result = json.get("result").unwrap().as_object().unwrap();
    assert!(
        result.get("healthy").and_then(serde_json::Value::as_bool).unwrap_or(false),
        "health should report healthy"
    );
}

#[tokio::test]
async fn test_jsonrpc_endpoint_session_create() {
    let primal = create_test_primal().await;
    let app = JsonRpcServer::router(primal);

    let request_body = serde_json::json!({
        "jsonrpc": "2.0",
        "method": "dag.session.create",
        "params": {"session_type": "General", "description": "test session"},
        "id": 1
    });

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/rpc")
                .header("content-type", "application/json")
                .body(Body::from(serde_json::to_string(&request_body).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), 200);
    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert!(json.get("result").is_some());
    let session_id = json.get("result").unwrap().as_str().unwrap();
    assert!(uuid::Uuid::parse_str(session_id).is_ok());
}

#[tokio::test]
async fn test_jsonrpc_endpoint_method_not_found() {
    let primal = create_test_primal().await;
    let app = JsonRpcServer::router(primal);

    let request_body = serde_json::json!({
        "jsonrpc": "2.0",
        "method": "unknown.method.that.does.not.exist",
        "params": {},
        "id": 1
    });

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/rpc")
                .header("content-type", "application/json")
                .body(Body::from(serde_json::to_string(&request_body).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), 200);
    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert!(json.get("error").is_some());
    let err = json.get("error").unwrap().as_object().unwrap();
    assert_eq!(err.get("code").and_then(serde_json::Value::as_i64), Some(-32601));
}

#[tokio::test]
async fn test_jsonrpc_error_invalid_json() {
    let primal = create_test_primal().await;
    let app = JsonRpcServer::router(primal);

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/rpc")
                .header("content-type", "application/json")
                .body(Body::from("not valid json {{{"))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), 400);
    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert!(json.get("error").is_some());
    let err = json.get("error").unwrap().as_object().unwrap();
    assert_eq!(err.get("code").and_then(serde_json::Value::as_i64), Some(-32700));
}

#[test]
fn test_jsonrpc_server_creation() {
    let config = RhizoCryptConfig::default();
    let primal = Arc::new(RhizoCrypt::new(config.clone()));
    let addr: SocketAddr = "127.0.0.1:0".parse().expect("parse");
    let _server = JsonRpcServer::new(primal, addr);
    let _router = JsonRpcServer::router(Arc::new(RhizoCrypt::new(config)));
    assert_eq!(rhizo_crypt_core::constants::JSON_RPC_PATH, "/rpc");
}

#[tokio::test]
async fn test_jsonrpc_invalid_utf8() {
    let primal = create_test_primal().await;
    let app = JsonRpcServer::router(primal);

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/rpc")
                .header("content-type", "application/json")
                .body(Body::from(vec![0xFF, 0xFE, 0xFD]))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), 400);
    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    let err = json.get("error").unwrap().as_object().unwrap();
    assert_eq!(err.get("code").and_then(serde_json::Value::as_i64), Some(-32700));
}

#[tokio::test]
async fn test_jsonrpc_empty_batch() {
    let primal = create_test_primal().await;
    let app = JsonRpcServer::router(primal);

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/rpc")
                .header("content-type", "application/json")
                .body(Body::from("[]"))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), 400);
    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    let err = json.get("error").unwrap().as_object().unwrap();
    assert_eq!(err.get("code").and_then(serde_json::Value::as_i64), Some(-32600));
}

#[tokio::test]
async fn test_jsonrpc_batch_request() {
    let primal = create_test_primal().await;
    let app = JsonRpcServer::router(primal);

    let batch = serde_json::json!([
        {"jsonrpc": "2.0", "method": "health.check", "params": {}, "id": 1},
        {"jsonrpc": "2.0", "method": "health.metrics", "params": {}, "id": 2}
    ]);

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/rpc")
                .header("content-type", "application/json")
                .body(Body::from(serde_json::to_string(&batch).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), 200);
    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    let arr = json.as_array().unwrap();
    assert_eq!(arr.len(), 2);
    assert!(arr[0].get("result").is_some());
    assert!(arr[1].get("result").is_some());
}

#[tokio::test]
async fn test_jsonrpc_wrong_version() {
    let primal = create_test_primal().await;
    let app = JsonRpcServer::router(primal);

    let request_body = serde_json::json!({
        "jsonrpc": "1.0",
        "method": "health.check",
        "params": {},
        "id": 1
    });

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/rpc")
                .header("content-type", "application/json")
                .body(Body::from(serde_json::to_string(&request_body).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), 200);
    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    let err = json.get("error").unwrap().as_object().unwrap();
    assert_eq!(err.get("code").and_then(serde_json::Value::as_i64), Some(-32600));
}

#[tokio::test]
async fn test_jsonrpc_missing_id() {
    let primal = create_test_primal().await;
    let app = JsonRpcServer::router(primal);

    let request_body = serde_json::json!({
        "jsonrpc": "2.0",
        "method": "health.check",
        "params": {}
    });

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/rpc")
                .header("content-type", "application/json")
                .body(Body::from(serde_json::to_string(&request_body).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), 200);
    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    let err = json.get("error").unwrap().as_object().unwrap();
    assert_eq!(err.get("code").and_then(serde_json::Value::as_i64), Some(-32600));
}

#[tokio::test]
async fn test_jsonrpc_invalid_params() {
    let primal = create_test_primal().await;
    let app = JsonRpcServer::router(primal);

    let request_body = serde_json::json!({
        "jsonrpc": "2.0",
        "method": "dag.session.get",
        "params": {"session_id": "not-a-uuid"},
        "id": 1
    });

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/rpc")
                .header("content-type", "application/json")
                .body(Body::from(serde_json::to_string(&request_body).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), 200);
    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    let err = json.get("error").unwrap().as_object().unwrap();
    assert_eq!(err.get("code").and_then(serde_json::Value::as_i64), Some(-32602));
}

#[tokio::test]
async fn test_jsonrpc_not_an_object_request() {
    let primal = create_test_primal().await;
    let app = JsonRpcServer::router(primal);

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/rpc")
                .header("content-type", "application/json")
                .body(Body::from("42"))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), 200);
    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert!(json.get("error").is_some());
}

#[tokio::test]
async fn test_dual_mode_raw_newline_client() {
    use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};

    let primal = create_test_primal().await;
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    drop(listener);

    let server = JsonRpcServer::new(primal, addr);
    let ready = Arc::new(tokio::sync::Notify::new());
    let ready_rx = Arc::clone(&ready);
    tokio::spawn(async move { server.serve_with_ready(ready_rx).await });
    ready.notified().await;

    let mut stream = tokio::net::TcpStream::connect(addr).await.unwrap();
    let req = b"{\"jsonrpc\":\"2.0\",\"method\":\"health.check\",\"params\":{},\"id\":1}\n";
    AsyncWriteExt::write_all(&mut stream, req).await.unwrap();
    AsyncWriteExt::shutdown(&mut stream).await.unwrap();

    let mut lines = BufReader::new(stream).lines();
    let line = lines.next_line().await.unwrap().expect("response");
    let resp: serde_json::Value = serde_json::from_str(&line).unwrap();
    assert_eq!(resp["jsonrpc"], "2.0");
    assert!(resp["result"].is_object(), "expected result, got: {resp}");
}

#[tokio::test]
async fn test_dual_mode_http_client() {
    use tokio::io::{AsyncReadExt, AsyncWriteExt};

    let primal = create_test_primal().await;
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    drop(listener);

    let server = JsonRpcServer::new(primal, addr);
    let ready = Arc::new(tokio::sync::Notify::new());
    let ready_rx = Arc::clone(&ready);
    tokio::spawn(async move { server.serve_with_ready(ready_rx).await });
    ready.notified().await;

    let mut stream = tokio::net::TcpStream::connect(addr).await.unwrap();

    let body = r#"{"jsonrpc":"2.0","method":"health.check","params":{},"id":1}"#;
    let http_req = format!(
        "POST /rpc HTTP/1.1\r\nHost: {addr}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
        body.len(),
    );
    AsyncWriteExt::write_all(&mut stream, http_req.as_bytes()).await.unwrap();

    let mut buf = Vec::new();
    AsyncReadExt::read_to_end(&mut stream, &mut buf).await.unwrap();
    let response = String::from_utf8_lossy(&buf);

    assert!(
        response.starts_with("HTTP/1.1 200"),
        "expected 200, got: {}",
        response.lines().next().unwrap_or("")
    );

    let body_start = response.find("\r\n\r\n").expect("HTTP body separator") + 4;
    let body_str = &response[body_start..];
    let json: serde_json::Value = serde_json::from_str(body_str.trim()).unwrap();
    assert_eq!(json["jsonrpc"], "2.0");
    assert!(json["result"].is_object(), "expected result, got: {json}");
}
