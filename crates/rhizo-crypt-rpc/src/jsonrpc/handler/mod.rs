// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2024–2026 ecoPrimals Project

//! JSON-RPC 2.0 method handler.
//!
//! Dispatches semantic method names to `RhizoCryptRpcServer` operations.

mod branch;
mod capabilities;
mod dehydration;
mod events;
mod health;
mod merkle;
mod mesh;
mod params;
mod session;
mod slice;
mod tools;
mod vertex;

use crate::error::RpcError;
use crate::jsonrpc::method_gate::{CallerContext, MethodAccessLevel, MethodGate, classify_method};
use crate::jsonrpc::types::JsonRpcRequest;
use crate::service::RhizoCryptRpcServer;
use rhizo_crypt_core::PrimalLifecycle;
use serde_json::Value;
use std::borrow::Cow;
use thiserror::Error;
use tracing::{debug, warn};

/// Handler error for JSON-RPC.
#[derive(Debug, Error)]
pub enum HandlerError {
    /// Invalid parameters.
    #[error("invalid params: {0}")]
    InvalidParams(Cow<'static, str>),

    /// Method not found.
    #[error("method not found: {0}")]
    MethodNotFound(Cow<'static, str>),

    /// RPC error from service.
    #[error("rpc error: {0}")]
    Rpc(#[from] RpcError),

    /// Primal subsystem not ready to serve requests.
    #[error("not ready")]
    NotReady,

    /// Caller lacks permission to invoke a protected method.
    #[error("permission denied: method '{0}' requires a capability token")]
    PermissionDenied(String),
}

/// Handle a single JSON-RPC request.
///
/// Two pre-dispatch gates run before the method dispatch table:
///
/// 1. **Readiness gate** (S61 / PG-60): public methods are always allowed;
///    protected methods return `-32002 NOT_READY` if the primal isn't running.
///
/// 2. **Method gate** (S62 / JH-0): public methods are always allowed;
///    protected methods without a capability token are logged (permissive)
///    or rejected with `-32001 PERMISSION_DENIED` (enforced).
pub async fn handle_request(
    server: &RhizoCryptRpcServer,
    request: JsonRpcRequest,
    gate: &MethodGate,
    caller: &CallerContext,
) -> Result<Value, HandlerError> {
    let normalized = rhizo_crypt_core::niche::normalize_method(&request.method);
    let params = request.params.unwrap_or(Value::Null);

    debug!(method = %normalized, "JSON-RPC request");

    let is_public = classify_method(normalized) == MethodAccessLevel::Public;

    // Gate 1: readiness — public methods bypass
    if !is_public && !server.primal().state().is_running() {
        warn!(method = %normalized, state = %server.primal().state(), "rejected: primal not ready");
        return Err(HandlerError::NotReady);
    }

    // Gate 2: method authorization — public methods bypass
    if let Err(rejection) = gate.check(normalized, caller) {
        return Err(HandlerError::PermissionDenied(rejection.method));
    }

    match normalized {
        // Auth introspection (always public, handled by the gate)
        "auth.check" => Ok(gate.auth_check_response(caller)),
        "auth.mode" => Ok(gate.auth_mode_response()),
        "auth.peer_info" => Ok(gate.auth_peer_info_response(caller)),
        "dag.session.create" => session::dispatch_session_create(server, params).await,
        "dag.session.get" => session::dispatch_session_get(server, params).await,
        "dag.session.list" => session::dispatch_session_list(server).await,
        "dag.session.discard" => session::dispatch_session_discard(server, params).await,
        "dag.event.append" => events::dispatch_event_append(server, params).await,
        "dag.event.append_batch" => events::dispatch_event_append_batch(server, params).await,
        "dag.vertex.get" => vertex::dispatch_vertex_get(server, params).await,
        "dag.frontier.get" => vertex::dispatch_frontier_get(server, params).await,
        "dag.genesis.get" => vertex::dispatch_genesis_get(server, params).await,
        "dag.vertex.query" => vertex::dispatch_vertex_query(server, params).await,
        "dag.vertex.children" => vertex::dispatch_vertex_children(server, params).await,
        "dag.merkle.root" => merkle::dispatch_merkle_root(server, params).await,
        "dag.merkle.proof" => merkle::dispatch_merkle_proof(server, params).await,
        "dag.merkle.verify" => merkle::dispatch_merkle_verify(server, params).await,
        "dag.slice.checkout" => slice::dispatch_slice_checkout(server, params).await,
        "dag.slice.get" => slice::dispatch_slice_get(server, params).await,
        "dag.slice.list" => slice::dispatch_slice_list(server).await,
        "dag.slice.resolve" => slice::dispatch_slice_resolve(server, params).await,
        "dag.branch" => branch::dispatch_branch(server, params).await,
        "dag.diff" => branch::dispatch_diff(server, params).await,
        "dag.merge" => branch::dispatch_merge(server, params).await,
        "dag.federate" => branch::dispatch_federate(server, params).await,
        "dag.partial_dehydrate" => dehydration::dispatch_partial_dehydrate(server, params).await,
        "dag.dehydration.trigger" | "dag.dehydrate" => {
            dehydration::dispatch_dehydrate(server, params).await
        }
        "dag.dehydration.status" => dehydration::dispatch_dehydrate_status(server, params).await,
        "health.check" | "status" | "check" => health::dispatch_health(server).await,
        "health.liveness" | "ping" | "health" => Ok(health::dispatch_liveness(server)),
        "health.readiness" => health::dispatch_readiness(server).await,
        "health.metrics" => health::dispatch_metrics(server).await,
        "identity.get" => Ok(capabilities::dispatch_identity_get()),
        "capabilities.list" | "capability.list" | "primal.capabilities" => {
            capabilities::dispatch_capability_list(server).await
        }
        "mesh.events.record" => mesh::dispatch_mesh_events_record(server, params).await,
        "tools.list" | "mcp.tools.list" => Ok(tools::dispatch_tools_list()),
        "tools.call" | "mcp.tools.call" => tools::dispatch_tools_call(server, params).await,
        _ => Err(HandlerError::MethodNotFound(request.method.into())),
    }
}

#[cfg(test)]
#[path = "../handler_test_support.rs"]
mod test_support;

#[cfg(test)]
#[path = "../handler_tests.rs"]
mod tests;

#[cfg(test)]
#[path = "../handler_tests_validation.rs"]
mod tests_validation;

#[cfg(test)]
#[path = "../handler_tests_branching.rs"]
mod tests_branching;

#[cfg(test)]
#[path = "../handler_tests_gates.rs"]
mod tests_gates;

#[cfg(test)]
#[path = "../handler_tests_composition.rs"]
mod tests_composition;

#[cfg(test)]
#[path = "../handler_tests_provenance.rs"]
mod tests_provenance;

#[cfg(test)]
#[path = "../handler_tests_dehydrate.rs"]
mod tests_dehydrate;

#[cfg(test)]
#[path = "../handler_proptests.rs"]
mod handler_proptests;
