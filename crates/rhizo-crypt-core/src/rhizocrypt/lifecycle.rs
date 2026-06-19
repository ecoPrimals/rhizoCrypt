// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2024–2026 ecoPrimals Project

//! `PrimalLifecycle` and `PrimalHealth` trait implementations for `RhizoCrypt`.
//!
//! Handles startup (DAG store initialization, payload store, provenance
//! notifier connection, mesh event listener connection), shutdown (store
//! teardown, index cleanup), and health reporting.

use crate::config::StorageBackend;
use crate::primal::{
    HealthReport, HealthStatus, PrimalError, PrimalHealth, PrimalLifecycle, PrimalState,
};
use crate::store::{DagBackend, InMemoryDagStore, InMemoryPayloadStore};

use std::sync::Arc;
use std::time::Instant;

use super::RhizoCrypt;

impl PrimalLifecycle for RhizoCrypt {
    fn state(&self) -> PrimalState {
        self.state
    }

    async fn start(&mut self) -> std::result::Result<(), PrimalError> {
        if self.state != PrimalState::Created && self.state != PrimalState::Stopped {
            return Err(PrimalError::InvalidTransition {
                from: self.state,
                to: PrimalState::Starting,
            });
        }

        self.state = PrimalState::Starting;
        tracing::info!(primal = %self.config.name, backend = ?self.config.storage.backend, "starting");

        // Initialize DAG store based on configured backend
        {
            let backend = match &self.config.storage.backend {
                StorageBackend::Memory => {
                    tracing::info!("using in-memory DAG store");
                    DagBackend::Memory(InMemoryDagStore::new())
                }
                #[cfg(feature = "redb")]
                StorageBackend::Redb => {
                    let path = self
                        .config
                        .storage
                        .path
                        .as_deref()
                        .unwrap_or(crate::constants::DEFAULT_REDB_FILENAME);
                    tracing::info!(path = %path, "using redb DAG store");
                    let store = crate::store_redb::RedbDagStore::open(path).map_err(|e| {
                        PrimalError::StartupFailed(format!("redb open failed: {e}"))
                    })?;
                    DagBackend::Redb(store)
                }
                #[cfg(not(feature = "redb"))]
                StorageBackend::Redb => {
                    return Err(PrimalError::StartupFailed(
                        "Redb storage requested but 'redb' feature not enabled. \
                         Recompile with `--features redb`."
                            .to_string(),
                    ));
                }
            };
            let mut dag_store = self.dag_store.write().await;
            *dag_store = Some(Arc::new(backend));
        }
        {
            let mut payload_store = self.payload_store.write().await;
            *payload_store = Some(Arc::new(InMemoryPayloadStore::new()));
        }

        self.started_at = Some(Instant::now());
        self.state = PrimalState::Running;
        tracing::info!(primal = %self.config.name, "running");

        // Attempt provenance provider connection (non-fatal: trio is optional)
        if let Err(e) = self.provenance_notifier.connect().await {
            tracing::warn!(error = %e, "Provenance notifier connect failed (non-fatal)");
        }

        // Attempt mesh event listener connection (non-fatal: bearDog is optional)
        if let Err(e) = self.mesh_listener.connect().await {
            tracing::warn!(error = %e, "Mesh event listener connect failed (non-fatal)");
        }

        Ok(())
    }

    async fn stop(&mut self) -> std::result::Result<(), PrimalError> {
        if self.state != PrimalState::Running {
            return Err(PrimalError::InvalidTransition {
                from: self.state,
                to: PrimalState::Stopping,
            });
        }

        self.state = PrimalState::Stopping;
        tracing::info!(primal = %self.config.name, "stopping");

        // Clean up stores
        {
            let mut dag_store = self.dag_store.write().await;
            *dag_store = None;
        }
        {
            let mut payload_store = self.payload_store.write().await;
            *payload_store = None;
        }

        self.vertex_session_index.clear();
        self.started_at = None;
        self.state = PrimalState::Stopped;
        tracing::info!(primal = %self.config.name, "stopped");

        Ok(())
    }
}

impl PrimalHealth for RhizoCrypt {
    fn health_status(&self) -> HealthStatus {
        if self.state.is_running() {
            HealthStatus::Healthy
        } else {
            HealthStatus::Unhealthy {
                reason: format!("state: {}", self.state),
            }
        }
    }

    async fn health_check(&self) -> std::result::Result<HealthReport, PrimalError> {
        let mut report = HealthReport::new(&self.config.name, env!("CARGO_PKG_VERSION"))
            .with_status(self.health_status());

        if let Some(uptime) = self.uptime_secs() {
            report = report.with_uptime(uptime);
        }

        Ok(report)
    }
}
