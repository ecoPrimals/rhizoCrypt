// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2024–2026 ecoPrimals Project

//! Client operations against a running rhizoCrypt server.

use std::net::SocketAddr;

use crate::{ClientOperation, ServiceError};

/// Run a client operation against a running rhizoCrypt server.
///
/// # Errors
///
/// Returns [`ServiceError::AddrParse`] if `address` is not a valid socket address.
/// Returns [`ServiceError::Config`] if the RPC call fails.
pub async fn run_client(address: &str, operation: ClientOperation) -> Result<(), ServiceError> {
    let addr: SocketAddr = address.parse()?;

    let client = rhizo_crypt_rpc::RpcClient::connect(addr)
        .await
        .map_err(|e| ServiceError::Config(format!("Failed to connect: {e}")))?;

    match operation {
        ClientOperation::Health => {
            let health = client
                .health()
                .await
                .map_err(|e| ServiceError::Config(format!("Health check failed: {e}")))?;
            println!(
                "{}",
                serde_json::to_string_pretty(&health).unwrap_or_else(|_| format!("{health:?}"))
            );
        }
        ClientOperation::ListSessions => {
            let sessions = client
                .list_sessions()
                .await
                .map_err(|e| ServiceError::Config(format!("List sessions failed: {e}")))?;
            println!(
                "{}",
                serde_json::to_string_pretty(&sessions).unwrap_or_else(|_| format!("{sessions:?}"))
            );
        }
        ClientOperation::Metrics => {
            let metrics = client
                .metrics()
                .await
                .map_err(|e| ServiceError::Config(format!("Metrics failed: {e}")))?;
            println!(
                "{}",
                serde_json::to_string_pretty(&metrics).unwrap_or_else(|_| format!("{metrics:?}"))
            );
        }
    }

    Ok(())
}
