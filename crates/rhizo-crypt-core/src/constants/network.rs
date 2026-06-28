// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2024–2026 ecoPrimals Project

//! Network, RPC, and general service configuration constants.

use std::time::Duration;

// ============================================================================
// PRIMAL IDENTITY
// ============================================================================

/// This primal's canonical name.
///
/// Used for discovery registration and service identification.
pub const PRIMAL_NAME: &str = "rhizoCrypt";

/// This primal's role description.
pub const PRIMAL_ROLE: &str = "Ephemeral DAG Engine";

// ============================================================================
// NETWORK CONSTANTS
// ============================================================================

/// Default RPC port (0 = OS-assigned for automatic port selection).
///
/// Using port 0 allows the OS to assign an available port automatically,
/// preventing port conflicts in testing and development.
pub const DEFAULT_RPC_PORT: u16 = 0;

/// Production RPC port.
///
/// Used when `RHIZOCRYPT_ENV` is not "development" and no port override is set.
pub const PRODUCTION_RPC_PORT: u16 = 9400;

/// Default RPC host for local development.
///
/// Binds to localhost only for security in development mode.
pub const DEFAULT_RPC_HOST: &str = "127.0.0.1";

/// Discovery endpoint scheme per wateringHole transport conventions.
///
/// `http` = HTTP JSON-RPC, `tcp` = newline-delimited JSON-RPC.
/// rhizoCrypt exposes dual-mode (HTTP + newline) on the same port.
pub const DISCOVERY_ENDPOINT_SCHEME: &str = "http";

/// Production bind address (all interfaces).
///
/// In production, bind to all interfaces to accept external connections.
pub const PRODUCTION_BIND_ADDRESS: &str = "0.0.0.0";

/// Localhost IPv4 address.
pub const LOCALHOST: &str = "127.0.0.1";

/// Localhost IPv6 address.
pub const LOCALHOST_V6: &str = "::1";

/// Localhost hostname for HTTP Host headers (especially UDS where IP is N/A).
pub const LOCALHOST_HOSTNAME: &str = "localhost";

/// Port offset for the JSON-RPC server relative to the tarpc port.
///
/// When the JSON-RPC port is not explicitly configured, it is calculated as
/// `tarpc_port + JSONRPC_PORT_OFFSET`. Override via `RHIZOCRYPT_JSONRPC_PORT`.
pub const JSONRPC_PORT_OFFSET: u16 = 1;

// ============================================================================
// TIMEOUT CONSTANTS
// ============================================================================

/// Default timeout for network operations (in seconds).
///
/// Derivation: 30s covers 99th-percentile latency for cross-primal IPC on
/// loaded Tower Atomic deployments. Validated: session 4 E2E tests.
pub const DEFAULT_TIMEOUT_SECS: u64 = 30;

/// Default request timeout.
///
/// Derivation: 60s covers full dehydration pipeline (DAG walk + Merkle
/// computation + IPC to permanent storage). Validated: session 4 dehydration bench.
pub const REQUEST_TIMEOUT: Duration = Duration::from_secs(60);

/// Default health check timeout.
///
/// Derivation: health.check is a trivial in-memory operation; 5s provides
/// generous margin for cold-start and GC pauses. Aligns with biomeOS
/// health probe interval (5s default).
pub const HEALTH_CHECK_TIMEOUT: Duration = Duration::from_secs(5);

/// Discovery adapter connection timeout.
///
/// Derivation: discovery adapter socket resolution + capability probe.
/// 10s covers 4-tier fallback (env → XDG → /run → /tmp).
pub const DISCOVERY_TIMEOUT: Duration = Duration::from_secs(10);

// ============================================================================
// RESOURCE LIMITS
// ============================================================================

/// Default maximum concurrent connections.
///
/// Derivation: 1000 matches biomeOS `DEFAULT_MAX_CONNECTIONS` and handles
/// all springs + primals in a fully-composed niche with headroom for
/// graph-driven parallel requests. Validated: session 3 load tests.
pub const DEFAULT_MAX_CONNECTIONS: usize = 1000;

/// Default cache size for various caches.
///
/// Derivation: 1000 entries balances memory usage (~256 KB at 256 B/entry)
/// against cache hit rate for typical session sizes.
pub const DEFAULT_CACHE_SIZE: usize = 1000;

/// Default maximum payload size (100 MB).
///
/// Derivation: accommodates large experiment data payloads from springs
/// (genomic sequences, sensor time series). Well below typical system
/// memory; individual vertex payloads are typically < 1 KB.
pub const DEFAULT_MAX_PAYLOAD_SIZE: usize = 100 * 1024 * 1024;

/// Default maximum vertices per session.
///
/// Derivation: 100K vertices at ~256 B/vertex = ~25 MB working memory.
/// Covers long-running spring experiment sessions. `DashMap` remains
/// performant at this scale. Validated: session 3 property tests.
pub const DEFAULT_MAX_VERTICES_PER_SESSION: usize = 100_000;

/// Default maximum sessions.
///
/// Derivation: 10K concurrent sessions × ~25 MB peak = ~250 GB theoretical
/// max. Practical deployments use < 100 concurrent sessions.
pub const DEFAULT_MAX_SESSIONS: usize = 10_000;

/// Default maximum slices per session.
///
/// Derivation: 100 slices per session covers iterative experiment workflows
/// (checkout → modify → re-checkout). Limited by `MAX_RESLICE_DEPTH`.
pub const DEFAULT_MAX_SLICES_PER_SESSION: usize = 100;

// ============================================================================
// SESSION CONSTANTS
// ============================================================================

/// Default session timeout (7 days).
///
/// Derivation: springs run multi-day experiment campaigns. 7 days covers
/// a work week with weekend buffer. Sessions are ephemeral — GC reclaims
/// expired sessions. Override via `SessionConfig::timeout`.
pub const DEFAULT_SESSION_TIMEOUT: Duration = Duration::from_secs(7 * 24 * 3600);

/// Default loan grace period (1 day).
///
/// Derivation: loan slices may be held overnight during iterative analysis.
/// 24h grace prevents premature reclamation while maintaining freshness.
pub const DEFAULT_LOAN_GRACE: Duration = Duration::from_secs(24 * 3600);

/// Maximum reslice depth.
///
/// Derivation: prevents unbounded re-slicing chains. 3 levels (slice →
/// reslice → re-reslice) covers the practical use case of progressive
/// refinement without creating deep dependency chains.
pub const MAX_RESLICE_DEPTH: usize = 3;

// ============================================================================
// DEHYDRATION CONSTANTS
// ============================================================================

/// Default attestation timeout (60 seconds).
pub const DEFAULT_ATTESTATION_TIMEOUT_SECS: u64 = 60;

/// Default commit timeout (60 seconds).
pub const DEFAULT_COMMIT_TIMEOUT_SECS: u64 = 60;

/// Default garbage collection interval for expired sessions.
///
/// Derivation: 60s keeps memory pressure low without excessive overhead.
/// Shorter than the production 120s variant because dev sessions are small.
pub const DEFAULT_GC_INTERVAL: Duration = Duration::from_secs(60);

/// Default expiration grace period for sessions (1 hour).
///
/// Derivation: gives agents time to complete dehydration after session
/// timeout before GC reclaims the session data.
pub const DEFAULT_EXPIRATION_GRACE: Duration = Duration::from_secs(3600);

/// Default dehydration attestation timeout.
pub const DEFAULT_ATTESTATION_TIMEOUT: Duration = Duration::from_secs(60);

/// Default dehydration retry delay.
pub const DEFAULT_DEHYDRATION_RETRY_DELAY: Duration = Duration::from_secs(5);

/// Default maximum payload size per session (1 GiB).
///
/// Derivation: accommodates large experiment data while preventing a
/// single session from exhausting host memory.
pub const DEFAULT_MAX_PAYLOAD_BYTES: usize = 1024 * 1024 * 1024;

/// Default session max duration (1 hour).
///
/// Derivation: individual session config default; distinct from
/// `DEFAULT_SESSION_TIMEOUT` which is the GC boundary (7 days).
pub const DEFAULT_SESSION_MAX_DURATION: Duration = Duration::from_secs(3600);

/// Rate-limit cleanup interval for production (1 minute).
pub const RATE_LIMIT_CLEANUP_INTERVAL: Duration = Duration::from_secs(60);

/// Rate-limit cleanup interval for development (5 minutes).
pub const RATE_LIMIT_CLEANUP_INTERVAL_DEV: Duration = Duration::from_secs(300);

// ============================================================================
// BUFFER SIZES
// ============================================================================

/// Default event buffer size for subscriptions.
pub const DEFAULT_EVENT_BUFFER_SIZE: usize = 1000;

/// Default channel buffer size.
pub const DEFAULT_CHANNEL_BUFFER_SIZE: usize = 100;

// ============================================================================
// RETRY CONSTANTS
// ============================================================================

/// Default maximum retry attempts.
pub const DEFAULT_MAX_RETRIES: u8 = 3;

/// Default retry backoff base (milliseconds).
pub const DEFAULT_RETRY_BACKOFF_MS: u64 = 100;

// ============================================================================
// DISCOVERY CONSTANTS
// ============================================================================

/// Discovery source connection/query timeout.
pub const DISCOVERY_QUERY_TIMEOUT: Duration = Duration::from_secs(5);

/// Discovery response buffer initial capacity (bytes).
pub const DISCOVERY_RESPONSE_BUFFER_SIZE: usize = 4096;

// ============================================================================
// PROVENANCE CONSTANTS
// ============================================================================

/// Provenance provider connection timeout.
pub const PROVENANCE_CONNECTION_TIMEOUT: Duration = Duration::from_secs(5);

/// Provenance provider response timeout.
pub const PROVENANCE_RESPONSE_TIMEOUT: Duration = Duration::from_secs(10);

/// Default maximum results per provenance query.
pub const PROVENANCE_DEFAULT_MAX_RESULTS: usize = 1000;

// ============================================================================
// DEFAULT FILENAMES
// ============================================================================

/// Default redb database filename when no path is configured.
pub const DEFAULT_REDB_FILENAME: &str = "rhizocrypt.redb";

/// Default manifest filename written to the biomeOS socket directory.
pub const DEFAULT_MANIFEST_FILENAME: &str = "rhizocrypt.json";

// ============================================================================
// COST TIER THRESHOLDS (Pathway Learner)
// ============================================================================

/// Operations at or below this latency (ms) are classified as "low" cost.
///
/// Derivation: `DashMap` lookup + BLAKE3 hash = ~0.5ms. 2ms covers single
/// in-memory operations with overhead. biomeOS Pathway Learner uses this
/// to identify parallelizable low-cost ops. Validated: session 4 benchmarks.
pub const COST_TIER_LOW_THRESHOLD_MS: u32 = 2;

/// Operations at or below this latency (ms) are classified as "medium" cost.
///
/// Derivation: 10ms covers multi-step in-memory operations (DAG walk +
/// Merkle path). Above 10ms indicates I/O involvement (disk, network).
pub const COST_TIER_MEDIUM_THRESHOLD_MS: u32 = 10;

// ============================================================================
// RATE LIMIT DEFAULTS
// ============================================================================

/// Default rate limit for read operations (requests per second).
pub const DEFAULT_RATE_LIMIT_READ_RPS: u32 = 1000;

/// Default rate limit for write operations (requests per second).
pub const DEFAULT_RATE_LIMIT_WRITE_RPS: u32 = 100;

/// Default rate limit for expensive operations (requests per second).
pub const DEFAULT_RATE_LIMIT_EXPENSIVE_RPS: u32 = 10;

// ============================================================================
// CAPABILITY ADVERTISEMENT
// ============================================================================

/// Capability domains this primal advertises for discovery registration.
///
/// Used by `SongbirdConfig` default and capability descriptors. Aligned
/// with `config/capability_registry.toml` provider entries for `rhizocrypt`.
pub const ADVERTISED_CAPABILITIES: &[&str] =
    &["dag-engine", "session-management", "merkle-proofs", "slice-checkout", "dehydration"];

// ============================================================================
// API PATH CONSTANTS
// ============================================================================

/// JSON-RPC API path suffix.
pub const JSON_RPC_PATH: &str = "/rpc";

/// REST API version prefix.
pub const API_VERSION_PREFIX: &str = "/api/v1";

/// Health check endpoint path.
pub const HEALTH_CHECK_PATH: &str = "/api/v1/health";

/// Default metrics endpoint path.
pub const METRICS_PATH: &str = "/metrics";

// ============================================================================
// TEST CONSTANTS
// ============================================================================

/// Port range start for test isolation.
///
/// Tests should use OS-assigned ports (0) instead of hardcoded ports.
/// This constant is for documentation only.
#[cfg(test)]
pub const TEST_PORT_RANGE_START: u16 = 0;
