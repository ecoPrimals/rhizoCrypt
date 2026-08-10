// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2024–2026 ecoPrimals Project

//! Bearer extraction, caller context, gate authorization, auth responses,
//! G63 peer credentials, and BTSP bridge tests.

use super::*;

// ── Bearer token extraction ──────────────────────────────────────

#[test]
fn extract_bearer_token_from_params() {
    let mut params = serde_json::json!({
        "_bearer_token": "my-token",
        "session_id": "abc"
    });
    let token = extract_bearer_token(&mut params);
    assert_eq!(token.as_deref(), Some("my-token"));
    assert!(params.get("_bearer_token").is_none());
    assert_eq!(params["session_id"], "abc");
}

#[test]
fn extract_bearer_token_missing() {
    let mut params = serde_json::json!({"session_id": "abc"});
    assert!(extract_bearer_token(&mut params).is_none());
}

#[test]
fn extract_bearer_token_null_params() {
    let mut params = Value::Null;
    assert!(extract_bearer_token(&mut params).is_none());
}

#[test]
fn extract_bearer_token_non_string() {
    let mut params = serde_json::json!({"_bearer_token": 42});
    assert!(extract_bearer_token(&mut params).is_none());
}

// ── Caller context ───────────────────────────────────────────────

#[test]
fn caller_context_verify_populates_claims() {
    let verifier = NoopVerifier;
    let mut ctx = CallerContext::with_bearer_token(Some("tok".to_owned()), ConnectionOrigin::Unix);
    assert!(!ctx.is_verified());
    ctx.verify_token(&verifier);
    assert!(ctx.is_verified());
    let claims = ctx.verified_claims.as_ref().unwrap();
    assert_eq!(claims.scopes, vec!["*"]);
}

#[test]
fn caller_context_verify_without_token() {
    let verifier = NoopVerifier;
    let mut ctx = CallerContext::unix();
    ctx.verify_token(&verifier);
    assert!(!ctx.is_verified());
}

#[test]
fn caller_context_loopback_constructor() {
    let ctx = CallerContext::loopback();
    assert_eq!(ctx.origin, ConnectionOrigin::Loopback);
    assert!(ctx.bearer_token.is_none());
    assert!(!ctx.is_verified());
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn caller_context_verify_token_async() {
    let verifier = NoopVerifier;
    let mut ctx = CallerContext::with_bearer_token(Some("tok".to_owned()), ConnectionOrigin::Unix);
    ctx.verify_token_async(&verifier).await;
    assert!(ctx.is_verified());
    assert_eq!(ctx.verified_claims.as_ref().unwrap().subject, "unknown");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn caller_context_verify_token_async_without_token() {
    let verifier = NoopVerifier;
    let mut ctx = CallerContext::unix();
    ctx.verify_token_async(&verifier).await;
    assert!(!ctx.is_verified());
}

// ── Gate check (basic) ───────────────────────────────────────────

#[test]
fn gate_allows_public_methods_without_token() {
    let gate = enforced_gate();
    let caller = CallerContext::unix();
    assert!(gate.check("health.check", &caller).is_ok());
    assert!(gate.check("identity.get", &caller).is_ok());
    assert!(gate.check("auth.mode", &caller).is_ok());
}

#[test]
fn gate_allows_protected_with_verified_token() {
    let gate = enforced_gate();
    let caller = verified_caller("test-token");
    assert!(gate.check("dag.session.create", &caller).is_ok());
}

#[test]
fn gate_permissive_allows_protected_without_token() {
    let gate = test_gate();
    let caller = CallerContext::unix();
    assert!(gate.check("dag.session.create", &caller).is_ok());
}

#[test]
fn gate_enforced_rejects_protected_without_token() {
    let gate = enforced_gate();
    let caller = CallerContext::unix();
    let result = gate.check("dag.session.create", &caller);
    assert!(result.is_err());
    let rejection = result.unwrap_err();
    assert_eq!(rejection.method, "dag.session.create");
}

// ── Gate check (scope enforcement) ───────────────────────────────

#[test]
fn gate_rejects_when_scope_does_not_cover_method() {
    let gate = enforced_gate();
    let mut caller =
        CallerContext::with_bearer_token(Some("tok".to_owned()), ConnectionOrigin::Unix);
    caller.verified_claims = Some(VerifiedClaims {
        subject: "alice".to_owned(),
        scopes: vec!["crypto.*".to_owned()],
        expires_in: None,
    });
    assert!(gate.check("dag.session.create", &caller).is_err());
}

#[test]
fn gate_allows_when_scope_covers_method() {
    let gate = enforced_gate();
    let mut caller =
        CallerContext::with_bearer_token(Some("tok".to_owned()), ConnectionOrigin::Unix);
    caller.verified_claims = Some(VerifiedClaims {
        subject: "alice".to_owned(),
        scopes: vec!["dag.*".to_owned()],
        expires_in: None,
    });
    assert!(gate.check("dag.session.create", &caller).is_ok());
}

#[test]
fn gate_rejects_unverified_token() {
    let gate = enforced_gate();
    let caller =
        CallerContext::with_bearer_token(Some("bad-token".to_owned()), ConnectionOrigin::Unix);
    assert!(gate.check("dag.session.create", &caller).is_err());
}

#[test]
fn gate_permissive_rejects_failed_token() {
    let gate = test_gate();
    let caller = CallerContext::with_bearer_token(Some("bad".to_owned()), ConnectionOrigin::Remote);
    let result = gate.check("dag.session.create", &caller);
    assert!(result.is_err());
}

// ── Auth responses ───────────────────────────────────────────────

#[test]
fn auth_check_response_unauthenticated() {
    let gate = test_gate();
    let caller = CallerContext::unix();
    let resp = gate.auth_check_response(&caller);
    assert_eq!(resp["authenticated"], false);
    assert_eq!(resp["verified"], false);
    assert_eq!(resp["enforcement"], "permissive");
    assert!(resp.get("scopes").is_none());
    assert!(resp.get("subject").is_none());
}

#[test]
fn auth_check_response_verified_with_claims() {
    let gate = enforced_gate();
    let caller = verified_caller("tok");
    let resp = gate.auth_check_response(&caller);
    assert_eq!(resp["authenticated"], true);
    assert_eq!(resp["verified"], true);
    assert_eq!(resp["enforcement"], "enforced");
    assert_eq!(resp["scopes"], serde_json::json!(["*"]));
    assert!(resp["subject"].is_string());
}

#[test]
fn auth_check_response_includes_expires_in() {
    let gate = test_gate();
    let mut caller =
        CallerContext::with_bearer_token(Some("tok".to_owned()), ConnectionOrigin::Unix);
    caller.verified_claims = Some(VerifiedClaims {
        subject: "alice".to_owned(),
        scopes: vec!["dag.*".to_owned()],
        expires_in: Some(3600),
    });
    let resp = gate.auth_check_response(&caller);
    assert_eq!(resp["expires_in"], 3600);
}

#[test]
fn auth_mode_response() {
    let gate = test_gate();
    assert_eq!(gate.auth_mode_response()["mode"], "permissive");
    let gate = enforced_gate();
    assert_eq!(gate.auth_mode_response()["mode"], "enforced");
}

#[test]
fn auth_peer_info_response() {
    let gate = test_gate();
    let caller = CallerContext::unix();
    let resp = gate.auth_peer_info_response(&caller);
    assert_eq!(resp["origin"], "Unix");
    assert_eq!(resp["has_token"], false);
}

#[test]
fn auth_peer_info_response_loopback_with_token() {
    let gate = test_gate();
    let caller =
        CallerContext::with_bearer_token(Some("tok".to_owned()), ConnectionOrigin::Loopback);
    let resp = gate.auth_peer_info_response(&caller);
    assert_eq!(resp["origin"], "Loopback");
    assert_eq!(resp["has_token"], true);
}

#[test]
fn auth_peer_info_response_remote() {
    let gate = test_gate();
    let caller = CallerContext::with_bearer_token(None, ConnectionOrigin::Remote);
    let resp = gate.auth_peer_info_response(&caller);
    assert_eq!(resp["origin"], "Remote");
    assert_eq!(resp["has_token"], false);
}

// ── G63 peer credential tests ────────────────────────────────────

#[test]
fn auth_peer_info_response_with_peer_cred() {
    let gate = test_gate();
    let cred = PeerCredentials {
        uid: 1000,
        gid: 1000,
        pid: Some(42),
    };
    let caller = CallerContext::unix_with_peer(cred);
    let resp = gate.auth_peer_info_response(&caller);
    assert_eq!(resp["origin"], "Unix");
    assert_eq!(resp["peer_uid"], 1000);
    assert_eq!(resp["peer_gid"], 1000);
    assert_eq!(resp["peer_pid"], 42);
    assert_eq!(resp["has_token"], false);
}

#[test]
fn auth_peer_info_response_btsp_with_peer_cred() {
    let gate = test_gate();
    let cred = PeerCredentials {
        uid: 0,
        gid: 0,
        pid: None,
    };
    let caller = CallerContext::btsp_authenticated_with_peer(cred);
    let resp = gate.auth_peer_info_response(&caller);
    assert_eq!(resp["origin"], "BtspAuthenticated");
    assert_eq!(resp["peer_uid"], 0);
    assert_eq!(resp["peer_gid"], 0);
    assert!(resp.get("peer_pid").is_none());
}

#[test]
fn auth_peer_info_response_no_peer_cred() {
    let gate = test_gate();
    let caller = CallerContext::unix();
    let resp = gate.auth_peer_info_response(&caller);
    assert!(resp.get("peer_uid").is_none());
    assert!(resp.get("peer_gid").is_none());
    assert!(resp.get("peer_pid").is_none());
}

#[test]
fn peer_credentials_has_peer_cred() {
    let cred = PeerCredentials {
        uid: 1000,
        gid: 1000,
        pid: Some(123),
    };
    let with = CallerContext::unix_with_peer(cred);
    assert!(with.has_peer_cred());
    let without = CallerContext::unix();
    assert!(!without.has_peer_cred());
}

// ============================================================================
// BTSP → CallerContext bridge tests
// ============================================================================

#[test]
fn btsp_authenticated_origin_identity() {
    let caller = CallerContext::btsp_authenticated();
    assert_eq!(caller.origin, ConnectionOrigin::BtspAuthenticated);
    assert!(caller.origin.is_btsp_authenticated());
    assert!(caller.bearer_token.is_none());
    assert!(caller.verified_claims.is_none());
}

#[test]
fn btsp_authenticated_origin_identity_and_exclusivity() {
    assert_eq!(ConnectionOrigin::BtspAuthenticated.as_str(), "BtspAuthenticated");
    assert!(!ConnectionOrigin::Unix.is_btsp_authenticated());
    assert!(!ConnectionOrigin::Loopback.is_btsp_authenticated());
    assert!(!ConnectionOrigin::Remote.is_btsp_authenticated());
}

#[test]
fn btsp_authenticated_grants_protected_method_permissive() {
    let gate = test_gate();
    let caller = CallerContext::btsp_authenticated();
    assert!(gate.check("dag.event.append", &caller).is_ok());
}

#[test]
fn btsp_authenticated_grants_protected_method_enforced() {
    let gate = enforced_gate();
    let caller = CallerContext::btsp_authenticated();
    assert!(gate.check("dag.event.append", &caller).is_ok());
    assert!(gate.check("dag.federate", &caller).is_ok());
    assert!(gate.check("dag.dehydrate", &caller).is_ok());
}

#[test]
fn unauthenticated_unix_rejected_enforced() {
    let gate = enforced_gate();
    let caller = CallerContext::unix();
    assert!(gate.check("dag.event.append", &caller).is_err());
}

#[test]
fn btsp_authenticated_allows_all_public_methods() {
    let gate = enforced_gate();
    let caller = CallerContext::btsp_authenticated();
    assert!(gate.check("health", &caller).is_ok());
    assert!(gate.check("capability.list", &caller).is_ok());
}
