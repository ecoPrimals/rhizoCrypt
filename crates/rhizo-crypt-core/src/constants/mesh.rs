// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2024–2026 ecoPrimals Project

//! Mesh event polling, heartbeat, and trust-domain session constants.

use std::time::Duration;

/// Default heartbeat interval for discovery registration.
///
/// Derivation: 45s is 1.5x the health probe interval (30s), ensuring
/// the service ID stays fresh between probes. Validated: spring composition sessions.
pub const DEFAULT_HEARTBEAT_INTERVAL: Duration = Duration::from_secs(45);

/// Default health check interval for service endpoints.
///
/// Derivation: 30s matches biomeOS health probe default and `CONNECTION_TIMEOUT`.
pub const DEFAULT_HEALTH_CHECK_INTERVAL: Duration = Duration::from_secs(30);

/// Mesh event poller interval (how often to poll the signing provider for trust events).
pub const MESH_POLL_INTERVAL: Duration = Duration::from_secs(30);

/// Mesh event poller connection timeout.
pub const MESH_CONNECTION_TIMEOUT: Duration = Duration::from_secs(5);

/// Mesh event poller response timeout.
pub const MESH_RESPONSE_TIMEOUT: Duration = Duration::from_secs(10);

/// Mesh trust session domain name for DAG auto-provisioning.
pub const MESH_TRUST_DOMAIN: &str = "mesh-trust";

/// Mesh trust session display name.
pub const MESH_TRUST_SESSION_NAME: &str = "mesh-trust-events";
