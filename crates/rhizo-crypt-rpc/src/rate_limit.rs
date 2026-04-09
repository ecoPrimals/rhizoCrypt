// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2024–2026 ecoPrimals Project

//! Rate limiting infrastructure for RPC endpoints.
//!
//! Provides token bucket rate limiting per client, with configurable
//! limits for different operation types.

use dashmap::DashMap;
use rhizo_crypt_core::constants;
use std::net::IpAddr;
use std::sync::Arc;
use std::time::Duration;

use tokio::time::Instant;

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
            cleanup_interval: constants::RATE_LIMIT_CLEANUP_INTERVAL,
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
            cleanup_interval: constants::RATE_LIMIT_CLEANUP_INTERVAL,
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
            cleanup_interval: constants::RATE_LIMIT_CLEANUP_INTERVAL_DEV,
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

        if let Ok(val) = std::env::var("RHIZOCRYPT_RATE_LIMIT_READ_RPS")
            && let Ok(rps) = val.parse()
        {
            config.read_rps = rps;
        }

        if let Ok(val) = std::env::var("RHIZOCRYPT_RATE_LIMIT_WRITE_RPS")
            && let Ok(rps) = val.parse()
        {
            config.write_rps = rps;
        }

        if let Ok(val) = std::env::var("RHIZOCRYPT_RATE_LIMIT_EXPENSIVE_RPS")
            && let Ok(rps) = val.parse()
        {
            config.expensive_rps = rps;
        }

        config
    }
}

/// Operation type for rate limiting.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum OperationType {
    /// Read operations (`get_vertex`, `get_frontier`, etc.).
    Read,
    /// Write operations (`append_event`, `create_session`, etc.).
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
    clients: Arc<DashMap<IpAddr, ClientState>>,
    /// Whether rate limiting is enabled.
    enabled: bool,
}

impl RateLimiter {
    /// Create a new rate limiter with the given configuration.
    #[must_use]
    pub fn new(config: RateLimitConfig) -> Self {
        Self {
            config,
            clients: Arc::new(DashMap::new()),
            enabled: true,
        }
    }

    /// Create a rate limiter that allows everything (for testing).
    #[must_use]
    pub fn disabled() -> Self {
        Self {
            config: RateLimitConfig::default(),
            clients: Arc::new(DashMap::new()),
            enabled: false,
        }
    }

    /// Check if an operation is allowed for a client.
    ///
    /// Returns `true` if the operation is allowed, `false` if rate limited.
    #[must_use]
    pub fn check(&self, client: IpAddr, op: OperationType) -> bool {
        if !self.enabled {
            return true;
        }

        self.clients.entry(client).or_insert_with(|| ClientState::new(&self.config)).try_consume(op)
    }

    /// Get the number of tracked clients.
    #[must_use]
    pub fn client_count(&self) -> usize {
        self.clients.len()
    }

    /// Clean up stale client entries.
    ///
    /// Removes entries that haven't been seen within the cleanup interval.
    pub fn cleanup(&self) {
        let now = Instant::now();
        let cleanup_interval = self.config.cleanup_interval;
        self.clients.retain(|_, state| now.duration_since(state.last_seen) < cleanup_interval);
    }

    /// Clean up stale client entries with a custom staleness threshold.
    #[cfg(test)]
    pub fn cleanup_with_threshold(&self, threshold: Duration) {
        let now = Instant::now();
        self.clients.retain(|_, state| now.duration_since(state.last_seen) < threshold);
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
#[derive(Debug, Clone, thiserror::Error)]
#[error("Rate limit exceeded for {operation:?} operation from {client}")]
pub struct RateLimitExceeded {
    /// The operation that was rate limited.
    pub operation: OperationType,
    /// Client that was rate limited.
    pub client: IpAddr,
}

#[cfg(test)]
#[path = "rate_limit_tests.rs"]
mod tests;
