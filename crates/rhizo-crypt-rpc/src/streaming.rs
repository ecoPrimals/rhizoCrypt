// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2024–2026 ecoPrimals Project

//! NDJSON streaming support for pipeline coordination.
//!
//! Absorbed from biomeOS v2.43 Pipeline coordination pattern and the
//! `SPRING_AS_NICHE_DEPLOYMENT_STANDARD` streaming specification.
//!
//! Springs that produce or consume data streams use NDJSON (newline-delimited
//! JSON) as the transport format. Each line is an independent JSON object
//! representing a [`StreamItem`].
//!
//! ## Protocol
//!
//! ```text
//! Client sends:  {"jsonrpc":"2.0","method":"dag.event.append_batch","params":{...},"id":1}\n
//! Server writes: {"type":"Data","payload":{"vertex_id":"..."}}\n
//! Server writes: {"type":"Data","payload":{"vertex_id":"..."}}\n
//! Server writes: {"type":"End"}\n
//! ```
//!
//! For primals that return a single result, no changes are needed — the
//! standard JSON-RPC response is automatically compatible.

use rhizo_crypt_core::VertexId;
use serde::{Deserialize, Serialize};

/// A single item in an NDJSON stream.
///
/// Used by biomeOS Pipeline coordination graphs to wire bounded `mpsc`
/// channels between springs. Items flow through as each node produces them.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum StreamItem {
    /// A data payload in the stream.
    Data {
        /// The payload content (schema depends on the method).
        payload: serde_json::Value,
    },
    /// Stream progress indicator.
    Progress {
        /// Items processed so far.
        processed: u64,
        /// Total items (if known).
        total: Option<u64>,
    },
    /// End of stream marker.
    End,
    /// Stream-level error (non-fatal; stream may continue).
    Error {
        /// Error message.
        message: String,
        /// Whether the stream can continue after this error.
        recoverable: bool,
    },
}

/// Result of a streaming `event.append_batch` operation.
///
/// Each successfully appended vertex is emitted as a [`StreamItem::Data`]
/// with the vertex ID, allowing the consumer to process results as they
/// arrive rather than waiting for the entire batch.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct StreamingAppendResult {
    /// The vertex ID that was appended.
    pub vertex_id: VertexId,
    /// Zero-based index within the batch.
    pub batch_index: usize,
}

impl StreamItem {
    /// Create a data item from a serializable value.
    ///
    /// # Errors
    ///
    /// Returns an error if the value cannot be serialized to JSON.
    pub fn data(value: &impl Serialize) -> Result<Self, serde_json::Error> {
        Ok(Self::Data {
            payload: serde_json::to_value(value)?,
        })
    }

    /// Create an end-of-stream marker.
    #[must_use]
    pub const fn end() -> Self {
        Self::End
    }

    /// Create a progress item.
    #[must_use]
    pub const fn progress(processed: u64, total: Option<u64>) -> Self {
        Self::Progress {
            processed,
            total,
        }
    }

    /// Create a recoverable error item.
    #[must_use]
    pub fn error(message: impl Into<String>) -> Self {
        Self::Error {
            message: message.into(),
            recoverable: true,
        }
    }

    /// Create a fatal error item.
    #[must_use]
    pub fn fatal(message: impl Into<String>) -> Self {
        Self::Error {
            message: message.into(),
            recoverable: false,
        }
    }

    /// Serialize this item as a single NDJSON line (with trailing newline).
    ///
    /// # Errors
    ///
    /// Returns an error if JSON serialization fails.
    pub fn to_ndjson_line(&self) -> Result<String, serde_json::Error> {
        let mut line = serde_json::to_string(self)?;
        line.push('\n');
        Ok(line)
    }

    /// Returns `true` if this is the end-of-stream marker.
    #[must_use]
    pub const fn is_end(&self) -> bool {
        matches!(self, Self::End)
    }

    /// Returns `true` if this is a data item.
    #[must_use]
    pub const fn is_data(&self) -> bool {
        matches!(self, Self::Data { .. })
    }
}

/// Parse a single NDJSON line into a [`StreamItem`].
///
/// # Errors
///
/// Returns an error if the line is not valid JSON or doesn't match
/// the [`StreamItem`] schema.
pub fn parse_ndjson_line(line: &str) -> Result<StreamItem, serde_json::Error> {
    serde_json::from_str(line.trim())
}

#[cfg(test)]
#[expect(clippy::unwrap_used, reason = "test code")]
mod tests {
    use super::*;

    #[test]
    fn stream_item_data_roundtrip() {
        let item = StreamItem::data(&serde_json::json!({"vertex_id": "abc123"})).unwrap();
        let line = item.to_ndjson_line().unwrap();
        assert!(line.ends_with('\n'));
        let parsed = parse_ndjson_line(&line).unwrap();
        assert!(parsed.is_data());
    }

    #[test]
    fn stream_item_end_roundtrip() {
        let item = StreamItem::end();
        let line = item.to_ndjson_line().unwrap();
        let parsed = parse_ndjson_line(&line).unwrap();
        assert!(parsed.is_end());
    }

    #[test]
    fn stream_item_progress_roundtrip() {
        let item = StreamItem::progress(42, Some(100));
        let line = item.to_ndjson_line().unwrap();
        let parsed = parse_ndjson_line(&line).unwrap();
        match parsed {
            StreamItem::Progress {
                processed,
                total,
            } => {
                assert_eq!(processed, 42);
                assert_eq!(total, Some(100));
            }
            _ => panic!("expected Progress"),
        }
    }

    #[test]
    fn stream_item_error_roundtrip() {
        let item = StreamItem::error("vertex hash mismatch");
        let line = item.to_ndjson_line().unwrap();
        let parsed = parse_ndjson_line(&line).unwrap();
        match parsed {
            StreamItem::Error {
                message,
                recoverable,
            } => {
                assert_eq!(message, "vertex hash mismatch");
                assert!(recoverable);
            }
            _ => panic!("expected Error"),
        }
    }

    #[test]
    fn stream_item_fatal_error() {
        let item = StreamItem::fatal("session expired");
        match &item {
            StreamItem::Error {
                recoverable,
                ..
            } => assert!(!recoverable),
            _ => panic!("expected Error"),
        }
    }

    #[test]
    fn streaming_append_result_serialization() {
        let result = StreamingAppendResult {
            vertex_id: VertexId::from_bytes(b"test-vertex-content-for-hash"),
            batch_index: 5,
        };
        let item = StreamItem::data(&result).unwrap();
        assert!(item.is_data());
        assert!(!item.is_end());
    }

    #[test]
    fn parse_ndjson_line_invalid_json() {
        let result = parse_ndjson_line("not json");
        assert!(result.is_err());
    }

    #[test]
    fn parse_ndjson_line_trims_whitespace() {
        let item = StreamItem::end();
        let line = format!("  {}  \n", serde_json::to_string(&item).unwrap());
        let parsed = parse_ndjson_line(&line).unwrap();
        assert!(parsed.is_end());
    }

    #[test]
    fn ndjson_multiline_stream() {
        let items = [
            StreamItem::data(&serde_json::json!({"id": 1})).unwrap(),
            StreamItem::data(&serde_json::json!({"id": 2})).unwrap(),
            StreamItem::progress(2, Some(3)),
            StreamItem::data(&serde_json::json!({"id": 3})).unwrap(),
            StreamItem::end(),
        ];

        let stream: String = items.iter().map(|i| i.to_ndjson_line().unwrap()).collect();
        let parsed: Vec<StreamItem> =
            stream.lines().map(|l| parse_ndjson_line(l).unwrap()).collect();
        assert_eq!(parsed.len(), 5);
        assert!(parsed[0].is_data());
        assert!(parsed[4].is_end());
    }
}
