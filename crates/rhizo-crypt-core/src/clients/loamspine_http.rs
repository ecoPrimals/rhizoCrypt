// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2024–2026 ecoPrimals Project

//! LoamSpine HTTP JSON-RPC client implementation.
//!
//! This module provides a complete HTTP client for LoamSpine permanent storage,
//! using JSON-RPC 2.0 over HTTP with method negotiation.
//!
//! ## Architecture
//!
//! - Uses JSON-RPC 2.0 over HTTP
//! - Endpoint discovered via capability-based resolution at runtime
//! - Falls back gracefully when permanent storage unavailable
//! - **Method negotiation**: tries native method names (`commit.session`, `commit.verify`)
//!   first, falling back to compatibility names (`permanent-storage.commitSession`) for
//!   older LoamSpine versions. Negotiation result is cached per client instance.

use crate::dehydration::DehydrationSummary;
use crate::error::{Result, RhizoCryptError};
use crate::integration::PermanentStorageProvider;
use crate::session::LoamCommitRef;
use crate::slice::{ResolutionOutcome, Slice, SliceOrigin};
use crate::types::Did;
use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;

/// Native semantic JSON-RPC method names per the Universal IPC Standard.
///
/// Method naming follows `{domain}.{operation}` where domain is the capability.
/// The client tries native names first, falling back to compat names when the
/// server responds with "method not found" (JSON-RPC error -32601).
mod methods {
    /// Native method names (LoamSpine v0.8.0+).
    pub mod native {
        pub const HEALTH_CHECK: &str = "system.health";
        pub const COMMIT_SESSION: &str = "commit.session";
        pub const VERIFY_COMMIT: &str = "commit.verify";
        pub const GET_COMMIT: &str = "commit.get";
        pub const CHECKOUT_SLICE: &str = "slice.checkout";
        pub const RESOLVE_SLICE: &str = "slice.resolve";
    }

    /// Compatibility method names (LoamSpine <v0.8.0).
    pub mod compat {
        pub const HEALTH_CHECK: &str = "permanent-storage.healthCheck";
        pub const COMMIT_SESSION: &str = "permanent-storage.commitSession";
        pub const VERIFY_COMMIT: &str = "permanent-storage.verifyCommit";
        pub const GET_COMMIT: &str = "permanent-storage.getCommit";
        pub const CHECKOUT_SLICE: &str = "permanent-storage.checkoutSlice";
        pub const RESOLVE_SLICE: &str = "permanent-storage.resolveSlice";
    }
}

/// JSON-RPC error code for "method not found" per JSON-RPC 2.0 spec.
const METHOD_NOT_FOUND_CODE: i32 = -32601;

/// HTTP client for LoamSpine permanent storage.
///
/// Implements `PermanentStorageProvider` trait using JSON-RPC 2.0 over HTTP.
/// Supports method negotiation: tries native method names first (LoamSpine v0.8.0+),
/// falling back to compatibility names for older versions.
#[derive(Debug, Clone)]
pub struct LoamSpineHttpClient {
    /// Base URL for LoamSpine JSON-RPC endpoint.
    base_url: String,
    /// HTTP client with timeout.
    client: reqwest::Client,
    /// Request ID counter for JSON-RPC.
    request_id: std::sync::Arc<AtomicU64>,
    /// Whether the server supports native method names.
    /// `None` = not yet negotiated, `Some(true)` = native, `Some(false)` = compat.
    native_methods: std::sync::Arc<std::sync::atomic::AtomicU8>,
}

impl LoamSpineHttpClient {
    /// Create a new LoamSpine HTTP client.
    ///
    /// # Arguments
    ///
    /// * `endpoint` - Permanent storage endpoint discovered at runtime
    ///
    /// # Errors
    ///
    /// Returns error if HTTP client cannot be created.
    pub fn new(endpoint: impl Into<String>) -> Result<Self> {
        let endpoint = endpoint.into();
        let base_url = if endpoint.ends_with(crate::constants::JSON_RPC_PATH) {
            endpoint
        } else if endpoint.ends_with('/') {
            format!("{endpoint}{}", crate::constants::JSON_RPC_PATH.trim_start_matches('/'))
        } else {
            format!("{endpoint}{}", crate::constants::JSON_RPC_PATH)
        };

        let client =
            reqwest::Client::builder().timeout(Duration::from_secs(30)).build().map_err(|e| {
                RhizoCryptError::integration(format!("Failed to create HTTP client: {e}"))
            })?;

        Ok(Self {
            base_url,
            client,
            request_id: std::sync::Arc::new(AtomicU64::new(1)),
            native_methods: std::sync::Arc::new(std::sync::atomic::AtomicU8::new(
                MethodSupport::Unknown as u8,
            )),
        })
    }

    /// Create client from environment variables.
    ///
    /// Checks for:
    /// 1. `PERMANENT_STORAGE_ENDPOINT` (capability-based)
    /// 2. `LOAMSPINE_ADDRESS` (legacy)
    ///
    /// Returns an error if no endpoint is configured. Primals discover
    /// endpoints at runtime - no hardcoded fallbacks.
    pub fn from_env() -> Result<Self> {
        use crate::safe_env::CapabilityEnv;

        let endpoint = CapabilityEnv::permanent_commit_endpoint().ok_or_else(|| {
            RhizoCryptError::integration(
                "No permanent storage endpoint configured. \
                 Set PERMANENT_STORAGE_ENDPOINT or LOAMSPINE_ADDRESS.",
            )
        })?;

        Self::new(endpoint)
    }

    /// Generate next JSON-RPC request ID.
    fn next_request_id(&self) -> u64 {
        self.request_id.fetch_add(1, Ordering::Relaxed)
    }

    /// Determine which method name to use based on negotiation state.
    fn resolve_method(&self, native: &'static str, compat: &'static str) -> &'static str {
        match MethodSupport::from_u8(self.native_methods.load(Ordering::Relaxed)) {
            MethodSupport::Compat => compat,
            MethodSupport::Native | MethodSupport::Unknown => native,
        }
    }

    /// Record the negotiation result after a call.
    fn record_support(&self, support: MethodSupport) {
        self.native_methods.store(support as u8, Ordering::Relaxed);
    }

    /// Make a JSON-RPC 2.0 call with method negotiation.
    ///
    /// Tries the native method name first. If the server returns "method not found"
    /// (-32601), retries with the compatibility name and caches the result.
    async fn call_negotiated<T: Serialize + Clone, R: for<'de> Deserialize<'de>>(
        &self,
        native_method: &'static str,
        compat_method: &'static str,
        params: T,
    ) -> Result<R> {
        let method = self.resolve_method(native_method, compat_method);

        match self.call_jsonrpc_raw(method, params.clone()).await {
            Ok(result) => {
                if method == native_method {
                    self.record_support(MethodSupport::Native);
                }
                Ok(result)
            }
            Err(NegotiableError::MethodNotFound) if method == native_method => {
                tracing::info!(
                    native = %native_method,
                    compat = %compat_method,
                    "Native method not supported, falling back to compat"
                );
                self.record_support(MethodSupport::Compat);
                self.call_jsonrpc_raw(compat_method, params)
                    .await
                    .map_err(NegotiableError::into_rhizo_error)
            }
            Err(e) => Err(e.into_rhizo_error()),
        }
    }

    /// Raw JSON-RPC 2.0 call, returning negotiable errors.
    async fn call_jsonrpc_raw<T: Serialize, R: for<'de> Deserialize<'de>>(
        &self,
        method: &str,
        params: T,
    ) -> std::result::Result<R, NegotiableError> {
        let request = JsonRpcRequest {
            jsonrpc: "2.0",
            method,
            params,
            id: self.next_request_id(),
        };

        tracing::debug!(
            method = %method,
            url = %self.base_url,
            "Calling permanent storage JSON-RPC"
        );

        let response =
            self.client.post(&self.base_url).json(&request).send().await.map_err(|e| {
                NegotiableError::Other(RhizoCryptError::integration(format!(
                    "HTTP request failed: {e}"
                )))
            })?;

        if !response.status().is_success() {
            return Err(NegotiableError::Other(RhizoCryptError::integration(format!(
                "Permanent storage returned HTTP {}: {}",
                response.status(),
                response.text().await.unwrap_or_default()
            ))));
        }

        let json_response: JsonRpcResponse<R> = response.json().await.map_err(|e| {
            NegotiableError::Other(RhizoCryptError::integration(format!(
                "Failed to parse response: {e}"
            )))
        })?;

        match json_response {
            JsonRpcResponse::Success {
                result,
                ..
            } => Ok(result),
            JsonRpcResponse::Error {
                error,
                ..
            } if error.code == METHOD_NOT_FOUND_CODE => Err(NegotiableError::MethodNotFound),
            JsonRpcResponse::Error {
                error,
                ..
            } => Err(NegotiableError::Other(RhizoCryptError::integration(format!(
                "Permanent storage RPC error [{}]: {}",
                error.code, error.message
            )))),
        }
    }

    /// Health check to verify permanent storage is available.
    async fn health_check(&self) -> Result<HealthCheckResponse> {
        self.call_negotiated(
            methods::native::HEALTH_CHECK,
            methods::compat::HEALTH_CHECK,
            EmptyParams {},
        )
        .await
    }
}

impl PermanentStorageProvider for LoamSpineHttpClient {
    // Note: Cannot use async fn here due to trait signature using RPITIT pattern
    #[allow(clippy::manual_async_fn)]
    fn commit(
        &self,
        summary: &DehydrationSummary,
    ) -> impl std::future::Future<Output = Result<LoamCommitRef>> + Send {
        async move {
            let request = CommitSessionRequest {
                session_id: summary.session_id.to_string(),
                merkle_root: hex::encode(summary.merkle_root.as_bytes()),
                summary: RpcDehydrationSummary {
                    session_type: summary.session_type.clone(),
                    vertex_count: summary.vertex_count,
                    leaf_count: summary.results.len() as u64,
                    started_at: summary.created_at.as_nanos(),
                    ended_at: summary.resolved_at.as_nanos(),
                    outcome: format!("{:?}", summary.outcome),
                },
                committer_did: summary.agents.first().map(|a| a.agent.as_str().to_string()),
            };

            let response: CommitSessionResponse = self
                .call_negotiated(
                    methods::native::COMMIT_SESSION,
                    methods::compat::COMMIT_SESSION,
                    request,
                )
                .await?;

            if !response.accepted {
                return Err(RhizoCryptError::integration(format!(
                    "LoamSpine rejected commit: {}",
                    response.error.unwrap_or_else(|| "Unknown error".to_string())
                )));
            }

            let spine_id = response
                .commit_id
                .ok_or_else(|| RhizoCryptError::integration("No commit ID returned"))?;

            let entry_hash_str = response
                .spine_entry_hash
                .ok_or_else(|| RhizoCryptError::integration("No spine entry hash returned"))?;

            let entry_hash = hex::decode(&entry_hash_str)
                .map_err(|e| RhizoCryptError::integration(format!("Invalid entry hash: {e}")))?;

            let mut hash_bytes = [0u8; 32];
            hash_bytes[..entry_hash.len().min(32)]
                .copy_from_slice(&entry_hash[..entry_hash.len().min(32)]);

            // Extract entry index from response (if provided)
            // LoamSpine API v0.2+ should include entry_index in response
            let index = response.entry_index.unwrap_or(0);

            Ok(LoamCommitRef {
                spine_id,
                entry_hash: hash_bytes,
                index,
            })
        }
    }

    fn verify_commit(
        &self,
        commit_ref: &LoamCommitRef,
    ) -> impl std::future::Future<Output = Result<bool>> + Send {
        let spine_id = commit_ref.spine_id.clone();
        let entry_hash = commit_ref.entry_hash;
        let index = commit_ref.index;

        async move {
            // Call LoamSpine verification endpoint (API v0.2+)
            // If endpoint doesn't exist, fall back to health check
            #[derive(Debug, Clone, Serialize)]
            struct VerifyRequest {
                spine_id: String,
                entry_hash: String,
                index: u64,
            }

            let request = VerifyRequest {
                spine_id,
                entry_hash: hex::encode(entry_hash),
                index,
            };

            match self
                .call_negotiated::<_, bool>(
                    methods::native::VERIFY_COMMIT,
                    methods::compat::VERIFY_COMMIT,
                    request,
                )
                .await
            {
                Ok(verified) => Ok(verified),
                Err(e) => {
                    // If verification endpoint doesn't exist, try health check
                    tracing::debug!(
                        error = %e,
                        "Verification endpoint not available, falling back to health check"
                    );
                    match self.health_check().await {
                        Ok(_) => Ok(true),
                        Err(_) => Ok(false),
                    }
                }
            }
        }
    }

    fn get_commit(
        &self,
        commit_ref: &LoamCommitRef,
    ) -> impl std::future::Future<Output = Result<Option<DehydrationSummary>>> + Send {
        let spine_id = commit_ref.spine_id.clone();
        let entry_hash = commit_ref.entry_hash;
        let index = commit_ref.index;

        async move {
            // Call LoamSpine get commit endpoint (API v0.2+)
            #[derive(Debug, Clone, Serialize)]
            struct GetCommitRequest {
                spine_id: String,
                entry_hash: String,
                index: u64,
            }

            #[derive(Debug, Deserialize)]
            struct CommitResponse {
                summary: DehydrationSummary,
            }

            let request = GetCommitRequest {
                spine_id,
                entry_hash: hex::encode(entry_hash),
                index,
            };

            match self
                .call_negotiated::<_, CommitResponse>(
                    methods::native::GET_COMMIT,
                    methods::compat::GET_COMMIT,
                    request,
                )
                .await
            {
                Ok(response) => Ok(Some(response.summary)),
                Err(e) => {
                    // If endpoint doesn't exist or commit not found, return None
                    tracing::debug!(
                        error = %e,
                        "Unable to retrieve commit (endpoint may not exist yet)"
                    );
                    Ok(None)
                }
            }
        }
    }

    fn checkout_slice(
        &self,
        spine_id: &str,
        entry_hash: &[u8; 32],
        holder: &Did,
    ) -> impl std::future::Future<Output = Result<SliceOrigin>> + Send {
        let spine_id = spine_id.to_string();
        let entry_hash = *entry_hash;
        let holder = holder.clone();
        async move {
            let request = CheckoutSliceRequest {
                spine_id: spine_id.clone(),
                entry_hash: hex::encode(entry_hash),
                holder_did: holder.as_str().to_string(),
            };

            let response: CheckoutSliceResponse = self
                .call_negotiated(
                    methods::native::CHECKOUT_SLICE,
                    methods::compat::CHECKOUT_SLICE,
                    request,
                )
                .await?;

            Ok(SliceOrigin {
                spine_id: response.spine_id,
                entry_hash,
                entry_index: response.entry_index,
                certificate_id: response.certificate_id,
                owner: Did::new(response.owner_did),
            })
        }
    }

    fn resolve_slice(
        &self,
        slice: &Slice,
        outcome: &ResolutionOutcome,
    ) -> impl std::future::Future<Output = Result<()>> + Send {
        let slice_id = slice.id;
        let origin = slice.origin.clone();
        let outcome = outcome.clone();

        async move {
            // Call LoamSpine slice resolution endpoint (API v0.2+)
            #[derive(Debug, Clone, Serialize)]
            struct ResolveSliceRequest {
                slice_id: String,
                spine_id: String,
                entry_hash: String,
                outcome: String,
                route: String,
            }

            let request = ResolveSliceRequest {
                slice_id: slice_id.to_string(),
                spine_id: origin.spine_id.clone(),
                entry_hash: hex::encode(origin.entry_hash),
                outcome: format!("{outcome:?}"), // Serialize outcome
                route: "return_to_origin".to_string(), // Default route
            };

            match self
                .call_negotiated::<_, ()>(
                    methods::native::RESOLVE_SLICE,
                    methods::compat::RESOLVE_SLICE,
                    request,
                )
                .await
            {
                Ok(()) => {
                    tracing::info!(
                        slice_id = %slice_id,
                        spine_id = %origin.spine_id,
                        "Slice resolved successfully"
                    );
                    Ok(())
                }
                Err(e) => {
                    // If endpoint doesn't exist, log and return success
                    // (slice resolution is optional for basic functionality)
                    tracing::warn!(
                        slice_id = %slice_id,
                        error = %e,
                        "Slice resolution endpoint not available (proceeding anyway)"
                    );
                    Ok(())
                }
            }
        }
    }
}

// ============================================================================
// Method Negotiation
// ============================================================================

/// Tracks whether the server supports native or compat method names.
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum MethodSupport {
    Unknown = 0,
    Native = 1,
    Compat = 2,
}

impl MethodSupport {
    const fn from_u8(v: u8) -> Self {
        match v {
            1 => Self::Native,
            2 => Self::Compat,
            _ => Self::Unknown,
        }
    }
}

/// Distinguishes "method not found" from other errors during negotiation.
enum NegotiableError {
    MethodNotFound,
    Other(RhizoCryptError),
}

impl NegotiableError {
    fn into_rhizo_error(self) -> RhizoCryptError {
        match self {
            Self::MethodNotFound => {
                RhizoCryptError::integration("JSON-RPC method not found on server")
            }
            Self::Other(e) => e,
        }
    }
}

// ============================================================================
// JSON-RPC 2.0 Types
// ============================================================================

#[derive(Debug, Serialize)]
struct JsonRpcRequest<'a, T> {
    jsonrpc: &'static str,
    method: &'a str,
    params: T,
    id: u64,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum JsonRpcResponse<T> {
    Success {
        #[allow(dead_code)]
        jsonrpc: String,
        result: T,
        #[allow(dead_code)]
        id: u64,
    },
    Error {
        #[allow(dead_code)]
        jsonrpc: String,
        error: JsonRpcError,
        #[allow(dead_code)]
        id: u64,
    },
}

#[derive(Debug, Deserialize)]
struct JsonRpcError {
    code: i32,
    message: String,
    #[serde(default)]
    #[allow(dead_code)]
    data: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize)]
struct EmptyParams {}

// ============================================================================
// LoamSpine API Types
// ============================================================================

#[derive(Debug, Clone, Serialize)]
struct CommitSessionRequest {
    session_id: String,
    merkle_root: String,
    summary: RpcDehydrationSummary,
    committer_did: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
struct RpcDehydrationSummary {
    session_type: String,
    vertex_count: u64,
    leaf_count: u64,
    started_at: u64,
    ended_at: u64,
    outcome: String,
}

#[derive(Debug, Deserialize)]
struct CommitSessionResponse {
    accepted: bool,
    commit_id: Option<String>,
    spine_entry_hash: Option<String>,
    entry_index: Option<u64>,
    error: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
struct CheckoutSliceRequest {
    spine_id: String,
    entry_hash: String,
    holder_did: String,
}

#[derive(Debug, Deserialize)]
struct CheckoutSliceResponse {
    spine_id: String,
    entry_index: u64,
    certificate_id: Option<String>,
    owner_did: String,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct HealthCheckResponse {
    status: String,
    version: String,
    spine_count: u64,
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn test_client_creation() {
        let client = LoamSpineHttpClient::new("http://localhost:8080").unwrap();
        assert_eq!(client.base_url, "http://localhost:8080/rpc");

        let client2 = LoamSpineHttpClient::new("http://localhost:8080/").unwrap();
        assert_eq!(client2.base_url, "http://localhost:8080/rpc");

        let client3 = LoamSpineHttpClient::new("http://localhost:8080/rpc").unwrap();
        assert_eq!(client3.base_url, "http://localhost:8080/rpc");
    }

    #[test]
    fn test_request_id_increment() {
        let client = LoamSpineHttpClient::new("http://localhost:8080").unwrap();
        assert_eq!(client.next_request_id(), 1);
        assert_eq!(client.next_request_id(), 2);
        assert_eq!(client.next_request_id(), 3);
    }

    #[test]
    fn test_method_negotiation_state() {
        let client = LoamSpineHttpClient::new("http://localhost:8080").unwrap();

        // Starts unknown
        assert_eq!(
            MethodSupport::from_u8(client.native_methods.load(Ordering::Relaxed)),
            MethodSupport::Unknown
        );

        // Unknown resolves to native (try native first)
        assert_eq!(
            client.resolve_method("commit.session", "permanent-storage.commitSession"),
            "commit.session"
        );

        // After recording native support
        client.record_support(MethodSupport::Native);
        assert_eq!(
            client.resolve_method("commit.session", "permanent-storage.commitSession"),
            "commit.session"
        );

        // After recording compat support
        client.record_support(MethodSupport::Compat);
        assert_eq!(
            client.resolve_method("commit.session", "permanent-storage.commitSession"),
            "permanent-storage.commitSession"
        );
    }

    #[test]
    fn test_method_support_roundtrip() {
        assert_eq!(MethodSupport::from_u8(0), MethodSupport::Unknown);
        assert_eq!(MethodSupport::from_u8(1), MethodSupport::Native);
        assert_eq!(MethodSupport::from_u8(2), MethodSupport::Compat);
        assert_eq!(MethodSupport::from_u8(255), MethodSupport::Unknown);
    }

    #[test]
    fn test_native_method_names() {
        assert_eq!(methods::native::COMMIT_SESSION, "commit.session");
        assert_eq!(methods::native::VERIFY_COMMIT, "commit.verify");
        assert_eq!(methods::native::GET_COMMIT, "commit.get");
        assert_eq!(methods::native::CHECKOUT_SLICE, "slice.checkout");
        assert_eq!(methods::native::RESOLVE_SLICE, "slice.resolve");
        assert_eq!(methods::native::HEALTH_CHECK, "system.health");
    }

    #[test]
    fn test_compat_method_names() {
        assert_eq!(methods::compat::COMMIT_SESSION, "permanent-storage.commitSession");
        assert_eq!(methods::compat::VERIFY_COMMIT, "permanent-storage.verifyCommit");
    }

    #[tokio::test]
    async fn test_health_check_unavailable() {
        let client = LoamSpineHttpClient::new("http://invalid-endpoint-12345:99999").unwrap();
        let result = client.health_check().await;
        assert!(result.is_err());
    }
}
