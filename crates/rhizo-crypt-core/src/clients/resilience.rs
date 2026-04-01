// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2024–2026 ecoPrimals Project

//! IPC resilience primitives — circuit breaker and retry policy.
//!
//! Absorbed from healthSpring V28 and airSpring V15 resilience patterns.
//! These types are transport-agnostic: they work with Unix sockets, TCP,
//! or any future IPC mechanism.
//!
//! ## Circuit Breaker
//!
//! Prevents cascading failures when a sibling primal becomes unresponsive.
//! After `failure_threshold` consecutive transport errors the breaker opens,
//! fast-failing all calls for a cooldown window before probing again.
//!
//! ## Retry Policy
//!
//! Exponential backoff with jitter for transient transport errors. Only
//! transport-level (`IpcErrorPhase::is_retriable`) failures are retried;
//! application errors are returned immediately.

use crate::error::IpcErrorPhase;
use std::sync::atomic::{AtomicU8, AtomicU32, AtomicU64, Ordering};
use std::time::{Duration, Instant};

/// Circuit breaker state.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BreakerState {
    /// Normal operation — requests pass through.
    Closed,
    /// Fault threshold exceeded — requests are fast-failed.
    Open,
    /// Cooldown elapsed — a single probe request is allowed through.
    HalfOpen,
}

/// Lock-free, transport-agnostic circuit breaker for IPC clients.
///
/// All state is stored in atomics — no `Mutex`, no `await` in the hot path.
/// `last_failure_nanos` stores the `Instant` as nanoseconds elapsed from a
/// process-local epoch (captured once at construction). Zero means "no failure
/// recorded yet".
#[derive(Debug)]
pub struct CircuitBreaker {
    failure_threshold: u8,
    cooldown: Duration,
    consecutive_failures: AtomicU8,
    last_failure_nanos: AtomicU64,
    total_trips: AtomicU32,
    epoch: Instant,
}

impl CircuitBreaker {
    /// Create a new circuit breaker.
    ///
    /// - `failure_threshold`: consecutive failures before opening (1..=255)
    /// - `cooldown`: duration to wait before probing after open
    #[must_use]
    pub fn new(failure_threshold: u8, cooldown: Duration) -> Self {
        Self {
            failure_threshold: failure_threshold.max(1),
            cooldown,
            consecutive_failures: AtomicU8::new(0),
            last_failure_nanos: AtomicU64::new(0),
            total_trips: AtomicU32::new(0),
            epoch: Instant::now(),
        }
    }

    /// Create with defaults: 5 failures, 30s cooldown.
    #[must_use]
    pub fn default_ipc() -> Self {
        Self::new(5, Duration::from_secs(30))
    }

    /// Current breaker state (lock-free, no await).
    #[must_use]
    pub fn state(&self) -> BreakerState {
        let failures = self.consecutive_failures.load(Ordering::Acquire);
        if failures < self.failure_threshold {
            return BreakerState::Closed;
        }

        let last_nanos = self.last_failure_nanos.load(Ordering::Acquire);
        if last_nanos == 0 {
            return BreakerState::Open;
        }

        let last = self.epoch + Duration::from_nanos(last_nanos);
        if last.elapsed() >= self.cooldown {
            BreakerState::HalfOpen
        } else {
            BreakerState::Open
        }
    }

    /// Check whether a request should be allowed through (lock-free).
    #[must_use]
    pub fn allow_request(&self) -> bool {
        !matches!(self.state(), BreakerState::Open)
    }

    /// Record a successful call — resets consecutive failure count.
    pub fn record_success(&self) {
        self.consecutive_failures.store(0, Ordering::Release);
    }

    /// Record a transport failure. Only retriable phases trip the breaker.
    pub fn record_failure(&self, phase: &IpcErrorPhase) {
        if !phase.is_retriable() {
            return;
        }
        let prev = self.consecutive_failures.fetch_add(1, Ordering::AcqRel);
        if prev + 1 >= self.failure_threshold {
            self.total_trips.fetch_add(1, Ordering::Relaxed);
        }
        let nanos = self.epoch.elapsed().as_nanos();
        let nanos_u64 = u64::try_from(nanos).unwrap_or(u64::MAX);
        self.last_failure_nanos.store(nanos_u64, Ordering::Release);
    }

    /// Total number of times the breaker has tripped since creation.
    #[must_use]
    pub fn total_trips(&self) -> u32 {
        self.total_trips.load(Ordering::Relaxed)
    }

    /// Current consecutive failure count.
    #[must_use]
    pub fn consecutive_failures(&self) -> u8 {
        self.consecutive_failures.load(Ordering::Relaxed)
    }
}

/// Retry policy with exponential backoff and jitter.
///
/// Only retries transport-level failures (`IpcErrorPhase::is_retriable`).
/// Application errors (e.g., `JsonRpcError`, `NoResult`) are never retried.
#[derive(Debug, Clone)]
pub struct RetryPolicy {
    /// Maximum number of retry attempts (0 = no retries).
    pub max_retries: u8,
    /// Base backoff duration (doubled each attempt).
    pub base_backoff: Duration,
    /// Maximum backoff cap.
    pub max_backoff: Duration,
}

impl RetryPolicy {
    /// Create a new retry policy.
    #[must_use]
    pub const fn new(max_retries: u8, base_backoff: Duration, max_backoff: Duration) -> Self {
        Self {
            max_retries,
            base_backoff,
            max_backoff,
        }
    }

    /// Default IPC retry policy: 3 retries, 100ms base, 2s cap.
    #[must_use]
    pub const fn default_ipc() -> Self {
        Self::new(
            crate::constants::DEFAULT_MAX_RETRIES,
            Duration::from_millis(crate::constants::DEFAULT_RETRY_BACKOFF_MS),
            Duration::from_secs(2),
        )
    }

    /// No retries.
    #[must_use]
    pub const fn none() -> Self {
        Self::new(0, Duration::ZERO, Duration::ZERO)
    }

    /// Whether the given phase and attempt number should be retried.
    #[must_use]
    pub const fn should_retry(&self, phase: &IpcErrorPhase, attempt: u8) -> bool {
        attempt < self.max_retries && phase.is_retriable()
    }

    /// Calculate the backoff duration for a given attempt (0-indexed).
    ///
    /// Uses exponential backoff: `base * 2^attempt`, capped at `max_backoff`.
    /// Jitter is left to the caller (simple random or `tokio::time::sleep`).
    #[must_use]
    pub fn backoff_for(&self, attempt: u8) -> Duration {
        let multiplier = 1u64.checked_shl(u32::from(attempt)).unwrap_or(u64::MAX);
        let backoff =
            self.base_backoff.saturating_mul(u32::try_from(multiplier).unwrap_or(u32::MAX));
        backoff.min(self.max_backoff)
    }
}

impl Default for RetryPolicy {
    fn default() -> Self {
        Self::default_ipc()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn breaker_starts_closed() {
        let cb = CircuitBreaker::new(3, Duration::from_secs(1));
        assert_eq!(cb.state(), BreakerState::Closed);
        assert!(cb.allow_request());
    }

    #[test]
    fn breaker_opens_after_threshold() {
        let cb = CircuitBreaker::new(2, Duration::from_secs(60));

        cb.record_failure(&IpcErrorPhase::Connect);
        assert_eq!(cb.state(), BreakerState::Closed);

        cb.record_failure(&IpcErrorPhase::Connect);
        assert_eq!(cb.state(), BreakerState::Open);
        assert!(!cb.allow_request());
        assert_eq!(cb.total_trips(), 1);
    }

    #[test]
    fn breaker_resets_on_success() {
        let cb = CircuitBreaker::new(2, Duration::from_secs(60));

        cb.record_failure(&IpcErrorPhase::Connect);
        cb.record_success();
        assert_eq!(cb.consecutive_failures(), 0);
        assert_eq!(cb.state(), BreakerState::Closed);
    }

    #[test]
    fn breaker_ignores_application_errors() {
        let cb = CircuitBreaker::new(2, Duration::from_secs(60));

        cb.record_failure(&IpcErrorPhase::JsonRpcError(-32601));
        cb.record_failure(&IpcErrorPhase::NoResult);
        cb.record_failure(&IpcErrorPhase::InvalidJson);
        assert_eq!(cb.consecutive_failures(), 0);
        assert_eq!(cb.state(), BreakerState::Closed);
    }

    #[test]
    fn breaker_transitions_to_half_open() {
        let cb = CircuitBreaker::new(1, Duration::ZERO);

        cb.record_failure(&IpcErrorPhase::Connect);
        assert_eq!(cb.state(), BreakerState::HalfOpen);
        assert!(cb.allow_request());
    }

    #[test]
    fn retry_policy_should_retry_transport_errors() {
        let policy = RetryPolicy::default_ipc();
        assert!(policy.should_retry(&IpcErrorPhase::Connect, 0));
        assert!(policy.should_retry(&IpcErrorPhase::Write, 1));
        assert!(policy.should_retry(&IpcErrorPhase::Read, 2));
        assert!(!policy.should_retry(&IpcErrorPhase::Connect, 3));
    }

    #[test]
    fn retry_policy_never_retries_application_errors() {
        let policy = RetryPolicy::default_ipc();
        assert!(!policy.should_retry(&IpcErrorPhase::JsonRpcError(-32601), 0));
        assert!(!policy.should_retry(&IpcErrorPhase::NoResult, 0));
        assert!(!policy.should_retry(&IpcErrorPhase::InvalidJson, 0));
        assert!(!policy.should_retry(&IpcErrorPhase::HttpStatus(500), 0));
    }

    #[test]
    fn retry_policy_none_never_retries() {
        let policy = RetryPolicy::none();
        assert!(!policy.should_retry(&IpcErrorPhase::Connect, 0));
    }

    #[test]
    fn retry_backoff_exponential() {
        let policy = RetryPolicy::new(5, Duration::from_millis(100), Duration::from_secs(2));
        assert_eq!(policy.backoff_for(0), Duration::from_millis(100));
        assert_eq!(policy.backoff_for(1), Duration::from_millis(200));
        assert_eq!(policy.backoff_for(2), Duration::from_millis(400));
        assert_eq!(policy.backoff_for(3), Duration::from_millis(800));
        assert_eq!(policy.backoff_for(4), Duration::from_millis(1600));
        assert_eq!(policy.backoff_for(5), Duration::from_secs(2));
    }

    #[test]
    fn retry_backoff_caps_at_max() {
        let policy = RetryPolicy::new(10, Duration::from_millis(100), Duration::from_millis(500));
        assert_eq!(policy.backoff_for(10), Duration::from_millis(500));
        assert_eq!(policy.backoff_for(20), Duration::from_millis(500));
    }
}
