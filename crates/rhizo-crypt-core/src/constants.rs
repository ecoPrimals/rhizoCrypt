// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2024–2026 ecoPrimals Project

//! rhizoCrypt Canonical Constants System
//!
//! **SINGLE SOURCE OF TRUTH FOR ALL STATIC CONSTANTS** ✅
//!
//! This module consolidates all static constants from across rhizoCrypt.
//! All magic numbers are centralized here with clear semantic names,
//! following the ecoPrimals constants-as-single-source-of-truth pattern.
//!
//! ## Philosophy
//!
//! - **Static constants**: Defined here once
//! - **Dynamic constants**: Calculated in config based on environment
//! - **Test constants**: Specialized variants in test harness
//!
//! ## Usage
//!
//! ```rust
//! use rhizo_crypt_core::constants::*;
//! ```

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

/// Default connection timeout.
///
/// Derivation: matches `DEFAULT_TIMEOUT_SECS` — Unix socket connect is
/// sub-millisecond locally; 30s accommodates socket directory scanning.
pub const CONNECTION_TIMEOUT: Duration = Duration::from_secs(30);

/// Default read timeout.
///
/// Derivation: 10s covers slow dehydration summaries with many vertices.
/// Validated: session 3 load tests (1000 vertices → ~2s read).
pub const READ_TIMEOUT: Duration = Duration::from_secs(10);

/// Default write timeout.
///
/// Derivation: mirrors read timeout — symmetric for request/response.
pub const WRITE_TIMEOUT: Duration = Duration::from_secs(10);

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

/// Default capability client timeout (milliseconds).
///
/// Derivation: ecosystem type clients (compute, provenance) need more
/// headroom than local ops. 5s covers cross-primal round-trip + queuing.
/// Aligned with biomeOS `DEFAULT_DISCOVERY_TIMEOUT_MS` (V245).
pub const DEFAULT_CAPABILITY_TIMEOUT_MS: u64 = 5000;

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

/// Maximum retry backoff cap for IPC operations.
///
/// Derivation: 2s prevents unbounded backoff in tight retry loops while
/// giving enough breathing room for a transiently-overloaded peer.
pub const DEFAULT_RETRY_MAX_BACKOFF: Duration = Duration::from_secs(2);

/// Circuit breaker default failure threshold for IPC.
///
/// Derivation: 5 consecutive failures balances fast failure detection
/// against transient network hiccups. Aligned with biomeOS resilience defaults.
pub const CIRCUIT_BREAKER_FAILURE_THRESHOLD: u8 = 5;

/// Circuit breaker default cooldown for IPC.
///
/// Derivation: 30s allows overloaded peers to recover without retrying
/// too aggressively. Matches `CONNECTION_TIMEOUT`.
pub const CIRCUIT_BREAKER_COOLDOWN: Duration = Duration::from_secs(30);

/// Default heartbeat interval for discovery registration.
///
/// Derivation: 45s is 1.5x the health probe interval (30s), ensuring
/// the service ID stays fresh between probes. Validated: spring composition sessions.
pub const DEFAULT_HEARTBEAT_INTERVAL: Duration = Duration::from_secs(45);

/// Default health check interval for service endpoints.
///
/// Derivation: 30s matches biomeOS health probe default and `CONNECTION_TIMEOUT`.
pub const DEFAULT_HEALTH_CHECK_INTERVAL: Duration = Duration::from_secs(30);

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
// STORAGE KEY GEOMETRY
// ============================================================================

/// Size of a `SessionId` in bytes (UUID v7 = 128 bits).
///
/// Source: RFC 9562 (UUID v7) — 128-bit timestamp-ordered identifier.
pub const SESSION_ID_BYTES: usize = 16;

/// Size of a `VertexId` in bytes (BLAKE3 hash = 256 bits).
///
/// Source: BLAKE3 specification — 256-bit output is the default digest size.
pub const VERTEX_ID_BYTES: usize = 32;

/// Separator byte between session and vertex in composite keys.
///
/// Chosen: ASCII colon (0x3A) — never appears in hex-encoded IDs,
/// enables visual separation in debug output.
pub const VERTEX_KEY_SEPARATOR: u8 = b':';

/// Total size of a composite `session:vertex` key.
///
/// Derivation: 16 (UUID v7) + 1 (separator) + 32 (BLAKE3) = 49 bytes.
pub const VERTEX_KEY_SIZE: usize = SESSION_ID_BYTES + 1 + VERTEX_ID_BYTES;

/// Estimated average bytes per stored vertex (for memory estimation).
///
/// Derivation: empirical measurement across spring experiment sessions.
/// Median vertex: 32B hash + 16B session + 64B event + 48B metadata +
/// 32B parents + 64B overhead ≈ 256 B. Validated: session 3 benchmarks.
pub const ESTIMATED_BYTES_PER_VERTEX: u64 = 256;

// ============================================================================
// HTTP / IPC BUFFER SIZES
// ============================================================================

/// Initial capacity for HTTP response buffers over IPC.
///
/// Derivation: typical JSON-RPC responses are 200-2000 bytes. 4096 (1 page)
/// avoids reallocation for most responses while keeping initial allocation small.
pub const HTTP_RESPONSE_BUFFER_CAPACITY: usize = 4096;

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
// COMPRESSION CONSTANTS
// ============================================================================

/// Maximum line length for newline-delimited JSON-RPC (16 MiB).
///
/// Derivation: matches BTSP `MAX_FRAME_SIZE` (16 MiB). Largest realistic
/// single JSON-RPC request is a batch dehydration summary (~1 MiB). 16 MiB
/// provides generous headroom while preventing unbounded memory allocation
/// from misbehaving or adversarial clients.
pub const MAX_JSONRPC_LINE_LENGTH: usize = 16 * 1024 * 1024;

/// Default compression threshold (1 KB).
///
/// Payloads smaller than this are not compressed.
pub const DEFAULT_COMPRESSION_THRESHOLD: usize = 1024;

// ============================================================================
// UNIX SOCKET CONSTANTS (Tower Atomic IPC)
// ============================================================================

/// Subdirectory name for biomeOS socket discovery.
///
/// All ecoBin primals place sockets under `$XDG_RUNTIME_DIR/{BIOMEOS_SOCKET_SUBDIR}/`.
/// This matches the ecosystem standard from `IPC_COMPLIANCE_MATRIX.md` and
/// `CAPABILITY_BASED_DISCOVERY_STANDARD.md`.
pub const BIOMEOS_SOCKET_SUBDIR: &str = "biomeos";

/// Default directory for primal Unix sockets (Linux default).
///
/// This is the Linux-specific default. For platform-agnostic behavior, use
/// [`crate::transport::socket_dir()`] which performs runtime platform detection per ecoBin v2.0.
///
/// Each primal creates a socket at `{SOCKET_DIR}/{primal_name}.sock`.
pub const DEFAULT_SOCKET_DIR: &str = "/run/biomeos";

/// File extension for Unix domain sockets.
pub const SOCKET_FILE_EXTENSION: &str = ".sock";

// ============================================================================
// API PATH CONSTANTS
// ============================================================================

// ============================================================================
// CAPABILITY ADVERTISEMENT
// ============================================================================

/// Capability domains this primal advertises for discovery registration.
///
/// Used by `SongbirdConfig` default and capability descriptors. Aligned
/// with `capability_registry.toml` provider entries for `rhizocrypt`.
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

// ============================================================================
// TEST CONSTANTS
// ============================================================================

/// Port range start for test isolation.
///
/// Tests should use OS-assigned ports (0) instead of hardcoded ports.
/// This constant is for documentation only.
#[cfg(test)]
pub const TEST_PORT_RANGE_START: u16 = 0;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_network_constants() {
        assert_eq!(DEFAULT_RPC_PORT, 0);
        assert_eq!(DEFAULT_RPC_HOST, "127.0.0.1");
        assert_eq!(PRODUCTION_BIND_ADDRESS, "0.0.0.0");
    }

    #[test]
    fn test_timeout_constants() {
        assert_eq!(DEFAULT_TIMEOUT_SECS, 30);
        assert_eq!(CONNECTION_TIMEOUT, Duration::from_secs(30));
        assert_eq!(HEALTH_CHECK_TIMEOUT, Duration::from_secs(5));
    }

    #[test]
    fn test_resource_limits() {
        assert_eq!(DEFAULT_MAX_CONNECTIONS, 1000);
        assert_eq!(DEFAULT_CACHE_SIZE, 1000);
        assert_eq!(DEFAULT_MAX_PAYLOAD_SIZE, 100 * 1024 * 1024);
    }

    #[test]
    fn test_storage_key_geometry() {
        assert_eq!(SESSION_ID_BYTES, 16);
        assert_eq!(VERTEX_ID_BYTES, 32);
        assert_eq!(VERTEX_KEY_SEPARATOR, b':');
        assert_eq!(VERTEX_KEY_SIZE, SESSION_ID_BYTES + 1 + VERTEX_ID_BYTES);
        assert_eq!(VERTEX_KEY_SIZE, 49);
    }

    #[test]
    fn test_cost_tier_thresholds_are_ordered() {
        const { assert!(COST_TIER_LOW_THRESHOLD_MS < COST_TIER_MEDIUM_THRESHOLD_MS) };
    }

    #[test]
    fn test_session_constants() {
        assert_eq!(DEFAULT_SESSION_TIMEOUT, Duration::from_secs(7 * 24 * 3600));
        assert_eq!(DEFAULT_LOAN_GRACE, Duration::from_secs(24 * 3600));
        assert_eq!(MAX_RESLICE_DEPTH, 3);
    }

    #[test]
    fn test_buffer_sizes() {
        assert_eq!(DEFAULT_EVENT_BUFFER_SIZE, 1000);
        assert_eq!(DEFAULT_CHANNEL_BUFFER_SIZE, 100);
    }

    #[test]
    fn test_socket_constants_preserved() {
        assert_eq!(DEFAULT_SOCKET_DIR, "/run/biomeos");
        assert_eq!(SOCKET_FILE_EXTENSION, ".sock");
    }
}
