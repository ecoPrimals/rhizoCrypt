// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2024–2026 ecoPrimals Project

//! Songbird discovery operations.

use tracing::debug;

use crate::discovery::{Capability, DiscoveryRegistry, ServiceEndpoint};
use crate::error::{Result, RhizoCryptError};

use super::super::songbird_types::ServiceInfo;
use super::client::SongbirdClient;

impl SongbirdClient {
    /// Discover services by capability.
    ///
    /// # Errors
    ///
    /// Returns `RhizoCryptError::Integration` if:
    /// - Not connected to Songbird
    /// - Discovery query fails
    pub async fn discover(&self, capability: &str) -> Result<Vec<ServiceInfo>> {
        if !self.is_connected().await {
            return Err(RhizoCryptError::integration("Not connected to Songbird"));
        }

        debug!(capability = %capability, "Discovering services");

        // Check cache first
        {
            let cache = self.discovered_services.read().await;
            if let Some(services) = cache.get(capability) {
                debug!(count = services.len(), "Returning cached discovery results");
                return Ok(services.clone());
            }
        }

        #[cfg(feature = "live-clients")]
        {
            let client_guard = self.tarpc_client.read().await;
            if let Some(client) = client_guard.as_ref() {
                let rpc_services = client
                    .discover(tarpc::context::current(), capability.to_string())
                    .await
                    .map_err(|e| RhizoCryptError::integration(format!("tarpc error: {e}")))?;

                // Convert RpcServiceInfo to ServiceInfo
                let services: Vec<ServiceInfo> = rpc_services
                    .into_iter()
                    .map(|s| ServiceInfo {
                        id: s.id,
                        name: s.capability.clone(),
                        endpoint: s.endpoint,
                        capabilities: vec![s.capability],
                        status: s.status,
                        metadata: std::collections::HashMap::new(),
                    })
                    .collect();

                // Cache the results
                self.discovered_services
                    .write()
                    .await
                    .insert(capability.to_string(), services.clone());

                return Ok(services);
            }
        }

        // Scaffolded mode: return empty for capabilities we don't have cached
        Ok(Vec::new())
    }

    /// Discover a service providing signing/DID capability.
    ///
    /// Returns the first available provider of the `signing` capability.
    /// Capability-based: any primal implementing `signing` qualifies.
    ///
    /// # Errors
    ///
    /// Returns `RhizoCryptError::Integration` if not connected or discovery fails.
    pub async fn discover_signing_provider(&self) -> Result<Option<ServiceInfo>> {
        let services = self.discover("signing").await?;
        Ok(services.into_iter().next())
    }

    /// Discover a service providing permanent-storage capability.
    ///
    /// Returns the first available provider of the `permanent-storage` capability.
    /// Capability-based: any primal implementing `permanent-storage` qualifies.
    ///
    /// # Errors
    ///
    /// Returns `RhizoCryptError::Integration` if not connected or discovery fails.
    pub async fn discover_permanent_storage_provider(&self) -> Result<Option<ServiceInfo>> {
        let services = self.discover("permanent-storage").await?;
        Ok(services.into_iter().next())
    }

    /// Discover a service providing payload-storage capability.
    ///
    /// Returns the first available provider of the `payload-storage` capability.
    /// Capability-based: any primal implementing `payload-storage` qualifies.
    ///
    /// # Errors
    ///
    /// Returns `RhizoCryptError::Integration` if not connected or discovery fails.
    pub async fn discover_payload_storage_provider(&self) -> Result<Option<ServiceInfo>> {
        let services = self.discover("payload-storage").await?;
        Ok(services.into_iter().next())
    }

    /// Populate the discovery registry with discovered services.
    ///
    /// This bridges Songbird discovery to the capability-based registry.
    ///
    /// # Errors
    ///
    /// Returns `RhizoCryptError::Integration` if:
    /// - Not connected to Songbird
    /// - Discovery fails
    pub async fn populate_registry(&self, registry: &DiscoveryRegistry) -> Result<()> {
        if !self.is_connected().await {
            return Err(RhizoCryptError::integration("Not connected to Songbird"));
        }

        let capability_mappings: &[(&str, &[Capability])] = &[
            ("signing", &[Capability::DidVerification, Capability::Signing]),
            (
                "permanent-storage",
                &[
                    Capability::PermanentCommit,
                    Capability::SliceCheckout,
                    Capability::SliceResolution,
                ],
            ),
            ("payload-storage", &[Capability::PayloadStorage, Capability::PayloadRetrieval]),
        ];

        for (domain, capabilities) in capability_mappings {
            for service in self.discover(domain).await? {
                if let Ok(addr) = service.endpoint.parse() {
                    registry
                        .register_endpoint(ServiceEndpoint::new(
                            service.name,
                            addr,
                            capabilities.to_vec(),
                        ))
                        .await;
                }
            }
        }

        Ok(())
    }

    /// Update cached discovery results.
    pub async fn cache_discovery(&self, capability: &str, services: Vec<ServiceInfo>) {
        self.discovered_services.write().await.insert(capability.to_string(), services);
    }

    /// Clear discovery cache.
    pub async fn clear_cache(&self) {
        self.discovered_services.write().await.clear();
    }
}
