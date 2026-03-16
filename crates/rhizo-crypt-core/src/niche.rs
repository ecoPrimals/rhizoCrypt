// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2024–2026 ecoPrimals Project

//! Niche self-knowledge for the rhizoCrypt primal.
//!
//! Follows the ecoPrimals niche pattern established by squirrel, groundSpring,
//! wetSpring, and airSpring. Every primal defines its self-knowledge in a single
//! module so that biomeOS, Songbird, and the Pathway Learner can reason about it
//! without hardcoded primal names or port numbers.
//!
//! This module holds:
//! - Identity (who am I?)
//! - Capabilities (what do I expose via biomeOS?)
//! - Consumed capabilities (what do I need from other primals?)
//! - Cost estimates (scheduling hints for biomeOS Pathway Learner)
//! - Operation dependencies (parallelization DAG for Pathway Learner)
//! - Semantic mappings (short operation name → full method)
//!
//! Other modules reference these constants rather than duplicating string
//! literals. rhizoCrypt only knows itself — it discovers other primals at
//! runtime via capability-based discovery through Songbird.

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

/// All capabilities this primal exposes to biomeOS.
///
/// Each string is a fully qualified capability name (`{domain}.{method}`)
/// that biomeOS can route via `capability.call`.
pub const CAPABILITIES: &[&str] = &[
    // Session lifecycle
    "dag.session.create",
    "dag.session.get",
    "dag.session.list",
    "dag.session.discard",
    // Event operations
    "dag.event.append",
    "dag.event.append_batch",
    // Vertex queries
    "dag.vertex.get",
    "dag.vertex.query",
    "dag.vertex.children",
    // DAG topology
    "dag.frontier.get",
    "dag.genesis.get",
    // Merkle integrity
    "dag.merkle.root",
    "dag.merkle.proof",
    "dag.merkle.verify",
    // Slice operations
    "dag.slice.checkout",
    "dag.slice.get",
    "dag.slice.list",
    "dag.slice.resolve",
    // Dehydration
    "dag.dehydration.trigger",
    "dag.dehydration.status",
    // Health and introspection
    "health.check",
    "health.metrics",
    "capability.list",
];

/// Semantic mappings: short operation name → fully qualified capability.
///
/// biomeOS uses these during domain registration so
/// `capability.call { domain: "dag", operation: "session.create" }` routes
/// to the correct JSON-RPC method.
pub const SEMANTIC_MAPPINGS: &[(&str, &str)] = &[
    ("session.create", "dag.session.create"),
    ("session.get", "dag.session.get"),
    ("session.list", "dag.session.list"),
    ("session.discard", "dag.session.discard"),
    ("event.append", "dag.event.append"),
    ("event.append_batch", "dag.event.append_batch"),
    ("vertex.get", "dag.vertex.get"),
    ("vertex.query", "dag.vertex.query"),
    ("vertex.children", "dag.vertex.children"),
    ("frontier.get", "dag.frontier.get"),
    ("genesis.get", "dag.genesis.get"),
    ("merkle.root", "dag.merkle.root"),
    ("merkle.proof", "dag.merkle.proof"),
    ("merkle.verify", "dag.merkle.verify"),
    ("slice.checkout", "dag.slice.checkout"),
    ("slice.get", "dag.slice.get"),
    ("slice.list", "dag.slice.list"),
    ("slice.resolve", "dag.slice.resolve"),
    ("dehydration.trigger", "dag.dehydration.trigger"),
    ("dehydration.status", "dag.dehydration.status"),
    ("health", "health.check"),
    ("metrics", "health.metrics"),
    ("capabilities", "capability.list"),
];

/// Consumed capabilities — what rhizoCrypt calls on other primals.
///
/// rhizoCrypt discovers these at runtime via Songbird; it never hardcodes
/// which primal provides them. The Pathway Learner uses this list to
/// ensure required capabilities are available before routing to rhizoCrypt.
pub const CONSUMED_CAPABILITIES: &[&str] = &[
    // Signing (BearDog or any signing provider)
    "crypto.sign",
    "crypto.verify",
    // Permanent storage (LoamSpine or any commit provider)
    "commit.session",
    "commit.entry",
    // Payload storage (NestGate or any content-addressed store)
    "storage.store",
    "storage.get",
    // Attribution (sweetGrass or any provenance provider)
    "provenance.create_braid",
    "provenance.lineage",
    // Discovery (Songbird)
    "discovery.register",
    "discovery.query",
];

/// Primal dependencies for deployment.
///
/// Each entry: `(primal_capability_domain, required, description)`.
/// `required = true` means rhizoCrypt cannot function without it.
/// `required = false` means graceful degradation is supported.
///
/// Note: these reference capability domains, not primal names.
pub const DEPENDENCIES: &[(&str, bool, &str)] = &[
    ("crypto", false, "vertex signing and verification (graceful fallback to unsigned)"),
    ("discovery", false, "service mesh registration (graceful fallback to standalone)"),
    ("commit", false, "dehydration to permanent storage (graceful fallback to local-only)"),
    ("storage", false, "content-addressed payload storage (graceful fallback to inline)"),
    ("provenance", false, "attribution braids (graceful fallback to unattributed)"),
];

/// Cost estimates for biomeOS Pathway Learner scheduling.
///
/// Each entry: `(capability, estimated_ms, gpu_beneficial)`.
/// Times are representative for typical workloads. The Pathway Learner
/// uses these to make intelligent routing decisions.
pub const COST_ESTIMATES: &[(&str, u32, bool)] = &[
    // Session lifecycle — fast, in-memory
    ("dag.session.create", 1, false),
    ("dag.session.get", 1, false),
    ("dag.session.list", 1, false),
    ("dag.session.discard", 2, false),
    // Event operations — BLAKE3 hashing + DashMap insert
    ("dag.event.append", 2, false),
    ("dag.event.append_batch", 5, false),
    // Vertex queries — DashMap lookup
    ("dag.vertex.get", 1, false),
    ("dag.vertex.query", 3, false),
    ("dag.vertex.children", 1, false),
    // DAG topology — DashMap scan
    ("dag.frontier.get", 2, false),
    ("dag.genesis.get", 2, false),
    // Merkle operations — BLAKE3 tree computation
    ("dag.merkle.root", 5, false),
    ("dag.merkle.proof", 3, false),
    ("dag.merkle.verify", 2, false),
    // Slice operations — may involve permanent storage I/O
    ("dag.slice.checkout", 10, false),
    ("dag.slice.get", 1, false),
    ("dag.slice.list", 1, false),
    ("dag.slice.resolve", 5, false),
    // Dehydration — multi-step I/O to permanent storage
    ("dag.dehydration.trigger", 50, false),
    ("dag.dehydration.status", 1, false),
    // Health and introspection
    ("health.check", 1, false),
    ("health.metrics", 1, false),
    ("capability.list", 1, false),
];

/// Capability domain descriptor for biomeOS introspection.
///
/// Absorbed from ludoSpring V20 `capability_domains.rs` pattern. The `external`
/// flag tells biomeOS which methods require IPC routing vs in-process dispatch.
#[derive(Clone, Debug)]
pub struct CapabilityMethod {
    /// Short method name (e.g., "session.create").
    pub name: &'static str,
    /// Fully qualified capability (e.g., "dag.session.create").
    pub fqn: &'static str,
    /// Whether this method requires external IPC routing (false = in-process).
    pub external: bool,
}

/// Structured capability domain for biomeOS routing decisions.
///
/// biomeOS uses the `external` flag to determine whether a `capability.call`
/// for this primal needs IPC routing (external) or can be dispatched in-process
/// (local). All rhizoCrypt methods are local since it processes requests directly.
#[derive(Clone, Debug)]
pub struct CapabilityDomain {
    /// Domain prefix (e.g., "dag").
    pub prefix: &'static str,
    /// Human-readable domain description.
    pub description: &'static str,
    /// Methods within this domain.
    pub methods: &'static [CapabilityMethod],
}

/// Capability domain definitions with external/local classification.
///
/// All rhizoCrypt methods are local (in-process) since the primal processes
/// all requests directly. The classification helps biomeOS plan dispatch.
pub const CAPABILITY_DOMAINS: &[CapabilityDomain] = &[
    CapabilityDomain {
        prefix: "dag",
        description: "Ephemeral DAG session and vertex operations",
        methods: &[
            CapabilityMethod {
                name: "session.create",
                fqn: "dag.session.create",
                external: false,
            },
            CapabilityMethod {
                name: "session.get",
                fqn: "dag.session.get",
                external: false,
            },
            CapabilityMethod {
                name: "session.list",
                fqn: "dag.session.list",
                external: false,
            },
            CapabilityMethod {
                name: "session.discard",
                fqn: "dag.session.discard",
                external: false,
            },
            CapabilityMethod {
                name: "event.append",
                fqn: "dag.event.append",
                external: false,
            },
            CapabilityMethod {
                name: "event.append_batch",
                fqn: "dag.event.append_batch",
                external: false,
            },
            CapabilityMethod {
                name: "vertex.get",
                fqn: "dag.vertex.get",
                external: false,
            },
            CapabilityMethod {
                name: "vertex.query",
                fqn: "dag.vertex.query",
                external: false,
            },
            CapabilityMethod {
                name: "vertex.children",
                fqn: "dag.vertex.children",
                external: false,
            },
            CapabilityMethod {
                name: "frontier.get",
                fqn: "dag.frontier.get",
                external: false,
            },
            CapabilityMethod {
                name: "genesis.get",
                fqn: "dag.genesis.get",
                external: false,
            },
            CapabilityMethod {
                name: "merkle.root",
                fqn: "dag.merkle.root",
                external: false,
            },
            CapabilityMethod {
                name: "merkle.proof",
                fqn: "dag.merkle.proof",
                external: false,
            },
            CapabilityMethod {
                name: "merkle.verify",
                fqn: "dag.merkle.verify",
                external: false,
            },
            CapabilityMethod {
                name: "slice.checkout",
                fqn: "dag.slice.checkout",
                external: false,
            },
            CapabilityMethod {
                name: "slice.get",
                fqn: "dag.slice.get",
                external: false,
            },
            CapabilityMethod {
                name: "slice.list",
                fqn: "dag.slice.list",
                external: false,
            },
            CapabilityMethod {
                name: "slice.resolve",
                fqn: "dag.slice.resolve",
                external: false,
            },
            CapabilityMethod {
                name: "dehydration.trigger",
                fqn: "dag.dehydration.trigger",
                external: false,
            },
            CapabilityMethod {
                name: "dehydration.status",
                fqn: "dag.dehydration.status",
                external: false,
            },
        ],
    },
    CapabilityDomain {
        prefix: "health",
        description: "Health and introspection",
        methods: &[
            CapabilityMethod {
                name: "check",
                fqn: "health.check",
                external: false,
            },
            CapabilityMethod {
                name: "metrics",
                fqn: "health.metrics",
                external: false,
            },
        ],
    },
    CapabilityDomain {
        prefix: "capability",
        description: "Capability introspection",
        methods: &[CapabilityMethod {
            name: "list",
            fqn: "capability.list",
            external: false,
        }],
    },
];

/// Returns all methods across all domains.
#[must_use]
pub fn all_methods() -> Vec<&'static CapabilityMethod> {
    CAPABILITY_DOMAINS.iter().flat_map(|domain| domain.methods.iter()).collect()
}

/// Returns the count of local (in-process) vs external (IPC-routed) methods.
#[must_use]
pub fn method_locality_counts() -> (usize, usize) {
    let methods = all_methods();
    let local = methods.iter().filter(|m| !m.external).count();
    let external = methods.iter().filter(|m| m.external).count();
    (local, external)
}

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

/// Operation dependency hints for biomeOS Pathway Learner parallelization.
///
/// Maps each operation to the operations it depends on, enabling the
/// Pathway Learner to build a DAG and parallelize independent operations.
#[must_use]
pub fn operation_dependencies() -> serde_json::Value {
    serde_json::json!({
        "dag.session.create": [],
        "dag.session.get": [],
        "dag.session.list": [],
        "dag.session.discard": ["dag.session.create"],
        "dag.event.append": ["dag.session.create"],
        "dag.event.append_batch": ["dag.session.create"],
        "dag.vertex.get": ["dag.event.append"],
        "dag.vertex.query": ["dag.event.append"],
        "dag.vertex.children": ["dag.event.append"],
        "dag.frontier.get": ["dag.event.append"],
        "dag.genesis.get": ["dag.event.append"],
        "dag.merkle.root": ["dag.event.append"],
        "dag.merkle.proof": ["dag.event.append"],
        "dag.merkle.verify": [],
        "dag.slice.checkout": [],
        "dag.slice.get": ["dag.slice.checkout"],
        "dag.slice.list": [],
        "dag.slice.resolve": ["dag.slice.checkout"],
        "dag.dehydration.trigger": ["dag.session.create", "dag.event.append"],
        "dag.dehydration.status": ["dag.dehydration.trigger"],
        "health.check": [],
        "health.metrics": [],
        "capability.list": [],
    })
}

/// Return the capability list as a JSON-RPC response payload.
///
/// Implements the `capability.list` semantic method. Aligns with loamSpine
/// and sweetGrass enhanced format: domain, method, dependencies, cost tier.
/// Includes ludoSpring V20 domain introspection with external/local flags.
#[must_use]
pub fn capability_list() -> serde_json::Value {
    let deps = operation_dependencies();
    let methods: Vec<serde_json::Value> = COST_ESTIMATES
        .iter()
        .map(|(method, ms, _gpu)| {
            let all = all_methods();
            let external = all.iter().find(|m| m.fqn == *method).is_some_and(|m| m.external);
            serde_json::json!({
                "method": method,
                "domain": method.split('.').next().unwrap_or("unknown"),
                "cost": cost_tier(*ms),
                "external": external,
                "deps": deps.get(method).cloned().unwrap_or(serde_json::json!([])),
            })
        })
        .collect();

    let domains: Vec<serde_json::Value> = CAPABILITY_DOMAINS
        .iter()
        .map(|d| {
            serde_json::json!({
                "prefix": d.prefix,
                "description": d.description,
                "method_count": d.methods.len(),
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
        "capabilities": CAPABILITIES,
        "consumed_capabilities": CONSUMED_CAPABILITIES,
        "domains": domains,
        "locality": { "local": local_count, "external": external_count },
        "methods": methods,
    })
}

#[cfg(test)]
#[expect(clippy::expect_used, reason = "test code")]
mod tests {
    use super::*;

    #[test]
    fn primal_identity_is_consistent() {
        assert_eq!(PRIMAL_ID, "rhizocrypt");
        assert_eq!(DOMAIN, "dag");
        assert_eq!(LICENSE, "AGPL-3.0-or-later");
        assert!(!PRIMAL_VERSION.is_empty());
    }

    #[test]
    fn capabilities_count_matches_cost_estimates() {
        assert_eq!(
            CAPABILITIES.len(),
            COST_ESTIMATES.len(),
            "every capability must have a cost estimate"
        );
    }

    #[test]
    fn all_capabilities_have_cost_estimates() {
        for cap in CAPABILITIES {
            assert!(
                COST_ESTIMATES.iter().any(|(method, _, _)| method == cap),
                "missing cost estimate for {cap}"
            );
        }
    }

    #[test]
    fn all_cost_estimates_are_valid_capabilities() {
        for (method, _, _) in COST_ESTIMATES {
            assert!(
                CAPABILITIES.contains(method),
                "{method} in COST_ESTIMATES but not in CAPABILITIES"
            );
        }
    }

    #[test]
    fn semantic_mappings_resolve_to_capabilities() {
        for (_, full_method) in SEMANTIC_MAPPINGS {
            assert!(
                CAPABILITIES.contains(full_method),
                "semantic mapping {full_method} not in CAPABILITIES"
            );
        }
    }

    #[test]
    fn operation_dependencies_reference_valid_capabilities() {
        let deps = operation_dependencies();
        let obj = deps.as_object().expect("deps should be an object");
        for (key, val) in obj {
            assert!(
                CAPABILITIES.contains(&key.as_str()),
                "dependency key {key} not in CAPABILITIES"
            );
            if let Some(arr) = val.as_array() {
                for dep in arr {
                    let dep_str = dep.as_str().expect("dep should be a string");
                    assert!(
                        CAPABILITIES.contains(&dep_str),
                        "dependency {dep_str} for {key} not in CAPABILITIES"
                    );
                }
            }
        }
    }

    #[test]
    fn capability_list_has_required_fields() {
        let list = capability_list();
        assert!(list.get("primal").is_some());
        assert!(list.get("version").is_some());
        assert!(list.get("capabilities").is_some());
        assert!(list.get("consumed_capabilities").is_some());
        assert!(list.get("methods").is_some());

        let methods = list["methods"].as_array().expect("methods array");
        assert_eq!(methods.len(), CAPABILITIES.len());

        for method in methods {
            assert!(method.get("method").is_some());
            assert!(method.get("domain").is_some());
            assert!(method.get("cost").is_some());
            assert!(method.get("deps").is_some());
        }
    }

    #[test]
    fn cost_tier_thresholds() {
        assert_eq!(cost_tier(1), "low");
        assert_eq!(cost_tier(2), "low");
        assert_eq!(cost_tier(3), "medium");
        assert_eq!(cost_tier(10), "medium");
        assert_eq!(cost_tier(11), "high");
        assert_eq!(cost_tier(50), "high");
    }

    #[test]
    fn no_gpu_beneficial_operations() {
        for (method, _, gpu) in COST_ESTIMATES {
            assert!(
                !gpu,
                "{method} marked as GPU beneficial — rhizoCrypt is CPU-only infrastructure"
            );
        }
    }

    #[test]
    fn consumed_capabilities_are_not_self_capabilities() {
        for consumed in CONSUMED_CAPABILITIES {
            assert!(
                !CAPABILITIES.contains(consumed),
                "{consumed} appears in both CAPABILITIES and CONSUMED_CAPABILITIES"
            );
        }
    }

    #[test]
    fn dependencies_reference_capability_domains_not_primal_names() {
        let primal_names = ["beardog", "songbird", "loamspine", "nestgate", "sweetgrass"];
        for (domain, _, _) in DEPENDENCIES {
            assert!(
                !primal_names.contains(domain),
                "dependency {domain} references a primal name, not a capability domain"
            );
        }
    }

    #[test]
    fn capability_domains_cover_all_capabilities() {
        let domain_fqns: Vec<&str> = all_methods().iter().map(|m| m.fqn).collect();
        for cap in CAPABILITIES {
            assert!(
                domain_fqns.contains(cap),
                "capability {cap} not covered by any CapabilityDomain"
            );
        }
    }

    #[test]
    fn all_domain_methods_are_valid_capabilities() {
        for method in all_methods() {
            assert!(
                CAPABILITIES.contains(&method.fqn),
                "domain method {} not in CAPABILITIES",
                method.fqn
            );
        }
    }

    #[test]
    fn all_rhizocrypt_methods_are_local() {
        let (local, external) = method_locality_counts();
        assert_eq!(external, 0, "rhizoCrypt is CPU-only infrastructure — all methods local");
        assert_eq!(local, CAPABILITIES.len());
    }

    #[test]
    fn capability_list_includes_domains_and_locality() {
        let list = capability_list();
        assert!(list.get("domains").is_some(), "missing 'domains' field");
        assert!(list.get("locality").is_some(), "missing 'locality' field");

        let locality = &list["locality"];
        assert_eq!(
            locality["local"].as_u64().expect("local count"),
            u64::try_from(CAPABILITIES.len()).expect("cap len fits u64")
        );
        assert_eq!(locality["external"].as_u64().expect("external count"), 0);

        let methods = list["methods"].as_array().expect("methods array");
        for method in methods {
            assert!(method.get("external").is_some(), "method missing 'external' flag");
        }
    }

    #[test]
    fn capability_domains_have_consistent_prefixes() {
        for domain in CAPABILITY_DOMAINS {
            for method in domain.methods {
                assert!(
                    method.fqn.starts_with(domain.prefix),
                    "method {} doesn't start with domain prefix {}",
                    method.fqn,
                    domain.prefix
                );
            }
        }
    }
}
