// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2024–2026 ecoPrimals Project

//! Tests for provenance types and config.

#![allow(clippy::unwrap_used)]

use super::types::*;
use crate::types::{Did, PayloadRef, SessionId, Timestamp, VertexId};

#[test]
fn test_config_default() {
    let config = ProvenanceProviderConfig::default();
    assert!(config.push_address.is_none());
    assert_eq!(config.timeout_ms, 5000);
    assert!(config.cache_enabled);
}

#[test]
fn test_config_with_push_address() {
    let config = ProvenanceProviderConfig::with_push_address("127.0.0.1:9900");
    assert_eq!(config.push_address.as_deref(), Some("127.0.0.1:9900"));
}

#[test]
fn test_provenance_chain() {
    let mut chain = ProvenanceChain::new();
    assert!(chain.is_empty());

    chain.add_vertex(VertexRef {
        session_id: SessionId::now(),
        vertex_id: VertexId::from_bytes(b"test-vertex"),
        event_type: "test".to_string(),
        agent: Some(Did::new("did:key:test")),
        timestamp: Timestamp::now(),
        payload_ref: None,
    });

    assert_eq!(chain.len(), 1);
    assert!(chain.agents.contains("did:key:test"));
}

#[test]
fn test_vertex_query_builder() {
    let query = VertexQuery::new().with_agent(Did::new("did:key:test")).with_limit(100);

    assert!(query.agent.is_some());
    assert_eq!(query.limit, Some(100));
}

#[test]
fn test_vertex_ref_creation() {
    let session_id = SessionId::now();
    let vertex_id = VertexId::from_bytes(b"test-vertex-123");
    let did = Did::new("did:key:agent1");
    let timestamp = Timestamp::now();
    let payload = PayloadRef::from_bytes(b"payload-data");

    let vertex = VertexRef {
        session_id,
        vertex_id,
        event_type: "test.event".to_string(),
        agent: Some(did.clone()),
        timestamp,
        payload_ref: Some(payload),
    };

    assert_eq!(vertex.session_id, session_id);
    assert_eq!(vertex.vertex_id, vertex_id);
    assert_eq!(vertex.event_type, "test.event");
    assert_eq!(vertex.agent, Some(did));
    assert!(vertex.payload_ref.is_some());
}

#[test]
fn test_vertex_ref_without_optional_fields() {
    let vertex = VertexRef {
        session_id: SessionId::now(),
        vertex_id: VertexId::from_bytes(b"vertex"),
        event_type: "event".to_string(),
        agent: None,
        timestamp: Timestamp::now(),
        payload_ref: None,
    };

    assert!(vertex.agent.is_none());
    assert!(vertex.payload_ref.is_none());
}

#[test]
fn test_provenance_chain_new() {
    let chain = ProvenanceChain::new();
    assert!(chain.is_empty());
    assert_eq!(chain.len(), 0);
    assert!(chain.vertices.is_empty());
    assert!(chain.agents.is_empty());
    assert!(chain.data_hashes.is_empty());
}

#[test]
fn test_provenance_chain_add_vertex_with_agent() {
    let mut chain = ProvenanceChain::new();
    let did = Did::new("did:key:agent1");

    chain.add_vertex(VertexRef {
        session_id: SessionId::now(),
        vertex_id: VertexId::from_bytes(b"v1"),
        event_type: "test".to_string(),
        agent: Some(did),
        timestamp: Timestamp::now(),
        payload_ref: None,
    });

    assert_eq!(chain.len(), 1);
    assert_eq!(chain.agents.len(), 1);
    assert!(chain.agents.contains("did:key:agent1"));
}

#[test]
fn test_provenance_chain_add_vertex_with_payload() {
    let mut chain = ProvenanceChain::new();
    let payload = PayloadRef::from_bytes(b"test-data");

    chain.add_vertex(VertexRef {
        session_id: SessionId::now(),
        vertex_id: VertexId::from_bytes(b"v1"),
        event_type: "test".to_string(),
        agent: None,
        timestamp: Timestamp::now(),
        payload_ref: Some(payload),
    });

    assert_eq!(chain.len(), 1);
    assert_eq!(chain.data_hashes.len(), 1);
    assert!(chain.data_hashes.contains(&payload.hash));
}

#[test]
fn test_provenance_chain_multiple_vertices() {
    let mut chain = ProvenanceChain::new();
    let did1 = Did::new("did:key:agent1");
    let did2 = Did::new("did:key:agent2");

    chain.add_vertex(VertexRef {
        session_id: SessionId::now(),
        vertex_id: VertexId::from_bytes(b"v1"),
        event_type: "event1".to_string(),
        agent: Some(did1),
        timestamp: Timestamp::now(),
        payload_ref: None,
    });

    chain.add_vertex(VertexRef {
        session_id: SessionId::now(),
        vertex_id: VertexId::from_bytes(b"v2"),
        event_type: "event2".to_string(),
        agent: Some(did2),
        timestamp: Timestamp::now(),
        payload_ref: None,
    });

    assert_eq!(chain.len(), 2);
    assert_eq!(chain.agents.len(), 2);
    assert!(chain.agents.contains("did:key:agent1"));
    assert!(chain.agents.contains("did:key:agent2"));
}

#[test]
fn test_provenance_chain_default() {
    let chain = ProvenanceChain::default();
    assert!(chain.is_empty());
}

#[test]
fn test_agent_contribution_creation() {
    let did = Did::new("did:key:worker");
    let contribution = AgentContribution {
        agent: did.clone(),
        event_count: 42,
        event_types: vec!["task.created".to_string(), "task.completed".to_string()],
        first_event: Timestamp::now(),
        last_event: Timestamp::now(),
    };

    assert_eq!(contribution.agent, did);
    assert_eq!(contribution.event_count, 42);
    assert_eq!(contribution.event_types.len(), 2);
}

#[test]
fn test_session_attribution_creation() {
    let session_id = SessionId::now();
    let did = Did::new("did:key:agent");

    let attribution = SessionAttribution {
        session_id,
        session_type: "ml-training".to_string(),
        agents: vec![AgentContribution {
            agent: did,
            event_count: 10,
            event_types: vec!["compute".to_string()],
            first_event: Timestamp::now(),
            last_event: Timestamp::now(),
        }],
        data_inputs: vec![[1u8; 32], [2u8; 32]],
        data_outputs: vec![[3u8; 32]],
        merkle_root: [0u8; 32],
    };

    assert_eq!(attribution.session_id, session_id);
    assert_eq!(attribution.agents.len(), 1);
    assert_eq!(attribution.data_inputs.len(), 2);
    assert_eq!(attribution.data_outputs.len(), 1);
}

#[test]
fn test_vertex_query_new() {
    let query = VertexQuery::new();
    assert!(query.agent.is_none());
    assert!(query.session_id.is_none());
    assert!(query.event_types.is_none());
    assert!(query.after.is_none());
    assert!(query.before.is_none());
    assert!(query.payload_hash.is_none());
    assert!(query.limit.is_none());
}

#[test]
fn test_vertex_query_with_agent() {
    let did = Did::new("did:key:test");
    let query = VertexQuery::new().with_agent(did.clone());
    assert_eq!(query.agent, Some(did));
}

#[test]
fn test_vertex_query_with_session() {
    let session_id = SessionId::now();
    let query = VertexQuery::new().with_session(session_id);
    assert_eq!(query.session_id, Some(session_id));
}

#[test]
fn test_vertex_query_with_event_types() {
    let query = VertexQuery::new()
        .with_event_types(vec!["task.created".to_string(), "task.completed".to_string()]);
    assert_eq!(
        query.event_types,
        Some(vec!["task.created".to_string(), "task.completed".to_string()])
    );
}

#[test]
fn test_vertex_query_with_limit() {
    let query = VertexQuery::new().with_limit(50);
    assert_eq!(query.limit, Some(50));
}

#[test]
fn test_vertex_query_with_time_range() {
    let mut query = VertexQuery::new();
    let after_ts = Timestamp::now();
    let before_ts = Timestamp::now();
    query.after = Some(after_ts);
    query.before = Some(before_ts);
    assert_eq!(query.after, Some(after_ts));
    assert_eq!(query.before, Some(before_ts));
}

#[test]
fn test_vertex_query_chaining() {
    let did = Did::new("did:key:test");
    let session_id = SessionId::now();

    let query = VertexQuery::new()
        .with_agent(did.clone())
        .with_session(session_id)
        .with_event_types(vec!["test.event".to_string()])
        .with_limit(100);

    assert_eq!(query.agent, Some(did));
    assert_eq!(query.session_id, Some(session_id));
    assert_eq!(query.event_types, Some(vec!["test.event".to_string()]));
    assert_eq!(query.limit, Some(100));
}

#[test]
fn test_config_from_env() {
    let config = ProvenanceProviderConfig::from_env();
    assert!(config.timeout_ms > 0);
}

#[test]
fn test_config_with_custom_values() {
    let mut config = ProvenanceProviderConfig::default();
    config.timeout_ms = 10000;
    config.cache_ttl_secs = 120;
    config.cache_enabled = false;

    assert_eq!(config.timeout_ms, 10000);
    assert_eq!(config.cache_ttl_secs, 120);
    assert!(!config.cache_enabled);
}

#[test]
fn test_client_state_default() {
    let state = ClientState::default();
    assert_eq!(state, ClientState::Disconnected);
}

#[test]
fn test_client_state_connected() {
    let state1 = ClientState::Disconnected;
    let state2 = ClientState::Connected;
    assert_ne!(state1, state2);
}

#[test]
fn test_vertex_ref_serialization() {
    let vertex = VertexRef {
        session_id: SessionId::now(),
        vertex_id: VertexId::from_bytes(b"test"),
        event_type: "test".to_string(),
        agent: Some(Did::new("did:key:test")),
        timestamp: Timestamp::now(),
        payload_ref: None,
    };

    let serialized = serde_json::to_string(&vertex).unwrap();
    let deserialized: VertexRef = serde_json::from_str(&serialized).unwrap();
    assert_eq!(deserialized.event_type, "test");
}

#[test]
fn test_provenance_chain_serialization() {
    let mut chain = ProvenanceChain::new();
    chain.add_vertex(VertexRef {
        session_id: SessionId::now(),
        vertex_id: VertexId::from_bytes(b"v1"),
        event_type: "test".to_string(),
        agent: Some(Did::new("did:key:test")),
        timestamp: Timestamp::now(),
        payload_ref: None,
    });

    let serialized = serde_json::to_string(&chain).unwrap();
    let deserialized: ProvenanceChain = serde_json::from_str(&serialized).unwrap();
    assert_eq!(deserialized.len(), 1);
}

#[test]
fn test_agent_contribution_serialization() {
    let contribution = AgentContribution {
        agent: Did::new("did:key:test"),
        event_count: 5,
        event_types: vec!["test".to_string()],
        first_event: Timestamp::now(),
        last_event: Timestamp::now(),
    };

    let serialized = serde_json::to_string(&contribution).unwrap();
    let deserialized: AgentContribution = serde_json::from_str(&serialized).unwrap();
    assert_eq!(deserialized.event_count, 5);
}

#[test]
fn test_vertex_query_serialization() {
    let query = VertexQuery::new().with_agent(Did::new("did:key:test")).with_limit(100);

    let serialized = serde_json::to_string(&query).unwrap();
    let deserialized: VertexQuery = serde_json::from_str(&serialized).unwrap();
    assert_eq!(deserialized.limit, Some(100));
}
