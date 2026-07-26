// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2024–2026 ecoPrimals Project

//! BTSP client-side handshake for connecting to bearDog in strict mode.
//!
//! When `BEARDOG_UDS_REQUIRE_BTSP=1` is set, bearDog rejects plain JSON-RPC
//! with `-32600`. This module implements the consumer-side of the 4-step BTSP
//! handshake so rhizoCrypt can authenticate before sending requests.
//!
//! The challenge response uses LOCAL HMAC-SHA256 with the family seed — this
//! avoids the chicken-and-egg of needing bearDog to compute HMAC for the
//! handshake that authenticates us TO bearDog.
//!
//! ## Wire Format (NDJSON — newline-delimited)
//!
//! ```text
//! 1. Send  ClientHello       { protocol: "btsp", version: 1, client_ephemeral_pub }
//! 2. Read  ServerHello       { version, server_ephemeral_pub, challenge, session_id }
//! 3. Send  ChallengeResponse { response, preferred_cipher }
//! 4. Read  HandshakeComplete { cipher, session_id }
//! ```
//!
//! Reference: `songBird/crates/songbird-crypto-provider/src/btsp_client.rs`

use base64::Engine as _;
use base64::prelude::BASE64_STANDARD;
use hmac::{Hmac, Mac};
use serde::{Deserialize, Serialize};
use sha2::Sha256;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::UnixStream;
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
    #[expect(dead_code, reason = "used by session key derivation in Phase 3")]
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
    pub cipher: String,
    pub session_id: String,
}

#[derive(Debug, Deserialize)]
struct HandshakeError {
    #[expect(dead_code, reason = "logged but not matched on")]
    pub error: String,
    pub reason: String,
}

/// Result of a successful client-side BTSP handshake.
#[derive(Debug, Clone)]
pub struct BtspClientSession {
    /// Unique session identifier from the server.
    pub session_id: String,
    /// Negotiated cipher suite (e.g. `chacha20_poly1305` or `null`).
    pub cipher: String,
}

/// Errors from the BTSP client handshake.
#[derive(Debug, thiserror::Error)]
pub enum BtspClientError {
    /// Family seed not available in environment.
    #[error("FAMILY_SEED not available — cannot perform BTSP handshake")]
    NoFamilySeed,
    /// I/O error on the UDS stream during handshake.
    #[error("I/O error during BTSP handshake: {0}")]
    Io(#[from] std::io::Error),
    /// Server explicitly rejected the handshake (bad family seed, etc.).
    #[error("Server rejected handshake: {0}")]
    Rejected(String),
    /// Malformed or unexpected response from server.
    #[error("Malformed server response: {0}")]
    Protocol(String),
    /// HMAC computation failed (invalid key length).
    #[error("HMAC computation failed")]
    Hmac,
}

/// Check whether BTSP strict mode is expected.
#[must_use]
pub fn btsp_strict_mode_expected() -> bool {
    crate::SafeEnv::get_optional("BEARDOG_UDS_REQUIRE_BTSP")
        .or_else(|| crate::SafeEnv::get_optional("BTSP_STRICT_MODE"))
        .is_some_and(|v| v.trim() == "1")
}

/// Resolve family seed from environment (matches primalSpring convention).
fn resolve_family_seed() -> Option<String> {
    crate::SafeEnv::get_optional("RHIZOCRYPT_FAMILY_SEED")
        .or_else(|| crate::SafeEnv::get_optional(crate::SafeEnv::FAMILY_SEED))
        .or_else(|| crate::SafeEnv::get_optional("BEARDOG_FAMILY_SEED"))
        .filter(|s| !s.trim().is_empty())
}

/// Perform BTSP client handshake over a tokio [`UnixStream`].
/// After success, the stream is ready for NDJSON JSON-RPC.
///
/// # Errors
///
/// Returns [`BtspClientError`] if the family seed is unavailable, the server
/// rejects the handshake, or I/O fails.
pub async fn perform_client_handshake(
    stream: &mut UnixStream,
) -> std::result::Result<BtspClientSession, BtspClientError> {
    let family_seed = resolve_family_seed().ok_or(BtspClientError::NoFamilySeed)?;

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
    let mut mac = HmacSha256::new_from_slice(family_seed.trim().as_bytes())
        .map_err(|_| BtspClientError::Hmac)?;
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
    let mut line = String::new();
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

    Ok(BtspClientSession {
        session_id: complete.session_id,
        cipher: complete.cipher,
    })
}

#[cfg(test)]
#[expect(clippy::unwrap_used, reason = "test module")]
mod tests {
    use super::*;

    #[test]
    fn btsp_strict_mode_default_off() {
        temp_env::with_vars(
            [("BEARDOG_UDS_REQUIRE_BTSP", None::<&str>), ("BTSP_STRICT_MODE", None::<&str>)],
            || assert!(!btsp_strict_mode_expected()),
        );
    }

    #[test]
    fn btsp_strict_mode_on() {
        temp_env::with_vars([("BEARDOG_UDS_REQUIRE_BTSP", Some("1"))], || {
            assert!(btsp_strict_mode_expected());
        });
    }

    #[test]
    fn resolve_family_seed_from_env() {
        temp_env::with_vars(
            [("RHIZOCRYPT_FAMILY_SEED", None::<&str>), ("FAMILY_SEED", Some("test-seed"))],
            || assert_eq!(resolve_family_seed().unwrap(), "test-seed"),
        );
    }

    #[test]
    fn hmac_produces_32_bytes() {
        let key = b"test-family-seed";
        let challenge = b"random-challenge";
        let mut mac = HmacSha256::new_from_slice(key).unwrap();
        mac.update(challenge);
        assert_eq!(mac.finalize().into_bytes().len(), 32);
    }

    #[test]
    fn client_hello_serializes() {
        let hello = ClientHello {
            protocol: "btsp",
            version: 1,
            client_ephemeral_pub: String::from("AAAA"),
        };
        let json = serde_json::to_string(&hello).unwrap();
        assert!(json.contains("\"protocol\":\"btsp\""));
        assert!(json.contains("\"version\":1"));
    }
}
