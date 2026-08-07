// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2024–2026 ecoPrimals Project

//! Capability-domain symlink management for UDS discovery.
//!
//! Uses G68 `platform_link()` instead of raw `std::os::unix::fs::symlink`.

use std::path::Path;
use tracing::debug;

/// Create a `{domain}.sock` symlink pointing to the primal socket.
///
/// Enables Tier 3 capability-domain discovery: consumers looking for
/// `dag.sock` will find the rhizoCrypt socket without knowing the
/// primal name (ecosystem standard v1.3.0).
pub(super) fn create_capability_symlink(socket_path: &Path) {
    #[cfg(unix)]
    if let Some(parent) = socket_path.parent() {
        let domain = rhizo_crypt_core::niche::DOMAIN;
        let symlink_path =
            parent.join(format!("{domain}{}", rhizo_crypt_core::constants::SOCKET_FILE_EXTENSION));
        if symlink_path.exists() {
            let _ = std::fs::remove_file(&symlink_path);
        }
        if let Err(e) = rhizo_crypt_core::platform_link(socket_path, &symlink_path) {
            debug!(
                symlink = %symlink_path.display(),
                target = %socket_path.display(),
                error = %e,
                "failed to create capability symlink (discovery degraded)"
            );
        } else {
            debug!(
                symlink = %symlink_path.display(),
                "capability domain symlink created"
            );
        }
    }
}

/// Remove the capability domain symlink if it points to our socket.
pub(super) fn remove_capability_symlink(socket_path: &Path) {
    #[cfg(unix)]
    if let Some(parent) = socket_path.parent() {
        let domain = rhizo_crypt_core::niche::DOMAIN;
        let symlink_path =
            parent.join(format!("{domain}{}", rhizo_crypt_core::constants::SOCKET_FILE_EXTENSION));
        if rhizo_crypt_core::is_symlink_to(&symlink_path, socket_path) {
            let _ = std::fs::remove_file(&symlink_path);
            debug!(symlink = %symlink_path.display(), "capability symlink removed");
        }
    }
}
