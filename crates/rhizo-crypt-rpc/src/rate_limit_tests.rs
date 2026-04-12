// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2024–2026 ecoPrimals Project

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
        assert!(limiter.check(client, OperationType::Read));
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
        let _ = limiter.check(client, OperationType::Read);
    }

    // Next should be blocked
    assert!(!limiter.check(client, OperationType::Read));
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_rate_limiter_disabled() {
    let limiter = RateLimiter::disabled();
    let client = IpAddr::V4(Ipv4Addr::LOCALHOST);

    // Should always allow when disabled
    for _ in 0..1000 {
        assert!(limiter.check(client, OperationType::Expensive));
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
        let _ = limiter.check(client1, OperationType::Read);
    }

    // Client2 should still have tokens
    assert!(limiter.check(client2, OperationType::Read));
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
    let _ = limiter.check(client, OperationType::Read);
    assert_eq!(limiter.client_count(), 1);

    // Use test-only cleanup with zero threshold (removes all entries instantly)
    limiter.cleanup_with_threshold(Duration::from_nanos(0));

    // Client should be cleaned up (last_seen older than threshold)
    assert_eq!(limiter.client_count(), 0, "stale client should be removed");
}

#[test]
fn test_config_presets() {
    let prod = RateLimitConfig::production();
    assert_eq!(prod.read_rps, 500);

    let dev = RateLimitConfig::development();
    assert_eq!(dev.read_rps, 10000);
}

#[test]
fn test_config_default() {
    let config = RateLimitConfig::default();
    assert_eq!(config.read_rps, 1000);
    assert_eq!(config.write_rps, 100);
    assert_eq!(config.expensive_rps, 10);
    assert_eq!(config.burst_multiplier, 2);
}

#[test]
fn test_config_from_env() {
    temp_env::with_vars(
        [
            ("RHIZOCRYPT_RATE_LIMIT_READ_RPS", Some("42")),
            ("RHIZOCRYPT_RATE_LIMIT_WRITE_RPS", Some("21")),
            ("RHIZOCRYPT_RATE_LIMIT_EXPENSIVE_RPS", Some("7")),
        ],
        || {
            let config = RateLimitConfig::from_env();
            assert_eq!(config.read_rps, 42);
            assert_eq!(config.write_rps, 21);
            assert_eq!(config.expensive_rps, 7);
        },
    );
}

#[test]
fn test_config_from_env_invalid_ignored() {
    temp_env::with_vars([("RHIZOCRYPT_RATE_LIMIT_READ_RPS", Some("not-a-number"))], || {
        let config = RateLimitConfig::from_env();
        assert_eq!(config.read_rps, 1000);
    });
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_rate_limiter_very_short_window() {
    let config = RateLimitConfig {
        read_rps: 1,
        write_rps: 1,
        expensive_rps: 1,
        burst_multiplier: 1,
        cleanup_interval: Duration::from_secs(60),
    };

    let limiter = RateLimiter::new(config);
    let client = IpAddr::V4(Ipv4Addr::LOCALHOST);

    assert!(limiter.check(client, OperationType::Read));
    assert!(!limiter.check(client, OperationType::Read));
}

#[tokio::test(start_paused = true)]
async fn test_rate_limiter_reset_after_window() {
    let config = RateLimitConfig {
        read_rps: 1,
        write_rps: 1,
        expensive_rps: 1,
        burst_multiplier: 1,
        cleanup_interval: Duration::from_secs(60),
    };

    let limiter = RateLimiter::new(config);
    let client = IpAddr::V4(Ipv4Addr::LOCALHOST);

    assert!(limiter.check(client, OperationType::Read));
    assert!(!limiter.check(client, OperationType::Read));

    tokio::time::advance(Duration::from_secs(2)).await;

    assert!(limiter.check(client, OperationType::Read));
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_rate_limiter_write_and_expensive() {
    let config = RateLimitConfig {
        read_rps: 100,
        write_rps: 2,
        expensive_rps: 1,
        burst_multiplier: 1,
        cleanup_interval: Duration::from_secs(60),
    };

    let limiter = RateLimiter::new(config);
    let client = IpAddr::V4(Ipv4Addr::LOCALHOST);

    assert!(limiter.check(client, OperationType::Write));
    assert!(limiter.check(client, OperationType::Write));
    assert!(!limiter.check(client, OperationType::Write));

    assert!(limiter.check(client, OperationType::Expensive));
    assert!(!limiter.check(client, OperationType::Expensive));
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_rate_limiter_enable_disable() {
    let mut limiter = RateLimiter::disabled();
    assert!(!limiter.is_enabled());

    limiter.enable();
    assert!(limiter.is_enabled());

    limiter.disable();
    assert!(!limiter.is_enabled());

    let client = IpAddr::V4(Ipv4Addr::LOCALHOST);
    assert!(limiter.check(client, OperationType::Expensive));
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_rate_limiter_regular_cleanup() {
    let config = RateLimitConfig {
        read_rps: 10,
        write_rps: 5,
        expensive_rps: 2,
        burst_multiplier: 2,
        cleanup_interval: Duration::from_secs(1),
    };

    let limiter = RateLimiter::new(config);
    let client = IpAddr::V4(Ipv4Addr::LOCALHOST);

    let _ = limiter.check(client, OperationType::Read);
    assert_eq!(limiter.client_count(), 1);

    limiter.cleanup();
    assert_eq!(limiter.client_count(), 1);

    limiter.cleanup_with_threshold(Duration::from_nanos(0));
    assert_eq!(limiter.client_count(), 0);
}

#[test]
fn test_rate_limit_exceeded_display() {
    let err = RateLimitExceeded {
        operation: OperationType::Write,
        client: IpAddr::V4(Ipv4Addr::LOCALHOST),
    };
    let s = err.to_string();
    assert!(s.contains("Write"));
    assert!(s.contains("127.0.0.1"));
}

#[test]
fn test_rate_limit_exceeded_error() {
    use std::error::Error;
    let err = RateLimitExceeded {
        operation: OperationType::Read,
        client: IpAddr::V4(Ipv4Addr::new(192, 168, 1, 1)),
    };
    assert!(err.source().is_none());
}
