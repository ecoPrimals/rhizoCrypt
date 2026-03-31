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
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tracing::warn;

use super::types::{codes, error_response};
use super::{process_single_request, serialize_response};

/// Handle a newline-delimited JSON-RPC connection over any async stream.
///
/// Reads lines from `stream`, parses each as a JSON-RPC request (single
/// or batch), dispatches through the same handler as the HTTP path, and
/// writes the response followed by `\n`. Empty lines are silently skipped.
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
    let mut lines = BufReader::new(reader).lines();

    while let Some(line) = lines.next_line().await? {
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
}
