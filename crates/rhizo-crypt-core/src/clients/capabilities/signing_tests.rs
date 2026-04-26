// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2024–2026 ecoPrimals Project

use super::*;
use crate::discovery::{DiscoveryRegistry, ServiceEndpoint};
use std::net::SocketAddr;

#[test]
fn test_signing_client_with_endpoint() {
    let client = SigningClient::with_endpoint("127.0.0.1:9500").unwrap();
    assert_eq!(client.endpoint(), "127.0.0.1:9500");
    assert!(client.service_name().is_none());
}

#[test]
fn test_signing_client_invalid_endpoint() {
    let result = SigningClient::with_endpoint("not a url");
    assert!(result.is_err());
}

#[tokio::test]
async fn test_signing_client_availability() {
    let client = SigningClient::with_endpoint("127.0.0.1:9999").unwrap();
    let available = client.is_available().await;
    let _ = available;
}

#[tokio::test]
async fn test_signing_client_discover_no_providers() {
    let registry = DiscoveryRegistry::new("test-rhizocrypt");
    let result = SigningClient::discover(&registry).await;
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.to_string().contains("No signing provider available"));
}

#[tokio::test]
async fn test_signing_client_discover_with_provider() {
    let registry = DiscoveryRegistry::new("test-rhizocrypt");

    let addr: SocketAddr = "127.0.0.1:9500".parse().unwrap();
    let endpoint = ServiceEndpoint::new("test-signer".to_string(), addr, vec![Capability::Signing]);
    registry.register_endpoint(endpoint).await;

    let result = SigningClient::discover(&registry).await;
    assert!(result.is_ok());
    let client = result.unwrap();
    assert!(client.endpoint().contains("127.0.0.1:9500"));
    assert_eq!(client.service_name(), Some("test-signer"));
}

// ============================================================================
// Wire DTO serialization tests — BearDog-aligned shapes
// ============================================================================

#[test]
fn test_crypto_sign_request_serialization() {
    let request = CryptoSignRequest {
        message: "AQID".to_string(),
        key_id: Some("did:key:test".to_string()),
    };

    let serialized = serde_json::to_string(&request).unwrap();
    assert!(serialized.contains("\"message\":\"AQID\""));
    assert!(serialized.contains("\"key_id\":\"did:key:test\""));

    let deserialized: CryptoSignRequest = serde_json::from_str(&serialized).unwrap();
    assert_eq!(deserialized.message, "AQID");
    assert_eq!(deserialized.key_id.as_deref(), Some("did:key:test"));
}

#[test]
fn test_crypto_sign_request_no_key_id() {
    let request = CryptoSignRequest {
        message: "dGVzdA==".to_string(),
        key_id: None,
    };

    let serialized = serde_json::to_string(&request).unwrap();
    assert!(!serialized.contains("key_id"));
}

#[test]
fn test_crypto_sign_response_serialization() {
    let response = CryptoSignResponse {
        signature: "AQIDBA==".to_string(),
        algorithm: Some("Ed25519".to_string()),
        key_id: Some("default_signing_key".to_string()),
    };

    let serialized = serde_json::to_string(&response).unwrap();
    let deserialized: CryptoSignResponse = serde_json::from_str(&serialized).unwrap();
    assert_eq!(deserialized.signature, "AQIDBA==");
    assert_eq!(deserialized.algorithm.as_deref(), Some("Ed25519"));
}

#[test]
fn test_crypto_sign_response_minimal() {
    let json = r#"{"signature":"AQID"}"#;
    let response: CryptoSignResponse = serde_json::from_str(json).unwrap();
    assert_eq!(response.signature, "AQID");
    assert!(response.algorithm.is_none());
    assert!(response.key_id.is_none());
}

#[test]
fn test_crypto_verify_request_serialization() {
    let request = CryptoVerifyRequest {
        message: "BQYH".to_string(),
        signature: "CAkK".to_string(),
        public_key: "did:key:verifier".to_string(),
    };

    let serialized = serde_json::to_string(&request).unwrap();
    let deserialized: CryptoVerifyRequest = serde_json::from_str(&serialized).unwrap();
    assert_eq!(deserialized.message, "BQYH");
    assert_eq!(deserialized.signature, "CAkK");
    assert_eq!(deserialized.public_key, "did:key:verifier");
}

#[test]
fn test_crypto_verify_response_serialization() {
    let response = CryptoVerifyResponse {
        valid: true,
        algorithm: Some("Ed25519".to_string()),
    };
    let serialized = serde_json::to_string(&response).unwrap();
    let deserialized: CryptoVerifyResponse = serde_json::from_str(&serialized).unwrap();
    assert!(deserialized.valid);

    let response_false = CryptoVerifyResponse {
        valid: false,
        algorithm: None,
    };
    let serialized = serde_json::to_string(&response_false).unwrap();
    let deserialized: CryptoVerifyResponse = serde_json::from_str(&serialized).unwrap();
    assert!(!deserialized.valid);
}

#[test]
fn test_crypto_sign_contract_request_serialization() {
    let request = CryptoSignContractRequest {
        signer: "did:key:attester".to_string(),
        terms: serde_json::json!({"session_id": "abc-123", "vertex_count": 10}),
        context: Some("dehydration-attestation".to_string()),
    };

    let serialized = serde_json::to_string(&request).unwrap();
    assert!(serialized.contains("\"signer\":\"did:key:attester\""));
    assert!(serialized.contains("\"context\":\"dehydration-attestation\""));
}

#[test]
fn test_crypto_sign_contract_response_serialization() {
    let response = CryptoSignContractResponse {
        terms_hash: "abcdef1234567890".to_string(),
        signature: "deadbeef".repeat(16),
        public_key: "did:key:z6MkContractSigner".to_string(),
        signed_at: "2026-04-15T12:00:00Z".to_string(),
    };

    let serialized = serde_json::to_string(&response).unwrap();
    let deserialized: CryptoSignContractResponse = serde_json::from_str(&serialized).unwrap();
    assert_eq!(deserialized.terms_hash, "abcdef1234567890");
    assert_eq!(deserialized.signed_at, "2026-04-15T12:00:00Z");
}

#[test]
fn test_verify_did_request_serialization() {
    let did = Did::new("did:key:test123");
    let request = VerifyDidRequest {
        did: did.clone(),
    };

    let serialized = serde_json::to_string(&request).unwrap();
    assert!(serialized.contains("did:key:test123"));

    let deserialized: VerifyDidRequest = serde_json::from_str(&serialized).unwrap();
    assert_eq!(deserialized.did, did);
}

#[test]
fn test_verify_did_response_serialization() {
    let response = VerifyDidResponse {
        valid: true,
    };
    let serialized = serde_json::to_string(&response).unwrap();
    let deserialized: VerifyDidResponse = serde_json::from_str(&serialized).unwrap();
    assert!(deserialized.valid);

    let response_false = VerifyDidResponse {
        valid: false,
    };
    let serialized = serde_json::to_string(&response_false).unwrap();
    let deserialized: VerifyDidResponse = serde_json::from_str(&serialized).unwrap();
    assert!(!deserialized.valid);
}

// ============================================================================
// Base64 round-trip tests
// ============================================================================

#[test]
fn test_base64_roundtrip_sign() {
    let data = bytes::Bytes::from_static(&[0xDE, 0xAD, 0xBE, 0xEF]);
    let encoded = B64.encode(&data);
    assert_eq!(encoded, "3q2+7w==");
    let decoded = B64.decode(&encoded).unwrap();
    assert_eq!(decoded, &[0xDE, 0xAD, 0xBE, 0xEF]);
}

// ============================================================================
// Client unit tests
// ============================================================================

#[test]
fn test_signing_client_clone() {
    let client = SigningClient::with_endpoint("127.0.0.1:9500").unwrap();
    let cloned = client.clone();
    assert_eq!(client.endpoint(), cloned.endpoint());
    assert_eq!(client.service_name(), cloned.service_name());
}

#[test]
fn test_signing_client_debug() {
    let client = SigningClient::with_endpoint("127.0.0.1:9500").unwrap();
    let debug_str = format!("{client:?}");
    assert!(debug_str.contains("SigningClient"));
}

#[tokio::test]
async fn test_signing_client_multiple_providers() {
    let registry = DiscoveryRegistry::new("test-rhizocrypt");

    let addr1: SocketAddr = "127.0.0.1:9500".parse().unwrap();
    registry
        .register_endpoint(ServiceEndpoint::new(
            "signer-1".to_string(),
            addr1,
            vec![Capability::Signing],
        ))
        .await;

    let addr2: SocketAddr = "127.0.0.1:9501".parse().unwrap();
    registry
        .register_endpoint(ServiceEndpoint::new(
            "signer-2".to_string(),
            addr2,
            vec![Capability::Signing],
        ))
        .await;

    let result = SigningClient::discover(&registry).await;
    assert!(result.is_ok());
    let client = result.unwrap();
    assert!(
        client.endpoint().contains("127.0.0.1:9500")
            || client.endpoint().contains("127.0.0.1:9501")
    );
}

#[test]
fn test_signature_from_bytes() {
    let sig_bytes = vec![1, 2, 3, 4, 5];
    let signature = Signature::new(sig_bytes.clone());
    assert_eq!(signature.as_bytes(), &sig_bytes);
}

#[tokio::test]
async fn test_signing_client_service_name_tracking() {
    let registry = DiscoveryRegistry::new("test-rhizocrypt");

    let addr: SocketAddr = "127.0.0.1:9500".parse().unwrap();
    let endpoint = ServiceEndpoint::new("beardog-hsm".to_string(), addr, vec![Capability::Signing]);
    registry.register_endpoint(endpoint).await;

    let client = SigningClient::discover(&registry).await.unwrap();
    assert_eq!(client.service_name(), Some("beardog-hsm"));
    assert!(client.endpoint().contains("127.0.0.1:9500"));
}

#[tokio::test]
async fn test_verify_vertex_no_signature_or_agent() {
    use crate::event::EventType;
    use crate::vertex::VertexBuilder;

    let client = SigningClient::with_endpoint("127.0.0.1:9500").unwrap();
    let vertex = VertexBuilder::new(EventType::SessionStart).build();
    assert!(vertex.signature.is_none());
    assert!(vertex.agent.is_none());

    let result = client.verify_vertex(&vertex).await;
    assert!(result.is_ok());
    assert!(!result.unwrap());
}

#[tokio::test]
async fn test_verify_vertex_no_signature_with_agent() {
    use crate::event::EventType;
    use crate::vertex::VertexBuilder;

    let client = SigningClient::with_endpoint("127.0.0.1:9500").unwrap();
    let vertex =
        VertexBuilder::new(EventType::SessionStart).with_agent(Did::new("did:key:test")).build();
    assert!(vertex.signature.is_none());
    assert!(vertex.agent.is_some());

    let result = client.verify_vertex(&vertex).await;
    assert!(result.is_ok());
    assert!(!result.unwrap());
}

#[tokio::test]
async fn test_signing_client_discover_failed() {
    let registry = DiscoveryRegistry::new("test-rhizocrypt");
    registry.set_discovery_source("127.0.0.1:1".parse().unwrap()).await;

    let result = SigningClient::discover(&registry).await;
    assert!(result.is_err());
    let err_msg = result.unwrap_err().to_string();
    assert!(
        err_msg.contains("Discovery failed") || err_msg.contains("No signing provider"),
        "Expected discovery failure, got: {err_msg}"
    );
}

#[test]
#[cfg(feature = "http-clients")]
fn test_signing_client_endpoint_formats() {
    let http_client = SigningClient::with_endpoint("http://localhost:9500").unwrap();
    assert_eq!(http_client.endpoint(), "http://localhost:9500");

    let https_client = SigningClient::with_endpoint("https://signing.example.com:443").unwrap();
    assert_eq!(https_client.endpoint(), "https://signing.example.com:443");

    let auto_http = SigningClient::with_endpoint("localhost:9500").unwrap();
    assert!(auto_http.endpoint().contains("localhost:9500"));
}

// ============================================================================
// Mock adapter tests — exercise sign/verify/attest via BearDog wire format
// ============================================================================

fn mock_client() -> SigningClient {
    use crate::integration::mocks::MockProtocolAdapter;
    let adapter = MockProtocolAdapter::permissive();
    SigningClient::with_adapter(Box::new(adapter), "mock://test:9500")
}

#[tokio::test]
async fn test_sign_via_mock() {
    use crate::integration::mocks::MockProtocolAdapter;
    let adapter = MockProtocolAdapter::permissive();
    adapter
        .set_response(
            "crypto.sign_ed25519",
            CryptoSignResponse {
                signature: B64.encode([0xAA, 0xBB]),
                algorithm: Some("Ed25519".to_string()),
                key_id: Some("default_signing_key".to_string()),
            },
        )
        .await
        .unwrap();
    let client = SigningClient::with_adapter(Box::new(adapter), "mock://test:9500");
    let did = Did::new("did:key:signer");
    let sig = client.sign(b"hello", &did).await.unwrap();
    assert_eq!(sig.as_bytes(), &[0xAA, 0xBB]);
}

#[tokio::test]
async fn test_sign_owned_via_mock() {
    use crate::integration::mocks::MockProtocolAdapter;
    let adapter = MockProtocolAdapter::permissive();
    adapter
        .set_response(
            "crypto.sign_ed25519",
            CryptoSignResponse {
                signature: B64.encode([1, 2, 3]),
                algorithm: Some("Ed25519".to_string()),
                key_id: None,
            },
        )
        .await
        .unwrap();
    let client = SigningClient::with_adapter(Box::new(adapter), "mock://test:9500");
    let did = Did::new("did:key:signer");
    let sig = client.sign_owned(bytes::Bytes::from_static(b"data"), &did).await.unwrap();
    assert_eq!(sig.as_bytes(), &[1, 2, 3]);
}

#[tokio::test]
async fn test_verify_via_mock() {
    use crate::integration::mocks::MockProtocolAdapter;
    let adapter = MockProtocolAdapter::permissive();
    adapter
        .set_response(
            "crypto.verify_ed25519",
            CryptoVerifyResponse {
                valid: true,
                algorithm: Some("Ed25519".to_string()),
            },
        )
        .await
        .unwrap();
    let client = SigningClient::with_adapter(Box::new(adapter), "mock://test:9500");
    let did = Did::new("did:key:verifier");
    let sig = Signature::new(vec![1, 2, 3]);
    let valid = client.verify(b"data", &sig, &did).await.unwrap();
    assert!(valid);
}

#[tokio::test]
async fn test_verify_owned_via_mock() {
    use crate::integration::mocks::MockProtocolAdapter;
    let adapter = MockProtocolAdapter::permissive();
    adapter
        .set_response(
            "crypto.verify_ed25519",
            CryptoVerifyResponse {
                valid: false,
                algorithm: None,
            },
        )
        .await
        .unwrap();
    let client = SigningClient::with_adapter(Box::new(adapter), "mock://test:9500");
    let did = Did::new("did:key:verifier");
    let sig = Signature::new(vec![4, 5, 6]);
    let valid =
        client.verify_owned(bytes::Bytes::from_static(b"payload"), &sig, &did).await.unwrap();
    assert!(!valid);
}

#[tokio::test]
async fn test_verify_did_via_mock() {
    use crate::integration::mocks::MockProtocolAdapter;
    let adapter = MockProtocolAdapter::permissive();
    adapter
        .set_response(
            "verify_did",
            VerifyDidResponse {
                valid: true,
            },
        )
        .await
        .unwrap();
    let client = SigningClient::with_adapter(Box::new(adapter), "mock://test:9500");
    let did = Did::new("did:key:to-verify");
    let valid = client.verify_did(&did).await.unwrap();
    assert!(valid);
}

#[tokio::test]
async fn test_sign_vertex_via_mock() {
    use crate::event::EventType;
    use crate::integration::mocks::MockProtocolAdapter;
    use crate::vertex::VertexBuilder;

    let adapter = MockProtocolAdapter::permissive();
    adapter
        .set_response(
            "crypto.sign_ed25519",
            CryptoSignResponse {
                signature: B64.encode([0xDE, 0xAD]),
                algorithm: Some("Ed25519".to_string()),
                key_id: None,
            },
        )
        .await
        .unwrap();
    let client = SigningClient::with_adapter(Box::new(adapter), "mock://test:9500");
    let vertex = VertexBuilder::new(EventType::SessionStart).build();
    let did = Did::new("did:key:vertex-signer");
    let sig = client.sign_vertex(&vertex, &did).await.unwrap();
    assert_eq!(sig.as_bytes(), &[0xDE, 0xAD]);
}

#[tokio::test]
async fn test_request_attestation_via_mock() {
    use crate::event::SessionOutcome;
    use crate::integration::mocks::MockProtocolAdapter;
    use crate::merkle::MerkleRoot;
    use crate::types::ContentHash;

    let attester = Did::new("did:key:attester");

    let adapter = MockProtocolAdapter::permissive();
    adapter
        .set_response(
            "crypto.sign_contract",
            CryptoSignContractResponse {
                terms_hash: hex::encode([0xABu8; 32]),
                signature: hex::encode([0x01u8; 64]),
                public_key: "did:key:z6MkAttesterKey".to_string(),
                signed_at: "2026-04-15T12:00:00Z".to_string(),
            },
        )
        .await
        .unwrap();

    let client = SigningClient::with_adapter(Box::new(adapter), "mock://test:9500");
    let summary = DehydrationSummary {
        session_id: crate::types::SessionId::new(uuid::Uuid::now_v7()),
        session_type: "test".to_string(),
        created_at: crate::types::Timestamp::now(),
        resolved_at: crate::types::Timestamp::now(),
        outcome: SessionOutcome::Success,
        merkle_root: MerkleRoot::new(ContentHash::from([0u8; 32])),
        vertex_count: 5,
        payload_bytes: 500,
        results: vec![],
        agents: vec![],
        attestations: vec![],
    };

    let result = client.request_attestation(&attester, &summary).await.unwrap();
    assert_eq!(result.attester, attester);
    assert!(result.verified);
    assert_eq!(result.signature.len(), 64);
}

#[tokio::test]
async fn test_mock_client_is_available() {
    let client = mock_client();
    assert!(client.is_available().await);
}

#[tokio::test]
async fn test_signing_discover_discovering_status() {
    let registry = DiscoveryRegistry::new("test-rhizocrypt");
    registry
        .register_endpoint(ServiceEndpoint::new(
            "signer".to_string(),
            "127.0.0.1:9500".parse().unwrap(),
            vec![Capability::PayloadStorage],
        ))
        .await;

    let result = SigningClient::discover(&registry).await;
    assert!(result.is_err());
}
