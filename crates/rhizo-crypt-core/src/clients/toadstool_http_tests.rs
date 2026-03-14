// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2024–2026 ecoPrimals Project

#![allow(clippy::unwrap_used)]

use super::*;

// ============================================================================
// Unit tests (no HTTP)
// ============================================================================

#[test]
fn test_parse_deployment_id_uuid() {
    let uuid_str = "550e8400-e29b-41d4-a716-446655440000";
    let task_id = parse_deployment_id(uuid_str);
    assert!(task_id.is_some());
}

#[test]
fn test_parse_deployment_id_hex() {
    let hex_str = "deadbeef00000000deadbeef00000000";
    let task_id = parse_deployment_id(hex_str);
    assert!(task_id.is_some());
    assert_eq!(task_id.unwrap().0[0], 0xde);
}

#[test]
fn test_deployment_status_serde() {
    let json = r#""running""#;
    let status: DeploymentStatus = serde_json::from_str(json).unwrap();
    assert!(matches!(status, DeploymentStatus::Running));
}

#[test]
fn test_health_status_serde() {
    let json = r#"{"status":"healthy","service":"toadstool-byob-server","version":"0.1.0","message":"Ready"}"#;
    let health: HealthStatus = serde_json::from_str(json).unwrap();
    assert_eq!(health.status, "healthy");
    assert_eq!(health.service.as_deref(), Some("toadstool-byob-server"));
}

#[test]
fn test_resource_usage_serde() {
    let json = r#"{"cpu_usage":0.5,"memory_bytes":1048576,"network_tx_bytes":1024,"network_rx_bytes":2048}"#;
    let usage: ResourceUsage = serde_json::from_str(json).unwrap();
    assert!((usage.cpu_usage - 0.5).abs() < f64::EPSILON);
    assert_eq!(usage.memory_bytes, 1_048_576);
}

#[test]
fn test_parse_deployment_id_short_hex() {
    assert!(parse_deployment_id("ab").is_none());
}

#[test]
fn test_parse_deployment_id_invalid() {
    assert!(parse_deployment_id("not-a-uuid-or-hex").is_none());
    assert!(parse_deployment_id("").is_none());
    assert!(parse_deployment_id("zzzzzzzzzzzzzzzz").is_none());
}

#[test]
fn test_client_new() {
    let client = ToadStoolHttpClient::new("http://localhost:8084").unwrap();
    assert_eq!(client.base_url, "http://localhost:8084");
}

#[test]
fn test_deployment_to_event_pending() {
    let client = ToadStoolHttpClient::new("http://localhost:8084").unwrap();
    let did = Did::new("did:key:test");
    let worker = Did::new("did:compute:test-worker");
    let deployment = DeploymentResponse {
        deployment_id: "550e8400-e29b-41d4-a716-446655440000".to_string(),
        status: DeploymentStatus::Pending,
        biome_name: None,
        created_at: None,
        completed_at: None,
        error: None,
        result: None,
    };
    let event = client.deployment_to_event(&deployment, &did, &worker);
    assert!(event.is_some());
    assert!(matches!(event.unwrap(), ComputeEvent::TaskCreated { .. }));
}

#[test]
fn test_deployment_to_event_running() {
    let client = ToadStoolHttpClient::new("http://localhost:8084").unwrap();
    let did = Did::new("did:key:test");
    let worker = Did::new("did:compute:test-worker");
    let deployment = DeploymentResponse {
        deployment_id: "550e8400-e29b-41d4-a716-446655440000".to_string(),
        status: DeploymentStatus::Running,
        biome_name: Some("test-biome".to_string()),
        created_at: None,
        completed_at: None,
        error: None,
        result: None,
    };
    let event = client.deployment_to_event(&deployment, &did, &worker);
    assert!(matches!(event.unwrap(), ComputeEvent::TaskStarted { .. }));
}

#[test]
fn test_deployment_to_event_completed() {
    let client = ToadStoolHttpClient::new("http://localhost:8084").unwrap();
    let did = Did::new("did:key:test");
    let worker = Did::new("did:compute:test-worker");
    let deployment = DeploymentResponse {
        deployment_id: "550e8400-e29b-41d4-a716-446655440000".to_string(),
        status: DeploymentStatus::Completed,
        biome_name: None,
        created_at: None,
        completed_at: None,
        error: None,
        result: Some(serde_json::json!({"output": "data"})),
    };
    let event = client.deployment_to_event(&deployment, &did, &worker);
    assert!(matches!(event.unwrap(), ComputeEvent::TaskCompleted { .. }));
}

#[test]
fn test_deployment_to_event_failed() {
    let client = ToadStoolHttpClient::new("http://localhost:8084").unwrap();
    let did = Did::new("did:key:test");
    let worker = Did::new("did:compute:test-worker");
    let deployment = DeploymentResponse {
        deployment_id: "550e8400-e29b-41d4-a716-446655440000".to_string(),
        status: DeploymentStatus::Failed,
        biome_name: None,
        created_at: None,
        completed_at: None,
        error: Some("out of memory".to_string()),
        result: None,
    };
    let event = client.deployment_to_event(&deployment, &did, &worker);
    match event.unwrap() {
        ComputeEvent::TaskFailed {
            error,
            ..
        } => {
            assert_eq!(error, "out of memory");
        }
        _ => panic!("expected TaskFailed"),
    }
}

#[test]
fn test_deployment_to_event_failed_no_error_msg() {
    let client = ToadStoolHttpClient::new("http://localhost:8084").unwrap();
    let did = Did::new("did:key:test");
    let worker = Did::new("did:compute:test-worker");
    let deployment = DeploymentResponse {
        deployment_id: "550e8400-e29b-41d4-a716-446655440000".to_string(),
        status: DeploymentStatus::Failed,
        biome_name: None,
        created_at: None,
        completed_at: None,
        error: None,
        result: None,
    };
    let event = client.deployment_to_event(&deployment, &did, &worker);
    match event.unwrap() {
        ComputeEvent::TaskFailed {
            error,
            ..
        } => {
            assert_eq!(error, "Unknown error");
        }
        _ => panic!("expected TaskFailed"),
    }
}

#[test]
fn test_deployment_to_event_stopped() {
    let client = ToadStoolHttpClient::new("http://localhost:8084").unwrap();
    let did = Did::new("did:key:test");
    let worker = Did::new("did:compute:test-worker");
    let deployment = DeploymentResponse {
        deployment_id: "550e8400-e29b-41d4-a716-446655440000".to_string(),
        status: DeploymentStatus::Stopped,
        biome_name: None,
        created_at: None,
        completed_at: None,
        error: None,
        result: None,
    };
    let event = client.deployment_to_event(&deployment, &did, &worker);
    assert!(matches!(event.unwrap(), ComputeEvent::TaskCancelled { .. }));
}

#[test]
fn test_deployment_to_event_invalid_id() {
    let client = ToadStoolHttpClient::new("http://localhost:8084").unwrap();
    let did = Did::new("did:key:test");
    let worker = Did::new("did:compute:test-worker");
    let deployment = DeploymentResponse {
        deployment_id: "bad".to_string(),
        status: DeploymentStatus::Running,
        biome_name: None,
        created_at: None,
        completed_at: None,
        error: None,
        result: None,
    };
    assert!(client.deployment_to_event(&deployment, &did, &worker).is_none());
}

#[test]
fn test_poll_events_from_deployments() {
    let client = ToadStoolHttpClient::new("http://localhost:8084").unwrap();
    let did = Did::new("did:key:test");
    let worker = Did::new("did:compute:test-worker");
    let deployments = vec![
        DeploymentResponse {
            deployment_id: "550e8400-e29b-41d4-a716-446655440000".to_string(),
            status: DeploymentStatus::Running,
            biome_name: None,
            created_at: None,
            completed_at: None,
            error: None,
            result: None,
        },
        DeploymentResponse {
            deployment_id: "bad-id".to_string(),
            status: DeploymentStatus::Running,
            biome_name: None,
            created_at: None,
            completed_at: None,
            error: None,
            result: None,
        },
    ];
    let events = poll_events_from_deployments(&client, &deployments, &did, &worker);
    assert_eq!(events.len(), 1);
}

#[test]
fn test_poll_events_empty() {
    let client = ToadStoolHttpClient::new("http://localhost:8084").unwrap();
    let did = Did::new("did:key:test");
    let worker = Did::new("did:compute:test-worker");
    let events = poll_events_from_deployments(&client, &[], &did, &worker);
    assert!(events.is_empty());
}

#[test]
fn test_deployment_response_serde() {
    let json = serde_json::json!({
        "deployment_id": "abc-123",
        "status": "running",
        "biome_name": "test-biome",
        "created_at": "2024-01-01T00:00:00Z",
        "completed_at": null,
        "error": null,
        "result": null
    });
    let resp: DeploymentResponse = serde_json::from_value(json).unwrap();
    assert_eq!(resp.deployment_id, "abc-123");
    assert!(matches!(resp.status, DeploymentStatus::Running));
    assert_eq!(resp.biome_name.as_deref(), Some("test-biome"));
}

#[test]
fn test_stop_deployment_response_serde() {
    let json = r#"{"deployment_id":"abc","message":"stopped"}"#;
    let resp: StopDeploymentResponse = serde_json::from_str(json).unwrap();
    assert_eq!(resp.deployment_id, "abc");
    assert_eq!(resp.message, "stopped");
}

#[test]
fn test_byob_health_response_serde() {
    let json = r#"{"status":"healthy","message":"BYOB API operational"}"#;
    let resp: ByobHealthResponse = serde_json::from_str(json).unwrap();
    assert_eq!(resp.status, "healthy");
}

#[test]
fn test_deployment_status_all_variants() {
    for (json, expected) in [
        (r#""pending""#, "Pending"),
        (r#""running""#, "Running"),
        (r#""completed""#, "Completed"),
        (r#""failed""#, "Failed"),
        (r#""stopped""#, "Stopped"),
    ] {
        let status: DeploymentStatus = serde_json::from_str(json).unwrap();
        assert_eq!(format!("{status:?}"), expected);
    }
}

#[test]
fn test_toadstool_error_display() {
    let err = ToadStoolHttpError::InvalidResponse("bad json".to_string());
    assert!(err.to_string().contains("bad json"));

    let err = ToadStoolHttpError::Server {
        status: 500,
        message: "internal".to_string(),
    };
    assert!(err.to_string().contains("500"));
    assert!(err.to_string().contains("internal"));
}

// ============================================================================
// Wiremock-based HTTP integration tests (require http-clients feature)
// ============================================================================

#[cfg(feature = "http-clients")]
#[tokio::test]
async fn test_wiremock_health_success() {
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/health"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "status": "healthy",
            "service": "toadstool-byob-server",
            "version": "0.1.0",
            "message": "Ready"
        })))
        .mount(&mock_server)
        .await;

    let client = ToadStoolHttpClient::new(mock_server.uri()).unwrap();
    let result = client.health().await;
    assert!(result.is_ok());
    let health = result.unwrap();
    assert_eq!(health.status, "healthy");
    assert_eq!(health.service.as_deref(), Some("toadstool-byob-server"));
    assert_eq!(health.version.as_deref(), Some("0.1.0"));
}

#[cfg(feature = "http-clients")]
#[tokio::test]
async fn test_wiremock_health_http_error() {
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/health"))
        .respond_with(ResponseTemplate::new(500).set_body_string("Internal Server Error"))
        .mount(&mock_server)
        .await;

    let client = ToadStoolHttpClient::new(mock_server.uri()).unwrap();
    let result = client.health().await;
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.to_string().contains("500"));
    assert!(err.to_string().contains("Internal Server Error"));
}

#[cfg(feature = "http-clients")]
#[tokio::test]
async fn test_wiremock_health_parse_error() {
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/health"))
        .respond_with(ResponseTemplate::new(200).set_body_string("not valid json"))
        .mount(&mock_server)
        .await;

    let client = ToadStoolHttpClient::new(mock_server.uri()).unwrap();
    let result = client.health().await;
    assert!(result.is_err());
}

#[cfg(feature = "http-clients")]
#[tokio::test]
async fn test_wiremock_byob_health_success() {
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/byob/health"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "status": "healthy",
            "message": "BYOB API operational"
        })))
        .mount(&mock_server)
        .await;

    let client = ToadStoolHttpClient::new(mock_server.uri()).unwrap();
    let result = client.byob_health().await;
    assert!(result.is_ok());
    let health = result.unwrap();
    assert_eq!(health.status, "healthy");
    assert_eq!(health.message, "BYOB API operational");
}

#[cfg(feature = "http-clients")]
#[tokio::test]
async fn test_wiremock_list_deployments_success() {
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/byob/deployments"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([
            {
                "deployment_id": "550e8400-e29b-41d4-a716-446655440000",
                "status": "running",
                "biome_name": "test-biome",
                "created_at": "2024-01-01T00:00:00Z",
                "completed_at": null,
                "error": null,
                "result": null
            }
        ])))
        .mount(&mock_server)
        .await;

    let client = ToadStoolHttpClient::new(mock_server.uri()).unwrap();
    let result = client.list_deployments().await;
    assert!(result.is_ok());
    let deployments = result.unwrap();
    assert_eq!(deployments.len(), 1);
    assert_eq!(deployments[0].deployment_id, "550e8400-e29b-41d4-a716-446655440000");
    assert!(matches!(deployments[0].status, DeploymentStatus::Running));
}

#[cfg(feature = "http-clients")]
#[tokio::test]
async fn test_wiremock_list_deployments_empty() {
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/byob/deployments"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([])))
        .mount(&mock_server)
        .await;

    let client = ToadStoolHttpClient::new(mock_server.uri()).unwrap();
    let result = client.list_deployments().await;
    assert!(result.is_ok());
    assert!(result.unwrap().is_empty());
}

#[cfg(feature = "http-clients")]
#[tokio::test]
async fn test_wiremock_get_deployment_success() {
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    let mock_server = MockServer::start().await;
    let deployment_id = "550e8400-e29b-41d4-a716-446655440000";

    Mock::given(method("GET"))
        .and(path(format!("/byob/deployments/{deployment_id}")))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "deployment_id": deployment_id,
            "status": "completed",
            "biome_name": "test-biome",
            "created_at": "2024-01-01T00:00:00Z",
            "completed_at": "2024-01-01T00:05:00Z",
            "error": null,
            "result": {"output": "success"}
        })))
        .mount(&mock_server)
        .await;

    let client = ToadStoolHttpClient::new(mock_server.uri()).unwrap();
    let result = client.get_deployment(deployment_id).await;
    assert!(result.is_ok());
    let deployment = result.unwrap();
    assert_eq!(deployment.deployment_id, deployment_id);
    assert!(matches!(deployment.status, DeploymentStatus::Completed));
    assert_eq!(deployment.result.as_ref().unwrap().get("output").unwrap(), "success");
}

#[cfg(feature = "http-clients")]
#[tokio::test]
async fn test_wiremock_get_deployment_not_found() {
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    let mock_server = MockServer::start().await;
    let deployment_id = "550e8400-e29b-41d4-a716-446655440000";

    Mock::given(method("GET"))
        .and(path(format!("/byob/deployments/{deployment_id}")))
        .respond_with(ResponseTemplate::new(404).set_body_string("Deployment not found"))
        .mount(&mock_server)
        .await;

    let client = ToadStoolHttpClient::new(mock_server.uri()).unwrap();
    let result = client.get_deployment(deployment_id).await;
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.to_string().contains("404"));
}

#[cfg(feature = "http-clients")]
#[tokio::test]
async fn test_wiremock_stop_deployment_success() {
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    let mock_server = MockServer::start().await;
    let deployment_id = "550e8400-e29b-41d4-a716-446655440000";

    Mock::given(method("POST"))
        .and(path(format!("/byob/deployments/{deployment_id}/stop")))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "deployment_id": deployment_id,
            "message": "Deployment stopped successfully"
        })))
        .mount(&mock_server)
        .await;

    let client = ToadStoolHttpClient::new(mock_server.uri()).unwrap();
    let result = client.stop_deployment(deployment_id).await;
    assert!(result.is_ok());
    let resp = result.unwrap();
    assert_eq!(resp.deployment_id, deployment_id);
    assert_eq!(resp.message, "Deployment stopped successfully");
}

#[cfg(feature = "http-clients")]
#[tokio::test]
async fn test_wiremock_stop_deployment_http_error() {
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    let mock_server = MockServer::start().await;
    let deployment_id = "550e8400-e29b-41d4-a716-446655440000";

    Mock::given(method("POST"))
        .and(path(format!("/byob/deployments/{deployment_id}/stop")))
        .respond_with(ResponseTemplate::new(500).set_body_string("Server error"))
        .mount(&mock_server)
        .await;

    let client = ToadStoolHttpClient::new(mock_server.uri()).unwrap();
    let result = client.stop_deployment(deployment_id).await;
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("500"));
}

#[cfg(feature = "http-clients")]
#[tokio::test]
async fn test_wiremock_get_resource_usage_success() {
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    let mock_server = MockServer::start().await;
    let deployment_id = "550e8400-e29b-41d4-a716-446655440000";

    Mock::given(method("GET"))
        .and(path(format!("/byob/deployments/{deployment_id}/usage")))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "cpu_usage": 0.75,
            "memory_bytes": 524_288,
            "network_tx_bytes": 1024,
            "network_rx_bytes": 2048
        })))
        .mount(&mock_server)
        .await;

    let client = ToadStoolHttpClient::new(mock_server.uri()).unwrap();
    let result = client.get_resource_usage(deployment_id).await;
    assert!(result.is_ok());
    let usage = result.unwrap();
    assert!((usage.cpu_usage - 0.75).abs() < f64::EPSILON);
    assert_eq!(usage.memory_bytes, 524_288);
    assert_eq!(usage.network_tx_bytes, Some(1024));
    assert_eq!(usage.network_rx_bytes, Some(2048));
}

#[cfg(feature = "http-clients")]
#[tokio::test]
async fn test_wiremock_get_resource_usage_parse_error() {
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    let mock_server = MockServer::start().await;
    let deployment_id = "550e8400-e29b-41d4-a716-446655440000";

    Mock::given(method("GET"))
        .and(path(format!("/byob/deployments/{deployment_id}/usage")))
        .respond_with(ResponseTemplate::new(200).set_body_string("{invalid json"))
        .mount(&mock_server)
        .await;

    let client = ToadStoolHttpClient::new(mock_server.uri()).unwrap();
    let result = client.get_resource_usage(deployment_id).await;
    assert!(result.is_err());
}

#[cfg(feature = "http-clients")]
#[tokio::test]
async fn test_wiremock_create_http_client_success() {
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/health"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "status": "healthy",
            "service": "toadstool",
            "version": "0.1.0",
            "message": "Ready"
        })))
        .mount(&mock_server)
        .await;

    let addr = *mock_server.address();
    let result = create_http_client(addr).await;
    assert!(result.is_ok());
}

#[cfg(feature = "http-clients")]
#[tokio::test]
async fn test_wiremock_create_http_client_health_fails() {
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/health"))
        .respond_with(ResponseTemplate::new(503).set_body_string("Service Unavailable"))
        .mount(&mock_server)
        .await;

    let addr = *mock_server.address();
    let result = create_http_client(addr).await;
    assert!(result.is_err());
    if let Err(e) = result {
        assert!(e.to_string().to_lowercase().contains("health"));
    }
}

#[cfg(feature = "http-clients")]
#[tokio::test]
async fn test_wiremock_connection_timeout() {
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/health"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_json(serde_json::json!({"status": "healthy"}))
                .set_delay(std::time::Duration::from_secs(60)),
        )
        .mount(&mock_server)
        .await;

    let client = ToadStoolHttpClient::new(mock_server.uri()).unwrap();
    let result = client.health().await;
    assert!(result.is_err());
}
