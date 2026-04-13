// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2024–2026 ecoPrimals Project

//! JSON-RPC dispatch for health and metrics.

use super::HandlerError;
use super::params::to_json;
use crate::service::{RhizoCryptRpc, RhizoCryptRpcServer};
use serde_json::Value;

pub async fn dispatch_health(server: &RhizoCryptRpcServer) -> Result<Value, HandlerError> {
    let status = server.clone().health(tarpc::context::current()).await?;
    to_json(&status)
}

pub async fn dispatch_readiness(server: &RhizoCryptRpcServer) -> Result<Value, HandlerError> {
    let status = server.clone().health(tarpc::context::current()).await?;
    Ok(rhizo_crypt_core::niche::health_readiness(status.healthy))
}

pub async fn dispatch_metrics(server: &RhizoCryptRpcServer) -> Result<Value, HandlerError> {
    let metrics = server.clone().metrics(tarpc::context::current()).await?;
    to_json(&metrics)
}
