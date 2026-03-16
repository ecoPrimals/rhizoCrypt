// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2024–2026 ecoPrimals Project

use super::*;

#[test]
fn test_metrics_collector() {
    let collector = MetricsCollector::new();

    collector.record_request(RpcMethod::CreateSession, Duration::from_millis(5));
    collector.record_request(RpcMethod::CreateSession, Duration::from_millis(10));
    collector.record_request(RpcMethod::GetVertex, Duration::from_micros(100));

    assert_eq!(collector.request_count(RpcMethod::CreateSession), 2);
    assert_eq!(collector.request_count(RpcMethod::GetVertex), 1);
    assert_eq!(collector.request_count(RpcMethod::Health), 0);
}

#[test]
fn test_error_tracking() {
    let collector = MetricsCollector::new();

    collector.record_error(ErrorType::SessionNotFound);
    collector.record_error(ErrorType::SessionNotFound);
    collector.record_error(ErrorType::InvalidInput);

    assert_eq!(collector.error_count(ErrorType::SessionNotFound), 2);
    assert_eq!(collector.error_count(ErrorType::InvalidInput), 1);
    assert_eq!(collector.error_count(ErrorType::Internal), 0);
}

#[test]
fn test_session_gauge() {
    let collector = MetricsCollector::new();

    assert_eq!(collector.active_sessions(), 0);

    collector.inc_sessions();
    collector.inc_sessions();
    assert_eq!(collector.active_sessions(), 2);

    collector.dec_sessions();
    assert_eq!(collector.active_sessions(), 1);

    collector.set_sessions(10);
    assert_eq!(collector.active_sessions(), 10);
}

#[test]
fn test_histogram() {
    let collector = MetricsCollector::new();

    collector.record_request(RpcMethod::GetVertex, Duration::from_micros(50));
    collector.record_request(RpcMethod::GetVertex, Duration::from_micros(150));
    collector.record_request(RpcMethod::GetVertex, Duration::from_millis(1));

    let hist = collector.latency_histogram(RpcMethod::GetVertex);
    assert_eq!(hist.count, 3);
    assert!(hist.mean_seconds() > 0.0);
}

#[test]
fn test_prometheus_export() {
    let collector = MetricsCollector::new();

    collector.record_request(RpcMethod::CreateSession, Duration::from_millis(5));
    collector.inc_sessions();
    collector.inc_vertices(10);

    let output = collector.export_prometheus();

    assert!(output.contains("rhizocrypt_uptime_seconds"));
    assert!(output.contains("rhizocrypt_sessions_active 1"));
    assert!(output.contains("rhizocrypt_vertices_total 10"));
    assert!(output.contains("rhizocrypt_rpc_requests_total"));
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_request_timer() {
    let collector = MetricsCollector::new();

    {
        let timer = RequestTimer::start(&collector, RpcMethod::Health);
        tokio::task::yield_now().await;
        timer.finish();
    }

    assert_eq!(collector.request_count(RpcMethod::Health), 1);
    let hist = collector.latency_histogram(RpcMethod::Health);
    assert!(hist.mean_seconds() >= 0.0);
}

#[test]
fn test_shared_metrics() {
    let metrics = SharedMetrics::new();
    let metrics_clone = SharedMetrics::clone(&metrics);

    metrics.collector().inc_sessions();
    assert_eq!(metrics_clone.collector().active_sessions(), 1);
}

#[test]
fn test_shared_metrics_default() {
    let metrics = SharedMetrics::default();
    assert_eq!(metrics.collector().active_sessions(), 0);
}

#[test]
fn test_rpc_method_as_str() {
    assert_eq!(RpcMethod::CreateSession.as_str(), "create_session");
    assert_eq!(RpcMethod::GetSession.as_str(), "get_session");
    assert_eq!(RpcMethod::AppendEvent.as_str(), "append_event");
    assert_eq!(RpcMethod::GetMerkleProof.as_str(), "get_merkle_proof");
    assert_eq!(RpcMethod::Dehydrate.as_str(), "dehydrate");
    assert_eq!(RpcMethod::Health.as_str(), "health");
    assert_eq!(RpcMethod::Metrics.as_str(), "metrics");
}

#[test]
fn test_error_type_as_str() {
    assert_eq!(ErrorType::SessionNotFound.as_str(), "session_not_found");
    assert_eq!(ErrorType::VertexNotFound.as_str(), "vertex_not_found");
    assert_eq!(ErrorType::RateLimited.as_str(), "rate_limited");
    assert_eq!(ErrorType::Timeout.as_str(), "timeout");
}

#[test]
fn test_histogram_snapshot_empty() {
    let collector = MetricsCollector::new();
    let hist = collector.latency_histogram(RpcMethod::GetVertex);
    assert_eq!(hist.count, 0);
    assert!(hist.mean_seconds().abs() < f64::EPSILON);
}

#[test]
fn test_histogram_snapshot_bucket_bounds() {
    let bounds = HistogramSnapshot::bucket_bounds();
    assert_eq!(bounds.len(), 12);
    assert!(bounds[0] < bounds[11]);
}

#[test]
fn test_histogram_various_durations() {
    let collector = MetricsCollector::new();

    collector.record_request(RpcMethod::GetVertex, Duration::from_micros(50));
    collector.record_request(RpcMethod::GetVertex, Duration::from_millis(100));
    collector.record_request(RpcMethod::GetVertex, Duration::from_secs(2));
    collector.record_request(RpcMethod::GetVertex, Duration::from_secs_f64(10.0));

    let hist = collector.latency_histogram(RpcMethod::GetVertex);
    assert_eq!(hist.count, 4);
    assert!(hist.mean_seconds() > 0.0);
}

#[test]
fn test_all_error_types_recorded() {
    let collector = MetricsCollector::new();

    collector.record_error(ErrorType::SessionNotFound);
    collector.record_error(ErrorType::VertexNotFound);
    collector.record_error(ErrorType::InvalidInput);
    collector.record_error(ErrorType::Internal);
    collector.record_error(ErrorType::RateLimited);
    collector.record_error(ErrorType::Timeout);

    assert_eq!(collector.error_count(ErrorType::SessionNotFound), 1);
    assert_eq!(collector.error_count(ErrorType::VertexNotFound), 1);
    assert_eq!(collector.error_count(ErrorType::InvalidInput), 1);
    assert_eq!(collector.error_count(ErrorType::Internal), 1);
    assert_eq!(collector.error_count(ErrorType::RateLimited), 1);
    assert_eq!(collector.error_count(ErrorType::Timeout), 1);
}

#[test]
fn test_vertices_total() {
    let collector = MetricsCollector::new();
    assert_eq!(collector.vertices_total(), 0);

    collector.inc_vertices(5);
    assert_eq!(collector.vertices_total(), 5);

    collector.inc_vertices(3);
    assert_eq!(collector.vertices_total(), 8);
}

#[test]
fn test_prometheus_export_errors() {
    let collector = MetricsCollector::new();
    collector.record_error(ErrorType::Internal);
    collector.record_error(ErrorType::RateLimited);

    let output = collector.export_prometheus();
    assert!(output.contains("rhizocrypt_rpc_errors_total"));
    assert!(output.contains("type=\"internal\""));
    assert!(output.contains("type=\"rate_limited\""));
}

#[test]
fn test_prometheus_export_latency_mean() {
    let collector = MetricsCollector::new();
    collector.record_request(RpcMethod::AppendEvent, Duration::from_millis(25));

    let output = collector.export_prometheus();
    assert!(output.contains("rhizocrypt_rpc_request_duration_seconds_mean"));
    assert!(output.contains("method=\"append_event\""));
}

#[test]
fn test_metrics_collector_default() {
    let collector = MetricsCollector::default();
    assert_eq!(collector.request_count(RpcMethod::Health), 0);
}

#[test]
fn test_uptime_seconds() {
    let collector = MetricsCollector::new();
    let uptime = collector.uptime_seconds();
    assert!(uptime >= 0.0);
}
