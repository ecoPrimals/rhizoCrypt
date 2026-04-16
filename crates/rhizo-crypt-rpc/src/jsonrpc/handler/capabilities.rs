// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2024–2026 ecoPrimals Project

//! JSON-RPC dispatch for capabilities and identity.

use super::HandlerError;
use super::params::to_json;
use crate::service::{RhizoCryptRpc, RhizoCryptRpcServer};
use serde_json::{Value, json};

pub fn dispatch_identity_get() -> Value {
    rhizo_crypt_core::niche::identity_get()
}

pub async fn dispatch_capability_list(server: &RhizoCryptRpcServer) -> Result<Value, HandlerError> {
    use rhizo_crypt_core::niche;

    let descriptors = server.clone().list_capabilities(tarpc::context::current()).await?;

    let provided_capabilities: Vec<Value> = descriptors
        .iter()
        .map(|d| {
            json!({
                "type": d.domain,
                "methods": d.methods.iter().map(|m| &m.name).collect::<Vec<_>>(),
            })
        })
        .collect();

    let cost_estimates: serde_json::Map<String, Value> = niche::METHOD_CATALOG
        .iter()
        .map(|m| {
            (
                m.fqn.to_string(),
                json!({ "cpu": niche::cost_tier(m.estimated_ms), "latency_ms": m.estimated_ms }),
            )
        })
        .collect();

    Ok(json!({
        "primal": niche::PRIMAL_ID,
        "version": niche::PRIMAL_VERSION,
        "methods": *niche::CAPABILITIES,
        "provided_capabilities": provided_capabilities,
        "consumed_capabilities": niche::CONSUMED_CAPABILITIES,
        "cost_estimates": cost_estimates,
        "operation_dependencies": niche::operation_dependencies(),
        "protocol": niche::PROTOCOL,
        "transport": [niche::TRANSPORT, "uds", "tcp"],
        "descriptors": to_json(&descriptors)?,
    }))
}
