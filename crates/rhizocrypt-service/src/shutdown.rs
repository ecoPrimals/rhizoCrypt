// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2024–2026 ecoPrimals Project

//! Graceful shutdown signal handling.

use tracing::{debug, warn};

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
