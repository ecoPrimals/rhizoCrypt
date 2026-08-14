// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2024–2026 ecoPrimals Project

//! Graceful shutdown signal handling.

use tracing::debug;
#[cfg(unix)]
use tracing::warn;

#[cfg(test)]
pub static SIGNAL_TEST_MUTEX: tokio::sync::Mutex<()> = tokio::sync::Mutex::const_new(());

/// Wait for SIGTERM or SIGINT (Unix) or Ctrl+C (other platforms).
pub async fn shutdown_signal() {
    #[cfg(unix)]
    {
        use tokio::signal::unix::{SignalKind, signal};
        let Ok(mut sigterm) = signal(SignalKind::terminate()) else {
            warn!("Failed to register SIGTERM handler, falling back to ctrl_c");
            if let Err(e) = tokio::signal::ctrl_c().await {
                debug!(error = %e, "ctrl_c signal handler unavailable");
            }
            return;
        };
        let Ok(mut sigint) = signal(SignalKind::interrupt()) else {
            warn!("Failed to register SIGINT handler, falling back to ctrl_c");
            if let Err(e) = tokio::signal::ctrl_c().await {
                debug!(error = %e, "ctrl_c signal handler unavailable");
            }
            return;
        };
        tokio::select! {
            _ = sigterm.recv() => {}
            _ = sigint.recv() => {}
        }
    }

    #[cfg(not(unix))]
    {
        if let Err(e) = tokio::signal::ctrl_c().await {
            debug!(error = %e, "ctrl_c signal handler unavailable");
        }
    }
}

#[cfg(all(test, unix))]
#[expect(clippy::expect_used, reason = "test code")]
mod tests {
    use super::shutdown_signal;
    use nix::sys::signal::{Signal, kill};
    use nix::unistd::Pid;
    use std::time::Duration;
    use tokio::time::timeout;

    async fn assert_shutdown_on_signal(signal: Signal) {
        let _guard = super::SIGNAL_TEST_MUTEX.lock().await;
        let sender = tokio::task::spawn(async move {
            tokio::time::sleep(Duration::from_millis(50)).await;
            kill(Pid::this(), signal).expect("send signal to current process");
        });

        timeout(Duration::from_secs(2), shutdown_signal())
            .await
            .expect("shutdown_signal should return within timeout after signal");

        sender.await.expect("signal sender task completed");
    }

    #[tokio::test]
    async fn shutdown_signal_returns_on_sigterm() {
        assert_shutdown_on_signal(Signal::SIGTERM).await;
    }

    #[tokio::test]
    async fn shutdown_signal_returns_on_sigint() {
        assert_shutdown_on_signal(Signal::SIGINT).await;
    }
}
