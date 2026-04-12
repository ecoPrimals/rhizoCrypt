// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2024–2026 ecoPrimals Project

use super::*;

#[test]
fn test_error_display() {
    let err = RhizoCryptError::config("test error");
    assert_eq!(err.to_string(), "configuration error: test error");
}

#[test]
fn test_invalid_config() {
    let err = RhizoCryptError::invalid_config("max_vertices", "must be positive");
    assert!(err.to_string().contains("max_vertices"));
    assert!(err.to_string().contains("must be positive"));
}

#[test]
fn test_is_recoverable() {
    assert!(RhizoCryptError::Timeout(1000).is_recoverable());
    assert!(RhizoCryptError::storage("disk full").is_recoverable());
    assert!(RhizoCryptError::integration("service unavailable").is_recoverable());
    assert!(RhizoCryptError::capability_provider("signing", "timeout").is_recoverable());
    assert!(!RhizoCryptError::config("invalid").is_recoverable());
}

#[test]
fn test_capability_provider_error() {
    let err = RhizoCryptError::capability_provider("signing", "key not found");
    assert!(err.to_string().contains("signing"));
    assert!(err.to_string().contains("key not found"));
    assert!(err.is_recoverable());
}

#[test]
fn test_is_not_found() {
    let session_id = SessionId::now();
    assert!(RhizoCryptError::SessionNotFound(session_id).is_not_found());
    assert!(!RhizoCryptError::config("test").is_not_found());
}

#[test]
fn test_ipc_error_phases() {
    let connect = RhizoCryptError::ipc(IpcErrorPhase::Connect, "socket missing");
    assert!(connect.to_string().contains("connect"));
    assert!(connect.to_string().contains("socket missing"));
    assert!(connect.is_recoverable());

    let write = RhizoCryptError::ipc(IpcErrorPhase::Write, "broken pipe");
    assert!(write.to_string().contains("write"));

    let read = RhizoCryptError::ipc(IpcErrorPhase::Read, "timeout");
    assert!(read.to_string().contains("read"));

    let invalid = RhizoCryptError::ipc(IpcErrorPhase::InvalidJson, "unexpected EOF");
    assert!(invalid.to_string().contains("invalid_json"));

    let http = RhizoCryptError::ipc(IpcErrorPhase::HttpStatus(500), "internal");
    assert!(http.to_string().contains("http_500"));

    let no_result = RhizoCryptError::ipc(IpcErrorPhase::NoResult, "missing field");
    assert!(no_result.to_string().contains("no_result"));

    let rpc = RhizoCryptError::ipc(IpcErrorPhase::JsonRpcError(-32601), "method not found");
    assert!(rpc.to_string().contains("jsonrpc_-32601"));
}

#[test]
fn test_ipc_error_phase_display() {
    assert_eq!(IpcErrorPhase::Connect.to_string(), "connect");
    assert_eq!(IpcErrorPhase::Write.to_string(), "write");
    assert_eq!(IpcErrorPhase::Read.to_string(), "read");
    assert_eq!(IpcErrorPhase::InvalidJson.to_string(), "invalid_json");
    assert_eq!(IpcErrorPhase::HttpStatus(404).to_string(), "http_404");
    assert_eq!(IpcErrorPhase::NoResult.to_string(), "no_result");
    assert_eq!(IpcErrorPhase::JsonRpcError(-32600).to_string(), "jsonrpc_-32600");
}

#[test]
fn test_ipc_error_phase_equality() {
    assert_eq!(IpcErrorPhase::Connect, IpcErrorPhase::Connect);
    assert_ne!(IpcErrorPhase::Connect, IpcErrorPhase::Write);
    assert_eq!(IpcErrorPhase::HttpStatus(500), IpcErrorPhase::HttpStatus(500));
    assert_ne!(IpcErrorPhase::HttpStatus(500), IpcErrorPhase::HttpStatus(404));
}

#[test]
fn test_ipc_phase_is_method_not_found() {
    assert!(IpcErrorPhase::JsonRpcError(-32601).is_method_not_found());
    assert!(!IpcErrorPhase::JsonRpcError(-32600).is_method_not_found());
    assert!(!IpcErrorPhase::Connect.is_method_not_found());
}

#[test]
fn test_ipc_phase_is_timeout_likely() {
    assert!(IpcErrorPhase::Connect.is_timeout_likely());
    assert!(IpcErrorPhase::Read.is_timeout_likely());
    assert!(!IpcErrorPhase::Write.is_timeout_likely());
    assert!(!IpcErrorPhase::JsonRpcError(-1).is_timeout_likely());
}

#[test]
fn test_ipc_phase_is_retriable() {
    assert!(IpcErrorPhase::Connect.is_retriable());
    assert!(IpcErrorPhase::Write.is_retriable());
    assert!(IpcErrorPhase::Read.is_retriable());
    assert!(!IpcErrorPhase::InvalidJson.is_retriable());
    assert!(!IpcErrorPhase::NoResult.is_retriable());
    assert!(!IpcErrorPhase::HttpStatus(500).is_retriable());
    assert!(!IpcErrorPhase::JsonRpcError(-32601).is_retriable());
}

#[test]
fn test_ipc_phase_is_application_error() {
    assert!(IpcErrorPhase::JsonRpcError(-32601).is_application_error());
    assert!(IpcErrorPhase::NoResult.is_application_error());
    assert!(!IpcErrorPhase::Connect.is_application_error());
    assert!(!IpcErrorPhase::Read.is_application_error());
}

#[test]
fn test_dispatch_outcome_ok() {
    let outcome: DispatchOutcome<i32> = DispatchOutcome::Ok(42);
    assert!(outcome.is_ok());
    assert_eq!(outcome.into_result().unwrap(), 42);
}

#[test]
fn test_dispatch_outcome_application_error() {
    let outcome: DispatchOutcome<i32> = DispatchOutcome::ApplicationError {
        code: -32601,
        message: "method not found".into(),
    };
    assert!(!outcome.is_ok());
    let err = outcome.into_result().unwrap_err();
    assert!(err.to_string().contains("jsonrpc_-32601"));
    assert!(err.to_string().contains("method not found"));
}

#[test]
fn test_dispatch_outcome_protocol_error() {
    let outcome: DispatchOutcome<i32> =
        DispatchOutcome::ProtocolError(RhizoCryptError::ipc(IpcErrorPhase::Connect, "timeout"));
    assert!(!outcome.is_ok());
    let err = outcome.into_result().unwrap_err();
    assert!(err.to_string().contains("connect"));
}

#[test]
fn test_extract_rpc_error_present() {
    let response = serde_json::json!({
        "jsonrpc": "2.0",
        "error": {"code": -32601, "message": "Method not found"},
        "id": 1
    });
    let (code, msg) = extract_rpc_error(&response).unwrap();
    assert_eq!(code, -32601);
    assert_eq!(msg, "Method not found");
}

#[test]
fn test_extract_rpc_error_absent() {
    let response = serde_json::json!({
        "jsonrpc": "2.0",
        "result": true,
        "id": 1
    });
    assert!(extract_rpc_error(&response).is_none());
}

#[test]
fn test_extract_rpc_error_defaults() {
    let response = serde_json::json!({
        "error": {}
    });
    let (code, msg) = extract_rpc_error(&response).unwrap();
    assert_eq!(code, -1);
    assert_eq!(msg, "Unknown error");
}
