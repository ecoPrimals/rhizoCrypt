// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2024–2026 ecoPrimals Project

//! Classification, scope matching, enforcement mode, and `MethodGate`
//! constructor tests.

use super::*;
use std::sync::Arc;

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
    assert_eq!(classify_method("dag.session.tree_hash"), MethodAccessLevel::Protected);
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
