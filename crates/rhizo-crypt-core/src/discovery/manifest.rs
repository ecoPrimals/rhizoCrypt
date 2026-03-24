// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2024–2026 ecoPrimals Project

//! Manifest-based capability discovery.
//!
//! Absorbed from toadStool S156 and barraCuda v0.3.5 manifest discovery pattern.
//! Scans `$XDG_RUNTIME_DIR/ecoPrimals/*.json` for primal capability manifests,
//! providing a file-system-based discovery fallback when Songbird is unavailable.
//!
//! ## Manifest Format
//!
//! Each primal publishes a JSON manifest at its XDG runtime path:
//!
//! ```json
//! {
//!     "primal": "rhizocrypt",
//!     "version": "0.14.0-dev",
//!     "socket": "/run/user/1000/ecoPrimals/rhizocrypt.sock",
//!     "capabilities": ["dag.session.create", "dag.event.append", "health.check"]
//! }
//! ```
//!
//! ## Discovery Flow
//!
//! 1. Resolve `$XDG_RUNTIME_DIR/ecoPrimals/`
//! 2. List all `*.json` files
//! 3. Parse each as a [`PrimalManifest`]
//! 4. Filter by requested capability
//! 5. Return matching socket paths

use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use tokio::fs;

/// A primal's capability manifest published to the filesystem.
///
/// Each primal writes this to `$XDG_RUNTIME_DIR/ecoPrimals/{primal}.json`
/// so that sibling primals can discover capabilities without a live
/// discovery service.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrimalManifest {
    /// Primal identifier (e.g., "rhizocrypt").
    pub primal: String,
    /// Primal version.
    #[serde(default)]
    pub version: String,
    /// Unix socket path for IPC.
    #[serde(default)]
    pub socket: String,
    /// TCP address (host:port) for network access.
    #[serde(default)]
    pub address: Option<String>,
    /// Fully qualified capabilities this primal provides.
    #[serde(default)]
    pub capabilities: Vec<String>,
}

impl PrimalManifest {
    /// Check if this manifest advertises a given capability.
    #[must_use]
    pub fn has_capability(&self, capability: &str) -> bool {
        self.capabilities.iter().any(|c| c == capability)
    }
}

/// Resolve the ecoPrimals manifest directory.
///
/// Returns `$XDG_RUNTIME_DIR/ecoPrimals/` if the env var is set, otherwise `None`.
#[must_use]
pub fn manifest_dir() -> Option<PathBuf> {
    std::env::var("XDG_RUNTIME_DIR").ok().map(|xdg| PathBuf::from(xdg).join("ecoPrimals"))
}

/// Scan the manifest directory for all primal manifests.
///
/// Skips files that fail to parse (graceful degradation).
pub async fn scan_manifests() -> Vec<PrimalManifest> {
    let Some(dir) = manifest_dir() else {
        return Vec::new();
    };

    scan_manifests_in(&dir).await
}

/// Scan a specific directory for primal manifests (testable).
pub async fn scan_manifests_in(dir: &Path) -> Vec<PrimalManifest> {
    let Ok(mut entries) = fs::read_dir(dir).await else {
        return Vec::new();
    };

    let mut manifests = Vec::new();
    while let Ok(Some(entry)) = entries.next_entry().await {
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("json") {
            continue;
        }
        let Ok(contents) = fs::read_to_string(&path).await else {
            continue;
        };
        if let Ok(manifest) = serde_json::from_str::<PrimalManifest>(&contents) {
            manifests.push(manifest);
        }
    }
    manifests
}

/// Find all manifests that advertise a specific capability.
pub async fn discover_by_capability(capability: &str) -> Vec<PrimalManifest> {
    scan_manifests().await.into_iter().filter(|m| m.has_capability(capability)).collect()
}

/// Write this primal's manifest to the manifest directory.
///
/// Creates the directory if it doesn't exist.
///
/// # Errors
///
/// Returns an error if the directory can't be created or the file can't be written.
pub async fn publish_manifest(manifest: &PrimalManifest) -> std::io::Result<PathBuf> {
    let Some(dir) = manifest_dir() else {
        return Err(std::io::Error::new(std::io::ErrorKind::NotFound, "XDG_RUNTIME_DIR not set"));
    };

    fs::create_dir_all(&dir).await?;
    let path = dir.join(format!("{}.json", manifest.primal));
    let json = serde_json::to_string_pretty(manifest)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e.to_string()))?;
    fs::write(&path, json).await?;
    Ok(path)
}

/// Remove this primal's manifest from the manifest directory.
///
/// Called during graceful shutdown. Ignores errors (best-effort cleanup).
pub async fn unpublish_manifest(primal_name: &str) {
    if let Some(dir) = manifest_dir() {
        let path = dir.join(format!("{primal_name}.json"));
        let _ = fs::remove_file(path).await;
    }
}

#[cfg(test)]
#[expect(clippy::unwrap_used, reason = "test code")]
mod tests {
    use super::*;

    #[test]
    fn manifest_has_capability() {
        let manifest = PrimalManifest {
            primal: "rhizocrypt".into(),
            version: "0.14.0-dev".into(),
            socket: "/tmp/test.sock".into(),
            address: None,
            capabilities: vec!["dag.session.create".into(), "health.check".into()],
        };

        assert!(manifest.has_capability("dag.session.create"));
        assert!(manifest.has_capability("health.check"));
        assert!(!manifest.has_capability("dag.event.append"));
    }

    #[tokio::test]
    async fn scan_manifests_in_empty_dir() {
        let dir = tempfile::tempdir().unwrap();
        let manifests = scan_manifests_in(dir.path()).await;
        assert!(manifests.is_empty());
    }

    #[tokio::test]
    async fn scan_manifests_in_with_valid_file() {
        let dir = tempfile::tempdir().unwrap();
        let manifest = PrimalManifest {
            primal: "testprimal".into(),
            version: "1.0.0".into(),
            socket: "/tmp/test.sock".into(),
            address: Some("127.0.0.1:9000".into()),
            capabilities: vec!["test.op".into()],
        };
        let json = serde_json::to_string(&manifest).unwrap();
        fs::write(dir.path().join("testprimal.json"), json).await.unwrap();

        let manifests = scan_manifests_in(dir.path()).await;
        assert_eq!(manifests.len(), 1);
        assert_eq!(manifests[0].primal, "testprimal");
        assert!(manifests[0].has_capability("test.op"));
    }

    #[tokio::test]
    async fn scan_manifests_in_skips_invalid_json() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join("bad.json"), "not valid json").await.unwrap();
        fs::write(dir.path().join("not_json.txt"), "{}").await.unwrap();

        let manifests = scan_manifests_in(dir.path()).await;
        assert!(manifests.is_empty());
    }

    #[tokio::test]
    async fn scan_manifests_in_multiple_primals() {
        let dir = tempfile::tempdir().unwrap();

        for name in &["primalA", "primalB", "primalC"] {
            let manifest = PrimalManifest {
                primal: (*name).into(),
                version: "1.0.0".into(),
                socket: format!("/tmp/{name}.sock"),
                address: None,
                capabilities: vec![format!("{name}.health")],
            };
            let json = serde_json::to_string(&manifest).unwrap();
            fs::write(dir.path().join(format!("{name}.json")), json).await.unwrap();
        }

        let manifests = scan_manifests_in(dir.path()).await;
        assert_eq!(manifests.len(), 3);
    }

    /// Test publish + unpublish using direct filesystem operations
    /// (bypasses `manifest_dir()` to avoid env mutation in async context).
    #[tokio::test]
    async fn publish_and_unpublish_manifest() {
        let dir = tempfile::tempdir().unwrap();
        let eco_dir = dir.path().join("ecoPrimals");
        fs::create_dir_all(&eco_dir).await.unwrap();

        let manifest = PrimalManifest {
            primal: "testprimal".into(),
            version: "1.0.0".into(),
            socket: "/tmp/test.sock".into(),
            address: None,
            capabilities: vec!["test.op".into()],
        };

        let json = serde_json::to_string_pretty(&manifest).unwrap();
        let path = eco_dir.join("testprimal.json");
        fs::write(&path, &json).await.unwrap();
        assert!(path.exists());

        let contents = fs::read_to_string(&path).await.unwrap();
        let read_back: PrimalManifest = serde_json::from_str(&contents).unwrap();
        assert_eq!(read_back.primal, "testprimal");

        fs::remove_file(&path).await.unwrap();
        assert!(!path.exists());
    }

    #[tokio::test]
    async fn discover_by_capability_filters() {
        let dir = tempfile::tempdir().unwrap();
        let eco_dir = dir.path();

        for (name, caps) in
            [("signer", vec!["crypto.sign"]), ("storer", vec!["storage.store", "storage.get"])]
        {
            let m = PrimalManifest {
                primal: name.into(),
                version: "1.0".into(),
                socket: format!("/tmp/{name}.sock"),
                address: None,
                capabilities: caps.into_iter().map(String::from).collect(),
            };
            let json = serde_json::to_string(&m).unwrap();
            fs::write(eco_dir.join(format!("{name}.json")), json).await.unwrap();
        }

        let all = scan_manifests_in(eco_dir).await;
        let signers: Vec<_> = all.iter().filter(|m| m.has_capability("crypto.sign")).collect();
        assert_eq!(signers.len(), 1);
        assert_eq!(signers[0].primal, "signer");

        assert_eq!(all.iter().filter(|m| m.has_capability("storage.store")).count(), 1);

        assert!(!all.iter().any(|m| m.has_capability("nonexistent")));
    }

    #[tokio::test]
    async fn scan_manifests_nonexistent_dir() {
        let manifests = scan_manifests_in(Path::new("/nonexistent/dir")).await;
        assert!(manifests.is_empty());
    }
}
