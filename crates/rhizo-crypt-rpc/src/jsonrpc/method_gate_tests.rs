// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2024–2026 ecoPrimals Project

//! Tests for the JSON-RPC method gate (JH-0/JH-1).

#![expect(clippy::unwrap_used, reason = "test code")]

use super::*;
use std::sync::Arc;

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

async fn unused_tcp_port() -> u16 {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    listener.local_addr().unwrap().port()
}

async fn spawn_verify_ionic_mock_server(
    result: serde_json::Value,
    max_requests: u32,
) -> (std::net::SocketAddr, Arc<std::sync::atomic::AtomicU32>) {
    use std::sync::atomic::{AtomicU32, Ordering};
    use tokio::io::{AsyncBufReadExt, AsyncWriteExt};

    let hits = Arc::new(AtomicU32::new(0));
    let hits_bg = Arc::clone(&hits);
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();

    tokio::spawn(async move {
        for _ in 0..max_requests {
            let Ok((stream, _)) = listener.accept().await else {
                break;
            };
            hits_bg.fetch_add(1, Ordering::SeqCst);
            let (reader, mut writer) = tokio::io::split(stream);
            let mut buf_reader = tokio::io::BufReader::new(reader);
            let mut line = String::new();
            let _ = buf_reader.read_line(&mut line).await;
            let response = serde_json::json!({
                "jsonrpc": "2.0",
                "id": 1,
                "result": result,
            });
            let payload = format!("{response}\n");
            let _ = writer.write_all(payload.as_bytes()).await;
            let _ = writer.flush().await;
        }
    });

    (addr, hits)
}

async fn spawn_verify_ionic_raw_response_server(response_line: &str) -> std::net::SocketAddr {
    use tokio::io::{AsyncBufReadExt, AsyncWriteExt};

    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let response = format!("{response_line}\n");

    tokio::spawn(async move {
        if let Ok((stream, _)) = listener.accept().await {
            let (reader, mut writer) = tokio::io::split(stream);
            let mut buf_reader = tokio::io::BufReader::new(reader);
            let mut line = String::new();
            let _ = buf_reader.read_line(&mut line).await;
            let _ = writer.write_all(response.as_bytes()).await;
            let _ = writer.flush().await;
        }
    });

    addr
}

async fn register_signing_provider(registry: &DiscoveryRegistry, addr: std::net::SocketAddr) {
    use rhizo_crypt_core::discovery::{Capability, ServiceEndpoint};
    use rhizo_crypt_core::transport::TransportEndpoint;

    registry
        .register_endpoint(ServiceEndpoint::new(
            "mock-signer",
            TransportEndpoint::tcp("127.0.0.1", addr.port()),
            vec![Capability::Signing],
        ))
        .await;
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

// ── Capability verifier response parsing ─────────────────────────

#[test]
fn parse_verify_ionic_success_response() {
    let line = serde_json::json!({
        "jsonrpc": "2.0",
        "id": 1,
        "result": {
            "valid": true,
            "scopes": ["dag.*", "crypto.*"],
            "claims": {
                "sub": "alice",
                "scope": ["dag.*", "crypto.*"],
                "exp": 4_000_000_000u64
            }
        }
    });
    let claims = parse_verify_ionic_response(&line.to_string()).unwrap();
    assert_eq!(claims.subject, "alice");
    assert_eq!(claims.scopes, vec!["dag.*", "crypto.*"]);
}

#[test]
fn parse_verify_ionic_rejects_invalid_token() {
    let line = serde_json::json!({
        "jsonrpc": "2.0",
        "id": 1,
        "result": { "valid": false, "scopes": [] }
    });
    assert!(parse_verify_ionic_response(&line.to_string()).is_err());
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn capability_verifier_falls_back_without_provider() {
    use rhizo_crypt_core::discovery::DiscoveryRegistry;
    use std::sync::Arc;

    let registry = Arc::new(DiscoveryRegistry::new("test-gate"));
    let verifier = CapabilityVerifier::new(registry);
    let claims = verifier.verify_async("some-token").await.unwrap();
    assert_eq!(claims.subject, "unverified");
    assert_eq!(claims.scopes, vec!["*"]);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn capability_verifier_rejects_empty_token() {
    use rhizo_crypt_core::discovery::DiscoveryRegistry;
    use std::sync::Arc;

    let registry = Arc::new(DiscoveryRegistry::new("test-gate"));
    let verifier = CapabilityVerifier::new(registry);
    assert!(verifier.verify_async("").await.is_none());
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

// ── Parse verify_ionic error branches ────────────────────────────

#[test]
fn parse_verify_ionic_rejects_invalid_json() {
    let result = parse_verify_ionic_response("not-json");
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("not valid JSON"));
}

#[test]
fn parse_verify_ionic_rejects_error_response() {
    let line = serde_json::json!({
        "jsonrpc": "2.0",
        "id": 1,
        "error": { "code": -32600, "message": "bad request" }
    });
    let err = parse_verify_ionic_response(&line.to_string()).unwrap_err();
    assert!(err.contains("error response"), "got: {err}");
}

#[test]
fn parse_verify_ionic_rejects_missing_result() {
    let line = serde_json::json!({ "jsonrpc": "2.0", "id": 1 });
    let err = parse_verify_ionic_response(&line.to_string()).unwrap_err();
    assert!(err.contains("missing result"), "got: {err}");
}

#[test]
fn parse_verify_ionic_rejects_missing_claims() {
    let line = serde_json::json!({
        "jsonrpc": "2.0", "id": 1,
        "result": { "valid": true, "scopes": ["*"] }
    });
    let err = parse_verify_ionic_response(&line.to_string()).unwrap_err();
    assert!(err.contains("missing claims"), "got: {err}");
}

#[test]
fn parse_verify_ionic_rejects_missing_subject() {
    let line = serde_json::json!({
        "jsonrpc": "2.0", "id": 1,
        "result": {
            "valid": true,
            "scopes": ["*"],
            "claims": { "scope": ["*"] }
        }
    });
    let err = parse_verify_ionic_response(&line.to_string()).unwrap_err();
    assert!(err.contains("missing subject"), "got: {err}");
}

#[test]
fn parse_verify_ionic_rejects_missing_scopes() {
    let line = serde_json::json!({
        "jsonrpc": "2.0", "id": 1,
        "result": {
            "valid": true,
            "claims": { "sub": "alice" }
        }
    });
    let err = parse_verify_ionic_response(&line.to_string()).unwrap_err();
    assert!(err.contains("missing scopes"), "got: {err}");
}

// ── extract_scope_list alternate keys ────────────────────────────

#[test]
fn extract_scope_list_from_result_scope_key() {
    let result = serde_json::json!({
        "valid": true,
        "scope": ["dag.*"],
        "claims": { "sub": "bob" }
    });
    let scopes = extract_scope_list(&result).unwrap();
    assert_eq!(scopes, vec!["dag.*"]);
}

#[test]
fn extract_scope_list_from_claims_scope_key() {
    let result = serde_json::json!({
        "valid": true,
        "claims": {
            "sub": "carol",
            "scope": ["crypto.*"]
        }
    });
    let scopes = extract_scope_list(&result).unwrap();
    assert_eq!(scopes, vec!["crypto.*"]);
}

#[test]
fn extract_scope_list_from_claims_scopes_key() {
    let result = serde_json::json!({
        "valid": true,
        "claims": {
            "sub": "dave",
            "scopes": ["mesh.*", "dag.*"]
        }
    });
    let scopes = extract_scope_list(&result).unwrap();
    assert_eq!(scopes, vec!["mesh.*", "dag.*"]);
}

#[test]
fn extract_scope_list_empty_arrays_return_none() {
    let result = serde_json::json!({
        "valid": true,
        "scopes": [],
        "scope": [],
        "claims": { "sub": "eve", "scopes": [], "scope": [] }
    });
    assert!(extract_scope_list(&result).is_none());
}

#[test]
fn extract_scope_list_skips_non_string_entries() {
    let result = serde_json::json!({
        "valid": true,
        "scopes": ["dag.*", 42, null, "crypto.*"]
    });
    let scopes = extract_scope_list(&result).unwrap();
    assert_eq!(scopes, vec!["dag.*", "crypto.*"]);
}

// ── expires_in_from_claims ───────────────────────────────────────

#[test]
fn expires_in_from_claims_future_exp() {
    let future_exp =
        std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs()
            + 3600;
    let claims = serde_json::json!({ "sub": "x", "exp": future_exp });
    let remaining = expires_in_from_claims(&claims);
    assert!(remaining.is_some());
    assert!(remaining.unwrap() > 3500 && remaining.unwrap() <= 3600);
}

#[test]
fn expires_in_from_claims_past_exp_returns_none() {
    let claims = serde_json::json!({ "sub": "x", "exp": 100 });
    assert!(expires_in_from_claims(&claims).is_none());
}

#[test]
fn expires_in_from_claims_no_exp() {
    let claims = serde_json::json!({ "sub": "x" });
    assert!(expires_in_from_claims(&claims).is_none());
}

#[test]
fn expires_in_from_claims_non_numeric_exp() {
    let claims = serde_json::json!({ "sub": "x", "exp": "not-a-number" });
    assert!(expires_in_from_claims(&claims).is_none());
}

// ── CallerContext constructors & async verify ────────────────────

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

// ── MethodGate constructors & accessors ──────────────────────────

#[test]
fn method_gate_mode_accessor() {
    let gate = test_gate();
    assert_eq!(gate.mode(), EnforcementMode::Permissive);
    let gate = enforced_gate();
    assert_eq!(gate.mode(), EnforcementMode::Enforced);
}

#[test]
fn method_gate_verifier_accessor() {
    let gate = test_gate();
    let _verifier = gate.verifier();
    assert!(gate.verifier().verify("tok").is_some());
}

#[test]
fn method_gate_debug_format() {
    let gate = test_gate();
    let debug = format!("{gate:?}");
    assert!(debug.contains("MethodGate"));
    assert!(debug.contains("Permissive"));
}

#[test]
fn method_gate_from_env_defaults_to_permissive() {
    let gate = MethodGate::from_env();
    assert_eq!(gate.mode(), EnforcementMode::Permissive);
}

#[test]
fn method_gate_from_env_with_registry() {
    use rhizo_crypt_core::discovery::DiscoveryRegistry;

    let registry = Arc::new(DiscoveryRegistry::new("test-gate"));
    let gate = MethodGate::from_env_with_registry(Some(registry));
    assert_eq!(gate.mode(), EnforcementMode::Permissive);
}

#[test]
fn method_gate_with_discovery() {
    use rhizo_crypt_core::discovery::DiscoveryRegistry;

    let registry = Arc::new(DiscoveryRegistry::new("test-gate"));
    let gate = MethodGate::with_discovery(EnforcementMode::Enforced, registry);
    assert_eq!(gate.mode(), EnforcementMode::Enforced);
}

// ── Gate check: permissive with unverified token on protected ────

#[test]
fn gate_permissive_rejects_failed_token() {
    let gate = test_gate();
    let caller = CallerContext::with_bearer_token(Some("bad".to_owned()), ConnectionOrigin::Remote);
    let result = gate.check("dag.session.create", &caller);
    assert!(result.is_err());
}

// ── CapabilityVerifier: sync verify path ─────────────────────────

#[test]
fn capability_verifier_sync_verify_empty_token() {
    use rhizo_crypt_core::discovery::DiscoveryRegistry;

    let registry = Arc::new(DiscoveryRegistry::new("test"));
    let verifier = CapabilityVerifier::new(registry);
    assert!(verifier.verify("").is_none());
}

// ── CapabilityVerifier: provider discovery + verify paths ────────

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_capability_verifier_verify_with_provider_success() {
    use rhizo_crypt_core::discovery::DiscoveryRegistry;

    let result = serde_json::json!({
        "valid": true,
        "scopes": ["dag.*"],
        "claims": { "sub": "provider-alice", "scope": ["dag.*"] }
    });
    let (addr, hits) = spawn_verify_ionic_mock_server(result, 1).await;

    let registry = Arc::new(DiscoveryRegistry::new("test-gate"));
    register_signing_provider(&registry, addr).await;
    let verifier = CapabilityVerifier::new(registry);

    let claims = verifier.verify_async("ionic-token").await.unwrap();
    assert_eq!(claims.subject, "provider-alice");
    assert_eq!(claims.scopes, vec!["dag.*"]);
    assert_eq!(hits.load(std::sync::atomic::Ordering::SeqCst), 1);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_capability_verifier_transport_error_returns_none() {
    use rhizo_crypt_core::discovery::DiscoveryRegistry;

    let dead_port = unused_tcp_port().await;
    let registry = Arc::new(DiscoveryRegistry::new("test-gate"));
    register_signing_provider(&registry, format!("127.0.0.1:{dead_port}").parse().unwrap()).await;
    let verifier = CapabilityVerifier::new(registry);

    assert!(verifier.verify_async("ionic-token").await.is_none());
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_capability_verifier_invalid_provider_response_returns_none() {
    use rhizo_crypt_core::discovery::DiscoveryRegistry;

    let addr = spawn_verify_ionic_raw_response_server("not-valid-json").await;
    let registry = Arc::new(DiscoveryRegistry::new("test-gate"));
    register_signing_provider(&registry, addr).await;
    let verifier = CapabilityVerifier::new(registry);

    assert!(verifier.verify_async("ionic-token").await.is_none());
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_capability_verifier_discovery_failed_falls_back_to_presence() {
    use rhizo_crypt_core::discovery::DiscoveryRegistry;
    use std::net::SocketAddr;

    let registry = Arc::new(DiscoveryRegistry::new("test-gate"));
    registry.set_discovery_source("127.0.0.1:1".parse::<SocketAddr>().unwrap()).await;
    let verifier = CapabilityVerifier::new(registry);

    let claims = verifier.verify_async("ionic-token").await.unwrap();
    assert_eq!(claims.subject, "unverified");
    assert_eq!(claims.scopes, vec!["*"]);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_capability_verifier_cached_endpoint_used_within_ttl() {
    use rhizo_crypt_core::discovery::{Capability, DiscoveryRegistry, ServiceEndpoint};
    use rhizo_crypt_core::transport::TransportEndpoint;

    let result = serde_json::json!({
        "valid": true,
        "scopes": ["*"],
        "claims": { "sub": "cached-subject", "scope": ["*"] }
    });
    let (addr, hits) = spawn_verify_ionic_mock_server(result, 2).await;

    let registry = Arc::new(DiscoveryRegistry::new("test-gate"));
    register_signing_provider(&registry, addr).await;
    let verifier = CapabilityVerifier::new(Arc::clone(&registry));

    let first = verifier.verify_async("token-a").await.unwrap();
    assert_eq!(first.subject, "cached-subject");

    let stale_port = unused_tcp_port().await;
    registry
        .register_endpoint(ServiceEndpoint::new(
            "stale-signer",
            TransportEndpoint::tcp("127.0.0.1", stale_port),
            vec![Capability::Signing],
        ))
        .await;

    let second = verifier.verify_async("token-b").await.unwrap();
    assert_eq!(second.subject, "cached-subject");
    assert_eq!(hits.load(std::sync::atomic::Ordering::SeqCst), 2);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_capability_verifier_sync_verify_with_runtime_uses_provider() {
    use rhizo_crypt_core::discovery::DiscoveryRegistry;

    let result = serde_json::json!({
        "valid": true,
        "scopes": ["crypto.*"],
        "claims": { "sub": "sync-subject", "scope": ["crypto.*"] }
    });
    let (addr, _) = spawn_verify_ionic_mock_server(result, 1).await;

    let registry = Arc::new(DiscoveryRegistry::new("test-gate"));
    register_signing_provider(&registry, addr).await;
    let verifier = CapabilityVerifier::new(registry);

    let claims =
        tokio::task::spawn_blocking(move || verifier.verify("ionic-token")).await.unwrap().unwrap();
    assert_eq!(claims.subject, "sync-subject");
    assert_eq!(claims.scopes, vec!["crypto.*"]);
}

#[test]
fn test_capability_verifier_sync_without_runtime_falls_back_to_presence() {
    use rhizo_crypt_core::discovery::DiscoveryRegistry;

    let registry = Arc::new(DiscoveryRegistry::new("test-gate"));
    let verifier = CapabilityVerifier::new(registry);

    let claims = verifier.verify("ionic-token").unwrap();
    assert_eq!(claims.subject, "unverified");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_caller_context_verify_token_async_with_capability_verifier() {
    use rhizo_crypt_core::discovery::DiscoveryRegistry;

    let result = serde_json::json!({
        "valid": true,
        "scopes": ["dag.*"],
        "claims": { "sub": "ctx-subject", "scope": ["dag.*"] }
    });
    let (addr, _) = spawn_verify_ionic_mock_server(result, 1).await;

    let registry = Arc::new(DiscoveryRegistry::new("test-gate"));
    register_signing_provider(&registry, addr).await;
    let verifier = CapabilityVerifier::new(registry);

    let mut ctx = CallerContext::with_bearer_token(
        Some("ionic-token".to_owned()),
        ConnectionOrigin::Loopback,
    );
    ctx.verify_token_async(&verifier).await;

    assert!(ctx.is_verified());
    let claims = ctx.verified_claims.as_ref().unwrap();
    assert_eq!(claims.subject, "ctx-subject");
    assert_eq!(claims.scopes, vec!["dag.*"]);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_gate_with_discovery_verifier_allows_scoped_method() {
    use rhizo_crypt_core::discovery::DiscoveryRegistry;

    let result = serde_json::json!({
        "valid": true,
        "scopes": ["dag.*"],
        "claims": { "sub": "gate-subject", "scope": ["dag.*"] }
    });
    let (addr, _) = spawn_verify_ionic_mock_server(result, 1).await;

    let registry = Arc::new(DiscoveryRegistry::new("test-gate"));
    register_signing_provider(&registry, addr).await;
    let gate = MethodGate::with_discovery(EnforcementMode::Enforced, registry);

    let mut caller =
        CallerContext::with_bearer_token(Some("ionic-token".to_owned()), ConnectionOrigin::Remote);
    caller.verify_token_async(gate.verifier()).await;

    assert!(gate.check("dag.session.create", &caller).is_ok());
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_gate_with_discovery_verifier_rejects_failed_verification() {
    use rhizo_crypt_core::discovery::DiscoveryRegistry;

    let dead_port = unused_tcp_port().await;
    let registry = Arc::new(DiscoveryRegistry::new("test-gate"));
    register_signing_provider(&registry, format!("127.0.0.1:{dead_port}").parse().unwrap()).await;
    let gate = MethodGate::with_discovery(EnforcementMode::Enforced, registry);

    let mut caller =
        CallerContext::with_bearer_token(Some("ionic-token".to_owned()), ConnectionOrigin::Remote);
    caller.verify_token_async(gate.verifier()).await;

    assert!(!caller.is_verified());
    assert!(gate.check("dag.session.create", &caller).is_err());
}

// ── EnforcementMode::from_env variants ───────────────────────────

#[test]
fn enforcement_mode_from_env_enforce_variant() {
    temp_env::with_var("RHIZOCRYPT_AUTH_MODE", Some("enforce"), || {
        assert_eq!(EnforcementMode::from_env(), EnforcementMode::Enforced);
    });
}

#[test]
fn enforcement_mode_from_env_strict_variant() {
    temp_env::with_var("RHIZOCRYPT_AUTH_MODE", Some("strict"), || {
        assert_eq!(EnforcementMode::from_env(), EnforcementMode::Enforced);
    });
}

#[test]
fn enforcement_mode_from_env_enforced_variant() {
    temp_env::with_var("RHIZOCRYPT_AUTH_MODE", Some("Enforced"), || {
        assert_eq!(EnforcementMode::from_env(), EnforcementMode::Enforced);
    });
}

#[test]
fn enforcement_mode_from_env_unrecognized_defaults_permissive() {
    temp_env::with_var("RHIZOCRYPT_AUTH_MODE", Some("banana"), || {
        assert_eq!(EnforcementMode::from_env(), EnforcementMode::Permissive);
    });
}

#[test]
fn enforcement_mode_from_env_unset_defaults_permissive() {
    temp_env::with_var_unset("RHIZOCRYPT_AUTH_MODE", || {
        assert_eq!(EnforcementMode::from_env(), EnforcementMode::Permissive);
    });
}
