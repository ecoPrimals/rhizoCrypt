// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2024–2026 ecoPrimals Project

//! Token verifier, `parse_verify_ionic` error branches, scope extraction,
//! and `expires_in` tests.

use super::*;

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
    let verifier = CapabilityVerifier::new(registry, true);
    let claims = verifier.verify_async("some-token").await.unwrap();
    assert_eq!(claims.subject, "unverified");
    assert_eq!(claims.scopes, vec!["*"]);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn capability_verifier_rejects_empty_token() {
    use rhizo_crypt_core::discovery::DiscoveryRegistry;
    use std::sync::Arc;

    let registry = Arc::new(DiscoveryRegistry::new("test-gate"));
    let verifier = CapabilityVerifier::new(registry, true);
    assert!(verifier.verify_async("").await.is_none());
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

// ── CapabilityVerifier: sync verify path ─────────────────────────

#[test]
fn capability_verifier_sync_verify_empty_token() {
    use rhizo_crypt_core::discovery::DiscoveryRegistry;
    use std::sync::Arc;

    let registry = Arc::new(DiscoveryRegistry::new("test"));
    let verifier = CapabilityVerifier::new(registry, true);
    assert!(verifier.verify("").is_none());
}
