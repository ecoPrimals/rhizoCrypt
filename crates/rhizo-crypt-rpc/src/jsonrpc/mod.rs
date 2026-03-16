// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2024–2026 ecoPrimals Project

//! JSON-RPC 2.0 server for `rhizoCrypt`.
//!
//! Implements the ecoPrimals `UNIVERSAL_IPC_STANDARD` JSON-RPC 2.0 protocol
//! with semantic method naming: `{domain}.{operation}[.{variant}]`.

mod handler;
mod types;

use axum::{Json, Router, extract::State, http::StatusCode, response::IntoResponse, routing::post};
use rhizo_crypt_core::{RhizoCrypt, constants::JSON_RPC_PATH};
use std::net::SocketAddr;
use std::sync::Arc;
use tower_http::cors::{Any, CorsLayer};
use tower_http::trace::TraceLayer;
use tracing::{info, warn};
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

    /// Start the JSON-RPC server.
    ///
    /// Binds to the configured address and serves POST requests at the RPC path.
    ///
    /// # Errors
    ///
    /// Returns `std::io::Error` if binding or serving fails.
    pub async fn serve(self) -> Result<(), std::io::Error> {
        let listener = tokio::net::TcpListener::bind(self.addr).await?;
        let local_addr = listener.local_addr()?;
        info!(address = %local_addr, "JSON-RPC server listening");

        let app = Self::router(self.primal);
        axum::serve(listener, app).await.map_err(std::io::Error::other)
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
            let (code, message) = match e {
                handler::HandlerError::InvalidParams(msg) => (codes::INVALID_PARAMS, msg),
                handler::HandlerError::MethodNotFound(m) => {
                    (codes::METHOD_NOT_FOUND, format!("Method not found: {m}"))
                }
                handler::HandlerError::Rpc(rpc_err) => (codes::INTERNAL_ERROR, rpc_err.to_string()),
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
}
