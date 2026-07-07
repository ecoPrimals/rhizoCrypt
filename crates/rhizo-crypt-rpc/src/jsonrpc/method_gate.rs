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
//! [`NoopVerifier`] accepts any token (presence-only check); production
//! uses [`CapabilityVerifier`] when a [`DiscoveryRegistry`] is available,
//! falling back to [`PresenceVerifier`] when no `crypto:signing` provider
//! is discoverable.
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

use rhizo_crypt_core::constants::{PROVENANCE_CONNECTION_TIMEOUT, PROVENANCE_RESPONSE_TIMEOUT};
use rhizo_crypt_core::discovery::{Capability, DiscoveryRegistry, ServiceEndpoint};
use rhizo_crypt_core::transport::{JsonRpcTransportError, send_jsonrpc_request};
use serde_json::Value;
use std::future::Future;
use std::pin::Pin;
use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant};

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
const PUBLIC_METHOD_PREFIXES: &[&str] = &[rhizo_crypt_core::constants::HEALTH_METHOD_PREFIX];

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
/// and production uses [`CapabilityVerifier`] (or [`PresenceVerifier`] fallback).
pub trait TokenVerifier: Send + Sync + std::fmt::Debug {
    /// Verify a bearer token string and return the embedded claims.
    ///
    /// Returns `None` if the token is invalid, expired, or unverifiable.
    fn verify(&self, token: &str) -> Option<VerifiedClaims>;

    /// Async verification — preferred on Tokio runtimes to avoid blocking.
    ///
    /// Default implementation delegates to [`Self::verify`].
    fn verify_async<'a>(
        &'a self,
        token: &'a str,
    ) -> Pin<Box<dyn Future<Output = Option<VerifiedClaims>> + Send + 'a>> {
        Box::pin(std::future::ready(self.verify(token)))
    }
}

/// Accepts any non-empty token as valid with wildcard scope.
///
/// Test-only verifier equivalent to the "presence-only" check from JH-0.
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

/// Presence-only token verifier (pre-JH-11 placeholder).
///
/// Accepts any non-empty token and grants `scopes: ["*"]`. When ecosystem
/// key distribution (JH-11) ships, this will be replaced by a verifier
/// that performs Ed25519 signature verification and scope extraction
/// via `auth.verify_ionic` IPC (capability-discovered, not primal-named).
#[derive(Debug)]
pub struct PresenceVerifier;

impl TokenVerifier for PresenceVerifier {
    fn verify(&self, token: &str) -> Option<VerifiedClaims> {
        if token.is_empty() {
            return None;
        }
        Some(VerifiedClaims {
            subject: "unverified".to_owned(),
            scopes: vec!["*".to_owned()],
            expires_in: None,
        })
    }
}

// ============================================================================
// CAPABILITY-BASED VERIFIER
// ============================================================================

/// TTL for cached `crypto:signing` provider endpoint lookups.
const SIGNING_PROVIDER_CACHE_TTL: Duration = Duration::from_secs(30);

/// JSON-RPC method invoked on discovered signing providers for ionic verification.
const AUTH_VERIFY_IONIC_METHOD: &str = "auth.verify_ionic";

/// Errors during capability-discovered token verification (internal only).
#[derive(Debug, thiserror::Error)]
enum CapabilityVerifyError {
    /// No `crypto:signing` provider could be discovered.
    #[error("no signing provider available")]
    NoProvider,
    /// JSON-RPC transport to the signing provider failed.
    #[error("transport error: {0}")]
    Transport(#[from] JsonRpcTransportError),
    /// Response body could not be parsed into claims.
    #[error("invalid verify response: {0}")]
    InvalidResponse(String),
}

/// Cached signing-provider endpoint from discovery.
#[derive(Debug, Clone)]
struct CachedSigningProvider {
    endpoint: ServiceEndpoint,
    fetched_at: Instant,
}

/// Runtime capability-discovered token verifier.
///
/// Discovers any primal advertising `crypto:signing` via [`DiscoveryRegistry`],
/// then delegates ionic token validation to `auth.verify_ionic` over JSON-RPC.
///
/// Fallback behavior depends on `fail_open`:
/// - **`true`** (Permissive): falls back to [`PresenceVerifier`] when no
///   signing provider is discovered — grants `scopes: ["*"]` to any token.
/// - **`false`** (Enforced): returns `None` when no provider is available,
///   meaning the token is treated as unverified and `MethodGate::check()`
///   will reject the call.
#[derive(Debug)]
pub struct CapabilityVerifier {
    registry: Arc<DiscoveryRegistry>,
    cached_provider: RwLock<Option<CachedSigningProvider>>,
    presence_fallback: PresenceVerifier,
    fail_open: bool,
}

impl CapabilityVerifier {
    /// Create a verifier backed by the given discovery registry.
    ///
    /// `fail_open` controls fallback when no `crypto:signing` provider is
    /// discovered: `true` falls back to presence-only verification (permissive),
    /// `false` treats the token as unverified (enforced / fail-closed).
    #[must_use]
    #[allow(clippy::missing_const_for_fn, reason = "Arc parameter is not const-constructible")]
    pub fn new(registry: Arc<DiscoveryRegistry>, fail_open: bool) -> Self {
        Self {
            registry,
            cached_provider: RwLock::new(None),
            presence_fallback: PresenceVerifier,
            fail_open,
        }
    }

    async fn verify_via_capability(&self, token: &str) -> Option<VerifiedClaims> {
        if token.is_empty() {
            return None;
        }

        match self.verify_with_provider(token).await {
            Ok(claims) => Some(claims),
            Err(CapabilityVerifyError::NoProvider) if self.fail_open => {
                tracing::warn!(
                    capability = %Capability::Signing,
                    "method gate: no crypto:signing provider — falling back to presence-only verification (permissive)"
                );
                self.presence_fallback.verify(token)
            }
            Err(CapabilityVerifyError::NoProvider) => {
                tracing::error!(
                    capability = %Capability::Signing,
                    "method gate: no crypto:signing provider — token unverified (fail-closed)"
                );
                None
            }
            Err(e) => {
                tracing::warn!(error = %e, "method gate: capability token verification failed");
                None
            }
        }
    }

    async fn verify_with_provider(
        &self,
        token: &str,
    ) -> Result<VerifiedClaims, CapabilityVerifyError> {
        let endpoint = self.resolve_signing_endpoint().await?;
        let transport = endpoint.endpoint.clone();

        let request = serde_json::json!({
            "jsonrpc": rhizo_crypt_core::constants::JSONRPC_VERSION,
            "method": AUTH_VERIFY_IONIC_METHOD,
            "params": { "token": token },
            "id": 1
        });

        let response_line = send_jsonrpc_request(
            &transport,
            &request,
            PROVENANCE_CONNECTION_TIMEOUT,
            PROVENANCE_RESPONSE_TIMEOUT,
        )
        .await?;

        parse_verify_ionic_response(&response_line).map_err(CapabilityVerifyError::InvalidResponse)
    }

    async fn resolve_signing_endpoint(&self) -> Result<ServiceEndpoint, CapabilityVerifyError> {
        if let Ok(guard) = self.cached_provider.read()
            && let Some(cached) = guard.as_ref()
            && cached.fetched_at.elapsed() < SIGNING_PROVIDER_CACHE_TTL
        {
            return Ok(cached.endpoint.clone());
        }

        let status = self.registry.discover(&Capability::Signing).await;
        let endpoint = match status {
            rhizo_crypt_core::discovery::DiscoveryStatus::Available(mut endpoints) => {
                endpoints.pop().ok_or(CapabilityVerifyError::NoProvider)?
            }
            rhizo_crypt_core::discovery::DiscoveryStatus::Unavailable
            | rhizo_crypt_core::discovery::DiscoveryStatus::Discovering => {
                return Err(CapabilityVerifyError::NoProvider);
            }
            rhizo_crypt_core::discovery::DiscoveryStatus::Failed(err) => {
                tracing::debug!(error = %err, "signing provider discovery failed");
                return Err(CapabilityVerifyError::NoProvider);
            }
        };

        if let Ok(mut guard) = self.cached_provider.write() {
            *guard = Some(CachedSigningProvider {
                endpoint: endpoint.clone(),
                fetched_at: Instant::now(),
            });
        }

        Ok(endpoint)
    }
}

impl TokenVerifier for CapabilityVerifier {
    fn verify(&self, token: &str) -> Option<VerifiedClaims> {
        if token.is_empty() {
            return None;
        }
        tokio::runtime::Handle::try_current().map_or_else(
            |_| {
                if self.fail_open {
                    tracing::debug!(
                        "method gate: no Tokio runtime — using presence fallback (permissive)"
                    );
                    self.presence_fallback.verify(token)
                } else {
                    tracing::error!(
                        "method gate: no Tokio runtime — token unverified (fail-closed)"
                    );
                    None
                }
            },
            |handle| handle.block_on(self.verify_via_capability(token)),
        )
    }

    fn verify_async<'a>(
        &'a self,
        token: &'a str,
    ) -> Pin<Box<dyn Future<Output = Option<VerifiedClaims>> + Send + 'a>> {
        Box::pin(self.verify_via_capability(token))
    }
}

/// Parse a JSON-RPC `auth.verify_ionic` response into [`VerifiedClaims`].
fn parse_verify_ionic_response(response_line: &str) -> Result<VerifiedClaims, String> {
    let envelope: Value =
        serde_json::from_str(response_line).map_err(|e| format!("not valid JSON: {e}"))?;

    if envelope.get("error").is_some() {
        return Err("JSON-RPC error response".to_owned());
    }

    let result = envelope.get("result").ok_or_else(|| "missing result field".to_owned())?;

    let valid = result.get("valid").and_then(Value::as_bool).unwrap_or(false);
    if !valid {
        return Err("token marked invalid by provider".to_owned());
    }

    let claims = result.get("claims").ok_or_else(|| "missing claims".to_owned())?;

    let subject = claims
        .get("sub")
        .and_then(Value::as_str)
        .map(str::to_owned)
        .ok_or_else(|| "missing subject (sub)".to_owned())?;

    let scopes = extract_scope_list(result).ok_or_else(|| "missing scopes".to_owned())?;

    let expires_in = expires_in_from_claims(claims);

    Ok(VerifiedClaims {
        subject,
        scopes,
        expires_in,
    })
}

/// Extract scope patterns from an `auth.verify_ionic` result object.
fn extract_scope_list(result: &Value) -> Option<Vec<String>> {
    for key in ["scopes", "scope"] {
        if let Some(arr) = result.get(key).and_then(Value::as_array) {
            let scopes: Vec<String> =
                arr.iter().filter_map(|v| v.as_str().map(str::to_owned)).collect();
            if !scopes.is_empty() {
                return Some(scopes);
            }
        }
    }
    result
        .get("claims")
        .and_then(|c| c.get("scope").or_else(|| c.get("scopes")))
        .and_then(Value::as_array)
        .map(|arr| arr.iter().filter_map(|v| v.as_str().map(str::to_owned)).collect())
        .filter(|v: &Vec<String>| !v.is_empty())
}

/// Compute seconds until token expiry from JWT-style `exp` claim.
fn expires_in_from_claims(claims: &Value) -> Option<u64> {
    let exp = claims.get("exp")?.as_u64()?;
    let now = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).ok()?.as_secs();
    exp.checked_sub(now)
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

    /// Run token verification and populate `verified_claims` (sync).
    pub fn verify_token(&mut self, verifier: &dyn TokenVerifier) {
        self.verified_claims = self.bearer_token.as_deref().and_then(|t| verifier.verify(t));
    }

    /// Run token verification asynchronously (preferred on Tokio runtimes).
    pub async fn verify_token_async(&mut self, verifier: &dyn TokenVerifier) {
        self.verified_claims = match self.bearer_token.as_deref() {
            Some(t) => verifier.verify_async(t).await,
            None => None,
        };
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
        rhizo_crypt_core::SafeEnv::get_optional(rhizo_crypt_core::SafeEnv::RHIZOCRYPT_AUTH_MODE)
            .map_or(Self::Permissive, |v| match v.to_lowercase().as_str() {
                "enforced" | "enforce" | "strict" => Self::Enforced,
                _ => Self::Permissive,
            })
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
    /// Without a discovery registry this uses [`PresenceVerifier`]. Prefer
    /// [`Self::from_env_with_registry`] or [`Self::with_discovery`] when a
    /// [`DiscoveryRegistry`] is available.
    #[must_use]
    pub fn from_env() -> Self {
        Self::from_env_with_registry(None)
    }

    /// Create a gate from the environment, wiring [`CapabilityVerifier`] when
    /// a discovery registry is provided.
    #[must_use]
    pub fn from_env_with_registry(registry: Option<Arc<DiscoveryRegistry>>) -> Self {
        let mode = EnforcementMode::from_env();
        registry.map_or_else(
            || Self::new(mode, Box::new(PresenceVerifier)),
            |reg| Self::with_discovery(mode, reg),
        )
    }

    /// Create a gate with capability-discovered token verification.
    ///
    /// In `Enforced` mode the verifier is fail-closed: tokens are rejected
    /// when no signing provider is available. In `Permissive` mode the
    /// verifier falls back to presence-only verification.
    #[must_use]
    pub fn with_discovery(mode: EnforcementMode, registry: Arc<DiscoveryRegistry>) -> Self {
        let fail_open = mode == EnforcementMode::Permissive;
        Self::new(mode, Box::new(CapabilityVerifier::new(registry, fail_open)))
    }

    /// Create a gate from the environment using a primal's discovery registry.
    #[must_use]
    pub fn for_primal(primal: &rhizo_crypt_core::RhizoCrypt) -> Self {
        Self::from_env_with_registry(Some(Arc::clone(primal.discovery_registry())))
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
#[path = "method_gate_tests.rs"]
mod tests;

#[cfg(test)]
#[path = "method_gate_tests_provider.rs"]
mod tests_provider;
