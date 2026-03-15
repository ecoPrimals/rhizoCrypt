// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2024–2026 ecoPrimals Project

//! Discovery and client failure tests for rhizoCrypt.
//!
//! Tests system behavior when discovery fails or clients are unavailable.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use rhizo_crypt_core::{
    Capability, ClientFactory, ClientProvider, DiscoveryRegistry, DiscoveryStatus,
    IntegrationStatus, ServiceEndpoint, ServiceStatus,
};
use std::sync::Arc;
use std::time::Duration;

/// Test discovery with no registered services.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_discovery_no_services() {
    let registry = DiscoveryRegistry::new("rhizoCrypt");

    // All capabilities should be unavailable
    let status = registry.discover(&Capability::Signing).await;
    assert!(!status.is_available());
    assert!(status.first_endpoint().is_none());

    let status = registry.discover(&Capability::PayloadStorage).await;
    assert!(!status.is_available());

    let status = registry.discover(&Capability::PermanentCommit).await;
    assert!(!status.is_available());
}

/// Test discovery fallback when discovery source is not set.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_discovery_no_source() {
    let registry = DiscoveryRegistry::new("rhizoCrypt");

    // Without a discovery source, should return Unavailable
    let status = registry.discover(&Capability::ServiceDiscovery).await;
    matches!(status, DiscoveryStatus::Unavailable);
}

/// Test client provider graceful degradation.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_client_provider_unavailable() {
    let registry = Arc::new(DiscoveryRegistry::new("rhizoCrypt"));
    let provider = ClientProvider::new(Arc::clone(&registry));

    assert!(!provider.has_signing().await);
    assert!(!provider.has_permanent_storage().await);
    assert!(!provider.has_payload_storage().await);

    assert!(provider.signing_endpoint().await.is_err());
    assert!(provider.permanent_storage_endpoint().await.is_err());
    assert!(provider.payload_storage_endpoint().await.is_err());
}

/// Test client factory graceful degradation.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_client_factory_unavailable() {
    let registry = Arc::new(DiscoveryRegistry::new("rhizoCrypt"));
    let factory = ClientFactory::new(Arc::clone(&registry));

    // All capabilities should be unavailable
    assert!(!factory.has_signing_capability().await);
    assert!(!factory.has_commit_capability().await);
    assert!(!factory.has_storage_capability().await);

    // Endpoint requests should error
    assert!(factory.signing_endpoint().await.is_err());
    assert!(factory.commit_endpoint().await.is_err());
    assert!(factory.storage_endpoint().await.is_err());

    // Integration status should reflect unavailability
    let status = factory.integration_status().await;
    assert!(!status.all_healthy());
}

/// Test partial service availability.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_partial_service_availability() {
    let registry = Arc::new(DiscoveryRegistry::new("rhizoCrypt"));
    let factory = ClientFactory::new(Arc::clone(&registry));

    // Register only BearDog
    registry
        .register_endpoint(ServiceEndpoint::new(
            "bearDog",
            "127.0.0.1:9000".parse().unwrap(),
            vec![Capability::Signing, Capability::DidVerification],
        ))
        .await;

    // BearDog should be available
    assert!(factory.has_signing_capability().await);
    assert!(factory.signing_endpoint().await.is_ok());

    // Others should not be
    assert!(!factory.has_commit_capability().await);
    assert!(!factory.has_storage_capability().await);
    assert!(factory.commit_endpoint().await.is_err());
    assert!(factory.storage_endpoint().await.is_err());

    // Integration status should show partial health
    let status = factory.integration_status().await;
    assert!(!status.all_healthy()); // Not all healthy
    assert!(status.signing.is_healthy()); // Signing capability is healthy
}

/// Test service registration and deregistration.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_service_lifecycle() {
    let registry = Arc::new(DiscoveryRegistry::new("rhizoCrypt"));

    // Initially no services
    let endpoints = registry.all_endpoints().await;
    assert!(endpoints.is_empty());

    // Register services
    registry
        .register_endpoint(ServiceEndpoint::new(
            "bearDog",
            "127.0.0.1:9000".parse().unwrap(),
            vec![Capability::Signing],
        ))
        .await;

    registry
        .register_endpoint(ServiceEndpoint::new(
            "nestGate",
            "127.0.0.1:9001".parse().unwrap(),
            vec![Capability::PayloadStorage],
        ))
        .await;

    // Should have endpoints now
    let endpoints = registry.all_endpoints().await;
    assert!(!endpoints.is_empty());

    // Both capabilities should be available
    assert!(registry.is_available(&Capability::Signing).await);
    assert!(registry.is_available(&Capability::PayloadStorage).await);
}

/// Test discovery with multiple endpoints for same capability.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_multiple_providers() {
    let registry = DiscoveryRegistry::new("rhizoCrypt");

    // Register multiple signing services
    for i in 0..3 {
        registry
            .register_endpoint(ServiceEndpoint::new(
                format!("bearDog-{i}"),
                format!("127.0.0.1:900{i}").parse().unwrap(),
                vec![Capability::Signing],
            ))
            .await;
    }

    // Discovery should return all
    let status = registry.discover(&Capability::Signing).await;
    match status {
        DiscoveryStatus::Available(endpoints) => {
            assert_eq!(endpoints.len(), 3);
        }
        _ => panic!("Expected Available status"),
    }

    // get_endpoint should return one
    let endpoint = registry.get_endpoint(&Capability::Signing).await;
    assert!(endpoint.is_some());
}

/// Test integration status variations.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_integration_status_variations() {
    // All healthy
    let mut status = IntegrationStatus::new();
    status.signing = ServiceStatus::healthy("127.0.0.1:9000");
    status.permanent_storage = ServiceStatus::healthy("127.0.0.1:9001");
    status.payload_storage = ServiceStatus::healthy("127.0.0.1:9002");
    assert!(status.all_healthy());
    assert!(!status.any_unavailable());

    // One unhealthy
    let mut status = IntegrationStatus::new();
    status.signing = ServiceStatus::healthy("127.0.0.1:9000");
    status.permanent_storage = ServiceStatus::unhealthy("127.0.0.1:9001", "connection refused");
    status.payload_storage = ServiceStatus::healthy("127.0.0.1:9002");
    assert!(!status.all_healthy()); // Not all healthy
    assert!(!status.any_unavailable()); // But none unavailable

    // One unavailable
    let mut status = IntegrationStatus::new();
    status.signing = ServiceStatus::healthy("127.0.0.1:9000");
    status.permanent_storage = ServiceStatus::unavailable("no provider");
    status.payload_storage = ServiceStatus::healthy("127.0.0.1:9002");
    assert!(!status.all_healthy());
    assert!(status.any_unavailable());

    // Summary output
    assert!(status.summary().contains("unavailable"));
}

/// Test discovery source setting.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_discovery_source() {
    let registry = DiscoveryRegistry::new("rhizoCrypt");

    // Set discovery source
    let songbird_addr = "127.0.0.1:8091".parse().unwrap();
    registry.set_discovery_source(songbird_addr).await;

    // Discovery still returns unavailable (no cache, no real Songbird)
    // but the code path through discovery source check is exercised
    let status = registry.discover(&Capability::ServiceDiscovery).await;
    assert!(!status.is_available());
}

/// Test custom capability.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_custom_capability() {
    let registry = DiscoveryRegistry::new("rhizoCrypt");

    let custom_cap = Capability::custom("myapp:feature");

    // Register with custom capability
    registry
        .register_endpoint(ServiceEndpoint::new(
            "customService",
            "127.0.0.1:9999".parse().unwrap(),
            vec![custom_cap.clone()],
        ))
        .await;

    // Should be discoverable
    assert!(registry.is_available(&custom_cap).await);
    let endpoint = registry.get_endpoint(&custom_cap).await;
    assert!(endpoint.is_some());
    assert_eq!(endpoint.unwrap().service_id.as_ref(), "customService");
}
