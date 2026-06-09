// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2024–2026 ecoPrimals Project

//! Shared test helpers for JSON-RPC handler tests.
//!
//! Consumed by `handler_tests.rs`, `handler_tests_validation.rs`, and
//! property-based test modules via `use super::test_support::*`.

#![expect(clippy::unwrap_used, reason = "test code")]

use crate::jsonrpc::method_gate::{CallerContext, EnforcementMode, MethodGate};
use crate::jsonrpc::types::{JsonRpcId, JsonRpcRequest};
use rhizo_crypt_core::{PrimalLifecycle, RhizoCryptConfig};
use serde_json::Value;
use std::sync::Arc;

pub fn test_gate() -> MethodGate {
    MethodGate::with_noop(EnforcementMode::Permissive)
}

pub fn test_caller() -> CallerContext {
    CallerContext::unix()
}

pub async fn create_test_primal() -> Arc<rhizo_crypt_core::RhizoCrypt> {
    let mut primal = rhizo_crypt_core::RhizoCrypt::new(RhizoCryptConfig::default());
    primal.start().await.unwrap();
    Arc::new(primal)
}

pub async fn create_test_server() -> crate::service::RhizoCryptRpcServer {
    let primal = create_test_primal().await;
    crate::service::RhizoCryptRpcServer::new(primal)
}

pub fn make_request(method: &str, params: Option<Value>) -> JsonRpcRequest {
    JsonRpcRequest {
        jsonrpc: "2.0".to_string(),
        method: method.to_string(),
        params,
        id: Some(JsonRpcId::Number(1)),
    }
}
