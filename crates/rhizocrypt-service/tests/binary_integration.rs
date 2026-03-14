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

fn service_binary_path() -> String {
    env!("CARGO_BIN_EXE_rhizocrypt").to_string()
}

fn server_command(binary_path: &str) -> Command {
    let mut cmd = Command::new(binary_path);
    cmd.arg("server");
    cmd
}

#[tokio::test]
async fn test_service_binary_exists() {
    let binary_path = service_binary_path();
    assert!(
        std::path::Path::new(&binary_path).exists(),
        "Service binary should exist at: {binary_path}",
    );
}

#[tokio::test]
async fn test_service_version_subcommand() {
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
    let binary_path = service_binary_path();

    let output =
        Command::new(&binary_path).arg("status").output().expect("Failed to run status subcommand");

    assert!(output.status.success(), "status subcommand should succeed");
}

#[tokio::test]
async fn test_service_doctor_subcommand() {
    let binary_path = service_binary_path();

    let output =
        Command::new(&binary_path).arg("doctor").output().expect("Failed to run doctor subcommand");

    assert!(output.status.success(), "doctor subcommand should succeed");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("rhizoCrypt Doctor"), "Should contain doctor header");
    assert!(stdout.contains("DAG engine"), "Should contain DAG engine check");
    assert!(stdout.contains("Overall:"), "Should contain overall status");
}

#[tokio::test]
async fn test_service_doctor_comprehensive_subcommand() {
    let binary_path = service_binary_path();

    let output = Command::new(&binary_path)
        .arg("doctor")
        .arg("--comprehensive")
        .output()
        .expect("Failed to run doctor --comprehensive subcommand");

    assert!(output.status.success(), "doctor --comprehensive should succeed");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("rhizoCrypt Doctor"), "Should contain doctor header");
    assert!(stdout.contains("Discovery service"), "Should contain discovery check");
}

#[tokio::test]
async fn test_service_starts_with_defaults() {
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
    let binary_path = service_binary_path();

    let mut child = server_command(&binary_path)
        .env("RHIZOCRYPT_PORT", "19410")
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
    let binary_path = service_binary_path();

    let mut child = Command::new(&binary_path)
        .args(["server", "--port", "19420"])
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
    let binary_path = service_binary_path();

    let mut child = server_command(&binary_path)
        .env("RHIZOCRYPT_PORT", "19430")
        .env("RUST_LOG", "info")
        .env_remove("SONGBIRD_ADDRESS")
        .env_remove("RHIZOCRYPT_DISCOVERY_ADAPTER")
        .env_remove("DISCOVERY_ENDPOINT")
        .env_remove("DISCOVERY_ADDRESS")
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
    let binary_path = service_binary_path();

    let mut child = server_command(&binary_path)
        .env("RHIZOCRYPT_PORT", "19440")
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
    let binary_path = service_binary_path();
    let test_port = "19450";

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
    let binary_path = service_binary_path();

    let mut child = server_command(&binary_path)
        .env("RHIZOCRYPT_PORT", "19460")
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
    let binary_path = service_binary_path();

    let mut child1 = server_command(&binary_path)
        .env("RHIZOCRYPT_PORT", "19470")
        .env("RUST_LOG", "error")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("Failed to start service 1");

    let mut child2 = server_command(&binary_path)
        .env("RHIZOCRYPT_PORT", "19480")
        .env("RUST_LOG", "error")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("Failed to start service 2");

    let mut child3 = server_command(&binary_path)
        .env("RHIZOCRYPT_PORT", "19490")
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

    let binary_path = service_binary_path();

    let mut child = server_command(&binary_path)
        .env("RHIZOCRYPT_PORT", "19500")
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
    let binary_path = service_binary_path();

    let mut child = server_command(&binary_path)
        .env("RHIZOCRYPT_PORT", "19510")
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
