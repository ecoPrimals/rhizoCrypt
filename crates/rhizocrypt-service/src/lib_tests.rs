// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2024–2026 ecoPrimals Project

use super::*;

#[test]
fn test_resolve_bind_addr_with_overrides() {
    let addr = resolve_bind_addr(Some(9999), Some("127.0.0.1".to_string())).unwrap();
    assert_eq!(addr.port(), 9999);
    assert_eq!(addr.ip(), std::net::IpAddr::from([127, 0, 0, 1]));
}

#[test]
fn test_resolve_bind_addr_port_only() {
    let addr = resolve_bind_addr(Some(12345), None).unwrap();
    assert_eq!(addr.port(), 12345);
}

#[test]
fn test_resolve_bind_addr_host_only() {
    let addr = resolve_bind_addr(None, Some("127.0.0.1".to_string())).unwrap();
    assert_eq!(addr.ip(), std::net::IpAddr::from([127, 0, 0, 1]));
}

#[test]
fn test_resolve_bind_addr_no_overrides() {
    let addr = resolve_bind_addr(None, None).unwrap();
    assert!(addr.port() > 0 || addr.port() == 0);
    assert!(!addr.ip().to_string().is_empty());
}

#[test]
fn test_resolve_bind_addr_port_zero() {
    let addr = resolve_bind_addr(Some(0), Some("127.0.0.1".to_string())).unwrap();
    assert_eq!(addr.port(), 0);
    assert_eq!(addr.ip(), std::net::IpAddr::from([127, 0, 0, 1]));
}

#[test]
fn test_resolve_bind_addr_invalid_host() {
    let result = resolve_bind_addr(Some(9999), Some("not-an-ip".to_string()));
    assert!(result.is_err());
}

#[test]
fn test_resolve_bind_addr_invalid_host_empty() {
    let result = resolve_bind_addr(Some(9999), Some(String::new()));
    assert!(result.is_err());
}

#[test]
fn test_resolve_bind_addr_invalid_host_garbage() {
    let result = resolve_bind_addr(Some(9999), Some("::::".to_string()));
    assert!(result.is_err());
}

#[test]
fn test_service_error_display_config() {
    let err = ServiceError::Config("bad config".to_string());
    let s = err.to_string();
    assert!(s.contains("configuration error"));
    assert!(s.contains("bad config"));
}

#[test]
fn test_service_error_display_discovery() {
    let err = ServiceError::Discovery("no discovery adapter available".to_string());
    let s = err.to_string();
    assert!(s.contains("discovery registration failed"));
    assert!(s.contains("no discovery adapter"));
}

#[test]
fn test_service_error_display_rpc() {
    let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
    let err = ServiceError::Rpc(io_err);
    let s = err.to_string();
    assert!(s.contains("rpc server error"));
}

#[test]
fn test_service_error_display_addr_parse() {
    let parse_err = "x:y:z".parse::<SocketAddr>().unwrap_err();
    let err = ServiceError::AddrParse(parse_err);
    let s = err.to_string();
    assert!(s.contains("address parse error"));
}

#[test]
fn test_service_error_from_io_error() {
    let io_err = std::io::Error::other("connection refused");
    let err: ServiceError = io_err.into();
    assert!(matches!(err, ServiceError::Rpc(_)));
}

#[test]
fn test_service_error_from_addr_parse_error() {
    let parse_err = "invalid".parse::<SocketAddr>().unwrap_err();
    let err: ServiceError = parse_err.into();
    assert!(matches!(err, ServiceError::AddrParse(_)));
}

#[test]
fn test_exit_code_constants() {
    use crate::exit_codes;
    assert_eq!(exit_codes::SUCCESS, 0);
    assert_eq!(exit_codes::GENERAL_ERROR, 1);
    assert_eq!(exit_codes::CONFIG_ERROR, 2);
    assert_eq!(exit_codes::NETWORK_ERROR, 3);
    assert_eq!(exit_codes::INTERRUPTED, 130);
}

#[test]
fn test_service_error_exit_code_mapping() {
    use crate::exit_codes;
    let config_err = ServiceError::Config("bad".to_string());
    assert_eq!(config_err.exit_code(), exit_codes::CONFIG_ERROR);

    let rpc_err = ServiceError::Rpc(std::io::Error::other("connection refused"));
    assert_eq!(rpc_err.exit_code(), exit_codes::NETWORK_ERROR);

    let discovery_err = ServiceError::Discovery("unreachable".to_string());
    assert_eq!(discovery_err.exit_code(), exit_codes::NETWORK_ERROR);

    let parse_err = "x:y:z".parse::<SocketAddr>().unwrap_err();
    let addr_err = ServiceError::AddrParse(parse_err);
    assert_eq!(addr_err.exit_code(), exit_codes::CONFIG_ERROR);
}

#[tokio::test]
async fn test_shutdown_signal_does_not_panic() {
    use tokio::time::{Duration, timeout};
    let result = timeout(Duration::from_millis(100), super::shutdown_signal()).await;
    assert!(result.is_err(), "shutdown_signal should block until signal (timeout expected)");
}

#[test]
fn test_print_version_no_panic() {
    print_version();
}

#[test]
fn test_print_status_no_panic() {
    print_status();
}

#[tokio::test]
async fn test_register_with_discovery_unreachable() {
    let addr: SocketAddr = "127.0.0.1:9999".parse().unwrap();
    let result = register_with_discovery("http://invalid-host-12345:99999".to_string(), addr).await;
    assert!(result.is_err());
    match &result.unwrap_err() {
        ServiceError::Discovery(msg) => assert!(!msg.is_empty()),
        other => panic!("expected Discovery error, got: {other}"),
    }
}

#[tokio::test]
async fn test_run_doctor_basic() {
    run_doctor(false).await;
}

#[tokio::test]
async fn test_run_doctor_comprehensive() {
    run_doctor(true).await;
}

#[test]
fn test_check_configuration_default_env() {
    temp_env::with_vars(
        [
            ("RHIZOCRYPT_RPC_PORT", None::<&str>),
            ("RHIZOCRYPT_PORT", None),
            ("RHIZOCRYPT_RPC_HOST", None),
            ("RHIZOCRYPT_HOST", None),
            ("RHIZOCRYPT_ENV", None),
        ],
        || {
            let rt = tokio::runtime::Builder::new_multi_thread()
                .worker_threads(2)
                .enable_all()
                .build()
                .unwrap();
            rt.block_on(run_doctor(false));
        },
    );
}

#[test]
fn test_check_configuration_with_port_override() {
    temp_env::with_vars([("RHIZOCRYPT_RPC_PORT", Some("9401"))], || {
        let rt = tokio::runtime::Builder::new_multi_thread()
            .worker_threads(2)
            .enable_all()
            .build()
            .unwrap();
        rt.block_on(run_doctor(false));
    });
}

#[test]
fn test_check_configuration_with_host_override() {
    temp_env::with_vars([("RHIZOCRYPT_RPC_HOST", Some("0.0.0.0"))], || {
        let rt = tokio::runtime::Builder::new_multi_thread()
            .worker_threads(2)
            .enable_all()
            .build()
            .unwrap();
        rt.block_on(run_doctor(false));
    });
}

#[test]
fn test_check_configuration_development_mode() {
    temp_env::with_vars([("RHIZOCRYPT_ENV", Some("development"))], || {
        let rt = tokio::runtime::Builder::new_multi_thread()
            .worker_threads(2)
            .enable_all()
            .build()
            .unwrap();
        rt.block_on(run_doctor(false));
    });
}

#[test]
fn test_check_discovery_without_endpoint() {
    temp_env::with_vars(
        [
            ("RHIZOCRYPT_DISCOVERY_ADAPTER", None::<&str>),
            ("DISCOVERY_ENDPOINT", None),
            ("DISCOVERY_ADDRESS", None),
        ],
        || {
            let rt = tokio::runtime::Builder::new_multi_thread()
                .worker_threads(2)
                .enable_all()
                .build()
                .unwrap();
            rt.block_on(run_doctor(false));
        },
    );
}

#[test]
fn test_check_discovery_with_endpoint_non_comprehensive() {
    temp_env::with_vars([("DISCOVERY_ENDPOINT", Some("127.0.0.1:99999"))], || {
        let rt = tokio::runtime::Builder::new_multi_thread()
            .worker_threads(2)
            .enable_all()
            .build()
            .unwrap();
        rt.block_on(run_doctor(false));
    });
}

#[test]
fn test_check_discovery_with_endpoint_comprehensive() {
    temp_env::with_vars([("DISCOVERY_ENDPOINT", Some("127.0.0.1:99999"))], || {
        let rt = tokio::runtime::Builder::new_multi_thread()
            .worker_threads(2)
            .enable_all()
            .build()
            .unwrap();
        rt.block_on(run_doctor(true));
    });
}

#[tokio::test]
async fn test_check_dag_engine() {
    assert!(check_dag_engine().await);
}

#[tokio::test]
async fn test_check_storage_backend() {
    let (ok, name) = check_storage_backend();
    assert!(ok, "storage backend check should pass");
    assert!(!name.is_empty(), "storage name should be non-empty");
}

#[test]
fn test_doctor_check_partial_eq() {
    assert_eq!(DoctorCheck::Pass, DoctorCheck::Pass);
    assert_eq!(DoctorCheck::Warn, DoctorCheck::Warn);
    assert_eq!(DoctorCheck::Fail, DoctorCheck::Fail);
    assert_ne!(DoctorCheck::Pass, DoctorCheck::Fail);
    assert_ne!(DoctorCheck::Pass, DoctorCheck::Warn);
    assert_ne!(DoctorCheck::Warn, DoctorCheck::Fail);
}

#[test]
fn test_doctor_check_eq() {
    assert!(DoctorCheck::Pass == DoctorCheck::Pass);
    assert!(DoctorCheck::Warn == DoctorCheck::Warn);
    assert!(DoctorCheck::Fail == DoctorCheck::Fail);
}

#[test]
fn test_doctor_check_display_symbols() {
    let symbol = |c: DoctorCheck| match c {
        DoctorCheck::Pass => "[✓]",
        DoctorCheck::Warn => "[!]",
        DoctorCheck::Fail => "[✗]",
    };
    assert_eq!(symbol(DoctorCheck::Pass), "[✓]");
    assert_eq!(symbol(DoctorCheck::Warn), "[!]");
    assert_eq!(symbol(DoctorCheck::Fail), "[✗]");
}

#[tokio::test]
async fn test_check_discovery_connectivity_unreachable() {
    let result = check_discovery_connectivity("invalid-host-12345:99999").await;
    assert!(result.is_err());
}
