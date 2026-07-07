// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2024–2026 ecoPrimals Project

//! Tests for `JsonRpcTransportError` display and source chaining.

use super::*;

#[test]
fn test_jsonrpc_transport_error_display_connect_timeout() {
    let err = JsonRpcTransportError::ConnectTimeout;
    assert_eq!(err.to_string(), "connection timed out");
    assert!(std::error::Error::source(&err).is_none());
}

#[test]
fn test_jsonrpc_transport_error_display_connect_failed() {
    let io_err = std::io::Error::new(std::io::ErrorKind::ConnectionRefused, "refused");
    let err = JsonRpcTransportError::ConnectFailed(io_err);
    assert!(err.to_string().contains("connection failed"));
    assert!(std::error::Error::source(&err).is_some());
}

#[test]
fn test_jsonrpc_transport_error_display_response_timeout() {
    let err = JsonRpcTransportError::ResponseTimeout;
    assert_eq!(err.to_string(), "response timed out");
    assert!(std::error::Error::source(&err).is_none());
}

#[test]
fn test_jsonrpc_transport_error_display_write() {
    let io_err = std::io::Error::new(std::io::ErrorKind::BrokenPipe, "broken");
    let err = JsonRpcTransportError::Write(io_err);
    assert!(err.to_string().contains("write failed"));
    assert!(std::error::Error::source(&err).is_some());
}

#[test]
fn test_jsonrpc_transport_error_display_read() {
    let io_err = std::io::Error::new(std::io::ErrorKind::UnexpectedEof, "eof");
    let err = JsonRpcTransportError::Read(io_err);
    assert!(err.to_string().contains("read failed"));
    assert!(std::error::Error::source(&err).is_some());
}

#[test]
fn test_jsonrpc_transport_error_display_serialize() {
    let serde_err = serde_json::from_str::<serde_json::Value>("bad").unwrap_err();
    let err = JsonRpcTransportError::Serialize(serde_err);
    assert!(err.to_string().contains("serialize failed"));
    assert!(std::error::Error::source(&err).is_some());
}
