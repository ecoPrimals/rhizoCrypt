// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2024–2026 ecoPrimals Project

//! JSON-RPC 2.0 method handler.
//!
//! Dispatches semantic method names to `RhizoCryptRpcServer` operations.

mod capabilities;
mod dehydration;
mod events;
mod health;
mod merkle;
mod params;
mod session;
mod slice;
mod tools;
mod vertex;

use crate::error::RpcError;
use crate::jsonrpc::types::JsonRpcRequest;
use crate::service::RhizoCryptRpcServer;
use serde_json::Value;
use std::borrow::Cow;
use std::sync::Arc;
use thiserror::Error;
use tracing::debug;

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
}

/// Handle a single JSON-RPC request.
pub async fn handle_request(
    primal: Arc<rhizo_crypt_core::RhizoCrypt>,
    request: JsonRpcRequest,
) -> Result<Value, HandlerError> {
    let server = RhizoCryptRpcServer::new(primal);
    let normalized = rhizo_crypt_core::niche::normalize_method(&request.method);
    let params = request.params.unwrap_or(Value::Null);

    debug!(method = %normalized, "JSON-RPC request");

    match normalized {
        "dag.session.create" => session::dispatch_session_create(&server, params).await,
        "dag.session.get" => session::dispatch_session_get(&server, params).await,
        "dag.session.list" => session::dispatch_session_list(&server).await,
        "dag.session.discard" => session::dispatch_session_discard(&server, params).await,
        "dag.event.append" => events::dispatch_event_append(&server, params).await,
        "dag.event.append_batch" => events::dispatch_event_append_batch(&server, params).await,
        "dag.vertex.get" => vertex::dispatch_vertex_get(&server, params).await,
        "dag.frontier.get" => vertex::dispatch_frontier_get(&server, params).await,
        "dag.genesis.get" => vertex::dispatch_genesis_get(&server, params).await,
        "dag.vertex.query" => vertex::dispatch_vertex_query(&server, params).await,
        "dag.vertex.children" => vertex::dispatch_vertex_children(&server, params).await,
        "dag.merkle.root" => merkle::dispatch_merkle_root(&server, params).await,
        "dag.merkle.proof" => merkle::dispatch_merkle_proof(&server, params).await,
        "dag.merkle.verify" => merkle::dispatch_merkle_verify(&server, params).await,
        "dag.slice.checkout" => slice::dispatch_slice_checkout(&server, params).await,
        "dag.slice.get" => slice::dispatch_slice_get(&server, params).await,
        "dag.slice.list" => slice::dispatch_slice_list(&server).await,
        "dag.slice.resolve" => slice::dispatch_slice_resolve(&server, params).await,
        "dag.dehydration.trigger" | "dag.dehydrate" => {
            dehydration::dispatch_dehydrate(&server, params).await
        }
        "dag.dehydration.status" => dehydration::dispatch_dehydrate_status(&server, params).await,
        "health.check" | "status" | "check" => health::dispatch_health(&server).await,
        "health.liveness" | "ping" | "health" => Ok(rhizo_crypt_core::niche::health_liveness()),
        "health.readiness" => health::dispatch_readiness(&server).await,
        "health.metrics" => health::dispatch_metrics(&server).await,
        "identity.get" => Ok(capabilities::dispatch_identity_get()),
        "capabilities.list" | "capability.list" | "primal.capabilities" => {
            capabilities::dispatch_capability_list(&server).await
        }
        "tools.list" | "mcp.tools.list" => Ok(tools::dispatch_tools_list()),
        "tools.call" | "mcp.tools.call" => tools::dispatch_tools_call(&server, params).await,
        _ => Err(HandlerError::MethodNotFound(request.method.into())),
    }
}

#[cfg(test)]
#[path = "../handler_tests.rs"]
mod tests;

#[cfg(test)]
#[path = "../handler_tests_validation.rs"]
mod tests_validation;

#[cfg(test)]
#[path = "../handler_proptests.rs"]
mod handler_proptests;
