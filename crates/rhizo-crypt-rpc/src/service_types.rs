// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2024–2026 ecoPrimals Project

//! RPC wire types for rhizoCrypt.
//!
//! Stable request/response DTOs shared between tarpc service trait, JSON-RPC
//! handler, and downstream client crate. Extracted from `service.rs` for
//! compile-time boundary: types change less often than behavior.

use rhizo_crypt_core::niche;
use rhizo_crypt_core::{
    Did, EventType, SessionId, SessionState, SessionType, SliceMode, Timestamp, VertexId,
};
use serde::{Deserialize, Serialize};

// ============================================================================
// Request/Response Types
// ============================================================================

/// Session creation request.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateSessionRequest {
    /// Session type.
    pub session_type: SessionType,
    /// Optional description.
    pub description: Option<String>,
    /// Optional parent session.
    pub parent_session: Option<SessionId>,
    /// Maximum vertices allowed.
    pub max_vertices: Option<u64>,
    /// TTL in seconds.
    pub ttl_seconds: Option<u64>,
}

/// Session info response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionInfo {
    /// Session ID.
    pub id: SessionId,
    /// Session type.
    pub session_type: SessionType,
    /// Current state.
    pub state: SessionState,
    /// Vertex count.
    pub vertex_count: u64,
    /// Creation time.
    pub created_at: Timestamp,
    /// Description.
    pub description: Option<String>,
}

/// Event append request.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppendEventRequest {
    /// Target session.
    pub session_id: SessionId,
    /// Event type.
    pub event_type: EventType,
    /// Agent DID.
    pub agent: Option<Did>,
    /// Parent vertices (empty = use frontier).
    pub parents: Vec<VertexId>,
    /// Metadata key-value pairs.
    pub metadata: Vec<(String, String)>,
    /// Optional payload reference.
    pub payload_ref: Option<String>,
}

/// Query request for vertices.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryRequest {
    /// Session to query.
    pub session_id: SessionId,
    /// Filter by event types.
    pub event_types: Option<Vec<EventType>>,
    /// Filter by agent.
    pub agent: Option<Did>,
    /// Start time filter.
    pub start_time: Option<Timestamp>,
    /// End time filter.
    pub end_time: Option<Timestamp>,
    /// Maximum results.
    pub limit: Option<u32>,
}

/// Slice checkout request.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CheckoutSliceRequest {
    /// Spine ID from permanent storage commit.
    pub spine_id: String,
    /// Entry hash from permanent storage commit (hex-encoded, 32 bytes).
    pub entry_hash: String,
    /// Entry index in the spine.
    pub entry_index: u64,
    /// Slice mode.
    pub mode: SliceMode,
    /// Owner DID (lender).
    pub owner: Did,
    /// Holder DID (borrower).
    pub holder: Did,
    /// Session ID to associate the slice with.
    pub session_id: SessionId,
    /// Vertex ID marking the checkout point.
    pub checkout_vertex: VertexId,
    /// Optional certificate ID from the spine.
    pub certificate_id: Option<String>,
    /// Duration in seconds.
    pub duration_seconds: Option<u64>,
}

/// Health status response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthStatus {
    /// Whether the service is healthy.
    pub healthy: bool,
    /// Current state description.
    pub state: String,
    /// Active session count.
    pub active_sessions: u64,
    /// Total vertices in memory.
    pub total_vertices: u64,
    /// Uptime in seconds.
    pub uptime_seconds: u64,
}

/// Service metrics.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceMetrics {
    /// Sessions created.
    pub sessions_created: u64,
    /// Sessions resolved.
    pub sessions_resolved: u64,
    /// Vertices appended.
    pub vertices_appended: u64,
    /// Queries executed.
    pub queries_executed: u64,
    /// Slices checked out.
    pub slices_checked_out: u64,
    /// Dehydrations completed.
    pub dehydrations_completed: u64,
}

// ============================================================================
// Capability Descriptors
// ============================================================================

/// Capability descriptor per Spring-as-Niche deployment standard.
///
/// Describes a capability this primal exposes for runtime discovery.
/// Enhanced with `cost` and `deps` per method to support orchestrator
/// scheduling (aligned with ecosystem Pathway Learner pattern).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapabilityDescriptor {
    /// Capability domain (e.g. "dag", "health").
    pub domain: String,
    /// Semantic method names within this domain.
    pub methods: Vec<MethodDescriptor>,
    /// Protocol version.
    pub version: String,
}

/// Per-method descriptor with cost and dependency information.
///
/// The biomeOS Pathway Learner uses `cost` and `deps` to optimize
/// graph execution order and parallelization.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MethodDescriptor {
    /// Fully qualified method name (e.g. "dag.session.create").
    pub name: String,
    /// Cost tier: "low" (<=2ms), "medium" (3-10ms), "high" (>10ms).
    pub cost: String,
    /// Prerequisite operations that must complete before this one.
    pub deps: Vec<String>,
}

/// Build the full capability descriptor list from `niche.rs` constants.
///
/// Groups capabilities by domain and attaches per-method cost/deps from
/// the niche module, ensuring a single source of truth.
#[must_use]
pub fn build_capability_descriptors() -> Vec<CapabilityDescriptor> {
    use std::collections::BTreeMap;

    let deps = niche::operation_dependencies();

    let mut domain_methods: BTreeMap<String, Vec<MethodDescriptor>> = BTreeMap::new();

    for &(method, estimated_ms, _gpu) in niche::COST_ESTIMATES {
        let domain = method.split('.').next().unwrap_or("unknown").to_string();

        let method_deps = deps
            .get(method)
            .and_then(serde_json::Value::as_array)
            .map(|arr| arr.iter().filter_map(serde_json::Value::as_str).map(String::from).collect())
            .unwrap_or_default();

        domain_methods.entry(domain).or_default().push(MethodDescriptor {
            name: method.to_string(),
            cost: niche::cost_tier(estimated_ms).to_string(),
            deps: method_deps,
        });
    }

    domain_methods
        .into_iter()
        .map(|(domain, methods)| CapabilityDescriptor {
            domain,
            methods,
            version: niche::PRIMAL_VERSION.to_string(),
        })
        .collect()
}

/// Cached capability descriptors — computed once, returned by reference.
///
/// The descriptor list is derived from compile-time constants in `niche.rs`
/// and never changes during the process lifetime. `OnceLock` avoids
/// rebuilding `String` metadata on every `capabilities.list` call.
pub fn cached_capability_descriptors() -> &'static [CapabilityDescriptor] {
    static CACHE: std::sync::OnceLock<Vec<CapabilityDescriptor>> = std::sync::OnceLock::new();
    CACHE.get_or_init(build_capability_descriptors)
}
