// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2024–2026 ecoPrimals Project

//! Server startup, transport binding, and lifecycle orchestration.

use rhizo_crypt_core::constants;
use rhizo_crypt_core::primal::PrimalLifecycle;
use rhizo_crypt_core::safe_env::SafeEnv;
use rhizo_crypt_core::{RhizoCrypt, RhizoCryptConfig};
use rhizo_crypt_rpc::jsonrpc::JsonRpcServer;
use rhizo_crypt_rpc::server::RpcServer;
use std::net::SocketAddr;
use std::sync::Arc;
use tracing::{debug, error, info, warn};

use crate::ServiceError;
use crate::config::{has_explicit_tcp_config, resolve_bind_addr};
use crate::discovery::{publish_capability_manifest, register_with_discovery};
use crate::shutdown::shutdown_signal;
#[cfg(unix)]
use crate::uds::start_uds_listener;

/// Start the RPC server with UDS unconditional (Unix) and TCP opt-in.
///
/// Transport model (Provenance Trio standard — LD-06):
/// - **UDS**: Always active on Unix when `unix_socket` is `Some`. Pass
///   `Some("")` for the default ecosystem path, `Some(path)` for a custom
///   path. `None` disables UDS (test backward-compat only).
/// - **TCP**: Opt-in. Starts tarpc + JSON-RPC TCP when `port_override` is
///   `Some`, or when `RHIZOCRYPT_PORT` / `RHIZOCRYPT_JSONRPC_PORT` env vars
///   are set.
///
/// # Errors
///
/// Returns [`ServiceError::AddrParse`] if the bind address is invalid.
/// Returns [`ServiceError::Config`] if the DAG engine fails to start.
/// Returns [`ServiceError::Rpc`] if the RPC server encounters a fatal I/O error.
pub async fn run_server(
    port_override: Option<u16>,
    host_override: Option<String>,
    unix_socket: Option<String>,
) -> Result<(), ServiceError> {
    run_server_with_ready(port_override, host_override, unix_socket, None).await
}

/// Run the server with an optional readiness notification.
///
/// When `ready` is `Some`, the notifier fires once the server is bound
/// and accepting connections. Used by integration tests to avoid sleep-based
/// readiness polling.
///
/// # Errors
///
/// Returns [`ServiceError`] on bind, config, or runtime failures.
pub async fn run_server_with_ready(
    port_override: Option<u16>,
    host_override: Option<String>,
    unix_socket: Option<String>,
    ready: Option<Arc<tokio::sync::Notify>>,
) -> Result<(), ServiceError> {
    let _ = tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .try_init();

    info!("Starting rhizoCrypt service...");

    rhizo_crypt_core::transport::btsp_env_guard(rhizo_crypt_core::niche::ENV_PREFIX)
        .map_err(|e| ServiceError::Config(e.to_string()))?;

    if rhizo_crypt_core::transport::is_biomeos_insecure() {
        warn!("BIOMEOS_INSECURE=1 — running in development mode (no BTSP handshake)");
    }

    if let Some(fid) =
        rhizo_crypt_core::transport::read_family_id(rhizo_crypt_core::niche::ENV_PREFIX)
    {
        info!(family_id = %fid, "BTSP Phase 1: family-scoped socket naming active");
    }

    if let Some(transport) = rhizo_crypt_core::SafeEnv::transport_endpoint() {
        info!(endpoint = %transport, "TRANSPORT_ENDPOINT injected by launcher");
    }

    let config = RhizoCryptConfig::default();
    let mut primal = RhizoCrypt::new(config);
    primal.start().await.map_err(|e| ServiceError::Config(e.to_string()))?;
    let primal = Arc::new(primal);
    info!("DAG engine initialized and running");

    // Background mesh event poller — polls bearDog auth.events.poll for
    // cross-gate trust events and appends them to a mesh-trust DAG session.
    // Non-fatal: runs silently if no signing provider is available.
    let _mesh_poller = primal.spawn_mesh_poller();

    // UDS is unconditional on Unix (Provenance Trio standard — LD-06).
    // None = no UDS (test backward-compat), Some("") = default, Some(path) = custom.
    #[cfg(unix)]
    let (uds_shutdown_tx, uds_socket_path) = start_uds_listener(unix_socket.as_deref(), &primal);
    #[cfg(not(unix))]
    let _ = unix_socket;

    let tcp_requested =
        port_override.is_some() || host_override.is_some() || has_explicit_tcp_config();

    // Resolve TCP address once — used for both manifest and serve_with_tcp.
    let tcp_addr = if tcp_requested {
        Some(resolve_bind_addr(port_override, host_override)?)
    } else {
        None
    };

    // Publish manifest so springs can discover us via capability-based lookup
    // (PG-32: discover_by_capability("dag") must find our socket).
    #[cfg(unix)]
    if let Some(ref socket_path) = uds_socket_path {
        publish_capability_manifest(socket_path, tcp_addr).await;

        // Neural API announce runs in background — non-blocking so the service
        // is ready for health probes immediately. Announce is best-effort and
        // non-fatal; no reason to delay readiness by up to 7s of UDS timeouts.
        let announce_path = socket_path.clone();
        tokio::spawn(async move {
            crate::neural_api::announce_to_biomeos(&announce_path).await;
        });
    }

    let result = if let Some(addr) = tcp_addr {
        serve_with_tcp(
            addr,
            &primal,
            ready,
            #[cfg(unix)]
            uds_shutdown_tx,
        )
        .await
    } else {
        info!("TCP disabled (UDS-only mode — pass --port to enable)");

        #[cfg(unix)]
        info!("rhizoCrypt service ready (UDS-only)");
        #[cfg(not(unix))]
        warn!("No transport active — UDS unavailable on this platform, pass --port for TCP");

        if let Some(notify) = ready {
            tokio::spawn(async move {
                tokio::time::sleep(constants::READINESS_NOTIFY_DELAY).await;
                notify.notify_one();
            });
        }

        shutdown_signal().await;
        info!("Received shutdown signal, stopping gracefully");
        #[cfg(unix)]
        {
            if uds_shutdown_tx.send(true).is_err() {
                debug!("UDS shutdown channel already closed (UDS-only mode)");
            }
        }
        info!("rhizoCrypt service shutdown cleanly");
        Ok(())
    };

    // Unpublish manifest on shutdown (best-effort cleanup).
    #[cfg(unix)]
    {
        rhizo_crypt_core::discovery::unpublish_manifest(rhizo_crypt_core::niche::PRIMAL_ID).await;
        debug!("Capability manifest unpublished");
    }

    result
}

/// Serve with TCP (tarpc + JSON-RPC) alongside UDS.
///
/// Takes a pre-resolved `SocketAddr` (computed once in `run_server_with_ready`
/// and shared with the capability manifest).
async fn serve_with_tcp(
    addr: SocketAddr,
    primal: &Arc<RhizoCrypt>,
    ready: Option<Arc<tokio::sync::Notify>>,
    #[cfg(unix)] uds_shutdown_tx: tokio::sync::watch::Sender<bool>,
) -> Result<(), ServiceError> {
    info!(address = %addr, "Binding TCP servers (opt-in)");

    let server = RpcServer::new(Arc::clone(primal), addr);

    let port = addr.port();
    let host = addr.ip();
    let jsonrpc_port = SafeEnv::get_jsonrpc_port(port);
    let jsonrpc_addr: SocketAddr = format!("{host}:{jsonrpc_port}").parse()?;
    let (jsonrpc_shutdown_tx, jsonrpc_shutdown_rx) = tokio::sync::watch::channel(false);
    let jsonrpc_server = JsonRpcServer::new(Arc::clone(primal), jsonrpc_addr);
    let jsonrpc_handle = tokio::spawn(async move {
        if let Err(e) = jsonrpc_server.serve(jsonrpc_shutdown_rx).await {
            error!(error = %e, "JSON-RPC server error");
        }
    });
    info!(address = %jsonrpc_addr, "JSON-RPC TCP started (dual-mode: HTTP + newline)");

    if let Some(discovery_addr) = SafeEnv::get_discovery_address() {
        info!(discovery = %discovery_addr, "Registering with discovery service");
        match register_with_discovery(&discovery_addr, addr).await {
            Ok(client) => {
                info!("Registered with discovery service");

                // Eagerly populate the engine's registry with peer endpoints so
                // capability clients (signing, permanent storage, provenance)
                // resolve without waiting for the first lazy query.
                if let Err(e) = client.populate_registry(primal.discovery_registry()).await {
                    warn!(error = %e, "Eager peer population failed (lazy fallback active)");
                } else {
                    info!("Discovery registry populated with peer endpoints");
                }
            }
            Err(e) => warn!(error = %e, "Discovery registration failed, continuing standalone"),
        }

        // Bootstrap the lazy-resolution fallback so capability clients can
        // query Songbird on demand for peers not yet in the registry.
        if let Some(ep) =
            rhizo_crypt_core::transport::TransportEndpoint::try_parse_address(&discovery_addr)
        {
            primal.discovery_registry().set_discovery_source(ep).await;
            info!("Discovery source bootstrapped for peer capability resolution");
        }
    } else {
        info!("No discovery service configured (standalone mode)");
    }

    info!("rhizoCrypt service ready (UDS + TCP)");

    let shutdown_tx = server.shutdown_sender();
    let server_ready = server.ready_notifier();
    let serve_handle = tokio::spawn(async move { server.serve().await });

    if let Some(notify) = ready {
        tokio::spawn(async move {
            server_ready.notified().await;
            notify.notify_one();
        });
    }

    tokio::pin!(serve_handle);

    tokio::select! {
        result = &mut serve_handle => {
            match result {
                Ok(Ok(())) => {
                    info!("rhizoCrypt service shutdown cleanly");
                    Ok(())
                }
                Ok(Err(e)) => {
                    error!(error = %e, "rhizoCrypt service error");
                    Err(ServiceError::Rpc(e))
                }
                Err(e) => {
                    error!(error = %e, "server task panicked");
                    Err(ServiceError::Config(format!("server task panicked: {e}")))
                }
            }
        }
        () = shutdown_signal() => {
            info!("Received shutdown signal, stopping gracefully");
            if shutdown_tx.send(true).is_err() {
                debug!("tarpc shutdown channel already closed");
            }
            if jsonrpc_shutdown_tx.send(true).is_err() {
                debug!("JSON-RPC shutdown channel already closed");
            }
            #[cfg(unix)]
            {
                if uds_shutdown_tx.send(true).is_err() {
                    debug!("UDS shutdown channel already closed");
                }
            }
            if let Ok(Err(e)) = serve_handle.await {
                error!(error = %e, "rhizoCrypt service error during shutdown");
                return Err(ServiceError::Rpc(e));
            }
            if let Err(e) = jsonrpc_handle.await {
                debug!(error = %e, "JSON-RPC task join error during shutdown");
            }
            info!("rhizoCrypt service shutdown cleanly");
            Ok(())
        }
    }
}
