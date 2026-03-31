// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2024–2026 ecoPrimals Project

//! Capability provider trait contracts.
//!
//! Pure interfaces for external capability providers. These traits are
//! agnostic to which primal (or non-primal service) implements them —
//! request **capabilities**, not **vendors**.

use crate::dehydration::{Attestation, DehydrationSummary};
use crate::error::Result;
use crate::session::CommitRef;
use crate::slice::{ResolutionOutcome, Slice, SliceOrigin};
use crate::types::{Did, PayloadRef, Signature};
use crate::vertex::Vertex;

// ============================================================================
// Signing Provider (Identity & Cryptographic Signing)
// ============================================================================

/// Generic signing provider interface — works with ANY signing service.
///
/// Implemented by services providing cryptographic signing capabilities:
/// hardware security keys, software signing services, HSMs, etc.
///
/// ## Philosophy
///
/// Request **capabilities**, not **vendors**:
/// - ✅ "I need crypto:signing capability"
/// - ❌ "I need BearDog"
///
/// ## Discovery
///
/// Providers are discovered at runtime via capability queries:
///
/// ```no_run
/// # use rhizo_crypt_core::clients::capabilities::SigningClient;
/// # use std::sync::Arc;
/// # tokio::runtime::Runtime::new().unwrap().block_on(async {
/// # let registry = Arc::new(rhizo_crypt_core::discovery::DiscoveryRegistry::new("doc-test"));
/// # registry.register_endpoint(rhizo_crypt_core::discovery::ServiceEndpoint::new(
/// #     "test-signer", "127.0.0.1:9500".parse().unwrap(),
/// #     vec![rhizo_crypt_core::discovery::Capability::Signing],
/// # )).await;
/// let signer = SigningClient::discover(&registry).await?;
/// # Ok::<(), rhizo_crypt_core::error::RhizoCryptError>(())
/// # });
/// ```
pub trait SigningProvider: Send + Sync {
    /// Resolve a DID to verify it exists and is active.
    fn verify_did(&self, did: &Did) -> impl std::future::Future<Output = Result<bool>> + Send;

    /// Sign data with a specific DID.
    fn sign(
        &self,
        data: &[u8],
        signer: &Did,
    ) -> impl std::future::Future<Output = Result<Signature>> + Send;

    /// Sign a vertex.
    fn sign_vertex(
        &self,
        vertex: &Vertex,
        signer: &Did,
    ) -> impl std::future::Future<Output = Result<Signature>> + Send;

    /// Verify a signature.
    fn verify_signature(
        &self,
        data: &[u8],
        signature: &Signature,
        signer: &Did,
    ) -> impl std::future::Future<Output = Result<bool>> + Send;

    /// Verify a vertex signature.
    fn verify_vertex_signature(
        &self,
        vertex: &Vertex,
    ) -> impl std::future::Future<Output = Result<bool>> + Send;

    /// Request an attestation from a party.
    fn request_attestation(
        &self,
        attester: &Did,
        summary: &DehydrationSummary,
    ) -> impl std::future::Future<Output = Result<Attestation>> + Send;
}

// ============================================================================
// Permanent Storage Provider (Commit & Slice Management)
// ============================================================================

/// Generic permanent storage provider interface — works with ANY commit service.
///
/// Implemented by services providing permanent commit capabilities:
/// append-only DAG stores, distributed permanent storage, blockchain-based
/// storage, etc.
///
/// ## Philosophy
///
/// Request **capabilities**, not **vendors**:
/// - ✅ "I need permanent commit capability"
/// - ❌ "I need LoamSpine"
///
/// ## Discovery
///
/// Providers are discovered at runtime via capability queries:
///
/// ```no_run
/// # use rhizo_crypt_core::clients::capabilities::PermanentStorageClient;
/// # use std::sync::Arc;
/// # tokio::runtime::Runtime::new().unwrap().block_on(async {
/// # let registry = Arc::new(rhizo_crypt_core::discovery::DiscoveryRegistry::new("doc-test"));
/// # registry.register_endpoint(rhizo_crypt_core::discovery::ServiceEndpoint::new(
/// #     "test-store", "127.0.0.1:9700".parse().unwrap(),
/// #     vec![rhizo_crypt_core::discovery::Capability::PermanentCommit],
/// # )).await;
/// let storage = PermanentStorageClient::discover(&registry).await?;
/// # Ok::<(), rhizo_crypt_core::error::RhizoCryptError>(())
/// # });
/// ```
pub trait PermanentStorageProvider: Send + Sync {
    /// Commit a dehydration summary to permanent storage.
    fn commit(
        &self,
        summary: &DehydrationSummary,
    ) -> impl std::future::Future<Output = Result<CommitRef>> + Send;

    /// Verify a commit exists.
    fn verify_commit(
        &self,
        commit_ref: &CommitRef,
    ) -> impl std::future::Future<Output = Result<bool>> + Send;

    /// Get a commit by reference.
    fn get_commit(
        &self,
        commit_ref: &CommitRef,
    ) -> impl std::future::Future<Output = Result<Option<DehydrationSummary>>> + Send;

    /// Check out a slice from permanent storage.
    fn checkout_slice(
        &self,
        spine_id: &str,
        entry_hash: &[u8; 32],
        holder: &Did,
    ) -> impl std::future::Future<Output = Result<SliceOrigin>> + Send;

    /// Resolve a slice back to permanent storage.
    fn resolve_slice(
        &self,
        slice: &Slice,
        outcome: &ResolutionOutcome,
    ) -> impl std::future::Future<Output = Result<()>> + Send;
}

// ============================================================================
// Payload Storage Provider (Content-Addressed Storage)
// ============================================================================

/// Generic payload storage provider interface — works with ANY content-addressed store.
///
/// Implemented by services providing content-addressed payload storage:
/// distributed file systems, S3-compatible storage with content addressing,
/// local file storage with Blake3 addressing, DHTs, etc.
///
/// ## Philosophy
///
/// Request **capabilities**, not **vendors**:
/// - ✅ "I need payload storage capability"
/// - ❌ "I need NestGate"
///
/// ## Discovery
///
/// Providers are discovered at runtime via capability queries:
///
/// ```no_run
/// # use rhizo_crypt_core::clients::capabilities::StorageClient;
/// # use std::sync::Arc;
/// # tokio::runtime::Runtime::new().unwrap().block_on(async {
/// # let registry = Arc::new(rhizo_crypt_core::discovery::DiscoveryRegistry::new("doc-test"));
/// # registry.register_endpoint(rhizo_crypt_core::discovery::ServiceEndpoint::new(
/// #     "test-store", "127.0.0.1:9600".parse().unwrap(),
/// #     vec![rhizo_crypt_core::discovery::Capability::PayloadStorage],
/// # )).await;
/// let storage = StorageClient::discover(&registry).await?;
/// # Ok::<(), rhizo_crypt_core::error::RhizoCryptError>(())
/// # });
/// ```
pub trait PayloadStorageProvider: Send + Sync {
    /// Store a payload (returns content-addressed reference).
    fn put_payload(
        &self,
        data: bytes::Bytes,
    ) -> impl std::future::Future<Output = Result<PayloadRef>> + Send;

    /// Get a payload by its content-address.
    fn get_payload(
        &self,
        payload_ref: &PayloadRef,
    ) -> impl std::future::Future<Output = Result<Option<bytes::Bytes>>> + Send;

    /// Check if payload exists at content-address.
    fn payload_exists(
        &self,
        payload_ref: &PayloadRef,
    ) -> impl std::future::Future<Output = Result<bool>> + Send;
}
