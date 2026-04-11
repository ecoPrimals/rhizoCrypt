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
use rhizo_crypt_core::RhizoCrypt;
use rhizo_crypt_core::constants::{DEFAULT_MAX_CONNECTIONS, JSON_RPC_PATH};
use std::borrow::Cow;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::{Semaphore, watch};
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

    /// Start the dual-mode JSON-RPC server with graceful shutdown.
    ///
    /// Accepts TCP connections and auto-detects the wire framing by peeking
    /// at the first byte:
    /// - `{` or `[` → raw newline-delimited JSON-RPC (ecosystem IPC standard)
    /// - Anything else → HTTP (Axum router, for `curl`/browser clients)
    ///
    /// The server stops accepting new connections when `shutdown` fires and
    /// enforces a connection limit of [`DEFAULT_MAX_CONNECTIONS`].
    ///
    /// # Errors
    ///
    /// Returns `std::io::Error` if binding fails.
    pub async fn serve(self, shutdown: watch::Receiver<bool>) -> Result<(), std::io::Error> {
        self.serve_inner(shutdown, None).await
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
        shutdown: watch::Receiver<bool>,
        ready: Arc<tokio::sync::Notify>,
    ) -> Result<(), std::io::Error> {
        self.serve_inner(shutdown, Some(ready)).await
    }

    async fn serve_inner(
        self,
        mut shutdown: watch::Receiver<bool>,
        ready: Option<Arc<tokio::sync::Notify>>,
    ) -> Result<(), std::io::Error> {
        let listener = tokio::net::TcpListener::bind(self.addr).await?;
        let local_addr = listener.local_addr()?;
        info!(address = %local_addr, "JSON-RPC server listening (dual-mode: HTTP + newline)");

        if let Some(notify) = ready {
            notify.notify_one();
        }

        let app = Self::router(Arc::clone(&self.primal));
        let semaphore = Arc::new(Semaphore::new(DEFAULT_MAX_CONNECTIONS));

        loop {
            tokio::select! {
                result = listener.accept() => {
                    let (stream, peer) = result?;
                    let Ok(permit) = Arc::clone(&semaphore).try_acquire_owned() else {
                        warn!(peer = %peer, "connection rejected: limit reached");
                        drop(stream);
                        continue;
                    };
                    let primal = Arc::clone(&self.primal);
                    let app = app.clone();

                    tokio::spawn(async move {
                        if let Err(e) = handle_tcp_connection(stream, peer, primal, app).await {
                            debug!(peer = %peer, error = %e, "TCP connection ended");
                        }
                        drop(permit);
                    });
                }
                _ = shutdown.changed() => {
                    info!("JSON-RPC TCP listener shutting down");
                    break;
                }
            }
        }
        Ok(())
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
#[path = "mod_tests.rs"]
mod tests;
