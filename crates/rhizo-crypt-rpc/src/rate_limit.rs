// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2024–2026 ecoPrimals Project

//! Rate limiting infrastructure for RPC endpoints.
//!
//! Provides token bucket rate limiting per client, with configurable
//! limits for different operation types.

use std::collections::HashMap;
use std::net::IpAddr;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;

/// Rate limit configuration.
#[derive(Debug, Clone)]
pub struct RateLimitConfig {
    /// Maximum requests per second for read operations.
    pub read_rps: u32,
    /// Maximum requests per second for write operations.
    pub write_rps: u32,
    /// Maximum requests per second for expensive operations (merkle, dehydration).
    pub expensive_rps: u32,
    /// Burst multiplier (how many tokens above rate can be stored).
    pub burst_multiplier: u32,
    /// Cleanup interval for stale entries.
    pub cleanup_interval: Duration,
}

impl Default for RateLimitConfig {
    fn default() -> Self {
        Self {
            read_rps: 1000,
            write_rps: 100,
            expensive_rps: 10,
            burst_multiplier: 2,
            cleanup_interval: Duration::from_secs(60),
        }
    }
}

impl RateLimitConfig {
    /// Create production config with conservative limits.
    #[must_use]
    pub const fn production() -> Self {
        Self {
            read_rps: 500,
            write_rps: 50,
            expensive_rps: 5,
            burst_multiplier: 2,
            cleanup_interval: Duration::from_secs(60),
        }
    }

    /// Create development config with relaxed limits.
    #[must_use]
    pub const fn development() -> Self {
        Self {
            read_rps: 10000,
            write_rps: 1000,
            expensive_rps: 100,
            burst_multiplier: 5,
            cleanup_interval: Duration::from_secs(300),
        }
    }

    /// Create config from environment variables.
    ///
    /// Variables:
    /// - `RHIZOCRYPT_RATE_LIMIT_READ_RPS`
    /// - `RHIZOCRYPT_RATE_LIMIT_WRITE_RPS`
    /// - `RHIZOCRYPT_RATE_LIMIT_EXPENSIVE_RPS`
    #[must_use]
    pub fn from_env() -> Self {
        let mut config = Self::default();

        if let Ok(val) = std::env::var("RHIZOCRYPT_RATE_LIMIT_READ_RPS") {
            if let Ok(rps) = val.parse() {
                config.read_rps = rps;
            }
        }

        if let Ok(val) = std::env::var("RHIZOCRYPT_RATE_LIMIT_WRITE_RPS") {
            if let Ok(rps) = val.parse() {
                config.write_rps = rps;
            }
        }

        if let Ok(val) = std::env::var("RHIZOCRYPT_RATE_LIMIT_EXPENSIVE_RPS") {
            if let Ok(rps) = val.parse() {
                config.expensive_rps = rps;
            }
        }

        config
    }
}

/// Operation type for rate limiting.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum OperationType {
    /// Read operations (get_vertex, get_frontier, etc.).
    Read,
    /// Write operations (append_event, create_session, etc.).
    Write,
    /// Expensive operations (merkle proofs, dehydration).
    Expensive,
}

/// Token bucket for a single client.
#[derive(Debug)]
struct TokenBucket {
    /// Current tokens available.
    tokens: f64,
    /// Maximum tokens (burst capacity).
    max_tokens: f64,
    /// Tokens added per second.
    refill_rate: f64,
    /// Last refill time.
    last_refill: Instant,
}

impl TokenBucket {
    fn new(rate: u32, burst_multiplier: u32) -> Self {
        let max = f64::from(rate * burst_multiplier);
        Self {
            tokens: max,
            max_tokens: max,
            refill_rate: f64::from(rate),
            last_refill: Instant::now(),
        }
    }

    fn refill(&mut self) {
        let now = Instant::now();
        let elapsed = now.duration_since(self.last_refill).as_secs_f64();
        self.tokens = self.refill_rate.mul_add(elapsed, self.tokens).min(self.max_tokens);
        self.last_refill = now;
    }

    fn try_consume(&mut self, tokens: f64) -> bool {
        self.refill();
        if self.tokens >= tokens {
            self.tokens -= tokens;
            true
        } else {
            false
        }
    }
}

/// Client-specific rate limit state.
#[derive(Debug)]
struct ClientState {
    read_bucket: TokenBucket,
    write_bucket: TokenBucket,
    expensive_bucket: TokenBucket,
    last_seen: Instant,
}

impl ClientState {
    fn new(config: &RateLimitConfig) -> Self {
        Self {
            read_bucket: TokenBucket::new(config.read_rps, config.burst_multiplier),
            write_bucket: TokenBucket::new(config.write_rps, config.burst_multiplier),
            expensive_bucket: TokenBucket::new(config.expensive_rps, config.burst_multiplier),
            last_seen: Instant::now(),
        }
    }

    fn try_consume(&mut self, op: OperationType) -> bool {
        self.last_seen = Instant::now();
        match op {
            OperationType::Read => self.read_bucket.try_consume(1.0),
            OperationType::Write => self.write_bucket.try_consume(1.0),
            OperationType::Expensive => self.expensive_bucket.try_consume(1.0),
        }
    }
}

/// Rate limiter for the RPC layer.
///
/// Implements per-client token bucket rate limiting with separate
/// buckets for read, write, and expensive operations.
#[derive(Debug)]
pub struct RateLimiter {
    config: RateLimitConfig,
    clients: Arc<RwLock<HashMap<IpAddr, ClientState>>>,
    /// Whether rate limiting is enabled.
    enabled: bool,
}

impl RateLimiter {
    /// Create a new rate limiter with the given configuration.
    #[must_use]
    pub fn new(config: RateLimitConfig) -> Self {
        Self {
            config,
            clients: Arc::new(RwLock::new(HashMap::new())),
            enabled: true,
        }
    }

    /// Create a rate limiter that allows everything (for testing).
    #[must_use]
    #[allow(clippy::missing_const_for_fn)] // Arc::new is not const
    pub fn disabled() -> Self {
        Self {
            config: RateLimitConfig::default(),
            clients: Arc::new(RwLock::new(HashMap::new())),
            enabled: false,
        }
    }

    /// Check if an operation is allowed for a client.
    ///
    /// Returns `true` if the operation is allowed, `false` if rate limited.
    pub async fn check(&self, client: IpAddr, op: OperationType) -> bool {
        if !self.enabled {
            return true;
        }

        let mut clients = self.clients.write().await;

        let state = clients.entry(client).or_insert_with(|| ClientState::new(&self.config));

        state.try_consume(op)
    }

    /// Get the number of tracked clients.
    pub async fn client_count(&self) -> usize {
        self.clients.read().await.len()
    }

    /// Clean up stale client entries.
    ///
    /// Removes entries that haven't been seen within the cleanup interval.
    pub async fn cleanup(&self) {
        let now = Instant::now();
        let cleanup_interval = self.config.cleanup_interval;
        let mut clients = self.clients.write().await;
        clients.retain(|_, state| now.duration_since(state.last_seen) < cleanup_interval);
    }

    /// Clean up stale client entries with a custom staleness threshold.
    ///
    /// Used for testing to avoid sleep calls. Only available in test builds.
    #[cfg(test)]
    pub async fn cleanup_with_threshold(&self, threshold: Duration) {
        let now = Instant::now();
        let mut clients = self.clients.write().await;
        clients.retain(|_, state| now.duration_since(state.last_seen) < threshold);
    }

    /// Check if rate limiting is enabled.
    #[must_use]
    pub const fn is_enabled(&self) -> bool {
        self.enabled
    }

    /// Enable rate limiting.
    pub const fn enable(&mut self) {
        self.enabled = true;
    }

    /// Disable rate limiting.
    pub const fn disable(&mut self) {
        self.enabled = false;
    }
}

/// Rate limit error.
#[derive(Debug, Clone)]
pub struct RateLimitExceeded {
    /// The operation that was rate limited.
    pub operation: OperationType,
    /// Client that was rate limited.
    pub client: IpAddr,
}

impl std::fmt::Display for RateLimitExceeded {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Rate limit exceeded for {:?} operation from {}", self.operation, self.client)
    }
}

impl std::error::Error for RateLimitExceeded {}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;
    use std::net::Ipv4Addr;

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn test_rate_limiter_allows_within_limit() {
        let config = RateLimitConfig {
            read_rps: 10,
            write_rps: 5,
            expensive_rps: 2,
            burst_multiplier: 2,
            cleanup_interval: Duration::from_secs(60),
        };

        let limiter = RateLimiter::new(config);
        let client = IpAddr::V4(Ipv4Addr::LOCALHOST);

        // Should allow burst of requests
        for _ in 0..20 {
            assert!(limiter.check(client, OperationType::Read).await);
        }
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn test_rate_limiter_blocks_when_exceeded() {
        let config = RateLimitConfig {
            read_rps: 10,
            write_rps: 5,
            expensive_rps: 1,
            burst_multiplier: 1,
            cleanup_interval: Duration::from_secs(60),
        };

        let limiter = RateLimiter::new(config);
        let client = IpAddr::V4(Ipv4Addr::LOCALHOST);

        // Consume all tokens
        for _ in 0..10 {
            let _ = limiter.check(client, OperationType::Read).await;
        }

        // Next should be blocked
        assert!(!limiter.check(client, OperationType::Read).await);
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn test_rate_limiter_disabled() {
        let limiter = RateLimiter::disabled();
        let client = IpAddr::V4(Ipv4Addr::LOCALHOST);

        // Should always allow when disabled
        for _ in 0..1000 {
            assert!(limiter.check(client, OperationType::Expensive).await);
        }
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn test_rate_limiter_per_client() {
        let config = RateLimitConfig {
            read_rps: 5,
            write_rps: 5,
            expensive_rps: 5,
            burst_multiplier: 1,
            cleanup_interval: Duration::from_secs(60),
        };

        let limiter = RateLimiter::new(config);
        let client1 = IpAddr::V4(Ipv4Addr::LOCALHOST);
        let client2 = IpAddr::V4(Ipv4Addr::new(127, 0, 0, 2));

        // Exhaust client1's tokens
        for _ in 0..5 {
            let _ = limiter.check(client1, OperationType::Read).await;
        }

        // Client2 should still have tokens
        assert!(limiter.check(client2, OperationType::Read).await);
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn test_rate_limiter_cleanup() {
        let config = RateLimitConfig {
            read_rps: 10,
            write_rps: 5,
            expensive_rps: 2,
            burst_multiplier: 2,
            cleanup_interval: Duration::from_secs(3600), // Default cleanup interval
        };

        let limiter = RateLimiter::new(config);
        let client = IpAddr::V4(Ipv4Addr::LOCALHOST);

        // Create client entry
        limiter.check(client, OperationType::Read).await;
        assert_eq!(limiter.client_count().await, 1);

        // Use test-only cleanup with zero threshold (removes all entries instantly)
        limiter.cleanup_with_threshold(Duration::from_nanos(0)).await;

        // Client should be cleaned up (last_seen older than threshold)
        assert_eq!(limiter.client_count().await, 0, "stale client should be removed");
    }

    #[test]
    fn test_config_presets() {
        let prod = RateLimitConfig::production();
        assert_eq!(prod.read_rps, 500);

        let dev = RateLimitConfig::development();
        assert_eq!(dev.read_rps, 10000);
    }
}
