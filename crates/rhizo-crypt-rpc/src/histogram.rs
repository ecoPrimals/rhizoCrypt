// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2024–2026 ecoPrimals Project

use std::sync::atomic::{AtomicU64, Ordering};

/// Histogram bucket boundaries for latency (in seconds).
pub const LATENCY_BUCKETS: [f64; 12] = [
    0.0001, // 100µs
    0.0005, // 500µs
    0.001,  // 1ms
    0.005,  // 5ms
    0.01,   // 10ms
    0.025,  // 25ms
    0.05,   // 50ms
    0.1,    // 100ms
    0.25,   // 250ms
    0.5,    // 500ms
    1.0,    // 1s
    5.0,    // 5s
];

/// Simple histogram for latency tracking.
#[derive(Debug)]
pub struct Histogram {
    /// Bucket counts.
    buckets: [AtomicU64; 12],
    /// Sum of all observations.
    sum: AtomicU64,
    /// Count of observations.
    count: AtomicU64,
}

impl Default for Histogram {
    fn default() -> Self {
        Self {
            buckets: std::array::from_fn(|_| AtomicU64::new(0)),
            sum: AtomicU64::new(0),
            count: AtomicU64::new(0),
        }
    }
}

impl Histogram {
    pub(crate) fn observe(&self, value: f64) {
        for (i, &bound) in LATENCY_BUCKETS.iter().enumerate() {
            if value <= bound {
                self.buckets[i].fetch_add(1, Ordering::Relaxed);
                break;
            }
        }

        // Integer microseconds for lock-free atomicity; negative/NaN → 0.
        let micros = value * 1_000_000.0;
        let value_micros = if micros.is_finite() && micros > 0.0 {
            // Truncation is acceptable: latencies exceeding u64::MAX µs (~584 millennia) are
            // not realistic, and sub-microsecond fractional loss is immaterial for observability.
            // Sign loss is guarded by the `> 0.0` predicate.
            #[expect(
                clippy::cast_possible_truncation,
                clippy::cast_sign_loss,
                reason = "truncation and sign loss are guarded by the > 0.0 predicate above"
            )]
            let v = micros as u64;
            v
        } else {
            0
        };
        self.sum.fetch_add(value_micros, Ordering::Relaxed);
        self.count.fetch_add(1, Ordering::Relaxed);
    }

    pub(crate) fn snapshot(&self) -> HistogramSnapshot {
        HistogramSnapshot {
            buckets: std::array::from_fn(|i| self.buckets[i].load(Ordering::Relaxed)),
            sum_micros: self.sum.load(Ordering::Relaxed),
            count: self.count.load(Ordering::Relaxed),
        }
    }
}

/// Snapshot of histogram data.
#[derive(Debug, Clone)]
pub struct HistogramSnapshot {
    /// Bucket counts.
    pub buckets: [u64; 12],
    /// Sum of observations in microseconds.
    pub sum_micros: u64,
    /// Total observation count.
    pub count: u64,
}

impl HistogramSnapshot {
    /// Get the bucket boundaries.
    #[must_use]
    pub const fn bucket_bounds() -> &'static [f64; 12] {
        &LATENCY_BUCKETS
    }

    /// Get the mean latency in seconds.
    ///
    /// Precision loss from `u64 → f64` is acceptable for observability data.
    #[must_use]
    pub fn mean_seconds(&self) -> f64 {
        if self.count == 0 {
            return 0.0;
        }
        // Precision loss: f64 mantissa is 53 bits; u64 values above 2^53 lose
        // low-order bits. For microsecond sums this corresponds to ~285 years of
        // accumulated latency — acceptable for observability.
        #[expect(
            clippy::cast_precision_loss,
            reason = "f64 precision loss only matters above 2^53 µs (~285 years of accumulated latency)"
        )]
        let mean = (self.sum_micros as f64 / 1_000_000.0) / self.count as f64;
        mean
    }
}
