// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2024–2026 ecoPrimals Project

//! Niche self-knowledge for the rhizoCrypt primal.
//!
//! Follows the ecoPrimals niche pattern. Every primal defines its self-knowledge
//! in a single module so that orchestrators and discovery providers can reason
//! about it without hardcoded primal names or port numbers.
//!
//! ## Single Source of Truth
//!
//! All method metadata lives in [`METHOD_CATALOG`]. The separate capability
//! lists, cost estimates, semantic mappings, and domain structures are all
//! derived from this one catalog — adding a new capability means a single edit.
//!
//! Other modules reference these constants rather than duplicating string
//! literals. rhizoCrypt only knows itself — it discovers other primals at
//! runtime via capability-based discovery through Songbird.

#[path = "niche_derived.rs"]
mod niche_derived;

pub use niche_derived::{
    CONSUMED_CAPABILITIES, DEPENDENCIES, DOMAIN_DESCRIPTIONS, PROVENANCE_ALIASES,
    SEMANTIC_MAPPINGS, announce_payload, capability_list, cost_tier, health_liveness,
    health_readiness, identity_get, mcp_tools, method_locality_counts, normalize_method,
    operation_dependencies,
};

use std::sync::LazyLock;

// ============================================================================
// PRIMAL IDENTITY
// ============================================================================

/// Primal identity — used in all JSON-RPC, IPC, and biomeOS interactions.
pub const PRIMAL_ID: &str = "rhizocrypt";

/// Environment variable prefix for this primal (e.g., `RHIZOCRYPT_PORT`).
pub const ENV_PREFIX: &str = "RHIZOCRYPT";

/// Human-readable description for biomeOS registration.
pub const PRIMAL_DESCRIPTION: &str =
    "Ephemeral content-addressed DAG engine for session-scoped working memory";

/// Primary capability domain.
pub const DOMAIN: &str = "dag";

/// Primal version (tracks crate version).
pub const PRIMAL_VERSION: &str = env!("CARGO_PKG_VERSION");

/// SPDX license identifier.
pub const LICENSE: &str = "AGPL-3.0-or-later";

/// IPC transport mechanism.
pub const TRANSPORT: &str = "http+tarpc";

/// Wire protocol.
pub const PROTOCOL: &str = "jsonrpc_2.0";

// ============================================================================
// METHOD CATALOG — Single Source of Truth
// ============================================================================

/// Complete metadata for one method this primal exposes.
#[derive(Clone, Debug)]
pub struct MethodSpec {
    /// Fully qualified name (e.g., "dag.session.create").
    pub fqn: &'static str,
    /// Domain prefix (e.g., "dag").
    pub domain: &'static str,
    /// Short name within the domain (e.g., "session.create").
    pub short_name: &'static str,
    /// Estimated latency in milliseconds for Pathway Learner scheduling.
    pub estimated_ms: u32,
    /// Whether this operation benefits from GPU acceleration.
    pub gpu_beneficial: bool,
    /// Whether this method requires external IPC routing (false = in-process).
    pub external: bool,
    /// Operations this method depends on (for parallelization DAG).
    pub deps: &'static [&'static str],
}

/// The single source of truth for every method this primal exposes.
///
/// [`CAPABILITIES`], cost estimates, semantic mappings, and domain groupings
/// are all derived from this catalog. To add a new capability, add one entry
/// here — everything else follows.
pub const METHOD_CATALOG: &[MethodSpec] = &[
    // Session lifecycle — fast, in-memory
    MethodSpec {
        fqn: "dag.session.create",
        domain: "dag",
        short_name: "session.create",
        estimated_ms: 1,
        gpu_beneficial: false,
        external: false,
        deps: &[],
    },
    MethodSpec {
        fqn: "dag.session.get",
        domain: "dag",
        short_name: "session.get",
        estimated_ms: 1,
        gpu_beneficial: false,
        external: false,
        deps: &[],
    },
    MethodSpec {
        fqn: "dag.session.list",
        domain: "dag",
        short_name: "session.list",
        estimated_ms: 1,
        gpu_beneficial: false,
        external: false,
        deps: &[],
    },
    MethodSpec {
        fqn: "dag.session.discard",
        domain: "dag",
        short_name: "session.discard",
        estimated_ms: 2,
        gpu_beneficial: false,
        external: false,
        deps: &["dag.session.create"],
    },
    // Event operations — BLAKE3 hashing + DashMap insert
    MethodSpec {
        fqn: "dag.event.append",
        domain: "dag",
        short_name: "event.append",
        estimated_ms: 2,
        gpu_beneficial: false,
        external: false,
        deps: &["dag.session.create"],
    },
    MethodSpec {
        fqn: "dag.event.append_batch",
        domain: "dag",
        short_name: "event.append_batch",
        estimated_ms: 5,
        gpu_beneficial: false,
        external: false,
        deps: &["dag.session.create"],
    },
    // Vertex queries — DashMap lookup
    MethodSpec {
        fqn: "dag.vertex.get",
        domain: "dag",
        short_name: "vertex.get",
        estimated_ms: 1,
        gpu_beneficial: false,
        external: false,
        deps: &["dag.event.append"],
    },
    MethodSpec {
        fqn: "dag.vertex.query",
        domain: "dag",
        short_name: "vertex.query",
        estimated_ms: 3,
        gpu_beneficial: false,
        external: false,
        deps: &["dag.event.append"],
    },
    MethodSpec {
        fqn: "dag.vertex.children",
        domain: "dag",
        short_name: "vertex.children",
        estimated_ms: 1,
        gpu_beneficial: false,
        external: false,
        deps: &["dag.event.append"],
    },
    // DAG topology — DashMap scan
    MethodSpec {
        fqn: "dag.frontier.get",
        domain: "dag",
        short_name: "frontier.get",
        estimated_ms: 2,
        gpu_beneficial: false,
        external: false,
        deps: &["dag.event.append"],
    },
    MethodSpec {
        fqn: "dag.genesis.get",
        domain: "dag",
        short_name: "genesis.get",
        estimated_ms: 2,
        gpu_beneficial: false,
        external: false,
        deps: &["dag.event.append"],
    },
    // Merkle operations — BLAKE3 tree computation
    MethodSpec {
        fqn: "dag.merkle.root",
        domain: "dag",
        short_name: "merkle.root",
        estimated_ms: 5,
        gpu_beneficial: false,
        external: false,
        deps: &["dag.event.append"],
    },
    MethodSpec {
        fqn: "dag.merkle.proof",
        domain: "dag",
        short_name: "merkle.proof",
        estimated_ms: 3,
        gpu_beneficial: false,
        external: false,
        deps: &["dag.event.append"],
    },
    MethodSpec {
        fqn: "dag.merkle.verify",
        domain: "dag",
        short_name: "merkle.verify",
        estimated_ms: 2,
        gpu_beneficial: false,
        external: false,
        deps: &[],
    },
    // Slice operations — may involve permanent storage I/O
    MethodSpec {
        fqn: "dag.slice.checkout",
        domain: "dag",
        short_name: "slice.checkout",
        estimated_ms: 10,
        gpu_beneficial: false,
        external: false,
        deps: &[],
    },
    MethodSpec {
        fqn: "dag.slice.get",
        domain: "dag",
        short_name: "slice.get",
        estimated_ms: 1,
        gpu_beneficial: false,
        external: false,
        deps: &["dag.slice.checkout"],
    },
    MethodSpec {
        fqn: "dag.slice.list",
        domain: "dag",
        short_name: "slice.list",
        estimated_ms: 1,
        gpu_beneficial: false,
        external: false,
        deps: &[],
    },
    MethodSpec {
        fqn: "dag.slice.resolve",
        domain: "dag",
        short_name: "slice.resolve",
        estimated_ms: 5,
        gpu_beneficial: false,
        external: false,
        deps: &["dag.slice.checkout"],
    },
    // Branch/Diff/Merge/Federate — version control over DAG composition (Wave 60)
    MethodSpec {
        fqn: "dag.branch",
        domain: "dag",
        short_name: "branch",
        estimated_ms: 10,
        gpu_beneficial: false,
        external: false,
        deps: &["dag.session.create", "dag.event.append"],
    },
    MethodSpec {
        fqn: "dag.diff",
        domain: "dag",
        short_name: "diff",
        estimated_ms: 5,
        gpu_beneficial: false,
        external: false,
        deps: &["dag.session.create"],
    },
    MethodSpec {
        fqn: "dag.merge",
        domain: "dag",
        short_name: "merge",
        estimated_ms: 5,
        gpu_beneficial: false,
        external: false,
        deps: &["dag.session.create", "dag.event.append"],
    },
    MethodSpec {
        fqn: "dag.federate",
        domain: "dag",
        short_name: "federate",
        estimated_ms: 20,
        gpu_beneficial: false,
        external: false,
        deps: &["dag.session.create"],
    },
    // Partial dehydration — read-only Merkle root without closing session
    MethodSpec {
        fqn: "dag.partial_dehydrate",
        domain: "dag",
        short_name: "partial_dehydrate",
        estimated_ms: 10,
        gpu_beneficial: false,
        external: false,
        deps: &["dag.session.create", "dag.event.append"],
    },
    // Dehydration — multi-step I/O to permanent storage
    MethodSpec {
        fqn: "dag.dehydration.trigger",
        domain: "dag",
        short_name: "dehydration.trigger",
        estimated_ms: 50,
        gpu_beneficial: false,
        external: false,
        deps: &["dag.session.create", "dag.event.append"],
    },
    MethodSpec {
        fqn: "dag.dehydration.status",
        domain: "dag",
        short_name: "dehydration.status",
        estimated_ms: 1,
        gpu_beneficial: false,
        external: false,
        deps: &["dag.dehydration.trigger"],
    },
    // Health and introspection
    MethodSpec {
        fqn: "health.check",
        domain: "health",
        short_name: "check",
        estimated_ms: 1,
        gpu_beneficial: false,
        external: false,
        deps: &[],
    },
    MethodSpec {
        fqn: "health.liveness",
        domain: "health",
        short_name: "liveness",
        estimated_ms: 1,
        gpu_beneficial: false,
        external: false,
        deps: &[],
    },
    MethodSpec {
        fqn: "health.readiness",
        domain: "health",
        short_name: "readiness",
        estimated_ms: 1,
        gpu_beneficial: false,
        external: false,
        deps: &[],
    },
    MethodSpec {
        fqn: "health.metrics",
        domain: "health",
        short_name: "metrics",
        estimated_ms: 1,
        gpu_beneficial: false,
        external: false,
        deps: &[],
    },
    MethodSpec {
        fqn: "capabilities.list",
        domain: "capabilities",
        short_name: "list",
        estimated_ms: 1,
        gpu_beneficial: false,
        external: false,
        deps: &[],
    },
    // Identity
    MethodSpec {
        fqn: "identity.get",
        domain: "identity",
        short_name: "get",
        estimated_ms: 1,
        gpu_beneficial: false,
        external: false,
        deps: &[],
    },
    // MCP tool exposure (AI coordination layer)
    MethodSpec {
        fqn: "tools.list",
        domain: "tools",
        short_name: "list",
        estimated_ms: 1,
        gpu_beneficial: false,
        external: false,
        deps: &[],
    },
    MethodSpec {
        fqn: "tools.call",
        domain: "tools",
        short_name: "call",
        estimated_ms: 5,
        gpu_beneficial: false,
        external: false,
        deps: &[],
    },
    // Mesh trust events — cross-gate DAG event recording (Wave 76c)
    MethodSpec {
        fqn: "mesh.events.record",
        domain: "mesh",
        short_name: "events.record",
        estimated_ms: 3,
        gpu_beneficial: false,
        external: false,
        deps: &["dag.session.create", "dag.event.append"],
    },
    // Auth introspection — JH-0 method gate (public, always allowed)
    MethodSpec {
        fqn: "auth.check",
        domain: "auth",
        short_name: "check",
        estimated_ms: 1,
        gpu_beneficial: false,
        external: false,
        deps: &[],
    },
    MethodSpec {
        fqn: "auth.mode",
        domain: "auth",
        short_name: "mode",
        estimated_ms: 1,
        gpu_beneficial: false,
        external: false,
        deps: &[],
    },
    MethodSpec {
        fqn: "auth.peer_info",
        domain: "auth",
        short_name: "peer_info",
        estimated_ms: 1,
        gpu_beneficial: false,
        external: false,
        deps: &[],
    },
];

/// Flat list of all capability FQN strings this primal exposes.
///
/// Derived from [`METHOD_CATALOG`] at startup. Used by JSON-RPC dispatch
/// and biomeOS capability advertisement.
pub static CAPABILITIES: LazyLock<Vec<&'static str>> =
    LazyLock::new(|| METHOD_CATALOG.iter().map(|m| m.fqn).collect());

#[cfg(test)]
#[expect(clippy::expect_used, reason = "test code")]
#[path = "niche_tests.rs"]
mod tests;
