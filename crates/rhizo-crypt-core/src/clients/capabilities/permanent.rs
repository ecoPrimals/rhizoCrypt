//! Generic permanent storage client - works with ANY permanent storage provider.
//!
//! This client provides permanent/immutable storage commits without knowing
//! or caring about who provides the service (LoamSpine, blockchain, etc.).

use crate::clients::adapters::{AdapterFactory, ProtocolAdapter, ProtocolAdapterExt};
use crate::dehydration::DehydrationSummary;
use crate::discovery::{Capability, DiscoveryRegistry};
use crate::error::{Result, RhizoCryptError};
use crate::session::LoamCommitRef;
use crate::slice::{ResolutionOutcome, Slice, SliceOrigin};
use crate::types::Did;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

/// Generic permanent storage client - works with ANY provider.
#[derive(Debug, Clone)]
pub struct PermanentStorageClient {
    adapter: Arc<Box<dyn ProtocolAdapter>>,
    endpoint: String,
    service_name: Option<String>,
}

impl PermanentStorageClient {
    /// Discover and connect to ANY permanent storage provider.
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
                return Err(RhizoCryptError::integration("No permanent storage provider available."));
            }
        };

        let service_name = Some(endpoint.service_id.as_ref().to_string());
        let endpoint_addr = endpoint.addr.to_string();

        tracing::info!(
            service = ?service_name,
            endpoint = %endpoint_addr,
            "✅ Discovered permanent storage provider"
        );

        let adapter = AdapterFactory::create(&endpoint_addr)?;

        Ok(Self {
            adapter: Arc::new(adapter),
            endpoint: endpoint_addr,
            service_name,
        })
    }

    /// Create client with explicit endpoint.
    pub fn with_endpoint(endpoint: &str) -> Result<Self> {
        let adapter = AdapterFactory::create(endpoint)?;

        Ok(Self {
            adapter: Arc::new(adapter),
            endpoint: endpoint.to_string(),
            service_name: None,
        })
    }

    /// Commit dehydration summary to permanent storage.
    pub async fn commit(&self, summary: &DehydrationSummary) -> Result<LoamCommitRef> {
        tracing::debug!("Committing dehydration summary to permanent storage");

        let request = CommitRequest {
            summary: summary.clone(),
        };

        let response: CommitResponse = self.adapter.call("commit", request).await?;

        Ok(response.commit_ref)
    }

    /// Verify a commit exists and is valid.
    pub async fn verify_commit(&self, commit_ref: &LoamCommitRef) -> Result<bool> {
        let request = VerifyCommitRequest {
            commit_ref: commit_ref.clone(),
        };

        let response: VerifyCommitResponse = self.adapter.call("verify_commit", request).await?;

        Ok(response.valid)
    }

    /// Get a commit by reference.
    pub async fn get_commit(
        &self,
        commit_ref: &LoamCommitRef,
    ) -> Result<Option<DehydrationSummary>> {
        let request = GetCommitRequest {
            commit_ref: commit_ref.clone(),
        };

        let response: GetCommitResponse = self.adapter.call("get_commit", request).await?;

        Ok(response.summary)
    }

    /// Checkout a slice from permanent storage.
    pub async fn checkout_slice(
        &self,
        spine_id: &str,
        entry_hash: &[u8; 32],
        holder: &Did,
    ) -> Result<SliceOrigin> {
        let request = CheckoutSliceRequest {
            spine_id: spine_id.to_string(),
            entry_hash: *entry_hash,
            holder: holder.clone(),
        };

        let response: CheckoutSliceResponse =
            self.adapter.call("checkout_slice", request).await?;

        Ok(response.origin)
    }

    /// Resolve a slice back to permanent storage.
    pub async fn resolve_slice(&self, slice: &Slice, outcome: &ResolutionOutcome) -> Result<()> {
        let request = ResolveSliceRequest {
            slice: slice.clone(),
            outcome: outcome.clone(),
        };

        let _response: ResolveSliceResponse =
            self.adapter.call("resolve_slice", request).await?;

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

#[derive(Debug, Clone, Serialize, Deserialize)]
struct CommitRequest {
    summary: DehydrationSummary,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct CommitResponse {
    commit_ref: LoamCommitRef,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct VerifyCommitRequest {
    commit_ref: LoamCommitRef,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct VerifyCommitResponse {
    valid: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct GetCommitRequest {
    commit_ref: LoamCommitRef,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct GetCommitResponse {
    summary: Option<DehydrationSummary>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct CheckoutSliceRequest {
    spine_id: String,
    entry_hash: [u8; 32],
    holder: Did,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct CheckoutSliceResponse {
    origin: SliceOrigin,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ResolveSliceRequest {
    slice: Slice,
    outcome: ResolutionOutcome,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ResolveSliceResponse {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_permanent_storage_client_with_endpoint() {
        let client = PermanentStorageClient::with_endpoint("http://localhost:9700").unwrap();
        assert_eq!(client.endpoint(), "http://localhost:9700");
    }
}

