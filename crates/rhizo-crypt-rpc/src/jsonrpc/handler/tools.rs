// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2024–2026 ecoPrimals Project

//! JSON-RPC dispatch for MCP-style tool listing and invocation.

use super::HandlerError;
use super::capabilities;
use super::dehydration;
use super::events;
use super::health;
use super::merkle;
use super::params::{get_obj, get_str};
use super::session;
use crate::service::RhizoCryptRpcServer;
use serde_json::Value;

pub fn dispatch_tools_list() -> Value {
    rhizo_crypt_core::niche::mcp_tools()
}

/// MCP `tools.call` dispatcher — routes tool invocations to JSON-RPC methods.
///
/// Absorbed from ecosystem MCP pattern. Enables
/// AI agent coordination by translating `tools.call { name, arguments }`
/// into the corresponding `dag.*` / `health.*` method dispatch.
pub async fn dispatch_tools_call(
    server: &RhizoCryptRpcServer,
    params: Value,
) -> Result<Value, HandlerError> {
    let obj = get_obj(&params)?;
    let tool_name = get_str(obj, "name")?;
    let arguments =
        obj.get("arguments").cloned().unwrap_or_else(|| Value::Object(serde_json::Map::new()));

    let normalized = rhizo_crypt_core::niche::normalize_method(tool_name);

    match normalized {
        "dag.session.create" => session::dispatch_session_create(server, arguments).await,
        "dag.event.append" => events::dispatch_event_append(server, arguments).await,
        "dag.merkle.root" => merkle::dispatch_merkle_root(server, arguments).await,
        "dag.dehydration.trigger" | "dag.dehydrate" => {
            dehydration::dispatch_dehydrate(server, arguments).await
        }
        "health.check" | "status" => health::dispatch_health(server).await,
        "capabilities.list" => capabilities::dispatch_capability_list(server).await,
        _ => Err(HandlerError::MethodNotFound(format!("tool not found: {tool_name}").into())),
    }
}
