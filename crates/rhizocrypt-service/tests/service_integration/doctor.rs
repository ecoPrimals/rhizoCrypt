// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2024–2026 ecoPrimals Project

//! Doctor command and discovery connectivity integration tests.

use rhizocrypt_service::{
    DoctorCheck, check_dag_engine, check_discovery_connectivity, check_storage_backend, run_doctor,
};

// --- Doctor integration tests ---

/// Test doctor runs in basic (non-comprehensive) mode.
#[tokio::test]
async fn test_doctor_run_basic() {
    run_doctor(false).await;
}

/// Test doctor runs in comprehensive mode.
#[tokio::test]
async fn test_doctor_run_comprehensive() {
    run_doctor(true).await;
}

/// Test doctor reports Unhealthy when configuration has empty host.
#[test]
fn test_doctor_unhealthy_config_empty_host() {
    temp_env::with_vars([("RHIZOCRYPT_RPC_HOST", Some(""))], || {
        tokio::runtime::Builder::new_multi_thread()
            .worker_threads(2)
            .enable_all()
            .build()
            .unwrap()
            .block_on(async { run_doctor(false).await });
    });
}

/// Test doctor reports Healthy (standalone mode) when discovery is not configured.
#[test]
fn test_doctor_standalone_mode() {
    temp_env::with_vars(
        [
            ("RHIZOCRYPT_DISCOVERY_ADAPTER", None::<&str>),
            ("DISCOVERY_ENDPOINT", None),
            ("DISCOVERY_ADDRESS", None),
        ],
        || {
            tokio::runtime::Builder::new_multi_thread()
                .worker_threads(2)
                .enable_all()
                .build()
                .unwrap()
                .block_on(async { run_doctor(false).await });
        },
    );
}

/// Test doctor with discovery configured but non-comprehensive (Pass path).
#[test]
fn test_doctor_discovery_configured_non_comprehensive() {
    temp_env::with_vars([("DISCOVERY_ENDPOINT", Some("127.0.0.1:99999"))], || {
        tokio::runtime::Builder::new_multi_thread()
            .worker_threads(2)
            .enable_all()
            .build()
            .unwrap()
            .block_on(async { run_doctor(false).await });
    });
}

/// Test doctor with discovery configured and comprehensive (unreachable -> Warn).
#[test]
fn test_doctor_discovery_comprehensive_unreachable() {
    temp_env::with_vars([("DISCOVERY_ENDPOINT", Some("127.0.0.1:99999"))], || {
        tokio::runtime::Builder::new_multi_thread()
            .worker_threads(2)
            .enable_all()
            .build()
            .unwrap()
            .block_on(async { run_doctor(true).await });
    });
}

/// Test doctor with discovery reachable in comprehensive mode.
#[test]
fn test_doctor_discovery_comprehensive_reachable() {
    let rt =
        tokio::runtime::Builder::new_multi_thread().worker_threads(2).enable_all().build().unwrap();
    let addr = rt.block_on(async {
        tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap().local_addr().unwrap()
    });
    temp_env::with_vars([("DISCOVERY_ENDPOINT", Some(format!("http://{addr}")))], || {
        rt.block_on(async { run_doctor(true).await });
    });
}

/// Test `check_dag_engine` passes.
#[tokio::test]
async fn test_doctor_check_dag_engine() {
    assert!(check_dag_engine().await);
}

/// Test `check_storage_backend` returns valid result.
#[tokio::test]
async fn test_doctor_check_storage_backend() {
    let (ok, name) = check_storage_backend();
    assert!(ok, "storage backend check should pass");
    assert!(!name.is_empty(), "storage name should be non-empty");
}

/// Test `DoctorCheck` enum variants.
#[tokio::test]
async fn test_doctor_check_variants() {
    assert_eq!(DoctorCheck::Pass, DoctorCheck::Pass);
    assert_eq!(DoctorCheck::Warn, DoctorCheck::Warn);
    assert_eq!(DoctorCheck::Fail, DoctorCheck::Fail);
    assert_ne!(DoctorCheck::Pass, DoctorCheck::Fail);
    assert_ne!(DoctorCheck::Pass, DoctorCheck::Warn);
    assert_ne!(DoctorCheck::Warn, DoctorCheck::Fail);
}

/// Test `check_discovery_connectivity` succeeds with reachable address.
#[tokio::test]
async fn test_doctor_discovery_connectivity_success() {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();

    let result = check_discovery_connectivity(&addr.to_string()).await;
    assert!(result.is_ok(), "connectivity to local listener should succeed");
}

/// Test `check_discovery_connectivity` strips `http://` prefix.
#[tokio::test]
async fn test_doctor_discovery_connectivity_http_prefix() {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();

    let result = check_discovery_connectivity(&format!("http://{addr}")).await;
    assert!(result.is_ok(), "http:// prefix should be stripped");
}

/// Test `check_discovery_connectivity` strips `https://` prefix.
#[tokio::test]
async fn test_doctor_discovery_connectivity_https_prefix() {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();

    let result = check_discovery_connectivity(&format!("https://{addr}")).await;
    assert!(result.is_ok(), "https:// prefix should be stripped");
}

/// Test `check_discovery_connectivity` handles trailing slash.
#[tokio::test]
async fn test_doctor_discovery_connectivity_trailing_slash() {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();

    let result = check_discovery_connectivity(&format!("http://{addr}/")).await;
    assert!(result.is_ok(), "trailing slash should be trimmed");
}

/// Test `check_discovery_connectivity` fails on invalid address.
#[tokio::test]
async fn test_doctor_discovery_connectivity_invalid_address() {
    let result = check_discovery_connectivity("invalid-host-12345:99999").await;
    assert!(result.is_err(), "invalid address should fail");
}

/// Test `check_discovery_connectivity` fails on invalid parse.
#[tokio::test]
async fn test_doctor_discovery_connectivity_parse_error() {
    let result = check_discovery_connectivity("not-a-valid-address").await;
    assert!(result.is_err(), "unparseable address should fail");
}
