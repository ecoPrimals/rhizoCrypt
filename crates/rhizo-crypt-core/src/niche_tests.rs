// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2024–2026 ecoPrimals Project

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
        assert!(CAPABILITIES.contains(&key.as_str()), "dependency key {key} not in CAPABILITIES");
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
fn all_capabilities_have_dependency_entries() {
    let deps = operation_dependencies();
    let obj = deps.as_object().expect("deps should be an object");
    for cap in CAPABILITIES {
        assert!(
            obj.contains_key(*cap),
            "CAPABILITIES entry '{cap}' has no key in operation_dependencies()"
        );
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
        assert!(!gpu, "{method} marked as GPU beneficial — rhizoCrypt is CPU-only infrastructure");
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
        assert!(domain_fqns.contains(cap), "capability {cap} not covered by any CapabilityDomain");
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

#[test]
fn health_liveness_returns_status_alive() {
    let result = health_liveness();
    assert_eq!(result["status"], "alive");
}

#[test]
fn health_readiness_running() {
    let result = health_readiness(true);
    assert_eq!(result["status"], "ready");
    assert_eq!(result["primal"], PRIMAL_ID);
    assert!(!result["version"].as_str().expect("version").is_empty());
}

#[test]
fn health_readiness_not_running() {
    let result = health_readiness(false);
    assert_eq!(result["status"], "not_ready");
}

#[test]
fn capabilities_include_health_probes() {
    assert!(CAPABILITIES.contains(&"health.liveness"));
    assert!(CAPABILITIES.contains(&"health.readiness"));
}

#[test]
fn normalize_method_strips_rhizocrypt_prefix() {
    assert_eq!(normalize_method("rhizocrypt.dag.session.create"), "dag.session.create");
    assert_eq!(normalize_method("rhizo_crypt.health.check"), "health.check");
    assert_eq!(normalize_method("dag.session.create"), "dag.session.create");
    assert_eq!(normalize_method("health.check"), "health.check");
}

#[test]
fn mcp_tools_has_expected_structure() {
    let tools = mcp_tools();
    let arr = tools.as_array().expect("tools should be an array");
    assert!(!arr.is_empty(), "MCP tools should not be empty");
    for tool in arr {
        assert!(tool.get("name").is_some(), "tool missing 'name'");
        assert!(tool.get("description").is_some(), "tool missing 'description'");
        assert!(tool.get("inputSchema").is_some(), "tool missing 'inputSchema'");
    }
}

#[test]
fn mcp_tool_names_are_valid_capabilities() {
    let tools = mcp_tools();
    let arr = tools.as_array().expect("tools array");
    for tool in arr {
        let name = tool["name"].as_str().expect("tool name");
        assert!(CAPABILITIES.contains(&name), "MCP tool {name} not in CAPABILITIES");
    }
}
