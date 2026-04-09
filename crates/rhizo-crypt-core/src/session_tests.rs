use super::*;

#[test]
fn test_session_builder() {
    let session = SessionBuilder::new(SessionType::General)
        .with_name("Test Session")
        .with_owner(Did::new("did:key:test"))
        .with_max_vertices(1000)
        .build();

    assert_eq!(session.name, Some("Test Session".to_string()));
    assert!(session.is_active());
    assert!(!session.is_terminal());
    assert_eq!(session.vertex_count, 0);
}

#[test]
fn test_session_state_transitions() {
    let mut session = SessionBuilder::new(SessionType::General).build();

    assert!(session.is_active());

    // Begin resolve
    session.begin_resolve().unwrap();
    assert!(!session.is_active());
    assert!(matches!(session.state, SessionState::Resolving { .. }));

    // Commit
    let commit_ref = CommitRef {
        spine_id: "test".to_string(),
        entry_hash: [0u8; 32],
        index: 1,
    };
    session.commit(commit_ref).unwrap();
    assert!(session.is_terminal());
}

#[test]
fn test_session_discard() {
    let mut session = SessionBuilder::new(SessionType::General).build();

    session.discard(DiscardReason::Timeout);
    assert!(session.is_terminal());
    assert!(matches!(session.state, SessionState::Discarded { .. }));
}

#[test]
fn test_session_frontier_update() {
    let mut session = SessionBuilder::new(SessionType::General).build();

    let v1 = VertexId::from_bytes(b"vertex1");
    session.update_frontier(v1, &[]);
    assert!(session.genesis.contains(&v1));
    assert!(session.frontier.contains(&v1));
    assert_eq!(session.vertex_count, 1);

    let v2 = VertexId::from_bytes(b"vertex2");
    session.update_frontier(v2, &[v1]);
    assert!(!session.frontier.contains(&v1));
    assert!(session.frontier.contains(&v2));
    assert_eq!(session.vertex_count, 2);
}

#[test]
fn test_session_type_default() {
    assert_eq!(SessionType::default(), SessionType::General);
}

#[test]
fn test_session_with_parent() {
    let mut session = SessionBuilder::new(SessionType::General).build();
    let v1 = VertexId::from_bytes(b"parent1");
    let v2 = VertexId::from_bytes(b"parent2");
    session.update_frontier(v1, &[]);
    session.update_frontier(v2, &[]);
    let v3 = VertexId::from_bytes(b"child");
    session.update_frontier(v3, &[v1, v2]);
    assert!(!session.frontier.contains(&v1));
    assert!(!session.frontier.contains(&v2));
    assert!(session.frontier.contains(&v3));
    assert!(session.genesis.contains(&v1));
    assert!(session.genesis.contains(&v2));
    assert!(!session.genesis.contains(&v3));
}

#[test]
fn test_session_with_max_vertices() {
    let session = SessionBuilder::new(SessionType::General).with_max_vertices(500).build();
    assert_eq!(session.config.max_vertices, 500);
}

#[test]
fn test_session_with_ttl() {
    let session = SessionBuilder::new(SessionType::General)
        .with_max_duration(Duration::from_secs(7200))
        .build();
    assert_eq!(session.config.max_duration, Duration::from_secs(7200));
}

#[test]
fn test_session_add_agent() {
    let mut session = SessionBuilder::new(SessionType::General).build();
    let agent1 = Did::new("did:key:agent1");
    let agent2 = Did::new("did:key:agent2");
    session.add_agent(agent1.clone());
    session.add_agent(agent2.clone());
    session.add_agent(agent1.clone());
    assert!(session.agents.contains(&agent1));
    assert!(session.agents.contains(&agent2));
    assert_eq!(session.agents.len(), 2);
}

#[test]
fn test_session_vertex_count() {
    let mut session = SessionBuilder::new(SessionType::General).build();
    assert_eq!(session.vertex_count, 0);
    let v1 = VertexId::from_bytes(b"v1");
    session.update_frontier(v1, &[]);
    assert_eq!(session.vertex_count, 1);
    let v2 = VertexId::from_bytes(b"v2");
    session.update_frontier(v2, &[v1]);
    assert_eq!(session.vertex_count, 2);
}

#[test]
fn test_session_frontier_operations() {
    let mut session = SessionBuilder::new(SessionType::General).build();
    let v1 = VertexId::from_bytes(b"a");
    let v2 = VertexId::from_bytes(b"b");
    let v3 = VertexId::from_bytes(b"c");
    let v4 = VertexId::from_bytes(b"d");
    session.update_frontier(v1, &[]);
    session.update_frontier(v2, &[]);
    session.update_frontier(v3, &[v1]);
    session.update_frontier(v4, &[v2, v3]);
    assert_eq!(session.frontier.len(), 1);
    assert!(session.frontier.contains(&v4));
    assert_eq!(session.vertex_count, 4);
    assert_eq!(session.genesis.len(), 2);
}

#[test]
fn test_session_serialization() {
    let state = SessionState::Resolving {
        started_at: Timestamp::now(),
    };
    let json = serde_json::to_string(&state).unwrap();
    let parsed: SessionState = serde_json::from_str(&json).unwrap();
    assert!(matches!(parsed, SessionState::Resolving { .. }));

    let discard = DiscardReason::LimitExceeded {
        limit: "vertices".to_string(),
    };
    let json = serde_json::to_string(&discard).unwrap();
    let parsed: DiscardReason = serde_json::from_str(&json).unwrap();
    assert!(matches!(parsed, DiscardReason::LimitExceeded { limit } if limit == "vertices"));
}

#[test]
fn test_loam_commit_ref_serialization() {
    let commit_ref = CommitRef {
        spine_id: "spine-42".to_string(),
        entry_hash: [1u8; 32],
        index: 99,
    };
    let json = serde_json::to_string(&commit_ref).unwrap();
    let parsed: LoamCommitRef = serde_json::from_str(&json).unwrap();
    assert_eq!(commit_ref.spine_id, parsed.spine_id);
    assert_eq!(commit_ref.entry_hash, parsed.entry_hash);
    assert_eq!(commit_ref.index, parsed.index);
}
