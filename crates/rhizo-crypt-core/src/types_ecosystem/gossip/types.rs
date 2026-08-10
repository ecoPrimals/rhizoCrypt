// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2024–2026 ecoPrimals Project

//! Gossip event types for swarmVine mesh injection.
//!
//! These types model the events rhizoCrypt announces to the gossip mesh
//! via `gossip.spread`. Each variant represents a lifecycle event that
//! cross-gate consumers might care about.

use serde::{Deserialize, Serialize};

/// An event rhizoCrypt injects into the gossip mesh.
///
/// Sent via `gossip.spread` JSON-RPC to any `gossip:relay` provider
/// (swarmVine in the deploy graph). Events are non-durable fire-and-forget
/// announcements — the gossip layer handles propagation and TTL.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind")]
pub enum GossipEvent {
    /// A session was dehydrated (committed to permanent storage).
    ///
    /// Most important gossip event — tells cross-gate consumers that
    /// permanent data is available for federation or slice checkout.
    SessionDehydrated {
        /// The dehydrated session's ID.
        session_id: String,
        /// Hex-encoded BLAKE3 Merkle root of the committed session.
        merkle_root: String,
        /// Number of vertices in the committed session.
        vertex_count: u64,
    },

    /// Multiple sessions were dehydrated in a single batch.
    BatchDehydrated {
        /// Number of sessions committed.
        session_count: u32,
        /// Individual session IDs (capped to first 64 for wire efficiency).
        session_ids: Vec<String>,
    },

    /// DAG vertices were federated from/to a remote gate.
    Federated {
        /// The session receiving federated vertices.
        session_id: String,
        /// Number of vertices imported.
        imported_count: u32,
        /// Source gate identifier (if known).
        source_gate: Option<String>,
    },
}

impl GossipEvent {
    /// Wire-level event kind string for logging and routing.
    #[must_use]
    pub const fn kind_str(&self) -> &'static str {
        match self {
            Self::SessionDehydrated {
                ..
            } => "session.dehydrated",
            Self::BatchDehydrated {
                ..
            } => "batch.dehydrated",
            Self::Federated {
                ..
            } => "federated",
        }
    }
}

#[cfg(test)]
#[expect(clippy::unwrap_used, reason = "test code")]
mod tests {
    use super::*;

    #[test]
    fn test_gossip_event_session_dehydrated_roundtrip() {
        let event = GossipEvent::SessionDehydrated {
            session_id: "sess-001".into(),
            merkle_root: "abcdef1234567890".into(),
            vertex_count: 42,
        };
        let json = serde_json::to_string(&event).unwrap();
        let parsed: GossipEvent = serde_json::from_str(&json).unwrap();
        assert_eq!(event, parsed);
    }

    #[test]
    fn test_gossip_event_batch_dehydrated_roundtrip() {
        let event = GossipEvent::BatchDehydrated {
            session_count: 3,
            session_ids: vec!["s1".into(), "s2".into(), "s3".into()],
        };
        let json = serde_json::to_string(&event).unwrap();
        let parsed: GossipEvent = serde_json::from_str(&json).unwrap();
        assert_eq!(event, parsed);
    }

    #[test]
    fn test_gossip_event_federated_roundtrip() {
        let event = GossipEvent::Federated {
            session_id: "sess-fed".into(),
            imported_count: 10,
            source_gate: Some("westGate".into()),
        };
        let json = serde_json::to_string(&event).unwrap();
        let parsed: GossipEvent = serde_json::from_str(&json).unwrap();
        assert_eq!(event, parsed);
    }

    #[test]
    fn test_gossip_event_federated_no_source_gate() {
        let event = GossipEvent::Federated {
            session_id: "sess-fed".into(),
            imported_count: 5,
            source_gate: None,
        };
        let json = serde_json::to_string(&event).unwrap();
        assert!(!json.contains("null") || json.contains("\"source_gate\":null"));
        let parsed: GossipEvent = serde_json::from_str(&json).unwrap();
        assert_eq!(event, parsed);
    }

    #[test]
    fn test_gossip_event_kind_str() {
        assert_eq!(
            GossipEvent::SessionDehydrated {
                session_id: String::new(),
                merkle_root: String::new(),
                vertex_count: 0,
            }
            .kind_str(),
            "session.dehydrated"
        );
        assert_eq!(
            GossipEvent::BatchDehydrated {
                session_count: 0,
                session_ids: vec![],
            }
            .kind_str(),
            "batch.dehydrated"
        );
        assert_eq!(
            GossipEvent::Federated {
                session_id: String::new(),
                imported_count: 0,
                source_gate: None,
            }
            .kind_str(),
            "federated"
        );
    }

    #[test]
    fn test_gossip_event_tagged_serialization() {
        let event = GossipEvent::SessionDehydrated {
            session_id: "s".into(),
            merkle_root: "r".into(),
            vertex_count: 1,
        };
        let json = serde_json::to_string(&event).unwrap();
        assert!(json.contains(r#""kind":"SessionDehydrated"#));
    }
}
