// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2024–2026 ecoPrimals Project

//! Token verification for the JSON-RPC method gate.

use rhizo_crypt_core::constants::{PROVENANCE_CONNECTION_TIMEOUT, PROVENANCE_RESPONSE_TIMEOUT};
use rhizo_crypt_core::discovery::{Capability, DiscoveryRegistry, ServiceEndpoint};
use rhizo_crypt_core::transport::{JsonRpcTransportError, send_jsonrpc_request};
use serde_json::Value;
use std::future::Future;
use std::pin::Pin;
use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant};

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
pub(super) fn parse_verify_ionic_response(response_line: &str) -> Result<VerifiedClaims, String> {
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
pub(super) fn extract_scope_list(result: &Value) -> Option<Vec<String>> {
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
pub(super) fn expires_in_from_claims(claims: &Value) -> Option<u64> {
    let exp = claims.get("exp")?.as_u64()?;
    let now = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).ok()?.as_secs();
    exp.checked_sub(now)
}
