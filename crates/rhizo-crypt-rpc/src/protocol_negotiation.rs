// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2024–2026 ecoPrimals Project

//! G65 Protocol Negotiation — single-socket protocol selection.
//!
//! Enables a primal to serve multiple RPC protocols on a single socket.
//! The client sends `PROTOCOLS: tarpc,jsonrpc\n`, the server selects the best
//! mutual match and responds `PROTOCOL: tarpc\n`. No negotiation = JSON-RPC
//! (full backward compatibility).
//!
//! G66: All negotiation functions are generic over `AsyncRead + AsyncWrite`,
//! operating on any [`TransportStream`](rhizo_crypt_core::TransportStream)
//! — UDS on Unix, TCP on Windows, any future transport.
//!
//! See `specs/PROTOCOL_NEGOTIATION_SPEC.md` in wateringHole.

use std::fmt;

/// Protocols that can be negotiated on a single socket (G65).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IpcProtocol {
    /// tarpc binary (bincode + length-delimited). Sub-ms, type-safe.
    Tarpc,
    /// JSON-RPC 2.0 (newline-delimited or HTTP). Default fallback.
    JsonRpc,
}

impl IpcProtocol {
    /// Wire name as sent in `PROTOCOLS:` / `PROTOCOL:` lines.
    #[must_use]
    pub const fn wire_name(self) -> &'static str {
        match self {
            Self::Tarpc => "tarpc",
            Self::JsonRpc => "jsonrpc",
        }
    }

    /// Parse from wire name (case-insensitive).
    #[must_use]
    pub fn from_wire_name(s: &str) -> Option<Self> {
        match s.trim().to_ascii_lowercase().as_str() {
            "tarpc" => Some(Self::Tarpc),
            "jsonrpc" => Some(Self::JsonRpc),
            _ => None,
        }
    }
}

impl fmt::Display for IpcProtocol {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.wire_name())
    }
}

/// Protocols this server supports (preference order: tarpc first).
pub const SERVER_SUPPORTED: &[IpcProtocol] = &[IpcProtocol::Tarpc, IpcProtocol::JsonRpc];

/// Select the best mutual protocol. Client preference order wins.
#[must_use]
pub fn select_protocol(
    client_supported: &[IpcProtocol],
    server_supported: &[IpcProtocol],
) -> IpcProtocol {
    for client_proto in client_supported {
        if server_supported.contains(client_proto) {
            return *client_proto;
        }
    }
    IpcProtocol::JsonRpc
}

/// Parse a `PROTOCOLS: tarpc,jsonrpc` line into a list of protocols.
///
/// Returns `None` if the line doesn't start with `PROTOCOLS:`.
#[must_use]
pub fn parse_protocols_line(line: &str) -> Option<Vec<IpcProtocol>> {
    let trimmed = line.trim();
    let rest = trimmed.strip_prefix("PROTOCOLS:")?;
    let protocols: Vec<IpcProtocol> =
        rest.split(',').filter_map(|s| IpcProtocol::from_wire_name(s.trim())).collect();
    Some(protocols)
}

/// Format the server's protocol selection response line.
#[must_use]
pub fn format_protocol_response(selected: IpcProtocol) -> String {
    format!("PROTOCOL: {}\n", selected.wire_name())
}

/// Result of attempting G65 protocol negotiation on an incoming connection.
pub enum NegotiationResult {
    /// tarpc was negotiated — serve binary framing on this stream.
    Tarpc,
    /// JSON-RPC was explicitly negotiated — proceed with JSON-RPC handler.
    JsonRpc,
    /// The first bytes were not a `PROTOCOLS:` line. The returned `Vec`
    /// contains all bytes consumed so far (must be chained back).
    NotNegotiation(Vec<u8>),
}

/// Attempt G65 protocol negotiation on any connected stream (G66 transport-agnostic).
///
/// Reads the first line from the stream (combining any `leftover` bytes from
/// prior mito-beacon detection). If the line starts with `PROTOCOLS:`,
/// negotiates and responds. Otherwise returns `NotNegotiation` with all
/// consumed bytes so the caller can chain them back.
///
/// # Errors
///
/// Returns `std::io::Error` on read/write failures.
pub async fn try_negotiate<S>(
    stream: &mut S,
    leftover: Vec<u8>,
) -> std::io::Result<NegotiationResult>
where
    S: tokio::io::AsyncRead + tokio::io::AsyncWrite + Unpin,
{
    use tokio::io::{AsyncReadExt, AsyncWriteExt};

    let mut line_buf = leftover;

    if !line_buf.contains(&b'\n') {
        let mut byte = [0u8; 1];
        loop {
            let n = stream.read(&mut byte).await?;
            if n == 0 {
                break;
            }
            line_buf.push(byte[0]);
            if byte[0] == b'\n' {
                break;
            }
            if line_buf.len() > 256 {
                return Ok(NegotiationResult::NotNegotiation(line_buf));
            }
        }
    }

    let Ok(line_str) = std::str::from_utf8(&line_buf) else {
        return Ok(NegotiationResult::NotNegotiation(line_buf));
    };

    let Some(client_protocols) = parse_protocols_line(line_str) else {
        return Ok(NegotiationResult::NotNegotiation(line_buf));
    };

    let selected = select_protocol(&client_protocols, SERVER_SUPPORTED);
    let response = format_protocol_response(selected);
    stream.write_all(response.as_bytes()).await?;
    stream.flush().await?;

    tracing::info!(
        selected = selected.wire_name(),
        client = ?client_protocols.iter().map(|p| p.wire_name()).collect::<Vec<_>>(),
        "G65 protocol negotiated"
    );

    match selected {
        IpcProtocol::Tarpc => Ok(NegotiationResult::Tarpc),
        IpcProtocol::JsonRpc => Ok(NegotiationResult::JsonRpc),
    }
}

/// Client-side protocol negotiation on any connected stream (G66 transport-agnostic).
///
/// Sends `PROTOCOLS:` line, reads server response, returns selected protocol.
///
/// # Errors
///
/// Returns `std::io::Error` on I/O or protocol errors.
pub async fn negotiate_client<S>(
    stream: &mut S,
    preferred: &[IpcProtocol],
) -> std::io::Result<IpcProtocol>
where
    S: tokio::io::AsyncRead + tokio::io::AsyncWrite + Unpin,
{
    use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};

    let protocols_str: Vec<&str> = preferred.iter().map(|p| p.wire_name()).collect();
    let line = format!("PROTOCOLS: {}\n", protocols_str.join(","));
    stream.write_all(line.as_bytes()).await?;
    stream.flush().await?;

    let mut reader = BufReader::new(&mut *stream);
    let mut response = String::new();
    reader.read_line(&mut response).await?;

    let trimmed = response.trim();
    let selected_name = trimmed
        .strip_prefix("PROTOCOL:")
        .ok_or_else(|| std::io::Error::other(format!("invalid negotiation response: {trimmed}")))?
        .trim();

    IpcProtocol::from_wire_name(selected_name)
        .ok_or_else(|| std::io::Error::other(format!("unknown protocol: {selected_name}")))
}

#[cfg(test)]
#[expect(clippy::unwrap_used, reason = "test code")]
mod tests {
    use super::*;

    #[test]
    fn test_wire_name_roundtrip() {
        assert_eq!(IpcProtocol::from_wire_name("tarpc"), Some(IpcProtocol::Tarpc));
        assert_eq!(IpcProtocol::from_wire_name("jsonrpc"), Some(IpcProtocol::JsonRpc));
        assert_eq!(IpcProtocol::from_wire_name("TARPC"), Some(IpcProtocol::Tarpc));
        assert_eq!(IpcProtocol::from_wire_name("unknown"), None);
    }

    #[test]
    fn test_parse_protocols_line() {
        let protos = parse_protocols_line("PROTOCOLS: tarpc,jsonrpc\n").unwrap();
        assert_eq!(protos, vec![IpcProtocol::Tarpc, IpcProtocol::JsonRpc]);

        let protos = parse_protocols_line("PROTOCOLS: jsonrpc\n").unwrap();
        assert_eq!(protos, vec![IpcProtocol::JsonRpc]);

        assert!(parse_protocols_line("{\"jsonrpc\":\"2.0\"}").is_none());
        assert!(parse_protocols_line("").is_none());
    }

    #[test]
    fn test_select_protocol_client_preference() {
        let client = [IpcProtocol::Tarpc, IpcProtocol::JsonRpc];
        let server = [IpcProtocol::Tarpc, IpcProtocol::JsonRpc];
        assert_eq!(select_protocol(&client, &server), IpcProtocol::Tarpc);

        let client = [IpcProtocol::JsonRpc, IpcProtocol::Tarpc];
        assert_eq!(select_protocol(&client, &server), IpcProtocol::JsonRpc);
    }

    #[test]
    fn test_select_protocol_fallback() {
        let client = [IpcProtocol::Tarpc];
        let server = [IpcProtocol::JsonRpc];
        assert_eq!(select_protocol(&client, &server), IpcProtocol::JsonRpc);
    }

    #[test]
    fn test_format_protocol_response() {
        assert_eq!(format_protocol_response(IpcProtocol::Tarpc), "PROTOCOL: tarpc\n");
        assert_eq!(format_protocol_response(IpcProtocol::JsonRpc), "PROTOCOL: jsonrpc\n");
    }

    // Stream-based tests use UnixStream::pair() — Unix-only
    #[cfg(unix)]
    mod stream_tests {
        use super::*;

        #[tokio::test]
        async fn test_negotiate_roundtrip_tarpc() {
            let (mut client, mut server) = tokio::net::UnixStream::pair().unwrap();

            let server_handle =
                tokio::spawn(async move { try_negotiate(&mut server, Vec::new()).await.unwrap() });

            let selected =
                negotiate_client(&mut client, &[IpcProtocol::Tarpc, IpcProtocol::JsonRpc])
                    .await
                    .unwrap();
            assert_eq!(selected, IpcProtocol::Tarpc);

            let result = server_handle.await.unwrap();
            assert!(matches!(result, NegotiationResult::Tarpc));
        }

        #[tokio::test]
        async fn test_negotiate_roundtrip_jsonrpc_only() {
            let (mut client, mut server) = tokio::net::UnixStream::pair().unwrap();

            let server_handle =
                tokio::spawn(async move { try_negotiate(&mut server, Vec::new()).await.unwrap() });

            let selected = negotiate_client(&mut client, &[IpcProtocol::JsonRpc]).await.unwrap();
            assert_eq!(selected, IpcProtocol::JsonRpc);

            let result = server_handle.await.unwrap();
            assert!(matches!(result, NegotiationResult::JsonRpc));
        }

        #[tokio::test]
        async fn test_not_negotiation_jsonrpc_passthrough() {
            let (mut client, mut server) = tokio::net::UnixStream::pair().unwrap();

            use tokio::io::AsyncWriteExt;
            let msg = b"{\"jsonrpc\":\"2.0\",\"method\":\"health.check\"}\n";
            client.write_all(msg).await.unwrap();
            drop(client);

            let leftover = b"{\"".to_vec();
            let result = try_negotiate(&mut server, leftover).await.unwrap();
            match result {
                NegotiationResult::NotNegotiation(bytes) => {
                    assert!(bytes.starts_with(b"{\""));
                }
                _ => panic!("expected NotNegotiation"),
            }
        }

        #[tokio::test]
        async fn test_negotiate_with_leftover_from_mito_beacon() {
            let (mut client, mut server) = tokio::net::UnixStream::pair().unwrap();

            use tokio::io::AsyncWriteExt;
            let line = b"OTOCOLS: tarpc,jsonrpc\n";
            client.write_all(line).await.unwrap();

            let leftover = b"PR".to_vec();
            let server_handle =
                tokio::spawn(async move { try_negotiate(&mut server, leftover).await.unwrap() });

            use tokio::io::{AsyncBufReadExt, BufReader};
            let mut reader = BufReader::new(&mut client);
            let mut response = String::new();
            reader.read_line(&mut response).await.unwrap();
            assert_eq!(response.trim(), "PROTOCOL: tarpc");

            let result = server_handle.await.unwrap();
            assert!(matches!(result, NegotiationResult::Tarpc));
        }
    }

    /// TCP-based negotiation test — proves G66 transport-agnostic (works on all platforms).
    #[tokio::test]
    async fn test_negotiate_over_tcp_transport() {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();

        let server_handle = tokio::spawn(async move {
            let (mut stream, _) = listener.accept().await.unwrap();
            try_negotiate(&mut stream, Vec::new()).await.unwrap()
        });

        let mut client = tokio::net::TcpStream::connect(addr).await.unwrap();
        let selected = negotiate_client(&mut client, &[IpcProtocol::Tarpc, IpcProtocol::JsonRpc])
            .await
            .unwrap();
        assert_eq!(selected, IpcProtocol::Tarpc);

        let result = server_handle.await.unwrap();
        assert!(matches!(result, NegotiationResult::Tarpc));
    }
}
