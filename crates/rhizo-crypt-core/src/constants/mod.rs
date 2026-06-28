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

mod crypto;
mod ipc;
mod mesh;
mod methods;
mod network;

pub use crypto::*;
pub use ipc::*;
pub use mesh::*;
pub use methods::*;
pub use network::*;

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

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
