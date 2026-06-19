// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2024–2026 ecoPrimals Project

//! `LoamSpine` HTTP JSON-RPC client implementation.
//!
//! This module provides a complete HTTP client for `LoamSpine` permanent storage,
//! using JSON-RPC 2.0 over HTTP with method negotiation.
//!
//! ## Architecture
//!
//! - Uses JSON-RPC 2.0 over HTTP
//! - Endpoint discovered via capability-based resolution at runtime
//! - Falls back gracefully when permanent storage unavailable
//! - **Method negotiation**: tries native method names (`commit.session`, `commit.verify`)
//!   first, falling back to compatibility names (`permanent-storage.commitSession`) for
//!   older `LoamSpine` versions. Negotiation result is cached per client instance.

mod wire_types;

use crate::dehydration::DehydrationSummary;
use crate::error::{Result, RhizoCryptError};
use crate::integration::PermanentStorageProvider;
use crate::session::CommitRef;
use crate::slice::{ResolutionOutcome, Slice, SliceOrigin};
use crate::types::Did;
use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicU64, Ordering};

use wire_types::{
    CheckoutSliceRequest, CheckoutSliceResponse, CommitSessionRequest, CommitSessionResponse,
    EmptyParams, HealthCheckResponse, JsonRpcRequest, JsonRpcResponse, MethodSupport,
    NegotiableError, RpcDehydrationSummary,
};

#[cfg(test)]
use wire_types::{JsonRpcError, METHOD_NOT_FOUND_CODE};

/// Native semantic JSON-RPC method names per the Universal IPC Standard.
///
/// Method naming follows `{domain}.{operation}` where domain is the capability.
/// The client tries native names first, falling back to compat names when the
/// server responds with "method not found" (JSON-RPC error -32601).
mod methods {
    /// Native method names (`LoamSpine` v0.8.0+).
    pub mod native {
        pub const HEALTH_CHECK: &str = "system.health";
        pub const COMMIT_SESSION: &str = "commit.session";
        pub const VERIFY_COMMIT: &str = "commit.verify";
        pub const GET_COMMIT: &str = "commit.get";
        pub const CHECKOUT_SLICE: &str = "slice.checkout";
        pub const RESOLVE_SLICE: &str = "slice.resolve";
    }

    /// Compatibility method names (for `LoamSpine` releases before v0.8.0).
    pub mod compat {
        pub const HEALTH_CHECK: &str = "permanent-storage.healthCheck";
        pub const COMMIT_SESSION: &str = "permanent-storage.commitSession";
        pub const VERIFY_COMMIT: &str = "permanent-storage.verifyCommit";
        pub const GET_COMMIT: &str = "permanent-storage.getCommit";
        pub const CHECKOUT_SLICE: &str = "permanent-storage.checkoutSlice";
        pub const RESOLVE_SLICE: &str = "permanent-storage.resolveSlice";
    }
}

/// HTTP client for `LoamSpine` permanent storage.
///
/// Implements `PermanentStorageProvider` trait using JSON-RPC 2.0 over HTTP.
/// Supports method negotiation: tries native method names first (`LoamSpine` v0.8.0+),
/// falling back to compatibility names for older versions.
#[derive(Debug, Clone)]
pub struct LoamSpineHttpClient {
    /// Base URL for `LoamSpine` JSON-RPC endpoint.
    base_url: String,
    /// HTTP client with timeout (pure Rust — hyper/tower stack).
    client: crate::clients::adapters::http::EcoHttpClient,
    /// Request ID counter for JSON-RPC.
    request_id: std::sync::Arc<AtomicU64>,
    /// Whether the server supports native method names.
    /// `None` = not yet negotiated, `Some(true)` = native, `Some(false)` = compat.
    native_methods: std::sync::Arc<std::sync::atomic::AtomicU8>,
}

impl LoamSpineHttpClient {
    /// Create a new `LoamSpine` HTTP client.
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

        let client = crate::clients::adapters::http::EcoHttpClient::new(
            crate::constants::CONNECTION_TIMEOUT,
        );

        Ok(Self {
            base_url,
            client,
            request_id: std::sync::Arc::new(AtomicU64::new(1)),
            native_methods: std::sync::Arc::new(std::sync::atomic::AtomicU8::new(
                MethodSupport::Unknown.to_u8(),
            )),
        })
    }

    /// Create client from environment variables.
    ///
    /// Checks for `STORAGE_PERMANENT_COMMIT_ENDPOINT` or `PERMANENT_STORAGE_ENDPOINT`
    /// (capability-based). Returns an error if no endpoint is configured.
    /// Primals discover endpoints at runtime — no hardcoded fallbacks.
    ///
    /// # Errors
    ///
    /// Returns an error if no permanent storage endpoint env var is set,
    /// or if [`Self::new`] fails while constructing the HTTP client.
    pub fn from_env() -> Result<Self> {
        use crate::safe_env::CapabilityEnv;

        let endpoint = CapabilityEnv::permanent_commit_endpoint().ok_or_else(|| {
            RhizoCryptError::integration(
                "No permanent storage endpoint configured. \
                 Set STORAGE_PERMANENT_COMMIT_ENDPOINT or PERMANENT_STORAGE_ENDPOINT.",
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
        self.native_methods.store(support.to_u8(), Ordering::Relaxed);
    }

    /// Make a JSON-RPC 2.0 call with method negotiation.
    ///
    /// Tries the native method name first. If the server returns "method not found"
    /// (-32601), retries with the compatibility name and caches the result.
    async fn call_negotiated<T, R>(
        &self,
        native_method: &'static str,
        compat_method: &'static str,
        params: T,
    ) -> Result<R>
    where
        T: Serialize + Send + Sync,
        R: for<'de> Deserialize<'de>,
    {
        let method = self.resolve_method(native_method, compat_method);

        // Fast path: server preference already cached
        if method != native_method {
            return self
                .call_jsonrpc_raw(method, &params)
                .await
                .map_err(NegotiableError::into_rhizo_error);
        }

        // Negotiation path: retry with compat name on method-not-found
        match self.call_jsonrpc_raw(method, &params).await {
            Ok(result) => {
                self.record_support(MethodSupport::Native);
                Ok(result)
            }
            Err(NegotiableError::MethodNotFound) => {
                tracing::info!(
                    native = %native_method,
                    compat = %compat_method,
                    "Native method not supported, falling back to compat"
                );
                self.record_support(MethodSupport::Compat);
                self.call_jsonrpc_raw(compat_method, &params)
                    .await
                    .map_err(NegotiableError::into_rhizo_error)
            }
            Err(e) => Err(e.into_rhizo_error()),
        }
    }

    /// Raw JSON-RPC 2.0 call, returning negotiable errors.
    ///
    /// Validates response protocol version ("2.0") and request/response ID matching.
    async fn call_jsonrpc_raw<T, R>(
        &self,
        method: &str,
        params: &T,
    ) -> std::result::Result<R, NegotiableError>
    where
        T: Serialize + Sync,
        R: for<'de> Deserialize<'de>,
    {
        let request_id = self.next_request_id();
        let request = JsonRpcRequest {
            jsonrpc: crate::constants::JSONRPC_VERSION,
            method,
            params,
            id: request_id,
        };

        tracing::debug!(
            method = %method,
            url = %self.base_url,
            "Calling permanent storage JSON-RPC"
        );

        let body = serde_json::to_string(&request).map_err(|e| {
            NegotiableError::Other(RhizoCryptError::integration(format!(
                "Failed to serialize request: {e}"
            )))
        })?;

        let (status, text) = self.client.post_json(&self.base_url, &body).await.map_err(|e| {
            NegotiableError::Other(RhizoCryptError::integration(format!(
                "HTTP request failed: {e}"
            )))
        })?;

        if !(200..300).contains(&status) {
            return Err(NegotiableError::Other(RhizoCryptError::integration(format!(
                "Permanent storage returned HTTP {status}: {text}"
            ))));
        }

        let json_response: JsonRpcResponse<R> = serde_json::from_str(&text).map_err(|e| {
            NegotiableError::Other(RhizoCryptError::integration(format!(
                "Failed to parse response: {e}"
            )))
        })?;

        json_response.into_result(request_id)
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
    async fn commit(&self, summary: &DehydrationSummary) -> Result<CommitRef> {
        let request = CommitSessionRequest {
            session_id: summary.session_id.to_string(),
            merkle_root: hex::encode(summary.merkle_root.as_bytes()),
            summary: RpcDehydrationSummary {
                session_type: summary.session_type.clone(),
                vertex_count: summary.vertex_count,
                leaf_count: u64::try_from(summary.results.len()).unwrap_or(u64::MAX),
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
                "Permanent storage provider rejected commit: {}",
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

        let index = response.entry_index.unwrap_or(0);

        Ok(CommitRef {
            spine_id,
            entry_hash: hash_bytes,
            index,
        })
    }

    async fn verify_commit(&self, commit_ref: &CommitRef) -> Result<bool> {
        #[derive(Debug, Clone, Serialize)]
        struct VerifyRequest {
            spine_id: String,
            entry_hash: String,
            index: u64,
        }

        let request = VerifyRequest {
            spine_id: commit_ref.spine_id.as_str().to_owned(),
            entry_hash: hex::encode(commit_ref.entry_hash),
            index: commit_ref.index,
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
                tracing::debug!(
                    error = %e,
                    "Verification endpoint not available, falling back to health check"
                );
                match self.health_check().await {
                    Ok(hc) => {
                        tracing::debug!(
                            status = %hc.status,
                            version = %hc.version,
                            spine_count = hc.spine_count,
                            "Health check response from permanent storage provider"
                        );
                        Ok(hc.is_healthy())
                    }
                    Err(_) => Ok(false),
                }
            }
        }
    }

    async fn get_commit(&self, commit_ref: &CommitRef) -> Result<Option<DehydrationSummary>> {
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
            spine_id: commit_ref.spine_id.as_str().to_owned(),
            entry_hash: hex::encode(commit_ref.entry_hash),
            index: commit_ref.index,
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
                tracing::debug!(
                    error = %e,
                    "Unable to retrieve commit (endpoint may not exist yet)"
                );
                Ok(None)
            }
        }
    }

    async fn checkout_slice(
        &self,
        spine_id: &str,
        entry_hash: &[u8; 32],
        holder: &Did,
    ) -> Result<SliceOrigin> {
        let request = CheckoutSliceRequest {
            spine_id: spine_id.to_string(),
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
            entry_hash: *entry_hash,
            entry_index: response.entry_index,
            certificate_id: response.certificate_id,
            owner: Did::new(response.owner_did),
        })
    }

    async fn resolve_slice(&self, slice: &Slice, outcome: &ResolutionOutcome) -> Result<()> {
        #[derive(Debug, Clone, Serialize)]
        struct ResolveSliceRequest {
            slice_id: String,
            spine_id: String,
            entry_hash: String,
            outcome: String,
            route: String,
        }

        let request = ResolveSliceRequest {
            slice_id: slice.id.to_string(),
            spine_id: slice.origin.spine_id.as_str().to_owned(),
            entry_hash: hex::encode(slice.origin.entry_hash),
            outcome: format!("{outcome:?}"),
            route: "return_to_origin".to_string(),
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
                    slice_id = %slice.id,
                    spine_id = %slice.origin.spine_id,
                    "Slice resolved successfully"
                );
                Ok(())
            }
            Err(e) => {
                tracing::warn!(
                    slice_id = %slice.id,
                    error = %e,
                    "Slice resolution endpoint not available (proceeding anyway)"
                );
                Ok(())
            }
        }
    }
}

#[cfg(test)]
#[path = "../loamspine_http_tests.rs"]
mod tests;

#[cfg(test)]
#[path = "../loamspine_http_tests_wiremock.rs"]
mod tests_wiremock;
