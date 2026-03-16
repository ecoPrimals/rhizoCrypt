// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2024–2026 ecoPrimals Project

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
                return Err(RhizoCryptError::integration(
                    "No permanent storage provider available.",
                ));
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

        let response: CheckoutSliceResponse = self.adapter.call("checkout_slice", request).await?;

        Ok(response.origin)
    }

    /// Resolve a slice back to permanent storage.
    pub async fn resolve_slice(&self, slice: &Slice, outcome: &ResolutionOutcome) -> Result<()> {
        let request = ResolveSliceRequest {
            slice: slice.clone(),
            outcome: outcome.clone(),
        };

        let _response: ResolveSliceResponse = self.adapter.call("resolve_slice", request).await?;

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
#[expect(clippy::unwrap_used, reason = "test code")]
mod tests {
    use super::*;
    use crate::dehydration::DehydrationSummary;
    use crate::discovery::{DiscoveryRegistry, ServiceEndpoint};
    use crate::event::SessionOutcome;
    use crate::merkle::MerkleRoot;
    use crate::slice::{ResolutionOutcome, SliceOrigin};
    use crate::types::{ContentHash, Did, SessionId, Timestamp, VertexId};
    use std::net::SocketAddr;

    fn make_loam_commit_ref() -> LoamCommitRef {
        LoamCommitRef {
            spine_id: "test-spine".to_string(),
            entry_hash: [1u8; 32],
            index: 42,
        }
    }

    fn make_dehydration_summary() -> DehydrationSummary {
        DehydrationSummary {
            session_id: SessionId::new(uuid::Uuid::now_v7()),
            session_type: "test".to_string(),
            created_at: Timestamp::now(),
            resolved_at: Timestamp::now(),
            outcome: SessionOutcome::Success,
            merkle_root: MerkleRoot::new(ContentHash::from([0u8; 32])),
            vertex_count: 5,
            payload_bytes: 100,
            results: vec![],
            agents: vec![],
            attestations: vec![],
        }
    }

    fn make_slice_origin() -> SliceOrigin {
        SliceOrigin {
            spine_id: "test-spine".to_string(),
            entry_hash: ContentHash::from([2u8; 32]),
            entry_index: 10,
            certificate_id: None,
            owner: Did::new("did:key:owner"),
        }
    }

    #[test]
    fn test_commit_request_serialization() {
        let summary = make_dehydration_summary();
        let request = CommitRequest {
            summary: summary.clone(),
        };
        let serialized = serde_json::to_string(&request).unwrap();
        let deserialized: CommitRequest = serde_json::from_str(&serialized).unwrap();
        assert_eq!(deserialized.summary.session_type, summary.session_type);
    }

    #[test]
    fn test_commit_response_serialization() {
        let commit_ref = make_loam_commit_ref();
        let response = CommitResponse {
            commit_ref: commit_ref.clone(),
        };
        let serialized = serde_json::to_string(&response).unwrap();
        let deserialized: CommitResponse = serde_json::from_str(&serialized).unwrap();
        assert_eq!(deserialized.commit_ref.spine_id, commit_ref.spine_id);
        assert_eq!(deserialized.commit_ref.index, 42);
    }

    #[test]
    fn test_verify_commit_request_serialization() {
        let commit_ref = make_loam_commit_ref();
        let request = VerifyCommitRequest {
            commit_ref: commit_ref.clone(),
        };
        let serialized = serde_json::to_string(&request).unwrap();
        let deserialized: VerifyCommitRequest = serde_json::from_str(&serialized).unwrap();
        assert_eq!(deserialized.commit_ref.spine_id, commit_ref.spine_id);
    }

    #[test]
    fn test_verify_commit_response_serialization() {
        let response = VerifyCommitResponse {
            valid: true,
        };
        let serialized = serde_json::to_string(&response).unwrap();
        let deserialized: VerifyCommitResponse = serde_json::from_str(&serialized).unwrap();
        assert!(deserialized.valid);

        let response_false = VerifyCommitResponse {
            valid: false,
        };
        let serialized = serde_json::to_string(&response_false).unwrap();
        let deserialized: VerifyCommitResponse = serde_json::from_str(&serialized).unwrap();
        assert!(!deserialized.valid);
    }

    #[test]
    fn test_get_commit_request_serialization() {
        let commit_ref = make_loam_commit_ref();
        let request = GetCommitRequest {
            commit_ref: commit_ref.clone(),
        };
        let serialized = serde_json::to_string(&request).unwrap();
        let deserialized: GetCommitRequest = serde_json::from_str(&serialized).unwrap();
        assert_eq!(deserialized.commit_ref.spine_id, commit_ref.spine_id);
    }

    #[test]
    fn test_get_commit_response_serialization() {
        let response = GetCommitResponse {
            summary: Some(make_dehydration_summary()),
        };
        let serialized = serde_json::to_string(&response).unwrap();
        let deserialized: GetCommitResponse = serde_json::from_str(&serialized).unwrap();
        assert!(deserialized.summary.is_some());

        let response_none = GetCommitResponse {
            summary: None,
        };
        let serialized = serde_json::to_string(&response_none).unwrap();
        let deserialized: GetCommitResponse = serde_json::from_str(&serialized).unwrap();
        assert!(deserialized.summary.is_none());
    }

    #[test]
    fn test_checkout_slice_request_serialization() {
        let request = CheckoutSliceRequest {
            spine_id: "spine-1".to_string(),
            entry_hash: [3u8; 32],
            holder: Did::new("did:key:holder"),
        };
        let serialized = serde_json::to_string(&request).unwrap();
        let deserialized: CheckoutSliceRequest = serde_json::from_str(&serialized).unwrap();
        assert_eq!(deserialized.spine_id, "spine-1");
        assert_eq!(deserialized.entry_hash, [3u8; 32]);
    }

    #[test]
    fn test_checkout_slice_response_serialization() {
        let origin = make_slice_origin();
        let response = CheckoutSliceResponse {
            origin: origin.clone(),
        };
        let serialized = serde_json::to_string(&response).unwrap();
        let deserialized: CheckoutSliceResponse = serde_json::from_str(&serialized).unwrap();
        assert_eq!(deserialized.origin.spine_id, origin.spine_id);
    }

    #[test]
    fn test_resolve_slice_request_serialization() {
        use crate::slice::{SliceBuilder, SliceMode};

        let origin = make_slice_origin();
        let slice = SliceBuilder::new(
            origin,
            Did::new("did:key:holder"),
            SliceMode::Copy {
                allow_recopy: false,
            },
            SessionId::new(uuid::Uuid::now_v7()),
            VertexId::from_bytes(b"checkout"),
        )
        .build();
        let outcome = ResolutionOutcome::ReturnedUnchanged;

        let request = ResolveSliceRequest {
            slice: slice.clone(),
            outcome,
        };
        let serialized = serde_json::to_string(&request).unwrap();
        let deserialized: ResolveSliceRequest = serde_json::from_str(&serialized).unwrap();
        assert_eq!(deserialized.slice.id, slice.id);
    }

    #[test]
    fn test_resolve_slice_response_serialization() {
        let response = ResolveSliceResponse {};
        let serialized = serde_json::to_string(&response).unwrap();
        let _deserialized: ResolveSliceResponse = serde_json::from_str(&serialized).unwrap();
    }

    #[test]
    fn test_permanent_storage_client_invalid_endpoint() {
        let result = PermanentStorageClient::with_endpoint("not a valid endpoint");
        assert!(result.is_err());
    }

    #[test]
    fn test_permanent_storage_client_with_endpoint() {
        let client = PermanentStorageClient::with_endpoint("127.0.0.1:9700").unwrap();
        assert_eq!(client.endpoint(), "127.0.0.1:9700");
    }

    #[cfg(feature = "http-clients")]
    #[test]
    fn test_permanent_storage_client_endpoint_formats() {
        let http_client = PermanentStorageClient::with_endpoint("http://localhost:9700").unwrap();
        assert_eq!(http_client.endpoint(), "http://localhost:9700");

        let auto_http = PermanentStorageClient::with_endpoint("localhost:9700").unwrap();
        assert!(auto_http.endpoint().contains("localhost:9700"));
    }

    #[tokio::test]
    async fn test_permanent_storage_client_discover() {
        let registry = Arc::new(DiscoveryRegistry::new("test-primal"));

        // Register a permanent storage provider
        let addr: SocketAddr = "127.0.0.1:9700".parse().unwrap();
        let endpoint = ServiceEndpoint::new(
            "test-loamspine",
            addr,
            vec![Capability::PermanentCommit, Capability::SliceCheckout],
        );
        registry.register_endpoint(endpoint).await;

        // Discover should find the provider
        let result = PermanentStorageClient::discover(&registry).await;
        assert!(result.is_ok());

        let client = result.unwrap();
        assert!(client.endpoint().contains("127.0.0.1:9700"));
        assert_eq!(client.service_name(), Some("test-loamspine"));
    }

    #[tokio::test]
    async fn test_permanent_storage_client_discover_no_provider() {
        let registry = Arc::new(DiscoveryRegistry::new("test-primal"));

        // No providers registered
        let result = PermanentStorageClient::discover(&registry).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_permanent_storage_client_multiple_providers() {
        let registry = Arc::new(DiscoveryRegistry::new("test-primal"));

        // Register multiple providers
        let addr1: SocketAddr = "127.0.0.1:9700".parse().unwrap();
        let endpoint1 =
            ServiceEndpoint::new("loamspine-1", addr1, vec![Capability::PermanentCommit]);
        registry.register_endpoint(endpoint1).await;

        let addr2: SocketAddr = "127.0.0.1:9701".parse().unwrap();
        let endpoint2 =
            ServiceEndpoint::new("loamspine-2", addr2, vec![Capability::PermanentCommit]);
        registry.register_endpoint(endpoint2).await;

        // Should discover one of them
        let result = PermanentStorageClient::discover(&registry).await;
        assert!(result.is_ok());

        let client = result.unwrap();
        // Should connect to one of the providers
        assert!(
            client.endpoint().contains("127.0.0.1:9700")
                || client.endpoint().contains("127.0.0.1:9701")
        );
    }

    #[test]
    fn test_permanent_storage_client_clone() {
        let client1 = PermanentStorageClient::with_endpoint("127.0.0.1:9700").unwrap();
        let client2 = client1.clone();

        assert_eq!(client1.endpoint(), client2.endpoint());
    }

    #[test]
    fn test_permanent_storage_client_debug() {
        let client = PermanentStorageClient::with_endpoint("127.0.0.1:9700").unwrap();
        let debug_str = format!("{client:?}");
        assert!(debug_str.contains("PermanentStorageClient"));
    }

    #[tokio::test]
    async fn test_permanent_storage_client_concurrent_discovery() {
        let registry = Arc::new(DiscoveryRegistry::new("test-primal"));

        // Register provider
        let addr: SocketAddr = "127.0.0.1:9700".parse().unwrap();
        let endpoint = ServiceEndpoint::new("loamspine", addr, vec![Capability::PermanentCommit]);
        registry.register_endpoint(endpoint).await;

        // Discover concurrently
        let registry1 = registry.clone();
        let registry2 = registry.clone();

        let handle1 =
            tokio::spawn(async move { PermanentStorageClient::discover(&registry1).await });

        let handle2 =
            tokio::spawn(async move { PermanentStorageClient::discover(&registry2).await });

        let result1 = handle1.await.unwrap();
        let result2 = handle2.await.unwrap();

        assert!(result1.is_ok());
        assert!(result2.is_ok());
    }

    #[tokio::test]
    async fn test_permanent_storage_client_discover_with_slice_capability() {
        let registry = Arc::new(DiscoveryRegistry::new("test-primal"));

        // Register provider with both commit and slice capabilities
        let addr: SocketAddr = "127.0.0.1:9700".parse().unwrap();
        let endpoint = ServiceEndpoint::new(
            "full-loamspine",
            addr,
            vec![
                Capability::PermanentCommit,
                Capability::SliceCheckout,
                Capability::SliceResolution,
            ],
        );
        registry.register_endpoint(endpoint).await;

        let result = PermanentStorageClient::discover(&registry).await;
        assert!(result.is_ok());

        let client = result.unwrap();
        assert_eq!(client.service_name(), Some("full-loamspine"));
    }

    #[test]
    fn test_permanent_storage_client_service_name_tracking() {
        // Client with explicit endpoint has no service name
        let client1 = PermanentStorageClient::with_endpoint("127.0.0.1:9700").unwrap();
        assert_eq!(client1.service_name(), None);
    }

    #[test]
    fn test_permanent_storage_client_various_addresses() {
        // Transport-agnostic formats (work with tarpc when http-clients is off)
        let formats = vec!["localhost:9700", "127.0.0.1:9700"];

        for format in formats {
            let result = PermanentStorageClient::with_endpoint(format);
            assert!(result.is_ok(), "Failed for format: {format}");
        }
    }

    #[cfg(feature = "http-clients")]
    #[test]
    fn test_permanent_storage_client_http_endpoint_formats() {
        let formats = vec!["http://localhost:9700", "http://127.0.0.1:9700"];

        for format in formats {
            let result = PermanentStorageClient::with_endpoint(format);
            assert!(result.is_ok(), "Failed for format: {format}");
        }
    }
}
