// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2024–2026 ecoPrimals Project

//! Pre-dispatch capability gate for JSON-RPC methods (JH-0 + JH-1 prep).
//!
//! Every incoming RPC call passes through [`MethodGate::check`] *before*
//! reaching the dispatch table. The gate classifies methods into
//! [`MethodAccessLevel::Public`] (allowed without any token — health probes,
//! identity, capability advertisement, auth introspection) and
//! [`MethodAccessLevel::Protected`] (require a valid capability token once
//! enforcement is activated).
//!
//! Two enforcement modes control behavior:
//! - **Permissive** (default): protected methods are logged but allowed,
//!   preserving backward compatibility during ecosystem rollout.
//! - **Enforced**: protected methods without a valid token are rejected
//!   with `PERMISSION_DENIED` (-32001).
//!
//! ## Token verification (JH-1 ready)
//!
//! The [`TokenVerifier`] trait abstracts token validation. The default
//! [`NoopVerifier`] accepts any token (presence-only check), while
//! [`BearDogVerifier`] validates `BearDog` ionic tokens via IPC once key
//! distribution is available (JH-11).
//!
//! ## Bearer token extraction
//!
//! Per the ecoPrimals convention, callers pass `_bearer_token` in the
//! JSON-RPC `params` object. [`extract_bearer_token`] pulls this field
//! and returns a [`CallerContext`] enriched with the token. The field is
//! stripped from `params` before method dispatch.
//!
//! Follows the ecosystem-standard pattern from primalSpring v0.9.25
//! `wateringHole/METHOD_GATE_STANDARD.md`.

use serde_json::Value;

// ============================================================================
// METHOD CLASSIFICATION
// ============================================================================

/// Access level for a JSON-RPC method.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MethodAccessLevel {
    /// Health probes, identity, capability advertisement, auth introspection.
    Public,
    /// Requires a valid capability token when enforcement is active.
    Protected,
}

/// Methods whose prefix marks them as public.
const PUBLIC_METHOD_PREFIXES: &[&str] = &["health."];

/// Exact method names that are always public.
const PUBLIC_METHODS: &[&str] = &[
    "ping",
    "health",
    "status",
    "check",
    "identity.get",
    "capabilities.list",
    "capability.list",
    "primal.capabilities",
    "lifecycle.status",
    "auth.check",
    "auth.mode",
    "auth.peer_info",
    "tools.list",
    "mcp.tools.list",
];

/// Classify a normalized method string into its access level.
#[must_use]
pub fn classify_method(method: &str) -> MethodAccessLevel {
    if PUBLIC_METHODS.contains(&method) {
        return MethodAccessLevel::Public;
    }
    for prefix in PUBLIC_METHOD_PREFIXES {
        if method.starts_with(prefix) {
            return MethodAccessLevel::Public;
        }
    }
    MethodAccessLevel::Protected
}

// ============================================================================
// SCOPE MATCHING
// ============================================================================

/// Check whether a set of scope patterns permits a given method.
///
/// Scope patterns follow the ecosystem convention:
/// - `"*"` — wildcard, permits everything
/// - `"domain.*"` — permits any method in that domain
/// - `"exact.method"` — permits only an exact match
#[must_use]
pub fn scope_permits_method(scopes: &[String], method: &str) -> bool {
    for scope in scopes {
        if scope == "*" {
            return true;
        }
        if let Some(prefix) = scope.strip_suffix(".*")
            && method.starts_with(prefix)
            && method.as_bytes().get(prefix.len()) == Some(&b'.')
        {
            return true;
        }
        if scope == method {
            return true;
        }
    }
    false
}

// ============================================================================
// TOKEN VERIFIER TRAIT
// ============================================================================

/// Verified token claims returned by a [`TokenVerifier`].
#[derive(Debug, Clone)]
pub struct VerifiedClaims {
    /// Subject (DID or identifier) of the token holder.
    pub subject: String,
    /// Scope patterns granted by this token.
    pub scopes: Vec<String>,
    /// Seconds until the token expires (`None` if unknown or no expiry).
    pub expires_in: Option<u64>,
}

/// Abstraction over token verification so tests can use [`NoopVerifier`]
/// and production uses [`BearDogVerifier`] (or any future provider).
pub trait TokenVerifier: Send + Sync + std::fmt::Debug {
    /// Verify a bearer token string and return the embedded claims.
    ///
    /// Returns `None` if the token is invalid, expired, or unverifiable.
    fn verify(&self, token: &str) -> Option<VerifiedClaims>;
}

/// Accepts any non-empty token as valid with wildcard scope.
///
/// Used as the default verifier until `BearDog` key distribution (JH-11)
/// is available. Equivalent to the "presence-only" check from JH-0.
#[derive(Debug)]
pub struct NoopVerifier;

impl TokenVerifier for NoopVerifier {
    fn verify(&self, token: &str) -> Option<VerifiedClaims> {
        if token.is_empty() {
            return None;
        }
        Some(VerifiedClaims {
            subject: "unknown".to_owned(),
            scopes: vec!["*".to_owned()],
            expires_in: None,
        })
    }
}

/// Verifies `BearDog` ionic tokens via IPC (`auth.verify_ionic`).
///
/// Currently a placeholder that delegates to [`NoopVerifier`] semantics.
/// When `BearDog` ships key distribution (JH-11), this will perform
/// Ed25519 signature verification and scope extraction locally or via
/// a single `auth.verify_ionic` IPC call.
#[derive(Debug)]
pub struct BearDogVerifier;

impl TokenVerifier for BearDogVerifier {
    fn verify(&self, token: &str) -> Option<VerifiedClaims> {
        if token.is_empty() {
            return None;
        }
        // Presence-only until JH-11 key distribution ships (see BearDogVerifier doc).
        Some(VerifiedClaims {
            subject: "unverified".to_owned(),
            scopes: vec!["*".to_owned()],
            expires_in: None,
        })
    }
}

// ============================================================================
// BEARER TOKEN EXTRACTION
// ============================================================================

/// Extract `_bearer_token` from a JSON-RPC params object, returning
/// the token string and mutating `params` to strip the field.
///
/// Per the ecoPrimals convention, callers pass the ionic token as
/// `"_bearer_token"` inside the `params` JSON object.
pub fn extract_bearer_token(params: &mut Value) -> Option<String> {
    params
        .as_object_mut()
        .and_then(|obj| obj.remove("_bearer_token"))
        .and_then(|v| v.as_str().map(str::to_owned))
}

// ============================================================================
// CONNECTION ORIGIN + CALLER CONTEXT
// ============================================================================

/// How the caller connected.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConnectionOrigin {
    /// Local Unix domain socket.
    Unix,
    /// TCP loopback (127.0.0.1 / `::1`).
    Loopback,
    /// Remote TCP connection.
    Remote,
}

impl ConnectionOrigin {
    /// Human-readable label for `auth.peer_info` responses.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Unix => "Unix",
            Self::Loopback => "Loopback",
            Self::Remote => "Remote",
        }
    }
}

/// Identity and authorization context for an incoming RPC call.
///
/// Built per-connection with origin info, then enriched per-request
/// with `_bearer_token` from params via [`extract_bearer_token`].
#[derive(Debug, Clone)]
pub struct CallerContext {
    /// Optional bearer / capability token sent in the request.
    pub bearer_token: Option<String>,
    /// Verified claims from the bearer token (populated after verification).
    pub verified_claims: Option<VerifiedClaims>,
    /// Where the connection came from.
    pub origin: ConnectionOrigin,
}

impl CallerContext {
    /// Default caller context for a Unix domain socket connection.
    #[must_use]
    pub const fn unix() -> Self {
        Self {
            bearer_token: None,
            verified_claims: None,
            origin: ConnectionOrigin::Unix,
        }
    }

    /// Default caller context for a TCP loopback connection.
    #[must_use]
    pub const fn loopback() -> Self {
        Self {
            bearer_token: None,
            verified_claims: None,
            origin: ConnectionOrigin::Loopback,
        }
    }

    /// Create a new context with a bearer token and connection origin.
    #[must_use]
    pub const fn with_bearer_token(token: Option<String>, origin: ConnectionOrigin) -> Self {
        Self {
            bearer_token: token,
            verified_claims: None,
            origin,
        }
    }

    /// Run token verification and populate `verified_claims`.
    pub fn verify_token(&mut self, verifier: &dyn TokenVerifier) {
        self.verified_claims = self.bearer_token.as_deref().and_then(|t| verifier.verify(t));
    }

    /// Whether the token has been verified (not just present).
    #[must_use]
    pub const fn is_verified(&self) -> bool {
        self.verified_claims.is_some()
    }
}

// ============================================================================
// ENFORCEMENT MODE
// ============================================================================

/// Enforcement mode for the method gate.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EnforcementMode {
    /// Log violations but allow all calls (backward-compatible default).
    Permissive,
    /// Reject unauthenticated calls to protected methods.
    Enforced,
}

impl EnforcementMode {
    /// Resolve from `RHIZOCRYPT_AUTH_MODE` env var.
    /// Defaults to `Permissive` if unset or unrecognized.
    #[must_use]
    pub fn from_env() -> Self {
        rhizo_crypt_core::SafeEnv::get_optional("RHIZOCRYPT_AUTH_MODE").map_or(
            Self::Permissive,
            |v| match v.to_lowercase().as_str() {
                "enforced" | "enforce" | "strict" => Self::Enforced,
                _ => Self::Permissive,
            },
        )
    }

    /// Human-readable label for diagnostics and `auth.mode` responses.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Permissive => "permissive",
            Self::Enforced => "enforced",
        }
    }
}

// ============================================================================
// METHOD GATE
// ============================================================================

/// Pre-dispatch gate that checks caller authorization before method execution.
pub struct MethodGate {
    mode: EnforcementMode,
    verifier: Box<dyn TokenVerifier>,
}

impl std::fmt::Debug for MethodGate {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MethodGate").field("mode", &self.mode).finish_non_exhaustive()
    }
}

impl MethodGate {
    /// Create a gate with the given enforcement mode and token verifier.
    #[must_use]
    pub fn new(mode: EnforcementMode, verifier: Box<dyn TokenVerifier>) -> Self {
        Self {
            mode,
            verifier,
        }
    }

    /// Create a gate from the environment (`RHIZOCRYPT_AUTH_MODE`).
    ///
    /// Uses [`BearDogVerifier`] as the default verifier (falls back to
    /// presence-only until JH-11 key distribution is available).
    #[must_use]
    pub fn from_env() -> Self {
        Self::new(EnforcementMode::from_env(), Box::new(BearDogVerifier))
    }

    /// Create a gate with a specific verifier (useful for testing).
    #[cfg(test)]
    #[must_use]
    pub fn with_noop(mode: EnforcementMode) -> Self {
        Self::new(mode, Box::new(NoopVerifier))
    }

    /// Current enforcement mode.
    #[must_use]
    pub const fn mode(&self) -> EnforcementMode {
        self.mode
    }

    /// Access the token verifier.
    #[must_use]
    pub fn verifier(&self) -> &dyn TokenVerifier {
        &*self.verifier
    }

    /// Pre-dispatch authorization check.
    ///
    /// Returns `Ok(())` if the call should proceed. When a bearer token
    /// is present, it is verified and scope-checked against the method.
    ///
    /// # Errors
    ///
    /// Returns [`GateRejection`] when a protected method is called without
    /// a valid capability token and the gate is in `Enforced` mode, or
    /// when the token's scope does not cover the requested method.
    pub fn check(&self, method: &str, caller: &CallerContext) -> Result<(), GateRejection> {
        let level = classify_method(method);

        if level == MethodAccessLevel::Public {
            return Ok(());
        }

        if let Some(ref claims) = caller.verified_claims {
            if scope_permits_method(&claims.scopes, method) {
                return Ok(());
            }
            tracing::warn!(
                method,
                subject = %claims.subject,
                "method gate: token scope does not cover method"
            );
            return Err(GateRejection {
                method: method.to_owned(),
            });
        }

        let has_token = caller.bearer_token.is_some();

        if has_token {
            tracing::warn!(
                method,
                origin = caller.origin.as_str(),
                "method gate: token present but verification failed"
            );
            return Err(GateRejection {
                method: method.to_owned(),
            });
        }

        match self.mode {
            EnforcementMode::Permissive => {
                tracing::warn!(
                    method,
                    origin = caller.origin.as_str(),
                    "method gate: unauthenticated call to protected method (permissive — allowing)"
                );
                Ok(())
            }
            EnforcementMode::Enforced => {
                tracing::warn!(
                    method,
                    origin = caller.origin.as_str(),
                    "method gate: REJECTED unauthenticated call to protected method"
                );
                Err(GateRejection {
                    method: method.to_owned(),
                })
            }
        }
    }

    /// Build the enriched `auth.check` response per primalSpring JH-1 spec.
    #[must_use]
    pub fn auth_check_response(&self, caller: &CallerContext) -> Value {
        let authenticated = caller.bearer_token.is_some();
        let verified = caller.is_verified();

        let mut resp = serde_json::json!({
            "authenticated": authenticated,
            "verified": verified,
            "enforcement": self.mode.as_str(),
        });

        if let Some(ref claims) = caller.verified_claims {
            resp["scopes"] = serde_json::json!(claims.scopes);
            resp["subject"] = serde_json::json!(claims.subject);
            if let Some(exp) = claims.expires_in {
                resp["expires_in"] = serde_json::json!(exp);
            }
        }

        resp
    }

    /// Build the `auth.mode` response.
    #[must_use]
    pub fn auth_mode_response(&self) -> Value {
        serde_json::json!({
            "mode": self.mode.as_str(),
        })
    }

    /// Build the `auth.peer_info` response.
    #[must_use]
    pub fn auth_peer_info_response(&self, caller: &CallerContext) -> Value {
        serde_json::json!({
            "origin": caller.origin.as_str(),
            "has_token": caller.bearer_token.is_some(),
        })
    }
}

/// A gate rejection — method was protected and caller was not authorized.
#[derive(Debug)]
pub struct GateRejection {
    /// The method that was rejected.
    pub method: String,
}

#[cfg(test)]
#[expect(clippy::unwrap_used, reason = "test code")]
mod tests {
    use super::*;

    fn test_gate() -> MethodGate {
        MethodGate::with_noop(EnforcementMode::Permissive)
    }

    fn enforced_gate() -> MethodGate {
        MethodGate::with_noop(EnforcementMode::Enforced)
    }

    fn verified_caller(token: &str) -> CallerContext {
        let verifier = NoopVerifier;
        let mut ctx =
            CallerContext::with_bearer_token(Some(token.to_owned()), ConnectionOrigin::Unix);
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
    fn beardog_verifier_accepts_nonempty() {
        let v = BearDogVerifier;
        let claims = v.verify("ionic-tok").unwrap();
        assert_eq!(claims.subject, "unverified");
    }

    #[test]
    fn beardog_verifier_rejects_empty() {
        let v = BearDogVerifier;
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
        let mut ctx =
            CallerContext::with_bearer_token(Some("tok".to_owned()), ConnectionOrigin::Unix);
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
}
