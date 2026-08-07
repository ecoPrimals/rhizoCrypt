// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2024–2026 ecoPrimals Project

//! Transport connection and JSON-RPC request helpers.

use super::{TransportEndpoint, TransportStream};

/// Connect to a service via its resolved [`TransportEndpoint`].
///
/// # Errors
///
/// Returns `io::Error` on connection failure. `MeshRelay` endpoints require
/// routing through Songbird and are not directly connectable.
pub async fn connect_transport(endpoint: &TransportEndpoint) -> std::io::Result<TransportStream> {
    match endpoint {
        #[cfg(unix)]
        TransportEndpoint::Uds {
            path,
        } => {
            let stream = tokio::net::UnixStream::connect(path).await?;
            Ok(TransportStream::Unix(stream))
        }
        #[cfg(not(unix))]
        TransportEndpoint::Uds {
            path,
        } => Err(std::io::Error::new(
            std::io::ErrorKind::Unsupported,
            format!("UDS not available on this platform for {path}"),
        )),
        TransportEndpoint::Tcp {
            host,
            port,
        } => {
            let addr = format!("{host}:{port}");
            let stream = tokio::net::TcpStream::connect(&addr).await?;
            Ok(TransportStream::Tcp(stream))
        }
        TransportEndpoint::MeshRelay {
            peer_id,
            capability,
        } => Err(std::io::Error::new(
            std::io::ErrorKind::Unsupported,
            format!("mesh relay ({peer_id}/{capability}) requires discovery routing"),
        )),
    }
}

/// Structured error for JSON-RPC transport operations.
#[derive(Debug)]
pub enum JsonRpcTransportError {
    /// Timed out connecting to the endpoint.
    ConnectTimeout,
    /// Failed to establish a connection.
    ConnectFailed(std::io::Error),
    /// Failed to serialize the request body.
    Serialize(serde_json::Error),
    /// Failed to write the request to the transport.
    Write(std::io::Error),
    /// Timed out waiting for the response.
    ResponseTimeout,
    /// Failed to read the response from the transport.
    Read(std::io::Error),
}

impl std::fmt::Display for JsonRpcTransportError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ConnectTimeout => write!(f, "connection timed out"),
            Self::ConnectFailed(e) => write!(f, "connection failed: {e}"),
            Self::Serialize(e) => write!(f, "serialize failed: {e}"),
            Self::Write(e) => write!(f, "write failed: {e}"),
            Self::ResponseTimeout => write!(f, "response timed out"),
            Self::Read(e) => write!(f, "read failed: {e}"),
        }
    }
}

impl std::error::Error for JsonRpcTransportError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::ConnectFailed(e) | Self::Write(e) | Self::Read(e) => Some(e),
            Self::Serialize(e) => Some(e),
            Self::ConnectTimeout | Self::ResponseTimeout => None,
        }
    }
}

/// Send a single JSON-RPC request over a transport endpoint and return the
/// response line.
///
/// This is the shared implementation used by the provenance client, mesh
/// listener, and any future ecosystem client that needs fire-and-forget
/// JSON-RPC over `TransportEndpoint`.
///
/// # Errors
///
/// Returns [`JsonRpcTransportError`] on connection timeout/failure, write
/// failure, serialization failure, or response timeout/read failure.
pub async fn send_jsonrpc_request(
    endpoint: &TransportEndpoint,
    request: &serde_json::Value,
    connect_timeout: std::time::Duration,
    response_timeout: std::time::Duration,
) -> std::result::Result<String, JsonRpcTransportError> {
    use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};

    #[cfg_attr(not(unix), allow(unused_mut))]
    let mut stream = tokio::time::timeout(connect_timeout, connect_transport(endpoint))
        .await
        .map_err(|_| JsonRpcTransportError::ConnectTimeout)?
        .map_err(JsonRpcTransportError::ConnectFailed)?;

    match &stream {
        TransportStream::Tcp(tcp) => {
            let _ = tcp.set_nodelay(true);
        }
        #[cfg(unix)]
        TransportStream::Unix(_) => {}
    }

    #[cfg(unix)]
    if crate::btsp_client::btsp_strict_mode_expected()
        && let TransportStream::Unix(s) = &mut stream
    {
        crate::btsp_client::perform_client_handshake(s).await.map_err(|e| {
            JsonRpcTransportError::ConnectFailed(std::io::Error::other(format!(
                "BTSP handshake failed: {e}",
            )))
        })?;
    }

    let (reader, mut writer) = tokio::io::split(stream);

    let payload =
        format!("{}\n", serde_json::to_string(request).map_err(JsonRpcTransportError::Serialize)?);
    writer.write_all(payload.as_bytes()).await.map_err(JsonRpcTransportError::Write)?;
    writer.flush().await.map_err(JsonRpcTransportError::Write)?;

    let mut buf_reader = BufReader::new(reader);
    let mut response = String::new();

    tokio::time::timeout(response_timeout, buf_reader.read_line(&mut response))
        .await
        .map_err(|_| JsonRpcTransportError::ResponseTimeout)?
        .map_err(JsonRpcTransportError::Read)?;

    Ok(response)
}
