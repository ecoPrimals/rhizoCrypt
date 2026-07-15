// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2024–2026 ecoPrimals Project

//! Generic permanent storage client - works with ANY permanent storage provider.
//!
//! This client provides permanent/immutable storage commits without knowing
//! or caring about who provides the service.

use crate::clients::adapters::{AdapterFactory, ProtocolAdapter, ProtocolAdapterExt};
use crate::dehydration::DehydrationSummary;
use crate::discovery::{Capability, DiscoveryRegistry};
use crate::error::{Result, RhizoCryptError};
use crate::session::CommitRef;
use crate::slice::{ResolutionOutcome, Slice, SliceOrigin};
use crate::types::Did;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

/// Wire method names for permanent storage JSON-RPC calls.
///
/// Aligned with `LoamSpine`'s native method negotiation:
/// `commit.session`, `commit.verify`, `commit.get`, `slice.checkout`, `slice.resolve`.
mod wire {
    pub const COMMIT_SESSION: &str = "commit.session";
    pub const COMMIT_VERIFY: &str = "commit.verify";
    pub const COMMIT_GET: &str = "commit.get";
    pub const SLICE_CHECKOUT: &str = "slice.checkout";
    pub const SLICE_RESOLVE: &str = "slice.resolve";
}

/// Generic permanent storage client - works with ANY provider.
#[derive(Debug, Clone)]
pub struct PermanentStorageClient {
    adapter: Arc<dyn ProtocolAdapter>,
    endpoint: String,
    service_name: Option<String>,
}

impl PermanentStorageClient {
    /// Discover and connect to ANY permanent storage provider.
    ///
    /// # Errors
    ///
    /// Returns an error if no provider advertises `PermanentCommit` or the
    /// adapter cannot be created for the discovered endpoint.
    pub async fn discover(registry: &DiscoveryRegistry) -> Result<Self> {
        tracing::info!("🔍 Discovering permanent storage capability provider...");

        let status = registry.discover(&Capability::PermanentCommit).await;

        let endpoint = match status {
            crate::discovery::DiscoveryStatus::Available(endpoints) => {
                endpoints.into_iter().next().ok_or_else(|| {
                    RhizoCryptError::integration("No permanent storage providers in available list")
                })?
            }
            _ => {
                return Err(RhizoCryptError::integration(
                    "No permanent storage provider available.",
                ));
            }
        };

        let service_name = Some(endpoint.service_id.as_ref().to_string());
        let endpoint_addr = endpoint.endpoint.to_string();

        tracing::info!(
            service = ?service_name,
            endpoint = %endpoint_addr,
            "✅ Discovered permanent storage provider"
        );

        let adapter = AdapterFactory::create(&endpoint_addr)?;

        Ok(Self {
            adapter: Arc::from(adapter),
            endpoint: endpoint_addr,
            service_name,
        })
    }

    /// Create client with explicit endpoint.
    ///
    /// # Errors
    ///
    /// Returns an error if the endpoint address is invalid or unsupported.
    pub fn with_endpoint(endpoint: &str) -> Result<Self> {
        let adapter = AdapterFactory::create(endpoint)?;

        Ok(Self {
            adapter: Arc::from(adapter),
            endpoint: endpoint.to_string(),
            service_name: None,
        })
    }

    /// Commit dehydration summary to permanent storage.
    ///
    /// # Errors
    ///
    /// Returns an error if the RPC call fails or the provider rejects the commit.
    pub async fn commit(&self, summary: &DehydrationSummary) -> Result<CommitRef> {
        tracing::debug!("Committing dehydration summary to permanent storage");

        let request = CommitRequestRef {
            summary,
        };

        let response: CommitResponse = self.adapter.call(wire::COMMIT_SESSION, request).await?;

        Ok(response.commit_ref)
    }

    /// Verify a commit exists and is valid.
    ///
    /// # Errors
    ///
    /// Returns an error if the RPC call fails.
    pub async fn verify_commit(&self, commit_ref: &CommitRef) -> Result<bool> {
        let request = VerifyCommitRequestRef {
            commit_ref,
        };

        let response: VerifyCommitResponse =
            self.adapter.call(wire::COMMIT_VERIFY, request).await?;

        Ok(response.valid)
    }

    /// Get a commit by reference.
    ///
    /// # Errors
    ///
    /// Returns an error if the RPC call fails.
    pub async fn get_commit(&self, commit_ref: &CommitRef) -> Result<Option<DehydrationSummary>> {
        let request = GetCommitRequestRef {
            commit_ref,
        };

        let response: GetCommitResponse = self.adapter.call(wire::COMMIT_GET, request).await?;

        Ok(response.summary)
    }

    /// Checkout a slice from permanent storage.
    ///
    /// # Errors
    ///
    /// Returns an error if the RPC call fails or the slice is unavailable.
    pub async fn checkout_slice(
        &self,
        spine_id: &str,
        entry_hash: &[u8; 32],
        holder: &Did,
    ) -> Result<SliceOrigin> {
        let request = CheckoutSliceRequestRef {
            spine_id,
            entry_hash: *entry_hash,
            holder,
        };

        let response: CheckoutSliceResponse =
            self.adapter.call(wire::SLICE_CHECKOUT, request).await?;

        Ok(response.origin)
    }

    /// Resolve a slice back to permanent storage.
    ///
    /// # Errors
    ///
    /// Returns an error if the RPC call fails.
    pub async fn resolve_slice(&self, slice: &Slice, outcome: &ResolutionOutcome) -> Result<()> {
        let request = ResolveSliceRequestRef {
            slice,
            outcome,
        };

        let _response: ResolveSliceResponse =
            self.adapter.call(wire::SLICE_RESOLVE, request).await?;

        Ok(())
    }

    /// Check if service is available.
    pub async fn is_available(&self) -> bool {
        self.adapter.is_healthy().await
    }

    /// Get service endpoint.
    #[must_use]
    pub fn endpoint(&self) -> &str {
        &self.endpoint
    }

    /// Get service name (if known).
    #[must_use]
    pub fn service_name(&self) -> Option<&str> {
        self.service_name.as_deref()
    }
}

// ============================================================================
// Request/Response DTOs
// ============================================================================

#[derive(Serialize)]
struct CommitRequestRef<'a> {
    summary: &'a DehydrationSummary,
}

#[derive(Serialize)]
struct VerifyCommitRequestRef<'a> {
    commit_ref: &'a CommitRef,
}

#[derive(Serialize)]
struct GetCommitRequestRef<'a> {
    commit_ref: &'a CommitRef,
}

#[derive(Serialize)]
struct CheckoutSliceRequestRef<'a> {
    spine_id: &'a str,
    entry_hash: [u8; 32],
    holder: &'a Did,
}

#[derive(Serialize)]
struct ResolveSliceRequestRef<'a> {
    slice: &'a Slice,
    outcome: &'a ResolutionOutcome,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct CommitResponse {
    commit_ref: CommitRef,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct VerifyCommitResponse {
    valid: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct GetCommitResponse {
    summary: Option<DehydrationSummary>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct CheckoutSliceResponse {
    origin: SliceOrigin,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ResolveSliceResponse {}

#[cfg(test)]
#[path = "permanent_tests.rs"]
mod tests;
