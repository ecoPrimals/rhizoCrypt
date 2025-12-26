//! # rhizo-crypt-rpc
//!
//! Pure Rust RPC layer for rhizoCrypt using tarpc.
//!
//! This crate provides compile-time type-safe RPC for rhizoCrypt operations,
//! following the ecoPrimals pattern of leaning into the Rust compiler rather
//! than external code generation (protobuf/gRPC).
//!
//! ## Design Philosophy
//!
//! - **Pure Rust**: No `.proto` files, no code generation
//! - **Compile-time safety**: Trait-based RPC, all types checked at compile time
//! - **tarpc**: High-performance async RPC built on Rust's type system
//! - **Ecosystem alignment**: Matches songBird's RPC patterns
//!
//! ## Usage
//!
//! ```rust,ignore
//! use rhizo_crypt_rpc::{RhizoCryptRpc, RhizoCryptRpcClient};
//!
//! // Connect to a rhizoCrypt service
//! let client = RhizoCryptRpcClient::connect("127.0.0.1:9400").await?;
//!
//! // Create a session (compile-time type checked)
//! let session_id = client.create_session(session_config).await?;
//!
//! // Append events
//! let vertex_id = client.append_event(session_id, event).await?;
//! ```

#![forbid(unsafe_code)]
#![warn(missing_docs)]

mod client;
mod error;
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
