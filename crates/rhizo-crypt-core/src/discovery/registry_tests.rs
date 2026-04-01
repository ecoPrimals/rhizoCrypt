// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2024–2026 ecoPrimals Project

use super::*;

use std::sync::Arc;
use std::sync::atomic::{AtomicU32, Ordering};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;

async fn serve_one_http_json_response(
    listener: TcpListener,
    json_body: &str,
    request_count: Option<Arc<AtomicU32>>,
) {
    if let Ok((mut stream, _)) = listener.accept().await {
        if let Some(c) = request_count {
            c.fetch_add(1, Ordering::SeqCst);
        }
        let mut buf = vec![0u8; 8192];
        let _ = stream.read(&mut buf).await;
        let body = json_body.as_bytes();
        let header = format!(
            "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
            body.len(),
        );
        let _ = stream.write_all(header.as_bytes()).await;
        let _ = stream.write_all(body).await;
        let _ = stream.flush().await;
        let _ = stream.shutdown().await;
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_registry_self_knowledge() {
    let registry = DiscoveryRegistry::new("rhizoCrypt");

    assert!(!registry.is_available(&Capability::DidVerification).await);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_registry_registration() {
    let registry = DiscoveryRegistry::new("rhizoCrypt");

    let endpoint = ServiceEndpoint::new(
        "bearDog",
        "127.0.0.1:9000".parse().unwrap(),
        vec![Capability::DidVerification, Capability::Signing],
    );

    registry.register_endpoint(endpoint).await;

    assert!(registry.is_available(&Capability::DidVerification).await);
    assert!(registry.is_available(&Capability::Signing).await);
    assert!(!registry.is_available(&Capability::PayloadStorage).await);
}

#[test]
fn test_discovery_status() {
    let unavailable = DiscoveryStatus::Unavailable;
    assert!(!unavailable.is_available());
    assert!(unavailable.first_endpoint().is_none());

    let discovering = DiscoveryStatus::Discovering;
    assert!(!discovering.is_available());

    let failed = DiscoveryStatus::Failed("test error".to_string());
    assert!(!failed.is_available());

    let endpoint =
        ServiceEndpoint::new("test", "127.0.0.1:9000".parse().unwrap(), vec![Capability::Signing]);
    let available = DiscoveryStatus::Available(vec![endpoint]);
    assert!(available.is_available());
    assert!(available.first_endpoint().is_some());
}

#[test]
fn test_discovery_status_clone() {
    let status = DiscoveryStatus::Failed("error".to_string());
    let cloned = status;
    match cloned {
        DiscoveryStatus::Failed(msg) => assert_eq!(msg, "error"),
        _ => panic!("Clone failed"),
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_registry_discover() {
    let registry = DiscoveryRegistry::new("rhizoCrypt");

    let status = registry.discover(&Capability::Signing).await;
    assert!(!status.is_available());

    registry
        .register_endpoint(ServiceEndpoint::new(
            "bearDog",
            "127.0.0.1:9000".parse().unwrap(),
            vec![Capability::Signing],
        ))
        .await;

    let status = registry.discover(&Capability::Signing).await;
    assert!(status.is_available());
    assert!(status.first_endpoint().is_some());
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_registry_get_endpoint() {
    let registry = DiscoveryRegistry::new("rhizoCrypt");

    assert!(registry.get_endpoint(&Capability::PayloadStorage).await.is_none());

    registry
        .register_endpoint(ServiceEndpoint::new(
            "nestGate",
            "127.0.0.1:9001".parse().unwrap(),
            vec![Capability::PayloadStorage],
        ))
        .await;

    let endpoint = registry.get_endpoint(&Capability::PayloadStorage).await;
    assert!(endpoint.is_some());
    assert_eq!(endpoint.unwrap().service_id.as_ref(), "nestGate");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_registry_local_name() {
    let registry = DiscoveryRegistry::new("myPrimal");
    assert_eq!(registry.local_name(), "myPrimal");

    let owned_name = String::from("dynamicPrimal");
    let registry2 = DiscoveryRegistry::new(owned_name);
    assert_eq!(registry2.local_name(), "dynamicPrimal");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_registry_set_discovery_source() {
    let registry = DiscoveryRegistry::new("rhizoCrypt");

    let addr: SocketAddr = "127.0.0.1:8091".parse().unwrap();
    registry.set_discovery_source(addr).await;

    let status = registry.discover(&Capability::ServiceDiscovery).await;
    assert!(!status.is_available());
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_registry_all_endpoints() {
    let registry = DiscoveryRegistry::new("rhizoCrypt");

    let all = registry.all_endpoints().await;
    assert!(all.is_empty());

    registry
        .register_endpoint(ServiceEndpoint::new(
            "bearDog",
            "127.0.0.1:9000".parse().unwrap(),
            vec![Capability::DidVerification, Capability::Signing],
        ))
        .await;
    registry
        .register_endpoint(ServiceEndpoint::new(
            "nestGate",
            "127.0.0.1:9001".parse().unwrap(),
            vec![Capability::PayloadStorage],
        ))
        .await;

    let all = registry.all_endpoints().await;
    assert!(all.len() >= 2);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_registry_multiple_endpoints_for_capability() {
    let registry = DiscoveryRegistry::new("rhizoCrypt");

    registry
        .register_endpoint(ServiceEndpoint::new(
            "bearDog1",
            "127.0.0.1:9000".parse().unwrap(),
            vec![Capability::Signing],
        ))
        .await;
    registry
        .register_endpoint(ServiceEndpoint::new(
            "bearDog2",
            "127.0.0.1:9001".parse().unwrap(),
            vec![Capability::Signing],
        ))
        .await;

    let status = registry.discover(&Capability::Signing).await;
    match status {
        DiscoveryStatus::Available(endpoints) => {
            assert_eq!(endpoints.len(), 2);
        }
        _ => panic!("Expected Available status"),
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_registry_toadstool() {
    let registry = DiscoveryRegistry::new("rhizoCrypt");

    registry
        .register_endpoint(ServiceEndpoint::new(
            "toadStool",
            "127.0.0.1:9003".parse().unwrap(),
            vec![Capability::ComputeOrchestration, Capability::ComputeEvents],
        ))
        .await;

    assert!(registry.is_available(&Capability::ComputeOrchestration).await);
    assert!(registry.is_available(&Capability::ComputeEvents).await);

    let endpoint = registry.get_endpoint(&Capability::ComputeOrchestration).await;
    assert!(endpoint.is_some());
    assert_eq!(endpoint.unwrap().service_id.as_ref(), "toadStool");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_registry_sweetgrass() {
    let registry = DiscoveryRegistry::new("rhizoCrypt");

    registry
        .register_endpoint(ServiceEndpoint::new(
            "sweetGrass",
            "127.0.0.1:9004".parse().unwrap(),
            vec![Capability::ProvenanceQuery, Capability::Attribution],
        ))
        .await;

    assert!(registry.is_available(&Capability::ProvenanceQuery).await);
    assert!(registry.is_available(&Capability::Attribution).await);

    let endpoint = registry.get_endpoint(&Capability::ProvenanceQuery).await;
    assert!(endpoint.is_some());
    let ep = endpoint.unwrap();
    assert_eq!(ep.service_id.as_ref(), "sweetGrass");
    assert_eq!(ep.addr.port(), 9004);
}

#[test]
fn test_parse_capability_all_variants() {
    assert!(matches!(parse_capability("DidVerification"), Some(Capability::DidVerification)));
    assert!(matches!(parse_capability("did_verification"), Some(Capability::DidVerification)));
    assert!(matches!(parse_capability("Signing"), Some(Capability::Signing)));
    assert!(matches!(parse_capability("signing"), Some(Capability::Signing)));
    assert!(matches!(
        parse_capability("SignatureVerification"),
        Some(Capability::SignatureVerification)
    ));
    assert!(matches!(
        parse_capability("signature_verification"),
        Some(Capability::SignatureVerification)
    ));
    assert!(matches!(parse_capability("Attestation"), Some(Capability::Attestation)));
    assert!(matches!(parse_capability("attestation"), Some(Capability::Attestation)));
    assert!(matches!(parse_capability("ServiceDiscovery"), Some(Capability::ServiceDiscovery)));
    assert!(matches!(parse_capability("service_discovery"), Some(Capability::ServiceDiscovery)));
    assert!(matches!(parse_capability("PayloadStorage"), Some(Capability::PayloadStorage)));
    assert!(matches!(parse_capability("payload_storage"), Some(Capability::PayloadStorage)));
    assert!(matches!(parse_capability("PayloadRetrieval"), Some(Capability::PayloadRetrieval)));
    assert!(matches!(parse_capability("payload_retrieval"), Some(Capability::PayloadRetrieval)));
    assert!(matches!(parse_capability("PermanentCommit"), Some(Capability::PermanentCommit)));
    assert!(matches!(parse_capability("permanent_commit"), Some(Capability::PermanentCommit)));
    assert!(matches!(parse_capability("SliceCheckout"), Some(Capability::SliceCheckout)));
    assert!(matches!(parse_capability("slice_checkout"), Some(Capability::SliceCheckout)));
    assert!(matches!(parse_capability("SliceResolution"), Some(Capability::SliceResolution)));
    assert!(matches!(parse_capability("slice_resolution"), Some(Capability::SliceResolution)));
    assert!(matches!(
        parse_capability("ComputeOrchestration"),
        Some(Capability::ComputeOrchestration)
    ));
    assert!(matches!(
        parse_capability("compute_orchestration"),
        Some(Capability::ComputeOrchestration)
    ));
    assert!(matches!(parse_capability("ComputeEvents"), Some(Capability::ComputeEvents)));
    assert!(matches!(parse_capability("compute_events"), Some(Capability::ComputeEvents)));
    assert!(matches!(parse_capability("ProvenanceQuery"), Some(Capability::ProvenanceQuery)));
    assert!(matches!(parse_capability("provenance_query"), Some(Capability::ProvenanceQuery)));
    assert!(matches!(parse_capability("Attribution"), Some(Capability::Attribution)));
    assert!(matches!(parse_capability("attribution"), Some(Capability::Attribution)));
}

#[test]
fn test_parse_capability_colon_format() {
    assert!(matches!(parse_capability("did:verification"), Some(Capability::DidVerification)));
    assert!(matches!(parse_capability("crypto:signing"), Some(Capability::Signing)));
    assert!(matches!(
        parse_capability("crypto:verification"),
        Some(Capability::SignatureVerification)
    ));
    assert!(matches!(parse_capability("attestation:request"), Some(Capability::Attestation)));
    assert!(matches!(parse_capability("discovery:service"), Some(Capability::ServiceDiscovery)));
    assert!(matches!(parse_capability("payload:storage"), Some(Capability::PayloadStorage)));
    assert!(matches!(parse_capability("payload:retrieval"), Some(Capability::PayloadRetrieval)));
    assert!(matches!(
        parse_capability("storage:permanent:commit"),
        Some(Capability::PermanentCommit)
    ));
    assert!(matches!(parse_capability("slice:checkout"), Some(Capability::SliceCheckout)));
    assert!(matches!(parse_capability("slice:resolution"), Some(Capability::SliceResolution)));
    assert!(matches!(
        parse_capability("compute:orchestration"),
        Some(Capability::ComputeOrchestration)
    ));
    assert!(matches!(parse_capability("compute:events"), Some(Capability::ComputeEvents)));
    assert!(matches!(parse_capability("provenance:query"), Some(Capability::ProvenanceQuery)));
    assert!(matches!(parse_capability("provenance:attribution"), Some(Capability::Attribution)));
}

#[test]
fn test_parse_capability_custom_and_empty() {
    let custom = parse_capability("MyCustomCapability");
    assert!(custom.is_some());
    match custom.unwrap() {
        Capability::Custom(name) => assert_eq!(name, "MyCustomCapability"),
        _ => panic!("Expected Custom variant"),
    }

    assert!(parse_capability("").is_none());
}

#[test]
fn test_dual_format_capabilities_flat_strings() {
    let json = r#"[
        {"service_id": "svc1", "address": "127.0.0.1:9000", "capabilities": ["Signing", "Attestation"]}
    ]"#;
    let endpoints: Vec<serde_json::Value> = serde_json::from_str(json).unwrap();
    assert_eq!(endpoints.len(), 1);
}

#[test]
fn test_dual_format_capabilities_nested_objects() {
    #[derive(serde::Deserialize)]
    struct TestEndpoint {
        #[serde(deserialize_with = "super::deserialize_dual_capabilities")]
        capabilities: Vec<String>,
    }

    let json =
        r#"{"capabilities": [{"name": "Signing", "version": "1.0"}, {"name": "Attestation"}]}"#;
    let ep: TestEndpoint = serde_json::from_str(json).unwrap();
    assert_eq!(ep.capabilities, vec!["Signing", "Attestation"]);
}

#[test]
fn test_dual_format_capabilities_mixed() {
    #[derive(serde::Deserialize)]
    struct TestEndpoint {
        #[serde(deserialize_with = "super::deserialize_dual_capabilities")]
        capabilities: Vec<String>,
    }

    let json = r#"{"capabilities": ["signing", {"name": "Attestation"}]}"#;
    let ep: TestEndpoint = serde_json::from_str(json).unwrap();
    assert_eq!(ep.capabilities, vec!["signing", "Attestation"]);
}

#[test]
fn test_dual_format_capabilities_empty() {
    #[derive(serde::Deserialize)]
    struct TestEndpoint {
        #[serde(default, deserialize_with = "super::deserialize_dual_capabilities")]
        capabilities: Vec<String>,
    }

    let json = r#"{"capabilities": []}"#;
    let ep: TestEndpoint = serde_json::from_str(json).unwrap();
    assert!(ep.capabilities.is_empty());
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_discover_unhealthy_endpoints_filtered() {
    let registry = DiscoveryRegistry::new("rhizoCrypt");

    let mut endpoint = ServiceEndpoint::new(
        "bearDog",
        "127.0.0.1:9000".parse().unwrap(),
        vec![Capability::Signing],
    );
    endpoint.last_healthy =
        std::time::Instant::now().checked_sub(std::time::Duration::from_secs(300)).unwrap();

    registry.register_endpoint(endpoint).await;

    let status = registry.discover(&Capability::Signing).await;
    assert!(!status.is_available(), "Unhealthy endpoints should be filtered out");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_discover_with_source_connection_refused() {
    let registry = DiscoveryRegistry::new("rhizoCrypt");
    registry.set_discovery_source("127.0.0.1:1".parse().unwrap()).await;

    let status = registry.discover(&Capability::Signing).await;
    match status {
        DiscoveryStatus::Failed(msg) => {
            assert!(
                msg.contains("connection")
                    || msg.contains("Connection")
                    || msg.contains("timed out"),
                "Expected connection error, got: {msg}"
            );
        }
        _ => panic!("Expected Failed status from unreachable discovery source"),
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_get_endpoint_with_multiple() {
    let registry = DiscoveryRegistry::new("rhizoCrypt");

    registry
        .register_endpoint(ServiceEndpoint::new(
            "bearDog1",
            "127.0.0.1:9000".parse().unwrap(),
            vec![Capability::Signing],
        ))
        .await;
    registry
        .register_endpoint(ServiceEndpoint::new(
            "bearDog2",
            "127.0.0.1:9001".parse().unwrap(),
            vec![Capability::Signing],
        ))
        .await;

    let ep = registry.get_endpoint(&Capability::Signing).await;
    assert!(ep.is_some());
}

#[test]
fn extract_capabilities_format_a_flat_strings() {
    let v = serde_json::json!(["dag.session.create", "health.check"]);
    let caps = extract_capabilities(&v);
    assert_eq!(caps, vec!["dag.session.create", "health.check"]);
}

#[test]
fn extract_capabilities_format_b_nested_objects() {
    let v = serde_json::json!([
        {"name": "dag.session.create", "version": "1.0"},
        {"name": "health.check"}
    ]);
    let caps = extract_capabilities(&v);
    assert_eq!(caps, vec!["dag.session.create", "health.check"]);
}

#[test]
fn extract_capabilities_format_c_wrapper() {
    let v = serde_json::json!({"capabilities": ["dag.session.create", "health.check"]});
    let caps = extract_capabilities(&v);
    assert_eq!(caps, vec!["dag.session.create", "health.check"]);
}

#[test]
fn extract_capabilities_format_d_double_nested() {
    let v = serde_json::json!({"capabilities": [{"name": "dag.session.create"}, {"name": "health.check"}]});
    let caps = extract_capabilities(&v);
    assert_eq!(caps, vec!["dag.session.create", "health.check"]);
}

#[test]
fn extract_capabilities_methods_key() {
    let v = serde_json::json!({"methods": ["a.b", "c.d"]});
    let caps = extract_capabilities(&v);
    assert_eq!(caps, vec!["a.b", "c.d"]);
}

#[test]
fn extract_capabilities_empty() {
    let v = serde_json::json!([]);
    assert!(extract_capabilities(&v).is_empty());

    let v = serde_json::json!(null);
    assert!(extract_capabilities(&v).is_empty());
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_clear_discovery_source() {
    let registry = DiscoveryRegistry::new("rhizoCrypt");
    registry.set_discovery_source("127.0.0.1:8092".parse().unwrap()).await;
    registry.clear_discovery_source().await;

    let status = registry.discover(&Capability::Signing).await;
    assert!(matches!(status, DiscoveryStatus::Unavailable));
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_discover_rpc_empty_array_returns_unavailable() {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let body = r#"{"jsonrpc":"2.0","result":[],"id":1}"#;
    tokio::spawn(serve_one_http_json_response(listener, body, None));

    let registry = DiscoveryRegistry::new("rhizoCrypt");
    registry.set_discovery_source(addr).await;

    let status = registry.discover(&Capability::Signing).await;
    assert!(matches!(status, DiscoveryStatus::Unavailable));
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_discover_rpc_result_null_returns_unavailable() {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let body = r#"{"jsonrpc":"2.0","result":null,"id":1}"#;
    tokio::spawn(serve_one_http_json_response(listener, body, None));

    let registry = DiscoveryRegistry::new("rhizoCrypt");
    registry.set_discovery_source(addr).await;

    let status = registry.discover(&Capability::PayloadStorage).await;
    assert!(matches!(status, DiscoveryStatus::Unavailable));
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_discover_rpc_invalid_json_returns_failed() {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let body = "not valid json {{{";
    tokio::spawn(serve_one_http_json_response(listener, body, None));

    let registry = DiscoveryRegistry::new("rhizoCrypt");
    registry.set_discovery_source(addr).await;

    let status = registry.discover(&Capability::Attestation).await;
    match status {
        DiscoveryStatus::Failed(msg) => assert!(msg.contains("parse") || msg.contains("Parse")),
        _ => panic!("expected Failed"),
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_discover_rpc_skips_invalid_address() {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let body = r#"{"jsonrpc":"2.0","result":[{"service_id":"bad","address":"not-a-socket-addr","capabilities":["Signing"]}],"id":1}"#;
    tokio::spawn(serve_one_http_json_response(listener, body, None));

    let registry = DiscoveryRegistry::new("rhizoCrypt");
    registry.set_discovery_source(addr).await;

    let status = registry.discover(&Capability::Signing).await;
    assert!(matches!(status, DiscoveryStatus::Unavailable));
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_discover_rpc_skips_when_capabilities_empty() {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let body = r#"{"jsonrpc":"2.0","result":[{"service_id":"x","address":"127.0.0.1:9300","capabilities":[]}],"id":1}"#;
    tokio::spawn(serve_one_http_json_response(listener, body, None));

    let registry = DiscoveryRegistry::new("rhizoCrypt");
    registry.set_discovery_source(addr).await;

    let status = registry.discover(&Capability::Signing).await;
    assert!(matches!(status, DiscoveryStatus::Unavailable));
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_discover_rpc_caches_endpoints_clear_source_still_hits_cache() {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let body = r#"{"jsonrpc":"2.0","result":[{"service_id":"rpcSvc","address":"127.0.0.1:9201","capabilities":["Signing"]}],"id":1}"#;
    let hits = Arc::new(AtomicU32::new(0));
    let hits_bg = Arc::clone(&hits);
    tokio::spawn(serve_one_http_json_response(listener, body, Some(hits_bg)));

    let registry = DiscoveryRegistry::new("rhizoCrypt");
    registry.set_discovery_source(addr).await;

    let s1 = registry.discover(&Capability::Signing).await;
    assert!(s1.is_available());

    registry.clear_discovery_source().await;

    let s2 = registry.discover(&Capability::Signing).await;
    assert!(s2.is_available());
    assert_eq!(hits.load(Ordering::SeqCst), 1);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_discover_unhealthy_then_empty_rpc_returns_unavailable() {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let body = r#"{"jsonrpc":"2.0","result":[],"id":1}"#;
    tokio::spawn(serve_one_http_json_response(listener, body, None));

    let registry = DiscoveryRegistry::new("rhizoCrypt");
    let mut endpoint =
        ServiceEndpoint::new("stale", "127.0.0.1:9000".parse().unwrap(), vec![Capability::Signing]);
    endpoint.last_healthy =
        std::time::Instant::now().checked_sub(std::time::Duration::from_secs(300)).unwrap();
    registry.register_endpoint(endpoint).await;

    registry.set_discovery_source(addr).await;

    let status = registry.discover(&Capability::Signing).await;
    assert!(matches!(status, DiscoveryStatus::Unavailable));
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_registry_concurrent_register_and_discover() {
    let registry = Arc::new(DiscoveryRegistry::new("rhizoCrypt"));
    let mut handles = vec![];
    for i in 0u16..8 {
        let r = Arc::clone(&registry);
        handles.push(tokio::spawn(async move {
            let port = 19600 + i;
            let sock: SocketAddr = format!("127.0.0.1:{port}").parse().unwrap();
            r.register_endpoint(ServiceEndpoint::new(
                format!("svc{i}"),
                sock,
                vec![Capability::Signing],
            ))
            .await;
            r.discover(&Capability::Signing).await
        }));
    }
    for h in handles {
        let status = h.await.unwrap();
        assert!(status.is_available());
    }

    match registry.discover(&Capability::Signing).await {
        DiscoveryStatus::Available(eps) => assert!(eps.len() >= 8),
        _ => panic!("expected Available"),
    }
}
