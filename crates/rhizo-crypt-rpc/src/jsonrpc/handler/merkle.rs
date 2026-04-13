// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2024–2026 ecoPrimals Project

//! JSON-RPC dispatch for Merkle tree operations.

use super::HandlerError;
use super::params::{
    get_deserialized, get_obj, get_str, parse_session_id, parse_vertex_id, to_json,
};
use crate::service::{RhizoCryptRpc, RhizoCryptRpcServer};
use rhizo_crypt_core::MerkleRoot;
use serde_json::{Value, json};
use std::borrow::Cow;

pub async fn dispatch_merkle_root(
    server: &RhizoCryptRpcServer,
    params: Value,
) -> Result<Value, HandlerError> {
    let obj = get_obj(&params)?;
    let session_id = parse_session_id(get_str(obj, "session_id")?)?;
    let root = server.clone().get_merkle_root(tarpc::context::current(), session_id).await?;
    Ok(json!(hex::encode(root.0)))
}

pub async fn dispatch_merkle_proof(
    server: &RhizoCryptRpcServer,
    params: Value,
) -> Result<Value, HandlerError> {
    let obj = get_obj(&params)?;
    let session_id = parse_session_id(get_str(obj, "session_id")?)?;
    let vertex_id = parse_vertex_id(get_str(obj, "vertex_id")?)?;
    let proof =
        server.clone().get_merkle_proof(tarpc::context::current(), session_id, vertex_id).await?;
    to_json(&proof)
}

pub async fn dispatch_merkle_verify(
    server: &RhizoCryptRpcServer,
    params: Value,
) -> Result<Value, HandlerError> {
    let obj = get_obj(&params)?;
    let root_hex = get_str(obj, "root")?;
    let root_bytes = hex::decode(root_hex)
        .map_err(|e| HandlerError::InvalidParams(format!("root: {e}").into()))?;
    if root_bytes.len() != 32 {
        return Err(HandlerError::InvalidParams(Cow::Borrowed("root must be 32 bytes hex")));
    }
    let mut root_arr = [0u8; 32];
    root_arr.copy_from_slice(&root_bytes);
    let root = MerkleRoot(root_arr);
    let proof = get_deserialized(obj, "proof")?;
    let ok = server.clone().verify_proof(tarpc::context::current(), root, proof).await?;
    Ok(json!(ok))
}
