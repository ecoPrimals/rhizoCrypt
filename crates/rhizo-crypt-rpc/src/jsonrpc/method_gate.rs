// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2024–2026 ecoPrimals Project

//! Pre-dispatch capability gate for JSON-RPC methods (JH-0).
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
//! Follows the ecosystem-standard pattern from primalSpring v0.9.25
//! `wateringHole/METHOD_GATE_STANDARD.md`.

use serde_json::Value;

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
#[derive(Debug, Clone)]
pub struct CallerContext {
    /// Optional bearer / capability token sent in the request.
    pub bearer_token: Option<String>,
    /// Where the connection came from.
    pub origin: ConnectionOrigin,
}

impl CallerContext {
    /// Default caller context for a Unix domain socket connection.
    #[must_use]
    pub const fn unix() -> Self {
        Self {
            bearer_token: None,
            origin: ConnectionOrigin::Unix,
        }
    }

    /// Default caller context for a TCP loopback connection.
    #[must_use]
    pub const fn loopback() -> Self {
        Self {
            bearer_token: None,
            origin: ConnectionOrigin::Loopback,
        }
    }
}

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

/// Pre-dispatch gate that checks caller authorization before method execution.
#[derive(Debug)]
pub struct MethodGate {
    mode: EnforcementMode,
}

impl MethodGate {
    /// Create a gate with the given enforcement mode.
    #[must_use]
    pub const fn new(mode: EnforcementMode) -> Self {
        Self {
            mode,
        }
    }

    /// Create a gate from the environment (`RHIZOCRYPT_AUTH_MODE`).
    #[must_use]
    pub fn from_env() -> Self {
        Self::new(EnforcementMode::from_env())
    }

    /// Current enforcement mode.
    #[must_use]
    pub const fn mode(&self) -> EnforcementMode {
        self.mode
    }

    /// Pre-dispatch authorization check.
    ///
    /// Returns `Ok(())` if the call should proceed.
    ///
    /// # Errors
    ///
    /// Returns [`GateRejection`] when a protected method is called without
    /// a valid capability token and the gate is in `Enforced` mode.
    pub fn check(&self, method: &str, caller: &CallerContext) -> Result<(), GateRejection> {
        let level = classify_method(method);

        if level == MethodAccessLevel::Public {
            return Ok(());
        }

        let authorized = caller.bearer_token.is_some();

        if authorized {
            return Ok(());
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

    /// Build the `auth.check` response.
    #[must_use]
    pub fn auth_check_response(&self, caller: &CallerContext) -> Value {
        serde_json::json!({
            "authenticated": caller.bearer_token.is_some(),
            "enforcement": self.mode.as_str(),
        })
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
#[expect(clippy::unwrap_used, clippy::expect_used, reason = "test code")]
mod tests {
    use super::*;

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

    #[test]
    fn gate_allows_public_methods_without_token() {
        let gate = MethodGate::new(EnforcementMode::Enforced);
        let caller = CallerContext::unix();
        assert!(gate.check("health.check", &caller).is_ok());
        assert!(gate.check("identity.get", &caller).is_ok());
        assert!(gate.check("auth.mode", &caller).is_ok());
    }

    #[test]
    fn gate_allows_protected_with_token() {
        let gate = MethodGate::new(EnforcementMode::Enforced);
        let caller = CallerContext {
            bearer_token: Some("test-token".to_owned()),
            origin: ConnectionOrigin::Unix,
        };
        assert!(gate.check("dag.session.create", &caller).is_ok());
    }

    #[test]
    fn gate_permissive_allows_protected_without_token() {
        let gate = MethodGate::new(EnforcementMode::Permissive);
        let caller = CallerContext::unix();
        assert!(gate.check("dag.session.create", &caller).is_ok());
    }

    #[test]
    fn gate_enforced_rejects_protected_without_token() {
        let gate = MethodGate::new(EnforcementMode::Enforced);
        let caller = CallerContext::unix();
        let result = gate.check("dag.session.create", &caller);
        assert!(result.is_err());
        let rejection = result.unwrap_err();
        assert_eq!(rejection.method, "dag.session.create");
    }

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

    #[test]
    fn auth_check_response_unauthenticated() {
        let gate = MethodGate::new(EnforcementMode::Permissive);
        let caller = CallerContext::unix();
        let resp = gate.auth_check_response(&caller);
        assert_eq!(resp["authenticated"], false);
        assert_eq!(resp["enforcement"], "permissive");
    }

    #[test]
    fn auth_check_response_authenticated() {
        let gate = MethodGate::new(EnforcementMode::Enforced);
        let caller = CallerContext {
            bearer_token: Some("tok".to_owned()),
            origin: ConnectionOrigin::Loopback,
        };
        let resp = gate.auth_check_response(&caller);
        assert_eq!(resp["authenticated"], true);
        assert_eq!(resp["enforcement"], "enforced");
    }

    #[test]
    fn auth_mode_response() {
        let gate = MethodGate::new(EnforcementMode::Permissive);
        assert_eq!(gate.auth_mode_response()["mode"], "permissive");
        let gate = MethodGate::new(EnforcementMode::Enforced);
        assert_eq!(gate.auth_mode_response()["mode"], "enforced");
    }

    #[test]
    fn auth_peer_info_response() {
        let gate = MethodGate::new(EnforcementMode::Permissive);
        let caller = CallerContext::unix();
        let resp = gate.auth_peer_info_response(&caller);
        assert_eq!(resp["origin"], "Unix");
        assert_eq!(resp["has_token"], false);
    }
}
