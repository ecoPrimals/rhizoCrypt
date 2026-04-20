// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2024–2026 ecoPrimals Project

//! JSON-RPC 2.0 envelope types.
//!
//! Defines request, response, and error structures per the JSON-RPC 2.0 spec.

use serde::{Deserialize, Serialize};
use serde_json::Value;

/// JSON-RPC 2.0 request envelope.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcRequest {
    /// Must be "2.0".
    pub jsonrpc: String,

    /// Method name (semantic: domain.operation.variant).
    pub method: String,

    /// Optional parameters (object or array).
    #[serde(default)]
    pub params: Option<Value>,

    /// Request ID (number, string, or null for notifications).
    pub id: Option<JsonRpcId>,
}

/// JSON-RPC request ID (number or string).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(untagged)]
pub enum JsonRpcId {
    /// Numeric ID.
    Number(i64),

    /// String ID.
    String(String),
}

impl From<i64> for JsonRpcId {
    fn from(n: i64) -> Self {
        Self::Number(n)
    }
}

impl From<String> for JsonRpcId {
    fn from(s: String) -> Self {
        Self::String(s)
    }
}

/// JSON-RPC 2.0 success response envelope.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcSuccessResponse {
    /// Must be "2.0".
    pub jsonrpc: String,

    /// Result payload.
    pub result: Value,

    /// Request ID (echoed from request).
    pub id: JsonRpcId,
}

/// JSON-RPC 2.0 error response envelope.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcErrorResponse {
    /// Must be "2.0".
    pub jsonrpc: String,

    /// Error object.
    pub error: JsonRpcError,

    /// Request ID (null if parse error).
    pub id: Option<JsonRpcId>,
}

/// JSON-RPC 2.0 error object.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcError {
    /// Standard error code.
    pub code: i32,

    /// Human-readable message.
    pub message: String,

    /// Optional additional data.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<Value>,
}

/// Standard JSON-RPC 2.0 error codes.
pub mod codes {
    /// Parse error - Invalid JSON.
    pub const PARSE_ERROR: i32 = -32700;

    /// Invalid Request - JSON is valid but not a valid Request object.
    pub const INVALID_REQUEST: i32 = -32600;

    /// Method not found.
    pub const METHOD_NOT_FOUND: i32 = -32601;

    /// Invalid params.
    pub const INVALID_PARAMS: i32 = -32602;

    /// Internal error.
    pub const INTERNAL_ERROR: i32 = -32603;

    /// Authentication required (BTSP handshake not performed).
    pub const FORBIDDEN: i32 = -32000;
}

/// Create a success response.
#[must_use]
pub fn success(id: JsonRpcId, result: Value) -> JsonRpcSuccessResponse {
    JsonRpcSuccessResponse {
        jsonrpc: "2.0".to_string(),
        result,
        id,
    }
}

/// Create an error response.
#[must_use]
pub fn error_response(
    id: Option<JsonRpcId>,
    code: i32,
    message: &str,
    data: Option<Value>,
) -> JsonRpcErrorResponse {
    JsonRpcErrorResponse {
        jsonrpc: "2.0".to_string(),
        error: JsonRpcError {
            code,
            message: message.to_string(),
            data,
        },
        id,
    }
}

#[cfg(test)]
#[expect(clippy::expect_used, reason = "test code")]
mod tests {
    use super::*;

    #[test]
    fn test_request_deserialize() {
        let json = r#"{"jsonrpc":"2.0","method":"health.check","params":{},"id":1}"#;
        let req: JsonRpcRequest = serde_json::from_str(json).expect("parse");
        assert_eq!(req.method, "health.check");
        assert_eq!(req.id, Some(JsonRpcId::Number(1)));
    }

    #[test]
    fn test_success_response_serialize() {
        let resp = success(JsonRpcId::Number(1), serde_json::json!({"healthy": true}));
        let json = serde_json::to_string(&resp).expect("serialize");
        assert!(json.contains("\"result\""));
        assert!(json.contains("\"healthy\""));
    }

    #[test]
    fn test_error_response_serialize() {
        let resp = error_response(
            Some(JsonRpcId::Number(1)),
            codes::METHOD_NOT_FOUND,
            "Method not found",
            None,
        );
        let json = serde_json::to_string(&resp).expect("serialize");
        assert!(json.contains("\"error\""));
        assert!(json.contains("-32601"));
    }
}
