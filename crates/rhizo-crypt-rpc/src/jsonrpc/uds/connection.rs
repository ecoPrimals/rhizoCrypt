// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2024–2026 ecoPrimals Project

//! Per-connection UDS handler: mito-beacon stripping, BTSP negotiation, JSON-RPC.

use rhizo_crypt_core::constants::{MITO_BEACON_EXTENDED, MITO_BEACON_SIGNAL};
use tracing::{debug, warn};

/// Handle a single UDS connection with optional BTSP handshake enforcement.
///
/// When the first two bytes are a mito-beacon signal (`0xEC`/`0xED` + sub-type),
/// they are silently consumed before protocol detection proceeds.
///
/// When `btsp_required` is `true`, uses first-byte auto-detect with three
/// branches:
///
/// 1. `{` → read the full first line, then:
///    - `"protocol":"btsp"` → **JSON-line BTSP handshake** (primalSpring
///      interop), then serve full JSON-RPC on the authenticated stream
///    - otherwise → **full JSON-RPC** — UDS is filesystem-authenticated and
///      family-scoped (BTSP Phase 1), so all methods are available without a
///      Phase 2 handshake
/// 2. `[` → batch JSON-RPC (no BTSP required on UDS)
/// 3. Any other byte → **length-prefixed BTSP handshake** (internal), then
///    serve full JSON-RPC
///
/// When `btsp_required` is `false` (development mode), the connection serves
/// raw newline-delimited JSON-RPC immediately with all methods.
#[allow(clippy::redundant_pub_crate, reason = "re-exported for UDS integration tests")]
pub(crate) async fn handle_uds_connection(
    mut stream: tokio::net::UnixStream,
    server: crate::service::RhizoCryptRpcServer,
    btsp_required: bool,
    family_seed: Option<&[u8]>,
) -> std::io::Result<()> {
    use crate::btsp::BtspServer;
    use tokio::io::{AsyncReadExt, AsyncWriteExt};

    let gate = super::super::method_gate::MethodGate::for_primal(server.primal());
    let caller = super::super::method_gate::CallerContext::unix();

    let leftover = consume_mito_beacon_prefix(&mut stream).await?;

    if btsp_required {
        let Some(seed) = family_seed else {
            warn!("BTSP required but no FAMILY_SEED — rejecting connection");
            return Err(std::io::Error::other("BTSP: no family seed"));
        };

        let first_byte = if let Some(&b) = leftover.first() {
            b
        } else {
            let mut first = [0u8; 1];
            let n = stream.read(&mut first).await?;
            if n == 0 {
                return Ok(());
            }
            first[0]
        };
        let extra = if leftover.len() > 1 {
            &leftover[1..]
        } else {
            &[]
        };

        if first_byte == b'{' {
            detect_btsp_or_jsonrpc(stream, extra, seed, &server, &gate, &caller).await
        } else if first_byte == b'[' {
            debug!("batch JSON-RPC on UDS (filesystem-authenticated, all methods)");
            let (reader, writer) = stream.into_split();
            let chained_reader = leftover.as_slice().chain(reader);
            let joined = tokio::io::join(chained_reader, writer);
            super::super::newline::handle_newline_connection(joined, &server, &gate, &caller).await
        } else {
            let (reader, writer) = stream.into_split();
            let chained_reader = leftover.as_slice().chain(reader);
            let mut rw = tokio::io::join(chained_reader, writer);

            match BtspServer::accept_handshake(&mut rw, seed).await {
                Ok(session) => {
                    debug!(cipher = session.cipher.as_str(), "BTSP handshake complete");
                    serve_after_handshake(rw, &server, session).await
                }
                Err(e) => {
                    warn!(error = %e, "BTSP handshake failed, dropping connection");
                    let (_, mut writer) = rw.into_inner();
                    if let Err(e2) = BtspServer::send_handshake_error(&mut writer).await {
                        debug!(error = %e2, "failed to send BTSP handshake error to client");
                    }
                    let _ = writer.shutdown().await;
                    Err(std::io::Error::other(format!("BTSP handshake failed: {e}")))
                }
            }
        }
    } else if leftover.is_empty() {
        super::super::newline::handle_newline_connection(stream, &server, &gate, &caller).await
    } else {
        let (reader, writer) = stream.into_split();
        let chained = leftover.as_slice().chain(reader);
        let joined = tokio::io::join(chained, writer);
        super::super::newline::handle_newline_connection(joined, &server, &gate, &caller).await
    }
}

/// Distinguish a JSON-line BTSP handshake from plain JSON-RPC.
///
/// The caller has already consumed the leading `{` and any extra bytes from
/// the mito-beacon prefix probe. This function reads the rest of the first
/// line to check for `"protocol":"btsp"`.
async fn detect_btsp_or_jsonrpc(
    mut stream: tokio::net::UnixStream,
    extra: &[u8],
    seed: &[u8],
    server: &crate::service::RhizoCryptRpcServer,
    gate: &super::super::method_gate::MethodGate,
    caller: &super::super::method_gate::CallerContext,
) -> std::io::Result<()> {
    use crate::btsp::BtspServer;
    use tokio::io::{AsyncReadExt, AsyncWriteExt};

    let mut first_line = vec![b'{'];
    first_line.extend_from_slice(extra);
    if !first_line.contains(&b'\n') {
        let mut byte = [0u8; 1];
        loop {
            let n = stream.read(&mut byte).await?;
            if n == 0 {
                break;
            }
            first_line.push(byte[0]);
            if byte[0] == b'\n' {
                break;
            }
        }
    }

    let json_end = first_line.iter().rposition(|&b| b != b'\n' && b != b'\r').map_or(0, |i| i + 1);
    let json_bytes = &first_line[..json_end];

    let is_btsp = serde_json::from_slice::<serde_json::Value>(json_bytes)
        .ok()
        .and_then(|v| v.get("protocol")?.as_str().map(|s| s == "btsp"))
        .unwrap_or(false);

    let (reader, writer) = stream.into_split();

    if is_btsp {
        debug!("detected BTSP JSON-line handshake (protocol:btsp)");
        let mut rw = tokio::io::join(reader, writer);
        match BtspServer::accept_handshake_jsonline(&mut rw, seed, json_bytes).await {
            Ok(session) => {
                debug!(cipher = session.cipher.as_str(), "BTSP JSON-line handshake complete");
                serve_after_handshake(rw, server, session).await
            }
            Err(e) => {
                warn!(error = %e, "BTSP JSON-line handshake failed");
                let (_, mut writer) = rw.into_inner();
                let reason = e.to_string();
                if let Err(e2) =
                    BtspServer::send_handshake_error_jsonline(&mut writer, &reason).await
                {
                    debug!(error = %e2, "failed to send BTSP JSON-line error");
                }
                let _ = writer.shutdown().await;
                Err(std::io::Error::other(format!("BTSP JSON-line handshake failed: {e}")))
            }
        }
    } else {
        debug!("plain JSON-RPC on UDS (filesystem-authenticated, all methods)");
        let chained_reader = first_line.as_slice().chain(reader);
        let joined = tokio::io::join(chained_reader, writer);
        super::super::newline::handle_newline_connection(joined, server, gate, caller).await
    }
}

/// Read the first two bytes from a UDS connection and strip a mito-beacon
/// signal prefix if present.
///
/// Accepts any 2-byte prefix where the first byte is a mito-beacon signal
/// (`0xEC` or `0xED`). The second byte is a sub-type indicator (discarded).
///
/// Returns any consumed bytes that were **not** a mito-beacon signal so the
/// caller can chain them back onto the stream. An empty `Vec` means either
/// the signal was stripped or the connection was empty.
async fn consume_mito_beacon_prefix(
    stream: &mut tokio::net::UnixStream,
) -> std::io::Result<Vec<u8>> {
    use tokio::io::AsyncReadExt;

    let mut probe = [0u8; 2];
    let mut total = 0;
    while total < 2 {
        let n = stream.read(&mut probe[total..]).await?;
        if n == 0 {
            return Ok(probe[..total].to_vec());
        }
        total += n;
    }

    if probe[0] == MITO_BEACON_SIGNAL || probe[0] == MITO_BEACON_EXTENDED {
        debug!(
            signal = format_args!("0x{:02X}", probe[0]),
            sub_type = format_args!("0x{:02X}", probe[1]),
            "mito-beacon signal accepted, proceeding with protocol detection"
        );
        Ok(Vec::new())
    } else {
        Ok(probe.to_vec())
    }
}

/// After a successful BTSP handshake, attempt Phase 3 negotiation.
///
/// Reads the first JSON-RPC line from the client. If it's a `btsp.negotiate`
/// request, handles cipher negotiation and (on success) serves subsequent
/// traffic through encrypted framing. If the first request is anything else,
/// chains it back and falls through to the standard newline handler.
async fn serve_after_handshake<S>(
    mut stream: S,
    server: &crate::service::RhizoCryptRpcServer,
    session: crate::btsp::BtspSession,
) -> std::io::Result<()>
where
    S: tokio::io::AsyncRead + tokio::io::AsyncWrite + Unpin,
{
    use crate::btsp::framing;

    let session_id_hex = hex::encode(session.session_id);

    let first_line = match framing::read_json_line(&mut stream).await {
        Ok(line) => line,
        Err(e) if e.kind() == std::io::ErrorKind::UnexpectedEof => return Ok(()),
        Err(e) => return Err(e),
    };

    let first_json: serde_json::Value = match serde_json::from_slice(&first_line) {
        Ok(v) => v,
        Err(_) => {
            return chain_and_serve(first_line.to_vec(), stream, server).await;
        }
    };

    let is_negotiate =
        first_json.get("method").and_then(|m| m.as_str()).is_some_and(|m| m == "btsp.negotiate");

    if is_negotiate {
        match crate::btsp::phase3::handle_negotiate(
            &mut stream,
            &session.handshake_key,
            &session_id_hex,
            &first_json,
        )
        .await
        {
            Ok(Some(keys)) => {
                debug!("BTSP Phase 3 active, serving encrypted JSON-RPC");
                handle_encrypted_connection(stream, server, keys).await
            }
            Ok(None) => {
                debug!("BTSP Phase 3 declined (null cipher), serving plaintext JSON-RPC");
                let gate = super::super::method_gate::MethodGate::for_primal(server.primal());
                let caller = super::super::method_gate::CallerContext::unix();
                super::super::newline::handle_newline_connection(stream, server, &gate, &caller)
                    .await
            }
            Err(e) => {
                warn!(error = %e, "BTSP Phase 3 negotiate failed");
                let gate = super::super::method_gate::MethodGate::for_primal(server.primal());
                let caller = super::super::method_gate::CallerContext::unix();
                super::super::newline::handle_newline_connection(stream, server, &gate, &caller)
                    .await
            }
        }
    } else {
        debug!("no Phase 3 negotiate, serving plaintext JSON-RPC");
        chain_and_serve(first_line.to_vec(), stream, server).await
    }
}

/// Prepend a consumed first line back onto the stream and serve plaintext.
async fn chain_and_serve<S>(
    first_line: Vec<u8>,
    stream: S,
    server: &crate::service::RhizoCryptRpcServer,
) -> std::io::Result<()>
where
    S: tokio::io::AsyncRead + tokio::io::AsyncWrite + Unpin,
{
    let gate = super::super::method_gate::MethodGate::for_primal(server.primal());
    let caller = super::super::method_gate::CallerContext::unix();
    let mut prepend = first_line;
    prepend.push(b'\n');
    let (reader, writer) = tokio::io::split(stream);
    let cursor = std::io::Cursor::new(prepend);
    let chained = tokio::io::AsyncReadExt::chain(cursor, reader);
    let joined = tokio::io::join(chained, writer);
    super::super::newline::handle_newline_connection(joined, server, &gate, &caller).await
}

/// Serve JSON-RPC over a BTSP Phase 3 encrypted channel.
///
/// Each message is framed as `[4B length BE u32][encrypted payload]` where
/// the encrypted payload is `[12B nonce][ciphertext + Poly1305 tag]`.
async fn handle_encrypted_connection<S>(
    mut stream: S,
    server: &crate::service::RhizoCryptRpcServer,
    keys: crate::btsp::Phase3Keys,
) -> std::io::Result<()>
where
    S: tokio::io::AsyncRead + tokio::io::AsyncWrite + Unpin,
{
    use crate::btsp::framing;
    use tokio::io::AsyncWriteExt;

    let gate = super::super::method_gate::MethodGate::for_primal(server.primal());
    let caller = super::super::method_gate::CallerContext::unix();

    loop {
        let frame = match framing::read_frame(&mut stream).await {
            Ok(f) => f,
            Err(e) if e.kind() == std::io::ErrorKind::UnexpectedEof => return Ok(()),
            Err(e) => return Err(e),
        };

        let plaintext = keys
            .decrypt(&frame)
            .map_err(|e| std::io::Error::other(format!("Phase 3 decrypt: {e}")))?;

        let request_str = String::from_utf8(plaintext)
            .map_err(|e| std::io::Error::other(format!("Phase 3: not UTF-8: {e}")))?;

        let value: serde_json::Value = match serde_json::from_str(&request_str) {
            Ok(v) => v,
            Err(e) => {
                warn!(error = %e, "Phase 3 encrypted JSON-RPC parse error");
                let resp = super::super::serialize_response(&super::super::types::error_response(
                    None,
                    super::super::types::codes::PARSE_ERROR,
                    "Parse error",
                    Some(serde_json::json!(e.to_string())),
                ));
                let resp_bytes = serde_json::to_vec(&resp)
                    .map_err(|e| std::io::Error::other(format!("serialize: {e}")))?;
                let encrypted = keys
                    .encrypt(&resp_bytes)
                    .map_err(|e| std::io::Error::other(format!("Phase 3 encrypt: {e}")))?;
                framing::write_frame(&mut stream, &encrypted).await?;
                continue;
            }
        };

        let response = super::super::process_single_request(server, value, &gate, &caller).await;
        let resp_bytes = serde_json::to_vec(&response)
            .map_err(|e| std::io::Error::other(format!("serialize: {e}")))?;
        let encrypted = keys
            .encrypt(&resp_bytes)
            .map_err(|e| std::io::Error::other(format!("Phase 3 encrypt: {e}")))?;
        framing::write_frame(&mut stream, &encrypted).await?;
        stream.flush().await?;
    }
}
