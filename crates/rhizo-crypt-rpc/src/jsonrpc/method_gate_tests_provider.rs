// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2024–2026 ecoPrimals Project

//! Provider-backed verification, caching, async paths, and enforcement mode
//! tests for the JSON-RPC method gate.

#![expect(clippy::unwrap_used, reason = "test code")]

use super::*;
use std::sync::Arc;

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_capability_verifier_verify_with_provider_success() {
    use rhizo_crypt_core::discovery::DiscoveryRegistry;

    let result = serde_json::json!({
        "valid": true,
        "scopes": ["dag.*"],
        "claims": { "sub": "provider-alice", "scope": ["dag.*"] }
    });
    let (addr, hits) = super::tests::spawn_verify_ionic_mock_server(result, 1).await;

    let registry = Arc::new(DiscoveryRegistry::new("test-gate"));
    super::tests::register_signing_provider(&registry, addr).await;
    let verifier = CapabilityVerifier::new(registry, true);

    let claims = verifier.verify_async("ionic-token").await.unwrap();
    assert_eq!(claims.subject, "provider-alice");
    assert_eq!(claims.scopes, vec!["dag.*"]);
    assert_eq!(hits.load(std::sync::atomic::Ordering::SeqCst), 1);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_capability_verifier_transport_error_returns_none() {
    use rhizo_crypt_core::discovery::DiscoveryRegistry;

    let dead_port = super::tests::unused_tcp_port().await;
    let registry = Arc::new(DiscoveryRegistry::new("test-gate"));
    super::tests::register_signing_provider(
        &registry,
        format!("127.0.0.1:{dead_port}").parse().unwrap(),
    )
    .await;
    let verifier = CapabilityVerifier::new(registry, true);

    assert!(verifier.verify_async("ionic-token").await.is_none());
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_capability_verifier_invalid_provider_response_returns_none() {
    use rhizo_crypt_core::discovery::DiscoveryRegistry;

    let addr = super::tests::spawn_verify_ionic_raw_response_server("not-valid-json").await;
    let registry = Arc::new(DiscoveryRegistry::new("test-gate"));
    super::tests::register_signing_provider(&registry, addr).await;
    let verifier = CapabilityVerifier::new(registry, true);

    assert!(verifier.verify_async("ionic-token").await.is_none());
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_capability_verifier_discovery_failed_falls_back_to_presence() {
    use rhizo_crypt_core::discovery::DiscoveryRegistry;
    use std::net::SocketAddr;

    let registry = Arc::new(DiscoveryRegistry::new("test-gate"));
    registry.set_discovery_source("127.0.0.1:1".parse::<SocketAddr>().unwrap()).await;
    let verifier = CapabilityVerifier::new(registry, true);

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
    let (addr, hits) = super::tests::spawn_verify_ionic_mock_server(result, 2).await;

    let registry = Arc::new(DiscoveryRegistry::new("test-gate"));
    super::tests::register_signing_provider(&registry, addr).await;
    let verifier = CapabilityVerifier::new(Arc::clone(&registry), true);

    let first = verifier.verify_async("token-a").await.unwrap();
    assert_eq!(first.subject, "cached-subject");

    let stale_port = super::tests::unused_tcp_port().await;
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
    let (addr, _) = super::tests::spawn_verify_ionic_mock_server(result, 1).await;

    let registry = Arc::new(DiscoveryRegistry::new("test-gate"));
    super::tests::register_signing_provider(&registry, addr).await;
    let verifier = CapabilityVerifier::new(registry, true);

    let claims =
        tokio::task::spawn_blocking(move || verifier.verify("ionic-token")).await.unwrap().unwrap();
    assert_eq!(claims.subject, "sync-subject");
    assert_eq!(claims.scopes, vec!["crypto.*"]);
}

#[test]
fn test_capability_verifier_sync_without_runtime_falls_back_to_presence() {
    use rhizo_crypt_core::discovery::DiscoveryRegistry;

    let registry = Arc::new(DiscoveryRegistry::new("test-gate"));
    let verifier = CapabilityVerifier::new(registry, true);

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
    let (addr, _) = super::tests::spawn_verify_ionic_mock_server(result, 1).await;

    let registry = Arc::new(DiscoveryRegistry::new("test-gate"));
    super::tests::register_signing_provider(&registry, addr).await;
    let verifier = CapabilityVerifier::new(registry, true);

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
    let (addr, _) = super::tests::spawn_verify_ionic_mock_server(result, 1).await;

    let registry = Arc::new(DiscoveryRegistry::new("test-gate"));
    super::tests::register_signing_provider(&registry, addr).await;
    let gate = MethodGate::with_discovery(EnforcementMode::Enforced, registry);

    let mut caller =
        CallerContext::with_bearer_token(Some("ionic-token".to_owned()), ConnectionOrigin::Remote);
    caller.verify_token_async(gate.verifier()).await;

    assert!(gate.check("dag.session.create", &caller).is_ok());
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_gate_with_discovery_verifier_rejects_failed_verification() {
    use rhizo_crypt_core::discovery::DiscoveryRegistry;

    let dead_port = super::tests::unused_tcp_port().await;
    let registry = Arc::new(DiscoveryRegistry::new("test-gate"));
    super::tests::register_signing_provider(
        &registry,
        format!("127.0.0.1:{dead_port}").parse().unwrap(),
    )
    .await;
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

#[tokio::test]
async fn test_capability_verifier_fail_closed_no_provider_returns_none() {
    use rhizo_crypt_core::discovery::DiscoveryRegistry;

    let registry = Arc::new(DiscoveryRegistry::new("test-gate"));
    let verifier = CapabilityVerifier::new(registry, false);

    let result = verifier.verify_async("ionic-token").await;
    assert!(result.is_none());
}

#[tokio::test]
async fn test_capability_verifier_fail_open_no_provider_returns_presence() {
    use rhizo_crypt_core::discovery::DiscoveryRegistry;

    let registry = Arc::new(DiscoveryRegistry::new("test-gate"));
    let verifier = CapabilityVerifier::new(registry, true);

    let claims = verifier.verify_async("ionic-token").await.unwrap();
    assert_eq!(claims.subject, "unverified");
    assert_eq!(claims.scopes, vec!["*"]);
}

#[tokio::test]
async fn test_gate_enforced_with_discovery_rejects_when_no_provider() {
    use rhizo_crypt_core::discovery::DiscoveryRegistry;

    let registry = Arc::new(DiscoveryRegistry::new("test-gate"));
    let gate = MethodGate::with_discovery(EnforcementMode::Enforced, registry);

    let mut caller =
        CallerContext::with_bearer_token(Some("some-token".into()), ConnectionOrigin::Remote);
    caller.verify_token_async(gate.verifier()).await;
    assert!(!caller.is_verified());
    let result = gate.check("dag.append", &caller);
    assert!(result.is_err());
}

#[tokio::test]
async fn test_gate_permissive_with_discovery_allows_when_no_provider() {
    use rhizo_crypt_core::discovery::DiscoveryRegistry;

    let registry = Arc::new(DiscoveryRegistry::new("test-gate"));
    let gate = MethodGate::with_discovery(EnforcementMode::Permissive, registry);

    let mut caller =
        CallerContext::with_bearer_token(Some("some-token".into()), ConnectionOrigin::Remote);
    caller.verify_token_async(gate.verifier()).await;
    assert!(caller.is_verified());
    let result = gate.check("dag.append", &caller);
    assert!(result.is_ok());
}
