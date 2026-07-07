// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2024–2026 ecoPrimals Project

//! Tests for `socket_dir`, `socket_path_for_primal`, and XDG fallbacks.

use super::*;

#[test]
fn test_socket_dir_respects_xdg_runtime_dir() {
    let temp = tempfile::tempdir().expect("temp dir");
    let runtime_path = temp.path().to_path_buf();
    let path_str = runtime_path.to_str().unwrap();
    temp_env::with_vars([("XDG_RUNTIME_DIR", Some(path_str))], || {
        let result = socket_dir();
        if cfg!(target_os = "android") || cfg!(target_os = "windows") {
            assert!(result.is_none(), "Android/Windows should return None");
        } else {
            let dir = result.expect("Unix-like should return Some");
            assert_eq!(dir, runtime_path.join("biomeos"));
        }
    });
}

#[test]
fn test_socket_dir_fallback_without_xdg() {
    temp_env::with_vars([("XDG_RUNTIME_DIR", None::<&str>)], || {
        let result = socket_dir();
        if cfg!(target_os = "android") || cfg!(target_os = "windows") {
            assert!(result.is_none());
        } else if cfg!(target_os = "linux") {
            assert_eq!(result, Some(PathBuf::from(DEFAULT_SOCKET_DIR)));
        } else {
            assert_eq!(result, Some(std::env::temp_dir().join("biomeos")));
        }
    });
}

#[test]
fn test_socket_path_for_primal_unix_like() {
    if cfg!(target_os = "android") || cfg!(target_os = "windows") {
        let result = socket_path_for_primal("rhizoCrypt");
        assert!(result.is_none());
        return;
    }

    let temp = tempfile::tempdir().expect("temp dir");
    let path_str = temp.path().to_str().unwrap();
    temp_env::with_vars([("XDG_RUNTIME_DIR", Some(path_str))], || {
        let path = socket_path_for_primal("rhizoCrypt").expect("should return path");
        assert!(path.ends_with("rhizoCrypt.sock"));
        assert_eq!(path.file_name().unwrap(), "rhizoCrypt.sock");
    });
}

#[test]
fn test_socket_path_for_primal_uses_socket_extension() {
    if cfg!(target_os = "android") || cfg!(target_os = "windows") {
        return;
    }

    let path = socket_path_for_primal("testPrimal").expect("should return path");
    assert!(path.to_string_lossy().ends_with(".sock"));
}

#[test]
fn test_socket_dir_xdg_runtime_dir_empty_string() {
    if cfg!(target_os = "android") || cfg!(target_os = "windows") {
        return;
    }
    temp_env::with_vars([("XDG_RUNTIME_DIR", Some(""))], || {
        let result = socket_dir();
        let dir = result.expect("Empty XDG_RUNTIME_DIR still yields Some on Unix");
        assert_eq!(dir, PathBuf::from("biomeos"));
    });
}

#[test]
fn test_socket_path_for_primal_empty_name() {
    if cfg!(target_os = "android") || cfg!(target_os = "windows") {
        return;
    }
    let path = socket_path_for_primal("").expect("should return path");
    assert!(path.to_string_lossy().ends_with(".sock"));
    assert_eq!(path.file_name().unwrap(), ".sock");
}

#[test]
fn test_socket_path_for_primal_special_characters() {
    if cfg!(target_os = "android") || cfg!(target_os = "windows") {
        return;
    }
    let path = socket_path_for_primal("my-primal.v2").expect("should return path");
    assert!(path.to_string_lossy().ends_with("my-primal.v2.sock"));
    assert_eq!(path.file_name().unwrap(), "my-primal.v2.sock");
}

#[test]
fn test_socket_path_for_primal_none_on_unsupported_platform() {
    if !cfg!(target_os = "android") && !cfg!(target_os = "windows") {
        return;
    }
    assert!(socket_path_for_primal("rhizoCrypt").is_none());
    assert!(socket_path_for_primal("any").is_none());
}

#[test]
fn test_unix_socket_dir_fallback() {
    let dir = unix_socket_dir_fallback();
    if cfg!(target_os = "linux") {
        assert_eq!(dir, PathBuf::from(DEFAULT_SOCKET_DIR));
    } else {
        assert_eq!(dir, std::env::temp_dir().join("biomeos"));
    }
}
