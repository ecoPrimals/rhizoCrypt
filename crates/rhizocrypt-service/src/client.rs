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

#[cfg(test)]
#[expect(clippy::expect_used, reason = "test assertions use expect for descriptive failures")]
mod tests {
    use super::*;
    use crate::ClientOperation;

    #[tokio::test]
    async fn invalid_address_returns_addr_parse_for_all_operations() {
        for operation in
            [ClientOperation::Health, ClientOperation::ListSessions, ClientOperation::Metrics]
        {
            let result = run_client("not-a-valid-address", operation.clone()).await;
            assert!(
                matches!(result, Err(ServiceError::AddrParse(_))),
                "expected AddrParse for {operation:?}"
            );
        }
    }

    #[tokio::test]
    async fn connection_failure_returns_config_error_for_all_operations() {
        for operation in
            [ClientOperation::Health, ClientOperation::ListSessions, ClientOperation::Metrics]
        {
            let result = run_client("127.0.0.1:1", operation.clone()).await;
            let err = result.expect_err("connection to closed port should fail");
            assert!(
                matches!(err, ServiceError::Config(_)),
                "expected Config for {operation:?}, got {err}"
            );
            assert!(
                err.to_string().contains("Failed to connect"),
                "expected connect message for {operation:?}, got {err}"
            );
        }
    }
}
