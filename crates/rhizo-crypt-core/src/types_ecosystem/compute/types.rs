// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2024–2026 ecoPrimals Project

//! Compute Provider Types - Task Events & Configuration
//!
//! Type definitions for compute orchestration capability providers.

use std::borrow::Cow;

use serde::{Deserialize, Serialize};

use crate::types::{Did, PayloadRef, Timestamp};

/// Task identifier for compute provider compute tasks.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TaskId(pub [u8; 32]);

impl TaskId {
    /// Create a new task ID from bytes.
    #[must_use]
    pub const fn new(bytes: [u8; 32]) -> Self {
        Self(bytes)
    }

    /// Create a task ID from a UUID v7.
    #[must_use]
    pub fn now() -> Self {
        let uuid = uuid::Uuid::now_v7();
        let mut bytes = [0u8; 32];
        bytes[..16].copy_from_slice(uuid.as_bytes());
        Self(bytes)
    }

    /// Get the underlying bytes.
    #[must_use]
    pub const fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }
}

impl Default for TaskId {
    fn default() -> Self {
        Self::now()
    }
}

impl std::fmt::Display for TaskId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // Format first 8 bytes as hex without external dependency
        for byte in &self.0[..8] {
            write!(f, "{byte:02x}")?;
        }
        Ok(())
    }
}

/// Compute events from compute provider.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum ComputeEvent {
    /// Task created.
    TaskCreated {
        /// Task identifier.
        task_id: TaskId,
        /// Task type (e.g., "ml-training", "inference").
        task_type: String,
        /// Requester DID.
        requester: Did,
        /// Creation timestamp.
        created_at: Timestamp,
    },
    /// Task started execution.
    TaskStarted {
        /// Task identifier.
        task_id: TaskId,
        /// Worker DID.
        worker: Did,
        /// Start timestamp.
        started_at: Timestamp,
    },
    /// Task progress update.
    TaskProgress {
        /// Task identifier.
        task_id: TaskId,
        /// Progress (0.0 to 1.0).
        progress: f32,
        /// Optional status message.
        message: Option<String>,
        /// Update timestamp.
        updated_at: Timestamp,
    },
    /// Task completed successfully.
    TaskCompleted {
        /// Task identifier.
        task_id: TaskId,
        /// Result payload reference.
        result_ref: PayloadRef,
        /// Completion timestamp.
        completed_at: Timestamp,
    },
    /// Task failed.
    TaskFailed {
        /// Task identifier.
        task_id: TaskId,
        /// Error message.
        error: String,
        /// Failure timestamp.
        failed_at: Timestamp,
    },
    /// Task cancelled.
    TaskCancelled {
        /// Task identifier.
        task_id: TaskId,
        /// Cancellation reason.
        reason: String,
        /// Cancellation timestamp.
        cancelled_at: Timestamp,
    },
}

impl ComputeEvent {
    /// Get the task ID for this event.
    #[must_use]
    pub const fn task_id(&self) -> TaskId {
        match self {
            Self::TaskCreated {
                task_id,
                ..
            }
            | Self::TaskStarted {
                task_id,
                ..
            }
            | Self::TaskProgress {
                task_id,
                ..
            }
            | Self::TaskCompleted {
                task_id,
                ..
            }
            | Self::TaskFailed {
                task_id,
                ..
            }
            | Self::TaskCancelled {
                task_id,
                ..
            } => *task_id,
        }
    }

    /// Get the event type name.
    #[must_use]
    pub const fn event_type(&self) -> &'static str {
        match self {
            Self::TaskCreated {
                ..
            } => "task.created",
            Self::TaskStarted {
                ..
            } => "task.started",
            Self::TaskProgress {
                ..
            } => "task.progress",
            Self::TaskCompleted {
                ..
            } => "task.completed",
            Self::TaskFailed {
                ..
            } => "task.failed",
            Self::TaskCancelled {
                ..
            } => "task.cancelled",
        }
    }

    /// Check if this is a terminal event.
    #[must_use]
    pub const fn is_terminal(&self) -> bool {
        matches!(
            self,
            Self::TaskCompleted { .. } | Self::TaskFailed { .. } | Self::TaskCancelled { .. }
        )
    }
}

/// Configuration for compute provider client.
#[derive(Debug, Clone)]
pub struct ComputeProviderConfig {
    /// compute provider service address (fallback when discovery unavailable).
    pub fallback_address: Option<Cow<'static, str>>,

    /// Connection timeout in milliseconds.
    pub timeout_ms: u64,

    /// Event buffer size for subscriptions.
    pub event_buffer_size: usize,

    /// Retry attempts for failed connections.
    pub max_retries: u8,
}

impl Default for ComputeProviderConfig {
    fn default() -> Self {
        Self {
            fallback_address: None, // No fallback - use discovery
            timeout_ms: crate::constants::DEFAULT_CAPABILITY_TIMEOUT_MS,
            event_buffer_size: 1000,
            max_retries: 3,
        }
    }
}

impl ComputeProviderConfig {
    /// Create config from environment variables.
    ///
    /// Environment variables:
    /// - `COMPUTE_ENDPOINT` or `COMPUTE_ORCHESTRATION_ENDPOINT`: Compute capability endpoint
    /// - `COMPUTE_TIMEOUT_MS`: Connection timeout in milliseconds
    #[must_use]
    pub fn from_env() -> Self {
        use crate::safe_env::CapabilityEnv;
        let mut config = Self::default();

        if let Some(addr) = CapabilityEnv::compute_endpoint() {
            config.fallback_address = Some(Cow::Owned(addr));
        }

        if let Ok(timeout) = std::env::var("COMPUTE_TIMEOUT_MS")
            && let Ok(ms) = timeout.parse()
        {
            config.timeout_ms = ms;
        }

        config
    }

    /// Create config with a specific fallback address (for testing).
    #[must_use]
    pub fn with_fallback(address: impl Into<Cow<'static, str>>) -> Self {
        Self {
            fallback_address: Some(address.into()),
            ..Self::default()
        }
    }
}

/// Client state.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ClientState {
    /// Not connected.
    #[default]
    Disconnected,
    /// Connection in progress.
    Connecting,
    /// Connected and ready.
    Connected,
    /// Disconnected due to error.
    Error,
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;

    #[test]
    fn test_config_default() {
        let config = ComputeProviderConfig::default();
        assert!(config.fallback_address.is_none());
        assert_eq!(config.timeout_ms, 5000);
    }

    #[test]
    fn test_config_with_fallback() {
        let config = ComputeProviderConfig::with_fallback("127.0.0.1:9800");
        assert_eq!(config.fallback_address.as_deref(), Some("127.0.0.1:9800"));
    }

    #[test]
    fn test_task_id_display() {
        let id = TaskId::new([
            0xDE, 0xAD, 0xBE, 0xEF, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0,
        ]);
        assert_eq!(format!("{id}"), "deadbeef00000000");
    }

    #[test]
    fn test_compute_event_type() {
        let event = ComputeEvent::TaskCreated {
            task_id: TaskId::now(),
            task_type: "test".to_string(),
            requester: Did::new("did:key:test"),
            created_at: Timestamp::now(),
        };
        assert_eq!(event.event_type(), "task.created");
        assert!(!event.is_terminal());

        let completed = ComputeEvent::TaskCompleted {
            task_id: TaskId::now(),
            result_ref: PayloadRef::from_bytes(b"test-result"),
            completed_at: Timestamp::now(),
        };
        assert!(completed.is_terminal());
    }

    #[test]
    fn test_task_id_new() {
        let bytes = [1u8; 32];
        let id = TaskId::new(bytes);
        assert_eq!(id.as_bytes(), &bytes);
    }

    #[test]
    fn test_task_id_now() {
        let id1 = TaskId::now();
        let id2 = TaskId::now();
        // UUIDs should be different
        assert_ne!(id1, id2);
    }

    #[test]
    fn test_task_id_default() {
        let id = TaskId::default();
        // Should not be all zeros
        assert_ne!(id.as_bytes(), &[0u8; 32]);
    }

    #[test]
    fn test_task_id_equality() {
        let bytes = [42u8; 32];
        let id1 = TaskId::new(bytes);
        let id2 = TaskId::new(bytes);
        assert_eq!(id1, id2);

        let id3 = TaskId::now();
        assert_ne!(id1, id3);
    }

    #[test]
    fn test_compute_event_task_started() {
        let event = ComputeEvent::TaskStarted {
            task_id: TaskId::now(),
            worker: Did::new("did:key:worker"),
            started_at: Timestamp::now(),
        };
        assert_eq!(event.event_type(), "task.started");
        assert!(!event.is_terminal());
    }

    #[test]
    fn test_compute_event_task_progress() {
        let event = ComputeEvent::TaskProgress {
            task_id: TaskId::now(),
            progress: 0.5,
            message: Some("Processing...".to_string()),
            updated_at: Timestamp::now(),
        };
        assert_eq!(event.event_type(), "task.progress");
        assert!(!event.is_terminal());
    }

    #[test]
    fn test_compute_event_task_failed() {
        let event = ComputeEvent::TaskFailed {
            task_id: TaskId::now(),
            error: "Out of memory".to_string(),
            failed_at: Timestamp::now(),
        };
        assert_eq!(event.event_type(), "task.failed");
        assert!(event.is_terminal());
    }

    #[test]
    fn test_compute_event_task_cancelled() {
        let event = ComputeEvent::TaskCancelled {
            task_id: TaskId::now(),
            reason: "Cancelled by admin".to_string(),
            cancelled_at: Timestamp::now(),
        };
        assert_eq!(event.event_type(), "task.cancelled");
        assert!(event.is_terminal());
    }

    #[test]
    fn test_config_from_env() {
        let config = ComputeProviderConfig::from_env();
        // Default values (may be overridden by env vars)
        assert!(config.timeout_ms > 0);
        assert!(config.event_buffer_size > 0);
    }

    #[test]
    fn test_config_with_custom_values() {
        let mut config = ComputeProviderConfig::default();
        config.timeout_ms = 10000;
        config.event_buffer_size = 200;

        assert_eq!(config.timeout_ms, 10000);
        assert_eq!(config.event_buffer_size, 200);
    }

    #[test]
    fn test_client_state_default() {
        let state = ClientState::default();
        assert_eq!(state, ClientState::Disconnected);
    }

    #[test]
    fn test_client_state_transitions() {
        let state1 = ClientState::Disconnected;
        let state2 = ClientState::Connecting;
        let state3 = ClientState::Connected;
        let state4 = ClientState::Error;

        assert_ne!(state1, state2);
        assert_ne!(state2, state3);
        assert_ne!(state3, state4);
    }

    #[test]
    fn test_compute_event_all_types() {
        let task_id = TaskId::now();
        let timestamp = Timestamp::now();
        let did = Did::new("did:key:test");

        // TaskCreated
        let event = ComputeEvent::TaskCreated {
            task_id,
            task_type: "ml-training".to_string(),
            requester: did.clone(),
            created_at: timestamp,
        };
        assert_eq!(event.event_type(), "task.created");
        assert!(!event.is_terminal());

        // TaskStarted
        let event = ComputeEvent::TaskStarted {
            task_id,
            worker: did,
            started_at: timestamp,
        };
        assert_eq!(event.event_type(), "task.started");
        assert!(!event.is_terminal());

        // TaskProgress with message
        let event = ComputeEvent::TaskProgress {
            task_id,
            progress: 0.75,
            message: Some("Almost done".to_string()),
            updated_at: timestamp,
        };
        assert_eq!(event.event_type(), "task.progress");
        assert!(!event.is_terminal());

        // TaskProgress without message
        let event = ComputeEvent::TaskProgress {
            task_id,
            progress: 0.5,
            message: None,
            updated_at: timestamp,
        };
        assert!(!event.is_terminal());

        // TaskCompleted
        let event = ComputeEvent::TaskCompleted {
            task_id,
            result_ref: PayloadRef::from_bytes(b"result"),
            completed_at: timestamp,
        };
        assert_eq!(event.event_type(), "task.completed");
        assert!(event.is_terminal());

        // TaskFailed
        let event = ComputeEvent::TaskFailed {
            task_id,
            error: "Test error".to_string(),
            failed_at: timestamp,
        };
        assert_eq!(event.event_type(), "task.failed");
        assert!(event.is_terminal());

        // TaskCancelled
        let event = ComputeEvent::TaskCancelled {
            task_id,
            reason: "User requested".to_string(),
            cancelled_at: timestamp,
        };
        assert_eq!(event.event_type(), "task.cancelled");
        assert!(event.is_terminal());
    }

    #[test]
    fn test_task_id_serialization() {
        let id = TaskId::now();
        let serialized = serde_json::to_string(&id).unwrap();
        let deserialized: TaskId = serde_json::from_str(&serialized).unwrap();
        assert_eq!(id, deserialized);
    }

    #[test]
    fn test_compute_event_serialization() {
        let event = ComputeEvent::TaskCreated {
            task_id: TaskId::now(),
            task_type: "test".to_string(),
            requester: Did::new("did:key:test"),
            created_at: Timestamp::now(),
        };

        let serialized = serde_json::to_string(&event).unwrap();
        let deserialized: ComputeEvent = serde_json::from_str(&serialized).unwrap();
        assert_eq!(deserialized.event_type(), "task.created");
    }
}
