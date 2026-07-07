// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2024–2026 ecoPrimals Project

//! Tests for `read_family_id`, `is_biomeos_insecure`, `btsp_env_guard`, and
//! `family_scoped_socket_path`.

use super::*;

#[test]
fn test_read_family_id_from_ecosystem_var() {
    temp_env::with_vars(
        [("FAMILY_ID", Some("acme-prod")), ("RHIZOCRYPT_FAMILY_ID", None::<&str>)],
        || {
            let fid = read_family_id("RHIZOCRYPT");
            assert_eq!(fid.as_deref(), Some("acme-prod"));
        },
    );
}

#[test]
fn test_read_family_id_primal_override_takes_precedence() {
    temp_env::with_vars(
        [("FAMILY_ID", Some("eco-wide")), ("RHIZOCRYPT_FAMILY_ID", Some("primal-specific"))],
        || {
            let fid = read_family_id("RHIZOCRYPT");
            assert_eq!(fid.as_deref(), Some("primal-specific"));
        },
    );
}

#[test]
fn test_read_family_id_none_when_unset() {
    temp_env::with_vars(
        [("FAMILY_ID", None::<&str>), ("RHIZOCRYPT_FAMILY_ID", None::<&str>)],
        || {
            assert!(read_family_id("RHIZOCRYPT").is_none());
        },
    );
}

#[test]
fn test_read_family_id_default_treated_as_none() {
    temp_env::with_vars([("FAMILY_ID", Some("default"))], || {
        assert!(read_family_id("RHIZOCRYPT").is_none());
    });
}

#[test]
fn test_read_family_id_empty_treated_as_none() {
    temp_env::with_vars([("FAMILY_ID", Some(""))], || {
        assert!(read_family_id("RHIZOCRYPT").is_none());
    });
}

#[test]
fn test_read_family_id_whitespace_trimmed() {
    temp_env::with_vars([("FAMILY_ID", Some("  acme  "))], || {
        let fid = read_family_id("RHIZOCRYPT");
        assert_eq!(fid.as_deref(), Some("acme"));
    });
}

#[test]
fn test_is_biomeos_insecure_truthy_values() {
    for val in &["1", "true", "yes"] {
        temp_env::with_vars([("BIOMEOS_INSECURE", Some(*val))], || {
            assert!(is_biomeos_insecure(), "Expected insecure for '{val}'");
        });
    }
}

#[test]
fn test_is_biomeos_insecure_falsy_values() {
    for val in &["0", "false", "no", ""] {
        temp_env::with_vars([("BIOMEOS_INSECURE", Some(*val))], || {
            assert!(!is_biomeos_insecure(), "Expected secure for '{val}'");
        });
    }
    temp_env::with_vars([("BIOMEOS_INSECURE", None::<&str>)], || {
        assert!(!is_biomeos_insecure(), "Expected secure when unset");
    });
}

#[test]
fn test_btsp_guard_ok_production() {
    temp_env::with_vars(
        [("FAMILY_ID", Some("acme-prod")), ("BIOMEOS_INSECURE", None::<&str>)],
        || {
            assert!(btsp_env_guard("RHIZOCRYPT").is_ok());
        },
    );
}

#[test]
fn test_btsp_guard_ok_development() {
    temp_env::with_vars([("FAMILY_ID", None::<&str>), ("BIOMEOS_INSECURE", Some("1"))], || {
        assert!(btsp_env_guard("RHIZOCRYPT").is_ok());
    });
}

#[test]
fn test_btsp_guard_ok_neither_set() {
    temp_env::with_vars([("FAMILY_ID", None::<&str>), ("BIOMEOS_INSECURE", None::<&str>)], || {
        assert!(btsp_env_guard("RHIZOCRYPT").is_ok());
    });
}

#[test]
fn test_btsp_guard_rejects_conflict() {
    temp_env::with_vars(
        [("FAMILY_ID", Some("acme-prod")), ("BIOMEOS_INSECURE", Some("1"))],
        || {
            let result = btsp_env_guard("RHIZOCRYPT");
            assert_eq!(result, Err(BtspConfigError::FamilyInsecureConflict));
            assert!(result.unwrap_err().to_string().contains("BTSP conflict"));
        },
    );
}

#[test]
fn test_btsp_guard_default_family_not_conflict() {
    temp_env::with_vars([("FAMILY_ID", Some("default")), ("BIOMEOS_INSECURE", Some("1"))], || {
        assert!(btsp_env_guard("RHIZOCRYPT").is_ok(), "default is not a real FAMILY_ID");
    });
}

#[test]
fn test_family_scoped_socket_path_with_family() {
    if cfg!(target_os = "android") || cfg!(target_os = "windows") {
        return;
    }
    let temp = tempfile::tempdir().expect("temp dir");
    let path_str = temp.path().to_str().unwrap();
    temp_env::with_vars(
        [
            ("XDG_RUNTIME_DIR", Some(path_str)),
            ("FAMILY_ID", Some("acme-42")),
            ("RHIZOCRYPT_FAMILY_ID", None::<&str>),
        ],
        || {
            let path = family_scoped_socket_path("rhizocrypt", "RHIZOCRYPT").unwrap();
            assert!(
                path.to_string_lossy().ends_with("rhizocrypt-acme-42.sock"),
                "Expected family-scoped path, got: {path:?}"
            );
        },
    );
}

#[test]
fn test_family_scoped_socket_path_without_family() {
    if cfg!(target_os = "android") || cfg!(target_os = "windows") {
        return;
    }
    let temp = tempfile::tempdir().expect("temp dir");
    let path_str = temp.path().to_str().unwrap();
    temp_env::with_vars(
        [
            ("XDG_RUNTIME_DIR", Some(path_str)),
            ("FAMILY_ID", None::<&str>),
            ("RHIZOCRYPT_FAMILY_ID", None::<&str>),
        ],
        || {
            let path = family_scoped_socket_path("rhizocrypt", "RHIZOCRYPT").unwrap();
            assert!(
                path.to_string_lossy().ends_with("rhizocrypt.sock"),
                "Expected unscoped path, got: {path:?}"
            );
        },
    );
}

#[test]
fn test_family_scoped_socket_primal_override() {
    if cfg!(target_os = "android") || cfg!(target_os = "windows") {
        return;
    }
    let temp = tempfile::tempdir().expect("temp dir");
    let path_str = temp.path().to_str().unwrap();
    temp_env::with_vars(
        [
            ("XDG_RUNTIME_DIR", Some(path_str)),
            ("FAMILY_ID", Some("eco-wide")),
            ("RHIZOCRYPT_FAMILY_ID", Some("override-99")),
        ],
        || {
            let path = family_scoped_socket_path("rhizocrypt", "RHIZOCRYPT").unwrap();
            assert!(
                path.to_string_lossy().ends_with("rhizocrypt-override-99.sock"),
                "Primal-specific FAMILY_ID should take precedence, got: {path:?}"
            );
        },
    );
}
