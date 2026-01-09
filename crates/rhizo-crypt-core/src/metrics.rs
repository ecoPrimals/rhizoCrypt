//! Primal metrics tracking.
//!
//! Lock-free atomic counters for monitoring primal performance.

use std::sync::atomic::{AtomicU64, Ordering};

/// Atomic metrics counters for the primal.
///
/// All operations are lock-free using atomic operations for maximum concurrency.
#[derive(Debug, Default)]
pub struct PrimalMetrics {
    /// Sessions created.
    pub sessions_created: AtomicU64,
    /// Sessions resolved/committed.
    pub sessions_resolved: AtomicU64,
    /// Vertices appended.
    pub vertices_appended: AtomicU64,
    /// Queries executed.
    pub queries_executed: AtomicU64,
    /// Slices checked out.
    pub slices_checked_out: AtomicU64,
    /// Dehydrations completed.
    pub dehydrations_completed: AtomicU64,
}

impl PrimalMetrics {
    /// Create new metrics.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Increment sessions created.
    #[inline]
    pub fn inc_sessions_created(&self) {
        self.sessions_created.fetch_add(1, Ordering::Relaxed);
    }

    /// Increment sessions resolved.
    #[inline]
    pub fn inc_sessions_resolved(&self) {
        self.sessions_resolved.fetch_add(1, Ordering::Relaxed);
    }

    /// Increment vertices appended.
    #[inline]
    pub fn inc_vertices_appended(&self) {
        self.vertices_appended.fetch_add(1, Ordering::Relaxed);
    }

    /// Increment queries executed.
    #[inline]
    pub fn inc_queries_executed(&self) {
        self.queries_executed.fetch_add(1, Ordering::Relaxed);
    }

    /// Increment slices checked out.
    #[inline]
    pub fn inc_slices_checked_out(&self) {
        self.slices_checked_out.fetch_add(1, Ordering::Relaxed);
    }

    /// Increment dehydrations completed.
    #[inline]
    pub fn inc_dehydrations_completed(&self) {
        self.dehydrations_completed.fetch_add(1, Ordering::Relaxed);
    }

    /// Get sessions created count.
    #[inline]
    #[must_use]
    pub fn get_sessions_created(&self) -> u64 {
        self.sessions_created.load(Ordering::Relaxed)
    }

    /// Get sessions resolved count.
    #[inline]
    #[must_use]
    pub fn get_sessions_resolved(&self) -> u64 {
        self.sessions_resolved.load(Ordering::Relaxed)
    }

    /// Get vertices appended count.
    #[inline]
    #[must_use]
    pub fn get_vertices_appended(&self) -> u64 {
        self.vertices_appended.load(Ordering::Relaxed)
    }

    /// Get queries executed count.
    #[inline]
    #[must_use]
    pub fn get_queries_executed(&self) -> u64 {
        self.queries_executed.load(Ordering::Relaxed)
    }

    /// Get slices checked out count.
    #[inline]
    #[must_use]
    pub fn get_slices_checked_out(&self) -> u64 {
        self.slices_checked_out.load(Ordering::Relaxed)
    }

    /// Get dehydrations completed count.
    #[inline]
    #[must_use]
    pub fn get_dehydrations_completed(&self) -> u64 {
        self.dehydrations_completed.load(Ordering::Relaxed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_metrics_increment() {
        let metrics = PrimalMetrics::new();

        assert_eq!(metrics.get_sessions_created(), 0);
        metrics.inc_sessions_created();
        assert_eq!(metrics.get_sessions_created(), 1);

        metrics.inc_vertices_appended();
        metrics.inc_vertices_appended();
        assert_eq!(metrics.get_vertices_appended(), 2);
    }

    #[test]
    fn test_metrics_concurrent() {
        use std::sync::Arc;
        use std::thread;

        let metrics = Arc::new(PrimalMetrics::new());
        let mut handles = vec![];

        // Spawn 10 threads, each incrementing 100 times
        for _ in 0..10 {
            let m = Arc::clone(&metrics);
            let handle = thread::spawn(move || {
                for _ in 0..100 {
                    m.inc_sessions_created();
                }
            });
            handles.push(handle);
        }

        // Wait for all threads
        for handle in handles {
            handle.join().unwrap();
        }

        // Should have exactly 1000 increments
        assert_eq!(metrics.get_sessions_created(), 1000);
    }
}
