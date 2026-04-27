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

use std::sync::LazyLock;

// ============================================================================
// PRIMAL IDENTITY
// ============================================================================

/// Primal identity — used in all JSON-RPC, IPC, and biomeOS interactions.
pub const PRIMAL_ID: &str = "rhizocrypt";

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
];

/// Flat list of all capability FQN strings this primal exposes.
///
/// Derived from [`METHOD_CATALOG`] at startup. Used by JSON-RPC dispatch
/// and biomeOS capability advertisement.
pub static CAPABILITIES: LazyLock<Vec<&'static str>> =
    LazyLock::new(|| METHOD_CATALOG.iter().map(|m| m.fqn).collect());

// ============================================================================
// SEMANTIC ALIASES
// ============================================================================

/// Extra semantic aliases beyond the standard `short_name → fqn` mapping.
///
/// These allow callers to use shorter or alternative names when routing
/// through `capability.call`. Standard mappings (e.g., "session.create" →
/// "dag.session.create") are derived automatically from [`METHOD_CATALOG`].
const SEMANTIC_ALIASES: &[(&str, &str)] = &[
    ("health", "health.check"),
    ("liveness", "health.liveness"),
    ("readiness", "health.readiness"),
    ("metrics", "health.metrics"),
    ("capabilities", "capabilities.list"),
    ("capability.list", "capabilities.list"),
    ("primal.capabilities", "capabilities.list"),
    ("identity", "identity.get"),
    ("tools", "tools.list"),
    ("mcp.tools.list", "tools.list"),
    ("mcp.tools.call", "tools.call"),
];

/// All semantic mappings: standard (`short_name` → `fqn`) + aliases.
///
/// biomeOS uses these during domain registration so
/// `capability.call { domain: "dag", operation: "session.create" }` routes
/// to the correct JSON-RPC method.
pub static SEMANTIC_MAPPINGS: LazyLock<Vec<(&'static str, &'static str)>> = LazyLock::new(|| {
    let mut mappings: Vec<(&str, &str)> =
        METHOD_CATALOG.iter().map(|m| (m.short_name, m.fqn)).collect();
    mappings.extend_from_slice(SEMANTIC_ALIASES);
    mappings
});

// ============================================================================
// CONSUMED CAPABILITIES & DEPENDENCIES
// ============================================================================

/// Consumed capabilities — what rhizoCrypt calls on other primals.
///
/// rhizoCrypt discovers these at runtime via the discovery adapter; it
/// never hardcodes which primal provides them.
pub const CONSUMED_CAPABILITIES: &[&str] = &[
    "crypto.sign",
    "crypto.verify",
    "commit.session",
    "commit.entry",
    "storage.store",
    "storage.get",
    "provenance.create_braid",
    "provenance.lineage",
    "discovery.register",
    "discovery.query",
];

/// Primal dependencies for deployment.
///
/// Each entry: `(capability_domain, required, description)`.
/// Note: these reference capability domains, not primal names.
pub const DEPENDENCIES: &[(&str, bool, &str)] = &[
    ("crypto", false, "vertex signing and verification (graceful fallback to unsigned)"),
    ("discovery", false, "service mesh registration (graceful fallback to standalone)"),
    ("commit", false, "dehydration to permanent storage (graceful fallback to local-only)"),
    ("storage", false, "content-addressed payload storage (graceful fallback to inline)"),
    ("provenance", false, "attribution braids (graceful fallback to unattributed)"),
];

// ============================================================================
// DOMAIN DESCRIPTIONS
// ============================================================================

/// Human-readable descriptions for each capability domain.
const DOMAIN_DESCRIPTIONS: &[(&str, &str)] = &[
    ("dag", "Ephemeral DAG session and vertex operations"),
    ("health", "Health probes and introspection"),
    ("capabilities", "Capability introspection"),
    ("identity", "Primal identity for biomeOS discovery"),
    ("tools", "MCP tool exposure for AI coordination"),
];

// ============================================================================
// DERIVED ACCESSORS
// ============================================================================

/// Cost tier for a given estimated latency.
#[must_use]
pub const fn cost_tier(estimated_ms: u32) -> &'static str {
    if estimated_ms <= crate::constants::COST_TIER_LOW_THRESHOLD_MS {
        "low"
    } else if estimated_ms <= crate::constants::COST_TIER_MEDIUM_THRESHOLD_MS {
        "medium"
    } else {
        "high"
    }
}

/// Returns the count of local (in-process) vs external (IPC-routed) methods.
#[must_use]
pub fn method_locality_counts() -> (usize, usize) {
    let local = METHOD_CATALOG.iter().filter(|m| !m.external).count();
    let external = METHOD_CATALOG.iter().filter(|m| m.external).count();
    (local, external)
}

/// Operation dependency hints for biomeOS Pathway Learner parallelization.
#[must_use]
pub fn operation_dependencies() -> serde_json::Value {
    let map: serde_json::Map<String, serde_json::Value> = METHOD_CATALOG
        .iter()
        .map(|m| {
            let deps: Vec<&str> = m.deps.to_vec();
            (m.fqn.to_string(), serde_json::json!(deps))
        })
        .collect();
    serde_json::Value::Object(map)
}

/// Return the capability list as a JSON-RPC response payload.
///
/// Implements the `capabilities.list` semantic method. Aligns with the ecosystem
/// enhanced format: domain, method, dependencies, cost tier.
#[must_use]
pub fn capability_list() -> serde_json::Value {
    let deps = operation_dependencies();

    let methods: Vec<serde_json::Value> = METHOD_CATALOG
        .iter()
        .map(|m| {
            serde_json::json!({
                "method": m.fqn,
                "domain": m.domain,
                "cost": cost_tier(m.estimated_ms),
                "external": m.external,
                "deps": deps.get(m.fqn).cloned().unwrap_or(serde_json::json!([])),
            })
        })
        .collect();

    let domains: Vec<serde_json::Value> = DOMAIN_DESCRIPTIONS
        .iter()
        .map(|(prefix, description)| {
            let count = METHOD_CATALOG.iter().filter(|m| m.domain == *prefix).count();
            serde_json::json!({
                "prefix": prefix,
                "description": description,
                "method_count": count,
            })
        })
        .collect();

    let (local_count, external_count) = method_locality_counts();

    serde_json::json!({
        "primal": PRIMAL_ID,
        "version": PRIMAL_VERSION,
        "description": PRIMAL_DESCRIPTION,
        "domain": DOMAIN,
        "license": LICENSE,
        "transport": TRANSPORT,
        "protocol": PROTOCOL,
        "capabilities": *CAPABILITIES,
        "consumed_capabilities": CONSUMED_CAPABILITIES,
        "domains": domains,
        "locality": { "local": local_count, "external": external_count },
        "methods": methods,
    })
}

/// Identity probe for biomeOS primal discovery.
#[must_use]
pub fn identity_get() -> serde_json::Value {
    serde_json::json!({
        "primal": PRIMAL_ID,
        "version": PRIMAL_VERSION,
        "domain": DOMAIN,
        "description": PRIMAL_DESCRIPTION,
        "license": LICENSE,
        "transport": TRANSPORT,
        "protocol": PROTOCOL,
    })
}

/// Zero-cost liveness probe.
#[must_use]
pub fn health_liveness() -> serde_json::Value {
    serde_json::json!({ "status": "alive" })
}

/// Readiness probe — checks whether the primal can accept work.
#[must_use]
pub fn health_readiness(is_running: bool) -> serde_json::Value {
    serde_json::json!({
        "status": if is_running { "ready" } else { "not_ready" },
        "primal": PRIMAL_ID,
        "version": PRIMAL_VERSION,
    })
}

/// Normalize a JSON-RPC method name, handling legacy prefixes.
#[must_use]
pub fn normalize_method(method: &str) -> &str {
    method
        .strip_prefix("rhizocrypt.")
        .or_else(|| method.strip_prefix("rhizo_crypt."))
        .unwrap_or(method)
}

/// MCP tool definitions for AI coordination layer.
#[must_use]
pub fn mcp_tools() -> serde_json::Value {
    serde_json::json!([
        {
            "name": "dag.session.create",
            "description": "Create a new ephemeral DAG session for scoped working memory",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "session_type": { "type": "string", "enum": ["Ephemeral", "Persistent"], "default": "Ephemeral" },
                    "description": { "type": "string" },
                    "ttl_seconds": { "type": "integer" }
                }
            }
        },
        {
            "name": "dag.event.append",
            "description": "Append a content-addressed event vertex to a session DAG",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "session_id": { "type": "string", "format": "uuid" },
                    "event_type": { "type": "string" },
                    "agent": { "type": "string" },
                    "parents": { "type": "array", "items": { "type": "string" } },
                    "payload_ref": { "type": "string" }
                },
                "required": ["session_id", "event_type"]
            }
        },
        {
            "name": "dag.merkle.root",
            "description": "Compute the Merkle root hash proving integrity of an entire session DAG",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "session_id": { "type": "string", "format": "uuid" }
                },
                "required": ["session_id"]
            }
        },
        {
            "name": "dag.dehydration.trigger",
            "description": "Dehydrate a session: collapse to summary + commit to permanent storage",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "session_id": { "type": "string", "format": "uuid" }
                },
                "required": ["session_id"]
            }
        },
        {
            "name": "health.check",
            "description": "Check rhizoCrypt service health, uptime, and version",
            "inputSchema": { "type": "object", "properties": {} }
        },
        {
            "name": "capabilities.list",
            "description": "List all capabilities this primal exposes with cost and dependency info",
            "inputSchema": { "type": "object", "properties": {} }
        }
    ])
}

#[cfg(test)]
#[expect(clippy::expect_used, reason = "test code")]
#[path = "niche_tests.rs"]
mod tests;
