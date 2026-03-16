// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2024–2026 ecoPrimals Project

//! rhizoCrypt Canonical Constants System
//!
//! **SINGLE SOURCE OF TRUTH FOR ALL STATIC CONSTANTS** ✅
//!
//! This module consolidates all static constants from across rhizoCrypt.
//! Following the pattern established by Songbird, all magic numbers
//! are centralized here with clear semantic names.
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

use std::path::{Path, PathBuf};
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

/// Production bind address (all interfaces).
///
/// In production, bind to all interfaces to accept external connections.
pub const PRODUCTION_BIND_ADDRESS: &str = "0.0.0.0";

/// Localhost IPv4 address.
pub const LOCALHOST: &str = "127.0.0.1";

/// Localhost IPv6 address.
pub const LOCALHOST_V6: &str = "::1";

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
/// computation + IPC to LoamSpine). Validated: session 4 dehydration bench.
pub const REQUEST_TIMEOUT: Duration = Duration::from_secs(60);

/// Default health check timeout.
///
/// Derivation: health.check is a trivial in-memory operation; 5s provides
/// generous margin for cold-start and GC pauses. Aligns with biomeOS
/// health probe interval (5s default).
pub const HEALTH_CHECK_TIMEOUT: Duration = Duration::from_secs(5);

/// Discovery adapter connection timeout.
///
/// Derivation: Songbird socket resolution + capability probe. 10s
/// covers 4-tier fallback discovery (env → XDG → /run → /tmp).
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
/// Covers long-running spring experiment sessions. DashMap remains
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

/// Default garbage collection interval for expired sessions (2 minutes).
pub const DEFAULT_GC_INTERVAL: Duration = Duration::from_secs(120);

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
// SLED STORAGE CONSTANTS
// ============================================================================

/// Sled database cache capacity (128 MB).
///
/// Derivation: 128 MB is sled's recommended default for workloads with
/// ~100K active keys. Balances read performance against memory pressure.
pub const SLED_CACHE_SIZE_BYTES: u64 = 128 * 1024 * 1024;

/// Sled flush interval in milliseconds.
///
/// Derivation: 1000ms provides near-real-time durability while batching
/// writes for performance. Ephemeral sessions tolerate up to 1s loss.
pub const SLED_FLUSH_INTERVAL_MS: u64 = 1000;

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
/// Derivation: DashMap lookup + BLAKE3 hash = ~0.5ms. 2ms covers single
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

/// Default compression threshold (1 KB).
///
/// Payloads smaller than this are not compressed.
pub const DEFAULT_COMPRESSION_THRESHOLD: usize = 1024;

// ============================================================================
// UNIX SOCKET CONSTANTS (Tower Atomic IPC)
// ============================================================================

/// Default directory for primal Unix sockets (Linux default).
///
/// This is the Linux-specific default. For platform-agnostic behavior, use
/// [`socket_dir()`] which performs runtime platform detection per ecoBin v2.0.
///
/// Each primal creates a socket at `{SOCKET_DIR}/{primal_name}.sock`.
pub const DEFAULT_SOCKET_DIR: &str = "/run/ecoPrimals";

/// File extension for Unix domain sockets.
pub const SOCKET_FILE_EXTENSION: &str = ".sock";

// ============================================================================
// PLATFORM-AGNOSTIC IPC (ecoBin v2.0)
// ============================================================================

/// Returns the directory for path-based Unix sockets, or `None` on platforms
/// that use non-path transports (Android abstract sockets, Windows named pipes).
///
/// Platform behavior:
/// - **Linux/macOS/BSD**: Checks `XDG_RUNTIME_DIR` first; falls back to
///   `/run/ecoPrimals` on Linux, `/tmp/ecoPrimals` elsewhere.
/// - **Android**: Returns `None` (use abstract sockets).
/// - **Windows**: Returns `None` (use named pipes or TCP).
/// - **General fallback**: `/tmp/ecoPrimals`.
#[must_use]
pub fn socket_dir() -> Option<PathBuf> {
    if cfg!(target_os = "android") || cfg!(target_os = "windows") {
        return None;
    }

    if let Ok(runtime_dir) = std::env::var("XDG_RUNTIME_DIR") {
        let path = Path::new(&runtime_dir).join("ecoPrimals");
        return Some(path);
    }

    let fallback = if cfg!(target_os = "linux") {
        PathBuf::from(DEFAULT_SOCKET_DIR)
    } else {
        std::env::temp_dir().join("ecoPrimals")
    };
    Some(fallback)
}

/// Constructs the full socket path for a primal, or `None` if path-based
/// sockets are not available on this platform.
///
/// Returns `{socket_dir}/{name}.sock` when [`socket_dir()`] is `Some`.
#[must_use]
pub fn socket_path_for_primal(name: &str) -> Option<PathBuf> {
    let dir = socket_dir()?;
    let filename = format!("{name}{SOCKET_FILE_EXTENSION}");
    Some(dir.join(filename))
}

/// Transport hint for primal IPC, selected per-platform per ecoBin v2.0.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum TransportHint {
    /// Path-based Unix domain socket.
    UnixSocket(PathBuf),
    /// TCP connection (fallback or Windows).
    Tcp {
        /// Host to connect to.
        host: String,
        /// Port number.
        port: u16,
    },
    /// Abstract namespace socket (Android, Linux abstract).
    AbstractSocket(String),
}

/// Returns the preferred transport for the current platform.
///
/// Platform selection:
/// - **Linux/macOS/BSD**: Unix socket when path-based sockets are available;
///   otherwise TCP with localhost.
/// - **Android**: Abstract socket.
/// - **Windows**: TCP with localhost.
#[must_use]
pub fn preferred_transport(primal_name: &str, port: u16) -> TransportHint {
    if cfg!(target_os = "android") {
        return TransportHint::AbstractSocket(format!("ecoPrimals.{primal_name}"));
    }

    if cfg!(target_os = "windows") {
        return TransportHint::Tcp {
            host: DEFAULT_RPC_HOST.to_string(),
            port,
        };
    }

    socket_path_for_primal(primal_name).map_or_else(
        || TransportHint::Tcp {
            host: DEFAULT_RPC_HOST.to_string(),
            port,
        },
        TransportHint::UnixSocket,
    )
}

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
#[expect(clippy::unwrap_used, reason = "test code")]
mod tests {
    use super::*;
    use std::path::Path;

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
    fn test_socket_dir_respects_xdg_runtime_dir() {
        let temp = tempfile::tempdir().expect("temp dir");
        let runtime_path = temp.path().to_path_buf();
        let path_str = runtime_path.to_str().unwrap();
        temp_env::with_vars([("XDG_RUNTIME_DIR", Some(path_str))], || {
            let result = socket_dir();
            if cfg!(target_os = "android") || cfg!(target_os = "windows") {
                assert!(result.is_none(), "Android/Windows should return None");
            } else {
                let dir = result.expect("Unix-like should return Some");
                assert_eq!(dir, runtime_path.join("ecoPrimals"));
            }
        });
    }

    #[test]
    fn test_socket_dir_fallback_without_xdg() {
        temp_env::with_vars([("XDG_RUNTIME_DIR", None::<&str>)], || {
            let result = socket_dir();
            if cfg!(target_os = "android") || cfg!(target_os = "windows") {
                assert!(result.is_none());
            } else if cfg!(target_os = "linux") {
                assert_eq!(result, Some(PathBuf::from(DEFAULT_SOCKET_DIR)));
            } else {
                assert_eq!(result, Some(PathBuf::from("/tmp/ecoPrimals")));
            }
        });
    }

    #[test]
    fn test_socket_path_for_primal_unix_like() {
        if cfg!(target_os = "android") || cfg!(target_os = "windows") {
            let result = socket_path_for_primal("rhizoCrypt");
            assert!(result.is_none());
            return;
        }

        let temp = tempfile::tempdir().expect("temp dir");
        let path_str = temp.path().to_str().unwrap();
        temp_env::with_vars([("XDG_RUNTIME_DIR", Some(path_str))], || {
            let path = socket_path_for_primal("rhizoCrypt").expect("should return path");
            assert!(path.ends_with("rhizoCrypt.sock"));
            assert_eq!(path.file_name().unwrap(), "rhizoCrypt.sock");
        });
    }

    #[test]
    fn test_socket_path_for_primal_uses_socket_extension() {
        if cfg!(target_os = "android") || cfg!(target_os = "windows") {
            return;
        }

        let path = socket_path_for_primal("testPrimal").expect("should return path");
        assert!(path.to_string_lossy().ends_with(".sock"));
    }

    #[test]
    fn test_transport_hint_android_abstract() {
        if !cfg!(target_os = "android") {
            return;
        }

        let hint = preferred_transport("rhizoCrypt", 9400);
        assert!(matches!(hint, TransportHint::AbstractSocket(_)));
        if let TransportHint::AbstractSocket(name) = hint {
            assert_eq!(name, "ecoPrimals.rhizoCrypt");
        }
    }

    #[test]
    fn test_transport_hint_windows_tcp() {
        if !cfg!(target_os = "windows") {
            return;
        }

        let hint = preferred_transport("rhizoCrypt", 9400);
        assert!(matches!(hint, TransportHint::Tcp { .. }));
        if let TransportHint::Tcp {
            host,
            port,
        } = hint
        {
            assert_eq!(host, "127.0.0.1");
            assert_eq!(port, 9400);
        }
    }

    #[test]
    fn test_transport_hint_unix_socket_when_available() {
        if cfg!(target_os = "android") || cfg!(target_os = "windows") {
            return;
        }

        let temp = tempfile::tempdir().expect("temp dir");
        let path_str = temp.path().to_str().unwrap();
        temp_env::with_vars([("XDG_RUNTIME_DIR", Some(path_str))], || {
            let hint = preferred_transport("rhizoCrypt", 9400);
            assert!(matches!(hint, TransportHint::UnixSocket(_)));
            if let TransportHint::UnixSocket(path) = hint {
                assert!(path.ends_with("rhizoCrypt.sock"));
            }
        });
    }

    #[test]
    fn test_transport_hint_tcp_fallback() {
        if cfg!(target_os = "android") || cfg!(target_os = "windows") {
            return;
        }

        temp_env::with_vars([("XDG_RUNTIME_DIR", None::<&str>)], || {
            let hint = preferred_transport("rhizoCrypt", 9400);
            match &hint {
                TransportHint::UnixSocket(path) => {
                    assert!(
                        path.starts_with(Path::new(DEFAULT_SOCKET_DIR))
                            || path.starts_with(Path::new("/tmp/ecoPrimals"))
                    );
                }
                TransportHint::Tcp {
                    host,
                    port,
                } => {
                    assert_eq!(host, "127.0.0.1");
                    assert_eq!(*port, 9400);
                }
                TransportHint::AbstractSocket(_) => panic!("Unexpected AbstractSocket on Unix"),
            }
        });
    }

    #[test]
    fn test_transport_hint_equality() {
        let tcp1 = TransportHint::Tcp {
            host: "127.0.0.1".to_string(),
            port: 9400,
        };
        let tcp2 = TransportHint::Tcp {
            host: "127.0.0.1".to_string(),
            port: 9400,
        };
        assert_eq!(tcp1, tcp2);

        let abstract1 = TransportHint::AbstractSocket("ecoPrimals.test".to_string());
        let abstract2 = TransportHint::AbstractSocket("ecoPrimals.test".to_string());
        assert_eq!(abstract1, abstract2);
    }

    #[test]
    fn test_default_socket_dir_constant_preserved() {
        assert_eq!(DEFAULT_SOCKET_DIR, "/run/ecoPrimals");
        assert_eq!(SOCKET_FILE_EXTENSION, ".sock");
    }

    #[test]
    fn test_socket_dir_xdg_runtime_dir_empty_string() {
        if cfg!(target_os = "android") || cfg!(target_os = "windows") {
            return;
        }
        temp_env::with_vars([("XDG_RUNTIME_DIR", Some(""))], || {
            let result = socket_dir();
            let dir = result.expect("Empty XDG_RUNTIME_DIR still yields Some on Unix");
            assert_eq!(dir, PathBuf::from("ecoPrimals"));
        });
    }

    #[test]
    fn test_socket_path_for_primal_empty_name() {
        if cfg!(target_os = "android") || cfg!(target_os = "windows") {
            return;
        }
        let path = socket_path_for_primal("").expect("should return path");
        assert!(path.to_string_lossy().ends_with(".sock"));
        assert_eq!(path.file_name().unwrap(), ".sock");
    }

    #[test]
    fn test_socket_path_for_primal_special_characters() {
        if cfg!(target_os = "android") || cfg!(target_os = "windows") {
            return;
        }
        let path = socket_path_for_primal("my-primal.v2").expect("should return path");
        assert!(path.to_string_lossy().ends_with("my-primal.v2.sock"));
        assert_eq!(path.file_name().unwrap(), "my-primal.v2.sock");
    }

    #[test]
    fn test_transport_hint_debug_format() {
        let unix = TransportHint::UnixSocket(PathBuf::from("/run/ecoPrimals/rhizoCrypt.sock"));
        let debug_str = format!("{unix:?}");
        assert!(debug_str.contains("UnixSocket"));
        assert!(debug_str.contains("rhizoCrypt.sock"));

        let tcp = TransportHint::Tcp {
            host: "127.0.0.1".to_string(),
            port: 9400,
        };
        let debug_str = format!("{tcp:?}");
        assert!(debug_str.contains("Tcp"));
        assert!(debug_str.contains("127.0.0.1"));
        assert!(debug_str.contains("9400"));

        let abstract_sock = TransportHint::AbstractSocket("ecoPrimals.test".to_string());
        let debug_str = format!("{abstract_sock:?}");
        assert!(debug_str.contains("AbstractSocket"));
        assert!(debug_str.contains("ecoPrimals.test"));
    }

    #[test]
    fn test_transport_hint_unix_socket_equality() {
        let p1 = PathBuf::from("/run/ecoPrimals/a.sock");
        let p2 = PathBuf::from("/run/ecoPrimals/a.sock");
        let u1 = TransportHint::UnixSocket(p1);
        let u2 = TransportHint::UnixSocket(p2);
        assert_eq!(u1, u2);
    }

    #[test]
    fn test_preferred_transport_empty_primal_name() {
        if cfg!(target_os = "android") {
            let hint = preferred_transport("", 9400);
            assert!(matches!(hint, TransportHint::AbstractSocket(_)));
            if let TransportHint::AbstractSocket(name) = hint {
                assert_eq!(name, "ecoPrimals.");
            }
            return;
        }
        if cfg!(target_os = "windows") {
            let hint = preferred_transport("", 9400);
            assert!(matches!(hint, TransportHint::Tcp { .. }));
            return;
        }
        let hint = preferred_transport("", 9400);
        match hint {
            TransportHint::UnixSocket(p) => assert!(p.to_string_lossy().ends_with(".sock")),
            TransportHint::Tcp {
                host,
                port,
            } => {
                assert_eq!(host, "127.0.0.1");
                assert_eq!(port, 9400);
            }
            TransportHint::AbstractSocket(_) => panic!("Unexpected AbstractSocket on Unix"),
        }
    }

    #[test]
    fn test_socket_path_for_primal_none_on_unsupported_platform() {
        if !cfg!(target_os = "android") && !cfg!(target_os = "windows") {
            return;
        }
        assert!(socket_path_for_primal("rhizoCrypt").is_none());
        assert!(socket_path_for_primal("any").is_none());
    }
}
