// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2024–2026 ecoPrimals Project

use std::fmt::Write;

use super::{ERROR_TYPE_COUNT, ErrorType, MetricsCollector, RPC_METHOD_COUNT, RpcMethod};

const ALL_METHODS: [RpcMethod; RPC_METHOD_COUNT] = [
    RpcMethod::CreateSession,
    RpcMethod::GetSession,
    RpcMethod::ListSessions,
    RpcMethod::DiscardSession,
    RpcMethod::AppendEvent,
    RpcMethod::AppendBatch,
    RpcMethod::GetVertex,
    RpcMethod::GetFrontier,
    RpcMethod::GetGenesis,
    RpcMethod::QueryVertices,
    RpcMethod::GetChildren,
    RpcMethod::GetMerkleRoot,
    RpcMethod::GetMerkleProof,
    RpcMethod::VerifyProof,
    RpcMethod::CheckoutSlice,
    RpcMethod::GetSlice,
    RpcMethod::ListSlices,
    RpcMethod::ResolveSlice,
    RpcMethod::Dehydrate,
    RpcMethod::GetDehydrationStatus,
    RpcMethod::Health,
    RpcMethod::Metrics,
    RpcMethod::ListCapabilities,
];

const ALL_ERROR_TYPES: [ErrorType; ERROR_TYPE_COUNT] = [
    ErrorType::SessionNotFound,
    ErrorType::VertexNotFound,
    ErrorType::InvalidInput,
    ErrorType::Internal,
    ErrorType::RateLimited,
    ErrorType::Timeout,
];

// All `let _ = writeln!()` below target `String`, whose `fmt::Write` impl is
// infallible. The discarded `Result` cannot be `Err` in practice.

#[expect(
    clippy::redundant_pub_crate,
    reason = "crate-private helper; pub(crate) matches metrics API contract"
)]
pub(crate) fn export_prometheus(collector: &MetricsCollector) -> String {
    let mut output = String::with_capacity(4096);

    output.push_str("# HELP rhizocrypt_uptime_seconds Time since service start\n");
    output.push_str("# TYPE rhizocrypt_uptime_seconds gauge\n");
    let _ = writeln!(output, "rhizocrypt_uptime_seconds {:.3}\n", collector.uptime_seconds());

    output.push_str("# HELP rhizocrypt_sessions_active Currently active sessions\n");
    output.push_str("# TYPE rhizocrypt_sessions_active gauge\n");
    let _ = writeln!(output, "rhizocrypt_sessions_active {}\n", collector.active_sessions());

    output.push_str("# HELP rhizocrypt_vertices_total Total vertices created\n");
    output.push_str("# TYPE rhizocrypt_vertices_total counter\n");
    let _ = writeln!(output, "rhizocrypt_vertices_total {}\n", collector.vertices_total());

    output.push_str("# HELP rhizocrypt_rpc_requests_total Total RPC requests\n");
    output.push_str("# TYPE rhizocrypt_rpc_requests_total counter\n");
    for method in ALL_METHODS {
        let count = collector.request_count(method);
        if count > 0 {
            let _ = writeln!(
                output,
                "rhizocrypt_rpc_requests_total{{method=\"{}\"}} {}",
                method.as_str(),
                count
            );
        }
    }
    output.push('\n');

    output.push_str("# HELP rhizocrypt_rpc_errors_total Total RPC errors\n");
    output.push_str("# TYPE rhizocrypt_rpc_errors_total counter\n");
    for error_type in ALL_ERROR_TYPES {
        let count = collector.error_count(error_type);
        if count > 0 {
            let _ = writeln!(
                output,
                "rhizocrypt_rpc_errors_total{{type=\"{}\"}} {}",
                error_type.as_str(),
                count
            );
        }
    }
    output.push('\n');

    output.push_str("# HELP rhizocrypt_rpc_request_duration_seconds_mean Mean request duration\n");
    output.push_str("# TYPE rhizocrypt_rpc_request_duration_seconds_mean gauge\n");
    for method in ALL_METHODS {
        let hist = collector.latency_histogram(method);
        if hist.count > 0 {
            let _ = writeln!(
                output,
                "rhizocrypt_rpc_request_duration_seconds_mean{{method=\"{}\"}} {:.6}",
                method.as_str(),
                hist.mean_seconds()
            );
        }
    }

    output
}
