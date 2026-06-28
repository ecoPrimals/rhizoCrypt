// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2024–2026 ecoPrimals Project

use super::{
    AnnounceResponseOutcome, build_announce_request, neural_api_socket_candidates,
    resolve_neural_api_socket,
};
use rhizo_crypt_core::constants;

#[test]
fn build_announce_request_has_jsonrpc_envelope() {
    let request = build_announce_request("/run/biomeos/rhizocrypt.sock", Some(42));

    assert_eq!(
        request.get("jsonrpc").and_then(serde_json::Value::as_str),
        Some(constants::JSONRPC_VERSION)
    );
    assert_eq!(request.get("method").and_then(serde_json::Value::as_str), Some("primal.announce"));
    assert_eq!(request.get("id"), Some(&serde_json::json!(1)));
}

#[test]
fn build_announce_request_embeds_params() {
    let request = build_announce_request("/tmp/rhizocrypt.sock", Some(99));
    let params = request.get("params").expect("params object");

    assert_eq!(
        params.get("socket").and_then(serde_json::Value::as_str),
        Some("/tmp/rhizocrypt.sock")
    );
    assert_eq!(params.get("pid"), Some(&serde_json::json!(99)));
    assert!(params.get("capabilities").is_some());
    assert!(params.get("methods").is_some());
    assert!(params.get("cost_hints").is_some());
    assert!(params.get("latency_estimates").is_some());
}

#[test]
fn neural_api_socket_candidates_xdg_before_tmp() {
    let candidates = neural_api_socket_candidates("test-family", Some("/run/user/1000"));

    assert_eq!(candidates.len(), 2);
    assert_eq!(
        candidates[0],
        std::path::PathBuf::from("/run/user/1000/biomeos/neural-api-test-family.sock")
    );
    assert_eq!(candidates[1], std::path::PathBuf::from("/tmp/biomeos/neural-api-test-family.sock"));
}

#[test]
fn neural_api_socket_candidates_tmp_only_without_xdg() {
    let candidates = neural_api_socket_candidates("solo", None);

    assert_eq!(candidates.len(), 1);
    assert_eq!(candidates[0], std::path::PathBuf::from("/tmp/biomeos/neural-api-solo.sock"));
}

#[test]
fn resolve_neural_api_socket_prefers_env_override() {
    let env_path = "/custom/neural.sock";
    let resolved =
        resolve_neural_api_socket(Some(env_path), "family", Some("/run/user/1000"), |path| {
            path == std::path::Path::new(env_path)
        });

    assert_eq!(resolved, Some(std::path::PathBuf::from(env_path)));
}

#[test]
fn resolve_neural_api_socket_skips_missing_env_override() {
    let resolved = resolve_neural_api_socket(
        Some("/missing/neural.sock"),
        "family",
        Some("/run/user/1000"),
        |_| false,
    );

    assert!(resolved.is_none());
}

#[test]
fn resolve_neural_api_socket_falls_back_to_xdg_tier() {
    let xdg_path = std::path::PathBuf::from("/run/user/1000/biomeos/neural-api-dev.sock");

    let resolved = resolve_neural_api_socket(None, "dev", Some("/run/user/1000"), |path| {
        path == xdg_path.as_path()
    });

    assert_eq!(resolved, Some(xdg_path));
}

#[test]
fn resolve_neural_api_socket_falls_back_to_tmp_tier() {
    let tmp_path = std::path::PathBuf::from("/tmp/biomeos/neural-api-dev.sock");

    let resolved = resolve_neural_api_socket(None, "dev", None, |path| path == tmp_path.as_path());

    assert_eq!(resolved, Some(tmp_path));
}

#[test]
fn resolve_neural_api_socket_prefers_xdg_over_tmp() {
    let xdg_path = std::path::PathBuf::from("/run/user/1000/biomeos/neural-api-dev.sock");
    let tmp_path = std::path::PathBuf::from("/tmp/biomeos/neural-api-dev.sock");

    let resolved = resolve_neural_api_socket(None, "dev", Some("/run/user/1000"), |path| {
        path == xdg_path.as_path() || path == tmp_path.as_path()
    });

    assert_eq!(resolved, Some(xdg_path));
}

#[test]
fn parse_announce_response_registered() {
    let resp = serde_json::json!({
        "jsonrpc": "2.0",
        "result": {
            "capabilities_registered": 3,
            "methods_registered": 12
        },
        "id": 1
    });

    assert_eq!(
        super::parse_announce_response(&resp),
        AnnounceResponseOutcome::Registered {
            capabilities: 3,
            methods: 12,
        }
    );
}

#[test]
fn parse_announce_response_rejected() {
    let resp = serde_json::json!({
        "jsonrpc": "2.0",
        "error": { "code": -32600, "message": "invalid announce" },
        "id": 1
    });

    match super::parse_announce_response(&resp) {
        AnnounceResponseOutcome::Rejected(err) => {
            assert_eq!(err.get("code"), Some(&serde_json::json!(-32600)));
        }
        other => panic!("expected Rejected, got {other:?}"),
    }
}

#[test]
fn parse_announce_response_no_result_defaults_counts() {
    let resp = serde_json::json!({
        "jsonrpc": "2.0",
        "result": {},
        "id": 1
    });

    assert_eq!(
        super::parse_announce_response(&resp),
        AnnounceResponseOutcome::Registered {
            capabilities: 0,
            methods: 0,
        }
    );
}

#[test]
fn parse_announce_response_empty_envelope() {
    assert_eq!(
        super::parse_announce_response(&serde_json::json!({})),
        AnnounceResponseOutcome::NoResult
    );
}

#[cfg(unix)]
mod announce_integration {
    use super::super::{
        AnnounceResponseOutcome, announce_to_biomeos, build_announce_request,
        parse_announce_response, send_jsonrpc_uds,
    };
    use tokio::io::{AsyncBufReadExt, AsyncWriteExt};
    use tokio::net::UnixListener;

    #[test]
    fn announce_to_biomeos_registered_response() {
        let dir = tempfile::tempdir().unwrap();
        let neural_sock = dir.path().join("neural-api-mock.sock");
        let rhizo_sock = dir.path().join("rhizocrypt.sock");

        let response = serde_json::json!({
            "jsonrpc": "2.0",
            "result": {
                "capabilities_registered": 5,
                "methods_registered": 20
            },
            "id": 1
        });

        temp_env::with_var("NEURAL_API_SOCKET", Some(neural_sock.to_str().unwrap()), || {
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async {
                let listener = UnixListener::bind(&neural_sock).unwrap();
                let mock = tokio::spawn(async move {
                    if let Ok((stream, _)) = listener.accept().await {
                        let (reader, mut writer) = tokio::io::split(stream);
                        let mut lines = tokio::io::BufReader::new(reader).lines();
                        let _ = lines.next_line().await;
                        let body = format!("{response}\n");
                        let _ = writer.write_all(body.as_bytes()).await;
                    }
                });
                announce_to_biomeos(&rhizo_sock).await;
                mock.abort();
            });
        });
    }

    #[test]
    fn announce_to_biomeos_rejected_response() {
        let dir = tempfile::tempdir().unwrap();
        let neural_sock = dir.path().join("neural-api-reject.sock");
        let rhizo_sock = dir.path().join("rhizocrypt.sock");

        let response = serde_json::json!({
            "jsonrpc": "2.0",
            "error": { "code": -32600, "message": "invalid announce" },
            "id": 1
        });

        temp_env::with_var("NEURAL_API_SOCKET", Some(neural_sock.to_str().unwrap()), || {
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async {
                let listener = UnixListener::bind(&neural_sock).unwrap();
                let mock = tokio::spawn(async move {
                    if let Ok((stream, _)) = listener.accept().await {
                        let (reader, mut writer) = tokio::io::split(stream);
                        let mut lines = tokio::io::BufReader::new(reader).lines();
                        let _ = lines.next_line().await;
                        let body = format!("{response}\n");
                        let _ = writer.write_all(body.as_bytes()).await;
                    }
                });
                announce_to_biomeos(&rhizo_sock).await;
                mock.abort();
            });
        });
    }

    #[test]
    fn announce_to_biomeos_socket_not_found_is_non_fatal() {
        let dir = tempfile::tempdir().unwrap();
        let rhizo_sock = dir.path().join("rhizocrypt.sock");

        temp_env::with_vars(
            [
                ("NEURAL_API_SOCKET", None::<&str>),
                ("XDG_RUNTIME_DIR", Some("/nonexistent/xdg/path")),
            ],
            || {
                let rt = tokio::runtime::Runtime::new().unwrap();
                rt.block_on(announce_to_biomeos(&rhizo_sock));
            },
        );
    }

    #[test]
    fn build_announce_request_without_pid() {
        let request = build_announce_request("/run/biomeos/rhizocrypt.sock", None);
        let params = request.get("params").expect("params object");
        assert!(params.get("pid").is_none() || params.get("pid").unwrap().is_null());
    }

    #[test]
    fn parse_announce_response_no_result_no_error() {
        let resp = serde_json::json!({ "jsonrpc": "2.0", "id": 1 });
        assert_eq!(parse_announce_response(&resp), AnnounceResponseOutcome::NoResult);
    }

    #[test]
    fn parse_announce_response_result_missing_fields_defaults_zero() {
        let resp = serde_json::json!({
            "jsonrpc": "2.0", "id": 1,
            "result": {}
        });
        assert_eq!(
            parse_announce_response(&resp),
            AnnounceResponseOutcome::Registered {
                capabilities: 0,
                methods: 0,
            }
        );
    }

    #[test]
    fn announce_to_biomeos_send_failure_is_non_fatal() {
        use tokio::net::UnixListener;

        let dir = tempfile::tempdir().unwrap();
        let rhizo_sock = dir.path().join("rhizocrypt.sock");
        let neural_sock = dir.path().join("neural-api.sock");

        temp_env::with_var("NEURAL_API_SOCKET", Some(neural_sock.to_str().unwrap()), || {
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async {
                let listener = UnixListener::bind(&neural_sock).unwrap();
                let mock = tokio::spawn(async move {
                    if let Ok((stream, _)) = listener.accept().await {
                        drop(stream);
                    }
                });
                announce_to_biomeos(&rhizo_sock).await;
                mock.abort();
            });
        });
    }

    #[test]
    fn send_jsonrpc_uds_connect_failure() {
        let rt = tokio::runtime::Runtime::new().unwrap();
        let result = rt.block_on(send_jsonrpc_uds(
            std::path::Path::new("/tmp/nonexistent_neural_api_rhizo_test.sock"),
            &serde_json::json!({"jsonrpc": "2.0", "method": "test", "id": 1}),
        ));
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("neural-api connect"), "got: {err}");
    }

    #[test]
    fn send_jsonrpc_uds_invalid_json_response() {
        use tokio::io::AsyncWriteExt;
        use tokio::net::UnixListener;

        let dir = tempfile::tempdir().unwrap();
        let neural_sock = dir.path().join("neural-bad-json.sock");

        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let listener = UnixListener::bind(&neural_sock).unwrap();
            let mock = tokio::spawn(async move {
                if let Ok((stream, _)) = listener.accept().await {
                    let (reader, mut writer) = tokio::io::split(stream);
                    let mut lines = tokio::io::BufReader::new(reader);
                    let mut line = String::new();
                    let _ = tokio::io::AsyncBufReadExt::read_line(&mut lines, &mut line).await;
                    let _ = writer.write_all(b"not-json\n").await;
                }
            });

            let result = send_jsonrpc_uds(
                &neural_sock,
                &serde_json::json!({"jsonrpc": "2.0", "method": "test", "id": 1}),
            )
            .await;
            assert!(result.is_err());
            let err = result.unwrap_err().to_string();
            assert!(err.contains("neural-api parse"), "got: {err}");
            mock.abort();
        });
    }
}
