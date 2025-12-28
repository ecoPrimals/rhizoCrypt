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

use std::time::Duration;

// ============================================================================
// NETWORK CONSTANTS
// ============================================================================

/// Default RPC port (0 = OS-assigned for automatic port selection).
///
/// Using port 0 allows the OS to assign an available port automatically,
/// preventing port conflicts in testing and development.
pub const DEFAULT_RPC_PORT: u16 = 0;

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
pub const DEFAULT_TIMEOUT_SECS: u64 = 30;

/// Default connection timeout.
pub const CONNECTION_TIMEOUT: Duration = Duration::from_secs(30);

/// Default read timeout.
pub const READ_TIMEOUT: Duration = Duration::from_secs(10);

/// Default write timeout.
pub const WRITE_TIMEOUT: Duration = Duration::from_secs(10);

/// Default request timeout.
pub const REQUEST_TIMEOUT: Duration = Duration::from_secs(60);

/// Default health check timeout.
pub const HEALTH_CHECK_TIMEOUT: Duration = Duration::from_secs(5);

/// Discovery adapter connection timeout.
pub const DISCOVERY_TIMEOUT: Duration = Duration::from_secs(10);

// ============================================================================
// RESOURCE LIMITS
// ============================================================================

/// Default maximum concurrent connections.
pub const DEFAULT_MAX_CONNECTIONS: usize = 1000;

/// Default cache size for various caches.
pub const DEFAULT_CACHE_SIZE: usize = 1000;

/// Default maximum payload size (100 MB).
pub const DEFAULT_MAX_PAYLOAD_SIZE: usize = 100 * 1024 * 1024;

/// Default maximum vertices per session.
pub const DEFAULT_MAX_VERTICES_PER_SESSION: usize = 100_000;

/// Default maximum sessions.
pub const DEFAULT_MAX_SESSIONS: usize = 10_000;

/// Default maximum slices per session.
pub const DEFAULT_MAX_SLICES_PER_SESSION: usize = 100;

// ============================================================================
// SESSION CONSTANTS
// ============================================================================

/// Default session timeout (7 days).
pub const DEFAULT_SESSION_TIMEOUT: Duration = Duration::from_secs(7 * 24 * 3600);

/// Default loan grace period (1 day).
pub const DEFAULT_LOAN_GRACE: Duration = Duration::from_secs(24 * 3600);

/// Maximum reslice depth.
pub const MAX_RESLICE_DEPTH: usize = 3;

// ============================================================================
// DEHYDRATION CONSTANTS
// ============================================================================

/// Default attestation timeout (60 seconds).
pub const DEFAULT_ATTESTATION_TIMEOUT_SECS: u64 = 60;

/// Default commit timeout (60 seconds).
pub const DEFAULT_COMMIT_TIMEOUT_SECS: u64 = 60;

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
// COMPRESSION CONSTANTS
// ============================================================================

/// Default compression threshold (1 KB).
///
/// Payloads smaller than this are not compressed.
pub const DEFAULT_COMPRESSION_THRESHOLD: usize = 1024;

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
        assert!(DEFAULT_MAX_PAYLOAD_SIZE > 0);
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
}

