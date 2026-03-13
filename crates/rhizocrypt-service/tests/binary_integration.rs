// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2024–2026 ecoPrimals Project

//! Integration tests for rhizocrypt binary (UniBin)
//!
//! Tests the main service entry point, configuration, startup, shutdown,
//! and error handling scenarios using the `server` subcommand.

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::cast_possible_wrap)]

use std::process::{Command, Stdio};
use std::time::Duration;
use tokio::time::sleep;

fn build_service() -> Result<(), Box<dyn std::error::Error>> {
    let output = Command::new("cargo").args(["build", "--bin", "rhizocrypt"]).output()?;

    if !output.status.success() {
        return Err(format!(
            "Failed to build service: {}",
            String::from_utf8_lossy(&output.stderr)
        )
        .into());
    }

    Ok(())
}

fn service_binary_path() -> String {
    let manifest_dir =
        std::env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR should be set");

    let workspace_root = std::path::Path::new(&manifest_dir)
        .parent()
        .and_then(|p| p.parent())
        .expect("Should have workspace root");

    let target_dir = std::env::var("CARGO_TARGET_DIR")
        .unwrap_or_else(|_| workspace_root.join("target").to_string_lossy().to_string());

    format!("{target_dir}/debug/rhizocrypt")
}

fn server_command(binary_path: &str) -> Command {
    let mut cmd = Command::new(binary_path);
    cmd.arg("server");
    cmd
}

#[tokio::test]
async fn test_service_binary_exists() {
    let binary_path = service_binary_path();

    if !std::path::Path::new(&binary_path).exists() {
        build_service().expect("Failed to build service binary");
    }

    assert!(
        std::path::Path::new(&binary_path).exists(),
        "Service binary should exist after build at: {binary_path}",
    );
}

#[tokio::test]
async fn test_service_version_subcommand() {
    build_service().expect("Failed to build service");
    let binary_path = service_binary_path();

    let output = Command::new(&binary_path)
        .arg("version")
        .output()
        .expect("Failed to run version subcommand");

    assert!(output.status.success(), "version subcommand should succeed");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("rhizoCrypt"), "Should contain project name");
    assert!(stdout.contains("AGPL-3.0"), "Should contain license");
}

#[tokio::test]
async fn test_service_status_subcommand() {
    build_service().expect("Failed to build service");
    let binary_path = service_binary_path();

    let output =
        Command::new(&binary_path).arg("status").output().expect("Failed to run status subcommand");

    assert!(output.status.success(), "status subcommand should succeed");
}

#[tokio::test]
async fn test_service_starts_with_defaults() {
    build_service().expect("Failed to build service");
    let binary_path = service_binary_path();

    let mut child = server_command(&binary_path)
        .env("RHIZOCRYPT_PORT", "19400")
        .env("RUST_LOG", "error")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("Failed to start service");

    sleep(Duration::from_millis(500)).await;

    match child.try_wait() {
        Ok(Some(status)) => panic!("Service exited unexpectedly with status: {status}"),
        Ok(None) => {}
        Err(e) => panic!("Error checking service status: {e}"),
    }

    let _ = child.kill();
    let _ = child.wait();
}

#[tokio::test]
async fn test_service_handles_invalid_port() {
    build_service().expect("Failed to build service");
    let binary_path = service_binary_path();

    let mut child = server_command(&binary_path)
        .env("RHIZOCRYPT_PORT", "99999")
        .env("RUST_LOG", "error")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("Failed to start service");

    let result = tokio::time::timeout(Duration::from_secs(5), async {
        loop {
            match child.try_wait() {
                Ok(Some(status)) => return Some(status),
                Ok(None) => sleep(Duration::from_millis(100)).await,
                Err(_) => return None,
            }
        }
    })
    .await;

    if let Ok(Some(status)) = result {
        assert!(!status.success(), "Service should fail with invalid port");
    } else {
        let _ = child.kill();
        let _ = child.wait();
    }
}

#[tokio::test]
async fn test_service_custom_configuration() {
    build_service().expect("Failed to build service");
    let binary_path = service_binary_path();

    let mut child = server_command(&binary_path)
        .env("RHIZOCRYPT_PORT", "19401")
        .env("RHIZOCRYPT_HOST", "127.0.0.1")
        .env("RUST_LOG", "error")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("Failed to start service");

    sleep(Duration::from_millis(500)).await;

    match child.try_wait() {
        Ok(Some(status)) => panic!("Service exited unexpectedly: {status}"),
        Ok(None) => {}
        Err(e) => panic!("Error checking service: {e}"),
    }

    let _ = child.kill();
    let _ = child.wait();
}

#[tokio::test]
async fn test_service_cli_port_override() {
    build_service().expect("Failed to build service");
    let binary_path = service_binary_path();

    let mut child = Command::new(&binary_path)
        .args(["server", "--port", "19411"])
        .env("RUST_LOG", "error")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("Failed to start service");

    sleep(Duration::from_millis(500)).await;

    match child.try_wait() {
        Ok(Some(status)) => panic!("Service exited unexpectedly: {status}"),
        Ok(None) => {}
        Err(e) => panic!("Error checking service: {e}"),
    }

    let _ = child.kill();
    let _ = child.wait();
}

#[tokio::test]
async fn test_service_without_discovery() {
    build_service().expect("Failed to build service");
    let binary_path = service_binary_path();

    let mut child = server_command(&binary_path)
        .env("RHIZOCRYPT_PORT", "19402")
        .env("RUST_LOG", "info")
        .env_remove("SONGBIRD_ADDRESS")
        .env_remove("RHIZOCRYPT_DISCOVERY_ADAPTER")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Failed to start service");

    sleep(Duration::from_millis(500)).await;

    match child.try_wait() {
        Ok(Some(status)) => panic!("Service should run without discovery, exited: {status}"),
        Ok(None) => {}
        Err(e) => panic!("Error: {e}"),
    }

    let _ = child.kill();
    let _ = child.wait();
}

#[tokio::test]
async fn test_service_graceful_shutdown_sigterm() {
    build_service().expect("Failed to build service");
    let binary_path = service_binary_path();

    let mut child = server_command(&binary_path)
        .env("RHIZOCRYPT_PORT", "19403")
        .env("RUST_LOG", "error")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("Failed to start service");

    sleep(Duration::from_millis(500)).await;

    #[cfg(unix)]
    {
        use nix::sys::signal::{kill, Signal};
        use nix::unistd::Pid;

        let pid = Pid::from_raw(child.id() as i32);
        let _ = kill(pid, Signal::SIGTERM);
    }

    #[cfg(not(unix))]
    {
        let _ = child.kill();
    }

    let wait_result = tokio::time::timeout(Duration::from_secs(5), async {
        loop {
            match child.try_wait() {
                Ok(Some(_)) => return Ok(()),
                Ok(None) => sleep(Duration::from_millis(100)).await,
                Err(e) => return Err(e),
            }
        }
    })
    .await;

    assert!(wait_result.is_ok(), "Service should shutdown gracefully within 5 seconds");
}

#[tokio::test]
async fn test_service_port_already_in_use() {
    build_service().expect("Failed to build service");
    let binary_path = service_binary_path();
    let test_port = "19404";

    let mut child1 = server_command(&binary_path)
        .env("RHIZOCRYPT_PORT", test_port)
        .env("RUST_LOG", "error")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("Failed to start first service");

    sleep(Duration::from_millis(500)).await;

    let mut child2 = server_command(&binary_path)
        .env("RHIZOCRYPT_PORT", test_port)
        .env("RUST_LOG", "error")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("Failed to start second service");

    let result = tokio::time::timeout(Duration::from_secs(5), async {
        loop {
            match child2.try_wait() {
                Ok(Some(status)) => return Some(status),
                Ok(None) => sleep(Duration::from_millis(100)).await,
                Err(_) => return None,
            }
        }
    })
    .await;

    if let Ok(Some(status)) = result {
        assert!(!status.success(), "Second service should fail when port is in use");
    } else {
        let _ = child2.kill();
        let _ = child2.wait();
    }

    let _ = child1.kill();
    let _ = child1.wait();
}

#[tokio::test]
async fn test_service_environment_variable_parsing() {
    build_service().expect("Failed to build service");
    let binary_path = service_binary_path();

    let mut child = server_command(&binary_path)
        .env("RHIZOCRYPT_PORT", "19405")
        .env("RHIZOCRYPT_HOST", "0.0.0.0")
        .env("RHIZOCRYPT_ENV", "development")
        .env("RUST_LOG", "error")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("Failed to start service");

    sleep(Duration::from_millis(500)).await;

    match child.try_wait() {
        Ok(Some(status)) => panic!("Service failed to parse env vars: {status}"),
        Ok(None) => {}
        Err(e) => panic!("Error: {e}"),
    }

    let _ = child.kill();
    let _ = child.wait();
}

#[tokio::test]
async fn test_service_multiple_instances_different_ports() {
    build_service().expect("Failed to build service");
    let binary_path = service_binary_path();

    let mut child1 = server_command(&binary_path)
        .env("RHIZOCRYPT_PORT", "19406")
        .env("RUST_LOG", "error")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("Failed to start service 1");

    let mut child2 = server_command(&binary_path)
        .env("RHIZOCRYPT_PORT", "19408")
        .env("RUST_LOG", "error")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("Failed to start service 2");

    let mut child3 = server_command(&binary_path)
        .env("RHIZOCRYPT_PORT", "19410")
        .env("RUST_LOG", "error")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("Failed to start service 3");

    sleep(Duration::from_secs(1)).await;

    assert!(child1.try_wait().unwrap().is_none(), "Service 1 should be running");
    assert!(child2.try_wait().unwrap().is_none(), "Service 2 should be running");
    assert!(child3.try_wait().unwrap().is_none(), "Service 3 should be running");

    let _ = child1.kill();
    let _ = child2.kill();
    let _ = child3.kill();
    let _ = child1.wait();
    let _ = child2.wait();
    let _ = child3.wait();
}

#[cfg(unix)]
#[tokio::test]
async fn test_service_signal_handling() {
    use nix::sys::signal::{kill, Signal};
    use nix::unistd::Pid;

    build_service().expect("Failed to build service");
    let binary_path = service_binary_path();

    let mut child = server_command(&binary_path)
        .env("RHIZOCRYPT_PORT", "19409")
        .env("RUST_LOG", "info")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("Failed to start service");

    sleep(Duration::from_millis(500)).await;

    let pid = Pid::from_raw(child.id() as i32);

    let result = kill(pid, Signal::SIGTERM);
    assert!(result.is_ok(), "Should be able to send SIGTERM");

    let shutdown_result = tokio::time::timeout(Duration::from_secs(5), async {
        loop {
            if let Ok(Some(_)) = child.try_wait() {
                return true;
            }
            sleep(Duration::from_millis(100)).await;
        }
    })
    .await;

    assert!(shutdown_result.is_ok(), "Service should shutdown gracefully on SIGTERM");
}

#[tokio::test]
async fn test_service_with_discovery_fallback() {
    build_service().expect("Failed to build service");
    let binary_path = service_binary_path();

    let mut child = server_command(&binary_path)
        .env("RHIZOCRYPT_PORT", "19410")
        .env("SONGBIRD_ADDRESS", "invalid.nonexistent:9999")
        .env("RUST_LOG", "info")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Failed to start service");

    sleep(Duration::from_secs(1)).await;

    match child.try_wait() {
        Ok(Some(status)) => {
            panic!("Service should continue running even if discovery fails: {status}");
        }
        Ok(None) => {}
        Err(e) => panic!("Error: {e}"),
    }

    let _ = child.kill();
    let _ = child.wait();
}
