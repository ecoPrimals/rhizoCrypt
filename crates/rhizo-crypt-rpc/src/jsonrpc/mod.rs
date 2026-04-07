// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2024–2026 ecoPrimals Project

//! JSON-RPC 2.0 server for `rhizoCrypt`.
//!
//! Implements the ecoPrimals `UNIVERSAL_IPC_STANDARD` JSON-RPC 2.0 protocol
//! with semantic method naming: `{domain}.{operation}[.{variant}]`.

mod handler;
pub mod newline;
mod types;

#[cfg(unix)]
pub mod uds;

use axum::{Json, Router, extract::State, http::StatusCode, response::IntoResponse, routing::post};
use rhizo_crypt_core::{RhizoCrypt, constants::JSON_RPC_PATH};
use std::borrow::Cow;
use std::net::SocketAddr;
use std::sync::Arc;
use tower::Service;
use tower_http::cors::{Any, CorsLayer};
use tower_http::trace::TraceLayer;
use tracing::{debug, info, warn};
use types::{JsonRpcRequest, codes, error_response, success};

/// Serialize a JSON-RPC response value, logging any serialization failure.
fn serialize_response(value: &impl serde::Serialize) -> serde_json::Value {
    serde_json::to_value(value).unwrap_or_else(|e| {
        warn!(error = %e, "Failed to serialize JSON-RPC response, returning null");
        serde_json::Value::Null
    })
}

/// Shared state for the JSON-RPC handler.
#[derive(Clone)]
struct JsonRpcState {
    primal: Arc<RhizoCrypt>,
}

/// JSON-RPC 2.0 server.
///
/// Serves JSON-RPC requests at the configured path, sharing the same
/// `RhizoCrypt` primal instance as the tarpc server.
pub struct JsonRpcServer {
    primal: Arc<RhizoCrypt>,
    addr: SocketAddr,
}

impl JsonRpcServer {
    /// Create a new JSON-RPC server.
    #[must_use]
    pub const fn new(primal: Arc<RhizoCrypt>, addr: SocketAddr) -> Self {
        Self {
            primal,
            addr,
        }
    }

    /// Start the dual-mode JSON-RPC server.
    ///
    /// Accepts TCP connections and auto-detects the wire framing by peeking
    /// at the first byte:
    /// - `{` or `[` → raw newline-delimited JSON-RPC (ecosystem IPC standard)
    /// - Anything else → HTTP (Axum router, for `curl`/browser clients)
    ///
    /// This follows the ecosystem dual-mode protocol-detection pattern and
    /// resolves the IPC compliance matrix wire framing requirement.
    ///
    /// # Errors
    ///
    /// Returns `std::io::Error` if binding fails.
    pub async fn serve(self) -> Result<(), std::io::Error> {
        self.serve_inner(None).await
    }

    /// Start the server and signal `ready` once the listener is bound.
    ///
    /// Identical to [`serve`](Self::serve) but notifies the provided
    /// [`tokio::sync::Notify`] after the TCP listener is ready to accept
    /// connections. Use this in tests to avoid sleep-based synchronization.
    ///
    /// # Errors
    ///
    /// Returns `std::io::Error` if binding fails.
    pub async fn serve_with_ready(
        self,
        ready: Arc<tokio::sync::Notify>,
    ) -> Result<(), std::io::Error> {
        self.serve_inner(Some(ready)).await
    }

    async fn serve_inner(
        self,
        ready: Option<Arc<tokio::sync::Notify>>,
    ) -> Result<(), std::io::Error> {
        let listener = tokio::net::TcpListener::bind(self.addr).await?;
        let local_addr = listener.local_addr()?;
        info!(address = %local_addr, "JSON-RPC server listening (dual-mode: HTTP + newline)");

        if let Some(notify) = ready {
            notify.notify_one();
        }

        let app = Self::router(Arc::clone(&self.primal));

        loop {
            let (stream, peer) = listener.accept().await?;
            let primal = Arc::clone(&self.primal);
            let app = app.clone();

            tokio::spawn(async move {
                if let Err(e) = handle_tcp_connection(stream, peer, primal, app).await {
                    debug!(peer = %peer, error = %e, "TCP connection ended");
                }
            });
        }
    }

    /// Build the axum router for embedding in larger applications.
    ///
    /// Serves JSON-RPC at the path from [`rhizo_crypt_core::constants::JSON_RPC_PATH`].
    pub fn router(primal: Arc<RhizoCrypt>) -> Router {
        let state = JsonRpcState {
            primal,
        };
        let cors = CorsLayer::new()
            .allow_origin(Any)
            .allow_methods([axum::http::Method::POST, axum::http::Method::OPTIONS])
            .allow_headers([axum::http::header::CONTENT_TYPE]);

        Router::new()
            .route(JSON_RPC_PATH, post(handle_jsonrpc))
            .layer(TraceLayer::new_for_http())
            .layer(cors)
            .with_state(state)
    }
}

/// Route a TCP connection based on first-byte protocol detection.
///
/// Peeks the first byte without consuming it:
/// - `{` or `[` → newline-delimited JSON-RPC (raw IPC)
/// - Anything else → HTTP (serve through Axum)
async fn handle_tcp_connection(
    stream: tokio::net::TcpStream,
    peer: SocketAddr,
    primal: Arc<RhizoCrypt>,
    app: Router,
) -> std::io::Result<()> {
    let mut peek = [0u8; 1];
    let n = stream.peek(&mut peek).await?;

    if n > 0 && (peek[0] == b'{' || peek[0] == b'[') {
        debug!(peer = %peer, "detected newline JSON-RPC");
        newline::handle_newline_connection(stream, primal).await?;
    } else {
        debug!(peer = %peer, "detected HTTP");
        let io = hyper_util::rt::TokioIo::new(stream);
        let service =
            hyper::service::service_fn(move |req: hyper::Request<hyper::body::Incoming>| {
                let app = app.clone();
                async move { app.into_service().call(req).await }
            });
        hyper_util::server::conn::auto::Builder::new(hyper_util::rt::TokioExecutor::new())
            .serve_connection(io, service)
            .await
            .map_err(|e| std::io::Error::other(format!("hyper error: {e}")))?;
    }

    Ok(())
}

/// Handle JSON-RPC request body (single or batch).
async fn handle_jsonrpc(
    State(state): State<JsonRpcState>,
    body: axum::body::Bytes,
) -> impl IntoResponse {
    let parsed: Result<serde_json::Value, _> = serde_json::from_slice(&body);
    let value = match parsed {
        Ok(v) => v,
        Err(e) => {
            warn!(error = %e, "JSON-RPC parse error");
            return (
                StatusCode::BAD_REQUEST,
                Json(serialize_response(&error_response(
                    None,
                    codes::PARSE_ERROR,
                    "Parse error",
                    Some(serde_json::json!(e.to_string())),
                ))),
            )
                .into_response();
        }
    };

    match value {
        serde_json::Value::Array(arr) => {
            if arr.is_empty() {
                return (
                    StatusCode::BAD_REQUEST,
                    Json(serialize_response(&error_response(
                        None,
                        codes::INVALID_REQUEST,
                        "Batch request must not be empty",
                        None,
                    ))),
                )
                    .into_response();
            }
            let mut results = Vec::with_capacity(arr.len());
            for item in arr {
                let response = process_single_request(Arc::clone(&state.primal), item).await;
                results.push(response);
            }
            (StatusCode::OK, Json(serde_json::json!(results))).into_response()
        }
        value => {
            let response = process_single_request(state.primal, value).await;
            (StatusCode::OK, Json(response)).into_response()
        }
    }
}

/// Process a single JSON-RPC request.
async fn process_single_request(
    primal: Arc<RhizoCrypt>,
    value: serde_json::Value,
) -> serde_json::Value {
    let request: JsonRpcRequest = match serde_json::from_value(value) {
        Ok(r) => r,
        Err(e) => {
            return serialize_response(&error_response(
                None,
                codes::INVALID_REQUEST,
                "Invalid Request",
                Some(serde_json::json!(e.to_string())),
            ));
        }
    };

    if request.jsonrpc != "2.0" {
        return serialize_response(&error_response(
            request.id,
            codes::INVALID_REQUEST,
            "jsonrpc must be \"2.0\"",
            None,
        ));
    }

    let id = match &request.id {
        Some(i) => i.clone(),
        None => {
            return serialize_response(&error_response(
                None,
                codes::INVALID_REQUEST,
                "id is required (notifications not supported)",
                None,
            ));
        }
    };

    match handler::handle_request(primal, request).await {
        Ok(result) => serialize_response(&success(id, result)),
        Err(e) => {
            let detail = serde_json::json!(e.to_string());
            let (code, message): (i32, Cow<'_, str>) = match e {
                handler::HandlerError::InvalidParams(msg) => (codes::INVALID_PARAMS, msg),
                handler::HandlerError::MethodNotFound(m) => {
                    (codes::METHOD_NOT_FOUND, format!("Method not found: {m}").into())
                }
                handler::HandlerError::Rpc(rpc_err) => {
                    (codes::INTERNAL_ERROR, rpc_err.to_string().into())
                }
            };
            serialize_response(&error_response(Some(id), code, &message, Some(detail)))
        }
    }
}

#[cfg(test)]
#[expect(clippy::unwrap_used, clippy::expect_used, reason = "test code")]
mod tests {
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
}
