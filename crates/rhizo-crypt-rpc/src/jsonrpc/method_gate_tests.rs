// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2024–2026 ecoPrimals Project

//! Tests for the JSON-RPC method gate (JH-0/JH-1).

#![expect(clippy::unwrap_used, reason = "test code")]

use super::*;

fn test_gate() -> MethodGate {
    MethodGate::with_noop(EnforcementMode::Permissive)
}

fn enforced_gate() -> MethodGate {
    MethodGate::with_noop(EnforcementMode::Enforced)
}

fn verified_caller(token: &str) -> CallerContext {
    let verifier = NoopVerifier;
    let mut ctx = CallerContext::with_bearer_token(Some(token.to_owned()), ConnectionOrigin::Unix);
    ctx.verify_token(&verifier);
    ctx
}

// ── Classification ───────────────────────────────────────────────

#[test]
fn classify_health_methods_as_public() {
    assert_eq!(classify_method("health.check"), MethodAccessLevel::Public);
    assert_eq!(classify_method("health.liveness"), MethodAccessLevel::Public);
    assert_eq!(classify_method("health.readiness"), MethodAccessLevel::Public);
    assert_eq!(classify_method("health.metrics"), MethodAccessLevel::Public);
}

#[test]
fn classify_introspection_methods_as_public() {
    assert_eq!(classify_method("identity.get"), MethodAccessLevel::Public);
    assert_eq!(classify_method("capabilities.list"), MethodAccessLevel::Public);
    assert_eq!(classify_method("capability.list"), MethodAccessLevel::Public);
    assert_eq!(classify_method("lifecycle.status"), MethodAccessLevel::Public);
    assert_eq!(classify_method("ping"), MethodAccessLevel::Public);
    assert_eq!(classify_method("tools.list"), MethodAccessLevel::Public);
}

#[test]
fn classify_auth_methods_as_public() {
    assert_eq!(classify_method("auth.check"), MethodAccessLevel::Public);
    assert_eq!(classify_method("auth.mode"), MethodAccessLevel::Public);
    assert_eq!(classify_method("auth.peer_info"), MethodAccessLevel::Public);
}

#[test]
fn classify_dag_methods_as_protected() {
    assert_eq!(classify_method("dag.session.create"), MethodAccessLevel::Protected);
    assert_eq!(classify_method("dag.event.append"), MethodAccessLevel::Protected);
    assert_eq!(classify_method("dag.vertex.get"), MethodAccessLevel::Protected);
    assert_eq!(classify_method("dag.merkle.root"), MethodAccessLevel::Protected);
    assert_eq!(classify_method("dag.slice.checkout"), MethodAccessLevel::Protected);
    assert_eq!(classify_method("dag.dehydration.trigger"), MethodAccessLevel::Protected);
}

#[test]
fn classify_tools_call_as_protected() {
    assert_eq!(classify_method("tools.call"), MethodAccessLevel::Protected);
    assert_eq!(classify_method("mcp.tools.call"), MethodAccessLevel::Protected);
}

#[test]
fn classify_unknown_as_protected() {
    assert_eq!(classify_method("unknown.method"), MethodAccessLevel::Protected);
}

// ── Scope matching ───────────────────────────────────────────────

#[test]
fn scope_wildcard_permits_everything() {
    let scopes = vec!["*".to_owned()];
    assert!(scope_permits_method(&scopes, "dag.session.create"));
    assert!(scope_permits_method(&scopes, "anything"));
}

#[test]
fn scope_domain_wildcard_permits_domain() {
    let scopes = vec!["dag.*".to_owned()];
    assert!(scope_permits_method(&scopes, "dag.session.create"));
    assert!(scope_permits_method(&scopes, "dag.event.append"));
    assert!(!scope_permits_method(&scopes, "crypto.sign"));
}

#[test]
fn scope_domain_wildcard_requires_dot_boundary() {
    let scopes = vec!["dag.*".to_owned()];
    assert!(!scope_permits_method(&scopes, "dagger.something"));
}

#[test]
fn scope_exact_match() {
    let scopes = vec!["dag.session.create".to_owned()];
    assert!(scope_permits_method(&scopes, "dag.session.create"));
    assert!(!scope_permits_method(&scopes, "dag.session.get"));
}

#[test]
fn scope_multiple_patterns() {
    let scopes = vec!["crypto.*".to_owned(), "health.*".to_owned()];
    assert!(scope_permits_method(&scopes, "crypto.sign"));
    assert!(scope_permits_method(&scopes, "health.check"));
    assert!(!scope_permits_method(&scopes, "dag.session.create"));
}

#[test]
fn scope_empty_permits_nothing() {
    let scopes: Vec<String> = vec![];
    assert!(!scope_permits_method(&scopes, "anything"));
}

// ── Token verifiers ──────────────────────────────────────────────

#[test]
fn noop_verifier_accepts_nonempty() {
    let v = NoopVerifier;
    let claims = v.verify("some-token").unwrap();
    assert_eq!(claims.subject, "unknown");
    assert_eq!(claims.scopes, vec!["*"]);
}

#[test]
fn noop_verifier_rejects_empty() {
    let v = NoopVerifier;
    assert!(v.verify("").is_none());
}

#[test]
fn presence_verifier_accepts_nonempty() {
    let v = PresenceVerifier;
    let claims = v.verify("ionic-tok").unwrap();
    assert_eq!(claims.subject, "unverified");
}

#[test]
fn presence_verifier_rejects_empty() {
    let v = PresenceVerifier;
    assert!(v.verify("").is_none());
}

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

// ── Enforcement mode ─────────────────────────────────────────────

#[test]
fn enforcement_mode_as_str() {
    assert_eq!(EnforcementMode::Permissive.as_str(), "permissive");
    assert_eq!(EnforcementMode::Enforced.as_str(), "enforced");
}

#[test]
fn connection_origin_as_str() {
    assert_eq!(ConnectionOrigin::Unix.as_str(), "Unix");
    assert_eq!(ConnectionOrigin::Loopback.as_str(), "Loopback");
    assert_eq!(ConnectionOrigin::Remote.as_str(), "Remote");
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
