// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2024–2026 ecoPrimals Project

//! Tests for derived accessors: `normalize_method`, `health_liveness`,
//! `health_readiness`, `mcp_tools`, `announce_payload`, `identity_get`,
//! `capability_list`, `cost_tier`, and provenance alias coverage.

use super::*;

// ── normalize_method ─────────────────────────────────────────────

#[test]
fn normalize_strips_rhizocrypt_prefix() {
    assert_eq!(normalize_method("rhizocrypt.dag.session.create"), "dag.session.create");
}

#[test]
fn normalize_strips_rhizo_crypt_prefix() {
    assert_eq!(normalize_method("rhizo_crypt.dag.event.append"), "dag.event.append");
}

#[test]
fn normalize_maps_provenance_alias_to_dag() {
    assert_eq!(normalize_method("provenance.session.create"), "dag.session.create");
    assert_eq!(normalize_method("provenance.event.append"), "dag.event.append");
    assert_eq!(normalize_method("provenance.merkle.root"), "dag.merkle.root");
}

#[test]
fn normalize_passes_through_unknown_method() {
    assert_eq!(normalize_method("health.check"), "health.check");
    assert_eq!(normalize_method("unknown.method"), "unknown.method");
}

#[test]
fn normalize_all_provenance_aliases_resolve() {
    for (alias, canonical) in niche_derived::PROVENANCE_ALIASES {
        assert_eq!(
            normalize_method(alias),
            *canonical,
            "alias {alias} should resolve to {canonical}"
        );
    }
}

// ── cost_tier ────────────────────────────────────────────────────

#[test]
fn cost_tier_low() {
    assert_eq!(cost_tier(0), "low");
    assert_eq!(cost_tier(crate::constants::COST_TIER_LOW_THRESHOLD_MS), "low");
}

#[test]
fn cost_tier_medium() {
    assert_eq!(cost_tier(crate::constants::COST_TIER_LOW_THRESHOLD_MS + 1), "medium");
    assert_eq!(cost_tier(crate::constants::COST_TIER_MEDIUM_THRESHOLD_MS), "medium");
}

#[test]
fn cost_tier_high() {
    assert_eq!(cost_tier(crate::constants::COST_TIER_MEDIUM_THRESHOLD_MS + 1), "high");
    assert_eq!(cost_tier(u32::MAX), "high");
}

// ── health_liveness ──────────────────────────────────────────────

#[test]
fn health_liveness_with_uptime() {
    let val = health_liveness(Some(42));
    assert_eq!(val["status"], "alive");
    assert_eq!(val["primal"], "rhizocrypt");
    assert_eq!(val["uptime_s"], 42);
    assert!(!val["version"].as_str().unwrap().is_empty());
}

#[test]
fn health_liveness_without_uptime() {
    let val = health_liveness(None);
    assert_eq!(val["uptime_s"], 0);
    assert_eq!(val["status"], "alive");
}

// ── health_readiness ─────────────────────────────────────────────

#[test]
fn health_readiness_when_running() {
    let val = health_readiness(true);
    assert_eq!(val["status"], "ready");
    assert_eq!(val["primal"], "rhizocrypt");
}

#[test]
fn health_readiness_when_not_running() {
    let val = health_readiness(false);
    assert_eq!(val["status"], "not_ready");
}

// ── identity_get ─────────────────────────────────────────────────

#[test]
fn identity_get_contains_required_fields() {
    let val = identity_get();
    assert_eq!(val["primal"], "rhizocrypt");
    assert_eq!(val["domain"], "dag");
    assert_eq!(val["license"], "AGPL-3.0-or-later");
    assert!(val["version"].is_string());
    assert!(val["transport"].is_string());
    assert!(val["protocol"].is_string());
}

// ── announce_payload ─────────────────────────────────────────────

#[test]
fn announce_payload_structure() {
    let val = announce_payload("/run/biomeos/rhizocrypt.sock", Some(1234));
    assert_eq!(val["primal"], "rhizocrypt");
    assert_eq!(val["socket"], "/run/biomeos/rhizocrypt.sock");
    assert_eq!(val["pid"], 1234);
    assert!(val["capabilities"].is_array());
    assert!(val["methods"].is_array());
    assert!(val["semantic_mappings"].is_object());
    assert!(val["cost_hints"].is_object());
    assert!(val["latency_estimates"].is_object());
}

#[test]
fn announce_payload_with_no_pid() {
    let val = announce_payload("/tmp/test.sock", None);
    assert!(val["pid"].is_null());
}

#[test]
fn announce_payload_semantic_mappings_cover_provenance() {
    let val = announce_payload("/tmp/test.sock", None);
    let mappings = val["semantic_mappings"].as_object().expect("should be object");
    assert!(mappings.contains_key("provenance.session.create"));
    assert_eq!(mappings["provenance.session.create"], "dag.session.create");
}

// ── mcp_tools ────────────────────────────────────────────────────

#[test]
fn mcp_tools_returns_array() {
    let tools = mcp_tools();
    assert!(tools.is_array());
    let arr = tools.as_array().unwrap();
    assert!(arr.len() >= 4, "Expected at least 4 MCP tools, got {}", arr.len());
}

#[test]
fn mcp_tools_each_has_name_and_schema() {
    let tools = mcp_tools();
    for tool in tools.as_array().unwrap() {
        assert!(tool["name"].is_string(), "tool missing 'name'");
        assert!(tool["description"].is_string(), "tool missing 'description'");
        assert!(tool["inputSchema"].is_object(), "tool missing 'inputSchema'");
    }
}

#[test]
fn mcp_tools_includes_session_create() {
    let tools = mcp_tools();
    let names: Vec<&str> =
        tools.as_array().unwrap().iter().map(|t| t["name"].as_str().unwrap()).collect();
    assert!(names.contains(&"dag.session.create"));
    assert!(names.contains(&"health.check"));
    assert!(names.contains(&"capabilities.list"));
}

// ── capability_list ──────────────────────────────────────────────

#[test]
fn capability_list_has_expected_structure() {
    let val = capability_list();
    assert_eq!(val["primal"], "rhizocrypt");
    assert!(val["methods"].is_array());
    assert!(val["domains"].is_array());
    assert!(val["locality"]["local"].is_number());
    assert!(val["locality"]["external"].is_number());
    assert!(val["capabilities"].is_array());
}

#[test]
fn capability_list_methods_have_required_fields() {
    let val = capability_list();
    for method in val["methods"].as_array().unwrap() {
        assert!(method["method"].is_string());
        assert!(method["domain"].is_string());
        assert!(method["cost"].is_string());
    }
}

// ── method_locality_counts ───────────────────────────────────────

#[test]
fn method_locality_counts_sum_to_catalog_size() {
    let (local, external) = method_locality_counts();
    assert_eq!(local + external, METHOD_CATALOG.len());
}

// ── SEMANTIC_MAPPINGS ────────────────────────────────────────────

#[test]
fn semantic_mappings_include_catalog_entries() {
    let mappings = &*niche_derived::SEMANTIC_MAPPINGS;
    for spec in METHOD_CATALOG {
        assert!(
            mappings.iter().any(|(short, fqn)| *short == spec.short_name && *fqn == spec.fqn),
            "Missing semantic mapping for {} -> {}",
            spec.short_name,
            spec.fqn,
        );
    }
}

// ── CONSUMED_CAPABILITIES ────────────────────────────────────────

#[test]
fn consumed_capabilities_are_capability_qualified() {
    for cap in niche_derived::CONSUMED_CAPABILITIES {
        assert!(
            cap.contains('.'),
            "Consumed capability '{cap}' should be domain-qualified (contain a '.')"
        );
    }
}

// ── DEPENDENCIES ─────────────────────────────────────────────────

#[test]
fn dependencies_are_all_optional() {
    for (domain, required, _desc) in niche_derived::DEPENDENCIES {
        assert!(!required, "Dependency on '{domain}' should be optional (graceful fallback)");
    }
}
