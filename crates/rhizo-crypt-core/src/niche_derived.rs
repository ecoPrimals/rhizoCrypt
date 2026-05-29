// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2024–2026 ecoPrimals Project

//! Derived accessors and response builders for the rhizoCrypt niche.
//!
//! All functions and constants here derive from [`super::METHOD_CATALOG`]
//! (the Single Source of Truth). Extracted from `niche.rs` to stay under the
//! 800-line threshold while keeping the catalog definition clean.

use super::{
    CAPABILITIES, DOMAIN, LICENSE, METHOD_CATALOG, PRIMAL_DESCRIPTION, PRIMAL_ID, PRIMAL_VERSION,
    PROTOCOL, TRANSPORT,
};
use std::sync::LazyLock;

/// Human-readable descriptions for each capability domain.
pub const DOMAIN_DESCRIPTIONS: &[(&str, &str)] = &[
    ("dag", "Ephemeral DAG session and vertex operations"),
    ("health", "Health probes and introspection"),
    ("capabilities", "Capability introspection"),
    ("identity", "Primal identity for biomeOS discovery"),
    ("tools", "MCP tool exposure for AI coordination"),
    ("auth", "Method gate introspection and authorization (JH-0)"),
];

// ============================================================================
// PROVENANCE ALIASES (GAP-36 resolution)
// ============================================================================

/// `provenance.*` → `dag.*` wire-name aliases (GAP-36 resolution).
///
/// primalSpring's domain contract sweep, healthSpring's Nest atomic, and
/// the broader ecosystem use `provenance.*` as the external wire name
/// for rhizoCrypt's DAG surface. `dag.*` is canonical internally.
pub const PROVENANCE_ALIASES: &[(&str, &str)] = &[
    ("provenance.session.create", "dag.session.create"),
    ("provenance.session.get", "dag.session.get"),
    ("provenance.session.list", "dag.session.list"),
    ("provenance.session.discard", "dag.session.discard"),
    ("provenance.event.append", "dag.event.append"),
    ("provenance.event.append_batch", "dag.event.append_batch"),
    ("provenance.vertex.get", "dag.vertex.get"),
    ("provenance.frontier.get", "dag.frontier.get"),
    ("provenance.genesis.get", "dag.genesis.get"),
    ("provenance.vertex.query", "dag.vertex.query"),
    ("provenance.vertex.children", "dag.vertex.children"),
    ("provenance.merkle.root", "dag.merkle.root"),
    ("provenance.merkle.proof", "dag.merkle.proof"),
    ("provenance.merkle.verify", "dag.merkle.verify"),
    ("provenance.slice.checkout", "dag.slice.checkout"),
    ("provenance.slice.get", "dag.slice.get"),
    ("provenance.slice.list", "dag.slice.list"),
    ("provenance.slice.resolve", "dag.slice.resolve"),
    ("provenance.dehydration.trigger", "dag.dehydration.trigger"),
    ("provenance.dehydrate", "dag.dehydrate"),
    ("provenance.dehydration.status", "dag.dehydration.status"),
    ("provenance.partial_dehydrate", "dag.partial_dehydrate"),
    ("provenance.branch", "dag.branch"),
    ("provenance.diff", "dag.diff"),
    ("provenance.merge", "dag.merge"),
    ("provenance.federate", "dag.federate"),
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

/// Build the `primal.announce` JSON-RPC params for biomeOS Neural API
/// registration. Called once on startup after the UDS socket is bound.
#[must_use]
pub fn announce_payload(socket_path: &str, pid: Option<u32>) -> serde_json::Value {
    let methods: Vec<&str> = CAPABILITIES.iter().copied().collect();

    serde_json::json!({
        "primal": PRIMAL_ID,
        "socket": socket_path,
        "pid": pid,
        "capabilities": ["dag", "integrity", "merkle"],
        "methods": methods,
        "semantic_mappings": semantic_mapping_object(),
        "signal_tiers": ["nest"],
        "cost_hints": {
            "dag": 10.0,
            "integrity": 5.0,
            "merkle": 8.0,
        },
        "latency_estimates": {
            "dag": 15,
            "integrity": 5,
            "merkle": 10,
        },
        "version": PRIMAL_VERSION,
        "attestation": null,
    })
}

/// Build the semantic mappings object for `primal.announce`.
///
/// Maps `provenance.*` aliases to their canonical `dag.*` methods
/// so biomeOS can translate consumer names.
fn semantic_mapping_object() -> serde_json::Value {
    let map: serde_json::Map<String, serde_json::Value> = PROVENANCE_ALIASES
        .iter()
        .map(|(alias, canonical)| {
            ((*alias).to_owned(), serde_json::Value::String((*canonical).to_owned()))
        })
        .collect();
    serde_json::Value::Object(map)
}

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

/// Extra semantic aliases beyond the standard `short_name → fqn` mapping.
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
    ("provenance.session.create", "dag.session.create"),
    ("provenance.event.append", "dag.event.append"),
    ("provenance.event.append_batch", "dag.event.append_batch"),
    ("provenance.dehydration.trigger", "dag.dehydration.trigger"),
    ("provenance.branch", "dag.branch"),
    ("provenance.diff", "dag.diff"),
    ("provenance.merge", "dag.merge"),
    ("provenance.federate", "dag.federate"),
];

/// Normalize a JSON-RPC method name, handling legacy and alias prefixes.
///
/// Strips `rhizocrypt.` / `rhizo_crypt.` legacy prefixes. Also maps
/// `provenance.*` wire names to `dag.*` — downstream springs
/// (primalSpring domain contract sweep, healthSpring Nest atomic) use
/// `provenance.session.create` / `provenance.event.append` as aliases
/// for `dag.session.create` / `dag.event.append`. Both are valid wire
/// names; `dag.*` is canonical.
#[must_use]
pub fn normalize_method(method: &str) -> &str {
    if let Some(stripped) =
        method.strip_prefix("rhizocrypt.").or_else(|| method.strip_prefix("rhizo_crypt."))
    {
        return stripped;
    }

    PROVENANCE_ALIASES
        .iter()
        .find(|(alias, _)| *alias == method)
        .map_or(method, |(_, canonical)| canonical)
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
