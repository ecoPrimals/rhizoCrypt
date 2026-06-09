// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2024–2026 ecoPrimals Project

//! Readiness gate (PG-60) and JH-0 method gate / auth tests.

#![expect(clippy::unwrap_used, reason = "test code")]

use super::test_support::{create_test_server, make_request, test_caller, test_gate};
use super::*;
use crate::jsonrpc::method_gate::{CallerContext, EnforcementMode, MethodGate};
use rhizo_crypt_core::RhizoCryptConfig;
use serde_json::json;
use std::sync::Arc;

// ============================================================================
// Readiness gate (PG-60)
// ============================================================================

fn create_unstarted_server() -> crate::service::RhizoCryptRpcServer {
    let primal = Arc::new(rhizo_crypt_core::RhizoCrypt::new(RhizoCryptConfig::default()));
    crate::service::RhizoCryptRpcServer::new(primal)
}

#[tokio::test]
async fn test_readiness_gate_rejects_dag_methods_when_not_running() {
    let server = create_unstarted_server();
    assert!(!server.primal().state().is_running());

    let dag_methods = [
        "dag.session.create",
        "dag.session.get",
        "dag.session.list",
        "dag.event.append",
        "dag.vertex.get",
        "dag.merkle.root",
        "dag.slice.checkout",
        "dag.dehydration.trigger",
    ];

    for method in dag_methods {
        let req = make_request(method, None);
        let err =
            handle_request(&server, req, &test_gate(), &test_caller()).await.unwrap_err();
        assert!(
            matches!(err, HandlerError::NotReady),
            "{method} should return NotReady when primal is not running"
        );
    }
}

#[tokio::test]
async fn test_readiness_gate_allows_health_probes_when_not_running() {
    let server = create_unstarted_server();

    let allowed_methods = [
        "health.liveness",
        "ping",
        "health",
        "health.check",
        "health.readiness",
        "identity.get",
        "capabilities.list",
        "tools.list",
    ];

    for method in allowed_methods {
        let result = handle_request(
            &server,
            make_request(method, None),
            &test_gate(),
            &test_caller(),
        )
        .await;
        assert!(
            result.is_ok(),
            "{method} should succeed even when primal is not running, got: {result:?}"
        );
    }
}

#[tokio::test]
async fn test_readiness_gate_passes_when_running() {
    let server = create_test_server().await;
    assert!(server.primal().state().is_running());

    let req = make_request(
        "dag.session.create",
        Some(json!({"session_type": "General", "description": "readiness test"})),
    );
    let result = handle_request(&server, req, &test_gate(), &test_caller()).await;
    assert!(result.is_ok(), "DAG methods should work when primal is running");
}

// =========================================================================
// JH-0 Method Gate Tests
// =========================================================================

#[tokio::test]
async fn test_auth_check_returns_unauthenticated_permissive() {
    let server = create_test_server().await;
    let gate = MethodGate::with_noop(EnforcementMode::Permissive);
    let caller = CallerContext::unix();

    let req = make_request("auth.check", None);
    let result = handle_request(&server, req, &gate, &caller).await.unwrap();
    assert_eq!(result["authenticated"], false);
    assert_eq!(result["enforcement"], "permissive");
}

#[tokio::test]
async fn test_auth_mode_returns_current_mode() {
    let server = create_test_server().await;
    let gate = MethodGate::with_noop(EnforcementMode::Enforced);
    let caller = CallerContext::unix();

    let req = make_request("auth.mode", None);
    let result = handle_request(&server, req, &gate, &caller).await.unwrap();
    assert_eq!(result["mode"], "enforced");
}

#[tokio::test]
async fn test_auth_peer_info_returns_origin() {
    let server = create_test_server().await;
    let gate = test_gate();
    let caller = CallerContext::unix();

    let req = make_request("auth.peer_info", None);
    let result = handle_request(&server, req, &gate, &caller).await.unwrap();
    assert_eq!(result["origin"], "Unix");
    assert_eq!(result["has_token"], false);
}

#[tokio::test]
async fn test_enforced_gate_rejects_protected_without_token() {
    let server = create_test_server().await;
    let gate = MethodGate::with_noop(EnforcementMode::Enforced);
    let caller = CallerContext::unix();

    let req = make_request("dag.session.create", Some(json!({"session_type": "General"})));
    let err = handle_request(&server, req, &gate, &caller).await.unwrap_err();
    assert!(
        matches!(err, HandlerError::PermissionDenied(_)),
        "expected PermissionDenied, got: {err:?}"
    );
}

#[tokio::test]
async fn test_enforced_gate_allows_public_without_token() {
    let server = create_test_server().await;
    let gate = MethodGate::with_noop(EnforcementMode::Enforced);
    let caller = CallerContext::unix();

    let public_methods = [
        "health.check",
        "health.liveness",
        "identity.get",
        "capabilities.list",
        "auth.check",
        "auth.mode",
        "auth.peer_info",
        "tools.list",
    ];

    for method in public_methods {
        let req = make_request(method, None);
        let result = handle_request(&server, req, &gate, &caller).await;
        assert!(result.is_ok(), "{method} should be allowed even in enforced mode without a token");
    }
}

#[tokio::test]
async fn test_enforced_gate_allows_protected_with_token() {
    let server = create_test_server().await;
    let gate = MethodGate::with_noop(EnforcementMode::Enforced);
    let mut caller = CallerContext::with_bearer_token(
        Some("test-ionic-token".to_owned()),
        crate::jsonrpc::method_gate::ConnectionOrigin::Unix,
    );
    caller.verify_token(gate.verifier());

    let req = make_request("dag.session.create", Some(json!({"session_type": "General"})));
    let result = handle_request(&server, req, &gate, &caller).await;
    assert!(result.is_ok(), "protected method with token should succeed in enforced mode");
}

#[tokio::test]
async fn test_permissive_gate_allows_protected_without_token() {
    let server = create_test_server().await;
    let gate = MethodGate::with_noop(EnforcementMode::Permissive);
    let caller = CallerContext::unix();

    let req = make_request("dag.session.create", Some(json!({"session_type": "General"})));
    let result = handle_request(&server, req, &gate, &caller).await;
    assert!(result.is_ok(), "permissive mode should allow unauthenticated protected calls");
}

#[tokio::test]
async fn test_auth_methods_work_when_primal_not_running() {
    let server = create_unstarted_server();
    let gate = MethodGate::with_noop(EnforcementMode::Enforced);
    let caller = CallerContext::unix();

    for method in ["auth.check", "auth.mode", "auth.peer_info"] {
        let req = make_request(method, None);
        let result = handle_request(&server, req, &gate, &caller).await;
        assert!(result.is_ok(), "{method} should work even when primal is not running");
    }
}

/// All `health.*` prefix methods bypass readiness via `PUBLIC_METHOD_PREFIXES`.
#[tokio::test]
async fn test_readiness_gate_allows_health_prefix_when_not_running() {
    let server = create_unstarted_server();
    assert!(!server.primal().state().is_running());

    for method in ["health.check", "health.liveness", "health.readiness", "health.metrics"] {
        let req = make_request(method, None);
        let result = handle_request(&server, req, &test_gate(), &test_caller()).await;
        assert!(result.is_ok(), "{method} should bypass readiness gate (health.* prefix)");
    }
}

/// Exact public methods bypass readiness gate even when not running.
#[tokio::test]
async fn test_readiness_gate_allows_exact_public_methods_when_not_running() {
    let server = create_unstarted_server();
    assert!(!server.primal().state().is_running());

    for method in ["primal.capabilities", "capabilities.list", "identity.get", "tools.list"] {
        let req = make_request(method, None);
        let result = handle_request(&server, req, &test_gate(), &test_caller()).await;
        assert!(result.is_ok(), "{method} should bypass readiness gate (exact public)");
    }
}
