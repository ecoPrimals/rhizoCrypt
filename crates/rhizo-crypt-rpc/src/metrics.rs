// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2024–2026 ecoPrimals Project

//! Prometheus metrics for RPC layer.
//!
//! Provides observability into `rhizoCrypt`'s RPC operations including:
//! - Request counts and latencies
//! - Session and vertex statistics
//! - Error rates by type
//!
//! ## Metric Names
//!
//! All metrics are prefixed with `rhizocrypt_` for namespace isolation.
//!
//! | Metric | Type | Description |
//! |--------|------|-------------|
//! | `rhizocrypt_rpc_requests_total` | Counter | Total RPC requests by method |
//! | `rhizocrypt_rpc_request_duration_seconds` | Histogram | Request latency |
//! | `rhizocrypt_rpc_errors_total` | Counter | Errors by type |
//! | `rhizocrypt_sessions_active` | Gauge | Currently active sessions |
//! | `rhizocrypt_vertices_total` | Counter | Total vertices created |

use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

/// Metric labels for RPC methods.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RpcMethod {
    /// Session creation.
    CreateSession,
    /// Session retrieval.
    GetSession,
    /// List sessions.
    ListSessions,
    /// Discard session.
    DiscardSession,
    /// Append event.
    AppendEvent,
    /// Append batch.
    AppendBatch,
    /// Get vertex.
    GetVertex,
    /// Get frontier.
    GetFrontier,
    /// Get genesis.
    GetGenesis,
    /// Query vertices.
    QueryVertices,
    /// Get children.
    GetChildren,
    /// Get merkle root.
    GetMerkleRoot,
    /// Get merkle proof.
    GetMerkleProof,
    /// Verify proof.
    VerifyProof,
    /// Checkout slice.
    CheckoutSlice,
    /// Get slice.
    GetSlice,
    /// List slices.
    ListSlices,
    /// Resolve slice.
    ResolveSlice,
    /// Dehydrate.
    Dehydrate,
    /// Get dehydration status.
    GetDehydrationStatus,
    /// Health check.
    Health,
    /// Metrics.
    Metrics,
}

impl RpcMethod {
    /// Get the method name as a string.
    #[must_use]
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::CreateSession => "create_session",
            Self::GetSession => "get_session",
            Self::ListSessions => "list_sessions",
            Self::DiscardSession => "discard_session",
            Self::AppendEvent => "append_event",
            Self::AppendBatch => "append_batch",
            Self::GetVertex => "get_vertex",
            Self::GetFrontier => "get_frontier",
            Self::GetGenesis => "get_genesis",
            Self::QueryVertices => "query_vertices",
            Self::GetChildren => "get_children",
            Self::GetMerkleRoot => "get_merkle_root",
            Self::GetMerkleProof => "get_merkle_proof",
            Self::VerifyProof => "verify_proof",
            Self::CheckoutSlice => "checkout_slice",
            Self::GetSlice => "get_slice",
            Self::ListSlices => "list_slices",
            Self::ResolveSlice => "resolve_slice",
            Self::Dehydrate => "dehydrate",
            Self::GetDehydrationStatus => "get_dehydration_status",
            Self::Health => "health",
            Self::Metrics => "metrics",
        }
    }
}

/// Error types for metric tracking.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ErrorType {
    /// Session not found.
    SessionNotFound,
    /// Vertex not found.
    VertexNotFound,
    /// Invalid input.
    InvalidInput,
    /// Internal error.
    Internal,
    /// Rate limited.
    RateLimited,
    /// Timeout.
    Timeout,
}

impl ErrorType {
    /// Get the error type as a string.
    #[must_use]
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::SessionNotFound => "session_not_found",
            Self::VertexNotFound => "vertex_not_found",
            Self::InvalidInput => "invalid_input",
            Self::Internal => "internal",
            Self::RateLimited => "rate_limited",
            Self::Timeout => "timeout",
        }
    }
}

/// Histogram bucket boundaries for latency (in seconds).
const LATENCY_BUCKETS: [f64; 12] = [
    0.0001, // 100µs
    0.0005, // 500µs
    0.001,  // 1ms
    0.005,  // 5ms
    0.01,   // 10ms
    0.025,  // 25ms
    0.05,   // 50ms
    0.1,    // 100ms
    0.25,   // 250ms
    0.5,    // 500ms
    1.0,    // 1s
    5.0,    // 5s
];

/// Simple histogram for latency tracking.
#[derive(Debug)]
struct Histogram {
    /// Bucket counts.
    buckets: [AtomicU64; 12],
    /// Sum of all observations.
    sum: AtomicU64,
    /// Count of observations.
    count: AtomicU64,
}

impl Default for Histogram {
    fn default() -> Self {
        Self {
            buckets: std::array::from_fn(|_| AtomicU64::new(0)),
            sum: AtomicU64::new(0),
            count: AtomicU64::new(0),
        }
    }
}

impl Histogram {
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)] // Acceptable for metrics
    fn observe(&self, value: f64) {
        // Increment appropriate bucket
        for (i, &bound) in LATENCY_BUCKETS.iter().enumerate() {
            if value <= bound {
                self.buckets[i].fetch_add(1, Ordering::Relaxed);
                break;
            }
        }

        // Update sum and count (using integer representation for atomicity)
        let value_micros = (value * 1_000_000.0) as u64;
        self.sum.fetch_add(value_micros, Ordering::Relaxed);
        self.count.fetch_add(1, Ordering::Relaxed);
    }

    fn snapshot(&self) -> HistogramSnapshot {
        HistogramSnapshot {
            buckets: std::array::from_fn(|i| self.buckets[i].load(Ordering::Relaxed)),
            sum_micros: self.sum.load(Ordering::Relaxed),
            count: self.count.load(Ordering::Relaxed),
        }
    }
}

/// Snapshot of histogram data.
#[derive(Debug, Clone)]
pub struct HistogramSnapshot {
    /// Bucket counts.
    pub buckets: [u64; 12],
    /// Sum of observations in microseconds.
    pub sum_micros: u64,
    /// Total observation count.
    pub count: u64,
}

impl HistogramSnapshot {
    /// Get the bucket boundaries.
    #[must_use]
    pub const fn bucket_bounds() -> &'static [f64; 12] {
        &LATENCY_BUCKETS
    }

    /// Get the mean latency in seconds.
    #[must_use]
    #[allow(clippy::cast_precision_loss)] // Acceptable for metrics
    pub fn mean_seconds(&self) -> f64 {
        if self.count == 0 {
            0.0
        } else {
            (self.sum_micros as f64 / 1_000_000.0) / self.count as f64
        }
    }
}

/// Prometheus-compatible metrics collector.
#[derive(Debug)]
pub struct MetricsCollector {
    /// Request counts by method.
    requests: [AtomicU64; 24],
    /// Request latency histograms by method.
    latencies: [Histogram; 24],
    /// Error counts by type.
    errors: [AtomicU64; 6],
    /// Active sessions gauge.
    active_sessions: AtomicU64,
    /// Total vertices created.
    vertices_total: AtomicU64,
    /// Start time for uptime calculation.
    start_time: Instant,
}

impl Default for MetricsCollector {
    fn default() -> Self {
        Self::new()
    }
}

impl MetricsCollector {
    /// Create a new metrics collector.
    #[must_use]
    pub fn new() -> Self {
        Self {
            requests: std::array::from_fn(|_| AtomicU64::new(0)),
            latencies: std::array::from_fn(|_| Histogram::default()),
            errors: std::array::from_fn(|_| AtomicU64::new(0)),
            active_sessions: AtomicU64::new(0),
            vertices_total: AtomicU64::new(0),
            start_time: Instant::now(),
        }
    }

    /// Record a request.
    pub fn record_request(&self, method: RpcMethod, duration: Duration) {
        let idx = method as usize;
        self.requests[idx].fetch_add(1, Ordering::Relaxed);
        self.latencies[idx].observe(duration.as_secs_f64());
    }

    /// Record an error.
    pub fn record_error(&self, error_type: ErrorType) {
        let idx = error_type as usize;
        self.errors[idx].fetch_add(1, Ordering::Relaxed);
    }

    /// Increment active sessions.
    pub fn inc_sessions(&self) {
        self.active_sessions.fetch_add(1, Ordering::Relaxed);
    }

    /// Decrement active sessions.
    pub fn dec_sessions(&self) {
        self.active_sessions.fetch_sub(1, Ordering::Relaxed);
    }

    /// Set active sessions count.
    pub fn set_sessions(&self, count: u64) {
        self.active_sessions.store(count, Ordering::Relaxed);
    }

    /// Increment vertex count.
    pub fn inc_vertices(&self, count: u64) {
        self.vertices_total.fetch_add(count, Ordering::Relaxed);
    }

    /// Get request count for a method.
    #[must_use]
    pub fn request_count(&self, method: RpcMethod) -> u64 {
        self.requests[method as usize].load(Ordering::Relaxed)
    }

    /// Get latency histogram for a method.
    #[must_use]
    pub fn latency_histogram(&self, method: RpcMethod) -> HistogramSnapshot {
        self.latencies[method as usize].snapshot()
    }

    /// Get error count for a type.
    #[must_use]
    pub fn error_count(&self, error_type: ErrorType) -> u64 {
        self.errors[error_type as usize].load(Ordering::Relaxed)
    }

    /// Get active session count.
    #[must_use]
    pub fn active_sessions(&self) -> u64 {
        self.active_sessions.load(Ordering::Relaxed)
    }

    /// Get total vertex count.
    #[must_use]
    pub fn vertices_total(&self) -> u64 {
        self.vertices_total.load(Ordering::Relaxed)
    }

    /// Get uptime in seconds.
    #[must_use]
    pub fn uptime_seconds(&self) -> f64 {
        self.start_time.elapsed().as_secs_f64()
    }

    /// Export metrics in Prometheus text format.
    #[must_use]
    pub fn export_prometheus(&self) -> String {
        use std::fmt::Write;

        let mut output = String::with_capacity(4096);

        // Uptime
        output.push_str("# HELP rhizocrypt_uptime_seconds Time since service start\n");
        output.push_str("# TYPE rhizocrypt_uptime_seconds gauge\n");
        let _ = writeln!(output, "rhizocrypt_uptime_seconds {:.3}\n", self.uptime_seconds());

        // Active sessions
        output.push_str("# HELP rhizocrypt_sessions_active Currently active sessions\n");
        output.push_str("# TYPE rhizocrypt_sessions_active gauge\n");
        let _ = writeln!(output, "rhizocrypt_sessions_active {}\n", self.active_sessions());

        // Total vertices
        output.push_str("# HELP rhizocrypt_vertices_total Total vertices created\n");
        output.push_str("# TYPE rhizocrypt_vertices_total counter\n");
        let _ = writeln!(output, "rhizocrypt_vertices_total {}\n", self.vertices_total());

        // Request counts
        output.push_str("# HELP rhizocrypt_rpc_requests_total Total RPC requests\n");
        output.push_str("# TYPE rhizocrypt_rpc_requests_total counter\n");
        for method in ALL_METHODS {
            let count = self.request_count(method);
            if count > 0 {
                let _ = writeln!(
                    output,
                    "rhizocrypt_rpc_requests_total{{method=\"{}\"}} {}",
                    method.as_str(),
                    count
                );
            }
        }
        output.push('\n');

        // Error counts
        output.push_str("# HELP rhizocrypt_rpc_errors_total Total RPC errors\n");
        output.push_str("# TYPE rhizocrypt_rpc_errors_total counter\n");
        for error_type in ALL_ERROR_TYPES {
            let count = self.error_count(error_type);
            if count > 0 {
                let _ = writeln!(
                    output,
                    "rhizocrypt_rpc_errors_total{{type=\"{}\"}} {}",
                    error_type.as_str(),
                    count
                );
            }
        }
        output.push('\n');

        // Request latencies (simplified - just mean)
        output.push_str(
            "# HELP rhizocrypt_rpc_request_duration_seconds_mean Mean request duration\n",
        );
        output.push_str("# TYPE rhizocrypt_rpc_request_duration_seconds_mean gauge\n");
        for method in ALL_METHODS {
            let hist = self.latency_histogram(method);
            if hist.count > 0 {
                let _ = writeln!(
                    output,
                    "rhizocrypt_rpc_request_duration_seconds_mean{{method=\"{}\"}} {:.6}",
                    method.as_str(),
                    hist.mean_seconds()
                );
            }
        }

        output
    }
}

/// All RPC methods for iteration.
const ALL_METHODS: [RpcMethod; 24] = [
    RpcMethod::CreateSession,
    RpcMethod::GetSession,
    RpcMethod::ListSessions,
    RpcMethod::DiscardSession,
    RpcMethod::AppendEvent,
    RpcMethod::AppendBatch,
    RpcMethod::GetVertex,
    RpcMethod::GetFrontier,
    RpcMethod::GetGenesis,
    RpcMethod::QueryVertices,
    RpcMethod::GetChildren,
    RpcMethod::GetMerkleRoot,
    RpcMethod::GetMerkleProof,
    RpcMethod::VerifyProof,
    RpcMethod::CheckoutSlice,
    RpcMethod::GetSlice,
    RpcMethod::ListSlices,
    RpcMethod::ResolveSlice,
    RpcMethod::Dehydrate,
    RpcMethod::GetDehydrationStatus,
    RpcMethod::Health,
    RpcMethod::Metrics,
    RpcMethod::Health,  // Padding to reach 24
    RpcMethod::Metrics, // Padding to reach 24
];

/// All error types for iteration.
const ALL_ERROR_TYPES: [ErrorType; 6] = [
    ErrorType::SessionNotFound,
    ErrorType::VertexNotFound,
    ErrorType::InvalidInput,
    ErrorType::Internal,
    ErrorType::RateLimited,
    ErrorType::Timeout,
];

/// Shared metrics instance for global access.
#[derive(Clone)]
pub struct SharedMetrics(Arc<MetricsCollector>);

impl SharedMetrics {
    /// Create a new shared metrics instance.
    #[must_use]
    pub fn new() -> Self {
        Self(Arc::new(MetricsCollector::new()))
    }

    /// Get reference to the collector.
    #[must_use]
    pub fn collector(&self) -> &MetricsCollector {
        &self.0
    }
}

impl Default for SharedMetrics {
    fn default() -> Self {
        Self::new()
    }
}

/// Request timer for automatic latency recording.
pub struct RequestTimer<'a> {
    collector: &'a MetricsCollector,
    method: RpcMethod,
    start: Instant,
}

impl<'a> RequestTimer<'a> {
    /// Start timing a request.
    #[must_use]
    pub fn start(collector: &'a MetricsCollector, method: RpcMethod) -> Self {
        Self {
            collector,
            method,
            start: Instant::now(),
        }
    }

    /// Finish timing and record the duration.
    pub fn finish(self) {
        self.collector.record_request(self.method, self.start.elapsed());
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;

    #[test]
    fn test_metrics_collector() {
        let collector = MetricsCollector::new();

        // Record some requests
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

        // Record various latencies
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
            // Simulate work without blocking
            tokio::task::yield_now().await;
            timer.finish();
        }

        assert_eq!(collector.request_count(RpcMethod::Health), 1);
        let hist = collector.latency_histogram(RpcMethod::Health);
        // Latency should be non-negative (we don't check exact value)
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
}
