// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2024–2026 ecoPrimals Project

//! Wire DTOs for cross-gate mesh trust events.
//!
//! These types model the JSON-RPC payloads that bearDog w135+ emits
//! when trust establishment events occur. rhizoCrypt deserializes
//! these into [`MeshTrustEvent`] and maps them to [`EventType`]
//! variants for DAG recording.

use serde::{Deserialize, Serialize};

use crate::event::{EventType, MeshLeaveReason};

/// A trust event received from a cross-gate auth provider (bearDog).
///
/// Deserialized from JSON-RPC notification payloads on the signing
/// endpoint. Each variant maps 1:1 to an [`EventType`] mesh variant.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct MeshTrustEvent {
    /// Event kind discriminator.
    pub kind: MeshTrustEventKind,
    /// Gate that originated the event.
    pub source_gate: String,
    /// Unix timestamp (seconds since epoch) when the event occurred.
    pub timestamp: u64,
}

/// Discriminated event kinds from bearDog's auth subsystem.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", content = "payload")]
pub enum MeshTrustEventKind {
    /// A trusted issuer was registered (bearDog `auth.trust_issuer`).
    TrustIssuerRegistered {
        /// Ed25519 public key fingerprint (hex-encoded).
        issuer_fingerprint: String,
    },

    /// An Ed25519 key exchange completed between gates.
    KeyExchangeCompleted {
        /// Remote gate identifier.
        remote_gate: String,
        /// Key exchange method.
        method: String,
    },

    /// A primal family enrolled in the mesh.
    FamilyEnrollment {
        /// Family identifier.
        family_id: String,
        /// Number of primals at enrollment.
        primal_count: u32,
    },

    /// A gate joined the mesh.
    MeshJoin {
        /// Mesh network identifier.
        mesh_id: String,
    },

    /// A gate left the mesh.
    MeshLeave {
        /// Mesh network identifier.
        mesh_id: String,
        /// Reason for leaving.
        reason: MeshLeaveReason,
    },
}

impl MeshTrustEvent {
    /// Map this wire event to an [`EventType`] for DAG recording without consuming it.
    #[must_use]
    pub fn to_event_type(&self) -> EventType {
        match &self.kind {
            MeshTrustEventKind::TrustIssuerRegistered {
                issuer_fingerprint,
            } => EventType::TrustIssuerRegistered {
                issuer_fingerprint: issuer_fingerprint.clone(),
                registering_gate: self.source_gate.clone(),
            },
            MeshTrustEventKind::KeyExchangeCompleted {
                remote_gate,
                method,
            } => EventType::KeyExchangeCompleted {
                local_gate: self.source_gate.clone(),
                remote_gate: remote_gate.clone(),
                method: method.clone(),
            },
            MeshTrustEventKind::FamilyEnrollment {
                family_id,
                primal_count,
            } => EventType::FamilyEnrollment {
                family_id: family_id.clone(),
                gate: self.source_gate.clone(),
                primal_count: *primal_count,
            },
            MeshTrustEventKind::MeshJoin {
                mesh_id,
            } => EventType::MeshJoin {
                gate: self.source_gate.clone(),
                mesh_id: mesh_id.clone(),
            },
            MeshTrustEventKind::MeshLeave {
                mesh_id,
                reason,
            } => EventType::MeshLeave {
                gate: self.source_gate.clone(),
                mesh_id: mesh_id.clone(),
                reason: reason.clone(),
            },
        }
    }

    /// Convert this wire event into an [`EventType`] for DAG recording.
    #[must_use]
    pub fn into_event_type(self) -> EventType {
        match self.kind {
            MeshTrustEventKind::TrustIssuerRegistered {
                issuer_fingerprint,
            } => EventType::TrustIssuerRegistered {
                issuer_fingerprint,
                registering_gate: self.source_gate,
            },
            MeshTrustEventKind::KeyExchangeCompleted {
                remote_gate,
                method,
            } => EventType::KeyExchangeCompleted {
                local_gate: self.source_gate,
                remote_gate,
                method,
            },
            MeshTrustEventKind::FamilyEnrollment {
                family_id,
                primal_count,
            } => EventType::FamilyEnrollment {
                family_id,
                gate: self.source_gate,
                primal_count,
            },
            MeshTrustEventKind::MeshJoin {
                mesh_id,
            } => EventType::MeshJoin {
                gate: self.source_gate,
                mesh_id,
            },
            MeshTrustEventKind::MeshLeave {
                mesh_id,
                reason,
            } => EventType::MeshLeave {
                gate: self.source_gate,
                mesh_id,
                reason,
            },
        }
    }
}

#[cfg(test)]
#[expect(clippy::unwrap_used, reason = "test code")]
mod tests {
    use super::*;

    #[test]
    fn test_trust_issuer_event_roundtrip() {
        let event = MeshTrustEvent {
            kind: MeshTrustEventKind::TrustIssuerRegistered {
                issuer_fingerprint: "a1b2c3d4".into(),
            },
            source_gate: "eastGate".into(),
            timestamp: 1_717_444_800,
        };
        let json = serde_json::to_string(&event).unwrap();
        let parsed: MeshTrustEvent = serde_json::from_str(&json).unwrap();
        assert_eq!(event, parsed);
    }

    #[test]
    fn test_trust_issuer_into_event_type() {
        let event = MeshTrustEvent {
            kind: MeshTrustEventKind::TrustIssuerRegistered {
                issuer_fingerprint: "deadbeef".into(),
            },
            source_gate: "southGate".into(),
            timestamp: 0,
        };
        let et = event.into_event_type();
        assert_eq!(
            et,
            EventType::TrustIssuerRegistered {
                issuer_fingerprint: "deadbeef".into(),
                registering_gate: "southGate".into(),
            }
        );
    }

    #[test]
    fn test_key_exchange_into_event_type() {
        let event = MeshTrustEvent {
            kind: MeshTrustEventKind::KeyExchangeCompleted {
                remote_gate: "ironGate".into(),
                method: "ed25519_dh".into(),
            },
            source_gate: "strandGate".into(),
            timestamp: 0,
        };
        let et = event.into_event_type();
        assert_eq!(
            et,
            EventType::KeyExchangeCompleted {
                local_gate: "strandGate".into(),
                remote_gate: "ironGate".into(),
                method: "ed25519_dh".into(),
            }
        );
    }

    #[test]
    fn test_family_enrollment_into_event_type() {
        let event = MeshTrustEvent {
            kind: MeshTrustEventKind::FamilyEnrollment {
                family_id: "ecoPrimal".into(),
                primal_count: 5,
            },
            source_gate: "biomeGate".into(),
            timestamp: 0,
        };
        let et = event.into_event_type();
        assert_eq!(
            et,
            EventType::FamilyEnrollment {
                family_id: "ecoPrimal".into(),
                gate: "biomeGate".into(),
                primal_count: 5,
            }
        );
    }

    #[test]
    fn test_mesh_join_leave_into_event_type() {
        let join = MeshTrustEvent {
            kind: MeshTrustEventKind::MeshJoin {
                mesh_id: "glacial-v1".into(),
            },
            source_gate: "westGate".into(),
            timestamp: 0,
        };
        assert_eq!(
            join.into_event_type(),
            EventType::MeshJoin {
                gate: "westGate".into(),
                mesh_id: "glacial-v1".into(),
            }
        );

        let leave = MeshTrustEvent {
            kind: MeshTrustEventKind::MeshLeave {
                mesh_id: "glacial-v1".into(),
                reason: MeshLeaveReason::TrustRevoked,
            },
            source_gate: "westGate".into(),
            timestamp: 0,
        };
        assert_eq!(
            leave.into_event_type(),
            EventType::MeshLeave {
                gate: "westGate".into(),
                mesh_id: "glacial-v1".into(),
                reason: MeshLeaveReason::TrustRevoked,
            }
        );
    }

    #[test]
    fn test_all_event_kinds_serialization() {
        let kinds = vec![
            MeshTrustEventKind::TrustIssuerRegistered {
                issuer_fingerprint: "ff".into(),
            },
            MeshTrustEventKind::KeyExchangeCompleted {
                remote_gate: "g".into(),
                method: "x25519".into(),
            },
            MeshTrustEventKind::FamilyEnrollment {
                family_id: "f".into(),
                primal_count: 1,
            },
            MeshTrustEventKind::MeshJoin {
                mesh_id: "m".into(),
            },
            MeshTrustEventKind::MeshLeave {
                mesh_id: "m".into(),
                reason: MeshLeaveReason::Graceful,
            },
        ];
        for kind in kinds {
            let event = MeshTrustEvent {
                kind: kind.clone(),
                source_gate: "test".into(),
                timestamp: 0,
            };
            let json = serde_json::to_string(&event).unwrap();
            let parsed: MeshTrustEvent = serde_json::from_str(&json).unwrap();
            assert_eq!(event, parsed, "roundtrip failed for {kind:?}");
        }
    }
}
