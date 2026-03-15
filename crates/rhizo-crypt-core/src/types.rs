// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2024–2026 ecoPrimals Project

//! Core type definitions for RhizoCrypt.
//!
//! This module defines the fundamental types used throughout the DAG engine.

use bytes::Bytes;
use serde::{Deserialize, Serialize};
use std::fmt;

/// 32-byte content hash (Blake3).
pub type ContentHash = [u8; 32];

/// Vertex identifier - Blake3 hash of the vertex content.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct VertexId(pub ContentHash);

impl VertexId {
    /// The zero vertex ID (used for padding in Merkle trees).
    pub const ZERO: Self = Self([0u8; 32]);

    /// Create a new vertex ID from a hash.
    #[inline]
    #[must_use]
    pub const fn new(hash: ContentHash) -> Self {
        Self(hash)
    }

    /// Create a vertex ID from bytes (computes Blake3 hash).
    #[inline]
    #[must_use]
    pub fn from_bytes(data: &[u8]) -> Self {
        Self(blake3::hash(data).into())
    }

    /// Get the underlying bytes (zero-copy).
    #[inline]
    #[must_use]
    pub const fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }

    /// Convert to a hex string.
    #[must_use]
    pub fn to_hex(&self) -> String {
        hex_encode(&self.0)
    }
}

impl fmt::Debug for VertexId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "VertexId({})", &self.to_hex()[..16])
    }
}

impl fmt::Display for VertexId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", &self.to_hex()[..16])
    }
}

/// Session identifier - UUID v7 for time-ordering.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SessionId(pub uuid::Uuid);

impl SessionId {
    /// Create a new session ID with current timestamp.
    #[must_use]
    pub fn now() -> Self {
        Self(uuid::Uuid::now_v7())
    }

    /// Create a session ID from a UUID.
    #[must_use]
    pub const fn new(uuid: uuid::Uuid) -> Self {
        Self(uuid)
    }

    /// Get the underlying UUID.
    #[must_use]
    pub const fn as_uuid(&self) -> &uuid::Uuid {
        &self.0
    }

    /// Get the session ID as bytes.
    #[must_use]
    pub const fn as_bytes(&self) -> &[u8] {
        self.0.as_bytes()
    }
}

impl fmt::Debug for SessionId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "SessionId({})", self.0)
    }
}

impl fmt::Display for SessionId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Slice identifier - UUID v7 for time-ordering.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SliceId(pub uuid::Uuid);

impl SliceId {
    /// Create a new slice ID with current timestamp.
    #[must_use]
    pub fn now() -> Self {
        Self(uuid::Uuid::now_v7())
    }

    /// Create a slice ID from a UUID.
    #[must_use]
    pub const fn new(uuid: uuid::Uuid) -> Self {
        Self(uuid)
    }
}

impl fmt::Debug for SliceId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "SliceId({})", self.0)
    }
}

impl fmt::Display for SliceId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Payload reference - Blake3 hash of payload content.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PayloadRef {
    /// Blake3 hash of the payload.
    pub hash: ContentHash,
    /// Size of the payload in bytes.
    pub size: u64,
}

impl PayloadRef {
    /// Create a new payload reference from bytes.
    #[must_use]
    pub fn from_bytes(data: &[u8]) -> Self {
        Self {
            hash: blake3::hash(data).into(),
            size: data.len() as u64,
        }
    }

    /// Create a payload reference with known hash and size.
    #[must_use]
    pub const fn new(hash: ContentHash, size: u64) -> Self {
        Self {
            hash,
            size,
        }
    }

    /// Convert hash to hex string.
    #[must_use]
    pub fn to_hex(&self) -> String {
        hex_encode(&self.hash)
    }

    /// Create a payload reference from a hash (size unknown, stored as 0).
    #[must_use]
    pub fn from_hash(hash: &[u8]) -> Self {
        let mut content_hash = [0u8; 32];
        let len = hash.len().min(32);
        content_hash[..len].copy_from_slice(&hash[..len]);
        Self {
            hash: content_hash,
            size: 0,
        }
    }

    /// Get the hash bytes.
    #[must_use]
    pub const fn as_bytes(&self) -> &[u8; 32] {
        &self.hash
    }
}

impl fmt::Debug for PayloadRef {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "PayloadRef({}..., {} bytes)", &self.to_hex()[..16], self.size)
    }
}

impl fmt::Display for PayloadRef {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_hex())
    }
}

/// BearDog Decentralized Identifier (DID).
#[derive(Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Did(pub String);

impl Did {
    /// Create a new DID from a string.
    #[must_use]
    pub fn new(did: impl Into<String>) -> Self {
        Self(did.into())
    }

    /// Get the DID as a string.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Debug for Did {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Did({})", self.0)
    }
}

impl fmt::Display for Did {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Default for Did {
    fn default() -> Self {
        Self("did:key:anonymous".to_string())
    }
}

/// Cryptographic signature.
///
/// Uses `bytes::Bytes` internally for O(1) cloning — signatures are
/// often shared across threads and passed through RPC layers.
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Signature(pub Bytes);

impl Signature {
    /// Create a new signature from a byte vector.
    #[must_use]
    pub fn new(bytes: Vec<u8>) -> Self {
        Self(Bytes::from(bytes))
    }

    /// Create a signature from a static byte slice (zero-copy).
    #[must_use]
    pub const fn from_static(bytes: &'static [u8]) -> Self {
        Self(Bytes::from_static(bytes))
    }

    /// Get the signature bytes.
    #[must_use]
    pub fn as_bytes(&self) -> &[u8] {
        &self.0
    }

    /// Get the inner `Bytes` (zero-copy clone).
    #[must_use]
    pub fn into_bytes(self) -> Bytes {
        self.0
    }
}

impl fmt::Debug for Signature {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Signature({} bytes)", self.0.len())
    }
}

impl From<Vec<u8>> for Signature {
    fn from(v: Vec<u8>) -> Self {
        Self(Bytes::from(v))
    }
}

impl From<Bytes> for Signature {
    fn from(b: Bytes) -> Self {
        Self(b)
    }
}

/// Timestamp in nanoseconds since Unix epoch.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct Timestamp(pub u64);

impl Timestamp {
    /// Create a timestamp for the current time.
    ///
    /// Uses zero as fallback if system time is somehow before Unix epoch.
    #[inline]
    #[must_use]
    pub fn now() -> Self {
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_or(0, |d| u64::try_from(d.as_nanos()).unwrap_or(u64::MAX));
        Self(nanos)
    }

    /// Create a timestamp from nanoseconds.
    #[inline]
    #[must_use]
    pub const fn from_nanos(nanos: u64) -> Self {
        Self(nanos)
    }

    /// Get the timestamp in nanoseconds (zero-copy).
    #[inline]
    #[must_use]
    pub const fn as_nanos(&self) -> u64 {
        self.0
    }

    /// Get the timestamp in seconds.
    #[inline]
    #[must_use]
    pub const fn as_secs(&self) -> u64 {
        self.0 / 1_000_000_000
    }
}

impl fmt::Debug for Timestamp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Timestamp({}ns)", self.0)
    }
}

impl fmt::Display for Timestamp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}ns", self.0)
    }
}

/// Encode bytes as hex string.
fn hex_encode(bytes: &[u8]) -> String {
    use std::fmt::Write;
    bytes.iter().fold(String::with_capacity(bytes.len() * 2), |mut s, b| {
        let _ = write!(s, "{b:02x}");
        s
    })
}

/// Compute Blake3 hash of two hashes (for Merkle trees).
///
/// This is a hot path function - inlined for performance.
#[inline]
#[must_use]
pub fn hash_pair(left: &ContentHash, right: &ContentHash) -> ContentHash {
    let mut hasher = blake3::Hasher::new();
    hasher.update(left);
    hasher.update(right);
    hasher.finalize().into()
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;
    use crate::merkle::MerkleRoot;

    #[test]
    fn test_vertex_id() {
        let id = VertexId::from_bytes(b"test data");
        assert!(!id.to_hex().is_empty());
        assert_eq!(id.as_bytes().len(), 32);
    }

    #[test]
    fn test_vertex_id_zero() {
        assert_eq!(VertexId::ZERO.0, [0u8; 32]);
    }

    #[test]
    fn test_session_id() {
        let id = SessionId::now();
        assert!(!id.to_string().is_empty());
    }

    #[test]
    fn test_payload_ref() {
        let data = b"test payload";
        let payload_ref = PayloadRef::from_bytes(data);
        assert_eq!(payload_ref.size, data.len() as u64);
    }

    #[test]
    fn test_timestamp() {
        let ts = Timestamp::now();
        assert!(ts.as_nanos() > 0);
        assert!(ts.as_secs() > 0);
    }

    #[test]
    fn test_hash_pair() {
        let a = [1u8; 32];
        let b = [2u8; 32];
        let result = hash_pair(&a, &b);
        assert_ne!(result, a);
        assert_ne!(result, b);
    }

    #[test]
    fn test_did() {
        let did = Did::new("did:key:z6MkTest");
        assert_eq!(did.as_str(), "did:key:z6MkTest");
    }

    #[test]
    fn test_session_id_from_uuid() {
        let uuid = uuid::Uuid::parse_str("018e1234-5678-7abc-def0-123456789abc").unwrap();
        let session_id = SessionId::new(uuid);
        assert_eq!(session_id.as_uuid(), &uuid);
        assert_eq!(session_id.to_string(), uuid.to_string());
    }

    #[test]
    fn test_session_id_as_bytes() {
        let uuid = uuid::Uuid::parse_str("018e1234-5678-7abc-def0-123456789abc").unwrap();
        let session_id = SessionId::new(uuid);
        let bytes = session_id.as_bytes();
        assert_eq!(bytes.len(), 16);
        let roundtrip = uuid::Uuid::from_bytes(bytes.try_into().unwrap());
        assert_eq!(roundtrip, uuid);
    }

    #[test]
    fn test_vertex_id_from_bytes() {
        let data = b"deterministic input";
        let id1 = VertexId::from_bytes(data);
        let id2 = VertexId::from_bytes(data);
        assert_eq!(id1, id2);
        assert_eq!(id1.as_bytes().len(), 32);
        assert_ne!(id1, VertexId::from_bytes(b"different"));
    }

    #[test]
    fn test_vertex_id_display() {
        let id = VertexId::from_bytes(b"display test");
        let s = format!("{id}");
        assert_eq!(s.len(), 16);
        assert!(s.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn test_timestamp_from_nanos() {
        let ts = Timestamp::from_nanos(1_000_000_000);
        assert_eq!(ts.as_nanos(), 1_000_000_000);
        assert_eq!(ts.as_secs(), 1);
    }

    #[test]
    fn test_timestamp_ordering() {
        let ts1 = Timestamp::from_nanos(100);
        let ts2 = Timestamp::from_nanos(200);
        let ts3 = Timestamp::from_nanos(100);
        assert!(ts1 < ts2);
        assert!(ts2 > ts1);
        assert_eq!(ts1, ts3);
        assert!(ts1 <= ts3);
    }

    #[test]
    fn test_payload_ref_from_bytes() {
        let data = b"content-addressed hash";
        let pref = PayloadRef::from_bytes(data);
        assert_eq!(pref.size, data.len() as u64);
        assert_eq!(pref.hash, *blake3::hash(data).as_bytes());
        let pref2 = PayloadRef::from_bytes(data);
        assert_eq!(pref.hash, pref2.hash);
    }

    #[test]
    fn test_did_equality() {
        let did1 = Did::new("did:key:z6MkTest");
        let did2 = Did::new("did:key:z6MkTest");
        let did3 = Did::new("did:key:other");
        assert_eq!(did1, did2);
        assert_ne!(did1, did3);
    }

    #[test]
    fn test_slice_id() {
        let slice_id = SliceId::now();
        assert!(!slice_id.to_string().is_empty());
        let uuid = uuid::Uuid::now_v7();
        let from_uuid = SliceId::new(uuid);
        assert_eq!(from_uuid.to_string(), uuid.to_string());
    }

    #[test]
    fn test_merkle_root() {
        let hash: ContentHash = [42u8; 32];
        let root = MerkleRoot::new(hash);
        assert_eq!(root.as_bytes(), &hash);
        assert!(!root.to_hex().is_empty());
        let s = format!("{root}");
        assert_eq!(s.len(), 16);
        assert_eq!(MerkleRoot::ZERO, MerkleRoot::new([0u8; 32]));
    }

    #[test]
    fn test_signature_new_and_access() {
        let sig = Signature::new(vec![1, 2, 3, 4]);
        assert_eq!(sig.as_bytes(), &[1, 2, 3, 4]);
        let bytes = sig.into_bytes();
        assert_eq!(&bytes[..], &[1, 2, 3, 4]);
    }

    #[test]
    fn test_signature_from_static() {
        let sig = Signature::from_static(&[0xDE, 0xAD]);
        assert_eq!(sig.as_bytes(), &[0xDE, 0xAD]);
    }

    #[test]
    fn test_signature_from_impls() {
        let from_vec: Signature = vec![10, 20].into();
        assert_eq!(from_vec.as_bytes(), &[10, 20]);

        let from_bytes: Signature = bytes::Bytes::from_static(&[30, 40]).into();
        assert_eq!(from_bytes.as_bytes(), &[30, 40]);
    }

    #[test]
    fn test_signature_debug() {
        let sig = Signature::new(vec![1, 2, 3]);
        let dbg = format!("{sig:?}");
        assert!(dbg.contains("3 bytes"));
    }

    #[test]
    fn test_signature_clone_is_cheap() {
        let sig = Signature::new(vec![0u8; 1024]);
        let cloned = sig.clone();
        assert_eq!(sig, cloned);
    }

    #[test]
    fn test_payload_ref_new_and_as_bytes() {
        let hash = [99u8; 32];
        let pref = PayloadRef::new(hash, 512);
        assert_eq!(pref.as_bytes(), &hash);
        assert_eq!(pref.size, 512);
    }

    #[test]
    fn test_payload_ref_from_hash_short() {
        let short_hash = &[1, 2, 3];
        let pref = PayloadRef::from_hash(short_hash);
        assert_eq!(&pref.hash[..3], &[1, 2, 3]);
        assert_eq!(&pref.hash[3..], &[0u8; 29]);
        assert_eq!(pref.size, 0);
    }

    #[test]
    fn test_payload_ref_from_hash_exact() {
        let exact = [42u8; 32];
        let pref = PayloadRef::from_hash(&exact);
        assert_eq!(pref.hash, exact);
    }

    #[test]
    fn test_payload_ref_from_hash_long() {
        let long = vec![7u8; 64];
        let pref = PayloadRef::from_hash(&long);
        assert_eq!(pref.hash, [7u8; 32]);
    }

    #[test]
    fn test_payload_ref_to_hex() {
        let pref = PayloadRef::from_bytes(b"hex test");
        let hex = pref.to_hex();
        assert_eq!(hex.len(), 64);
        assert!(hex.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn test_payload_ref_debug_display() {
        let pref = PayloadRef::from_bytes(b"debug test");
        let dbg = format!("{pref:?}");
        assert!(dbg.contains("PayloadRef("));
        assert!(dbg.contains("bytes)"));
        let disp = format!("{pref}");
        assert_eq!(disp.len(), 64);
    }

    #[test]
    fn test_did_default() {
        let did = Did::default();
        assert_eq!(did.as_str(), "did:key:anonymous");
    }

    #[test]
    fn test_did_debug_display() {
        let did = Did::new("did:key:test");
        let dbg = format!("{did:?}");
        assert!(dbg.contains("Did(did:key:test)"));
        let disp = format!("{did}");
        assert_eq!(disp, "did:key:test");
    }

    #[test]
    fn test_slice_id_debug_display() {
        let uuid = uuid::Uuid::now_v7();
        let sid = SliceId::new(uuid);
        let dbg = format!("{sid:?}");
        assert!(dbg.contains("SliceId("));
        let disp = format!("{sid}");
        assert_eq!(disp, uuid.to_string());
    }

    #[test]
    fn test_hex_encode_roundtrip() {
        let data = [0xAB, 0xCD, 0xEF, 0x01];
        let hex = hex_encode(&data);
        assert_eq!(hex, "abcdef01");
    }
}
