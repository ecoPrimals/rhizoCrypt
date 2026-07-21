// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2024–2026 ecoPrimals Project

//! Unix domain socket paths, UDS conventions, and IPC transport defaults.

use std::time::Duration;

// ============================================================================
// TIMEOUT CONSTANTS (IPC)
// ============================================================================

/// Default connection timeout.
///
/// Derivation: matches `DEFAULT_TIMEOUT_SECS` — Unix socket connect is
/// sub-millisecond locally; 30s accommodates socket directory scanning.
pub const CONNECTION_TIMEOUT: Duration = Duration::from_secs(30);

/// Default capability client timeout (milliseconds).
///
/// Derivation: ecosystem type clients (compute, provenance) need more
/// headroom than local ops. 5s covers cross-primal round-trip + queuing.
/// Aligned with biomeOS `DEFAULT_DISCOVERY_TIMEOUT_MS` (V245).
pub const DEFAULT_CAPABILITY_TIMEOUT_MS: u64 = 5000;

/// Maximum retry backoff cap for IPC operations.
///
/// Derivation: 2s prevents unbounded backoff in tight retry loops while
/// giving enough breathing room for a transiently-overloaded peer.
pub const DEFAULT_RETRY_MAX_BACKOFF: Duration = Duration::from_secs(2);

// ============================================================================
// HTTP / IPC BUFFER SIZES
// ============================================================================

/// Initial capacity for HTTP response buffers over IPC.
///
/// Derivation: typical JSON-RPC responses are 200-2000 bytes. 4096 (1 page)
/// avoids reallocation for most responses while keeping initial allocation small.
pub const HTTP_RESPONSE_BUFFER_CAPACITY: usize = 4096;

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

/// Fallback temporary directory for UDS discovery when `XDG_RUNTIME_DIR` is absent.
pub const POSIX_FALLBACK_TMPDIR: &str = "/tmp";

/// Default family identifier for neural API socket discovery.
///
/// Used when `ECOPRIMALS_FAMILY_ID` is not set.
pub const DEFAULT_FAMILY_ID: &str = "ecoPrimal";

/// Default in-memory storage cap (1 GiB).
pub const DEFAULT_MAX_MEMORY_BYTES: u64 = 1024 * 1024 * 1024;

/// Delay before firing the readiness notification after server bind.
///
/// Gives the OS a moment to finalize the socket bind before notifying
/// waiting integration tests. Sub-100ms is enough for all observed platforms.
pub const READINESS_NOTIFY_DELAY: Duration = Duration::from_millis(50);

// ============================================================================
// ANNOUNCE PAYLOAD CONSTANTS (biomeOS Neural API)
// ============================================================================

/// Cost hint for DAG operations in `primal.announce` payload.
pub const ANNOUNCE_COST_DAG: f64 = 10.0;
/// Cost hint for integrity operations in `primal.announce` payload.
pub const ANNOUNCE_COST_INTEGRITY: f64 = 5.0;
/// Cost hint for Merkle operations in `primal.announce` payload.
pub const ANNOUNCE_COST_MERKLE: f64 = 8.0;

/// Latency estimate (ms) for DAG operations in `primal.announce` payload.
pub const ANNOUNCE_LATENCY_DAG_MS: u64 = 15;
/// Latency estimate (ms) for integrity operations in `primal.announce` payload.
pub const ANNOUNCE_LATENCY_INTEGRITY_MS: u64 = 5;
/// Latency estimate (ms) for Merkle operations in `primal.announce` payload.
pub const ANNOUNCE_LATENCY_MERKLE_MS: u64 = 10;

/// Connect timeout for outbound Neural API UDS calls (seconds).
pub const NEURAL_API_CONNECT_TIMEOUT_SECS: u64 = 2;

/// Read timeout for outbound Neural API UDS responses (seconds).
pub const NEURAL_API_READ_TIMEOUT_SECS: u64 = 5;
