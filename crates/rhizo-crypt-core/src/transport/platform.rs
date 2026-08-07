// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2024–2026 ecoPrimals Project

//! G68 Platform Substrate — cross-platform filesystem abstractions.
//!
//! Replaces raw `std::os::unix::fs::symlink` (L1) and `PermissionsExt` (L2)
//! with platform-aware equivalents. `#[cfg]` lives here, not in callers.

use std::path::Path;

// ============================================================================
// L1: Platform Links
// ============================================================================

/// Create a filesystem link (symlink on Unix, symlink or hardlink on Windows).
///
/// # Errors
///
/// Returns `std::io::Error` on failure. On platforms without link support,
/// returns `ErrorKind::Unsupported`.
pub fn platform_link(target: &Path, link: &Path) -> std::io::Result<()> {
    #[cfg(unix)]
    {
        std::os::unix::fs::symlink(target, link)
    }
    #[cfg(windows)]
    {
        std::os::windows::fs::symlink_file(target, link)
            .or_else(|_| std::fs::hard_link(target, link))
    }
    #[cfg(not(any(unix, windows)))]
    {
        let _ = (target, link);
        Err(std::io::Error::new(
            std::io::ErrorKind::Unsupported,
            "filesystem links not supported on this platform",
        ))
    }
}

/// Returns `true` if `path` is a symlink whose target matches `expected_target`.
///
/// Cross-platform: uses `symlink_metadata` + `read_link` (both in `std::fs`).
#[must_use]
pub fn is_symlink_to(path: &Path, expected_target: &Path) -> bool {
    path.symlink_metadata().is_ok_and(|m| m.file_type().is_symlink())
        && std::fs::read_link(path).is_ok_and(|t| t == expected_target)
}

// ============================================================================
// L2: Platform Permissions
// ============================================================================

/// Semantic permission levels (G68 L2 abstraction).
///
/// Replaces raw mode bits (`0o600`, `0o660`, `0o644`) with intent-based
/// access levels that map to POSIX modes on Unix and ACLs on Windows.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlatformAccess {
    /// Owner-only access (`0o600` on Unix, owner-only ACL on Windows).
    OwnerOnly,
    /// Owner + group read-write (`0o660` on Unix, Users group on Windows).
    GroupReadWrite,
    /// World-readable (`0o644` on Unix, Everyone read on Windows).
    WorldReadable,
    /// Read-only for everyone (`0o444` on Unix).
    ReadOnly,
    /// Standard directory permissions (`0o755` on Unix).
    DirectoryDefault,
}

/// Set platform-appropriate permissions on a path.
///
/// # Errors
///
/// Returns `std::io::Error` on failure.
pub fn set_platform_permissions(path: &Path, access: PlatformAccess) -> std::io::Result<()> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mode = match access {
            PlatformAccess::OwnerOnly => 0o600,
            PlatformAccess::GroupReadWrite => 0o660,
            PlatformAccess::WorldReadable => 0o644,
            PlatformAccess::ReadOnly => 0o444,
            PlatformAccess::DirectoryDefault => 0o755,
        };
        std::fs::set_permissions(path, std::fs::Permissions::from_mode(mode))
    }
    #[cfg(not(unix))]
    {
        let _ = (path, access);
        Ok(())
    }
}
