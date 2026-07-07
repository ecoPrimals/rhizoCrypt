// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2024–2026 ecoPrimals Project

//! Tests for `TransportEndpoint` constructors, parsing, `Display`, and serde.

use super::*;

#[test]
fn test_transport_endpoint_tcp_constructor() {
    let ep = TransportEndpoint::tcp("myhost", 9400);
    assert_eq!(ep.tcp_addr(), Some(("myhost", 9400)));
}

#[test]
fn test_transport_endpoint_tcp_addr_returns_none_for_uds() {
    let ep = TransportEndpoint::uds("/run/test.sock");
    assert!(ep.tcp_addr().is_none());
}

#[test]
fn test_transport_endpoint_tcp_addr_returns_none_for_mesh_relay() {
    let ep = TransportEndpoint::MeshRelay {
        peer_id: "peer".into(),
        capability: "cap".into(),
    };
    assert!(ep.tcp_addr().is_none());
}

#[test]
fn test_transport_endpoint_uds_constructor() {
    let ep = TransportEndpoint::uds("/tmp/my.sock");
    match ep {
        TransportEndpoint::Uds {
            path,
        } => assert_eq!(path, "/tmp/my.sock"),
        _ => panic!("expected UDS"),
    }
}

// ── TransportEndpoint::try_parse_address ─────────────────────────

#[test]
fn test_try_parse_address_absolute_path() {
    let ep = TransportEndpoint::try_parse_address("/run/eco/rhizocrypt.sock").unwrap();
    assert!(matches!(ep, TransportEndpoint::Uds { path } if path == "/run/eco/rhizocrypt.sock"));
}

#[test]
fn test_try_parse_address_sock_suffix() {
    let ep = TransportEndpoint::try_parse_address("rhizocrypt.sock").unwrap();
    assert!(matches!(ep, TransportEndpoint::Uds { path } if path == "rhizocrypt.sock"));
}

#[test]
fn test_try_parse_address_sock_suffix_case_insensitive() {
    let ep = TransportEndpoint::try_parse_address("myService.SOCK").unwrap();
    assert!(matches!(ep, TransportEndpoint::Uds { .. }));
}

#[test]
fn test_try_parse_address_host_port() {
    let ep = TransportEndpoint::try_parse_address("192.168.1.1:9300").unwrap();
    assert_eq!(ep.tcp_addr(), Some(("192.168.1.1", 9300)));
}

#[test]
fn test_try_parse_address_localhost_port() {
    let ep = TransportEndpoint::try_parse_address("localhost:7700").unwrap();
    assert_eq!(ep.tcp_addr(), Some(("localhost", 7700)));
}

#[test]
fn test_try_parse_address_empty_host_returns_none() {
    assert!(TransportEndpoint::try_parse_address(":8080").is_none());
}

#[test]
fn test_try_parse_address_no_port_returns_none() {
    assert!(TransportEndpoint::try_parse_address("just-a-hostname").is_none());
}

#[test]
fn test_try_parse_address_invalid_port_returns_none() {
    assert!(TransportEndpoint::try_parse_address("host:notaport").is_none());
}

// ── TransportEndpoint::parse_address ─────────────────────────────

#[test]
fn test_parse_address_tcp() {
    let ep = TransportEndpoint::parse_address("myhost:9400");
    assert_eq!(ep.tcp_addr(), Some(("myhost", 9400)));
}

#[test]
fn test_parse_address_uds_with_slash() {
    let ep = TransportEndpoint::parse_address("/run/eco/test.sock");
    assert!(matches!(ep, TransportEndpoint::Uds { .. }));
}

#[test]
fn test_parse_address_sock_suffix_without_slash() {
    let ep = TransportEndpoint::parse_address("mysvc.sock");
    assert!(matches!(ep, TransportEndpoint::Uds { .. }));
}

#[test]
fn test_parse_address_unrecognized_falls_back_to_uds() {
    let ep = TransportEndpoint::parse_address("garbage");
    assert!(matches!(ep, TransportEndpoint::Uds { path } if path == "garbage"));
}

// ── TransportEndpoint Display ────────────────────────────────────

#[test]
fn test_display_uds() {
    let ep = TransportEndpoint::uds("/run/eco/test.sock");
    assert_eq!(ep.to_string(), "unix:///run/eco/test.sock");
}

#[test]
fn test_display_tcp() {
    let ep = TransportEndpoint::tcp("127.0.0.1", 9300);
    assert_eq!(ep.to_string(), "tcp://127.0.0.1:9300");
}

#[test]
fn test_display_mesh_relay() {
    let ep = TransportEndpoint::MeshRelay {
        peer_id: "strand-gate".into(),
        capability: "security".into(),
    };
    assert_eq!(ep.to_string(), "mesh://strand-gate/security");
}

// ── TransportEndpoint From<SocketAddr> ───────────────────────────

#[test]
fn test_from_socket_addr() {
    let addr: std::net::SocketAddr = "192.168.1.100:7700".parse().unwrap();
    let ep = TransportEndpoint::from(addr);
    assert_eq!(ep.tcp_addr(), Some(("192.168.1.100", 7700)));
}

// ── TransportEndpoint serde roundtrip ────────────────────────────

#[test]
fn test_serde_roundtrip_uds() {
    let ep = TransportEndpoint::uds("/run/eco/test.sock");
    let json = serde_json::to_string(&ep).unwrap();
    let back: TransportEndpoint = serde_json::from_str(&json).unwrap();
    assert_eq!(ep, back);
}

#[test]
fn test_serde_roundtrip_tcp() {
    let ep = TransportEndpoint::tcp("localhost", 9300);
    let json = serde_json::to_string(&ep).unwrap();
    let back: TransportEndpoint = serde_json::from_str(&json).unwrap();
    assert_eq!(ep, back);
}

#[test]
fn test_serde_roundtrip_mesh_relay() {
    let ep = TransportEndpoint::MeshRelay {
        peer_id: "peer-1".into(),
        capability: "storage".into(),
    };
    let json = serde_json::to_string(&ep).unwrap();
    let back: TransportEndpoint = serde_json::from_str(&json).unwrap();
    assert_eq!(ep, back);
}
