// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2024–2026 ecoPrimals Project

//! Property-based tests for JSON-RPC handler protocol invariants.

#![expect(clippy::unwrap_used, reason = "test code")]

use super::{HandlerError, handle_request};
use crate::jsonrpc::types::{JsonRpcId, JsonRpcRequest};
use proptest::prelude::*;
use rhizo_crypt_core::PrimalLifecycle;
use rhizo_crypt_core::RhizoCryptConfig;
use rhizo_crypt_core::niche::{self, CAPABILITIES, SEMANTIC_MAPPINGS};
use serde_json::{Value, json};
use std::sync::Arc;

fn capability_index() -> impl Strategy<Value = usize> {
    prop::sample::select((0..CAPABILITIES.len()).collect::<Vec<_>>())
}

fn semantic_mapping_index() -> impl Strategy<Value = usize> {
    prop::sample::select((0..SEMANTIC_MAPPINGS.len()).collect::<Vec<_>>())
}

fn liveness_method() -> impl Strategy<Value = &'static str> {
    prop::sample::select(&["health.liveness", "ping", "health"][..])
}

fn arbitrary_json_params() -> impl Strategy<Value = Option<Value>> {
    prop_oneof![
        Just(None),
        Just(Some(Value::Null)),
        Just(Some(json!({}))),
        any::<i64>().prop_map(|n| Some(json!(n))),
        prop::collection::vec(any::<u8>(), 0..32).prop_map(|bytes| {
            Some(Value::Array(bytes.into_iter().map(|b| json!(b)).collect::<Vec<_>>()))
        }),
    ]
}

async fn create_test_primal() -> Arc<rhizo_crypt_core::RhizoCrypt> {
    let mut primal = rhizo_crypt_core::RhizoCrypt::new(RhizoCryptConfig::default());
    primal.start().await.unwrap();
    Arc::new(primal)
}

fn make_request(method: &str, params: Option<Value>) -> JsonRpcRequest {
    JsonRpcRequest {
        jsonrpc: "2.0".to_string(),
        method: method.to_string(),
        params,
        id: Some(JsonRpcId::Number(1)),
    }
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    /// Every capability name in [`CAPABILITIES`] routes to a handler arm (never `MethodNotFound`).
    #[test]
    fn prop_capability_routes_without_method_not_found(idx in capability_index()) {
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();
        let cap = CAPABILITIES[idx];
        let primal = rt.block_on(create_test_primal());
        let req = make_request(cap, Some(json!({})));
        let result = rt.block_on(handle_request(primal, req));
        prop_assert!(
            !matches!(result, Err(HandlerError::MethodNotFound(_))),
            "capability {cap} must not produce MethodNotFound, got {result:?}"
        );
    }

    /// Unregistered method names always yield `MethodNotFound`.
    #[test]
    fn prop_unknown_method_names_return_method_not_found(salt in any::<u64>()) {
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();
        let method = format!("dag.__proptest_unknown.{salt}");
        let primal = rt.block_on(create_test_primal());
        let req = make_request(&method, Some(json!({})));
        let result = rt.block_on(handle_request(primal, req));
        prop_assert!(
            matches!(result, Err(HandlerError::MethodNotFound(_))),
            "expected MethodNotFound for {method}, got {result:?}"
        );
    }

    /// [`niche::normalize_method`] is idempotent.
    #[test]
    fn prop_normalize_method_is_idempotent(s in prop::collection::vec(any::<char>(), 0..256)
        .prop_map(|cs| cs.into_iter().collect::<String>()))
    {
        let once = niche::normalize_method(&s);
        let twice = niche::normalize_method(once);
        prop_assert_eq!(once, twice);
    }

    /// Each semantic short name maps to a string listed in [`CAPABILITIES`].
    #[test]
    fn prop_semantic_mappings_resolve_to_capabilities(idx in semantic_mapping_index()) {
        let (_short, full) = SEMANTIC_MAPPINGS[idx];
        prop_assert!(
            CAPABILITIES.contains(&full),
            "SEMANTIC_MAPPINGS[{idx}] target {full} must appear in CAPABILITIES"
        );
    }

    /// `health.liveness` / `ping` / `health` return the fixed liveness JSON regardless of params.
    #[test]
    fn prop_health_liveness_is_stateless(
        method in liveness_method(),
        params in arbitrary_json_params(),
    ) {
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();
        let primal = rt.block_on(create_test_primal());
        let req = make_request(method, params);
        let got = rt.block_on(handle_request(primal, req)).unwrap();
        prop_assert_eq!(got, niche::health_liveness());
    }
}
