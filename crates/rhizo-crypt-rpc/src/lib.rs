// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2024–2026 ecoPrimals Project

//! # rhizo-crypt-rpc
//!
//! Pure Rust RPC layer for `rhizoCrypt` using `tarpc`.
//!
//! This crate provides compile-time type-safe RPC for `rhizoCrypt` operations,
//! following the ecoPrimals pattern of leaning into the Rust compiler rather
//! than external code generation (protobuf/gRPC).
//!
//! ## Design Philosophy
//!
//! - **Pure Rust**: No `.proto` files, no code generation
//! - **Compile-time safety**: Trait-based RPC, all types checked at compile time
//! - **`tarpc`**: High-performance async RPC built on Rust's type system
//! - **Ecosystem alignment**: Matches `songBird`'s RPC patterns
//!
//! ## Usage
//!
//! ```no_run
//! # use rhizo_crypt_rpc::{RpcClient, CreateSessionRequest, AppendEventRequest};
//! # use rhizo_crypt_core::{SessionType, EventType};
//! # use std::net::SocketAddr;
//! # tokio::runtime::Runtime::new().unwrap().block_on(async {
//! // Connect to a `rhizoCrypt` service
//! let addr: SocketAddr = "127.0.0.1:9400".parse().unwrap();
//! let client = RpcClient::connect(addr).await?;
//!
//! // Create a session (compile-time type checked)
//! let session_config = CreateSessionRequest {
//!     session_type: SessionType::General,
//!     description: None,
//!     parent_session: None,
//!     max_vertices: None,
//!     ttl_seconds: None,
//! };
//! let session_id = client.create_session(session_config).await?;
//!
//! // Append events
//! let event = AppendEventRequest {
//!     session_id,
//!     event_type: EventType::SessionStart,
//!     agent: None,
//!     parents: vec![],
//!     metadata: vec![],
//!     payload_ref: None,
//! };
//! let _vertex_id = client.append_event(event).await?;
//! # Ok::<(), rhizo_crypt_rpc::RpcError>(())
//! # });
//! ```

#![forbid(unsafe_code)]
#![warn(missing_docs)]

mod client;
mod error;
pub mod jsonrpc;
mod metrics;
mod rate_limit;
pub mod server;
mod service;

pub use client::RpcClient;
pub use error::{RpcError, RpcResult};
pub use metrics::{
    ErrorType, HistogramSnapshot, MetricsCollector, RequestTimer, RpcMethod, SharedMetrics,
};
pub use rate_limit::{OperationType, RateLimitConfig, RateLimitExceeded, RateLimiter};
pub use server::RpcServer;
pub use service::{
    AppendEventRequest, CheckoutSliceRequest, CreateSessionRequest, HealthStatus, QueryRequest,
    RhizoCryptRpc, RhizoCryptRpcServer, ServiceMetrics, SessionInfo,
};

// Re-export core types for convenience
pub use rhizo_crypt_core::{
    DehydrationSummary, Did, EventType, MerkleProof, MerkleRoot, Session, SessionId, SessionState,
    SessionType, SliceId, SliceMode, Timestamp, Vertex, VertexId,
};
