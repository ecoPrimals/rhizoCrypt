// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2024–2026 ecoPrimals Project

//! tarpc integration tests for Songbird client (live-clients feature).
//!
//! These tests require the `live-clients` feature and use a mock tarpc
//! server for integration coverage. Songbird uses tarpc (TCP + bincode),
//! not HTTP, so WireMock cannot be used here.

#[cfg(feature = "live-clients")]
use super::{SongbirdClient, SongbirdConfig};
#[cfg(feature = "live-clients")]
use crate::clients::songbird_types::ClientState;

#[cfg(feature = "live-clients")]
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn tarpc_register_success() {
    use crate::clients::songbird_rpc::{MockSongbirdServer, SongbirdRpc};
    use futures_util::StreamExt;
    use tarpc::server::{self, Channel};
    use tarpc::tokio_serde::formats::Bincode;

    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();

    let server = MockSongbirdServer;

    let _accept_handle = tokio::spawn(async move {
        let (stream, _) = listener.accept().await.unwrap();
        let transport = tarpc::serde_transport::Transport::from((stream, Bincode::default()));
        let channel = server::BaseChannel::with_defaults(transport);
        channel
            .execute(server.serve())
            .for_each(|fut| async move {
                fut.await;
            })
            .await;
    });

    let config = SongbirdConfig::with_address(addr.to_string());
    let client = SongbirdClient::new(config);
    client.connect().await.unwrap();

    let result = client.register("127.0.0.1:9400").await.unwrap();
    assert!(result.success);
    assert!(result.service_id.is_some());
    assert_eq!(client.state().await, ClientState::Registered);
}

#[cfg(feature = "live-clients")]
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn tarpc_discover_signing() {
    use crate::clients::songbird_rpc::{MockSongbirdServer, SongbirdRpc};
    use futures_util::StreamExt;
    use tarpc::server::{self, Channel};
    use tarpc::tokio_serde::formats::Bincode;

    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();

    let server = MockSongbirdServer;

    let _accept_handle = tokio::spawn(async move {
        let (stream, _) = listener.accept().await.unwrap();
        let transport = tarpc::serde_transport::Transport::from((stream, Bincode::default()));
        let channel = server::BaseChannel::with_defaults(transport);
        channel
            .execute(server.serve())
            .for_each(|fut| async move {
                fut.await;
            })
            .await;
    });

    let config = SongbirdConfig::with_address(addr.to_string());
    let client = SongbirdClient::new(config);
    client.connect().await.unwrap();

    let services = client.discover("signing").await.unwrap();
    assert_eq!(services.len(), 1);
    assert_eq!(services[0].id, "mock-beardog-1");
    assert_eq!(services[0].endpoint, "127.0.0.1:9500");
}

#[cfg(feature = "live-clients")]
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn tarpc_discover_empty_for_unknown_capability() {
    use crate::clients::songbird_rpc::{MockSongbirdServer, SongbirdRpc};
    use futures_util::StreamExt;
    use tarpc::server::{self, Channel};
    use tarpc::tokio_serde::formats::Bincode;

    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();

    let server = MockSongbirdServer;

    let _accept_handle = tokio::spawn(async move {
        let (stream, _) = listener.accept().await.unwrap();
        let transport = tarpc::serde_transport::Transport::from((stream, Bincode::default()));
        let channel = server::BaseChannel::with_defaults(transport);
        channel
            .execute(server.serve())
            .for_each(|fut| async move {
                fut.await;
            })
            .await;
    });

    let config = SongbirdConfig::with_address(addr.to_string());
    let client = SongbirdClient::new(config);
    client.connect().await.unwrap();

    let services = client.discover("unknown-capability").await.unwrap();
    assert!(services.is_empty());
}

#[cfg(feature = "live-clients")]
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn tarpc_register_then_discover_signing_provider() {
    use crate::clients::songbird_rpc::{MockSongbirdServer, SongbirdRpc};
    use futures_util::StreamExt;
    use tarpc::server::{self, Channel};
    use tarpc::tokio_serde::formats::Bincode;

    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();

    let server = MockSongbirdServer;

    let _accept_handle = tokio::spawn(async move {
        let (stream, _) = listener.accept().await.unwrap();
        let transport = tarpc::serde_transport::Transport::from((stream, Bincode::default()));
        let channel = server::BaseChannel::with_defaults(transport);
        channel
            .execute(server.serve())
            .for_each(|fut| async move {
                fut.await;
            })
            .await;
    });

    let config = SongbirdConfig::with_address(addr.to_string());
    let client = SongbirdClient::new(config);
    client.connect().await.unwrap();
    client.register("127.0.0.1:9400").await.unwrap();

    let signer = client.discover_signing_provider().await.unwrap();
    assert!(signer.is_some());
    assert_eq!(signer.unwrap().endpoint, "127.0.0.1:9500");
}
