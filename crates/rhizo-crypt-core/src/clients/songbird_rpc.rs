//! Songbird tarpc RPC types and client.
//!
//! This module contains the tarpc service definition and related types
//! for connecting to the Songbird orchestrator.
//!
//! ## Feature Gate
//!
//! This module is only compiled when the `live-clients` feature is enabled.
//! Without the feature, the Songbird client operates in scaffolded mode.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ============================================================================
// tarpc Service Definition
// ============================================================================

/// tarpc service trait for Songbird discovery operations.
///
/// This trait mirrors the Songbird orchestrator's RPC interface.
#[tarpc::service]
pub trait SongbirdRpc {
    /// Discover services by capability.
    async fn discover(capability: String) -> Vec<RpcServiceInfo>;

    /// Discover all available services.
    async fn discover_all() -> Vec<RpcServiceInfo>;

    /// Register a service with the mesh.
    async fn register(registration: RpcServiceRegistration) -> RpcRegistrationResult;

    /// Unregister a service from the mesh.
    async fn unregister(service_id: String) -> RpcRegistrationResult;

    /// Get health status of the orchestrator.
    async fn health() -> RpcHealthStatus;

    /// Get version information.
    async fn version() -> RpcVersionInfo;
}

// ============================================================================
// RPC Types
// ============================================================================

/// Service info from tarpc discovery.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RpcServiceInfo {
    /// Service identifier.
    pub id: String,
    /// Service capability.
    pub capability: String,
    /// Service endpoint address.
    pub endpoint: String,
    /// Service status.
    pub status: String,
    /// Optional metadata.
    pub metadata: Option<serde_json::Value>,
}

/// Service registration for tarpc.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RpcServiceRegistration {
    /// Service ID.
    pub service_id: String,
    /// Service name.
    pub service_name: String,
    /// Primary capability.
    pub capability: String,
    /// Endpoint address.
    pub endpoint: String,
    /// Additional metadata.
    #[serde(default)]
    pub metadata: HashMap<String, String>,
}

/// Registration result from tarpc.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RpcRegistrationResult {
    /// Whether registration succeeded.
    pub success: bool,
    /// Result message.
    pub message: String,
}

/// Health status from tarpc.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RpcHealthStatus {
    /// Overall status.
    pub status: String,
    /// Orchestrator version.
    pub version: String,
    /// Uptime in seconds.
    pub uptime_seconds: u64,
    /// Number of registered services.
    pub services_count: usize,
}

/// Version info from tarpc.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RpcVersionInfo {
    /// Version string.
    pub version: String,
    /// Protocol version.
    pub protocol: String,
    /// Supported capabilities.
    pub capabilities: Vec<String>,
}
