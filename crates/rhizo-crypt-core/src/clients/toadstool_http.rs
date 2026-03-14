// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2024–2026 ecoPrimals Project

//! ToadStool HTTP Client - Live Integration
//!
//! Provides HTTP client for the ToadStool BYOB server API.
//! This module is only enabled with the `live-clients` feature.
//!
//! Without the feature, the ToadStool client operates in scaffolded mode.
//!
//! ## ToadStool BYOB API
//!
//! - `GET /health` — Service health check
//! - `GET /byob/health` — BYOB API health check
//! - `POST /byob/deploy` — Deploy a biome
//! - `GET /byob/deployments` — List deployments
//! - `GET /byob/deployments/:id` — Get deployment status
//! - `POST /byob/deployments/:id/stop` — Stop deployment
//! - `GET /byob/deployments/:id/usage` — Get resource usage

use crate::constants::{CONNECTION_TIMEOUT, HEALTH_CHECK_TIMEOUT};
use serde::{Deserialize, Serialize};
use tracing::{debug, info};

use crate::error::{Result, RhizoCryptError};
use crate::types::{Did, PayloadRef, Timestamp};
use crate::types_ecosystem::compute::{ComputeEvent, TaskId};

/// HTTP client for ToadStool BYOB API.
#[derive(Clone, Debug)]
pub struct ToadStoolHttpClient {
    client: reqwest::Client,
    base_url: String,
}

/// Error from ToadStool HTTP operations.
#[derive(Debug, thiserror::Error)]
pub enum ToadStoolHttpError {
    /// HTTP request failed.
    #[error("HTTP request failed: {0}")]
    Request(#[from] reqwest::Error),

    /// Invalid response format.
    #[error("Invalid response: {0}")]
    InvalidResponse(String),

    /// Server returned error.
    #[error("Server error {status}: {message}")]
    Server {
        /// HTTP status code.
        status: u16,
        /// Error message from server.
        message: String,
    },
}

/// Health status from ToadStool API.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthStatus {
    /// Service status ("healthy" or other).
    pub status: String,
    /// Service name.
    pub service: Option<String>,
    /// Service version.
    pub version: Option<String>,
    /// Health message.
    pub message: Option<String>,
}

/// BYOB API health response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ByobHealthResponse {
    /// Service status.
    pub status: String,
    /// Health message.
    pub message: String,
}

/// Deployment status from BYOB API.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DeploymentStatus {
    /// Deployment is pending.
    Pending,
    /// Deployment is running.
    Running,
    /// Deployment completed successfully.
    Completed,
    /// Deployment failed.
    Failed,
    /// Deployment was stopped.
    Stopped,
}

/// Deployment response from BYOB API.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeploymentResponse {
    /// Deployment ID (UUID).
    pub deployment_id: String,
    /// Current status.
    pub status: DeploymentStatus,
    /// Biome name.
    pub biome_name: Option<String>,
    /// Creation timestamp.
    pub created_at: Option<String>,
    /// Completion timestamp.
    pub completed_at: Option<String>,
    /// Error message (if failed).
    pub error: Option<String>,
    /// Result data (if completed).
    pub result: Option<serde_json::Value>,
}

/// Resource usage from BYOB API.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceUsage {
    /// CPU usage (0.0 - 1.0).
    pub cpu_usage: f64,
    /// Memory usage in bytes.
    pub memory_bytes: u64,
    /// Network bytes sent.
    pub network_tx_bytes: Option<u64>,
    /// Network bytes received.
    pub network_rx_bytes: Option<u64>,
}

/// Stop deployment response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StopDeploymentResponse {
    /// Deployment ID that was stopped.
    pub deployment_id: String,
    /// Result message.
    pub message: String,
}

impl ToadStoolHttpClient {
    /// Create a new HTTP client for ToadStool BYOB server.
    ///
    /// # Arguments
    ///
    /// * `base_url` - Base URL of the ToadStool BYOB server (e.g., `http://127.0.0.1:8084`)
    ///
    /// # Errors
    ///
    /// Returns error if client creation fails.
    pub fn new(base_url: impl Into<String>) -> std::result::Result<Self, ToadStoolHttpError> {
        let client = reqwest::Client::builder()
            .timeout(CONNECTION_TIMEOUT)
            .connect_timeout(HEALTH_CHECK_TIMEOUT)
            .build()?;

        Ok(Self {
            client,
            base_url: base_url.into(),
        })
    }

    /// Check general health of ToadStool service.
    ///
    /// # Errors
    ///
    /// Returns error if health check fails.
    pub async fn health(&self) -> std::result::Result<HealthStatus, ToadStoolHttpError> {
        let url = format!("{}/health", self.base_url);
        debug!(%url, "Checking ToadStool service health");

        let response = self.client.get(&url).send().await?;

        if response.status().is_success() {
            let health: HealthStatus = response.json().await?;
            info!(status = %health.status, "ToadStool health check passed");
            Ok(health)
        } else {
            let status = response.status().as_u16();
            let message = response.text().await.unwrap_or_default();
            Err(ToadStoolHttpError::Server {
                status,
                message,
            })
        }
    }

    /// Check BYOB API health.
    ///
    /// # Errors
    ///
    /// Returns error if health check fails.
    pub async fn byob_health(&self) -> std::result::Result<ByobHealthResponse, ToadStoolHttpError> {
        let url = format!("{}/byob/health", self.base_url);
        debug!(%url, "Checking ToadStool BYOB API health");

        let response = self.client.get(&url).send().await?;

        if response.status().is_success() {
            let health: ByobHealthResponse = response.json().await?;
            info!(status = %health.status, "BYOB API health check passed");
            Ok(health)
        } else {
            let status = response.status().as_u16();
            let message = response.text().await.unwrap_or_default();
            Err(ToadStoolHttpError::Server {
                status,
                message,
            })
        }
    }

    /// List all deployments.
    ///
    /// # Errors
    ///
    /// Returns error if request fails.
    pub async fn list_deployments(
        &self,
    ) -> std::result::Result<Vec<DeploymentResponse>, ToadStoolHttpError> {
        let url = format!("{}/byob/deployments", self.base_url);
        debug!(%url, "Listing BYOB deployments");

        let response = self.client.get(&url).send().await?;

        if response.status().is_success() {
            Ok(response.json().await?)
        } else {
            let status = response.status().as_u16();
            let message = response.text().await.unwrap_or_default();
            Err(ToadStoolHttpError::Server {
                status,
                message,
            })
        }
    }

    /// Get deployment status.
    ///
    /// # Errors
    ///
    /// Returns error if request fails or deployment not found.
    pub async fn get_deployment(
        &self,
        deployment_id: &str,
    ) -> std::result::Result<DeploymentResponse, ToadStoolHttpError> {
        let url = format!("{}/byob/deployments/{}", self.base_url, deployment_id);
        debug!(%url, "Getting deployment status");

        let response = self.client.get(&url).send().await?;

        if response.status().is_success() {
            Ok(response.json().await?)
        } else {
            let status = response.status().as_u16();
            let message = response.text().await.unwrap_or_default();
            Err(ToadStoolHttpError::Server {
                status,
                message,
            })
        }
    }

    /// Stop a deployment.
    ///
    /// # Errors
    ///
    /// Returns error if request fails.
    pub async fn stop_deployment(
        &self,
        deployment_id: &str,
    ) -> std::result::Result<StopDeploymentResponse, ToadStoolHttpError> {
        let url = format!("{}/byob/deployments/{}/stop", self.base_url, deployment_id);
        debug!(%url, "Stopping deployment");

        let response = self.client.post(&url).send().await?;

        if response.status().is_success() {
            info!(%deployment_id, "Deployment stopped");
            Ok(response.json().await?)
        } else {
            let status = response.status().as_u16();
            let message = response.text().await.unwrap_or_default();
            Err(ToadStoolHttpError::Server {
                status,
                message,
            })
        }
    }

    /// Get resource usage for a deployment.
    ///
    /// # Errors
    ///
    /// Returns error if request fails.
    pub async fn get_resource_usage(
        &self,
        deployment_id: &str,
    ) -> std::result::Result<ResourceUsage, ToadStoolHttpError> {
        let url = format!("{}/byob/deployments/{}/usage", self.base_url, deployment_id);
        debug!(%url, "Getting resource usage");

        let response = self.client.get(&url).send().await?;

        if response.status().is_success() {
            Ok(response.json().await?)
        } else {
            let status = response.status().as_u16();
            let message = response.text().await.unwrap_or_default();
            Err(ToadStoolHttpError::Server {
                status,
                message,
            })
        }
    }

    /// Convert deployment response to compute event.
    ///
    /// The `worker` DID identifies the compute provider discovered at runtime
    /// via capability-based discovery -- never hardcoded.
    #[must_use]
    pub fn deployment_to_event(
        &self,
        deployment: &DeploymentResponse,
        requester: &Did,
        worker: &Did,
    ) -> Option<ComputeEvent> {
        let task_id = parse_deployment_id(&deployment.deployment_id)?;
        let now = Timestamp::now();

        Some(match &deployment.status {
            DeploymentStatus::Pending => ComputeEvent::TaskCreated {
                task_id,
                task_type: "byob_deployment".to_string(),
                requester: requester.clone(),
                created_at: now,
            },
            DeploymentStatus::Running => ComputeEvent::TaskStarted {
                task_id,
                worker: worker.clone(),
                started_at: now,
            },
            DeploymentStatus::Completed => ComputeEvent::TaskCompleted {
                task_id,
                result_ref: PayloadRef::from_bytes(
                    deployment
                        .result
                        .as_ref()
                        .map(ToString::to_string)
                        .unwrap_or_default()
                        .as_bytes(),
                ),
                completed_at: now,
            },
            DeploymentStatus::Failed => ComputeEvent::TaskFailed {
                task_id,
                error: deployment.error.clone().unwrap_or_else(|| "Unknown error".to_string()),
                failed_at: now,
            },
            DeploymentStatus::Stopped => ComputeEvent::TaskCancelled {
                task_id,
                reason: "Stopped by user".to_string(),
                cancelled_at: now,
            },
        })
    }
}

/// Parse deployment ID string to TaskId.
fn parse_deployment_id(id: &str) -> Option<TaskId> {
    // Try to parse as UUID first
    if let Ok(uuid) = uuid::Uuid::parse_str(id) {
        let mut bytes = [0u8; 32];
        bytes[..16].copy_from_slice(uuid.as_bytes());
        return Some(TaskId::new(bytes));
    }

    // Try to parse as hex
    if id.len() >= 16 {
        let mut bytes = [0u8; 32];
        for (i, chunk) in id.as_bytes().chunks(2).enumerate().take(16) {
            if let Ok(byte) = u8::from_str_radix(std::str::from_utf8(chunk).ok()?, 16) {
                bytes[i] = byte;
            } else {
                return None;
            }
        }
        return Some(TaskId::new(bytes));
    }

    None
}

/// Create a ToadStool HTTP client from configuration.
///
/// This is the preferred way to create a live ToadStool client.
///
/// # Errors
///
/// Returns error if connection fails.
pub async fn create_http_client(endpoint: std::net::SocketAddr) -> Result<ToadStoolHttpClient> {
    let base_url = format!("http://{endpoint}");
    let http_client = ToadStoolHttpClient::new(&base_url)
        .map_err(|e| RhizoCryptError::integration(format!("Failed to create HTTP client: {e}")))?;

    // Verify connectivity
    http_client
        .health()
        .await
        .map_err(|e| RhizoCryptError::integration(format!("ToadStool health check failed: {e}")))?;

    info!(%endpoint, "Connected to ToadStool BYOB server via HTTP");
    Ok(http_client)
}

/// Poll for task events using HTTP.
///
/// The `worker` DID should be resolved from capability-based discovery at
/// runtime, identifying the compute provider that owns these deployments.
pub fn poll_events_from_deployments(
    http_client: &ToadStoolHttpClient,
    deployments: &[DeploymentResponse],
    requester: &Did,
    worker: &Did,
) -> Vec<ComputeEvent> {
    deployments
        .iter()
        .filter_map(|d| http_client.deployment_to_event(d, requester, worker))
        .collect()
}

#[cfg(test)]
#[path = "toadstool_http_tests.rs"]
mod tests;
