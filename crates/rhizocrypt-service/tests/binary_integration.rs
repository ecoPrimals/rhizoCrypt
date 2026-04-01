// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2024–2026 ecoPrimals Project

//! Integration tests for rhizocrypt binary (`UniBin`)
//!
//! Tests the main service entry point, configuration, startup, shutdown,
//! and error handling scenarios using the `server` subcommand.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use rhizocrypt_service::exit_codes;
use std::process::{Child, Command, ExitStatus, Stdio};
use std::time::Duration;

fn service_binary_path() -> String {
    env!("CARGO_BIN_EXE_rhizocrypt").to_string()
}

fn server_command(binary_path: &str) -> Command {
    let mut cmd = Command::new(binary_path);
    cmd.arg("server");
    cmd
}

/// Probe a TCP port until it accepts connections or the timeout expires.
/// Returns `Ok(())` on successful connect, `Err` on timeout.
async fn wait_for_tcp_ready(port: u16, timeout: Duration) -> Result<(), &'static str> {
    let addr = format!("127.0.0.1:{port}");
    tokio::time::timeout(timeout, async {
        loop {
            if tokio::net::TcpStream::connect(&addr).await.is_ok() {
                return;
            }
            tokio::task::yield_now().await;
        }
    })
    .await
    .map_err(|_| "server did not become ready within timeout")
}

/// Poll a child process for exit within a timeout, yielding between checks.
async fn wait_for_exit(child: &mut Child, timeout: Duration) -> Option<ExitStatus> {
    tokio::time::timeout(timeout, async {
        loop {
            match child.try_wait() {
                Ok(Some(status)) => return Some(status),
                Ok(None) => tokio::task::yield_now().await,
                Err(_) => return None,
            }
        }
    })
    .await
    .unwrap_or(None)
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
async fn test_client_invalid_address_exit_code() {
    let binary_path = service_binary_path();

    let output = Command::new(&binary_path)
        .args(["client", "--address", "not-a-valid-address", "health"])
        .output()
        .expect("Failed to run client with invalid address");

    assert!(!output.status.success(), "Should fail with invalid address");
    let code = output.status.code().expect("process should have exit code");
    assert_eq!(code, exit_codes::CONFIG_ERROR, "AddrParse should map to CONFIG_ERROR (2)");
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

    wait_for_tcp_ready(19400, Duration::from_secs(5)).await.unwrap();

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

    let result = wait_for_exit(&mut child, Duration::from_secs(5));

    if let Some(status) = result.await {
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

    wait_for_tcp_ready(19410, Duration::from_secs(5)).await.unwrap();

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

    wait_for_tcp_ready(19420, Duration::from_secs(5)).await.unwrap();

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

    wait_for_tcp_ready(19430, Duration::from_secs(5)).await.unwrap();

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

    wait_for_tcp_ready(19440, Duration::from_secs(5)).await.unwrap();

    #[cfg(unix)]
    {
        use nix::sys::signal::{Signal, kill};
        use nix::unistd::Pid;

        let raw_pid = i32::try_from(child.id()).expect("pid fits in i32");
        let pid = Pid::from_raw(raw_pid);
        let _ = kill(pid, Signal::SIGTERM);
    }

    #[cfg(not(unix))]
    {
        let _ = child.kill();
    }

    let status = wait_for_exit(&mut child, Duration::from_secs(5)).await;
    assert!(status.is_some(), "Service should shutdown gracefully within 5 seconds");
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

    wait_for_tcp_ready(19450, Duration::from_secs(5)).await.unwrap();

    let mut child2 = server_command(&binary_path)
        .env("RHIZOCRYPT_PORT", test_port)
        .env("RUST_LOG", "error")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("Failed to start second service");

    let result = wait_for_exit(&mut child2, Duration::from_secs(5)).await;

    if let Some(status) = result {
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

    wait_for_tcp_ready(19460, Duration::from_secs(5)).await.unwrap();

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

    wait_for_tcp_ready(19470, Duration::from_secs(5)).await.unwrap();
    wait_for_tcp_ready(19480, Duration::from_secs(5)).await.unwrap();
    wait_for_tcp_ready(19490, Duration::from_secs(5)).await.unwrap();

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
    use nix::sys::signal::{Signal, kill};
    use nix::unistd::Pid;

    let binary_path = service_binary_path();

    let mut child = server_command(&binary_path)
        .env("RHIZOCRYPT_PORT", "19500")
        .env("RUST_LOG", "info")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("Failed to start service");

    wait_for_tcp_ready(19500, Duration::from_secs(5)).await.unwrap();

    let raw_pid = i32::try_from(child.id()).expect("pid fits in i32");
    let pid = Pid::from_raw(raw_pid);

    let result = kill(pid, Signal::SIGTERM);
    assert!(result.is_ok(), "Should be able to send SIGTERM");

    let status = wait_for_exit(&mut child, Duration::from_secs(5)).await;
    assert!(status.is_some(), "Service should shutdown gracefully on SIGTERM");
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

    wait_for_tcp_ready(19510, Duration::from_secs(10)).await.unwrap();

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
