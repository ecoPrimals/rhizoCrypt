// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2024–2026 ecoPrimals Project

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
