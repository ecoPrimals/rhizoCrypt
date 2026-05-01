// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2024–2026 ecoPrimals Project

//! Newline-delimited JSON-RPC 2.0 connection handler.
//!
//! Provides a generic handler that reads newline-terminated JSON-RPC requests
//! from any `AsyncRead + AsyncWrite` stream and writes newline-terminated
//! responses. Shared between UDS and raw-newline TCP transports per the
//! ecoPrimals `PRIMAL_IPC_PROTOCOL` v3.1 wire framing standard.
//!
//! ## Wire Format
//!
//! ```text
//! → {"jsonrpc":"2.0","method":"health.liveness","id":1}\n
//! ← {"jsonrpc":"2.0","result":{...},"id":1}\n
//! ```

use rhizo_crypt_core::RhizoCrypt;
use rhizo_crypt_core::constants::MAX_JSONRPC_LINE_LENGTH;
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tracing::{debug, warn};

use super::types::{JsonRpcId, codes, error_response};
use super::{process_single_request, serialize_response};

/// Methods allowed without BTSP authentication (liveness probes only).
///
/// When a plain JSON-RPC connection arrives on a BTSP-enforced UDS socket
/// (first byte `{`/`[` instead of a length-prefix), these methods are served
/// without authentication. All other methods return a "BTSP authentication
/// required" error. This follows the ecosystem first-byte auto-detect pattern
/// (PG-35, PG-30).
const UNAUTHENTICATED_METHODS: &[&str] = &[
    "health.check",
    "health.liveness",
    "health.readiness",
    "health",
    "ping",
    "status",
    "check",
    "identity.get",
    "capabilities.list",
    "capability.list",
    "primal.capabilities",
    "lifecycle.status",
];

/// Handle a newline-delimited JSON-RPC connection over any async stream.
///
/// Reads lines from `stream`, parses each as a JSON-RPC request (single
/// or batch), dispatches through the same handler as the HTTP path, and
/// writes the response followed by `\n`. Empty lines are silently skipped.
///
/// Individual lines are capped at [`MAX_JSONRPC_LINE_LENGTH`] bytes to
/// prevent unbounded memory allocation from misbehaving clients.
///
/// The connection stays open until the peer closes or an I/O error occurs.
///
/// # Errors
///
/// Returns `std::io::Error` on stream read/write failures.
pub async fn handle_newline_connection<S>(stream: S, primal: Arc<RhizoCrypt>) -> std::io::Result<()>
where
    S: tokio::io::AsyncRead + tokio::io::AsyncWrite + Unpin,
{
    let (reader, mut writer) = tokio::io::split(stream);
    let buf_reader = BufReader::new(reader);
    let mut lines = buf_reader.lines();

    while let Some(line) = lines.next_line().await? {
        if line.len() > MAX_JSONRPC_LINE_LENGTH {
            warn!(
                length = line.len(),
                limit = MAX_JSONRPC_LINE_LENGTH,
                "JSON-RPC line exceeds maximum length, dropping"
            );
            let resp = serialize_response(&error_response(
                None,
                codes::INVALID_REQUEST,
                "Request too large",
                None,
            ));
            write_response(&mut writer, &resp).await?;
            continue;
        }
        if line.trim().is_empty() {
            continue;
        }

        let value: serde_json::Value = match serde_json::from_str(&line) {
            Ok(v) => v,
            Err(e) => {
                warn!(error = %e, "newline JSON-RPC parse error");
                let resp = serialize_response(&error_response(
                    None,
                    codes::PARSE_ERROR,
                    "Parse error",
                    Some(serde_json::json!(e.to_string())),
                ));
                write_response(&mut writer, &resp).await?;
                continue;
            }
        };

        match value {
            serde_json::Value::Array(arr) => {
                if arr.is_empty() {
                    let resp = serialize_response(&error_response(
                        None,
                        codes::INVALID_REQUEST,
                        "Batch request must not be empty",
                        None,
                    ));
                    write_response(&mut writer, &resp).await?;
                    continue;
                }
                let mut results = Vec::with_capacity(arr.len());
                for item in arr {
                    results.push(process_single_request(Arc::clone(&primal), item).await);
                }
                let batch = serde_json::json!(results);
                write_response(&mut writer, &batch).await?;
            }
            value => {
                let response = process_single_request(Arc::clone(&primal), value).await;
                write_response(&mut writer, &response).await?;
            }
        }
    }

    Ok(())
}

/// Handle a liveness-only JSON-RPC connection (no BTSP authentication).
///
/// Identical to [`handle_newline_connection`] but rejects any method not in
/// the unauthenticated allowlist with a JSON-RPC error. Available for
/// transports that need restricted method access (e.g. TCP probes). Not
/// currently used on UDS — since S49, all UDS paths route to the full
/// handler via filesystem-authenticated trust.
///
/// # Errors
///
/// Returns `std::io::Error` if reading from or writing to the stream fails.
pub async fn handle_liveness_connection<S>(
    stream: S,
    primal: Arc<RhizoCrypt>,
) -> std::io::Result<()>
where
    S: tokio::io::AsyncRead + tokio::io::AsyncWrite + Unpin,
{
    let (reader, mut writer) = tokio::io::split(stream);
    let buf_reader = BufReader::new(reader);
    let mut lines = buf_reader.lines();

    while let Some(line) = lines.next_line().await? {
        if line.len() > MAX_JSONRPC_LINE_LENGTH {
            let resp = serialize_response(&error_response(
                None,
                codes::INVALID_REQUEST,
                "Request too large",
                None,
            ));
            write_response(&mut writer, &resp).await?;
            continue;
        }
        if line.trim().is_empty() {
            continue;
        }

        let value: serde_json::Value = match serde_json::from_str(&line) {
            Ok(v) => v,
            Err(e) => {
                warn!(error = %e, "liveness JSON-RPC parse error");
                let resp = serialize_response(&error_response(
                    None,
                    codes::PARSE_ERROR,
                    "Parse error",
                    Some(serde_json::json!(e.to_string())),
                ));
                write_response(&mut writer, &resp).await?;
                continue;
            }
        };

        let response = match &value {
            serde_json::Value::Object(map) => {
                let method = map.get("method").and_then(serde_json::Value::as_str);
                let id =
                    map.get("id").and_then(|v| serde_json::from_value::<JsonRpcId>(v.clone()).ok());
                if let Some(m) = method {
                    if UNAUTHENTICATED_METHODS.contains(&m) {
                        process_single_request(Arc::clone(&primal), value).await
                    } else {
                        debug!(method = m, "rejected unauthenticated method (BTSP required)");
                        serialize_response(&error_response(
                            id,
                            codes::FORBIDDEN,
                            "BTSP authentication required for this method",
                            Some(serde_json::json!({
                                "method": m,
                                "hint": "Use BTSP handshake or call health.check / capability.list for unauthenticated probes"
                            })),
                        ))
                    }
                } else {
                    serialize_response(&error_response(
                        id,
                        codes::INVALID_REQUEST,
                        "Missing method field",
                        None,
                    ))
                }
            }
            _ => serialize_response(&error_response(
                None,
                codes::INVALID_REQUEST,
                "Batch requests require BTSP authentication",
                None,
            )),
        };

        write_response(&mut writer, &response).await?;
    }

    Ok(())
}

async fn write_response<W: tokio::io::AsyncWrite + Unpin>(
    writer: &mut W,
    value: &serde_json::Value,
) -> std::io::Result<()> {
    let mut resp_str =
        serde_json::to_string(value).unwrap_or_else(|_| r#"{"jsonrpc":"2.0","error":{"code":-32603,"message":"serialization failed"},"id":null}"#.to_string());
    resp_str.push('\n');
    writer.write_all(resp_str.as_bytes()).await?;
    writer.flush().await
}

#[cfg(test)]
#[expect(clippy::unwrap_used, reason = "test code")]
mod tests {
    use super::*;
    use rhizo_crypt_core::{PrimalLifecycle, RhizoCryptConfig};
    use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader, duplex};

    async fn test_primal() -> Arc<RhizoCrypt> {
        let mut p = RhizoCrypt::new(RhizoCryptConfig::default());
        p.start().await.unwrap();
        Arc::new(p)
    }

    /// Write lines to a duplex client, close the write half, then read
    /// all response lines from the read half.
    async fn roundtrip(primal: Arc<RhizoCrypt>, input: &[u8]) -> Vec<serde_json::Value> {
        let (mut client, server) = duplex(8192);
        let handle = tokio::spawn(handle_newline_connection(server, primal));

        AsyncWriteExt::write_all(&mut client, input).await.unwrap();
        AsyncWriteExt::shutdown(&mut client).await.unwrap();

        let mut lines = BufReader::new(client).lines();
        let mut results = Vec::new();
        while let Some(line) = lines.next_line().await.unwrap() {
            results.push(serde_json::from_str(&line).unwrap());
        }

        handle.await.unwrap().unwrap();
        results
    }

    #[tokio::test]
    async fn test_newline_health_check() {
        let primal = test_primal().await;
        let input = b"{\"jsonrpc\":\"2.0\",\"method\":\"health.check\",\"params\":{},\"id\":1}\n";
        let results = roundtrip(primal, input).await;
        assert_eq!(results.len(), 1);
        assert_eq!(results[0]["jsonrpc"], "2.0");
        assert!(results[0]["result"].is_object());
    }

    #[tokio::test]
    async fn test_newline_parse_error_continues() {
        let primal = test_primal().await;
        let input = b"{ invalid json }\n{\"jsonrpc\":\"2.0\",\"method\":\"health.check\",\"params\":{},\"id\":2}\n";
        let results = roundtrip(primal, input).await;
        assert_eq!(results.len(), 2);
        assert_eq!(results[0]["error"]["code"], -32700);
        assert!(results[1]["result"].is_object());
    }

    #[tokio::test]
    async fn test_newline_empty_lines_skipped() {
        let primal = test_primal().await;
        let input =
            b"\n\n{\"jsonrpc\":\"2.0\",\"method\":\"health.check\",\"params\":{},\"id\":3}\n";
        let results = roundtrip(primal, input).await;
        assert_eq!(results.len(), 1);
        assert_eq!(results[0]["id"], 3);
    }

    #[tokio::test]
    async fn test_newline_batch_request() {
        let primal = test_primal().await;
        let input = b"[{\"jsonrpc\":\"2.0\",\"method\":\"health.check\",\"params\":{},\"id\":1},{\"jsonrpc\":\"2.0\",\"method\":\"health.metrics\",\"params\":{},\"id\":2}]\n";
        let results = roundtrip(primal, input).await;
        assert_eq!(results.len(), 1);
        let arr = results[0].as_array().unwrap();
        assert_eq!(arr.len(), 2);
    }

    #[tokio::test]
    async fn test_newline_empty_batch_error() {
        let primal = test_primal().await;
        let input = b"[]\n";
        let results = roundtrip(primal, input).await;
        assert_eq!(results.len(), 1);
        assert_eq!(results[0]["error"]["code"], -32600);
    }

    /// EOF without trailing newline still delivers the response when the
    /// caller shuts down their write half. Validates the behavior reported
    /// by primalSpring's trio integration guide (PG-52 validation).
    #[tokio::test]
    async fn test_newline_eof_without_trailing_newline() {
        let primal = test_primal().await;
        let input = b"{\"jsonrpc\":\"2.0\",\"method\":\"health.check\",\"params\":{},\"id\":99}";
        let results = roundtrip(primal, input).await;
        assert_eq!(results.len(), 1, "EOF after data should deliver the response");
        assert_eq!(results[0]["id"], 99);
        assert!(results[0]["result"].is_object());
    }

    async fn liveness_roundtrip(primal: Arc<RhizoCrypt>, input: &[u8]) -> Vec<serde_json::Value> {
        let (mut client, server) = duplex(8192);
        let handle = tokio::spawn(handle_liveness_connection(server, primal));

        AsyncWriteExt::write_all(&mut client, input).await.unwrap();
        AsyncWriteExt::shutdown(&mut client).await.unwrap();

        let mut lines = BufReader::new(client).lines();
        let mut results = Vec::new();
        while let Some(line) = lines.next_line().await.unwrap() {
            results.push(serde_json::from_str(&line).unwrap());
        }

        handle.await.unwrap().unwrap();
        results
    }

    #[tokio::test]
    async fn test_liveness_allows_health_check() {
        let primal = test_primal().await;
        let input = b"{\"jsonrpc\":\"2.0\",\"method\":\"health.check\",\"params\":{},\"id\":1}\n";
        let results = liveness_roundtrip(primal, input).await;
        assert_eq!(results.len(), 1);
        assert!(results[0]["result"].is_object(), "health.check should succeed");
    }

    #[tokio::test]
    async fn test_liveness_allows_capability_list() {
        let primal = test_primal().await;
        let input =
            b"{\"jsonrpc\":\"2.0\",\"method\":\"capability.list\",\"params\":{},\"id\":2}\n";
        let results = liveness_roundtrip(primal, input).await;
        assert_eq!(results.len(), 1);
        assert!(results[0]["result"].is_object(), "capability.list should succeed");
    }

    #[tokio::test]
    async fn test_liveness_rejects_data_method() {
        let primal = test_primal().await;
        let input =
            b"{\"jsonrpc\":\"2.0\",\"method\":\"dag.session.create\",\"params\":{},\"id\":3}\n";
        let results = liveness_roundtrip(primal, input).await;
        assert_eq!(results.len(), 1);
        assert_eq!(
            results[0]["error"]["code"], -32000,
            "data methods should be rejected with FORBIDDEN"
        );
        let data = &results[0]["error"]["data"];
        assert_eq!(data["method"], "dag.session.create");
    }

    #[tokio::test]
    async fn test_liveness_rejects_batch() {
        let primal = test_primal().await;
        let input = b"[{\"jsonrpc\":\"2.0\",\"method\":\"health.check\",\"params\":{},\"id\":1}]\n";
        let results = liveness_roundtrip(primal, input).await;
        assert_eq!(results.len(), 1);
        assert_eq!(
            results[0]["error"]["code"], -32600,
            "batch requests should be rejected on liveness"
        );
    }
}
