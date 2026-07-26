// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2024–2026 ecoPrimals Project
//! BTSP consumer-side `ClientHello` handshake for connecting to bearDog in strict mode.
//!
//! When `BEARDOG_UDS_REQUIRE_BTSP=1` is set, bearDog rejects plain JSON-RPC.
//! Authenticates rhizoCrypt before JSON-RPC traffic using LOCAL HMAC-SHA256.
//!
//! Reference: `primals/songBird/crates/songbird-crypto-provider/src/btsp_client.rs`

use base64::Engine as _;
use base64::prelude::BASE64_STANDARD;
use hmac::{Hmac, Mac};
use serde::{Deserialize, Serialize};
use sha2::Sha256;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tracing::debug;

type HmacSha256 = Hmac<Sha256>;

const BTSP_VERSION: u8 = 1;
const PREFERRED_CIPHER: &str = "chacha20_poly1305";

#[derive(Debug, Serialize)]
struct ClientHello {
    protocol: &'static str,
    version: u8,
    client_ephemeral_pub: String,
}

#[derive(Debug, Deserialize)]
struct ServerHello {
    #[expect(dead_code, reason = "validated implicitly by successful parse")]
    version: u8,
    #[expect(dead_code, reason = "reserved for Phase 3 session key derivation")]
    server_ephemeral_pub: String,
    challenge: String,
    session_id: String,
}

#[derive(Debug, Serialize)]
struct ChallengeResponse {
    response: String,
    preferred_cipher: &'static str,
}

#[derive(Debug, Deserialize)]
struct HandshakeComplete {
    cipher: String,
    session_id: String,
}

#[derive(Debug, Deserialize)]
struct HandshakeError {
    #[expect(dead_code, reason = "logged but not matched on")]
    error: String,
    reason: String,
}

/// Errors from the BTSP client handshake.
#[derive(Debug, thiserror::Error)]
pub enum BtspClientError {
    /// Family seed not available in environment.
    #[error("FAMILY_SEED not available — cannot perform BTSP handshake")]
    NoFamilySeed,
    /// I/O error on the stream during handshake.
    #[error("I/O error during BTSP handshake: {0}")]
    Io(#[from] std::io::Error),
    /// Server explicitly rejected the handshake.
    #[error("Server rejected handshake: {0}")]
    Rejected(String),
    /// Malformed or unexpected response from server.
    #[error("Malformed server response: {0}")]
    Protocol(String),
    /// HMAC computation failed.
    #[error("HMAC computation failed")]
    Hmac,
}

/// Resolve the raw family seed from environment.
///
/// Checks: `BTSP_FAMILY_SEED` → `FAMILY_SEED` → `BIOMEOS_FAMILY_SEED`.
fn resolve_family_seed_raw() -> Option<String> {
    std::env::var("BTSP_FAMILY_SEED")
        .or_else(|_| std::env::var(crate::safe_env::SafeEnv::FAMILY_SEED))
        .or_else(|_| std::env::var("BIOMEOS_FAMILY_SEED"))
        .ok()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
}

/// Check whether BTSP strict mode is expected (bearDog requires handshake).
#[must_use]
pub fn btsp_strict_mode_expected() -> bool {
    std::env::var("BEARDOG_UDS_REQUIRE_BTSP")
        .or_else(|_| std::env::var("BTSP_STRICT_MODE"))
        .is_ok_and(|v| v.trim() == "1")
}

/// Perform the client-side BTSP handshake over an NDJSON stream.
///
/// # Errors
///
/// Returns [`BtspClientError`] if the family seed is unavailable, the server
/// rejects the handshake, or I/O fails.
pub async fn perform_client_handshake<S>(stream: &mut S) -> Result<(), BtspClientError>
where
    S: tokio::io::AsyncRead + tokio::io::AsyncWrite + Unpin,
{
    let family_seed = resolve_family_seed_raw().ok_or(BtspClientError::NoFamilySeed)?;

    let mut ephemeral_key = [0u8; 32];
    getrandom::fill(&mut ephemeral_key).map_err(|_| BtspClientError::Hmac)?;

    let hello = ClientHello {
        protocol: "btsp",
        version: BTSP_VERSION,
        client_ephemeral_pub: BASE64_STANDARD.encode(ephemeral_key),
    };
    let hello_json = serde_json::to_string(&hello)
        .map_err(|e| BtspClientError::Protocol(format!("serialize ClientHello: {e}")))?;
    stream.write_all(hello_json.as_bytes()).await.map_err(BtspClientError::Io)?;
    stream.write_all(b"\n").await.map_err(BtspClientError::Io)?;
    stream.flush().await.map_err(BtspClientError::Io)?;

    debug!("BTSP client: sent ClientHello");

    let mut buf_reader = BufReader::new(&mut *stream);
    let mut line = String::new();
    buf_reader.read_line(&mut line).await.map_err(BtspClientError::Io)?;

    if line.trim().is_empty() {
        return Err(BtspClientError::Protocol(String::from("empty response from server")));
    }

    if line.contains("\"error\"") && line.contains("\"reason\"") {
        let err: HandshakeError = serde_json::from_str(line.trim())
            .map_err(|e| BtspClientError::Protocol(format!("parse error response: {e}")))?;
        return Err(BtspClientError::Rejected(err.reason));
    }

    let server_hello: ServerHello = serde_json::from_str(line.trim())
        .map_err(|e| BtspClientError::Protocol(format!("parse ServerHello: {e}")))?;

    debug!(
        session_id = %server_hello.session_id,
        "BTSP client: received ServerHello"
    );

    let challenge_bytes = BASE64_STANDARD
        .decode(&server_hello.challenge)
        .map_err(|e| BtspClientError::Protocol(format!("decode challenge: {e}")))?;

    let mut mac =
        HmacSha256::new_from_slice(family_seed.as_bytes()).map_err(|_| BtspClientError::Hmac)?;
    mac.update(&challenge_bytes);
    let hmac_result = mac.finalize().into_bytes();

    let response = ChallengeResponse {
        response: BASE64_STANDARD.encode(hmac_result),
        preferred_cipher: PREFERRED_CIPHER,
    };
    let resp_json = serde_json::to_string(&response)
        .map_err(|e| BtspClientError::Protocol(format!("serialize ChallengeResponse: {e}")))?;

    let stream = buf_reader.into_inner();
    stream.write_all(resp_json.as_bytes()).await.map_err(BtspClientError::Io)?;
    stream.write_all(b"\n").await.map_err(BtspClientError::Io)?;
    stream.flush().await.map_err(BtspClientError::Io)?;

    debug!("BTSP client: sent ChallengeResponse");

    let mut buf_reader = BufReader::new(&mut *stream);
    line.clear();
    buf_reader.read_line(&mut line).await.map_err(BtspClientError::Io)?;

    if line.contains("\"error\"") && line.contains("\"reason\"") {
        let err: HandshakeError = serde_json::from_str(line.trim())
            .map_err(|e| BtspClientError::Protocol(format!("parse error response: {e}")))?;
        return Err(BtspClientError::Rejected(err.reason));
    }

    let complete: HandshakeComplete = serde_json::from_str(line.trim())
        .map_err(|e| BtspClientError::Protocol(format!("parse HandshakeComplete: {e}")))?;

    debug!(
        session_id = %complete.session_id,
        cipher = %complete.cipher,
        "BTSP client: handshake COMPLETE"
    );

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn btsp_strict_mode_default_off() {
        assert!(!btsp_strict_mode_expected());
    }

    #[test]
    fn hmac_produces_32_bytes() {
        let mut mac = HmacSha256::new_from_slice(b"test-seed").unwrap();
        mac.update(b"challenge-data");
        assert_eq!(mac.finalize().into_bytes().len(), 32);
    }
}
