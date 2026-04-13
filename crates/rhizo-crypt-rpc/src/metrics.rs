// SPDX-License-Identifier: AGPL-3.0-or-later
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

use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant};

#[path = "histogram.rs"]
mod histogram;
#[path = "prometheus.rs"]
mod prometheus;

#[allow(unused_imports)] // Re-export only; used by downstream crates and tests.
pub use histogram::{Histogram, HistogramSnapshot, LATENCY_BUCKETS};

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
    /// List capabilities.
    ListCapabilities,
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
            Self::ListCapabilities => "list_capabilities",
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

/// Number of distinct `RpcMethod` variants.
const RPC_METHOD_COUNT: usize = 23;

/// Number of distinct `ErrorType` variants.
const ERROR_TYPE_COUNT: usize = 6;

/// Prometheus-compatible metrics collector.
#[derive(Debug)]
pub struct MetricsCollector {
    /// Request counts by method.
    requests: [AtomicU64; RPC_METHOD_COUNT],
    /// Request latency histograms by method.
    latencies: [Histogram; RPC_METHOD_COUNT],
    /// Error counts by type.
    errors: [AtomicU64; ERROR_TYPE_COUNT],
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
        prometheus::export_prometheus(self)
    }
}

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
#[path = "metrics_tests.rs"]
mod tests;
