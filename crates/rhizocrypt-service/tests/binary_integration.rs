//! Integration tests for rhizocrypt-service binary
//!
//! Tests the main service entry point, configuration, startup, shutdown,
//! and error handling scenarios.

use rhizo_crypt_core::safe_env::SafeEnv;
use std::process::{Command, Stdio};
use std::time::Duration;
use tokio::time::sleep;

/// Helper to build the service binary
fn build_service() -> Result<(), Box<dyn std::error::Error>> {
    let output = Command::new("cargo")
        .args(["build", "--bin", "rhizocrypt-service"])
        .output()?;

    if !output.status.success() {
        return Err(format!(
            "Failed to build service: {}",
            String::from_utf8_lossy(&output.stderr)
        )
        .into());
    }

    Ok(())
}

/// Helper to get path to service binary
fn service_binary_path() -> String {
    // Tests run from workspace root, so we need absolute path or workspace-relative path
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR")
        .expect("CARGO_MANIFEST_DIR should be set");
    
    // Go up to workspace root (from crates/rhizocrypt-service to workspace root)
    let workspace_root = std::path::Path::new(&manifest_dir)
        .parent()
        .and_then(|p| p.parent())
        .expect("Should have workspace root");
    
    let target_dir = std::env::var("CARGO_TARGET_DIR")
        .unwrap_or_else(|_| workspace_root.join("target").to_string_lossy().to_string());
    
    format!("{}/debug/rhizocrypt-service", target_dir)
}

#[tokio::test]
async fn test_service_binary_exists() {
    let binary_path = service_binary_path();
    
    eprintln!("Looking for binary at: {}", binary_path);
    eprintln!("Binary exists: {}", std::path::Path::new(&binary_path).exists());
    
    // Build if not exists
    if !std::path::Path::new(&binary_path).exists() {
        eprintln!("Binary not found, attempting to build...");
        build_service().expect("Failed to build service binary");
    }
    
    eprintln!("After build, exists: {}", std::path::Path::new(&binary_path).exists());
    
    assert!(
        std::path::Path::new(&binary_path).exists(),
        "Service binary should exist after build at: {}",
        binary_path
    );
}

#[tokio::test]
async fn test_service_starts_with_defaults() {
    // Build service first
    build_service().expect("Failed to build service");
    
    let binary_path = service_binary_path();
    
    // Start service in background with unique port
    let mut child = Command::new(&binary_path)
        .env("RHIZOCRYPT_PORT", "19400") // Use unique port for test
        .env("RUST_LOG", "error") // Reduce noise
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("Failed to start service");
    
    // Give it time to start
    sleep(Duration::from_millis(500)).await;
    
    // Check if process is running
    match child.try_wait() {
        Ok(Some(status)) => {
            panic!("Service exited unexpectedly with status: {}", status);
        }
        Ok(None) => {
            // Still running - good!
        }
        Err(e) => {
            panic!("Error checking service status: {}", e);
        }
    }
    
    // Clean up
    let _ = child.kill();
    let _ = child.wait();
}

#[tokio::test]
async fn test_service_handles_invalid_port() {
    build_service().expect("Failed to build service");
    
    let binary_path = service_binary_path();
    
    // Try to start with invalid port
    let output = Command::new(&binary_path)
        .env("RHIZOCRYPT_PORT", "99999") // Invalid port number
        .env("RUST_LOG", "error")
        .output()
        .expect("Failed to execute service");
    
    // Service should fail to start with invalid port
    assert!(
        !output.status.success(),
        "Service should fail with invalid port"
    );
}

#[tokio::test]
async fn test_service_custom_configuration() {
    build_service().expect("Failed to build service");
    
    let binary_path = service_binary_path();
    
    // Start with custom configuration
    let mut child = Command::new(&binary_path)
        .env("RHIZOCRYPT_PORT", "19401")
        .env("RHIZOCRYPT_HOST", "127.0.0.1")
        .env("RUST_LOG", "error")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("Failed to start service");
    
    sleep(Duration::from_millis(500)).await;
    
    // Should be running with custom config
    match child.try_wait() {
        Ok(Some(status)) => {
            panic!("Service exited unexpectedly: {}", status);
        }
        Ok(None) => {
            // Running correctly
        }
        Err(e) => {
            panic!("Error checking service: {}", e);
        }
    }
    
    let _ = child.kill();
    let _ = child.wait();
}

#[tokio::test]
async fn test_service_without_discovery() {
    build_service().expect("Failed to build service");
    
    let binary_path = service_binary_path();
    
    // Start without discovery service configured
    let mut child = Command::new(&binary_path)
        .env("RHIZOCRYPT_PORT", "19402")
        .env("RUST_LOG", "info")
        .env_remove("SONGBIRD_ADDRESS")
        .env_remove("RHIZOCRYPT_DISCOVERY_ADAPTER")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Failed to start service");
    
    sleep(Duration::from_millis(500)).await;
    
    // Should start successfully even without discovery
    match child.try_wait() {
        Ok(Some(status)) => {
            panic!("Service should run without discovery, exited: {}", status);
        }
        Ok(None) => {
            // Good - running in standalone mode
        }
        Err(e) => {
            panic!("Error: {}", e);
        }
    }
    
    let _ = child.kill();
    let _ = child.wait();
}

#[tokio::test]
async fn test_service_graceful_shutdown_sigterm() {
    build_service().expect("Failed to build service");
    
    let binary_path = service_binary_path();
    
    // Start service
    let mut child = Command::new(&binary_path)
        .env("RHIZOCRYPT_PORT", "19403")
        .env("RUST_LOG", "error")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("Failed to start service");
    
    sleep(Duration::from_millis(500)).await;
    
    // Send SIGTERM for graceful shutdown
    #[cfg(unix)]
    {
        use nix::sys::signal::{kill, Signal};
        use nix::unistd::Pid;
        
        let pid = Pid::from_raw(child.id() as i32);
        let _ = kill(pid, Signal::SIGTERM);
    }
    
    // On non-Unix, just kill
    #[cfg(not(unix))]
    {
        let _ = child.kill();
    }
    
    // Wait for shutdown (with timeout)
    let wait_result = tokio::time::timeout(
        Duration::from_secs(5),
        async {
            loop {
                match child.try_wait() {
                    Ok(Some(_)) => return Ok(()),
                    Ok(None) => sleep(Duration::from_millis(100)).await,
                    Err(e) => return Err(e),
                }
            }
        }
    ).await;
    
    assert!(
        wait_result.is_ok(),
        "Service should shutdown gracefully within 5 seconds"
    );
}

#[tokio::test]
async fn test_service_port_already_in_use() {
    build_service().expect("Failed to build service");
    
    let binary_path = service_binary_path();
    let test_port = "19404";
    
    // Start first service
    let mut child1 = Command::new(&binary_path)
        .env("RHIZOCRYPT_PORT", test_port)
        .env("RUST_LOG", "error")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("Failed to start first service");
    
    sleep(Duration::from_millis(500)).await;
    
    // Try to start second service on same port
    let output = Command::new(&binary_path)
        .env("RHIZOCRYPT_PORT", test_port)
        .env("RUST_LOG", "error")
        .output()
        .expect("Failed to execute second service");
    
    // Second service should fail (port in use)
    assert!(
        !output.status.success(),
        "Second service should fail when port is in use"
    );
    
    // Clean up
    let _ = child1.kill();
    let _ = child1.wait();
}

#[tokio::test]
async fn test_service_environment_variable_parsing() {
    build_service().expect("Failed to build service");
    
    let binary_path = service_binary_path();
    
    // Test with various environment configurations
    let mut child = Command::new(&binary_path)
        .env("RHIZOCRYPT_PORT", "19405")
        .env("RHIZOCRYPT_HOST", "0.0.0.0")
        .env("RHIZOCRYPT_ENV", "development")
        .env("RUST_LOG", "error")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("Failed to start service");
    
    sleep(Duration::from_millis(500)).await;
    
    // Should parse environment vars correctly
    match child.try_wait() {
        Ok(Some(status)) => {
            panic!("Service failed to parse env vars: {}", status);
        }
        Ok(None) => {
            // Good - running with custom env vars
        }
        Err(e) => {
            panic!("Error: {}", e);
        }
    }
    
    let _ = child.kill();
    let _ = child.wait();
}

#[tokio::test]
async fn test_service_multiple_instances_different_ports() {
    build_service().expect("Failed to build service");
    
    let binary_path = service_binary_path();
    
    // Start multiple instances on different ports
    let mut child1 = Command::new(&binary_path)
        .env("RHIZOCRYPT_PORT", "19406")
        .env("RUST_LOG", "error")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("Failed to start service 1");
    
    let mut child2 = Command::new(&binary_path)
        .env("RHIZOCRYPT_PORT", "19407")
        .env("RUST_LOG", "error")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("Failed to start service 2");
    
    let mut child3 = Command::new(&binary_path)
        .env("RHIZOCRYPT_PORT", "19408")
        .env("RUST_LOG", "error")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("Failed to start service 3");
    
    sleep(Duration::from_secs(1)).await;
    
    // All should be running
    assert!(child1.try_wait().unwrap().is_none(), "Service 1 should be running");
    assert!(child2.try_wait().unwrap().is_none(), "Service 2 should be running");
    assert!(child3.try_wait().unwrap().is_none(), "Service 3 should be running");
    
    // Clean up
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
    
    // Start service
    let mut child = Command::new(&binary_path)
        .env("RHIZOCRYPT_PORT", "19409")
        .env("RUST_LOG", "info")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("Failed to start service");
    
    sleep(Duration::from_millis(500)).await;
    
    let pid = Pid::from_raw(child.id() as i32);
    
    // Test SIGTERM (graceful shutdown)
    let result = kill(pid, Signal::SIGTERM);
    assert!(result.is_ok(), "Should be able to send SIGTERM");
    
    // Wait for graceful shutdown
    let shutdown_result = tokio::time::timeout(
        Duration::from_secs(5),
        async {
            loop {
                if let Ok(Some(_)) = child.try_wait() {
                    return true;
                }
                sleep(Duration::from_millis(100)).await;
            }
        }
    ).await;
    
    assert!(
        shutdown_result.is_ok(),
        "Service should shutdown gracefully on SIGTERM"
    );
}

#[tokio::test]
async fn test_service_with_discovery_fallback() {
    build_service().expect("Failed to build service");
    
    let binary_path = service_binary_path();
    
    // Start with invalid discovery address (should fallback to standalone)
    let mut child = Command::new(&binary_path)
        .env("RHIZOCRYPT_PORT", "19410")
        .env("SONGBIRD_ADDRESS", "invalid.nonexistent:9999")
        .env("RUST_LOG", "info")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Failed to start service");
    
    sleep(Duration::from_secs(1)).await;
    
    // Service should still run even if discovery fails
    match child.try_wait() {
        Ok(Some(status)) => {
            panic!("Service should continue running even if discovery fails: {}", status);
        }
        Ok(None) => {
            // Good - running in standalone mode after discovery failure
        }
        Err(e) => {
            panic!("Error: {}", e);
        }
    }
    
    let _ = child.kill();
    let _ = child.wait();
}

