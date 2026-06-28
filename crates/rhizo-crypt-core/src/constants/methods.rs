// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2024–2026 ecoPrimals Project

//! JSON-RPC method names, protocol version, and method-prefix allowlists.

// ============================================================================
// JSON-RPC WIRE CONSTANTS
// ============================================================================

/// JSON-RPC 2.0 protocol version string.
pub const JSONRPC_VERSION: &str = "2.0";

/// Provenance RPC method: record a session contribution.
pub const PROVENANCE_RECORD_SESSION_METHOD: &str = "contribution.record_session";

/// Provenance RPC method: record a dehydration contribution.
pub const PROVENANCE_RECORD_DEHYDRATION_METHOD: &str = "contribution.record_dehydration";

/// Provenance RPC method: record a provenance event.
pub const PROVENANCE_RECORD_PROVENANCE_METHOD: &str = "contribution.record_provenance";

/// JSON-RPC method for polling cross-gate trust events from the signing provider.
///
/// Called by `MeshEventListener::poll_events()` against whatever primal
/// provides the `Signing` capability (currently bearDog).
pub const MESH_AUTH_EVENTS_POLL_METHOD: &str = "auth.events.poll";

// ============================================================================
// HEALTH METHOD FQNs (dispatch SSOT — matches METHOD_CATALOG in niche.rs)
// ============================================================================

/// Health check method (full status).
pub const HEALTH_CHECK: &str = "health.check";

/// Health liveness probe (minimal alive signal).
pub const HEALTH_LIVENESS: &str = "health.liveness";

/// Health readiness probe (service ready for traffic).
pub const HEALTH_READINESS: &str = "health.readiness";

/// Health metrics endpoint.
pub const HEALTH_METRICS: &str = "health.metrics";

/// Public method prefix for health domain (gating/allowlist SSOT).
pub const HEALTH_METHOD_PREFIX: &str = "health.";
