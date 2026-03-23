// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2024–2026 ecoPrimals Project

//! Normalization of capability list responses from ecoPrimals primals.
//!
//! Different primals expose method lists in several JSON shapes. [`parse_capability_response`]
//! maps any supported shape into a sorted list of fully qualified method names.

use serde_json::{Map, Value};

/// Extracts capability method names from a JSON value in any supported ecosystem format.
///
/// Supported shapes:
///
/// 1. **String array** — `["dag.session.create", "dag.vertex.get"]`
/// 2. **Enhanced object array** — `[{"method": "dag.session.create", "cost": "low"}]`
/// 3. **Domain-grouped** — `{"dag": {"methods": ["session.create"]}, "health": {"methods": ["check"]}}`
/// 4. **Full niche-style object** — `{"capabilities": [...], "methods": [...], "domains": [...]}`
///    (each field is optional; values are merged)
///
/// Object entries accept a `method` field (preferred) or a `name` field (legacy dual-format).
/// Domain-grouped and niche `domains` objects join each domain key with relative method names
/// when the method is not already prefixed with that domain.
///
/// The result is sorted alphabetically and deduplicated for stable, deterministic output.
///
/// # Examples
///
/// ```
/// use rhizo_crypt_core::clients::capabilities::parse_capability_response;
/// use serde_json::json;
///
/// let v = json!(["dag.vertex.get", "dag.session.create"]);
/// assert_eq!(
///     parse_capability_response(&v),
///     vec!["dag.session.create", "dag.vertex.get"]
/// );
/// ```
///
/// ```
/// use rhizo_crypt_core::clients::capabilities::parse_capability_response;
/// use serde_json::json;
///
/// let v = json!({
///     "capabilities": ["health.check"],
///     "methods": [{ "method": "dag.session.create", "cost": "low" }],
///     "domains": { "dag": { "methods": ["vertex.get"] } }
/// });
/// let names = parse_capability_response(&v);
/// assert!(names.contains(&"dag.session.create".to_string()));
/// assert!(names.contains(&"dag.vertex.get".to_string()));
/// assert!(names.contains(&"health.check".to_string()));
/// assert_eq!(names.len(), 3);
/// ```
#[must_use]
pub fn parse_capability_response(value: &Value) -> Vec<String> {
    let mut out = Vec::new();
    match value {
        Value::Array(arr) => parse_array_items(arr, None, &mut out),
        Value::Object(map) => {
            if is_niche_style_object(map) {
                parse_niche_object(map, &mut out);
            } else {
                parse_domain_grouped(map, &mut out);
            }
        }
        _ => {}
    }
    out.sort_unstable();
    out.dedup();
    out
}

fn is_niche_style_object(map: &Map<String, Value>) -> bool {
    map.contains_key("capabilities") || map.contains_key("methods") || map.contains_key("domains")
}

fn parse_niche_object(map: &Map<String, Value>, out: &mut Vec<String>) {
    if let Some(v) = map.get("capabilities") {
        parse_leaf_value(v, out);
    }
    if let Some(v) = map.get("methods") {
        parse_leaf_value(v, out);
    }
    if let Some(v) = map.get("domains") {
        parse_domains_value(v, out);
    }
}

fn parse_domains_value(value: &Value, out: &mut Vec<String>) {
    match value {
        Value::Object(map) => parse_domain_grouped(map, out),
        Value::Array(arr) => {
            for item in arr {
                let Value::Object(obj) = item else {
                    continue;
                };
                let prefix = obj
                    .get("prefix")
                    .and_then(Value::as_str)
                    .or_else(|| obj.get("domain").and_then(Value::as_str))
                    .or_else(|| obj.get("name").and_then(Value::as_str));
                let Some(domain) = prefix else {
                    continue;
                };
                let Some(Value::Array(methods)) = obj.get("methods") else {
                    continue;
                };
                parse_array_items(methods, Some(domain), out);
            }
        }
        _ => {}
    }
}

fn parse_leaf_value(value: &Value, out: &mut Vec<String>) {
    match value {
        Value::Array(arr) => parse_array_items(arr, None, out),
        Value::String(s) => {
            if !s.is_empty() {
                out.push(s.clone());
            }
        }
        Value::Object(map) => {
            if let Some(m) = method_name_from_object(map) {
                out.push(m);
            }
        }
        _ => {}
    }
}

fn parse_domain_grouped(map: &Map<String, Value>, out: &mut Vec<String>) {
    for (domain, val) in map {
        let Value::Object(inner) = val else {
            continue;
        };
        let Some(Value::Array(methods)) = inner.get("methods") else {
            continue;
        };
        parse_array_items(methods, Some(domain.as_str()), out);
    }
}

fn parse_array_items(arr: &[Value], domain: Option<&str>, out: &mut Vec<String>) {
    for item in arr {
        push_from_array_element(item, domain, out);
    }
}

fn push_from_array_element(item: &Value, domain: Option<&str>, out: &mut Vec<String>) {
    match item {
        Value::String(s) => {
            let joined = join_domain_method(domain, s);
            if !joined.is_empty() {
                out.push(joined);
            }
        }
        Value::Object(map) => {
            if let Some(m) = method_name_from_object(map) {
                let joined = join_domain_method(domain, &m);
                if !joined.is_empty() {
                    out.push(joined);
                }
            }
        }
        _ => {}
    }
}

fn method_name_from_object(map: &Map<String, Value>) -> Option<String> {
    map.get("method")
        .and_then(Value::as_str)
        .or_else(|| map.get("name").and_then(Value::as_str))
        .map(str::to_owned)
}

fn join_domain_method(domain: Option<&str>, method: &str) -> String {
    let Some(domain) = domain else {
        return method.to_string();
    };
    if method.is_empty() {
        return String::new();
    }
    let prefix = format!("{domain}.");
    if method == domain || method.starts_with(&prefix) {
        method.to_string()
    } else {
        format!("{domain}.{method}")
    }
}

#[cfg(test)]
mod tests {
    use super::parse_capability_response;
    use serde_json::json;

    #[test]
    fn format_simple_string_array() {
        let v = json!(["dag.session.create", "dag.vertex.get"]);
        assert_eq!(parse_capability_response(&v), vec!["dag.session.create", "dag.vertex.get"]);
    }

    #[test]
    fn format_simple_string_array_sorted() {
        let v = json!(["zebra.method", "aardvark.method"]);
        assert_eq!(parse_capability_response(&v), vec!["aardvark.method", "zebra.method"]);
    }

    #[test]
    fn format_enhanced_object_array() {
        let v = json!([
            { "method": "dag.session.create", "cost": "low" },
            { "method": "dag.vertex.get", "cost": "medium" }
        ]);
        assert_eq!(parse_capability_response(&v), vec!["dag.session.create", "dag.vertex.get"]);
    }

    #[test]
    fn format_enhanced_prefers_method_over_name() {
        let v = json!([{ "method": "dag.a", "name": "ignored" }]);
        assert_eq!(parse_capability_response(&v), vec!["dag.a"]);
    }

    #[test]
    fn format_domain_grouped() {
        let v = json!({
            "dag": { "methods": ["session.create", "vertex.get"] },
            "health": { "methods": ["check", "health.liveness"] }
        });
        assert_eq!(
            parse_capability_response(&v),
            vec!["dag.session.create", "dag.vertex.get", "health.check", "health.liveness"]
        );
    }

    #[test]
    fn format_domain_grouped_keeps_fully_qualified_methods() {
        let v = json!({
            "dag": { "methods": ["dag.session.create", "vertex.get"] }
        });
        assert_eq!(parse_capability_response(&v), vec!["dag.session.create", "dag.vertex.get"]);
    }

    #[test]
    fn format_full_niche_merges_capabilities_methods_domains() {
        let v = json!({
            "capabilities": ["capabilities.list", "health.check"],
            "methods": [{ "method": "dag.session.create", "cost": "low" }],
            "domains": {
                "dag": { "methods": ["vertex.get"] },
                "tools": { "methods": ["call"] }
            }
        });
        let got = parse_capability_response(&v);
        assert_eq!(
            got,
            vec![
                "capabilities.list",
                "dag.session.create",
                "dag.vertex.get",
                "health.check",
                "tools.call"
            ]
        );
    }

    #[test]
    fn format_full_niche_domains_array_with_prefix() {
        let v = json!({
            "domains": [
                { "prefix": "dag", "description": "DAG", "methods": ["session.list"] }
            ]
        });
        assert_eq!(parse_capability_response(&v), vec!["dag.session.list"]);
    }

    #[test]
    fn deduplicates_after_merge() {
        let v = json!({
            "capabilities": ["dag.session.create"],
            "methods": [{ "method": "dag.session.create" }]
        });
        assert_eq!(parse_capability_response(&v), vec!["dag.session.create"]);
    }

    #[test]
    fn empty_array() {
        assert!(parse_capability_response(&json!([])).is_empty());
    }

    #[test]
    fn unknown_scalar_yields_empty() {
        assert!(parse_capability_response(&json!(null)).is_empty());
        assert!(parse_capability_response(&json!(42)).is_empty());
    }
}
